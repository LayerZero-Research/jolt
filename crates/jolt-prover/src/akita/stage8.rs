//! Akita's final opening: one heterogeneous trusted-advice/main-trace opening,
//! followed by independently pointed untrusted-advice and program objects.

use std::collections::BTreeMap;

use jolt_akita::{GroupOpeningClaim, PrecommittedTraceBatching};
use jolt_claims::protocols::jolt::lattice::packing::{OneHotTraceShape, PrefixPackedObjectPlan};
use jolt_claims::protocols::jolt::lattice::strategy::ONE_HOT_TRACE_LAYOUT;
use jolt_claims::protocols::jolt::{JoltAdviceKind, JoltCommittedPolynomial, JoltRelationId};
use jolt_crypto::VectorCommitment;
use jolt_field::Field;
use jolt_openings::{CommitmentScheme, EvaluationClaim};
use jolt_poly::MultilinearPoly;
use jolt_transcript::{AppendToTranscript, Transcript};
use jolt_verifier::proof::AkitaJointOpeningProof;
use jolt_verifier::stages::stage6b::outputs::Stage6bClearOutput;
use jolt_verifier::stages::stage7::outputs::Stage7ClearOutput;
use jolt_verifier::stages::stage8::packed::{
    leaf_claims, object_leaf_claims, one_hot_trace_packed_claims,
};
use jolt_verifier::stages::stage8::reconstruction::ReconstructionClearOutput;
use jolt_verifier::{CheckedInputs, VerifierError};

use super::witness::{DenseAdviceObject, ProgramOneHot};
use crate::{JoltProverPreprocessing, ProverConfig, ProverError};

fn batch_failed<F: Field>(reason: impl ToString) -> ProverError<F> {
    ProverError::Verifier(VerifierError::FinalOpeningBatchFailed {
        reason: reason.to_string(),
    })
}

fn reduce_auxiliary<F, T>(
    plan: &PrefixPackedObjectPlan,
    leaves: &BTreeMap<JoltCommittedPolynomial, EvaluationClaim<F>>,
    transcript: &mut T,
) -> Result<EvaluationClaim<F>, ProverError<F>>
where
    F: Field,
    T: Transcript<Challenge = F>,
{
    let claims = object_leaf_claims(plan, leaves).map_err(ProverError::Verifier)?;
    let semantic = plan.packed_claims(&claims).map_err(batch_failed::<F>)?;
    plan.packing()
        .reduce_claims(&semantic, transcript)
        .map_err(batch_failed::<F>)
}

fn open_reduced_auxiliary<F, PCS, T, P>(
    polynomial: &P,
    setup: &PCS::ProverSetup,
    hint: PCS::OpeningHint,
    physical: &EvaluationClaim<F>,
    transcript: &mut T,
) -> Result<PCS::Proof, ProverError<F>>
where
    F: Field,
    PCS: CommitmentScheme<Field = F>,
    P: MultilinearPoly<F> + ?Sized,
    T: Transcript<Challenge = F>,
{
    PCS::open(
        polynomial,
        physical.point.as_slice(),
        physical.value,
        setup,
        Some(hint),
        transcript,
    )
    .map_err(batch_failed::<F>)
}

