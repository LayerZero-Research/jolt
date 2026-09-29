//! Every base catalog row through Jolt's Akita adapter (liveness).
//!
//! Rows come from the loaded artifacts, so the case list tracks the catalogs
//! without restating their grids. Every row is a supported statement, so any
//! failure (panic, clean error, or verifier rejection) of an honest opening is
//! a finding. Rows use Jolt's own setups:
//!
//! - one-hot, one polynomial: the production `OneHotTrace` group, streamed
//!   through `commit_trace_one_hot` at an input-chosen selector capacity and
//!   checked against the materialized one-hot polynomial;
//! - one-hot, two polynomials, and dense-bounded rows: `AkitaNativeBatching`'s
//!   same-point batch over one commitment group.
//!
//! The lane variant (`JOLT_FUZZ_GRID_FAMILY` = `k16`, `k256`, or `dense`)
//! fixes the family; without it the input picks. Rows above
//! `JOLT_FUZZ_MAX_CASE_COEFFS` are excluded; `jolt-fuzz-dev grid-sweep` runs
//! them in release mode.

use std::sync::{Arc, OnceLock};

use jolt_akita::{
    AkitaField, AkitaNativeBatchPolynomials, AkitaNativeBatching, AkitaScheduleArtifacts,
    AkitaScheme, AkitaSetupParams, AKITA_ONE_HOT_K16, AKITA_ONE_HOT_K256,
};
use jolt_claims::protocols::jolt::lattice::OneHotTraceSetupShape;
use jolt_field::{CanonicalEncoding, Ring};
use jolt_openings::{BatchOpeningScheme, CommitmentScheme, EvaluationClaim, VerifierOpeningClaim};
use jolt_poly::{MultilinearPoly, OneHotPolynomial, Polynomial};
use jolt_transcript::{Blake2bTranscript, Transcript};

use crate::input::{Reader, SplitMix64};
use crate::opening::{self, Witness};
use crate::shape::{Failure, SetupRequest, Stage};
use crate::{artifacts, env, liveness, stats, transport};

pub const FAMILY_ENV: &str = "JOLT_FUZZ_GRID_FAMILY";

/// Counter for identically zero committed polynomials the harness replaces
/// with one nonzero entry: openings of the zero polynomial fail verification
/// on the pinned Akita revision (FINDINGS J-5, fixed upstream by the proof
/// stream the Akita bump brings), so they are excluded until that bump.
pub const KNOWN_ZERO_POLYNOMIAL: &str = "known_zero_polynomial_adjusted";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    OneHot(usize),
    Dense,
}

impl Family {
    pub fn name(self) -> &'static str {
        match self {
            Family::OneHot(AKITA_ONE_HOT_K16) => "k16",
            Family::OneHot(_) => "k256",
            Family::Dense => "dense",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "k16" => Some(Family::OneHot(AKITA_ONE_HOT_K16)),
            "k256" => Some(Family::OneHot(AKITA_ONE_HOT_K256)),
            "dense" => Some(Family::Dense),
            _ => None,
        }
    }
}

pub const FAMILIES: [Family; 3] = [
    Family::OneHot(AKITA_ONE_HOT_K16),
    Family::OneHot(AKITA_ONE_HOT_K256),
    Family::Dense,
];

/// One scalar catalog row: the final group layout plus whether its schedule
/// offloads the setup to a recursive prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub family: Family,
    pub num_vars: usize,
    pub num_polys: usize,
    pub offloaded: bool,
}

impl Row {
    pub fn cost(&self) -> u128 {
        (1u128 << self.num_vars) * self.num_polys as u128
    }

    pub fn label(&self) -> String {
        format!(
            "{} {}:{}{}",
            self.family.name(),
            self.num_vars,
            self.num_polys,
            if self.offloaded { " offloaded" } else { "" }
        )
    }
}

/// Every row of the three shipped catalogs, sorted by family and shape.
pub fn rows(artifacts: &AkitaScheduleArtifacts) -> Vec<Row> {
    let mut rows = Vec::new();
    for family in FAMILIES {
        let catalog = match family {
            Family::OneHot(k) => artifacts.one_hot_catalog(k),
            Family::Dense => artifacts.dense_catalog(),
        }
        .unwrap_or_else(|error| panic!("load {} catalog: {error}", family.name()));
        for row in catalog.rows() {
            let group = row.profiles().final_group.group;
            rows.push(Row {
                family,
                num_vars: group.num_vars(),
                num_polys: group.num_polynomials(),
                offloaded: row
                    .schedule()
                    .recursive_folds
                    .iter()
                    .any(|fold| fold.params.setup_prefix().is_some()),
            });
        }
    }
    rows.sort_by_key(|row| (row.family.name(), row.num_polys, row.num_vars));
    rows
}

