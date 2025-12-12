use crate::poly::unipoly::UniPoly;
use crate::subprotocols::sumcheck_verifier::SumcheckInstanceParams;
use crate::transcripts::Transcript;

use crate::{
    field::{JoltField, MaybeAllocative},
    poly::opening_proof::ProverOpeningAccumulator,
};

pub trait SumcheckInstanceProver<F: JoltField, T: Transcript>:
    Send + Sync + MaybeAllocative
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        unimplemented!(
            "If get_params is unimplemented, degree, num_rounds, and \
            input_claim should be implemented directly"
        )
    }

    /// Returns the maximum degree of the sumcheck polynomial.
    fn degree(&self) -> usize {
        self.get_params().degree()
    }

    /// Returns the number of rounds/variables in this sumcheck instance.
    fn num_rounds(&self) -> usize {
        self.get_params().num_rounds()
    }

    /// Returns the initial claim of this sumcheck instance.
    fn input_claim(&self, accumulator: &ProverOpeningAccumulator<F>) -> F {
        self.get_params().input_claim(accumulator)
    }

    /// Computes the prover's message for a specific round of the sumcheck protocol.
    fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F>;

    /// Ingest the verifier's challenge for a sumcheck round.
    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize);

    /// Caches polynomial opening claims needed after the sumcheck protocol completes.
    /// These openings will later be proven using either an opening proof or another sumcheck.
    fn cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    );

    /// Returns trusted advice dimensions if this is a trusted advice polynomial.
    /// Returns `Some((log_rows, log_columns))` for trusted advice, `None` otherwise.
    /// For trusted advice polynomials, binding happens in two separate phases:
    /// - First phase: bind the row variables (last `log_rows` of the row rounds)
    /// - Second phase: bind the column variables (last `log_columns` of the column rounds)
    fn trusted_advice_dimensions(&self) -> Option<(usize, usize)> {
        None
    }

    /// Returns a debug name for this sumcheck instance (for logging purposes).
    fn debug_name(&self) -> String {
        "unknown".to_string()
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder);
}
