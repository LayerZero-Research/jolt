//! Top-level Jolt proof verification.
//!
//! [`verify`] orchestrates the full verification pipeline:
//!
//! 1. **S1 (Spartan)**: Verify R1CS satisfiability via [`UniformSpartanVerifier`]
//! 2. **S2–S7**: For each stage descriptor, verify the batched sumcheck proof,
//!    check claimed polynomial evaluations via expression evaluation, and
//!    accumulate opening claims
//! 3. **S8 (Openings)**: Reduce all opening claims via RLC and verify PCS
//!    opening proofs

use jolt_field::Field;
use jolt_openings::{
    AdditivelyHomomorphic, OpeningReduction, OpeningsError, RlcReduction, VerifierClaim,
};
use jolt_poly::EqPolynomial;
use jolt_spartan::{SpartanError, UniformSpartanKey, UniformSpartanProof, UniformSpartanVerifier};
use jolt_sumcheck::{BatchedSumcheckVerifier, SumcheckClaim};
use jolt_transcript::{AppendToTranscript, Transcript};
use jolt_verifier_backend::{evaluate_expr, helpers::eq_eval, FieldBackend};

use crate::error::JoltError;
use crate::key::JoltVerifyingKey;
use crate::proof::{JoltProof, SumcheckStageProof};
use crate::stage::StageDescriptor;

/// Verifies the uniform Spartan R1CS proof (PIOP only — no PCS).
///
/// The returned `(r_x, r_y)` are the outer and inner sumcheck challenge
/// points, needed by downstream stages to construct eq-weighted sumcheck
/// claims.
///
/// The caller must append the witness commitment to the transcript before
/// calling this, and verify the witness opening proof afterward.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all, name = "verify_spartan")]
pub fn verify_spartan<F, T>(
    key: &UniformSpartanKey<F>,
    proof: &UniformSpartanProof<F>,
    transcript: &mut T,
) -> Result<(Vec<F>, Vec<F>), SpartanError>
where
    F: Field,
    T: Transcript<Challenge = F>,
{
    UniformSpartanVerifier::verify_with_challenges(key, proof, transcript)
}

/// Verifies batch PCS opening proofs (stage 8).
///
/// Reduces all accumulated opening claims via random linear combination,
/// then verifies each reduced claim against the corresponding PCS proof.
#[tracing::instrument(skip_all, name = "verify_openings")]
pub fn verify_openings<PCS, T>(
    claims: Vec<VerifierClaim<PCS::Field, PCS::Output>>,
    opening_proofs: &[PCS::Proof],
    verifier_setup: &PCS::VerifierSetup,
    transcript: &mut T,
) -> Result<(), JoltError>
where
    PCS: AdditivelyHomomorphic,
    T: Transcript<Challenge = PCS::Field>,
{
    let reduced = <RlcReduction as OpeningReduction<PCS>>::reduce_verifier(claims, &(), transcript)
        .map_err(JoltError::Opening)?;

    if reduced.len() != opening_proofs.len() {
        return Err(JoltError::Opening(OpeningsError::VerificationFailed));
    }

    for (claim, proof) in reduced.iter().zip(opening_proofs.iter()) {
        PCS::verify(
            &claim.commitment,
            &claim.point,
            claim.eval,
            proof,
            verifier_setup,
            transcript,
        )
        .map_err(JoltError::Opening)?;
    }

    Ok(())
}

/// Verifies one sumcheck stage using its descriptor.
///
/// Returns the opening claims produced by this stage.
fn verify_stage<F, C, T>(
    stage_index: usize,
    desc: &StageDescriptor<F>,
    stage_proof: &SumcheckStageProof<F>,
    commitments: &[C],
    transcript: &mut T,
) -> Result<Vec<VerifierClaim<F, C>>, JoltError>
where
    F: Field,
    C: Clone,
    T: Transcript<Challenge = F>,
{
    let _span = tracing::info_span!("verify_stage", stage = stage_index).entered();

    let claims = [SumcheckClaim {
        num_vars: desc.num_vars,
        degree: desc.degree,
        claimed_sum: desc.claimed_sum,
    }];

    let (final_eval, challenges) =
        BatchedSumcheckVerifier::verify(&claims, &stage_proof.sumcheck_proof, transcript).map_err(
            |e| JoltError::StageVerification {
                stage: stage_index,
                reason: e.to_string(),
            },
        )?;

    let eval_point: Vec<F> = if desc.reverse_challenges {
        challenges.iter().rev().copied().collect()
    } else {
        challenges.clone()
    };

    if stage_proof.evaluations.len() != desc.commitment_indices.len() {
        return Err(JoltError::InvalidProof(format!(
            "stage {stage_index}: expected {} evaluations, got {}",
            desc.commitment_indices.len(),
            stage_proof.evaluations.len(),
        )));
    }

    // Check: eq(eq_point, eval_point) × g(evaluations, challenges) == final_eval
    let eq_eval = EqPolynomial::new(desc.eq_point.clone()).evaluate(&eval_point);

    let g_eval: F = desc
        .output_expr
        .evaluate(&stage_proof.evaluations, &desc.output_challenges);

    let expected = eq_eval * g_eval;
    if expected != final_eval {
        return Err(JoltError::EvaluationMismatch {
            stage: stage_index,
            reason: format!("eq * g = {expected:?}, final_eval = {final_eval:?}"),
        });
    }

    desc.commitment_indices
        .iter()
        .zip(stage_proof.evaluations.iter())
        .map(|(&idx, &eval)| {
            let commitment = commitments.get(idx).ok_or_else(|| {
                JoltError::InvalidProof(format!(
                    "stage {stage_index}: commitment index {idx} out of bounds ({})",
                    commitments.len(),
                ))
            })?;
            Ok(VerifierClaim {
                commitment: commitment.clone(),
                point: eval_point.clone(),
                eval,
            })
        })
        .collect()
}

