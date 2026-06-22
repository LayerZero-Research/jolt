use allocative::Allocative;
use rayon::prelude::*;
use tracer::instruction::Cycle;

use common::constants::XLEN;

use crate::field::JoltField;
use crate::poly::eq_poly::EqPolynomial;
use crate::poly::multilinear_polynomial::{BindingOrder, MultilinearPolynomial, PolynomialBinding};
use crate::poly::opening_proof::{
    AbstractVerifierOpeningAccumulator, OpeningAccumulator, OpeningPoint, ProverOpeningAccumulator,
    SumcheckId, BIG_ENDIAN, LITTLE_ENDIAN,
};
use crate::poly::unipoly::UniPoly;
use crate::subprotocols::sumcheck_prover::SumcheckInstanceProver;
use crate::subprotocols::sumcheck_verifier::{SumcheckInstanceParams, SumcheckInstanceVerifier};
use crate::transcripts::Transcript;
use crate::utils::math::Math;
use crate::zkvm::witness::{unsigned_inc, VirtualPolynomial};

const DEGREE_BOUND: usize = 2;

#[derive(Allocative, Clone)]
pub struct UnsignedIncClaimReductionParams<F: JoltField> {
    pub n_cycle_vars: usize,
    pub r_cycle_inc: OpeningPoint<BIG_ENDIAN, F>,
}

impl<F: JoltField> UnsignedIncClaimReductionParams<F> {
    pub fn new(trace_len: usize, accumulator: &dyn OpeningAccumulator<F>) -> Self {
        let (r_cycle_inc, _) = accumulator
            .get_virtual_polynomial_opening(VirtualPolynomial::Inc, SumcheckId::IncVirtualization);

        Self {
            n_cycle_vars: trace_len.log_2(),
            r_cycle_inc,
        }
    }
}

impl<F: JoltField> SumcheckInstanceParams<F> for UnsignedIncClaimReductionParams<F> {
    fn input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        let (_, inc_claim) = accumulator
            .get_virtual_polynomial_opening(VirtualPolynomial::Inc, SumcheckId::IncVirtualization);
        inc_claim + F::from_u128(1u128 << XLEN)
    }

    fn degree(&self) -> usize {
        DEGREE_BOUND
    }

    fn num_rounds(&self) -> usize {
        self.n_cycle_vars
    }

    fn normalize_opening_point(&self, challenges: &[F::Challenge]) -> OpeningPoint<BIG_ENDIAN, F> {
        OpeningPoint::<LITTLE_ENDIAN, F>::new(challenges.to_vec()).match_endianness()
    }
}

#[derive(Allocative)]
pub struct UnsignedIncClaimReductionProver<F: JoltField> {
    unsigned_inc: MultilinearPolynomial<F>,
    eq: MultilinearPolynomial<F>,
    pub params: UnsignedIncClaimReductionParams<F>,
}

impl<F: JoltField> UnsignedIncClaimReductionProver<F> {
    #[tracing::instrument(skip_all, name = "UnsignedIncClaimReductionProver::initialize")]
    pub fn initialize(params: UnsignedIncClaimReductionParams<F>, trace: &[Cycle]) -> Self {
        let coeffs: Vec<F> = trace
            .par_iter()
            .map(|cycle| F::from_u128(unsigned_inc(cycle)))
            .collect();
        let eq = EqPolynomial::evals(&params.r_cycle_inc.r);

        Self {
            unsigned_inc: coeffs.into(),
            eq: eq.into(),
            params,
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T>
    for UnsignedIncClaimReductionProver<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    #[tracing::instrument(skip_all, name = "UnsignedIncClaimReductionProver::compute_message")]
    fn compute_message(&mut self, _round: usize, previous_claim: F) -> UniPoly<F> {
        let half_n = self.unsigned_inc.len() / 2;
        let evals = (0..half_n)
            .into_par_iter()
            .fold(
                || [F::zero(); DEGREE_BOUND],
                |mut acc, j| {
                    let unsigned_inc = self
                        .unsigned_inc
                        .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                    let eq = self
                        .eq
                        .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);

                    for k in 0..DEGREE_BOUND {
                        acc[k] += unsigned_inc[k] * eq[k];
                    }
                    acc
                },
            )
            .reduce(
                || [F::zero(); DEGREE_BOUND],
                |mut a, b| {
                    for k in 0..DEGREE_BOUND {
                        a[k] += b[k];
                    }
                    a
                },
            );

        UniPoly::from_evals_and_hint(previous_claim, &evals)
    }

    fn ingest_challenge(&mut self, r_j: F::Challenge, _round: usize) {
        rayon::join(
            || {
                self.unsigned_inc
                    .bind_parallel(r_j, BindingOrder::LowToHigh);
            },
            || {
                self.eq.bind_parallel(r_j, BindingOrder::LowToHigh);
            },
        );
    }

    fn cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let opening_point = self.params.normalize_opening_point(sumcheck_challenges);
        accumulator.append_virtual(
            VirtualPolynomial::UnsignedInc,
            SumcheckId::UnsignedIncClaimReduction,
            opening_point,
            self.unsigned_inc.final_sumcheck_claim(),
        );
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
        flamegraph.visit_root(self);
    }
}

pub struct UnsignedIncClaimReductionVerifier<F: JoltField> {
    params: UnsignedIncClaimReductionParams<F>,
}

impl<F: JoltField> UnsignedIncClaimReductionVerifier<F> {
    pub fn new(trace_len: usize, accumulator: &dyn OpeningAccumulator<F>) -> Self {
        Self {
            params: UnsignedIncClaimReductionParams::new(trace_len, accumulator),
        }
    }
}

impl<F: JoltField, T: Transcript, A: AbstractVerifierOpeningAccumulator<F>>
    SumcheckInstanceVerifier<F, T, A> for UnsignedIncClaimReductionVerifier<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    fn expected_output_claim(&self, accumulator: &A, sumcheck_challenges: &[F::Challenge]) -> F {
        let opening_point = self.params.normalize_opening_point(sumcheck_challenges);
        let eq = EqPolynomial::mle(&opening_point.r, &self.params.r_cycle_inc.r);
        let (_, unsigned_inc_claim) = accumulator.get_virtual_polynomial_opening(
            VirtualPolynomial::UnsignedInc,
            SumcheckId::UnsignedIncClaimReduction,
        );

        unsigned_inc_claim * eq
    }

    fn cache_openings(&self, accumulator: &mut A, sumcheck_challenges: &[F::Challenge]) {
        let opening_point = self.params.normalize_opening_point(sumcheck_challenges);
        accumulator.append_virtual(
            VirtualPolynomial::UnsignedInc,
            SumcheckId::UnsignedIncClaimReduction,
            opening_point,
        );
    }
}