#[expect(clippy::too_many_arguments, reason = "the stage's upstream carriers")]
#[tracing::instrument(skip_all)]
pub fn prove_stage8<F, PCS, VC, T>(
    checked: &CheckedInputs,
    config: &ProverConfig,
    preprocessing: &JoltProverPreprocessing<PCS, VC>,
    one_hot_trace_commitment: &PCS::Output,
    one_hot_trace_hint: PCS::OpeningHint,
    untrusted_advice: Option<&DenseAdviceObject<PCS>>,
    trusted_advice: Option<&DenseAdviceObject<PCS>>,
    program: Option<&ProgramOneHot<PCS>>,
    stage6b: &Stage6bClearOutput<F>,
    stage7: &Stage7ClearOutput<F>,
    reconstruction: &ReconstructionClearOutput<F>,
    transcript: &mut T,
) -> Result<AkitaJointOpeningProof<PCS::Proof>, ProverError<F>>
where
    F: Field,
    PCS: CommitmentScheme<Field = F> + PrecommittedTraceBatching,
    PCS::Output: Clone + AppendToTranscript,
    VC: VectorCommitment<Field = F>,
    T: Transcript<Challenge = F>,
{
    let log_t = checked.trace_length.ilog2() as usize;
    let chunk_width = config.one_hot_config.committed_chunk_bits();
    let formula_dimensions = crate::stages::formula_dimensions(
        checked,
        config,
        preprocessing.verifier.program.bytecode_len(),
        JoltRelationId::HammingWeightClaimReduction,
    )?;
    let plan = ONE_HOT_TRACE_LAYOUT
        .plan(&OneHotTraceShape {
            ra_layout: formula_dimensions.ra_layout,
            log_t,
            log_k_chunk: chunk_width,
        })
        .map_err(batch_failed::<F>)?;

    // Every packed column's single leaf claim, resolved exactly as the
    // verifier resolves them.
    let leaves = leaf_claims(&checked.precommitted, stage6b, stage7, reconstruction)?;

    // OneHotTrace: assemble the shared-point packed statement, reduce it to
    // one physical claim on the transcript, and open it natively from the
    // retained stage-0 commit state (the hint is the committed object; it
    // owns the witness forms and the backend opening data).
    let packed_claims =
        one_hot_trace_packed_claims(&plan, chunk_width, &leaves).map_err(ProverError::Verifier)?;
    let packed_claim = plan
        .packing()
        .reduce_claims(&packed_claims, transcript)
        .map_err(batch_failed::<F>)?;

    // Preserve the protocol's main/untrusted/trusted prefix-reduction order.
    // The untrusted proof itself remains auxiliary and is emitted only after
    // the trusted/main group proof.
    let untrusted_physical = untrusted_advice
        .map(|object| reduce_auxiliary(&object.plan, &leaves, transcript))
        .transpose()?;
    let trusted_physical = trusted_advice
        .map(|object| reduce_auxiliary(&object.plan, &leaves, transcript))
        .transpose()?;

    let main_batch =
        if let (Some(trusted), Some(trusted_claim)) = (trusted_advice, trusted_physical.as_ref()) {
            let trusted_group = GroupOpeningClaim::new(
                trusted.commitment.clone(),
                trusted_claim.point.as_slice().to_vec(),
                vec![trusted_claim.value],
            );
            let main_group = GroupOpeningClaim::new(
                one_hot_trace_commitment.clone(),
                packed_claim.point.as_slice().to_vec(),
                vec![packed_claim.value],
            );
            tracing::info_span!("akita_trusted_main_batched_prove").in_scope(|| {
                PCS::prove_trusted_trace_batch(
                    &preprocessing.pcs_setup,
                    trusted_group,
                    trusted.hint.clone(),
                    main_group,
                    one_hot_trace_hint,
                    transcript,
                )
                .map_err(batch_failed::<F>)
            })?
        } else {
            tracing::info_span!(
                "CommitmentScheme::open_batch_from_hint",
                packed_num_vars = plan.packing().packed_num_vars()
            )
            .in_scope(|| {
                PCS::open_batch_from_hint(
                    packed_claim.point.as_slice(),
                    std::slice::from_ref(&packed_claim.value),
                    &preprocessing.pcs_setup,
                    one_hot_trace_hint,
                    transcript,
                )
            })
            .map_err(batch_failed::<F>)?
        };

    let mut auxiliary = Vec::new();
    if let (Some(object), Some(claim)) = (untrusted_advice, untrusted_physical.as_ref()) {
        auxiliary.push(
            tracing::info_span!("akita_dense_advice_open", kind = ?JoltAdviceKind::Untrusted)
                .in_scope(|| {
                    open_reduced_auxiliary::<F, PCS, T, _>(
                        &object.polynomial,
                        &object.setup,
                        object.hint.clone(),
                        claim,
                        transcript,
                    )
                })?,
        );
    }
    if let Some(program) = program {
        for object in &program.objects {
            let physical = reduce_auxiliary(&object.plan, &leaves, transcript)?;
            auxiliary.push(open_reduced_auxiliary::<F, PCS, T, _>(
                &object.witness,
                &object.setup,
                object.hint.clone(),
                &physical,
                transcript,
            )?);
        }
    }

    Ok(AkitaJointOpeningProof::new(main_batch, auxiliary))
}
