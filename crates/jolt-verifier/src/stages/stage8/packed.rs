//! The Akita final opening.
//!
//! `OneHotTrace` is one native group of uniform one-hot columns, all opened
//! directly at one canonical point. Optional advice and committed-program
//! objects have distinct domains and are discharged separately through
//! [`jolt_openings::verify_packed_openings`].

use std::collections::BTreeMap;

use jolt_claims::protocols::jolt::geometry::dimensions::JoltFormulaDimensions;
use jolt_claims::protocols::jolt::lattice::geometry::word_byte_num_vars;
use jolt_claims::protocols::jolt::lattice::packing::{
    advice_byte_column_one_hot_point, advice_bytes_packing, precommitted_packing, OneHotTraceShape,
    PrecommittedPackingShape,
};
use jolt_claims::protocols::jolt::lattice::strategy::{
    OneHotTraceLayoutPlan, ONE_HOT_TRACE_LAYOUT,
};
use jolt_claims::protocols::jolt::{
    JoltAdviceKind, JoltCommittedPolynomial, JoltOneHotConfig, JoltOpeningId, JoltPolynomialId,
};
use jolt_field::{Field, FixedByteSize};
use jolt_openings::{
    fused_stage8_open_eligible, verify_packed_openings, verify_packed_reduction, CommitmentScheme,
    EvaluationClaim, MultiGroupOpeningClaim, MultiGroupVerify, PackedObjectGroup,
    PackedOpeningProof, PackedVerifierObject, PrefixPackedStatement, PrefixPacking,
};
use jolt_poly::Point;
use jolt_transcript::{AppendToTranscript, Transcript};

use super::reconstruction::ReconstructionClearOutput;
use crate::stages::stage7::outputs::Stage7ClearOutput;
use crate::stages::stage8::{OneHotTraceCommitmentMetadata, OneHotTraceSetupMetadata};
use crate::VerifierError;

fn batch_failed(reason: impl ToString) -> VerifierError {
    VerifierError::FinalOpeningBatchFailed {
        reason: reason.to_string(),
    }
}

fn opening_failed(reason: impl ToString) -> VerifierError {
    VerifierError::FinalOpeningVerificationFailed {
        reason: reason.to_string(),
    }
}

fn validate_one_hot_trace_metadata<C, S>(
    commitment: &C,
    setup: &S,
    canonical_digest: [u8; 32],
    column_arity: usize,
    column_count: usize,
    one_hot_k: usize,
) -> Result<(), VerifierError>
where
    C: OneHotTraceCommitmentMetadata,
    S: OneHotTraceSetupMetadata,
{
    if !commitment.is_one_hot_backend() {
        return Err(batch_failed(
            "OneHotTrace commitment must use Akita's one-hot backend",
        ));
    }
    if commitment.one_hot_k() != one_hot_k || setup.one_hot_k() != one_hot_k {
        return Err(batch_failed(format!(
            "OneHotTrace commitment/setup one-hot chunk size must equal canonical K={one_hot_k}"
        )));
    }
    if commitment.layout_digest() != canonical_digest {
        return Err(batch_failed(
            "OneHotTrace commitment has a noncanonical layout digest",
        ));
    }
    if commitment.num_vars() != column_arity || setup.max_num_vars() != column_arity {
        return Err(batch_failed(format!(
            "OneHotTrace commitment/setup arity must equal canonical arity {column_arity}"
        )));
    }
    // The commitment carries exactly the trace columns; the shared setup's poly
    // capacity may exceed that when it also backs precommitted auxiliary groups
    // for a fused multi-group root fold (the trace column count plus one slot
    // per aux group), so the setup bound is `>=`, not `==`.
    if commitment.poly_count() != column_count
        || setup.max_num_polys_per_commitment_group() < column_count
    {
        return Err(batch_failed(format!(
            "OneHotTrace commitment column count must equal canonical count {column_count} and the setup must support at least that many"
        )));
    }
    if setup.default_layout_digest() != canonical_digest {
        return Err(batch_failed(
            "OneHotTrace verifier setup has a noncanonical layout digest",
        ));
    }
    Ok(())
}

fn validate_fused_aux_metadata<C, S>(
    commitment: &C,
    setup: &S,
    canonical_num_vars: usize,
    one_hot_k: usize,
) -> Result<(), VerifierError>
where
    C: OneHotTraceCommitmentMetadata,
    S: OneHotTraceSetupMetadata,
{
    if !commitment.is_one_hot_backend() {
        return Err(batch_failed(
            "fused auxiliary commitment must use Akita's one-hot backend",
        ));
    }
    if commitment.one_hot_k() != one_hot_k || setup.one_hot_k() != one_hot_k {
        return Err(batch_failed(format!(
            "fused auxiliary commitment/setup one-hot chunk size must equal canonical K={one_hot_k}"
        )));
    }
    if commitment.layout_digest() != setup.default_layout_digest() {
        return Err(batch_failed(
            "fused auxiliary commitment has a noncanonical layout digest",
        ));
    }
    if commitment.num_vars() != canonical_num_vars {
        return Err(batch_failed(format!(
            "fused auxiliary commitment arity {} does not equal canonical packed arity {canonical_num_vars}",
            commitment.num_vars()
        )));
    }
    if commitment.poly_count() != 1 {
        return Err(batch_failed(format!(
            "fused auxiliary commitment must contain exactly one polynomial, got {}",
            commitment.poly_count()
        )));
    }
    if canonical_num_vars > setup.max_num_vars() || setup.max_num_polys_per_commitment_group() < 1 {
        return Err(batch_failed(
            "fused auxiliary commitment exceeds the shared verifier setup",
        ));
    }
    Ok(())
}

