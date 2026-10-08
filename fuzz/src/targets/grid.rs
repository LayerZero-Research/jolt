//! Every base catalog row through Jolt's Akita adapter (liveness).
//!
//! Rows come from the loaded artifacts, so the case list tracks the catalogs
//! without restating their grids. Every row is a supported statement, so any
//! failure (panic, clean error, or verifier rejection) of an honest opening is
//! a finding. Rows use Jolt's own setups:
//!
//! - one-hot rows, in every chunk profile: the production `OneHotTrace`
//!   group, its columns streamed through `commit_trace_one_hot` and checked
//!   against the generic one-hot group commitment;
//! - dense-bounded rows: `AkitaNativeBatching`'s same-point batch over one
//!   commitment group.
//!
//! The lane variant (`JOLT_FUZZ_GRID_FAMILY` = `k16`, `k256`, or `dense`)
//! fixes the family; without it the input picks. Rows above
//! `JOLT_FUZZ_MAX_CASE_COEFFS` are excluded; `jolt-fuzz-dev grid-sweep` runs
//! them in release mode.

use std::sync::{Arc, OnceLock};

use jolt_akita::{
    AkitaChunkProfile, AkitaField, AkitaNativeBatchPolynomials, AkitaNativeBatching,
    AkitaScheduleArtifacts, AkitaScheme, AkitaSetupParams, AKITA_ONE_HOT_K16, AKITA_ONE_HOT_K256,
};
use jolt_claims::protocols::jolt::lattice::OneHotTraceSetupShape;
use jolt_field::{CanonicalEncoding, Ring};
use jolt_openings::{
    BatchOpeningScheme, CommitmentScheme, EvaluationClaim, OpeningsError, VerifierOpeningClaim,
};
use jolt_poly::{MultilinearPoly, Polynomial};
use jolt_transcript::{Blake2bTranscript, Transcript};

use crate::input::{Reader, SplitMix64};
use crate::opening::{self, Fill, Witness};
use crate::shape::{Failure, SetupRequest, Stage, PROFILES};
use crate::{artifacts, env, liveness, stats, transport};

pub const FAMILY_ENV: &str = "JOLT_FUZZ_GRID_FAMILY";

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

/// One scalar catalog row: the final group layout, the witness chunk profile
/// of its catalog, and whether its schedule offloads the setup to a recursive
/// prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub family: Family,
    pub profile: AkitaChunkProfile,
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
            "{} {:?} {}:{}{}",
            self.family.name(),
            self.profile,
            self.num_vars,
            self.num_polys,
            if self.offloaded { " offloaded" } else { "" }
        )
    }
}

/// Every row of the shipped one-hot catalogs (each profile) and the bounded
/// dense catalog, sorted by family, profile, and shape.
pub fn rows(artifacts: &AkitaScheduleArtifacts) -> Vec<Row> {
    let mut catalogs = Vec::new();
    for family in FAMILIES {
        match family {
            Family::OneHot(k) => {
                for profile in PROFILES {
                    catalogs.push((
                        family,
                        profile,
                        artifacts.one_hot_catalog_for_profile(k, profile),
                    ));
                }
            }
            Family::Dense => {
                catalogs.push((family, AkitaChunkProfile::Single, artifacts.dense_catalog()));
            }
        }
    }
    let mut rows = Vec::new();
    for (family, profile, catalog) in catalogs {
        let catalog = catalog
            .unwrap_or_else(|error| panic!("load {} {profile:?} catalog: {error}", family.name()));
        for row in catalog.rows() {
            let group = row.profiles().final_group.group;
            rows.push(Row {
                family,
                profile,
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
    rows.sort_by_key(|row| {
        (
            row.family.name(),
            row.profile as u8,
            row.num_polys,
            row.num_vars,
        )
    });
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
    let digest = reader.bytes::<32>();
    let witness = Witness::decode(&mut reader);
    check(row, digest, &witness);
}

/// Prove and verify one honest opening of `row`; panics on any failure.
pub fn check(row: Row, digest: [u8; 32], witness: &Witness) {
    let artifacts = artifacts::shared();
    let context = row.label();
    let result = env::on_large_stack(|| match row.family {
        Family::OneHot(k) => trace_row(&artifacts, row, k, digest, witness),
        Family::Dense => dense_row(&artifacts, row, digest, witness),
    });
    match result {
        Ok(()) => stats::count("rows_verified"),
        Err(failure) => {
            panic!("liveness: catalog row {context} failed at {failure} (witness {witness:?})")
        }
    }
}

fn trace_row(
    artifacts: &Arc<AkitaScheduleArtifacts>,
    row: Row,
    one_hot_k: usize,
    digest: [u8; 32],
    witness: &Witness,
) -> Result<(), Failure> {
    let request = SetupRequest {
        setup_shape: OneHotTraceSetupShape {
            num_vars: row.num_vars,
            num_polys: row.num_polys,
        },
        layout_digest: digest,
        one_hot_k,
        profile: row.profile,
        untrusted: None,
        trusted: None,
        program: Vec::new(),
    };
    opening::commit(artifacts, &request, witness)?.prove_and_verify(&row.label())
}

fn fail(stage: Stage) -> impl Fn(OpeningsError) -> Failure {
    move |error| Failure {
        stage: stage.clone(),
        message: error.to_string(),
    }
}

fn dense_row(
    artifacts: &Arc<AkitaScheduleArtifacts>,
    row: Row,
    digest: [u8; 32],
    witness: &Witness,
) -> Result<(), Failure> {
    let params =
        AkitaSetupParams::dense_only(row.num_vars, row.num_polys, digest, Arc::clone(artifacts));
    let (prover_setup, verifier_setup) =
        stats::time("setup", || AkitaScheme::setup(params)).map_err(fail(Stage::Setup))?;
    let mut rng = SplitMix64::new(witness.seed);
    let point: Vec<AkitaField> = (0..row.num_vars)
        .map(|_| AkitaField::from_u128_reduced(rng.next_u128()))
        .collect();
    let group: Vec<Polynomial<AkitaField>> = (0..row.num_polys)
        .map(|_| {
            Polynomial::new(
                (0..1usize << row.num_vars)
                    .map(|_| AkitaField::from_u64(witness.dense.word(&mut rng)))
                    .collect(),
            )
        })
        .collect();
    let (commitment, hint) = stats::time("commit", || {
        AkitaScheme::commit_group(&prover_setup, digest, &group)
    })
    .map_err(fail(Stage::Commit))?;
    let statement: Vec<_> = group
        .iter()
        .map(|polynomial| VerifierOpeningClaim {
            commitment: commitment.clone(),
            evaluation: EvaluationClaim::new(point.clone(), polynomial.evaluate(&point)),
        })
        .collect();
    let polynomials: AkitaNativeBatchPolynomials<'_> = group
        .iter()
        .map(|polynomial| polynomial as &dyn MultilinearPoly<AkitaField>)
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

/// The first rows of every family, with a random witness committing row zero
/// in the first column.
pub fn seeds() -> Vec<(String, Vec<u8>)> {
    let witness = Witness {
        seed: 11,
        dense: Fill::Random,
        trace: Fill::Random,
        zero_committed_columns: 1,
        point: Fill::Random,
    };
    let mut seeds = Vec::new();
    for (family_index, family) in FAMILIES.iter().enumerate() {
        for index in 0..4u16 {
            let mut bytes = ((family_index as u16) << 8 | index).to_le_bytes().to_vec();
            bytes.extend([family_index as u8 + 1; 32]);
            bytes.extend(witness.encode());
            seeds.push((format!("{}-{index}", family.name()), bytes));
        }
    }
    seeds
}
