use std::any::Any;

use akita_error::AkitaError;
use akita_prover::compute::{
    CommitInnerPlan, DecomposeFoldBatchPlan, DecomposeFoldPlan, OpeningBatchKernel,
    OpeningFoldKernel, OpeningFoldOutput, OpeningFoldPlan, SubringCoefficientPackingBatchKernel,
    SubringCoefficientPackingPartials, SubringCoefficientPackingPlan,
};
use akita_prover::{
    cpu_external_inner_commitment_capability, cpu_external_inner_prepared_setup,
    AvailablePolynomialTypes, BackendKindId, BatchDecomposeFoldOutcome, CommitInnerWitness,
    CommitSourceClass, CommitSourceDescriptor, CommitmentSource, CpuBackend, CpuPreparedSetup,
    DecomposeFoldWitness, ExternalInnerCommitmentCapability, ExternalInnerCommitmentInput,
    ExternalInnerCommitmentOperation, ExternalOperationIdentity, PolynomialRepresentation,
    PolynomialTypeSelection, PreparedExternalInnerCommitment,
};
use akita_types::FpExtEncoding;
use jolt_field::ExtField;
use rayon::prelude::*;

use super::commit::commit_packed;
use super::decomposition::decompose_fold_packed;
use super::opening::opening_fold_packed;
use super::source::{TracePackedOneHot, TracePackedOneHotBatchView, TracePackedOneHotView};
use super::traversal::coefficient_packing_partials_packed;
use crate::AkitaField;

struct TracePackedCommitAlgorithm;

struct TracePackedCommitOperation;

static TRACE_PACKED_COMMIT_OPERATION: TracePackedCommitOperation = TracePackedCommitOperation;

impl ExternalInnerCommitmentOperation<AkitaField> for TracePackedCommitOperation {
    fn identity(&self) -> ExternalOperationIdentity {
        ExternalOperationIdentity::of::<
            TracePackedOneHot,
            TracePackedCommitAlgorithm,
            CpuPreparedSetup<AkitaField>,
        >()
    }

    fn commit_group(
        &self,
        plan: &CommitInnerPlan,
        sources: &[ExternalInnerCommitmentInput<'_>],
        context: &dyn Any,
    ) -> Result<Vec<CommitInnerWitness<AkitaField>>, AkitaError> {
        let prepared = cpu_external_inner_prepared_setup::<AkitaField>(context)?;
        akita_types::dispatch_for_field!(
            akita_types::ProtocolDispatchSlot::Role(akita_types::RingRole::Inner),
            AkitaField,
            plan.ring_dimension,
            |D| sources
                .par_iter()
                .map(|source| {
                    commit_packed::<D>(
                        &CpuBackend::DEFAULT,
                        prepared,
                        source.payload::<TracePackedOneHot>()?,
                        *plan,
                    )
                })
                .collect()
        )
    }
}

impl CommitmentSource<AkitaField> for TracePackedOneHot {
    fn descriptor(&self) -> Result<CommitSourceDescriptor, AkitaError> {
        let logical_len = self.total_field_elems();
        CommitSourceDescriptor::new(
            self.num_vars,
            logical_len,
            logical_len,
            CommitSourceClass::OneHot {
                chunk_size: self.one_hot_k,
            },
            "jolt_trace_onehot",
        )
    }

    fn committed_centered_reach(
        &self,
        _modulus: u128,
        _centering_threshold: u128,
    ) -> Result<(u128, u128), AkitaError> {
        Ok((0, 1))
    }

    fn available_polynomial_types(
        &self,
        _plan: &CommitInnerPlan,
    ) -> Result<AvailablePolynomialTypes, AkitaError> {
        AvailablePolynomialTypes::new(Vec::new())
    }

    fn represent_as(
        &self,
        _selected: PolynomialTypeSelection,
        _plan: &CommitInnerPlan,
    ) -> Result<PolynomialRepresentation<'_, AkitaField>, AkitaError> {
        Err(AkitaError::InvalidInput(
            "trace one-hot uses its streamed external commitment operation".to_string(),
        ))
    }

    fn external_inner_commitment_capability(
        &self,
        backend: BackendKindId,
        _plan: &CommitInnerPlan,
    ) -> Result<Option<ExternalInnerCommitmentCapability>, AkitaError> {
        let capability = cpu_external_inner_commitment_capability::<
            TracePackedOneHot,
            TracePackedCommitAlgorithm,
            AkitaField,
        >("jolt-trace-onehot")?;
        Ok((backend == capability.backend()).then_some(capability))
    }

    fn prepare_external_inner_commitment(
        &self,
        selected: ExternalInnerCommitmentCapability,
        _plan: &CommitInnerPlan,
    ) -> Result<PreparedExternalInnerCommitment<'_, AkitaField>, AkitaError> {
        PreparedExternalInnerCommitment::new(selected, self, &TRACE_PACKED_COMMIT_OPERATION, None)
    }
}

impl<const D: usize> OpeningFoldKernel<TracePackedOneHotView<'_, D>, AkitaField, D> for CpuBackend {
    fn evaluate_and_fold(
        &self,
        _prepared: Option<&Self::PreparedSetup>,
        source: TracePackedOneHotView<'_, D>,
        plan: OpeningFoldPlan<'_, AkitaField>,
    ) -> Result<OpeningFoldOutput<AkitaField, D>, AkitaError> {
        opening_fold_packed(source.source(), plan)
    }

    fn decompose_fold(
        &self,
        _prepared: Option<&Self::PreparedSetup>,
        source: TracePackedOneHotView<'_, D>,
        plan: DecomposeFoldPlan<'_>,
    ) -> Result<DecomposeFoldWitness<AkitaField>, AkitaError> {
        decompose_fold_packed::<D>(
            source.source(),
            plan.challenges,
            plan.num_positions_per_block,
            plan.num_digits,
        )
    }
}

impl<const D: usize> OpeningBatchKernel<TracePackedOneHotBatchView<'_, D>, AkitaField, D>
    for CpuBackend
{
    fn decompose_fold_batch(
        &self,
        _prepared: Option<&Self::PreparedSetup>,
        source: TracePackedOneHotBatchView<'_, D>,
        plan: DecomposeFoldBatchPlan<'_>,
    ) -> Result<BatchDecomposeFoldOutcome<AkitaField, D>, AkitaError> {
        let source = source.source();
        match plan {
            DecomposeFoldBatchPlan::Sparse {
                challenges,
                num_positions_per_block,
                num_digits,
                ..
            } => Ok(BatchDecomposeFoldOutcome::Fused(
                decompose_fold_packed::<D>(
                    source,
                    challenges,
                    num_positions_per_block,
                    num_digits,
                )?,
            )),
        }
    }
}

impl<E, const D: usize>
    SubringCoefficientPackingBatchKernel<TracePackedOneHotBatchView<'_, D>, AkitaField, E, D>
    for CpuBackend
where
    E: ExtField<AkitaField> + FpExtEncoding<AkitaField>,
{
    fn coefficient_packing_partials_batch(
        &self,
        _prepared: Option<&Self::PreparedSetup>,
        source: TracePackedOneHotBatchView<'_, D>,
        plan: SubringCoefficientPackingPlan<'_, E>,
    ) -> Result<Vec<SubringCoefficientPackingPartials<AkitaField>>, AkitaError> {
        source
            .sources
            .iter()
            .map(|source| {
                let coordinates = coefficient_packing_partials_packed::<E, D>(source, plan)?;
                SubringCoefficientPackingPartials::new(
                    plan.point.geometry(),
                    plan.point.num_live_blocks(),
                    coordinates,
                )
            })
            .collect()
    }
}