/// Verifies a complete Jolt proof.
///
/// Orchestrates the full verification pipeline:
///
/// 1. Append witness commitment to Fiat-Shamir transcript
/// 2. **S1**: Verify uniform Spartan R1CS proof, extract `(r_x, r_y)`
/// 3. Build stage descriptors via `build_descriptors(r_x, r_y)`
/// 4. **S2–S7**: For each descriptor, verify sumcheck and accumulate opening claims
/// 5. Add witness opening claim (from Spartan)
/// 6. **S8**: Reduce opening claims via RLC and verify PCS opening proofs
///
/// The `build_descriptors` closure receives the Spartan challenge vectors and
/// transcript, and returns [`StageDescriptor`]s for S2–S7. Transcript access
/// allows squeezing batching challenges (e.g., γ) to match the prover's
/// Fiat-Shamir state. Each descriptor fully encodes the stage's verification
/// logic via its output expression — no per-stage trait implementations needed.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all, name = "verify")]
pub fn verify<PCS, T>(
    proof: &JoltProof<PCS::Field, PCS>,
    vk: &JoltVerifyingKey<PCS::Field, PCS>,
    build_descriptors: impl FnOnce(
        &[PCS::Field],
        &[PCS::Field],
        &mut T,
    ) -> Vec<StageDescriptor<PCS::Field>>,
    transcript: &mut T,
) -> Result<(Vec<PCS::Field>, Vec<PCS::Field>), JoltError>
where
    PCS: AdditivelyHomomorphic,
    T: Transcript<Challenge = PCS::Field>,
{
    // Append witness commitment to transcript (matches prover's commit step).
    transcript.append_bytes(format!("{:?}", proof.witness_commitment).as_bytes());

    let (r_x, r_y) = verify_spartan(&vk.spartan_key, &proof.spartan_proof, transcript)?;

    let descriptors = build_descriptors(&r_x, &r_y, transcript);

    if proof.stage_proofs.len() != descriptors.len() {
        return Err(JoltError::InvalidProof(format!(
            "expected {} stage proofs, got {}",
            descriptors.len(),
            proof.stage_proofs.len(),
        )));
    }

    let mut all_opening_claims: Vec<VerifierClaim<PCS::Field, PCS::Output>> = Vec::new();

    for (i, (desc, stage_proof)) in descriptors.iter().zip(&proof.stage_proofs).enumerate() {
        let new_claims = verify_stage(i + 2, desc, stage_proof, &proof.commitments, transcript)?;

        // Fiat-Shamir: absorb opening claim evaluations before the next
        // stage derives its challenges. Must match the prover's flush.
        for claim in &new_claims {
            claim.eval.append_to_transcript(transcript);
        }

        all_opening_claims.extend(new_claims);
    }

    // Witness opening claim from Spartan — must be added last to match prover ordering.
    all_opening_claims.push(VerifierClaim {
        commitment: proof.witness_commitment.clone(),
        point: r_y.clone(),
        eval: proof.spartan_proof.witness_eval,
    });

    verify_openings::<PCS, T>(
        all_opening_claims,
        &proof.opening_proofs[..],
        &vk.pcs_setup,
        transcript,
    )?;

    Ok((r_x, r_y))
}

