//! Packed stage 0: input validation, the Fiat-Shamir preamble, the
//! prefix-packed `OneHotTrace` commitment, and the packed commitment-object
//! absorbs.
//!
//! The transcript work is the verifier's own exported code
//! ([`validate_inputs_from_parts`], [`absorb_transcript_preamble`],
//! [`absorb_packed_commitments`]) — the two sides share the absorb sequence
//! structurally, so stage-0 Fiat-Shamir drift is impossible by construction.

use common::jolt_device::JoltDevice;
use jolt_akita::PrecommittedRole;
use jolt_claims::protocols::jolt::lattice::{OneHotTraceShape, ONE_HOT_TRACE_LAYOUT};
use jolt_claims::protocols::jolt::JoltRelationId;
use jolt_crypto::VectorCommitment;
use jolt_field::Field;
use jolt_openings::{
    CommitmentScheme, GroupCommitmentMetadata, GroupSetupMetadata, TransparentObjectSetup,
};
use jolt_transcript::{AppendToTranscript, Transcript};
use jolt_verifier::{
    absorb_packed_commitments, absorb_transcript_preamble, validate_inputs_from_parts,
    CheckedInputs, ProofTranscriptConfig, VerifierError,
};
use jolt_witness::JoltWitnessPlane;

use super::witness::{assemble_one_hot_trace_rows, commit_advice_dense, DenseAdviceObject};
use crate::{JoltProverPreprocessing, ProverConfig, ProverError};

/// Stage 0's outputs: the validated inputs, the seeded transcript (positioned
/// exactly where the packed verifier's own stage boundary leaves its own),
/// the prefix-packed `OneHotTrace` commitment with its opening hint
/// (consumed by stage 8's native opening), and the per-proof
/// untrusted-advice commitment object.
pub struct Stage0Output<PCS, T>
where
    PCS: CommitmentScheme,
{
    pub checked: CheckedInputs,
    pub transcript: T,
    pub commitment: PCS::Output,
    pub hint: PCS::OpeningHint,
    pub untrusted_advice: Option<DenseAdviceObject<PCS>>,
}

