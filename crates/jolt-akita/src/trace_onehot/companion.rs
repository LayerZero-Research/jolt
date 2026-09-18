use std::cell::RefCell;

use akita_algebra::ring::cyclotomic::BalancedDecomposePow2Params;
use akita_algebra::CyclotomicRing;
use akita_challenges::SparseChallenge;
use akita_error::AkitaError;
use akita_prover::compute::{
    DigitRowsComputeBackend, SubringCoefficientPackingBatchKernel,
    SubringCoefficientPackingPartials, SubringCoefficientPackingPlan,
};
use akita_prover::{CpuBackend, DecomposeFoldWitness, RootOpeningSource, RootPolyShape};
use akita_types::{
    BasisMode, CompressionChainPlan, DigitBlocks, OpeningMethod,
    PreparedSubringCoefficientPackingPoint, SubringCoefficientPackingGeometry,
};
use jolt_field::{CanonicalEncoding, One};

use super::traversal::coefficient_packing_partials_packed;
use crate::adapters::{AkitaHintPolynomials, AkitaProverHint};
use crate::{AkitaField, AkitaProverSetup};

#[derive(Debug, Clone)]
pub struct TraceFoldChallenges {
    pub subring_dimension: u32,
    pub offsets: Vec<u32>,
    pub positions: Vec<u32>,
    pub coefficients: Vec<i8>,
    pub z_coefficients: Vec<i32>,
}

#[derive(Default)]
struct CaptureState {
    armed: bool,
    challenge_subring_dimension: Option<u32>,
    challenges: Option<TraceFoldChallenges>,
}

thread_local! {
    static CAPTURE: RefCell<CaptureState> = RefCell::new(CaptureState::default());
}

/// Arms capture of the root trace's sparse decompose-fold challenges for a
/// comparison companion. The proving algorithm and transcript are unchanged.
pub fn begin_trace_fold_challenge_capture(challenge_subring_dimension: u32) {
    CAPTURE.with(|capture| {
        *capture.borrow_mut() = CaptureState {
            armed: true,
            challenge_subring_dimension: Some(challenge_subring_dimension),
            challenges: None,
        };
    });
}

/// Takes the challenges captured by the next root trace decompose-fold.
pub fn take_trace_fold_challenges() -> Result<TraceFoldChallenges, AkitaError> {
    CAPTURE.with(|capture| {
        let mut state = capture.borrow_mut();
        state.armed = false;
        state.challenges.take().ok_or_else(|| {
            AkitaError::InvalidInput(
                "the Akita trace opening produced no decompose-fold challenges".to_string(),
            )
        })
    })
}

pub(crate) fn trace_fold_capture_request() -> Option<u32> {
    CAPTURE.with(|capture| {
        let state = capture.borrow();
        state.armed.then_some(state.challenge_subring_dimension).flatten()
    })
}

pub(crate) fn with_worker_trace_fold_capture<R>(
    requested: Option<u32>,
    f: impl FnOnce() -> R,
) -> (R, Option<TraceFoldChallenges>) {
    CAPTURE.with(|capture| {
        let previous = std::mem::replace(
            &mut *capture.borrow_mut(),
            CaptureState {
                armed: requested.is_some(),
                challenge_subring_dimension: requested,
                challenges: None,
            },
        );
        let result = f();
        let captured = capture.borrow_mut().challenges.take();
        *capture.borrow_mut() = previous;
        (result, captured)
    })
}

pub(crate) fn publish_trace_fold_challenges(challenges: Option<TraceFoldChallenges>) {
    CAPTURE.with(|capture| {
        let mut state = capture.borrow_mut();
        if state.armed {
            state.challenges = challenges;
        }
    });
}