/// A byte column's word-variable count, recovered from its leaf claim's
/// arity (the `(byte ‖ place)` cell prefix is fixed).
fn leaf_word_vars(cell_vars: usize) -> Result<usize, VerifierError> {
    let cell_prefix_vars = word_byte_num_vars(0);
    cell_vars.checked_sub(cell_prefix_vars).ok_or_else(|| {
        batch_failed(format!(
            "byte-column leaf has {cell_vars} variables, below the \
             {cell_prefix_vars}-variable cell prefix"
        ))
    })
}

/// One resolved commitment object: its canonical packing plus the borrowed
/// commitment and shape-exact setup the final PCS opening runs against.
type ResolvedObject<'a, PCS> = (
    PrefixPacking<JoltCommittedPolynomial>,
    &'a <PCS as jolt_crypto::Commitment>::Output,
    &'a <PCS as CommitmentScheme>::VerifierSetup,
);

/// Resolve one advice object's packing/commitment/setup triple, or `None`
/// when the kind is absent; a commitment without a reconstruction leaf, or a
/// present object missing its commitment or setup, is rejected fail-closed.
fn advice_object<'a, PCS: CommitmentScheme>(
    present: Option<&Vec<PCS::Field>>,
    commitment: Option<&'a PCS::Output>,
    setup: Option<&'a PCS::VerifierSetup>,
    kind: JoltAdviceKind,
) -> Result<Option<ResolvedObject<'a, PCS>>, VerifierError> {
    let Some(leaf_point) = present else {
        if commitment.is_some() {
            return Err(batch_failed(format!(
                "{kind:?} advice commitment supplied without a reconstruction leaf"
            )));
        }
        return Ok(None);
    };
    let (Some(commitment), Some(setup)) = (commitment, setup) else {
        return Err(batch_failed(format!(
            "{kind:?} advice object without a commitment or setup"
        )));
    };
    let packing =
        advice_bytes_packing(kind, leaf_word_vars(leaf_point.len())?).map_err(batch_failed)?;
    Ok(Some((packing, commitment, setup)))
}

