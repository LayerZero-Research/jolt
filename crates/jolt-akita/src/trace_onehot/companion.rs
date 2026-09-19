use std::cell::RefCell;

use crate::AkitaField;
use akita_challenges::SparseChallenge;
use akita_error::AkitaError;
use akita_prover::DecomposeFoldWitness;

#[derive(Debug, Clone)]
pub struct TraceFoldChallenges {
    pub subring_dimension: u32,
    pub offsets: Vec<u32>,
    pub positions: Vec<u32>,
    pub coefficients: Vec<i8>,
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
        state
            .armed
            .then_some(state.challenge_subring_dimension)
            .flatten()
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
    _witness: &DecomposeFoldWitness<AkitaField>,
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
            if challenge.positions.iter().any(|position| {
                usize::try_from(*position).map_or(true, |position| position % embedding_stride != 0)
            }) {
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
                    usize::try_from(position).expect("a challenge position fits usize")
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
        });
        Ok(())
    })
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
    }
}