fn cached_rows() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| rows(&artifacts::shared()))
}

fn lane_family() -> Option<Family> {
    static FAMILY: OnceLock<Option<Family>> = OnceLock::new();
    *FAMILY.get_or_init(|| {
        std::env::var(FAMILY_ENV).ok().map(|name| {
            Family::from_name(&name).unwrap_or_else(|| panic!("unknown {FAMILY_ENV} {name}"))
        })
    })
}

pub fn run(data: &[u8]) {
    env::init();
    let mut reader = Reader::new(data);
    let selector = reader.u16();
    let family = lane_family().unwrap_or(FAMILIES[usize::from(selector >> 8) % FAMILIES.len()]);
    let limit = env::max_case_coefficients();
    let candidates: Vec<Row> = cached_rows()
        .iter()
        .copied()
        .filter(|row| row.family == family && row.cost() <= limit)
        .collect();
    let Some(&row) = candidates.get(usize::from(selector) % candidates.len().max(1)) else {
        return;
    };
    let log_capacity = reader.u8();
    let digest = reader.bytes::<32>();
    let witness = Witness::decode(&mut reader);
    check(row, log_capacity, digest, &witness);
}

/// Prove and verify one honest opening of `row`; panics on any failure.
pub fn check(row: Row, log_capacity: u8, digest: [u8; 32], witness: &Witness) {
    let artifacts = artifacts::shared();
    let context = row.label();
    let result = env::on_large_stack(|| match (row.family, row.num_polys) {
        (Family::OneHot(k), 1) => trace_row(&artifacts, row, k, log_capacity, digest, witness),
        _ => group_row(&artifacts, row, digest, witness),
    });
    match result {
        Ok(()) => stats::count("rows_verified"),
        Err(failure) => panic!(
            "liveness: catalog row {context} failed at {failure} (log_capacity byte {log_capacity}, witness {witness:?})"
        ),
    }
}

fn trace_row(
    artifacts: &Arc<AkitaScheduleArtifacts>,
    row: Row,
    one_hot_k: usize,
    log_capacity: u8,
    digest: [u8; 32],
    witness: &Witness,
) -> Result<(), Failure> {
    let log_k = one_hot_k.trailing_zeros() as usize;
    // Keep at least 2^10 coefficients per selector segment so every segment
    // stays ring-aligned; production segments hold `K * T >= 2^16`.
    let max_log_capacity = row.num_vars.saturating_sub(log_k.max(10)).min(6);
    let capacity = 1usize << (usize::from(log_capacity) % (max_log_capacity + 1));
    let request = SetupRequest {
        setup_shape: OneHotTraceSetupShape {
            num_vars: row.num_vars,
            num_polys: 1,
        },
        layout_digest: digest,
        one_hot_k,
        untrusted: None,
        trusted: None,
        program: Vec::new(),
    };
    opening::commit_with_capacity(artifacts, &request, witness, capacity)?
        .prove_and_verify(&row.label())
}

fn fail(stage: Stage) -> impl Fn(jolt_openings::OpeningsError) -> Failure {
    move |error| Failure {
        stage: stage.clone(),
        message: error.to_string(),
    }
}

