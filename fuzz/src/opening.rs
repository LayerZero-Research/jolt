//! Honest grouped openings through Jolt's Akita adapter.
//!
//! One opening is what Jolt's stage 8 discharges: every precommitted dense
//! object (advice words, committed-program chunks and image) followed by the
//! packed `OneHotTrace` group, proved in one heterogeneous batch. The driver
//! goes through the same public seams production uses (`AkitaScheme::setup`
//! with the grouped request, `transparent_object_setup` + `commit`,
//! `commit_trace_one_hot`, `prove_batch`, `verify_batch`) and transports the
//! verifier setup and proof through serde before verifying, as a deployed
//! verifier receives them.
//!
//! The honest claim for the packed trace comes from an independent
//! materialization: the same rows as a plain `OneHotPolynomial`, whose
//! commitment must equal the streamed one and whose evaluation is jolt-poly's
//! reference `evaluate`.

use std::sync::Arc;

use jolt_akita::{
    AkitaCommitment, AkitaField, AkitaProverHint, AkitaScheduleArtifacts, AkitaScheme,
    AkitaSetupParams, AkitaVerifierSetup, TraceOneHotRows,
};
use jolt_claims::protocols::jolt::lattice::packing::{
    ONE_HOT_TRACE_K16_CAPACITY, ONE_HOT_TRACE_K256_CAPACITY,
};
use jolt_field::{CanonicalEncoding, Ring};
use jolt_openings::{
    CommitmentScheme, GroupOpeningClaim, PrecommittedClaim, TransparentObjectSetup,
};
use jolt_poly::{MultilinearPoly, OneHotPolynomial, Polynomial};
use jolt_transcript::{Blake2bTranscript, Transcript};

use crate::input::{Reader, SplitMix64};
use crate::liveness;
use crate::shape::{Failure, SetupRequest, Stage};
use crate::stats;

fn fail(stage: Stage) -> impl Fn(jolt_openings::OpeningsError) -> Failure {
    move |error| Failure {
        stage: stage.clone(),
        message: error.to_string(),
    }
}

/// Row-major packed-trace source: `selected[row * columns + column]`, with
/// byte zero meaning "no entry" unless the column commits row zero.
pub struct TraceRows {
    pub num_rows: usize,
    pub num_columns: usize,
    pub selected: Vec<u8>,
    /// Columns whose zero byte is a committed selection of row zero (RAM).
    pub zero_committed_columns: u64,
}

impl TraceOneHotRows for TraceRows {
    fn num_rows(&self) -> usize {
        self.num_rows
    }
    fn num_columns(&self) -> usize {
        self.num_columns
    }
    fn fill_row(&self, row: usize, selected_rows: &mut [u8]) {
        let start = row * self.num_columns;
        selected_rows.copy_from_slice(&self.selected[start..start + self.num_columns]);
    }
    fn committed_digit_zero_mask(&self, _row: usize) -> u64 {
        self.zero_committed_columns
    }
}

impl TraceRows {
    /// Column-major one-hot rows of the packed polynomial: column `c` of
    /// trace row `t` is packed row `c * T + t`; padding columns are empty.
    pub fn materialize(&self, one_hot_k: usize, column_capacity: usize) -> OneHotPolynomial {
        let indices = (0..column_capacity)
            .flat_map(|column| {
                (0..self.num_rows).map(move |row| {
                    if column >= self.num_columns {
                        return None;
                    }
                    let selected = self.selected[row * self.num_columns + column];
                    (selected != 0 || self.zero_committed_columns >> column & 1 == 1)
                        .then_some(selected)
                })
            })
            .collect();
        OneHotPolynomial::new(one_hot_k, indices)
    }
}

/// Witness values: a pattern chosen by the input, expanded from a seed.
#[derive(Clone, Copy, Debug)]
pub enum Fill {
    Zero,
    Max,
    Random,
    Sparse,
}

impl Fill {
    pub fn decode(reader: &mut Reader<'_>) -> Self {
        match reader.u8() % 4 {
            0 => Fill::Zero,
            1 => Fill::Max,
            2 => Fill::Random,
            _ => Fill::Sparse,
        }
    }

    fn word(self, rng: &mut SplitMix64) -> u64 {
        match self {
            Fill::Zero => 0,
            Fill::Max => u64::MAX,
            Fill::Random => rng.next_u64(),
            Fill::Sparse => {
                let value = rng.next_u64();
                if value % 16 == 0 {
                    value
                } else {
                    0
                }
            }
        }
    }

