//! Honest grouped openings through Jolt's Akita adapter.
//!
//! One opening is what Jolt's stage 8 discharges: every auxiliary dense
//! object (advice words, committed-program chunks and image) followed by the
//! native `OneHotTrace` column group, proved in one heterogeneous batch. The
//! driver goes through the same public seams production uses
//! (`AkitaScheme::setup` with the grouped request, `transparent_object_setup`
//! and `commit`, `commit_trace_one_hot`, `prove_batch`, `verify_batch`) and
//! transports the verifier setup and proof through serde before verifying, as
//! a deployed verifier receives them.
//!
//! The honest claims for the trace come from an independent materialization:
//! each column as a plain `OneHotPolynomial`, evaluated by jolt-poly's
//! reference `evaluate`. Without auxiliary groups the streamed commitment must
//! also equal the generic one-hot group commitment of those polynomials.

use std::sync::Arc;

use jolt_akita::{
    AkitaCommitment, AkitaField, AkitaProverHint, AkitaProverSetup, AkitaScheduleArtifacts,
    AkitaScheme, AkitaVerifierSetup, TraceOneHotRows,
};
use jolt_field::{CanonicalEncoding, Ring};
use jolt_openings::{
    CommitmentScheme, GroupOpeningClaim, OpeningsError, TaggedGroupOpeningClaim,
    TransparentObjectSetup,
};
use jolt_poly::{MultilinearPoly, OneHotPolynomial, Polynomial};
use jolt_transcript::{Blake2bTranscript, Transcript};

use crate::input::{Reader, SplitMix64};
use crate::liveness;
use crate::shape::{Failure, SetupRequest, Stage};
use crate::stats;

fn fail(stage: Stage) -> impl Fn(OpeningsError) -> Failure {
    move |error| Failure {
        stage: stage.clone(),
        message: error.to_string(),
    }
}

/// Row-major trace source: `selected[row * columns + column]`, with byte
/// zero meaning "no entry" unless the row's zero mask commits it.
pub struct TraceRows {
    pub num_rows: usize,
    pub num_columns: usize,
    pub selected: Vec<u8>,
    /// Row zero's committed-zero mask; row `r` uses it rotated left by `r`,
    /// restricted to the real columns.
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
    fn committed_digit_zero_mask(&self, row: usize) -> u64 {
        let columns = if self.num_columns == 64 {
            u64::MAX
        } else {
            (1u64 << self.num_columns) - 1
        };
        self.zero_committed_columns.rotate_left((row % 64) as u32) & columns
    }
}

