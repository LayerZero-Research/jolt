//! Native Akita multi-group root fold for Jolt's fused stage-8 opening.
//!
//! Jolt reduces every committed group — the `OneHotTrace` columns plus the
//! precommitted advice/program objects — to evaluation claims at slices of one
//! shared reduction point `r*` (see [`jolt_openings::reduce_packed_openings`]).
//! This module discharges all of those claims in a **single** native
//! `batched_prove`/`batched_verify` over one multi-group root commitment,
//! rather than one native open per group.
//!
//! The native contract (verified against akita `scheme/tests/onehot.rs`):
//!
//! - Precommitted groups are committed under
//!   [`akita_config::PrecommittedCommitmentConfig`], which freezes each group's
//!   exact one-hot layout. They must have exactly one polynomial and at most
//!   the final group's `num_vars`.
//! - The final (widest) group — the trace — is committed via
//!   [`commit_final_one_hot_group`], which takes the precommitted layout keys in
//!   transcript order so the multi-group schedule can be resolved.
//! - Every group opens at a **prefix** of one shared point in backend
//!   coordinates. Jolt binds narrower groups at *suffixes* of `r*`; the one-hot
//!   backend reverses the whole point, so a Jolt-suffix of length `nv` becomes a
//!   backend-prefix of length `nv` (`reverse(r*)[..nv] == reverse(r*[pad..])`).
//! - Group order is transcript-bound: precommitteds first (in a fixed canonical
//!   order), then the trace as the final group.

use akita_config::PrecommittedCommitmentConfig;
use akita_pcs::{AkitaCommitmentScheme, AkitaTranscript};
use akita_prover::{ProverOpeningData, RootPolyMeta};
use akita_types::{
    BasisMode, OpeningClaims, PointVariableSelection, PolynomialGroupClaims, PolynomialGroupLayout,
};
use jolt_openings::{MultiGroupOpeningClaim, MultiGroupVerify, OpeningsError};
use jolt_poly::OneHotPolynomial;
use jolt_transcript::Transcript;
use tracing::info_span;

use crate::adapters::{
    akita_error, append_batch_statement, append_verifier_setup, backend_stack,
    bridge_jolt_statement_challenge, commit_failed, invalid_batch, one_hot_polynomial,
    prove_failed, reverse_point, serialize_akita, with_backend_pool, AkitaBackendCommitment,
    AkitaBackendHint, AkitaBackendOneHotPoly, AkitaBatchProof, AkitaCommitment, AkitaField,
    AkitaHintPolynomials, AkitaLayoutDigest, AkitaOneHotK256BackendScheme, AkitaProverHint,
    AkitaProverSetup, AkitaVerifierSetup, AKITA_ONE_HOT_K256,
};
use crate::configs::JoltD64OneHotK256;

/// The scheme instance whose commit selection is routed through the frozen
/// exact one-hot layout of a standalone precommitted group.
type PrecommittedOneHotK256Scheme =
    AkitaCommitmentScheme<PrecommittedCommitmentConfig<JoltD64OneHotK256>>;

/// The transcript label of the fused multi-group backend proof; distinct from
/// the single-group `jolt-akita/batch` so a single-group and a multi-group
/// opening never share a Fiat-Shamir domain.
const MULTI_GROUP_LABEL: &[u8] = b"jolt-akita/multi-group-batch";