pub(super) fn capture_trace_fold_challenges<const D: usize>(
    challenges: &[SparseChallenge],
    witness: &DecomposeFoldWitness<AkitaField>,
) -> Result<(), AkitaError> {
    CAPTURE.with(|capture| {
        let mut state = capture.borrow_mut();
        if !state.armed || state.challenges.is_some() {
            return Ok(());
        }
        let subring_dimension = state.challenge_subring_dimension.ok_or_else(|| {
            AkitaError::InvalidInput("challenge subring dimension was not armed".to_string())
        })?;
        let subring_dimension = usize::try_from(subring_dimension).map_err(|_| {
            AkitaError::InvalidInput("challenge subring dimension does not fit usize".to_string())
        })?;
        if subring_dimension == 0 || !D.is_multiple_of(subring_dimension) {
            return Err(AkitaError::InvalidInput(
                "challenge subring dimension does not divide the fold ring".to_string(),
            ));
        }
        let embedding_stride = D / subring_dimension;
        for challenge in challenges {
            challenge.validate::<D>()?;
            if challenge
                .positions
                .iter()
                .any(|position| {
                    usize::try_from(*position)
                        .map_or(true, |position| position % embedding_stride != 0)
                })
            {
                return Err(AkitaError::InvalidInput(
                    "sparse challenge is outside the scheduled subring embedding".to_string(),
                ));
            }
        }
        let challenge_count = u32::try_from(challenges.len()).map_err(|_| {
            AkitaError::InvalidInput("challenge count does not fit u32".to_string())
        })?;
        let mut offsets = Vec::with_capacity(challenges.len() + 1);
        let mut positions = Vec::new();
        let mut coefficients = Vec::new();
        offsets.push(0);
        for challenge in challenges {
            positions.extend(challenge.positions.iter().map(|&position| {
                u32::try_from(
                    usize::try_from(position)
                        .expect("a challenge position fits usize")
                        / embedding_stride,
                )
                    .expect("a u8 challenge position divided by its stride fits u32")
            }));
            coefficients.extend(challenge.coeffs.iter().copied());
            offsets.push(u32::try_from(positions.len()).map_err(|_| {
                AkitaError::InvalidInput("challenge term count does not fit u32".to_string())
            })?);
        }
        debug_assert_eq!(offsets.len(), challenge_count as usize + 1);
        state.challenges = Some(TraceFoldChallenges {
            subring_dimension: u32::try_from(subring_dimension)
                .expect("the capture subring dimension originated as u32"),
            offsets,
            positions,
            coefficients,
            z_coefficients: witness.centered_coeffs_flat().to_vec(),
        });
        Ok(())
    })
}

/// Reconstruct the uncompressed outer commitment image retained by Rust Akita's
/// real root commitment hint.
pub fn reference_commit_u(hint: &AkitaProverHint) -> Result<Vec<AkitaField>, AkitaError> {
    let (committed, backend_hint) = hint.backend.as_ref().ok_or_else(|| {
        AkitaError::InvalidInput("the Akita companion hint has no backend commitment".into())
    })?;
    let profile = committed.profile();
    let coefficient_count = profile
        .outer_slice_count
        .get()
        .checked_mul(profile.outer.matrix.output_rank())
        .and_then(|count| count.checked_mul(profile.outer.matrix.ring_dimension()))
        .ok_or_else(|| AkitaError::InvalidInput("outer image length overflow".into()))?;
    let plan = CompressionChainPlan::for_complete_source(
        profile.outer.matrix.sis_modulus_profile(),
        coefficient_count,
    )?;
    let witness = backend_hint.outer_compression_witness(&plan)?;
    witness
        .stages()
        .first()
        .ok_or(AkitaError::InvalidProof)?
        .recompose::<AkitaField>()
}

