use allocative::Allocative;
use rayon::prelude::*;

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
use crate::zkvm::instruction::CircuitFlags;
use crate::zkvm::witness::{CommittedPolynomial, VirtualPolynomial};

const DEGREE_BOUND: usize = 3;

#[derive(Allocative, Clone)]
pub struct IncVirtualizationParams<F: JoltField> {
    pub gamma_powers: [F; 3],
    pub n_cycle_vars: usize,
    pub r_cycle_stage2: OpeningPoint<BIG_ENDIAN, F>,
    pub r_cycle_stage4: OpeningPoint<BIG_ENDIAN, F>,
    pub s_cycle_stage4: OpeningPoint<BIG_ENDIAN, F>,
    pub s_cycle_stage5: OpeningPoint<BIG_ENDIAN, F>,
}

impl<F: JoltField> IncVirtualizationParams<F> {
    pub fn new(
        trace_len: usize,
        accumulator: &dyn OpeningAccumulator<F>,
        transcript: &mut impl Transcript,
    ) -> Self {
        let gamma: F = transcript.challenge_scalar();
        let gamma_sqr = gamma.square();
        let gamma_cub = gamma_sqr * gamma;

        let (r_cycle_stage2, _) = accumulator.get_committed_polynomial_opening(
            CommittedPolynomial::RamInc,
            SumcheckId::RamReadWriteChecking,
        );
        let (r_cycle_stage4, _) = accumulator
            .get_committed_polynomial_opening(CommittedPolynomial::RamInc, SumcheckId::RamValCheck);
        let (s_cycle_stage4, _) = accumulator.get_committed_polynomial_opening(
            CommittedPolynomial::RdInc,
            SumcheckId::RegistersReadWriteChecking,
        );
        let (s_cycle_stage5, _) = accumulator.get_committed_polynomial_opening(
            CommittedPolynomial::RdInc,
            SumcheckId::RegistersValEvaluation,
        );

        Self {
            gamma_powers: [gamma, gamma_sqr, gamma_cub],
            n_cycle_vars: trace_len.log_2(),
            r_cycle_stage2,
            r_cycle_stage4,
            s_cycle_stage4,
            s_cycle_stage5,
        }
    }

    fn opening_coeffs_at(&self, point: &OpeningPoint<BIG_ENDIAN, F>) -> (F, F) {
        let [gamma, gamma_sqr, gamma_cub] = self.gamma_powers;
        let eq_r2 = EqPolynomial::mle(&point.r, &self.r_cycle_stage2.r);
        let eq_r4 = EqPolynomial::mle(&point.r, &self.r_cycle_stage4.r);
        let eq_s4 = EqPolynomial::mle(&point.r, &self.s_cycle_stage4.r);
        let eq_s5 = EqPolynomial::mle(&point.r, &self.s_cycle_stage5.r);

        (eq_r2 + gamma * eq_r4, gamma_sqr * eq_s4 + gamma_cub * eq_s5)
    }
}

impl<F: JoltField> SumcheckInstanceParams<F> for IncVirtualizationParams<F> {
    fn input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        let [gamma, gamma_sqr, gamma_cub] = self.gamma_powers;

        let (_, v_1) = accumulator.get_committed_polynomial_opening(
            CommittedPolynomial::RamInc,
            SumcheckId::RamReadWriteChecking,
        );
        let (_, v_2) = accumulator
            .get_committed_polynomial_opening(CommittedPolynomial::RamInc, SumcheckId::RamValCheck);
        let (_, w_1) = accumulator.get_committed_polynomial_opening(
            CommittedPolynomial::RdInc,
            SumcheckId::RegistersReadWriteChecking,
        );
        let (_, w_2) = accumulator.get_committed_polynomial_opening(
            CommittedPolynomial::RdInc,
            SumcheckId::RegistersValEvaluation,
        );

        v_1 + gamma * v_2 + gamma_sqr * w_1 + gamma_cub * w_2
    }

    fn degree(&self) -> usize {
        DEGREE_BOUND
    }

    fn num_rounds(&self) -> usize {
        self.n_cycle_vars
    }

    fn normalize_opening_point(
        &self,
        challenges: &[<F as JoltField>::Challenge],
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        OpeningPoint::<LITTLE_ENDIAN, F>::new(challenges.to_vec()).match_endianness()
    }
}

#[derive(Allocative)]
pub struct IncVirtualizationProver<F: JoltField> {
    inc: MultilinearPolynomial<F>,
    store: MultilinearPolynomial<F>,
    ram_eq: MultilinearPolynomial<F>,
    rd_eq: MultilinearPolynomial<F>,
    pub params: IncVirtualizationParams<F>,
}