#[expect(
    clippy::too_many_arguments,
    reason = "the per-object commitments and their preprocessing setups, resolved here in one place"
)]
pub fn verify<PCS, VC, T>(
    formula_dimensions: &JoltFormulaDimensions,
    one_hot_config: JoltOneHotConfig,
    preprocessing: &crate::preprocessing::JoltVerifierPreprocessing<PCS, VC>,
    one_hot_trace_commitment: &PCS::Output,
    untrusted_advice_commitment: Option<&PCS::Output>,
    trusted_advice_commitment: Option<&PCS::Output>,
    proof: &PackedOpeningProof<PCS::Field, PCS::Proof>,
    transcript: &mut T,
    stage7: &Stage7ClearOutput<PCS::Field>,
    reconstruction: &ReconstructionClearOutput<PCS::Field>,
) -> Result<(), VerifierError>
where
    PCS: CommitmentScheme + MultiGroupVerify,
    PCS::Output: Clone + AppendToTranscript + OneHotTraceCommitmentMetadata,
    PCS::VerifierSetup: OneHotTraceSetupMetadata,
    VC: jolt_crypto::VectorCommitment<Field = PCS::Field>,
    T: Transcript<Challenge = PCS::Field>,
{
    // Per-object packings, commitments, and setups in canonical object order:
    // `OneHotTrace` is one native group of uniform one-hot columns, followed
    // by the optional auxiliary commitment objects. The shared layout is the
    // same one the prover committed under.
    // Optional objects join exactly when their reconstruction outputs exist;
    // presence must agree with the proof/preprocessing commitment slots.
    let chunk_width = one_hot_config.committed_chunk_bits();
    let one_hot_trace_shape = OneHotTraceShape {
        ra_layout: formula_dimensions.ra_layout,
        log_t: formula_dimensions.trace.log_t(),
        log_k_chunk: chunk_width,
    };
    let plan = ONE_HOT_TRACE_LAYOUT
        .plan(&one_hot_trace_shape)
        .map_err(batch_failed)?;
    let canonical_digest = ONE_HOT_TRACE_LAYOUT
        .layout_digest(&one_hot_trace_shape)
        .map_err(batch_failed)?;
    let OneHotTraceLayoutPlan {
        columns,
        column_arity,
    } = &plan;
    validate_one_hot_trace_metadata(
        one_hot_trace_commitment,
        &preprocessing.pcs_setup,
        canonical_digest,
        *column_arity,
        columns.len(),
        1 << chunk_width,
    )?;
    let leaves = leaf_claims(stage7, reconstruction);
    let mut common_point: Option<Vec<PCS::Field>> = None;
    let mut evaluations = Vec::with_capacity(columns.len());
    for polynomial in columns {
        let claim = leaves.get(polynomial).ok_or_else(|| {
            batch_failed(format!(
                "missing final OneHotTrace claim for {polynomial:?}"
            ))
        })?;
        let point = ONE_HOT_TRACE_LAYOUT
            .column_point(*polynomial, chunk_width, claim.point.as_slice())
            .map_err(batch_failed)?;
        if let Some(expected) = &common_point {
            if expected != &point {
                return Err(batch_failed(format!(
                    "OneHotTrace column {polynomial:?} does not share the canonical opening point"
                )));
            }
        } else {
            common_point = Some(point);
        }
        evaluations.push(claim.value);
    }
    let common_point = common_point.ok_or_else(|| batch_failed("OneHotTrace has no columns"))?;

    // Resolve advice objects and their canonical public packings before choosing
    // topology. Commitment metadata is never an input to the fused gate.
    let program_present = preprocessing.program.committed().is_some()
        || reconstruction.output_points.bytecode.is_some();
    let mut advice_objects = Vec::new();
    if let Some(object) = advice_object::<PCS>(
        reconstruction
            .output_points
            .untrusted_advice
            .as_ref()
            .map(|points| &points.bytes),
        untrusted_advice_commitment,
        preprocessing.untrusted_advice_setup.as_ref(),
        JoltAdviceKind::Untrusted,
    )? {
        advice_objects.push((JoltAdviceKind::Untrusted, object));
    }
    if let Some(object) = advice_object::<PCS>(
        reconstruction
            .output_points
            .trusted_advice
            .as_ref()
            .map(|points| &points.bytes),
        trusted_advice_commitment,
        preprocessing.trusted_advice_setup.as_ref(),
        JoltAdviceKind::Trusted,
    )? {
        advice_objects.push((JoltAdviceKind::Trusted, object));
    }
    let aux_num_vars: Vec<usize> = advice_objects
        .iter()
        .map(|(_, (packing, _, _))| packing.packed_num_vars)
        .collect();
    let trusted_advice_present = advice_objects
        .iter()
        .any(|(kind, _)| *kind == JoltAdviceKind::Trusted);
    let fallback_topology_present = program_present || trusted_advice_present;
    if fused_stage8_open_eligible(
        chunk_width,
        common_point.len(),
        fallback_topology_present,
        &aux_num_vars,
    ) {
        if preprocessing.pcs_setup.max_num_polys_per_commitment_group()
            < columns.len() + advice_objects.len()
        {
            return Err(batch_failed(
                "shared verifier setup cannot contain every fused commitment polynomial",
            ));
        }
        for (_, (packing, commitment, _)) in &advice_objects {
            validate_fused_aux_metadata(
                *commitment,
                &preprocessing.pcs_setup,
                packing.packed_num_vars,
                1 << chunk_width,
            )?;
        }
        return verify_fused(
            &plan,
            preprocessing,
            one_hot_trace_commitment,
            &advice_objects,
            &common_point,
            &evaluations,
            proof,
            transcript,
            &leaves,
        );
    }

    // The trace group leads (widest object → binds the whole reduced point);
    // advice/program follow as suffix-binding singletons. Trace objects are
    // identity-packed one-hot columns, each carrying the trace commitment and a
    // single claim at the shared point, mirroring the prover's construction.
    let mut packings: Vec<PrefixPacking<JoltCommittedPolynomial>> = Vec::new();
    let mut setups: Vec<&PCS::VerifierSetup> = Vec::new();
    let mut groups = Vec::new();

    groups.push(PackedObjectGroup {
        start: 0,
        len: columns.len(),
    });
    for column in columns {
        packings.push(PrefixPacking::new([(*column, common_point.len())]).map_err(batch_failed)?);
        setups.push(&preprocessing.pcs_setup);
    }
    let trace_object_count = columns.len();

    // Auxiliary commitments, aligned with the packings pushed after the trace.
    let mut aux_commitments = Vec::new();
    for (_, (packing, commitment, setup)) in advice_objects {
        groups.push(PackedObjectGroup::singleton(packings.len()));
        packings.push(packing);
        aux_commitments.push(commitment);
        setups.push(setup);
    }
    match (
        reconstruction.output_points.bytecode.as_ref(),
        preprocessing.program.committed(),
    ) {
        (Some(bytecode_points), Some(committed)) => {
            let setup = preprocessing
                .program_one_hot_setup
                .as_ref()
                .ok_or_else(|| {
                    batch_failed(
                        "committed-program object without a verifier setup in preprocessing",
                    )
                })?;
            // The `ProgramOneHot` shape is claim-derived: the packing must match the
            // committed witness or its PCS opening fails, so the lane/image
            // point arities are an honest source for the row/word counts.
            let log_bytecode_rows = bytecode_points
                .pc_bytes
                .first()
                .map(|point| leaf_word_vars(point.len()))
                .transpose()?
                .ok_or_else(|| batch_failed("program reconstruction has no pc lanes"))?;
            let program_image_log_words = reconstruction
                .output_points
                .program_image
                .as_ref()
                .map(|points| leaf_word_vars(points.bytes.len()))
                .transpose()?;
            groups.push(PackedObjectGroup::singleton(packings.len()));
            packings.push(
                precommitted_packing(&PrecommittedPackingShape {
                    bytecode_chunks: committed.bytecode_chunk_count(),
                    log_bytecode_rows,
                    imm_byte_width: <PCS::Field as FixedByteSize>::NUM_BYTES,
                    program_image_log_words,
                })
                .map_err(batch_failed)?,
            );
            aux_commitments.push(&committed.program_one_hot_commitment);
            setups.push(setup);
        }
        (None, None) => {}
        (Some(_), None) => {
            return Err(batch_failed(
                "program reconstruction leaves without a ProgramOneHot commitment",
            ));
        }
        (None, Some(_)) => {
            return Err(batch_failed(
                "ProgramOneHot commitment supplied without program reconstruction leaves",
            ));
        }
    }

    // Statements: the trace columns claim at the shared point under the trace
    // commitment; the auxiliary objects retain their own logical leaf points.
    let mut statements = Vec::with_capacity(packings.len());
    for (column, value) in columns.iter().zip(&evaluations) {
        statements.push(PrefixPackedStatement::new(
            one_hot_trace_commitment.clone(),
            vec![(*column, EvaluationClaim::new(common_point.clone(), *value))],
        ));
    }
    for (packing, commitment) in packings[trace_object_count..].iter().zip(&aux_commitments) {
        statements.push(object_statement(packing, (*commitment).clone(), &leaves)?);
    }

    let objects: Vec<PackedVerifierObject<'_, PCS, JoltCommittedPolynomial>> = packings
        .iter()
        .zip(&statements)
        .zip(setups)
        .map(|((packing, statement), setup)| PackedVerifierObject {
            packing,
            statement,
            setup,
        })
        .collect();

    verify_packed_openings(&objects, &groups, proof, transcript).map_err(opening_failed)
}