/// Compute the Rust Akita trace group's `v = D * e_hat` reference at the
/// exact opening point sent to the GPU companion.
pub fn reference_compute_v(
    setup: &AkitaProverSetup,
    hint: &AkitaProverHint,
    params: &akita_types::CommittedGroupParams,
    opening_point: &[AkitaField],
) -> Result<Vec<AkitaField>, AkitaError> {
    // The public/Jolt point is HIGH_TO_LOW. Akita's root-opening kernels consume
    // the backend order, matching `prove_one_hot` and the GPU service's internal
    // reversal at its RPC boundary.
    let backend_point = opening_point.iter().rev().copied().collect::<Vec<_>>();
    let challenge_subring_dimension = match params.opening_method() {
        OpeningMethod::SubringCoefficientPacking {
            challenge_subring_dimension,
        } => challenge_subring_dimension,
        OpeningMethod::EvaluationTrace => {
            return Err(AkitaError::InvalidSetup(
                "the Akita companion v reference requires coefficient packing".into(),
            ))
        }
    };
    let (_, prepared) = setup
        .one_hot_backend()
        .map_err(|error| AkitaError::InvalidSetup(error.to_string()))?;
    akita_types::dispatch_for_field!(
        akita_types::ProtocolDispatchSlot::Role(akita_types::RingRole::Inner),
        AkitaField,
        params.inner().matrix.ring_dimension(),
        |D_A| {
            let geometry =
                SubringCoefficientPackingGeometry::try_new(1, D_A, challenge_subring_dimension)?;
            let (source_num_vars, live_positions) = match &hint.polynomials {
                AkitaHintPolynomials::TraceOneHot(source) => {
                    (source.num_vars, source.total_field_elems() / D_A)
                }
                AkitaHintPolynomials::OneHot(sources) => {
                    let source = sources.first().ok_or_else(|| {
                        AkitaError::InvalidInput(
                            "the Akita companion v reference has no one-hot source".into(),
                        )
                    })?;
                    (
                        akita_prover::RootPolyMeta::num_vars(source),
                        <crate::adapters::AkitaBackendOneHotPoly as RootPolyShape<
                            AkitaField,
                            D_A,
                        >>::num_live_ring_elems(source),
                    )
                }
                AkitaHintPolynomials::Dense(_) => {
                    return Err(AkitaError::InvalidInput(
                        "the Akita companion v reference requires a one-hot hint".into(),
                    ));
                }
            };
            let point = PreparedSubringCoefficientPackingPoint::new(
                geometry,
                BasisMode::Lagrange,
                live_positions,
                params.own_group().num_positions_per_block(),
                source_num_vars,
                &backend_point,
            )?;
            let partials = match &hint.polynomials {
                AkitaHintPolynomials::TraceOneHot(source) => {
                    let coordinates = coefficient_packing_partials_packed::<AkitaField, D_A>(
                        source,
                        SubringCoefficientPackingPlan { point: &point },
                    )?;
                    vec![SubringCoefficientPackingPartials::new(
                        geometry,
                        point.num_live_blocks(),
                        coordinates,
                    )?]
                }
                AkitaHintPolynomials::OneHot(sources) => {
                    let refs = sources.iter().collect::<Vec<_>>();
                    let batch = <crate::adapters::AkitaBackendOneHotPoly as RootOpeningSource<
                        AkitaField,
                        D_A,
                    >>::opening_batch(&refs)?;
                    CpuBackend::DEFAULT.coefficient_packing_partials_batch(
                        Some(prepared),
                        batch,
                        SubringCoefficientPackingPlan { point: &point },
                    )?
                }
                AkitaHintPolynomials::Dense(_) => unreachable!(),
            };
            akita_types::dispatch_for_field!(
                akita_types::ProtocolDispatchSlot::Role(akita_types::RingRole::Opening),
                AkitaField,
                params.open().matrix.ring_dimension(),
                |D_D| {
                    let digits = materialize_d_input::<D_D>(params, &partials)?;
                    let rows = CpuBackend::DEFAULT.digit_rows(
                        prepared,
                        params.open().matrix.output_rank(),
                        &[digits.typed_planes::<D_D>()?],
                        params.own_group().log_basis_open(),
                    )?;
                    let [rows] = rows
                        .try_into()
                        .map_err(|_: Vec<_>| AkitaError::InvalidProof)?;
                    Ok::<_, AkitaError>(
                        rows.into_iter()
                            .flat_map(|row| row.coefficients().to_vec())
                            .collect(),
                    )
                }
            )
        }
    )
}