impl<F: JoltField> IncVirtualizationProver<F> {
    #[tracing::instrument(skip_all, name = "IncVirtualizationProver::initialize")]
    pub fn initialize(
        params: IncVirtualizationParams<F>,
        inc: MultilinearPolynomial<F>,
        store: MultilinearPolynomial<F>,
    ) -> Self {
        let [gamma, gamma_sqr, gamma_cub] = params.gamma_powers;

        let ram_eq = {
            let (eq_r2, eq_r4) = rayon::join(
                || EqPolynomial::evals(&params.r_cycle_stage2.r),
                || EqPolynomial::evals(&params.r_cycle_stage4.r),
            );
            eq_r2
                .into_par_iter()
                .zip(eq_r4)
                .map(|(a, b)| a + gamma * b)
                .collect::<Vec<_>>()
        };
        let rd_eq = {
            let (eq_s4, eq_s5) = rayon::join(
                || EqPolynomial::evals(&params.s_cycle_stage4.r),
                || EqPolynomial::evals(&params.s_cycle_stage5.r),
            );
            eq_s4
                .into_par_iter()
                .zip(eq_s5)
                .map(|(a, b)| gamma_sqr * a + gamma_cub * b)
                .collect::<Vec<_>>()
        };

        Self {
            inc,
            store,
            ram_eq: ram_eq.into(),
            rd_eq: rd_eq.into(),
            params,
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T> for IncVirtualizationProver<F> {
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    #[tracing::instrument(skip_all, name = "IncVirtualizationProver::compute_message")]
    fn compute_message(&mut self, _round: usize, previous_claim: F) -> UniPoly<F> {
        let half_n = self.inc.len() / 2;
        let evals = (0..half_n)
            .into_par_iter()
            .fold(
                || [F::zero(); DEGREE_BOUND],
                |mut acc, j| {
                    let inc = self
                        .inc
                        .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                    let store = self
                        .store
                        .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                    let ram_eq = self
                        .ram_eq
                        .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                    let rd_eq = self
                        .rd_eq
                        .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);

                    for k in 0..DEGREE_BOUND {
                        acc[k] +=
                            inc[k] * (ram_eq[k] * store[k] + rd_eq[k] * (F::one() - store[k]));
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
                rayon::join(
                    || self.inc.bind_parallel(r_j, BindingOrder::LowToHigh),
                    || self.store.bind_parallel(r_j, BindingOrder::LowToHigh),
                );
            },
            || {
                rayon::join(
                    || self.ram_eq.bind_parallel(r_j, BindingOrder::LowToHigh),
                    || self.rd_eq.bind_parallel(r_j, BindingOrder::LowToHigh),
                );
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
            VirtualPolynomial::Inc,
            SumcheckId::IncVirtualization,
            opening_point.clone(),
            self.inc.final_sumcheck_claim(),
        );
        accumulator.append_virtual(
            VirtualPolynomial::OpFlags(CircuitFlags::Store),
            SumcheckId::IncVirtualization,
            opening_point,
            self.store.final_sumcheck_claim(),
        );
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
        flamegraph.visit_root(self);
    }
}

pub struct IncVirtualizationVerifier<F: JoltField> {
    params: IncVirtualizationParams<F>,
}

impl<F: JoltField> IncVirtualizationVerifier<F> {
    pub fn new(
        trace_len: usize,
        accumulator: &dyn OpeningAccumulator<F>,
        transcript: &mut impl Transcript,
    ) -> Self {
        Self {
            params: IncVirtualizationParams::new(trace_len, accumulator, transcript),
        }
    }
}

impl<F: JoltField, T: Transcript, A: AbstractVerifierOpeningAccumulator<F>>
    SumcheckInstanceVerifier<F, T, A> for IncVirtualizationVerifier<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    fn expected_output_claim(&self, accumulator: &A, sumcheck_challenges: &[F::Challenge]) -> F {
        let opening_point = self.params.normalize_opening_point(sumcheck_challenges);
        let (ram_coeff, rd_coeff) = self.params.opening_coeffs_at(&opening_point);
        let (_, inc_claim) = accumulator
            .get_virtual_polynomial_opening(VirtualPolynomial::Inc, SumcheckId::IncVirtualization);
        let (_, store_claim) = accumulator.get_virtual_polynomial_opening(
            VirtualPolynomial::OpFlags(CircuitFlags::Store),
            SumcheckId::IncVirtualization,
        );

        inc_claim * (ram_coeff * store_claim + rd_coeff * (F::one() - store_claim))
    }

    fn cache_openings(&self, accumulator: &mut A, sumcheck_challenges: &[F::Challenge]) {
        let opening_point = self.params.normalize_opening_point(sumcheck_challenges);
        accumulator.append_virtual(
            VirtualPolynomial::Inc,
            SumcheckId::IncVirtualization,
            opening_point.clone(),
        );
        accumulator.append_virtual(
            VirtualPolynomial::OpFlags(CircuitFlags::Store),
            SumcheckId::IncVirtualization,
            opening_point,
        );
    }
}