/// Verifier mirror of the prover's fused stage-8 open: reduce the advice groups
/// and the trace columns together to one shared point `r*` (object order
/// `[aux…, trace]`), then check the single native multi-group root fold that
/// discharges every group at prefix/suffix slices of `r*`. The trace is the
/// final (widest) group and binds the whole point.
#[expect(
    clippy::too_many_arguments,
    reason = "the fused open resolves the trace and advice objects plus their shared inputs here in one place"
)]
fn verify_fused<PCS, VC, T>(
    plan: &OneHotTraceLayoutPlan,
    preprocessing: &crate::preprocessing::JoltVerifierPreprocessing<PCS, VC>,
    one_hot_trace_commitment: &PCS::Output,
    aux: &[(JoltAdviceKind, ResolvedObject<'_, PCS>)],
    common_point: &[PCS::Field],
    trace_evaluations: &[PCS::Field],
    proof: &PackedOpeningProof<PCS::Field, PCS::Proof>,
    transcript: &mut T,
    leaves: &BTreeMap<JoltCommittedPolynomial, EvaluationClaim<PCS::Field>>,
) -> Result<(), VerifierError>
where
    PCS: CommitmentScheme + MultiGroupVerify,
    PCS::Output: Clone + AppendToTranscript + OneHotTraceCommitmentMetadata,
    PCS::VerifierSetup: OneHotTraceSetupMetadata,
    VC: jolt_crypto::VectorCommitment<Field = PCS::Field>,
    T: Transcript<Challenge = PCS::Field>,
{
    let columns = &plan.columns;
    let n_trace = columns.len();
    let n_aux = aux.len();
    if proof.openings.len() != 1 {
        return Err(batch_failed(format!(
            "fused packed opening must carry exactly one native opening, got {}",
            proof.openings.len()
        )));
    }
    if proof.evaluations.len() != n_trace + n_aux {
        return Err(batch_failed(format!(
            "fused packed opening expects {} evaluations, got {}",
            n_trace + n_aux,
            proof.evaluations.len()
        )));
    }

    // Advice objects use the same canonical packings that selected the gate.
    let mut aux_statements = Vec::with_capacity(n_aux);
    for (_, (packing, commitment, _)) in aux {
        let statement = object_statement(packing, (*commitment).clone(), leaves)?;
        aux_statements.push(statement);
    }

    let trace_packings: Vec<PrefixPacking<JoltCommittedPolynomial>> = columns
        .iter()
        .map(|column| PrefixPacking::new([(*column, common_point.len())]).map_err(batch_failed))
        .collect::<Result<_, _>>()?;
    let trace_statements: Vec<
        PrefixPackedStatement<PCS::Field, JoltCommittedPolynomial, PCS::Output>,
    > = columns
        .iter()
        .zip(trace_evaluations)
        .map(|(column, value)| {
            PrefixPackedStatement::new(
                one_hot_trace_commitment.clone(),
                vec![(*column, EvaluationClaim::new(common_point.to_vec(), *value))],
            )
        })
        .collect();

    // Object order [aux…, trace] mirrors the prover.
    let mut objects: Vec<PackedVerifierObject<'_, PCS, JoltCommittedPolynomial>> =
        Vec::with_capacity(n_trace + n_aux);
    for ((_, (packing, _, _)), statement) in aux.iter().zip(&aux_statements) {
        objects.push(PackedVerifierObject {
            packing,
            statement,
            setup: &preprocessing.pcs_setup,
        });
    }
    for (packing, statement) in trace_packings.iter().zip(&trace_statements) {
        objects.push(PackedVerifierObject {
            packing,
            statement,
            setup: &preprocessing.pcs_setup,
        });
    }

    let shared_point = verify_packed_reduction::<PCS, JoltCommittedPolynomial, T>(
        &objects,
        &proof.round_polynomials,
        &proof.evaluations,
        transcript,
    )
    .map_err(opening_failed)?;
    drop(objects);

    let mut groups = Vec::with_capacity(n_aux + 1);
    for (index, (_, (packing, commitment, _))) in aux.iter().enumerate() {
        groups.push(MultiGroupOpeningClaim {
            commitment: *commitment,
            num_vars: packing.packed_num_vars,
            evaluations: std::slice::from_ref(&proof.evaluations[index]),
        });
    }
    groups.push(MultiGroupOpeningClaim {
        commitment: one_hot_trace_commitment,
        num_vars: common_point.len(),
        evaluations: &proof.evaluations[n_aux..n_aux + n_trace],
    });
    PCS::verify_multi_group(
        &preprocessing.pcs_setup,
        &shared_point,
        &groups,
        &proof.openings[0],
        transcript,
    )
    .map_err(opening_failed)
}

/// Assembles one object's statement: each of its packing's canonical columns
/// paired with that column's leaf claim.
fn object_statement<F, C>(
    packing: &PrefixPacking<JoltCommittedPolynomial>,
    commitment: C,
    leaves: &BTreeMap<JoltCommittedPolynomial, EvaluationClaim<F>>,
) -> Result<PrefixPackedStatement<F, JoltCommittedPolynomial, C>, VerifierError>
where
    F: Field,
{
    let claims = packing
        .iter()
        .map(|(polynomial, _slot)| {
            leaves
                .get(polynomial)
                .cloned()
                .map(|claim| (*polynomial, claim))
                .ok_or_else(|| {
                    batch_failed(format!(
                        "missing stage output claim for packed leaf {polynomial:?}"
                    ))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PrefixPackedStatement::new(commitment, claims))
}

/// Every packed column's single leaf claim, resolved from the stage-7 and
/// reconstruction outputs and keyed by committed polynomial. Coverage against
/// the packings is machine-checked downstream by `prepare_statement`
/// (one-claim-per-slot, no gaps, per-slot point arity).
fn leaf_claims<F: Field>(
    stage7: &Stage7ClearOutput<F>,
    reconstruction: &ReconstructionClearOutput<F>,
) -> BTreeMap<JoltCommittedPolynomial, EvaluationClaim<F>> {
    use JoltCommittedPolynomial as Poly;

    fn leaf<F: Field>(value: F, point: &[F]) -> EvaluationClaim<F> {
        EvaluationClaim::new(Point::high_to_low(point.to_vec()), value)
    }
    fn insert<F: Field>(
        leaves: &mut BTreeMap<JoltCommittedPolynomial, EvaluationClaim<F>>,
        polynomial: JoltCommittedPolynomial,
        claim: EvaluationClaim<F>,
    ) {
        // Keys are distinct by construction, so no entry is ever displaced.
        let _previous = BTreeMap::insert(leaves, polynomial, claim);
    }
    fn insert_indexed<F: Field>(
        leaves: &mut BTreeMap<JoltCommittedPolynomial, EvaluationClaim<F>>,
        values: &[F],
        points: &[Vec<F>],
        polynomial: impl Fn(usize) -> JoltCommittedPolynomial,
    ) {
        for (index, (value, point)) in values.iter().zip(points).enumerate() {
            insert(leaves, polynomial(index), leaf(*value, point));
        }
    }
    let mut leaves = BTreeMap::new();

    let hamming_values = &stage7.output_values.hamming_weight_claim_reduction;
    let hamming_points = &stage7.output_points.hamming_weight_claim_reduction;
    insert_indexed(
        &mut leaves,
        &hamming_values.instruction_ra,
        &hamming_points.instruction_ra,
        Poly::InstructionRa,
    );
    insert_indexed(
        &mut leaves,
        &hamming_values.bytecode_ra,
        &hamming_points.bytecode_ra,
        Poly::BytecodeRa,
    );
    insert_indexed(
        &mut leaves,
        &hamming_values.ram_ra,
        &hamming_points.ram_ra,
        Poly::RamRa,
    );

    insert_indexed(
        &mut leaves,
        &hamming_values.unsigned_inc_chunks,
        &hamming_points.unsigned_inc_chunks,
        Poly::UnsignedIncChunk,
    );
    insert(
        &mut leaves,
        Poly::UnsignedIncMsb,
        leaf(
            hamming_values.unsigned_inc_msb,
            &hamming_points.unsigned_inc_msb,
        ),
    );

    if let Some((values, points)) = reconstruction
        .output_values
        .untrusted_advice
        .as_ref()
        .zip(reconstruction.output_points.untrusted_advice.as_ref())
    {
        insert(
            &mut leaves,
            Poly::UntrustedAdviceBytes,
            leaf(
                values.bytes,
                &advice_byte_column_one_hot_point(&points.bytes),
            ),
        );
    }
    if let Some((values, points)) = reconstruction
        .output_values
        .trusted_advice
        .as_ref()
        .zip(reconstruction.output_points.trusted_advice.as_ref())
    {
        insert(
            &mut leaves,
            Poly::TrustedAdviceBytes,
            leaf(
                values.bytes,
                &advice_byte_column_one_hot_point(&points.bytes),
            ),
        );
    }
    if let Some((values, points)) = reconstruction
        .output_values
        .program_image
        .as_ref()
        .zip(reconstruction.output_points.program_image.as_ref())
    {
        insert(
            &mut leaves,
            Poly::ProgramImageBytes,
            leaf(
                values.bytes,
                &advice_byte_column_one_hot_point(&points.bytes),
            ),
        );
    }

    // The bytecode leaf keys are read off the canonical cell order jolt-claims
    // pins (`leaves()` pairs one-for-one with `opening_order`), instead of
    // re-deriving the chunk/lane index arithmetic here. Each reconstruction
    // leaf point carries the one-hot lane block leading; relabel it into the
    // lane-low commitment layout the program poly is committed under, exactly
    // as the advice leaves above.
    if let Some((values, points)) = reconstruction
        .output_values
        .bytecode
        .as_ref()
        .zip(reconstruction.output_points.bytecode.as_ref())
    {
        for ((id, value), (_, point)) in values.leaves().zip(points.leaves()) {
            let JoltOpeningId::Polynomial {
                polynomial: JoltPolynomialId::Committed(polynomial),
                ..
            } = id
            else {
                continue;
            };
            insert(
                &mut leaves,
                polynomial,
                leaf(*value, &advice_byte_column_one_hot_point(point)),
            );
        }
    }

    leaves
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;
    use jolt_claims::protocols::jolt::geometry::claim_reductions::bytecode::committed_lane_vars;
    use jolt_claims::protocols::jolt::geometry::ra::JoltRaPolynomialLayout;
    use jolt_claims::protocols::jolt::lattice::geometry::BYTE_BITS;
    use jolt_claims::protocols::jolt::lattice::relations::advice_reconstruction::{
        TrustedAdviceReconstructionOutputClaims, UntrustedAdviceReconstructionOutputClaims,
    };
    use jolt_claims::protocols::jolt::lattice::relations::bytecode_reconstruction::BytecodeChunkReconstructionOutputClaims;
    use jolt_claims::protocols::jolt::lattice::relations::program_image_reconstruction::ProgramImageReconstructionOutputClaims;
    use jolt_claims::protocols::jolt::BytecodeRegisterLane;
    use jolt_field::{Fr, FromPrimitiveInt};
    use jolt_poly::math::Math;
    use jolt_riscv::{NUM_CIRCUIT_FLAGS, NUM_INSTRUCTION_FLAGS};

    use super::super::reconstruction::{ReconstructionOutputClaims, ReconstructionOutputPoints};
    use crate::stages::stage7::hamming_weight_claim_reduction::HammingWeightClaimReductionOutputClaims;
    use crate::stages::stage7::outputs::{Stage7OutputClaims, Stage7OutputPoints};

    const LOG_T: usize = 4;
    const LOG_K_CHUNK: usize = 8;
    const INC_CHUNKS: usize = 8;
    const BYTECODE_CHUNKS: usize = 2;
    const LOG_BYTECODE_ROWS: usize = 6;
    const LOG_IMAGE_WORDS: usize = 5;
    const ADVICE_WORD_VARS: usize = 3;

    #[derive(Clone, Copy)]
    struct CommitmentMetadata {
        one_hot: bool,
        digest: [u8; 32],
        num_vars: usize,
        poly_count: usize,
        one_hot_k: usize,
    }

    impl OneHotTraceCommitmentMetadata for CommitmentMetadata {
        fn is_one_hot_backend(&self) -> bool {
            self.one_hot
        }

        fn layout_digest(&self) -> [u8; 32] {
            self.digest
        }

        fn num_vars(&self) -> usize {
            self.num_vars
        }

        fn poly_count(&self) -> usize {
            self.poly_count
        }

        fn one_hot_k(&self) -> usize {
            self.one_hot_k
        }
    }

    #[derive(Clone, Copy)]
    struct SetupMetadata {
        digest: [u8; 32],
        num_vars: usize,
        poly_count: usize,
        one_hot_k: usize,
    }

    impl OneHotTraceSetupMetadata for SetupMetadata {
        fn max_num_vars(&self) -> usize {
            self.num_vars
        }

        fn max_num_polys_per_commitment_group(&self) -> usize {
            self.poly_count
        }

        fn default_layout_digest(&self) -> [u8; 32] {
            self.digest
        }

        fn one_hot_k(&self) -> usize {
            self.one_hot_k
        }
    }

    fn fr(value: u64) -> Fr {
        Fr::from_u64(value)
    }

    #[test]
    fn one_hot_trace_metadata_is_enforced_before_pcs_verification() {
        let digest = [7; 32];
        let commitment = CommitmentMetadata {
            one_hot: true,
            digest,
            num_vars: 12,
            poly_count: 17,
            one_hot_k: 256,
        };
        let setup = SetupMetadata {
            digest,
            num_vars: 12,
            poly_count: 17,
            one_hot_k: 256,
        };
        assert!(validate_one_hot_trace_metadata(&commitment, &setup, digest, 12, 17, 256).is_ok());

        for invalid in [
            CommitmentMetadata {
                one_hot: false,
                ..commitment
            },
            CommitmentMetadata {
                digest: [8; 32],
                ..commitment
            },
            CommitmentMetadata {
                num_vars: 13,
                ..commitment
            },
            CommitmentMetadata {
                poly_count: 18,
                ..commitment
            },
            CommitmentMetadata {
                one_hot_k: 16,
                ..commitment
            },
        ] {
            assert!(
                validate_one_hot_trace_metadata(&invalid, &setup, digest, 12, 17, 256).is_err()
            );
        }
        for invalid in [
            SetupMetadata {
                digest: [9; 32],
                ..setup
            },
            SetupMetadata {
                num_vars: 13,
                ..setup
            },
            SetupMetadata {
                poly_count: 18,
                ..setup
            },
            SetupMetadata {
                one_hot_k: 16,
                ..setup
            },
        ] {
            assert!(
                validate_one_hot_trace_metadata(&commitment, &invalid, digest, 12, 17, 256)
                    .is_err()
            );
        }
    }

    fn point(arity: usize) -> Vec<Fr> {
        vec![fr(1); arity]
    }

    fn stage7(layout: JoltRaPolynomialLayout) -> Stage7ClearOutput<Fr> {
        let one_hot_arity = LOG_K_CHUNK + LOG_T;
        let hamming_values = HammingWeightClaimReductionOutputClaims {
            instruction_ra: (0..layout.instruction())
                .map(|i| fr(100 + i as u64))
                .collect(),
            bytecode_ra: (0..layout.bytecode()).map(|i| fr(200 + i as u64)).collect(),
            ram_ra: (0..layout.ram()).map(|i| fr(300 + i as u64)).collect(),
            unsigned_inc_chunks: (0..INC_CHUNKS).map(|i| fr(400 + i as u64)).collect(),
            unsigned_inc_msb: fr(500),
        };
        let hamming_points = HammingWeightClaimReductionOutputClaims {
            instruction_ra: vec![point(one_hot_arity); layout.instruction()],
            bytecode_ra: vec![point(one_hot_arity); layout.bytecode()],
            ram_ra: vec![point(one_hot_arity); layout.ram()],
            unsigned_inc_chunks: vec![point(one_hot_arity); INC_CHUNKS],
            unsigned_inc_msb: point(one_hot_arity),
        };
        Stage7ClearOutput {
            output_values: Stage7OutputClaims {
                hamming_weight_claim_reduction: hamming_values,
                trusted_advice: None,
                untrusted_advice: None,
                bytecode_address_phase: None,
                program_image_address_phase: None,
            },
            output_points: Stage7OutputPoints {
                hamming_weight_claim_reduction: hamming_points,
                trusted_advice: None,
                untrusted_advice: None,
                bytecode_address_phase: None,
                program_image_address_phase: None,
            },
        }
    }

    fn reconstruction() -> ReconstructionClearOutput<Fr> {
        let advice_arity = word_byte_num_vars(ADVICE_WORD_VARS);
        let selectors = BYTECODE_CHUNKS * BytecodeRegisterLane::ALL.len();
        let bytecode_values = BytecodeChunkReconstructionOutputClaims {
            register_selectors: (0..selectors).map(|i| fr(600 + i as u64)).collect(),
            circuit_flags: (0..BYTECODE_CHUNKS * NUM_CIRCUIT_FLAGS)
                .map(|i| fr(700 + i as u64))
                .collect(),
            instruction_flags: (0..BYTECODE_CHUNKS * NUM_INSTRUCTION_FLAGS)
                .map(|i| fr(800 + i as u64))
                .collect(),
            lookup_selectors: (0..BYTECODE_CHUNKS).map(|i| fr(900 + i as u64)).collect(),
            raf_flags: (0..BYTECODE_CHUNKS).map(|i| fr(910 + i as u64)).collect(),
            pc_bytes: (0..BYTECODE_CHUNKS).map(|i| fr(920 + i as u64)).collect(),
            imm_bytes: (0..BYTECODE_CHUNKS).map(|i| fr(930 + i as u64)).collect(),
        };
        // Every selector/flag leaf carries the shared 8-bit one-hot lane block
        // (leading, before the lane-low relabel) plus the row point.
        let lane_row_arity = BYTE_BITS + LOG_BYTECODE_ROWS;
        let bytecode_points = BytecodeChunkReconstructionOutputClaims {
            register_selectors: vec![point(lane_row_arity); selectors],
            circuit_flags: vec![point(lane_row_arity); BYTECODE_CHUNKS * NUM_CIRCUIT_FLAGS],
            instruction_flags: vec![point(lane_row_arity); BYTECODE_CHUNKS * NUM_INSTRUCTION_FLAGS],
            lookup_selectors: vec![point(lane_row_arity); BYTECODE_CHUNKS],
            raf_flags: vec![point(lane_row_arity); BYTECODE_CHUNKS],
            pc_bytes: vec![point(word_byte_num_vars(LOG_BYTECODE_ROWS)); BYTECODE_CHUNKS],
            imm_bytes: vec![
                point(
                    jolt_claims::protocols::jolt::lattice::geometry::byte_num_vars(
                        <Fr as FixedByteSize>::NUM_BYTES,
                        LOG_BYTECODE_ROWS,
                    )
                    .unwrap()
                );
                BYTECODE_CHUNKS
            ],
        };
        ReconstructionClearOutput {
            output_values: ReconstructionOutputClaims {
                untrusted_advice: Some(UntrustedAdviceReconstructionOutputClaims { bytes: fr(41) }),
                trusted_advice: Some(TrustedAdviceReconstructionOutputClaims { bytes: fr(43) }),
                bytecode: Some(bytecode_values),
                program_image: Some(ProgramImageReconstructionOutputClaims { bytes: fr(47) }),
            },
            output_points: ReconstructionOutputPoints {
                untrusted_advice: Some(UntrustedAdviceReconstructionOutputClaims {
                    bytes: point(advice_arity),
                }),
                trusted_advice: Some(TrustedAdviceReconstructionOutputClaims {
                    bytes: point(advice_arity),
                }),
                bytecode: Some(bytecode_points),
                program_image: Some(ProgramImageReconstructionOutputClaims {
                    bytes: point(word_byte_num_vars(LOG_IMAGE_WORDS)),
                }),
            },
        }
    }

    /// Every auxiliary object's packing resolves exactly one leaf claim per
    /// column at the slot's arity — `prepare_statement` machine-checks
    /// one-claim-per-slot, full coverage, and per-slot point arity, so a
    /// passing preparation pins the leaf-resolution map against the canonical
    /// packings.
    #[test]
    fn auxiliary_packed_statements_cover_every_column_at_slot_arity() {
        let layout = JoltRaPolynomialLayout::new(2, 1, 1).unwrap();
        let leaves = leaf_claims(&stage7(layout), &reconstruction());

        let objects = [
            advice_bytes_packing(JoltAdviceKind::Untrusted, ADVICE_WORD_VARS).unwrap(),
            advice_bytes_packing(JoltAdviceKind::Trusted, ADVICE_WORD_VARS).unwrap(),
            precommitted_packing(&PrecommittedPackingShape {
                bytecode_chunks: BYTECODE_CHUNKS,
                log_bytecode_rows: LOG_BYTECODE_ROWS,
                imm_byte_width: <Fr as FixedByteSize>::NUM_BYTES,
                program_image_log_words: Some(LOG_IMAGE_WORDS),
            })
            .unwrap(),
        ];
        for packing in &objects {
            let statement = object_statement(packing, (), &leaves).unwrap();
            assert_eq!(statement.claims.len(), packing.iter().count());
            let prepared = packing.prepare_statement(&statement).unwrap();
            assert_eq!(prepared.num_claims(), packing.iter().count());
        }
    }

    /// The lane-vars split the leaf resolver relies on matches the completed
    /// chunk claims the reconstruction consumes.
    #[test]
    fn committed_lane_split_matches_layout() {
        assert_eq!(
            committed_lane_vars(),
            jolt_claims::protocols::jolt::geometry::claim_reductions::bytecode::COMMITTED_BYTECODE_LANE_CAPACITY
                .log_2()
        );
    }
}