impl TraceRows {
    /// Each column as a one-hot polynomial over the trace rows.
    pub fn materialize(&self, one_hot_k: usize) -> Vec<OneHotPolynomial> {
        (0..self.num_columns)
            .map(|column| {
                let indices = (0..self.num_rows)
                    .map(|row| {
                        let selected = self.selected[row * self.num_columns + column];
                        (selected != 0 || self.committed_digit_zero_mask(row) >> column & 1 == 1)
                            .then_some(selected)
                    })
                    .collect();
                OneHotPolynomial::new(one_hot_k, indices)
            })
            .collect()
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

    pub fn word(self, rng: &mut SplitMix64) -> u64 {
        match self {
            Fill::Zero => 0,
            Fill::Max => u64::MAX,
            Fill::Random => rng.next_u64(),
            Fill::Sparse => {
                let value = rng.next_u64();
                if value.is_multiple_of(16) {
                    value
                } else {
                    0
                }
            }
        }
    }

    /// Selected one-hot row in `0..k`; `Max` selects the last row.
    pub fn selected(self, rng: &mut SplitMix64, k: usize) -> u8 {
        match self {
            Fill::Zero => 0,
            Fill::Max => (k - 1) as u8,
            Fill::Random => (rng.next_u64() % k as u64) as u8,
            Fill::Sparse => {
                let value = rng.next_u64();
                if value.is_multiple_of(8) {
                    ((value >> 8) % k as u64) as u8
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
    pub zero_committed_columns: u64,
    pub point: Fill,
}

impl Witness {
    pub fn decode(reader: &mut Reader<'_>) -> Self {
        Self {
            seed: reader.u64(),
            dense: Fill::decode(reader),
            trace: Fill::decode(reader),
            zero_committed_columns: reader.u64(),
            point: Fill::decode(reader),
        }
    }

    /// Bytes that [`Witness::decode`] maps back to `self`.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.seed.to_le_bytes().to_vec();
        out.push(self.dense.tag());
        out.push(self.trace.tag());
        out.extend_from_slice(&self.zero_committed_columns.to_le_bytes());
        out.push(self.point.tag());
        out
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
    pub prover_setup: AkitaProverSetup,
    pub verifier_setup: AkitaVerifierSetup,
    pub auxiliary: Vec<(
        TaggedGroupOpeningClaim<AkitaField, AkitaCommitment>,
        AkitaProverHint,
    )>,
    pub main: GroupOpeningClaim<AkitaField, AkitaCommitment>,
    pub main_hint: AkitaProverHint,
}

const TRANSCRIPT_LABEL: &[u8] = b"jolt-akita-fuzz/opening";

/// Setup and commit every group of `request` with honest data: one dense
/// object per auxiliary plan, then the trace's native columns over
/// `2^(num_vars - log K)` rows.
pub fn commit(
    artifacts: &Arc<AkitaScheduleArtifacts>,
    request: &SetupRequest,
    witness: &Witness,
) -> Result<Opening, Failure> {
    let shape = request.setup_shape;
    let (prover_setup, verifier_setup) = stats::time("setup", || {
        AkitaScheme::setup(request.setup_params(artifacts))
    })
    .map_err(fail(Stage::Setup))?;

    let mut rng = SplitMix64::new(witness.seed);
    let mut auxiliary = Vec::with_capacity(request.auxiliary_count());
    for plan in request.auxiliary() {
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
        auxiliary.push((
            TaggedGroupOpeningClaim::new(
                plan.group_role(),
                GroupOpeningClaim::new(commitment, at, vec![evaluation]),
            ),
            hint,
        ));
    }

    let log_k = request.one_hot_k.trailing_zeros() as usize;
    let log_rows = shape.num_vars.checked_sub(log_k).ok_or_else(|| Failure {
        stage: Stage::Commit,
        message: format!("final arity {} below log K {log_k}", shape.num_vars),
    })?;
    let num_rows = 1usize << log_rows;
    let num_columns = shape.num_polys;
    let selected = (0..num_rows * num_columns)
        .map(|_| witness.trace.selected(&mut rng, request.one_hot_k))
        .collect();
    let rows = Arc::new(TraceRows {
        num_rows,
        num_columns,
        selected,
        zero_committed_columns: witness.zero_committed_columns,
    });
    let columns = rows.materialize(request.one_hot_k);
    let hints: Vec<&AkitaProverHint> = auxiliary.iter().map(|(_, hint)| hint).collect();
    let (main_commitment, main_hint) = stats::time("commit", || {
        AkitaScheme::commit_trace_one_hot(
            &prover_setup,
            request.layout_digest,
            Arc::clone(&rows) as Arc<dyn TraceOneHotRows>,
            &hints,
        )
    })
    .map_err(fail(Stage::Commit))?;
    if hints.is_empty() {
        let (reference_commitment, _) = stats::time("commit_reference", || {
            AkitaScheme::commit_one_hot_group(&prover_setup, request.layout_digest, &columns)
        })
        .map_err(fail(Stage::Commit))?;
        assert_eq!(
            main_commitment, reference_commitment,
            "streamed OneHotTrace commitment differs from the one-hot group commitment of its columns"
        );
    }
    let at = point(witness.point, &mut rng, shape.num_vars);
    let evaluations = columns.iter().map(|column| column.evaluate(&at)).collect();
    Ok(Opening {
        prover_setup,
        verifier_setup,
        auxiliary,
        main: GroupOpeningClaim::new(main_commitment, at, evaluations),
        main_hint,
    })
}

impl Opening {
    /// Prove under the liveness oracle, transport, and verify.
    pub fn prove_and_verify(self, context: &str) -> Result<(), Failure> {
        let Opening {
            prover_setup,
            verifier_setup,
            auxiliary,
            main,
            main_hint,
        } = self;
        let claims: Vec<_> = auxiliary.iter().map(|(claim, _)| claim.clone()).collect();
        let mut prover_transcript = Blake2bTranscript::new(TRANSCRIPT_LABEL);
        let proof = stats::time("prove", || {
            liveness::observe(context, || {
                AkitaScheme::prove_batch(
                    &prover_setup,
                    auxiliary,
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