/// Backend-aware sibling of [`verify_stage`].
///
/// Runs the same algebraic check as [`verify_stage`], but every field
/// operation flows through `backend`. Concretely:
///
/// - Sumcheck verification dispatches to
///   [`BatchedSumcheckVerifier::verify_with_backend`].
/// - The eq-polynomial evaluation runs through [`eq_eval`].
/// - The output expression evaluates through [`evaluate_expr`].
/// - The final equality `eq * g == final_eval` is asserted via
///   [`FieldBackend::assert_eq`].
///
/// Returns `(claims, evals_w)` where `claims` carries the native commitments +
/// raw `F` evals (consumed by PCS) and `evals_w` carries the backend handle for
/// each eval (consumed by [`RlcReduction::reduce_verifier_with_backend`]).
#[allow(clippy::type_complexity)]
fn verify_stage_with_backend<B, C, T>(
    backend: &mut B,
    stage_index: usize,
    desc: &StageDescriptor<B::F>,
    stage_proof: &SumcheckStageProof<B::F>,
    commitments: &[C],
    transcript: &mut T,
) -> Result<(Vec<VerifierClaim<B::F, C>>, Vec<B::Scalar>), JoltError>
where
    B: FieldBackend,
    C: Clone,
    T: Transcript<Challenge = B::F>,
{
    let _span = tracing::info_span!("verify_stage_with_backend", stage = stage_index).entered();

    let claims = [SumcheckClaim {
        num_vars: desc.num_vars,
        degree: desc.degree,
        claimed_sum: desc.claimed_sum,
    }];

    let (final_eval_w, challenges_w, challenges_f) = BatchedSumcheckVerifier::verify_with_backend(
        backend,
        &claims,
        &stage_proof.sumcheck_proof,
        transcript,
    )
    .map_err(|e| JoltError::StageVerification {
        stage: stage_index,
        reason: e.to_string(),
    })?;

    if stage_proof.evaluations.len() != desc.commitment_indices.len() {
        return Err(JoltError::InvalidProof(format!(
            "stage {stage_index}: expected {} evaluations, got {}",
            desc.commitment_indices.len(),
            stage_proof.evaluations.len(),
        )));
    }

    let eval_point_w: Vec<B::Scalar> = if desc.reverse_challenges {
        challenges_w.iter().rev().cloned().collect()
    } else {
        challenges_w
    };
    let eval_point_f: Vec<B::F> = if desc.reverse_challenges {
        challenges_f.iter().rev().copied().collect()
    } else {
        challenges_f
    };

    let eq_point_w: Vec<B::Scalar> = desc
        .eq_point
        .iter()
        .map(|p| backend.wrap_public(*p, "eq_point"))
        .collect();
    let eq_eval_w = eq_eval(backend, &eq_point_w, &eval_point_w);

    let openings_w: Vec<B::Scalar> = stage_proof
        .evaluations
        .iter()
        .map(|e| backend.wrap_proof(*e, "stage_eval"))
        .collect();
    let challenges_for_expr_w: Vec<B::Scalar> = desc
        .output_challenges
        .iter()
        .map(|c| backend.wrap_public(*c, "output_challenge"))
        .collect();
    let g_eval_w = evaluate_expr(
        backend,
        &desc.output_expr,
        &openings_w,
        &challenges_for_expr_w,
    );

    let expected_w = backend.mul(&eq_eval_w, &g_eval_w);
    backend
        .assert_eq(&expected_w, &final_eval_w, "stage final eval check")
        .map_err(|e| JoltError::EvaluationMismatch {
            stage: stage_index,
            reason: e.to_string(),
        })?;

    let claims_out: Vec<VerifierClaim<B::F, C>> = desc
        .commitment_indices
        .iter()
        .zip(stage_proof.evaluations.iter())
        .map(|(&idx, &eval)| {
            let commitment = commitments.get(idx).ok_or_else(|| {
                JoltError::InvalidProof(format!(
                    "stage {stage_index}: commitment index {idx} out of bounds ({})",
                    commitments.len(),
                ))
            })?;
            Ok(VerifierClaim {
                commitment: commitment.clone(),
                point: eval_point_f.clone(),
                eval,
            })
        })
        .collect::<Result<Vec<_>, JoltError>>()?;

    Ok((claims_out, openings_w))
}

