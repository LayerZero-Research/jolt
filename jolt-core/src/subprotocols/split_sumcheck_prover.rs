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
        sumcheck_verifier::SumcheckInstanceParams,
    },
    transcripts::Transcript,
};


#[derive(Allocative)]
pub struct SplitSumcheckInstance<F: JoltField, T: Transcript, P, I>
where
    I: SplitSumcheckInstanceInner<F, T, P>,
{
    #[allocative(skip)]
    inner: I,
    #[allocative(skip)]
    params: Option<P>,
    lower_rounds: usize,
    #[allow(dead_code)]
    binding_order: BindingOrder,
    #[allocative(skip)]
    pb: Option<PartiallyBoundSumcheck<F>>,
    #[allocative(skip)]
    _marker: std::marker::PhantomData<fn(P) -> T>,
}

impl<F: JoltField, T: Transcript, P, I> SplitSumcheckInstance<F, T, P, I>
where
    I: SplitSumcheckInstanceInner<F, T, P>,
{
    pub fn new(
        inner: I,
        params: P,
        lower_rounds: usize,
        binding_order: BindingOrder,
    ) -> Self {
        Self {
            inner,
            params: Some(params),
            lower_rounds,
            binding_order,
            pb: None,
            _marker: std::marker::PhantomData,
        }
    }

    // remainder can be provided from outside
    pub fn initialize_partially_bound_sumcheck(&mut self, remainder: Vec<Vec<F>>) {
        let expr = self.inner.create_expr();
        self.pb = Some(PartiallyBoundSumcheck::new(
            remainder,
            self.inner.degree(),
            expr,
        ));
    }
}

impl<F: JoltField, T: Transcript, P, I> SumcheckInstanceProver<F, T> for SplitSumcheckInstance<F, T, P, I>
where
    P: Send + Sync,
    I: SplitSumcheckInstanceInner<F, T, P>,
{
    fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
        if self.inner.num_rounds() - round <= self.lower_rounds && self.pb.is_none() {
            let x = self.inner.create_remainder();
            // let params = self.params.take().expect("params already consumed");
            // self.inner = I::initialize_lower_rounds(params, x.clone(), round);
            self.initialize_partially_bound_sumcheck(x);
        }

        if let Some(ref mut pb) = self.pb {
            debug_assert_eq!(
                pb.compute_message(previous_claim), 
                self.inner.compute_message(round, previous_claim), 
                "compute_message mismatch {round}");

            pb.compute_message(previous_claim)
        } else {
            self.inner.compute_message(round, previous_claim)
        }
    }

    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        if let Some(ref mut pb) = self.pb {
            self.inner.ingest_challenge(r_j, round);
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
        // assert_eq!(self.inner.cache_openings_with_claims(accumulator, transcript, sumcheck_challenges, &poly_claims), 
        // self.inner.cache_openings(accumulator, transcript, sumcheck_challenges));
        
        self.inner.cache_openings_with_claims(accumulator, transcript, sumcheck_challenges, &poly_claims)
        // self.inner.cache_openings(accumulator, transcript, sumcheck_challenges)
    }

    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        self.inner.get_params()
    }

    fn degree(&self) -> usize {
        self.inner.degree()
    }

    fn num_rounds(&self) -> usize {
        self.inner.num_rounds()
    }
}


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

    /// Returns the number of final rounds to use for the partially-bound sumcheck phase.
    /// Override this to customize when the split happens for each sumcheck instance.
    fn split_lower_rounds(&self) -> usize {
        DEFAULT_SPLIT_LOWER_ROUNDS
    }

    /// Wraps this prover in a `SplitSumcheckInstance` for the final rounds.
    fn into_split(self, params: P) -> SplitSumcheckInstance<F, T, P, Self>
    where
        Self: Sized,
    {
        let lower_rounds = self.split_lower_rounds();
        SplitSumcheckInstance::new(
            self,
            params,
            lower_rounds,
            BindingOrder::LowToHigh,
        )
    }
}