    /// Selected one-hot row in `0..k`; `Max` selects the last row.
    fn selected(self, rng: &mut SplitMix64, k: usize) -> u8 {
        match self {
            Fill::Zero => 0,
            Fill::Max => (k - 1) as u8,
            Fill::Random => (rng.next_u64() % k as u64) as u8,
            Fill::Sparse => {
                let value = rng.next_u64();
                if value % 8 == 0 {
                    (value >> 8) as u8 % k as u8
                } else {
                    0
                }
            }
        }
    }
}

/// Everything the input chooses beyond the shape.
#[derive(Clone, Copy, Debug)]
pub struct Witness {
    pub seed: u64,
    pub dense: Fill,
    pub trace: Fill,
    pub columns: u8,
    pub zero_committed_columns: u64,
    pub point: Fill,
}

impl Witness {
    pub fn decode(reader: &mut Reader<'_>) -> Self {
        Self {
            seed: reader.u64(),
            dense: Fill::decode(reader),
            trace: Fill::decode(reader),
            columns: reader.u8(),
            zero_committed_columns: reader.u64(),
            point: Fill::decode(reader),
        }
    }
}

fn point(fill: Fill, rng: &mut SplitMix64, num_vars: usize) -> Vec<AkitaField> {
    (0..num_vars)
        .map(|_| match fill {
            Fill::Zero => AkitaField::from_u64(rng.next_u64() & 1),
            Fill::Max => -AkitaField::from_u64(1 + (rng.next_u64() & 1)),
            Fill::Random | Fill::Sparse => AkitaField::from_u128_reduced(rng.next_u128()),
        })
        .collect()
}

/// A statement the verifier checks, with the prover's retained hints.
pub struct Opening {
    pub prover_setup: jolt_akita::AkitaProverSetup,
    pub verifier_setup: AkitaVerifierSetup,
    pub precommitted: Vec<(
        PrecommittedClaim<AkitaField, AkitaCommitment>,
        AkitaProverHint,
    )>,
    pub main: GroupOpeningClaim<AkitaField, AkitaCommitment>,
    pub main_hint: AkitaProverHint,
}

const TRANSCRIPT_LABEL: &[u8] = b"jolt-akita-fuzz/opening";

/// Setup and commit every group of `request` with honest data, packing the
/// trace at Jolt's selector capacity for the request's chunk width.
pub fn commit(
    artifacts: &Arc<AkitaScheduleArtifacts>,
    request: &SetupRequest,
    witness: &Witness,
) -> Result<Opening, Failure> {
    let capacity = match request.one_hot_k {
        16 => ONE_HOT_TRACE_K16_CAPACITY,
        _ => ONE_HOT_TRACE_K256_CAPACITY,
    };
    commit_with_capacity(artifacts, request, witness, capacity)
}