/// Validates and converts a group's one-hot columns to the backend
/// representation, returning `(num_vars, backend_polynomials)`.
fn backend_one_hot_group(
    polynomials: &[OneHotPolynomial],
) -> Result<(usize, Vec<AkitaBackendOneHotPoly>), OpeningsError> {
    let first = polynomials
        .first()
        .ok_or_else(|| invalid_batch("Akita commitment group must contain a polynomial"))?;
    let num_vars = first.num_vars();
    let backend_polynomials = polynomials
        .iter()
        .map(|polynomial| {
            if polynomial.num_vars() != num_vars {
                return Err(invalid_batch(format!(
                    "Akita commitment group mixes {}-variable and {num_vars}-variable polynomials",
                    polynomial.num_vars()
                )));
            }
            one_hot_polynomial(polynomial, AKITA_ONE_HOT_K256)?.ok_or_else(|| {
                invalid_batch("Akita multi-group commit requires row-major K=256 one-hot columns")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((num_vars, backend_polynomials))
}

/// Commits a standalone precommitted one-hot group under the exact-layout
/// precommit config, returning the adapter commitment/hint plus the frozen
/// [`PolynomialGroupLayout`] key the final-group commit and the fused prove
/// consume. The group may be narrower than the shared setup (`num_vars <=
/// setup.max_num_vars()`), unlike the exact-arity single-group commit paths.
pub fn commit_precommitted_one_hot_group(
    setup: &AkitaProverSetup,
    layout_digest: AkitaLayoutDigest,
    polynomials: &[OneHotPolynomial],
) -> Result<(AkitaCommitment, AkitaProverHint, PolynomialGroupLayout), OpeningsError> {
    let (num_vars, backend_polynomials) = backend_one_hot_group(polynomials)?;
    if num_vars > setup.max_num_vars() {
        return Err(invalid_batch(format!(
            "Akita precommitted group has {num_vars} variables but the shared setup supports {}",
            setup.max_num_vars()
        )));
    }
    let (backend_prover_setup, prepared_backend_setup) = setup.one_hot_backend()?;
    let stack = backend_stack(backend_prover_setup, prepared_backend_setup)?;
    let (backend_commitment, backend_hint) = with_backend_pool(|| {
        PrecommittedOneHotK256Scheme::batched_commit(
            backend_prover_setup,
            &backend_polynomials,
            &stack,
        )
    })
    .map_err(commit_failed)?;
    let key = PolynomialGroupLayout::new(num_vars, polynomials.len());
    let (commitment, hint) = package_one_hot_commitment(
        layout_digest,
        num_vars,
        backend_commitment,
        backend_hint,
        backend_polynomials,
    )?;
    Ok((commitment, hint, key))
}

/// Commits the final (widest) one-hot group — the trace — as the head of a
/// multi-group root commitment. `precommitteds` are the precommitted groups'
/// layout keys **in transcript order**; the backend re-freezes them internally
/// to resolve the multi-group schedule. Errors if `precommitteds` is empty
/// (that is the single-group path, not this one).
pub fn commit_final_one_hot_group(
    setup: &AkitaProverSetup,
    layout_digest: AkitaLayoutDigest,
    polynomials: &[OneHotPolynomial],
    precommitteds: Vec<PolynomialGroupLayout>,
) -> Result<(AkitaCommitment, AkitaProverHint), OpeningsError> {
    if precommitteds.is_empty() {
        return Err(invalid_batch(
            "Akita final-group commit requires at least one precommitted group",
        ));
    }
    let (num_vars, backend_polynomials) = backend_one_hot_group(polynomials)?;
    if num_vars != setup.max_num_vars() {
        return Err(invalid_batch(format!(
            "Akita final group has {num_vars} variables but the shared setup dimension is {}",
            setup.max_num_vars()
        )));
    }
    let (backend_prover_setup, prepared_backend_setup) = setup.one_hot_backend()?;
    let stack = backend_stack(backend_prover_setup, prepared_backend_setup)?;
    let (backend_commitment, backend_hint) = with_backend_pool(|| {
        AkitaOneHotK256BackendScheme::commit_final_group(
            backend_prover_setup,
            &backend_polynomials,
            &stack,
            precommitteds,
        )
    })
    .map_err(commit_failed)?;
    package_one_hot_commitment(
        layout_digest,
        num_vars,
        backend_commitment,
        backend_hint,
        backend_polynomials,
    )
}

/// Wraps a one-hot backend commitment and its opening data into the adapter's
/// commitment/hint pair. Mirrors `AkitaScheme::package_commitment` for the
/// one-hot flavor (kept here so the crate's private packaging stays one place
/// per module; both derive the flavor/count from the hint polynomials).
fn package_one_hot_commitment(
    layout_digest: AkitaLayoutDigest,
    num_vars: usize,
    backend_commitment: AkitaBackendCommitment,
    backend_hint: AkitaBackendHint,
    backend_polynomials: Vec<AkitaBackendOneHotPoly>,
) -> Result<(AkitaCommitment, AkitaProverHint), OpeningsError> {
    crate::scheme::AkitaScheme::package_commitment(
        layout_digest,
        num_vars,
        backend_commitment,
        backend_hint,
        AkitaHintPolynomials::OneHot(backend_polynomials.into()),
    )
}

/// One commitment group of a fused multi-group opening, prover side.
pub struct MultiGroupProverGroup {
    pub commitment: AkitaCommitment,
    pub num_vars: usize,
    pub evaluations: Vec<AkitaField>,
    pub hint: AkitaProverHint,
}

/// One commitment group of a fused multi-group opening, verifier side.
pub struct MultiGroupVerifierGroup<'a> {
    pub commitment: &'a AkitaCommitment,
    pub num_vars: usize,
    pub evaluations: &'a [AkitaField],
}

/// Binds the verifier setup and every group's statement into Jolt's transcript
/// in group order, then bridges one Jolt challenge into a fresh Akita
/// transcript so the single fused backend proof is bound to everything Jolt
/// observed. Prover and verifier call this identically.
fn bind_multi_group_transcripts<T>(
    transcript: &mut T,
    verifier_setup: &AkitaVerifierSetup,
    group_commitments: &[&AkitaCommitment],
    group_evaluations: &[&[AkitaField]],
    shared_point: &[AkitaField],
) -> (AkitaTranscript<AkitaField>, Vec<u8>)
where
    T: Transcript<Challenge = AkitaField>,
{
    {
        let _span = info_span!("multi_group::append_setup_and_statements").entered();
        append_verifier_setup(
            transcript,
            verifier_setup,
            crate::adapters::AkitaBackendFlavor::OneHot,
        );
        for (commitment, evaluations) in group_commitments.iter().zip(group_evaluations) {
            let statement: Vec<_> = evaluations
                .iter()
                .map(|value| jolt_openings::VerifierOpeningClaim {
                    commitment: (*commitment).clone(),
                    evaluation: jolt_openings::EvaluationClaim::new(shared_point.to_vec(), *value),
                })
                .collect();
            append_batch_statement(transcript, &statement, commitment, shared_point);
        }
    }
    let mut akita_transcript = AkitaTranscript::<AkitaField>::new(MULTI_GROUP_LABEL);
    let statement_bridge = bridge_jolt_statement_challenge(transcript, &mut akita_transcript);
    (akita_transcript, statement_bridge)
}

/// Validates the group shapes shared by the prove and verify paths: nonempty,
/// canonical order (precommitteds narrower, final group last and widest), and
/// the final group spanning the whole shared point.
fn validate_group_shapes(
    group_num_vars: &[usize],
    shared_point_len: usize,
) -> Result<(), OpeningsError> {
    let Some((final_num_vars, precommitted)) = group_num_vars.split_last() else {
        return Err(invalid_batch(
            "Akita multi-group opening requires at least one commitment group",
        ));
    };
    if precommitted.is_empty() {
        return Err(invalid_batch(
            "Akita multi-group opening requires at least two commitment groups",
        ));
    }
    if *final_num_vars != shared_point_len {
        return Err(invalid_batch(format!(
            "Akita final group has {final_num_vars} variables but the shared point has {shared_point_len}"
        )));
    }
    for num_vars in precommitted {
        if *num_vars > shared_point_len {
            return Err(invalid_batch(format!(
                "Akita precommitted group has {num_vars} variables but the shared point has {shared_point_len}"
            )));
        }
    }
    Ok(())
}

fn validate_prover_group(
    setup: &AkitaProverSetup,
    group: &MultiGroupProverGroup,
) -> Result<(), OpeningsError> {
    if group.hint.commitment != group.commitment {
        return Err(invalid_batch(
            "Akita prover hint does not match the multi-group statement commitment",
        ));
    }
    if group.num_vars != group.commitment.num_vars {
        return Err(invalid_batch(format!(
            "Akita multi-group claim has {} variables but its commitment has {}",
            group.num_vars, group.commitment.num_vars
        )));
    }
    if group.commitment.backend_flavor != crate::adapters::AkitaBackendFlavor::OneHot
        || group.commitment.one_hot_k != AKITA_ONE_HOT_K256
        || group.commitment.layout_digest != setup.default_layout_digest()
    {
        return Err(invalid_batch(
            "Akita multi-group claim requires a canonical K=256 one-hot commitment",
        ));
    }
    if group.evaluations.len() != group.commitment.poly_count {
        return Err(invalid_batch(format!(
            "Akita multi-group claim has {} evaluations for {} commitment slots",
            group.evaluations.len(),
            group.commitment.poly_count
        )));
    }
    let AkitaHintPolynomials::OneHot(polynomials) = &group.hint.polynomials else {
        return Err(invalid_batch(
            "Akita multi-group opening requires one-hot prover hints",
        ));
    };
    if polynomials.len() != group.commitment.poly_count {
        return Err(invalid_batch(format!(
            "Akita group hint has {} polynomials for {} commitment slots",
            polynomials.len(),
            group.commitment.poly_count
        )));
    }
    for polynomial in polynomials.iter() {
        if RootPolyMeta::<AkitaField>::num_vars(polynomial) != group.num_vars {
            return Err(invalid_batch(format!(
                "Akita group hint polynomial has {} variables but the group has {}",
                RootPolyMeta::<AkitaField>::num_vars(polynomial),
                group.num_vars
            )));
        }
    }
    Ok(())
}

fn validate_verifier_group(
    setup: &AkitaVerifierSetup,
    group: &MultiGroupVerifierGroup<'_>,
) -> Result<(), OpeningsError> {
    if group.num_vars != group.commitment.num_vars {
        return Err(invalid_batch(format!(
            "Akita multi-group claim has {} variables but its commitment has {}",
            group.num_vars, group.commitment.num_vars
        )));
    }
    if group.evaluations.len() != group.commitment.poly_count {
        return Err(invalid_batch(format!(
            "Akita multi-group claim has {} evaluations for {} commitment slots",
            group.evaluations.len(),
            group.commitment.poly_count
        )));
    }
    if group.commitment.backend_flavor != crate::adapters::AkitaBackendFlavor::OneHot
        || group.commitment.one_hot_k != AKITA_ONE_HOT_K256
        || group.commitment.layout_digest != setup.default_layout_digest()
    {
        return Err(invalid_batch(
            "Akita multi-group claim requires a canonical K=256 one-hot commitment",
        ));
    }
    if group.num_vars > setup.max_num_vars()
        || group.commitment.poly_count > setup.max_num_polys_per_commitment_group()
    {
        return Err(invalid_batch(
            "Akita multi-group claim exceeds the verifier setup",
        ));
    }
    Ok(())
}

/// Proves a fused multi-group opening: one native `batched_prove` over all
/// groups at prefix slices of `reverse(shared_point)`. `groups` are in the
/// canonical fused order `[precommitteds…, trace-final]`; the final group must
/// span the whole shared point.
pub fn multi_group_prove_one_hot<T>(
    setup: &AkitaProverSetup,
    shared_point: &[AkitaField],
    groups: Vec<MultiGroupProverGroup>,
    transcript: &mut T,
) -> Result<AkitaBatchProof, OpeningsError>
where
    T: Transcript<Challenge = AkitaField>,
{
    let group_num_vars: Vec<usize> = groups.iter().map(|group| group.num_vars).collect();
    validate_group_shapes(&group_num_vars, shared_point.len())?;
    for group in &groups {
        validate_prover_group(setup, group)?;
    }

    let _span = info_span!(
        "multi_group::prove",
        num_groups = groups.len(),
        num_vars = shared_point.len(),
    )
    .entered();

    // Decompose each group into its backend commitment, backend hint, and the
    // backend one-hot witnesses (kept alive in `hint_polys` for borrowing).
    let mut backend_commitments = Vec::with_capacity(groups.len());
    let mut backend_hints = Vec::with_capacity(groups.len());
    let mut hint_polys: Vec<std::sync::Arc<[AkitaBackendOneHotPoly]>> =
        Vec::with_capacity(groups.len());
    let mut evaluations = Vec::with_capacity(groups.len());
    let mut jolt_commitments = Vec::with_capacity(groups.len());
    for group in groups {
        let (backend_commitment, backend_hint) = group
            .hint
            .backend
            .ok_or_else(|| invalid_batch("Akita prover hint is missing backend opening data"))?;
        let AkitaHintPolynomials::OneHot(polys) = group.hint.polynomials else {
            return Err(invalid_batch(
                "Akita multi-group opening requires one-hot prover hints",
            ));
        };
        backend_commitments.push(backend_commitment);
        backend_hints.push(backend_hint);
        hint_polys.push(polys);
        evaluations.push(group.evaluations);
        jolt_commitments.push(group.commitment);
    }

    let group_commitment_refs: Vec<&AkitaCommitment> = jolt_commitments.iter().collect();
    let group_eval_refs: Vec<&[AkitaField]> = evaluations.iter().map(Vec::as_slice).collect();
    let (mut akita_transcript, statement_bridge) = bind_multi_group_transcripts(
        transcript,
        &setup.verifier,
        &group_commitment_refs,
        &group_eval_refs,
        shared_point,
    );

    let backend_point = reverse_point(shared_point);
    let point_len = backend_point.len();
    let mut prover_groups = Vec::with_capacity(backend_commitments.len());
    for ((commitment, num_vars), evals) in backend_commitments
        .into_iter()
        .zip(&group_num_vars)
        .zip(&evaluations)
    {
        prover_groups.push(
            PolynomialGroupClaims::new(
                PointVariableSelection::prefix(*num_vars, point_len).map_err(akita_error)?,
                evals.clone(),
                commitment,
            )
            .map_err(akita_error)?,
        );
    }
    let claims = OpeningClaims::from_groups(backend_point, prover_groups).map_err(akita_error)?;

    let poly_refs_per_group: Vec<Vec<&AkitaBackendOneHotPoly>> = hint_polys
        .iter()
        .map(|polys| polys.iter().collect())
        .collect();
    let polynomials: Vec<&[&AkitaBackendOneHotPoly]> =
        poly_refs_per_group.iter().map(Vec::as_slice).collect();
    let opening_data =
        ProverOpeningData::new(claims, backend_hints, polynomials).map_err(akita_error)?;

    let (backend_prover_setup, prepared_backend_setup) = setup.one_hot_backend()?;
    let stack = backend_stack(backend_prover_setup, prepared_backend_setup)?;
    let backend_proof = {
        let _span = info_span!("multi_group::backend_batched_prove").entered();
        with_backend_pool(|| {
            AkitaOneHotK256BackendScheme::batched_prove(
                backend_prover_setup,
                opening_data,
                &stack,
                &mut akita_transcript,
                BasisMode::Lagrange,
            )
        })
        .map_err(prove_failed)?
    };

    let proof = {
        let _span = info_span!("multi_group::serialize_backend_proof").entered();
        let proof_shape = backend_proof.shape();
        AkitaBatchProof {
            statement_bridge,
            serialized_akita_proof_shape: serialize_akita(&proof_shape)?,
            serialized_akita_proof: serialize_akita(&backend_proof)?,
        }
    };
    transcript.append(&proof);
    Ok(proof)
}

/// Verifies a fused multi-group opening. Mirrors [`multi_group_prove_one_hot`]:
/// same group order, same transcript bridge, one native `batched_verify`.
pub fn multi_group_verify_one_hot<T>(
    setup: &AkitaVerifierSetup,
    shared_point: &[AkitaField],
    groups: &[MultiGroupVerifierGroup<'_>],
    proof: &AkitaBatchProof,
    transcript: &mut T,
) -> Result<(), OpeningsError>
where
    T: Transcript<Challenge = AkitaField>,
{
    let group_num_vars: Vec<usize> = groups.iter().map(|group| group.num_vars).collect();
    validate_group_shapes(&group_num_vars, shared_point.len())?;
    for group in groups {
        validate_verifier_group(setup, group)?;
    }

    let backend_point = reverse_point(shared_point);
    let group_commitments: Vec<&AkitaCommitment> =
        groups.iter().map(|group| group.commitment).collect();
    let group_poly_counts: Vec<usize> =
        groups.iter().map(|group| group.evaluations.len()).collect();
    let (backend_commitments, backend_proof) =
        crate::shape_guard::deserialize_checked_multi_group_payload(
            &group_commitments,
            &group_num_vars,
            &group_poly_counts,
            proof,
            &backend_point,
        )?;

    let group_eval_refs: Vec<&[AkitaField]> =
        groups.iter().map(|group| group.evaluations).collect();
    let (mut akita_transcript, statement_bridge) = bind_multi_group_transcripts(
        transcript,
        setup,
        &group_commitments,
        &group_eval_refs,
        shared_point,
    );
    if proof.statement_bridge != statement_bridge {
        return Err(OpeningsError::VerificationFailed);
    }
    transcript.append(proof);

    let point_len = backend_point.len();
    let mut verifier_groups = Vec::with_capacity(groups.len());
    for ((group, commitment), num_vars) in
        groups.iter().zip(&backend_commitments).zip(&group_num_vars)
    {
        verifier_groups.push(
            PolynomialGroupClaims::new(
                PointVariableSelection::prefix(*num_vars, point_len).map_err(akita_error)?,
                group.evaluations.to_vec(),
                commitment,
            )
            .map_err(akita_error)?,
        );
    }
    let claims = OpeningClaims::from_groups(backend_point, verifier_groups).map_err(akita_error)?;

    let backend_verifier = setup.backend_verifier(crate::adapters::AkitaBackendFlavor::OneHot)?;
    with_backend_pool(|| {
        AkitaOneHotK256BackendScheme::batched_verify(
            &backend_proof,
            backend_verifier,
            &mut akita_transcript,
            claims,
            BasisMode::Lagrange,
        )
    })
    .map_err(|_| OpeningsError::VerificationFailed)
}

impl MultiGroupVerify for crate::scheme::AkitaScheme {
    fn verify_multi_group<T: Transcript<Challenge = AkitaField>>(
        setup: &AkitaVerifierSetup,
        shared_point: &[AkitaField],
        groups: &[MultiGroupOpeningClaim<'_, AkitaField, AkitaCommitment>],
        proof: &AkitaBatchProof,
        transcript: &mut T,
    ) -> Result<(), OpeningsError> {
        let verifier_groups: Vec<MultiGroupVerifierGroup<'_>> = groups
            .iter()
            .map(|group| MultiGroupVerifierGroup {
                commitment: group.commitment,
                num_vars: group.num_vars,
                evaluations: group.evaluations,
            })
            .collect();
        multi_group_verify_one_hot(setup, shared_point, &verifier_groups, proof, transcript)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests assert successful proof setup")]

    use super::*;
    use crate::adapters::AkitaSetupParams;
    use crate::scheme::AkitaScheme;
    use jolt_openings::CommitmentScheme;
    use jolt_poly::MultilinearPoly;
    use jolt_transcript::Blake2bTranscript;

    /// Deterministic K=256 one-hot column of `1 << log_rows` rows.
    fn one_hot_column(log_rows: usize, seed: u64) -> OneHotPolynomial {
        let rows = 1usize << log_rows;
        let indices = (0..rows as u64)
            .map(|row| {
                let mixed = row.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(seed);
                if mixed % 11 == 0 {
                    None
                } else {
                    Some((mixed % 256) as u8)
                }
            })
            .collect();
        OneHotPolynomial::new(AKITA_ONE_HOT_K256, indices)
    }

    fn shared_setup(final_num_vars: usize, total_polys: usize) -> AkitaProverSetup {
        let (setup, _) = AkitaScheme::setup(AkitaSetupParams::one_hot_only(
            final_num_vars,
            total_polys,
            [9u8; 32],
            AKITA_ONE_HOT_K256,
        ))
        .expect("shared one-hot setup");
        setup
    }

    /// The go/no-go gate for the fused prover: two narrower precommitted
    /// one-hot groups (advice-like, program-like) plus a wider trace-like final
    /// group, all committed under one shared setup, open through a single
    /// native multi-group `batched_prove`/`batched_verify` at one shared point.
    #[test]
    fn jolt_shaped_multi_group_root_round_trips() {
        // Trace-like final group: wider than both precommitteds. Akita's
        // planner has no multi-fold schedule below ~16 one-hot variables, so
        // every group here sits at or above that floor (the user's
        // "bigger trace" guidance).
        const FINAL_LOG_ROWS: usize = 11; // 11 + 8 = 19 vars
        const TRACE_COLUMNS: usize = 3;
        // Advice-like (1 poly) and program-like (1 poly), both narrower.
        const ADVICE_LOG_ROWS: usize = 8; // 16 vars
        const PROGRAM_LOG_ROWS: usize = 9; // 17 vars
        let final_num_vars = FINAL_LOG_ROWS + 8;
        // The shared setup's max poly count is the TOTAL across all groups (the
        // native multi-group root batches every group's polynomials through one
        // setup), not the widest single group.
        let total_polys = TRACE_COLUMNS + 2;
        let setup = shared_setup(final_num_vars, total_polys);
        let verifier_setup = <AkitaScheme as CommitmentScheme>::verifier_setup(&setup);

        let advice = one_hot_column(ADVICE_LOG_ROWS, 0x1111);
        let program = one_hot_column(PROGRAM_LOG_ROWS, 0x2222);
        let trace: Vec<OneHotPolynomial> = (0..TRACE_COLUMNS)
            .map(|i| one_hot_column(FINAL_LOG_ROWS, 0x3000 + i as u64))
            .collect();

        let (advice_commitment, advice_hint, advice_key) =
            commit_precommitted_one_hot_group(&setup, [9u8; 32], std::slice::from_ref(&advice))
                .expect("advice precommit");
        let (program_commitment, program_hint, program_key) =
            commit_precommitted_one_hot_group(&setup, [9u8; 32], std::slice::from_ref(&program))
                .expect("program precommit");
        let (trace_commitment, trace_hint) =
            commit_final_one_hot_group(&setup, [9u8; 32], &trace, vec![advice_key, program_key])
                .expect("trace final-group commit");

        // Shared point r* (FINAL_NV coords). Each group's Jolt-suffix slice is
        // r*[FINAL_NV - nv ..]; the trace binds the whole point.
        let shared_point: Vec<AkitaField> = (0..final_num_vars as u64)
            .map(|i| AkitaField::from_u64(7 + 3 * i))
            .collect();
        let advice_nv = advice.num_vars();
        let program_nv = program.num_vars();
        let advice_slice = &shared_point[final_num_vars - advice_nv..];
        let program_slice = &shared_point[final_num_vars - program_nv..];

        let advice_eval = MultilinearPoly::<AkitaField>::evaluate(&advice, advice_slice);
        let program_eval = MultilinearPoly::<AkitaField>::evaluate(&program, program_slice);
        let trace_evals: Vec<AkitaField> = trace
            .iter()
            .map(|column| MultilinearPoly::<AkitaField>::evaluate(column, &shared_point))
            .collect();

        let other_advice = one_hot_column(ADVICE_LOG_ROWS, 0x4444);
        let (_, other_advice_hint, _) = commit_precommitted_one_hot_group(
            &setup,
            [9u8; 32],
            std::slice::from_ref(&other_advice),
        )
        .expect("second same-shape advice precommit");
        let mismatched_hint_groups = vec![
            MultiGroupProverGroup {
                commitment: advice_commitment.clone(),
                num_vars: advice_nv,
                evaluations: vec![advice_eval],
                hint: other_advice_hint,
            },
            MultiGroupProverGroup {
                commitment: program_commitment.clone(),
                num_vars: program_nv,
                evaluations: vec![program_eval],
                hint: program_hint.clone(),
            },
            MultiGroupProverGroup {
                commitment: trace_commitment.clone(),
                num_vars: final_num_vars,
                evaluations: trace_evals.clone(),
                hint: trace_hint.clone(),
            },
        ];
        let mut invalid_transcript =
            Blake2bTranscript::<AkitaField>::new(b"jolt-multi-group-invalid");
        assert!(
            multi_group_prove_one_hot(
                &setup,
                &shared_point,
                mismatched_hint_groups,
                &mut invalid_transcript
            )
            .is_err(),
            "a same-shaped hint for another commitment must reject before proving"
        );

        let mut wrong_arity_hint = advice_hint.clone();
        wrong_arity_hint.polynomials = program_hint.polynomials.clone();
        let wrong_arity_groups = vec![
            MultiGroupProverGroup {
                commitment: advice_commitment.clone(),
                num_vars: advice_nv,
                evaluations: vec![advice_eval],
                hint: wrong_arity_hint,
            },
            MultiGroupProverGroup {
                commitment: program_commitment.clone(),
                num_vars: program_nv,
                evaluations: vec![program_eval],
                hint: program_hint.clone(),
            },
            MultiGroupProverGroup {
                commitment: trace_commitment.clone(),
                num_vars: final_num_vars,
                evaluations: trace_evals.clone(),
                hint: trace_hint.clone(),
            },
        ];
        let mut invalid_transcript =
            Blake2bTranscript::<AkitaField>::new(b"jolt-multi-group-invalid");
        assert!(
            multi_group_prove_one_hot(
                &setup,
                &shared_point,
                wrong_arity_groups,
                &mut invalid_transcript
            )
            .is_err(),
            "a hint polynomial with the wrong arity must reject before proving"
        );

        let prover_groups = vec![
            MultiGroupProverGroup {
                commitment: advice_commitment.clone(),
                num_vars: advice_nv,
                evaluations: vec![advice_eval],
                hint: advice_hint,
            },
            MultiGroupProverGroup {
                commitment: program_commitment.clone(),
                num_vars: program_nv,
                evaluations: vec![program_eval],
                hint: program_hint,
            },
            MultiGroupProverGroup {
                commitment: trace_commitment.clone(),
                num_vars: final_num_vars,
                evaluations: trace_evals.clone(),
                hint: trace_hint,
            },
        ];

        let mut prover_transcript = Blake2bTranscript::<AkitaField>::new(b"jolt-multi-group");
        let proof =
            multi_group_prove_one_hot(&setup, &shared_point, prover_groups, &mut prover_transcript)
                .expect("fused multi-group prove");

        let verifier_groups = vec![
            MultiGroupVerifierGroup {
                commitment: &advice_commitment,
                num_vars: advice_nv,
                evaluations: std::slice::from_ref(&advice_eval),
            },
            MultiGroupVerifierGroup {
                commitment: &program_commitment,
                num_vars: program_nv,
                evaluations: std::slice::from_ref(&program_eval),
            },
            MultiGroupVerifierGroup {
                commitment: &trace_commitment,
                num_vars: final_num_vars,
                evaluations: &trace_evals,
            },
        ];
        let mut verifier_transcript = Blake2bTranscript::<AkitaField>::new(b"jolt-multi-group");
        multi_group_verify_one_hot(
            &verifier_setup,
            &shared_point,
            &verifier_groups,
            &proof,
            &mut verifier_transcript,
        )
        .expect("fused multi-group verify");
        assert_eq!(prover_transcript.state(), verifier_transcript.state());

        let mut wrong_num_vars = advice_commitment.clone();
        wrong_num_vars.num_vars += 1;
        let invalid_groups = vec![
            MultiGroupVerifierGroup {
                commitment: &wrong_num_vars,
                num_vars: advice_nv,
                evaluations: std::slice::from_ref(&advice_eval),
            },
            MultiGroupVerifierGroup {
                commitment: &program_commitment,
                num_vars: program_nv,
                evaluations: std::slice::from_ref(&program_eval),
            },
            MultiGroupVerifierGroup {
                commitment: &trace_commitment,
                num_vars: final_num_vars,
                evaluations: &trace_evals,
            },
        ];
        let mut transcript = Blake2bTranscript::<AkitaField>::new(b"jolt-multi-group");
        assert!(
            multi_group_verify_one_hot(
                &verifier_setup,
                &shared_point,
                &invalid_groups,
                &proof,
                &mut transcript
            )
            .is_err(),
            "commitment arity metadata must match the canonical group arity"
        );

        let mut wrong_poly_count = advice_commitment.clone();
        wrong_poly_count.poly_count += 1;
        let invalid_groups = vec![
            MultiGroupVerifierGroup {
                commitment: &wrong_poly_count,
                num_vars: advice_nv,
                evaluations: std::slice::from_ref(&advice_eval),
            },
            MultiGroupVerifierGroup {
                commitment: &program_commitment,
                num_vars: program_nv,
                evaluations: std::slice::from_ref(&program_eval),
            },
            MultiGroupVerifierGroup {
                commitment: &trace_commitment,
                num_vars: final_num_vars,
                evaluations: &trace_evals,
            },
        ];
        let mut transcript = Blake2bTranscript::<AkitaField>::new(b"jolt-multi-group");
        assert!(
            multi_group_verify_one_hot(
                &verifier_setup,
                &shared_point,
                &invalid_groups,
                &proof,
                &mut transcript
            )
            .is_err(),
            "commitment polynomial-count metadata must match the evaluation count"
        );

        let wrong_arity_groups = vec![
            MultiGroupVerifierGroup {
                commitment: &program_commitment,
                num_vars: advice_nv,
                evaluations: std::slice::from_ref(&advice_eval),
            },
            MultiGroupVerifierGroup {
                commitment: &program_commitment,
                num_vars: program_nv,
                evaluations: std::slice::from_ref(&program_eval),
            },
            MultiGroupVerifierGroup {
                commitment: &trace_commitment,
                num_vars: final_num_vars,
                evaluations: &trace_evals,
            },
        ];
        let mut transcript = Blake2bTranscript::<AkitaField>::new(b"jolt-multi-group");
        assert!(
            multi_group_verify_one_hot(
                &verifier_setup,
                &shared_point,
                &wrong_arity_groups,
                &proof,
                &mut transcript
            )
            .is_err(),
            "an honestly generated commitment at the wrong arity must reject"
        );

        // Decisive architectural check: is a precommit commitment independent of
        // the shared setup size (given a sufficiently large setup)? If a group
        // committed under a *larger* setup equals the same group committed under
        // the shared setup, the program commitment can be produced at
        // preprocessing under a fixed "large enough" setup instead of the
        // trace-length-dependent shared setup.
        let larger_setup = shared_setup(final_num_vars + 2, total_polys + 3);
        let (advice_larger_commitment, _, _) = commit_precommitted_one_hot_group(
            &larger_setup,
            [9u8; 32],
            std::slice::from_ref(&advice),
        )
        .expect("advice precommit under larger setup");
        assert_eq!(
            advice_larger_commitment, advice_commitment,
            "a precommit commitment must be independent of the (sufficiently large) shared setup \
             size, so a preprocessing-time commitment matches a prove-time re-commit"
        );

        // A tampered evaluation must reject.
        let mut tampered = verifier_groups;
        let bad = advice_eval + AkitaField::from_u64(1);
        tampered[0].evaluations = std::slice::from_ref(&bad);
        let mut transcript = Blake2bTranscript::<AkitaField>::new(b"jolt-multi-group");
        assert!(
            multi_group_verify_one_hot(
                &verifier_setup,
                &shared_point,
                &tampered,
                &proof,
                &mut transcript
            )
            .is_err(),
            "a tampered group evaluation must fail the fused opening"
        );
    }
}
