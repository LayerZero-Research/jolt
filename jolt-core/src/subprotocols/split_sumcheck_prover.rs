use allocative::Allocative;

use crate::{
    field::JoltField,
    poly::{
        multilinear_polynomial::BindingOrder,
        opening_proof::ProverOpeningAccumulator,
        unipoly::UniPoly,
    },
    subprotocols::{
        partially_bound_sumcheck::PartiallyBoundSumcheck,
        sumcheck_prover::SumcheckInstanceProver,
    },
    transcripts::Transcript,
};


#[derive(Allocative)]
pub struct SplitSumcheckInstance<F: JoltField, T: Transcript> {
    #[allocative(skip)]
    inner: Box<dyn SplitSumcheckInstanceInner<F, T>>,
    lower_rounds: usize,
    #[allow(dead_code)]
    binding_order: BindingOrder,
    #[allocative(skip)]
    pb: Option<PartiallyBoundSumcheck<F>>,
}

impl<F: JoltField, T: Transcript> SplitSumcheckInstance<F, T> {
    pub fn new(
        inner: Box<dyn SplitSumcheckInstanceInner<F, T>>,
        lower_rounds: usize,
        binding_order: BindingOrder,
    ) -> Self {
        Self {
            inner,
            lower_rounds,
            binding_order,
            pb: None,
        }
    }

    // remainder can be provided from outside
    pub fn initialize_partially_bound_sumcheck(&mut self, remainder: Vec<Vec<F>>) {
        let expr = self.inner.create_expr();
        self.pb = Some(PartiallyBoundSumcheck::new(
            remainder,
            self.degree(),
            expr,
        ));
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T> for SplitSumcheckInstance<F, T> {
    fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
        if self.inner.num_rounds() - round <= self.lower_rounds && self.pb.is_none() {
            self.initialize_partially_bound_sumcheck(self.inner.create_remainder());
        }

        if let Some(ref pb) = self.pb {
            // assert_eq!(
            //     self.pb.as_ref().unwrap().compute_message(previous_claim), 
            //     self.inner.compute_message(round, previous_claim), 
            //     "compute_message mismatch {round}");

            pb.compute_message(previous_claim)
        } else {
            self.inner.compute_message(round, previous_claim)
        }
    }

    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        if let Some(ref mut pb) = self.pb {
            pb.ingest_challenge(r_j);
        } else {
            self.inner.ingest_challenge(r_j, round);
        }
    }

    fn cache_openings(&self, accumulator: &mut ProverOpeningAccumulator<F>, transcript: &mut T, sumcheck_challenges: &[F::Challenge]) {
        // Get the final polynomial claims from the partially-bound sumcheck
        let poly_claims = self.pb.as_ref()
            .expect("cache_openings called before PartiallyBoundSumcheck was created")
            .final_poly_claims();
        
        self.inner.cache_openings_with_claims(accumulator, transcript, sumcheck_challenges, &poly_claims)
    }

    // Note: We intentionally don't implement get_params() here.
    // If the inner prover implements get_params(), callers can use it via inner.
    // We implement degree() and num_rounds() directly to avoid requiring inner to have get_params().

    fn degree(&self) -> usize {
        self.inner.degree()
    }

    fn num_rounds(&self) -> usize {
        self.inner.num_rounds()
    }

    fn input_claim(&self, accumulator: &ProverOpeningAccumulator<F>) -> F {
        self.inner.input_claim(accumulator)
    }
}


/// Extension of `SumcheckInstanceProver` for instances that can be used with `SplitSumcheckInstance`.
/// Adds methods to extract the remainder polynomials and expression closure needed for the
/// partially-bound sumcheck phase.
pub trait SplitSumcheckInstanceInner<F: JoltField, T: Transcript>:
    SumcheckInstanceProver<F, T>
{
    /// Creates the remainder polynomials for the partially-bound sumcheck phase.
    /// Each inner `Vec<F>` represents the evaluations of a polynomial over the remaining variables.
    fn create_remainder(&self) -> Vec<Vec<F>>;

    /// Creates the expression closure that computes the sumcheck polynomial value
    /// given the evaluations of each polynomial at a point.
    fn create_expr(&self) -> Box<dyn Fn(&[F]) -> F + Send + Sync>;

    /// Caches polynomial opening claims using the provided final polynomial claims.
    /// The `poly_claims` correspond to the final evaluations of each polynomial in
    /// the same order as returned by `create_remainder`.
    fn cache_openings_with_claims(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
        poly_claims: &[F],
    );
}