/// [`commit`] with an explicit selector capacity (a power of two with
/// `log K + log capacity <= final arity`); the trace fills the remaining
/// variables with rows.
pub fn commit_with_capacity(
    artifacts: &Arc<AkitaScheduleArtifacts>,
    request: &SetupRequest,
    witness: &Witness,
    capacity: usize,
) -> Result<Opening, Failure> {
    let shape = request.setup_shape;
    let count = request.precommitted_count();
    let (prover_setup, verifier_setup) = stats::time("setup", || {
        AkitaScheme::setup(AkitaSetupParams::one_hot_only_grouped(
            shape.num_vars,
            shape.num_polys,
            shape.num_polys + count,
            request.layout_digest,
            request.one_hot_k,
            request.schedule_params(),
            Arc::clone(artifacts),
        ))
    })
    .map_err(fail(Stage::Setup))?;

    let mut rng = SplitMix64::new(witness.seed);
    let mut precommitted = Vec::with_capacity(count);
    for plan in request.precommitted() {
        let arity = plan.packing().packed_num_vars();
        let (object_setup, _) =
            AkitaScheme::transparent_object_setup(artifacts, arity, plan.layout_digest())
                .map_err(fail(Stage::Setup))?;
        let polynomial = Polynomial::new(
            (0..1usize << arity)
                .map(|_| AkitaField::from_u64(witness.dense.word(&mut rng)))
                .collect(),
        );
        let (commitment, hint) =
            stats::time("commit", || AkitaScheme::commit(&polynomial, &object_setup))
                .map_err(fail(Stage::Commit))?;
        let at = point(witness.point, &mut rng, arity);
        let evaluation = polynomial.evaluate(&at);
        precommitted.push((
            PrecommittedClaim::new(
                plan.precommitted_role(),
                GroupOpeningClaim::new(commitment, at, vec![evaluation]),
            ),
            hint,
        ));
    }

    let log_k = request.one_hot_k.trailing_zeros() as usize;
    let log_capacity = capacity.trailing_zeros() as usize;
    let log_rows = shape
        .num_vars
        .checked_sub(log_k + log_capacity)
        .ok_or_else(|| Failure {
            stage: Stage::Commit,
            message: format!(
                "final arity {} below log K {log_k} + log capacity {log_capacity}",
                shape.num_vars
            ),
        })?;
    let num_rows = 1usize << log_rows;
    let num_columns = 1 + usize::from(witness.columns) % capacity.min(64);
    let mut selected = vec![0u8; num_rows * num_columns];
    for byte in &mut selected {
        *byte = witness.trace.selected(&mut rng, request.one_hot_k);
    }
    let mask = if num_columns == 64 {
        u64::MAX
    } else {
        (1u64 << num_columns) - 1
    };
    let rows = Arc::new(TraceRows {
        num_rows,
        num_columns,
        selected,
        zero_committed_columns: witness.zero_committed_columns & mask,
    });
    let materialized = rows.materialize(request.one_hot_k, capacity);
    let hints: Vec<&AkitaProverHint> = precommitted.iter().map(|(_, hint)| hint).collect();
    let (main_commitment, main_hint) = stats::time("commit", || {
        AkitaScheme::commit_trace_one_hot(
            &prover_setup,
            request.layout_digest,
            capacity,
            Arc::clone(&rows) as Arc<dyn TraceOneHotRows>,
            &hints,
        )
    })
    .map_err(fail(Stage::Commit))?;
    let (reference_commitment, _) = stats::time("commit_reference", || {
        if hints.is_empty() {
            AkitaScheme::commit_one_hot_group_owned(
                &prover_setup,
                request.layout_digest,
                vec![materialized.clone()],
            )
        } else {
            AkitaScheme::commit_one_hot_group_owned_with_precommitted(
                &prover_setup,
                request.layout_digest,
                vec![materialized.clone()],
                &hints,
            )
        }
    })
    .map_err(fail(Stage::Commit))?;
    assert_eq!(
        main_commitment, reference_commitment,
        "streamed OneHotTrace commitment differs from the materialized one-hot polynomial's"
    );
    let at = point(witness.point, &mut rng, shape.num_vars);
    let evaluation = materialized.evaluate(&at);
    Ok(Opening {
        prover_setup,
        verifier_setup,
        precommitted,
        main: GroupOpeningClaim::new(main_commitment, at, vec![evaluation]),
        main_hint,
    })
}

impl Opening {
    /// Prove under the liveness oracle, transport, and verify.
    pub fn prove_and_verify(self, context: &str) -> Result<(), Failure> {
        let Opening {
            prover_setup,
            verifier_setup,
            precommitted,
            main,
            main_hint,
        } = self;
        let claims: Vec<_> = precommitted
            .iter()
            .map(|(claim, _)| claim.clone())
            .collect();
        let mut prover_transcript = Blake2bTranscript::new(TRANSCRIPT_LABEL);
        let proof = stats::time("prove", || {
            liveness::observe(context, || {
                AkitaScheme::prove_batch(
                    &prover_setup,
                    precommitted,
                    main.clone(),
                    main_hint,
                    &mut prover_transcript,
                )
            })
        })
        .map_err(fail(Stage::Prove))?;
        stats::count("honest_proofs");

        let proof = crate::transport::roundtrip(&proof);
        let verifier_setup = crate::transport::roundtrip(&verifier_setup);
        let mut verifier_transcript = Blake2bTranscript::new(TRANSCRIPT_LABEL);
        stats::time("verify", || {
            AkitaScheme::verify_batch(
                &verifier_setup,
                &claims,
                &main,
                &proof,
                &mut verifier_transcript,
            )
        })
        .map_err(fail(Stage::Verify))?;
        assert_eq!(
            prover_transcript.state(),
            verifier_transcript.state(),
            "{context}: prover and verifier transcripts diverged on an accepted proof"
        );
        stats::count("honest_verified");
        Ok(())
    }
}

impl Fill {
    fn tag(self) -> u8 {
        match self {
            Fill::Zero => 0,
            Fill::Max => 1,
            Fill::Random => 2,
            Fill::Sparse => 3,
        }
    }
}

impl Witness {
    /// Bytes that [`Witness::decode`] maps back to `self`.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.seed.to_le_bytes().to_vec();
        out.push(self.dense.tag());
        out.push(self.trace.tag());
        out.push(self.columns);
        out.extend_from_slice(&self.zero_committed_columns.to_le_bytes());
        out.push(self.point.tag());
        out
    }
}
