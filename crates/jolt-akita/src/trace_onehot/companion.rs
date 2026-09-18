use std::cell::RefCell;

use akita_challenges::SparseChallenge;
use akita_error::AkitaError;

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
    challenges: Option<TraceFoldChallenges>,
}

thread_local! {
    static CAPTURE: RefCell<CaptureState> = RefCell::new(CaptureState::default());
}

/// Arms capture of the root trace's sparse decompose-fold challenges for a
/// comparison companion. The proving algorithm and transcript are unchanged.
pub fn begin_trace_fold_challenge_capture() {
    CAPTURE.with(|capture| {
        *capture.borrow_mut() = CaptureState {
            armed: true,
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

pub(crate) fn trace_fold_capture_requested() -> bool {
    CAPTURE.with(|capture| capture.borrow().armed)
}

pub(crate) fn with_worker_trace_fold_capture<R>(
    requested: bool,
    f: impl FnOnce() -> R,
) -> (R, Option<TraceFoldChallenges>) {
    CAPTURE.with(|capture| {
        let previous = std::mem::replace(
            &mut *capture.borrow_mut(),
            CaptureState {
                armed: requested,
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
) -> Result<(), AkitaError> {
    CAPTURE.with(|capture| {
        let mut state = capture.borrow_mut();
        if !state.armed || state.challenges.is_some() {
            return Ok(());
        }
        let subring_dimension = u32::try_from(D).map_err(|_| {
            AkitaError::InvalidInput("challenge subring dimension does not fit u32".to_string())
        })?;
        let challenge_count = u32::try_from(challenges.len()).map_err(|_| {
            AkitaError::InvalidInput("challenge count does not fit u32".to_string())
        })?;
        let mut offsets = Vec::with_capacity(challenges.len() + 1);
        let mut positions = Vec::new();
        let mut coefficients = Vec::new();
        offsets.push(0);
        for challenge in challenges {
            positions.extend(
                challenge
                    .positions
                    .iter()
                    .map(|&position| u32::from(position)),
            );
            coefficients.extend(challenge.coeffs.iter().copied());
            offsets.push(u32::try_from(positions.len()).map_err(|_| {
                AkitaError::InvalidInput("challenge term count does not fit u32".to_string())
            })?);
        }
        debug_assert_eq!(offsets.len(), challenge_count as usize + 1);
        state.challenges = Some(TraceFoldChallenges {
            subring_dimension,
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

    #[test]
    fn sparse_challenges_cross_the_backend_pool_boundary() {
        begin_trace_fold_challenge_capture();
        let requested = trace_fold_capture_requested();
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
            capture_trace_fold_challenges::<64>(&challenges).unwrap();
        });
        publish_trace_fold_challenges(captured);

        let captured = take_trace_fold_challenges().unwrap();
        assert_eq!(captured.subring_dimension, 64);
        assert_eq!(captured.offsets, [0, 2, 3]);
        assert_eq!(captured.positions, [1, 7, 3]);
        assert_eq!(captured.coefficients, [1, -1, 1]);
    }
}