fn group_row(
    artifacts: &Arc<AkitaScheduleArtifacts>,
    row: Row,
    digest: [u8; 32],
    witness: &Witness,
) -> Result<(), Failure> {
    let params = match row.family {
        Family::OneHot(k) => AkitaSetupParams::one_hot_only(
            row.num_vars,
            row.num_polys,
            digest,
            k,
            Arc::clone(artifacts),
        ),
        Family::Dense => {
            AkitaSetupParams::dense_only(row.num_vars, row.num_polys, digest, Arc::clone(artifacts))
        }
    };
    let (prover_setup, verifier_setup) =
        stats::time("setup", || AkitaScheme::setup(params)).map_err(fail(Stage::Setup))?;
    let mut rng = SplitMix64::new(witness.seed);
    let point: Vec<AkitaField> = (0..row.num_vars)
        .map(|_| AkitaField::from_u128_reduced(rng.next_u128()))
        .collect();

    enum Group {
        OneHot(Vec<OneHotPolynomial>),
        Dense(Vec<Polynomial<AkitaField>>),
    }
    let group = match row.family {
        Family::OneHot(k) => Group::OneHot(
            (0..row.num_polys)
                .map(|_| {
                    let rows = 1usize << (row.num_vars - k.trailing_zeros() as usize);
                    // Byte zero is "no entry" unless the zero-row mask (bit 0)
                    // commits it, as in the packed trace.
                    let keep_zero = witness.zero_committed_columns & 1 == 1;
                    let mut indices: Vec<Option<u8>> = (0..rows)
                        .map(|_| {
                            let selected = witness.trace.selected(&mut rng, k);
                            (selected != 0 || keep_zero).then_some(selected)
                        })
                        .collect();
                    if indices.iter().all(Option::is_none) {
                        indices[0] = Some(0);
                        stats::count(KNOWN_ZERO_POLYNOMIAL);
                    }
                    OneHotPolynomial::new(k, indices)
                })
                .collect(),
        ),
        Family::Dense => Group::Dense(
            (0..row.num_polys)
                .map(|_| {
                    let mut evaluations: Vec<AkitaField> = (0..1usize << row.num_vars)
                        .map(|_| AkitaField::from_u64(witness.dense.word(&mut rng)))
                        .collect();
                    if evaluations
                        .iter()
                        .all(|value| *value == AkitaField::from_u64(0))
                    {
                        evaluations[0] = AkitaField::from_u64(1);
                        stats::count(KNOWN_ZERO_POLYNOMIAL);
                    }
                    Polynomial::new(evaluations)
                })
                .collect(),
        ),
    };
    let (commitment, hint, evaluations, polynomials): (
        _,
        _,
        Vec<AkitaField>,
        AkitaNativeBatchPolynomials<'_>,
    ) = match &group {
        Group::OneHot(polys) => {
            let (commitment, hint) = stats::time("commit", || {
                AkitaScheme::commit_one_hot_group(&prover_setup, digest, polys)
            })
            .map_err(fail(Stage::Commit))?;
            (
                commitment,
                hint,
                polys.iter().map(|poly| poly.evaluate(&point)).collect(),
                polys
                    .iter()
                    .map(|poly| poly as &dyn MultilinearPoly<AkitaField>)
                    .collect(),
            )
        }
        Group::Dense(polys) => {
            let (commitment, hint) = stats::time("commit", || {
                AkitaScheme::commit_group(&prover_setup, digest, polys)
            })
            .map_err(fail(Stage::Commit))?;
            (
                commitment,
                hint,
                polys.iter().map(|poly| poly.evaluate(&point)).collect(),
                polys
                    .iter()
                    .map(|poly| poly as &dyn MultilinearPoly<AkitaField>)
                    .collect(),
            )
        }
    };
    let statement: Vec<_> = evaluations
        .into_iter()
        .map(|evaluation| VerifierOpeningClaim {
            commitment: commitment.clone(),
            evaluation: EvaluationClaim::new(point.clone(), evaluation),
        })
        .collect();
    let label = b"jolt-akita-fuzz/grid";
    let mut prover_transcript = Blake2bTranscript::new(label);
    let proof = stats::time("prove", || {
        liveness::observe(&row.label(), || {
            <AkitaNativeBatching as BatchOpeningScheme>::prove_batch(
                &prover_setup,
                statement.clone(),
                polynomials,
                hint,
                &mut prover_transcript,
            )
        })
    })
    .map_err(fail(Stage::Prove))?;
    stats::count("honest_proofs");
    let proof = transport::roundtrip(&proof);
    let verifier_setup = transport::roundtrip(&verifier_setup);
    let mut verifier_transcript = Blake2bTranscript::new(label);
    stats::time("verify", || {
        <AkitaNativeBatching as BatchOpeningScheme>::verify_batch(
            &verifier_setup,
            &statement,
            &proof,
            &mut verifier_transcript,
        )
    })
    .map_err(fail(Stage::Verify))?;
    assert_eq!(prover_transcript.state(), verifier_transcript.state());
    stats::count("honest_verified");
    Ok(())
}

/// One seed per family and polynomial count at its smallest row.
pub fn seeds() -> Vec<(String, Vec<u8>)> {
    let witness = Witness {
        seed: 11,
        dense: opening::Fill::Random,
        trace: opening::Fill::Random,
        columns: 63,
        zero_committed_columns: 1,
        point: opening::Fill::Random,
    };
    let mut seeds = Vec::new();
    for (family_index, family) in FAMILIES.iter().enumerate() {
        for index in 0..4u16 {
            let mut bytes = ((family_index as u16) << 8 | index).to_le_bytes().to_vec();
            bytes.push(6);
            bytes.extend([family_index as u8 + 1; 32]);
            bytes.extend(witness.encode());
            seeds.push((format!("{}-{index}", family.name()), bytes));
        }
    }
    seeds
}