/// Backend-aware sibling of [`verify_openings`].
///
/// Routes RLC reduction of opening evaluations through `backend` via
/// [`RlcReduction::reduce_verifier_with_backend`]. Commitment combination and
/// PCS opening verification stay native (group ops belong to a future
/// `GroupBackend`).
///
/// `claims` carries the native side; `evals_w` is parallel to `claims` and
/// carries each eval as a backend handle. Both must have the same length.
fn verify_openings_with_backend<B, PCS, T>(
    backend: &mut B,
    claims: Vec<VerifierClaim<B::F, PCS::Output>>,
    evals_w: Vec<B::Scalar>,
    opening_proofs: &[PCS::Proof],
    verifier_setup: &PCS::VerifierSetup,
    transcript: &mut T,
) -> Result<(), JoltError>
where
    B: FieldBackend,
    PCS: AdditivelyHomomorphic<Field = B::F>,
    T: Transcript<Challenge = B::F>,
{
    let (reduced, _reduced_evals_w) = RlcReduction::reduce_verifier_with_backend::<B, PCS, T>(
        backend, claims, evals_w, transcript,
    )
    .map_err(JoltError::Opening)?;

    if reduced.len() != opening_proofs.len() {
        return Err(JoltError::Opening(OpeningsError::VerificationFailed));
    }

    for (claim, proof) in reduced.iter().zip(opening_proofs.iter()) {
        PCS::verify(
            &claim.commitment,
            &claim.point,
            claim.eval,
            proof,
            verifier_setup,
            transcript,
        )
        .map_err(JoltError::Opening)?;
    }

    Ok(())
}

/// Backend-aware verification entry point.
///
/// Mirrors [`verify`] but routes every algebraic check through a pluggable
/// [`FieldBackend`]:
///
/// - **`Native`** is a zero-overhead pass-through (matches [`verify`] exactly).
/// - **`Tracing`** records the entire verifier into an [`AstGraph`] suitable
///   for recursion lowering, R1CS generation, or Lean export.
///
/// What flows through the backend in this version:
///
/// 1. **S1 (Spartan)** — outer + inner sumchecks, eq evaluations, sparse
///    matrix MLE evaluation, and final consistency checks all dispatch through
///    [`UniformSpartanVerifier::verify_with_backend`].
/// 2. **S2–S7** — per-stage sumcheck + expression evaluation flow through
///    [`BatchedSumcheckVerifier::verify_with_backend`] and [`evaluate_expr`].
/// 3. **S8 (Opening RLC reduction)** — scalar combination of opening evals
///    flows through [`RlcReduction::reduce_verifier_with_backend`]. The
///    commitment combination and final [`PCS::verify`] still use the native
///    group operations; lifting those into a `GroupBackend` is Phase 2.
///
/// # Errors
///
/// See [`verify`] — same error semantics.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all, name = "verify_with_backend")]
pub fn verify_with_backend<B, PCS, T>(
    backend: &mut B,
    proof: &JoltProof<PCS::Field, PCS>,
    vk: &JoltVerifyingKey<PCS::Field, PCS>,
    build_descriptors: impl FnOnce(
        &[PCS::Field],
        &[PCS::Field],
        &mut T,
    ) -> Vec<StageDescriptor<PCS::Field>>,
    transcript: &mut T,
) -> Result<(Vec<PCS::Field>, Vec<PCS::Field>), JoltError>
where
    PCS: AdditivelyHomomorphic,
    B: FieldBackend<F = PCS::Field>,
    T: Transcript<Challenge = PCS::Field>,
{
    transcript.append_bytes(format!("{:?}", proof.witness_commitment).as_bytes());

    let (_r_x_w, r_y_w, r_x, r_y, witness_eval_w) = UniformSpartanVerifier::verify_with_backend(
        backend,
        &vk.spartan_key,
        &proof.spartan_proof,
        transcript,
    )?;

    let descriptors = build_descriptors(&r_x, &r_y, transcript);

    if proof.stage_proofs.len() != descriptors.len() {
        return Err(JoltError::InvalidProof(format!(
            "expected {} stage proofs, got {}",
            descriptors.len(),
            proof.stage_proofs.len(),
        )));
    }

    let mut all_opening_claims: Vec<VerifierClaim<PCS::Field, PCS::Output>> = Vec::new();
    let mut all_evals_w: Vec<B::Scalar> = Vec::new();

    for (i, (desc, stage_proof)) in descriptors.iter().zip(&proof.stage_proofs).enumerate() {
        let (new_claims, new_evals_w) = verify_stage_with_backend(
            backend,
            i + 2,
            desc,
            stage_proof,
            &proof.commitments,
            transcript,
        )?;

        for claim in &new_claims {
            claim.eval.append_to_transcript(transcript);
        }

        all_opening_claims.extend(new_claims);
        all_evals_w.extend(new_evals_w);
    }

    let _ = r_y_w;
    all_opening_claims.push(VerifierClaim {
        commitment: proof.witness_commitment.clone(),
        point: r_y.clone(),
        eval: proof.spartan_proof.witness_eval,
    });
    all_evals_w.push(witness_eval_w);

    verify_openings_with_backend::<B, PCS, T>(
        backend,
        all_opening_claims,
        all_evals_w,
        &proof.opening_proofs[..],
        &vk.pcs_setup,
        transcript,
    )?;

    Ok((r_x, r_y))
}