/// Validate inputs, seed the transcript, assemble and commit the
/// prefix-packed `OneHotTrace` polynomial, commit the untrusted-advice dense
/// word object when advice bytes are present, and absorb the packed commitment
/// objects in canonical object order (the verifier's own absorb helper).
#[tracing::instrument(skip_all)]
pub fn prove_stage0<F, PCS, VC, T, W>(
    preprocessing: &JoltProverPreprocessing<PCS, VC>,
    config: &ProverConfig,
    trusted_advice: Option<&DenseAdviceObject<PCS>>,
    witness: &W,
    public_io: &JoltDevice,
) -> Result<Stage0Output<PCS, T>, ProverError<F>>
where
    F: Field,
    PCS: CommitmentScheme<Field = F>
        + TransparentObjectSetup
        + jolt_akita::TraceOneHotCommitment
        + jolt_akita::PrecommittedTraceBatching,
    PCS::ProverSetup: GroupSetupMetadata,
    PCS::Output: Clone + AppendToTranscript + GroupCommitmentMetadata,
    VC: VectorCommitment<Field = F>,
    T: Transcript<Challenge = F>,
    W: JoltWitnessPlane<F>,
{
    // Trusted-advice presence rides on the external precommitted object;
    // committed-program presence on the retained prover data. Require
    // agreement with the public shape so a mismatch fails here rather than
    // as an opaque downstream sumcheck error.
    if trusted_advice.is_some() == public_io.trusted_advice.is_empty() {
        return Err(ProverError::Unsupported {
            reason: "trusted-advice object presence disagrees with the trusted advice bytes",
        });
    }
    if let Some(object) = trusted_advice {
        if object.plan.packing().ids()
            != [jolt_claims::protocols::jolt::JoltCommittedPolynomial::TrustedAdvice]
            || object.commitment.layout_digest() != object.plan.layout_digest()
            || object.commitment.num_vars() != object.plan.packing().packed_num_vars()
            || object.commitment.poly_count() != 1
            || object.setup.max_num_vars() != object.commitment.num_vars()
            || object.setup.max_num_polys_per_commitment_group() != 1
            || object.setup.default_layout_digest() != object.plan.layout_digest()
        {
            return Err(ProverError::Unsupported {
                reason: "trusted-advice precommit artifact has inconsistent shape metadata",
            });
        }
        let words = common::advice::canonical_advice_words(
            &public_io.trusted_advice,
            public_io.memory_layout.max_trusted_advice_size as usize,
        )
        .map_err(|_| ProverError::Unsupported {
            reason: "trusted-advice bytes do not fit the scheduled precommit capacity",
        })?;
        let expected_word_vars = words.len().ilog2() as usize;
        let evaluations = object.polynomial.evals();
        if object.word_vars != expected_word_vars
            || evaluations.len() != 1usize << object.plan.packing().packed_num_vars()
            || evaluations[..words.len()]
                .iter()
                .zip(words)
                .any(|(evaluation, word)| *evaluation != F::from_u64(word))
            || evaluations[1usize << expected_word_vars..]
                .iter()
                .any(|evaluation| *evaluation != F::default())
        {
            return Err(ProverError::Unsupported {
                reason: "trusted-advice precommit artifact does not match the public advice bytes",
            });
        }
    }
    if preprocessing.committed_program.is_some()
        != preprocessing.verifier.program.committed().is_some()
    {
        return Err(ProverError::Unsupported {
            reason: "retained ProgramOneHot presence disagrees with the preprocessing mode",
        });
    }
    // The verifier absorbs the VERIFIER-preprocessing-held ProgramOneHot
    // commitments; retained objects committed from different data would only
    // surface as an opaque Fiat-Shamir divergence at verification, so reject
    // them by name here.
    if let (Some(data), Some(committed)) = (
        preprocessing.committed_program.as_ref(),
        preprocessing.verifier.program.committed(),
    ) {
        let objects = &data.program_one_hot.objects;
        if objects.len() != committed.program_one_hot_commitments.len()
            || objects
                .iter()
                .zip(&committed.program_one_hot_commitments)
                .any(|(object, commitment)| object.commitment != *commitment)
        {
            return Err(ProverError::Unsupported {
                reason: "the retained ProgramOneHot commitments disagree with the preprocessing",
            });
        }
    }
    let untrusted_advice_present = !public_io.untrusted_advice.is_empty();
    // The verifier's own input validation doubles as the prover's self-check
    // and produces the normalized `CheckedInputs` the preamble absorbs.
    let checked = validate_inputs_from_parts(
        &preprocessing.verifier,
        public_io,
        config.trace_length,
        config.ram_K,
        config.trace_polynomial_order,
        config.one_hot_config,
        trusted_advice.is_some(),
        untrusted_advice_present,
        false,
    )?;

    let mut transcript = T::new(b"Jolt");
    absorb_transcript_preamble(
        &checked,
        ProofTranscriptConfig {
            rw_config: config.rw_config,
            one_hot_config: config.one_hot_config,
            trace_polynomial_order: config.trace_polynomial_order,
        },
        &mut transcript,
    );

    // The canonical OneHotTrace geometry; the setup's layout digest is what
    // the commitment carries (as legacy does), cross-checked against the
    // protocol-derived canonical digest fail-closed.
    let log_t = config.trace_length.ilog2() as usize;
    let log_k_chunk = config.one_hot_config.committed_chunk_bits();
    let formula_dimensions = crate::stages::formula_dimensions(
        &checked,
        config,
        preprocessing.verifier.program.bytecode_len(),
        JoltRelationId::HammingWeightClaimReduction,
    )?;
    let one_hot_trace_shape = OneHotTraceShape {
        ra_layout: formula_dimensions.ra_layout,
        log_t,
        log_k_chunk,
    };
    let plan = ONE_HOT_TRACE_LAYOUT
        .plan(&one_hot_trace_shape)
        .map_err(|error| VerifierError::FinalOpeningBatchFailed {
            reason: error.to_string(),
        })?;
    let canonical_digest = ONE_HOT_TRACE_LAYOUT
        .layout_digest(&one_hot_trace_shape)
        .map_err(|error| VerifierError::FinalOpeningBatchFailed {
            reason: error.to_string(),
        })?;
    if preprocessing.pcs_setup.default_layout_digest() != canonical_digest {
        return Err(ProverError::Unsupported {
            reason: "the packed setup's layout digest is not the canonical OneHotTrace digest",
        });
    }
    // The setup's declared dimensions must equal the canonical group shape
    // (the verifier enforces the same equalities on its setup before the
    // native opening) — a shape-exact setup with the right digest but the
    // wrong arity would otherwise fail minutes later inside the backend.
    // The untrusted object joins the batch as a precommitted group, so it is
    // committed before the trace: the final commit is conditioned on the frozen
    // profile of every precommitted group.
    let untrusted_advice = if untrusted_advice_present {
        Some(commit_advice_dense::<PCS>(
            jolt_claims::protocols::jolt::JoltAdviceKind::Untrusted,
            &public_io.untrusted_advice,
            public_io.memory_layout.max_untrusted_advice_size as usize,
        )?)
    } else {
        None
    };

    // Canonical public batch order: [UntrustedAdvice, TrustedAdvice, OneHotTrace].
    let precommitted: Vec<(PrecommittedRole, &PCS::Output, &PCS::OpeningHint)> = untrusted_advice
        .as_ref()
        .map(|object| {
            (
                PrecommittedRole::UntrustedAdvice,
                &object.commitment,
                &object.hint,
            )
        })
        .into_iter()
        .chain(trusted_advice.map(|object| {
            (
                PrecommittedRole::TrustedAdvice,
                &object.commitment,
                &object.hint,
            )
        }))
        .collect();
    let required_batch_polys = precommitted.len() + 1;
    if preprocessing.pcs_setup.max_num_vars() != plan.packing().packed_num_vars()
        || preprocessing.pcs_setup.max_num_polys_per_commitment_group() != 1
        || preprocessing.pcs_setup.max_total_batch_polys() < required_batch_polys
        || preprocessing.pcs_setup.one_hot_k() != 1usize << log_k_chunk
    {
        return Err(ProverError::Unsupported {
            reason: "the packed setup's dimensions disagree with the canonical OneHotTrace shape",
        });
    }
    // Resolve the exact grouped row before building the expensive final
    // source: an unschedulable precommit shape fails here, not minutes later.
    if !precommitted.is_empty() {
        PCS::validate_trace_precommits(
            &preprocessing.pcs_setup,
            &precommitted,
            plan.packing().packed_num_vars(),
        )
        .map_err(|error| VerifierError::FinalOpeningVerificationFailed {
            reason: error.to_string(),
        })?;
    }

    let (commitment, hint) =
        tracing::info_span!("akita_main_commit_with_precommitted").in_scope(|| {
            let packed_trace_rows = assemble_one_hot_trace_rows(
                witness,
                &plan,
                formula_dimensions.ra_layout,
                log_k_chunk,
                log_t,
            )?;
            let precommitted_hints = precommitted
                .iter()
                .map(|(_, _, hint)| *hint)
                .collect::<Vec<_>>();
            let committed = if precommitted_hints.is_empty() {
                PCS::commit_trace_one_hot(
                    &preprocessing.pcs_setup,
                    preprocessing.pcs_setup.default_layout_digest(),
                    plan.packing().slot_capacity(),
                    packed_trace_rows,
                )
            } else {
                PCS::commit_trace_one_hot_with_precommitted(
                    &preprocessing.pcs_setup,
                    preprocessing.pcs_setup.default_layout_digest(),
                    plan.packing().slot_capacity(),
                    packed_trace_rows,
                    &precommitted_hints,
                )
            };
            let (commitment, hint) =
                committed.map_err(|error| VerifierError::FinalOpeningVerificationFailed {
                    reason: error.to_string(),
                })?;
            PCS::release_post_commit_residency(&preprocessing.pcs_setup).map_err(
                |error| VerifierError::FinalOpeningVerificationFailed {
                    reason: error.to_string(),
                },
            )?;
            Ok::<_, ProverError<F>>((commitment, hint))
        })?;

    absorb_packed_commitments(
        &commitment,
        untrusted_advice.as_ref().map(|object| &object.commitment),
        trusted_advice.map(|object| &object.commitment),
        preprocessing
            .verifier
            .program
            .committed()
            .map_or(&[][..], |committed| &committed.program_one_hot_commitments),
        &mut transcript,
    );

    Ok(Stage0Output {
        checked,
        transcript,
        commitment,
        hint,
        untrusted_advice,
    })
}