fn materialize_d_input<const D: usize>(
    params: &akita_types::CommittedGroupParams,
    partials_by_claim: &[SubringCoefficientPackingPartials<AkitaField>],
) -> Result<DigitBlocks, AkitaError> {
    let partials = partials_by_claim
        .first()
        .ok_or_else(|| AkitaError::InvalidInput("D input has no partials".into()))?;
    let geometry = partials.geometry();
    if !geometry.partial_base_field_width().is_multiple_of(D) {
        return Err(AkitaError::InvalidSetup(
            "coefficient-packing width is not divisible by the D ring".into(),
        ));
    }
    let subcolumns = geometry.partial_base_field_width() / D;
    let num_digits = params.own_group().num_digits_open();
    let planes_per_block = subcolumns
        .checked_mul(num_digits)
        .ok_or_else(|| AkitaError::InvalidInput("D input plane count overflow".into()))?;
    if partials_by_claim.iter().any(|candidate| {
        candidate.geometry() != geometry
            || candidate.num_live_blocks() != partials.num_live_blocks()
    }) {
        return Err(AkitaError::InvalidInput(
            "D input partials have inconsistent geometry".into(),
        ));
    }
    let semantic_blocks = partials_by_claim
        .len()
        .checked_mul(partials.num_live_blocks())
        .ok_or_else(|| AkitaError::InvalidInput("D input block count overflow".into()))?;
    let mut digits = DigitBlocks::zeroed(vec![planes_per_block; semantic_blocks], D)?;
    let modulus = (-AkitaField::one())
        .to_u128_checked()
        .expect("Akita field modulus fits u128")
        + 1;
    let decomposition =
        BalancedDecomposePow2Params::new(num_digits, params.own_group().log_basis_open(), modulus);
    let planes = digits.typed_planes_mut::<D>()?;
    for (claim, partials) in partials_by_claim.iter().enumerate() {
        for block in 0..partials.num_live_blocks() {
            let block_start = block * geometry.partial_base_field_width();
            for subcolumn in 0..subcolumns {
                let start = block_start + subcolumn * D;
                let ring = CyclotomicRing::<AkitaField, D>::from_slice(
                    &partials.coordinates()[start..start + D],
                );
                let semantic_block = claim * partials.num_live_blocks() + block;
                let plane_start = semantic_block * planes_per_block + subcolumn * num_digits;
                ring.balanced_decompose_pow2_i8_into_with_params(
                    &mut planes[plane_start..plane_start + num_digits],
                    &decomposition,
                );
            }
        }
    }
    Ok(digits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jolt_field::Zero;

    #[test]
    fn sparse_challenges_cross_the_backend_pool_boundary() {
        begin_trace_fold_challenge_capture(64);
        let requested = trace_fold_capture_request();
        let challenges = [
            SparseChallenge {
                positions: vec![1, 7].into(),
                coeffs: vec![1, -1].into(),
            },
            SparseChallenge {
                positions: vec![3].into(),
                coeffs: vec![1].into(),
            },
        ];
        let ((), captured) = with_worker_trace_fold_capture(requested, || {
            let witness = DecomposeFoldWitness::from_coefficient_parts(
                vec![[AkitaField::zero(); 64]],
                vec![[0; 64]],
            );
            capture_trace_fold_challenges::<64>(&challenges, &witness).unwrap();
        });
        publish_trace_fold_challenges(captured);

        let captured = take_trace_fold_challenges().unwrap();
        assert_eq!(captured.subring_dimension, 64);
        assert_eq!(captured.offsets, [0, 2, 3]);
        assert_eq!(captured.positions, [1, 7, 3]);
        assert_eq!(captured.coefficients, [1, -1, 1]);
        assert_eq!(captured.z_coefficients, [0; 64]);
    }
}
