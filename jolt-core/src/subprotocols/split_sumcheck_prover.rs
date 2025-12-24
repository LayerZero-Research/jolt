use crate::{
    field::JoltField,
    subprotocols::{
        sumcheck_prover::SumcheckInstanceProver,
    },
    transcripts::Transcript,
};


/// Default number of rounds to use for the partially-bound sumcheck phase.
const DEFAULT_SPLIT_LOWER_ROUNDS: usize = 8;

/// Extension of `SumcheckInstanceProver` for instances that can be used with `SplitSumcheckInstance`.
/// Adds methods to extract the remainder polynomials and expression closure needed for the
/// partially-bound sumcheck phase.
pub trait SplitSumcheckInstanceInner<F: JoltField, T: Transcript, P>:
    SumcheckInstanceProver<F, T>
{
    /// Creates a new instance for the lower rounds of the split sumcheck.
    /// Called when transitioning to the partially-bound sumcheck phase.
    /// This is a static constructor that takes params, remainder polynomials, and round number.
    fn initialize_lower_rounds(params: P, remainder: Vec<Vec<F>>, round_number: usize) -> Self;

    /// Returns the number of final rounds to use for the partially-bound sumcheck phase.
    /// Override this to customize when the split happens for each sumcheck instance.
    fn split_lower_rounds(&self) -> usize {
        DEFAULT_SPLIT_LOWER_ROUNDS
    }
}