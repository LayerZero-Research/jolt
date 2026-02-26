use crate::field::JoltField;
use crate::poly::commitment::dory::{DoryGlobals, DoryLayout};
use crate::poly::eq_poly::EqPolynomial;
use crate::poly::multilinear_polynomial::{BindingOrder, MultilinearPolynomial, PolynomialBinding};
use crate::poly::opening_proof::{
    OpeningAccumulator, OpeningPoint, PolynomialId, ProverOpeningAccumulator, SumcheckId,
    VerifierOpeningAccumulator, BIG_ENDIAN, LITTLE_ENDIAN,
};
use crate::poly::split_eq_poly::GruenSplitEqPolynomial;
use crate::poly::unipoly::UniPoly;
use crate::subprotocols::sumcheck_claim::{
    CachedPointRef, ChallengePart, Claim, ClaimExpr, InputOutputClaims, SumcheckFrontend,
    VerifierEvaluablePolynomial,
};
use crate::subprotocols::sumcheck_prover::SumcheckInstanceProver;
use crate::subprotocols::sumcheck_verifier::{SumcheckInstanceParams, SumcheckInstanceVerifier};
use crate::transcripts::Transcript;
use crate::utils::math::Math;
use crate::zkvm::config::OneHotParams;
use crate::zkvm::witness::VirtualPolynomial;
use allocative::Allocative;
#[cfg(feature = "allocative")]
use allocative::FlameGraphBuilder;
use rayon::prelude::*;
use std::ops::Range;
use tracer::instruction::Cycle;

// RAM Hamming booleanity sumcheck
//
// Proves a zero-check of the form
//   0 = Σ_j eq(r_cycle, j) · (H(j)^2 − H(j))
// where:
// - r_cycle are the time/cycle variables bound in this sumcheck
// - H(j) is an indicator of whether a RAM access occurred at cycle j (1 if address != 0, 0 otherwise)

/// Degree bound of the sumcheck round polynomials in [`HammingBooleanitySumcheckVerifier`].
const DEGREE_BOUND: usize = 3;

#[derive(Allocative, Clone)]
pub struct HammingBooleanitySumcheckParams<F: JoltField> {
    pub r_cycle: OpeningPoint<BIG_ENDIAN, F>,
    pub target_log_t: usize,
    #[allocative(skip)]
    pub cycle_phase_col_rounds: Range<usize>,
    #[allocative(skip)]
    pub cycle_phase_row_rounds: Range<usize>,
}

impl<F: JoltField> HammingBooleanitySumcheckParams<F> {
    pub fn new(
        trace_len: usize,
        target_log_t: usize,
        dory_layout: DoryLayout,
        one_hot_params: &OneHotParams,
        opening_accumulator: &dyn OpeningAccumulator<F>,
    ) -> Self {
        let (r_cycle, _) = opening_accumulator.get_virtual_polynomial_opening(
            VirtualPolynomial::LookupOutput,
            SumcheckId::SpartanOuter,
        );

        let log_t = trace_len.log_2();
        let target_log_t = target_log_t.max(log_t);
        let (sigma_main, _) = DoryGlobals::main_sigma_nu(one_hot_params.log_k_chunk, target_log_t);
        let main_col_vars = sigma_main;
        let (poly_col_vars, poly_row_vars) = DoryGlobals::balanced_sigma_nu(log_t);
        let (cycle_phase_col_rounds, cycle_phase_row_rounds) = match dory_layout {
            DoryLayout::CycleMajor => {
                let col_end = std::cmp::min(target_log_t, poly_col_vars);
                let col_binding_rounds = 0..col_end;
                let row_start_unclamped = main_col_vars.saturating_sub(one_hot_params.log_k_chunk);
                let row_start = std::cmp::min(
                    target_log_t,
                    std::cmp::max(row_start_unclamped, col_end),
                );
                let row_end = std::cmp::min(target_log_t, row_start + poly_row_vars);
                (col_binding_rounds, row_start..row_end)
            }
            DoryLayout::AddressMajor => {
                let col_end = std::cmp::min(target_log_t, poly_col_vars);
                let col_binding_rounds = 0..col_end;
                let row_start_unclamped = main_col_vars.saturating_sub(one_hot_params.log_k_chunk);
                let row_start = std::cmp::min(
                    target_log_t,
                    std::cmp::max(row_start_unclamped, col_end),
                );
                let row_end = std::cmp::min(target_log_t, row_start + poly_row_vars);
                (col_binding_rounds, row_start..row_end)
            }
        };
        assert_eq!(
            cycle_phase_col_rounds.len() + cycle_phase_row_rounds.len(),
            log_t,
            "hamming booleanity cycle schedule must bind exactly log_t rounds (layout={dory_layout:?}, log_t={log_t}, target_log_t={target_log_t}, col_rounds={:?}, row_rounds={:?})",
            cycle_phase_col_rounds,
            cycle_phase_row_rounds
        );

        Self {
            r_cycle,
            target_log_t,
            cycle_phase_col_rounds,
            cycle_phase_row_rounds,
        }
    }

    #[inline]
    fn active_cycle_rounds(&self) -> usize {
        self.cycle_phase_col_rounds.len() + self.cycle_phase_row_rounds.len()
    }

    #[inline]
    fn dummy_rounds(&self) -> usize {
        self.target_log_t.saturating_sub(self.active_cycle_rounds())
    }

    #[inline]
    fn dummy_scale(&self) -> F {
        let two_inv = F::from_u64(2).inverse().unwrap();
        (0..self.dummy_rounds()).fold(F::one(), |acc, _| acc * two_inv)
    }

    #[inline]
    fn is_cycle_dummy_round(&self, round: usize) -> bool {
        !self.cycle_phase_col_rounds.contains(&round) && !self.cycle_phase_row_rounds.contains(&round)
    }

    #[inline]
    fn compact_cycle_challenges(&self, sumcheck_challenges: &[F::Challenge]) -> Vec<F::Challenge> {
        let mut compact =
            Vec::with_capacity(self.cycle_phase_col_rounds.len() + self.cycle_phase_row_rounds.len());
        compact.extend_from_slice(&sumcheck_challenges[self.cycle_phase_col_rounds.clone()]);
        if !self.cycle_phase_row_rounds.is_empty() {
            compact.extend_from_slice(&sumcheck_challenges[self.cycle_phase_row_rounds.clone()]);
        }
        compact
    }
}

impl<F: JoltField> SumcheckInstanceParams<F> for HammingBooleanitySumcheckParams<F> {
    fn degree(&self) -> usize {
        DEGREE_BOUND
    }

    fn num_rounds(&self) -> usize {
        self.target_log_t
    }

    fn input_claim(&self, _: &dyn OpeningAccumulator<F>) -> F {
        F::zero()
    }

    fn normalize_opening_point(
        &self,
        challenges: &[<F as JoltField>::Challenge],
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        OpeningPoint::<LITTLE_ENDIAN, F>::new(challenges.to_vec()).match_endianness()
    }
}

#[derive(Allocative)]
pub struct HammingBooleanitySumcheckProver<F: JoltField> {
    eq_r_cycle: GruenSplitEqPolynomial<F>,
    H: MultilinearPolynomial<F>,
    pub params: HammingBooleanitySumcheckParams<F>,
    scale: F,
}

impl<F: JoltField> HammingBooleanitySumcheckProver<F> {
    #[tracing::instrument(skip_all, name = "RamHammingBooleanitySumcheckProver::initialize")]
    pub fn initialize(params: HammingBooleanitySumcheckParams<F>, trace: &[Cycle]) -> Self {
        let H = trace
            .par_iter()
            .map(|cycle| cycle.ram_access().address() != 0)
            .collect::<Vec<bool>>();
        let H = MultilinearPolynomial::from(H);

        let eq_r_cycle = GruenSplitEqPolynomial::new(&params.r_cycle.r, BindingOrder::LowToHigh);

        Self {
            eq_r_cycle,
            H,
            params,
            scale: F::one(),
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T>
    for HammingBooleanitySumcheckProver<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    #[tracing::instrument(skip_all, name = "RamHammingBooleanitySumcheckProver::compute_message")]
    fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
        if self.params.is_cycle_dummy_round(round) {
            let two_inv = F::from_u64(2).inverse().unwrap();
            return UniPoly::from_coeff(vec![previous_claim * two_inv, F::zero()]);
        }
        let inv_scale = self.scale.inverse().unwrap();
        let previous_claim = previous_claim * inv_scale;
        let eq = &self.eq_r_cycle;
        let H = &self.H;

        // Accumulate constant (c0) and quadratic (e) coefficients via generic split-eq fold.
        let [c0, e] = eq.par_fold_out_in_unreduced::<9, 2>(&|g| {
            let h0 = H.get_bound_coeff(2 * g);
            let h1 = H.get_bound_coeff(2 * g + 1);
            let delta = h1 - h0;
            [h0.square() - h0, delta.square()]
        });
        eq.gruen_poly_deg_3(c0, e, previous_claim) * self.scale
    }

    #[tracing::instrument(
        skip_all,
        name = "RamHammingBooleanitySumcheckProver::ingest_challenge"
    )]
    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        if self.params.is_cycle_dummy_round(round) {
            let two_inv = F::from_u64(2).inverse().unwrap();
            self.scale *= two_inv;
            return;
        }
        self.eq_r_cycle.bind(r_j);
        self.H.bind_parallel(r_j, BindingOrder::LowToHigh);
    }

    fn cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let active_cycle_challenges = self.params.compact_cycle_challenges(sumcheck_challenges);
        accumulator.append_virtual(
            transcript,
            VirtualPolynomial::RamHammingWeight,
            SumcheckId::RamHammingBooleanity,
            self.params.normalize_opening_point(&active_cycle_challenges),
            self.H.final_sumcheck_claim(),
        );
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut FlameGraphBuilder) {
        flamegraph.visit_root(self);
    }
}

pub struct HammingBooleanitySumcheckVerifier<F: JoltField> {
    params: HammingBooleanitySumcheckParams<F>,
}

impl<F: JoltField> HammingBooleanitySumcheckVerifier<F> {
    pub fn new(
        trace_len: usize,
        target_log_t: usize,
        dory_layout: DoryLayout,
        one_hot_params: &OneHotParams,
        opening_accumulator: &dyn OpeningAccumulator<F>,
    ) -> Self {
        Self {
            params: HammingBooleanitySumcheckParams::new(
                trace_len,
                target_log_t,
                dory_layout,
                one_hot_params,
                opening_accumulator,
            ),
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceVerifier<F, T>
    for HammingBooleanitySumcheckVerifier<F>
{
    fn input_claim(&self, accumulator: &VerifierOpeningAccumulator<F>) -> F {
        let result = self.params.input_claim(accumulator);

        #[cfg(test)]
        {
            let reference_result =
                Self::input_output_claims().input_claim(&[F::one()], accumulator);
            assert_eq!(result, reference_result);
        }

        result
    }

    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    fn expected_output_claim(
        &self,
        accumulator: &VerifierOpeningAccumulator<F>,
        sumcheck_challenges: &[F::Challenge],
    ) -> F {
        let H_claim = accumulator
            .get_virtual_polynomial_opening(
                VirtualPolynomial::RamHammingWeight,
                SumcheckId::RamHammingBooleanity,
            )
            .1;

        let (r_cycle, _) = accumulator.get_virtual_polynomial_opening(
            VirtualPolynomial::LookupOutput,
            SumcheckId::SpartanOuter,
        );

        let active_cycle_challenges = self.params.compact_cycle_challenges(sumcheck_challenges);
        let r_cycle_final = self.params.normalize_opening_point(&active_cycle_challenges);
        let eq = EqPolynomial::<F>::mle_endian(&r_cycle, &r_cycle_final);

        let result = self.params.dummy_scale() * (H_claim.square() - H_claim) * eq;

        #[cfg(test)]
        {
            let r = self.params.normalize_opening_point(&active_cycle_challenges);
            let reference_result =
                Self::input_output_claims().expected_output_claim(&r, &[F::one()], accumulator);
            assert_eq!(result, self.params.dummy_scale() * reference_result);
        }

        result
    }

    fn cache_openings(
        &self,
        accumulator: &mut VerifierOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let active_cycle_challenges = self.params.compact_cycle_challenges(sumcheck_challenges);
        accumulator.append_virtual(
            transcript,
            VirtualPolynomial::RamHammingWeight,
            SumcheckId::RamHammingBooleanity,
            self.params.normalize_opening_point(&active_cycle_challenges),
        );
    }
}

impl<F: JoltField> SumcheckFrontend<F> for HammingBooleanitySumcheckVerifier<F> {
    fn input_output_claims() -> InputOutputClaims<F> {
        let ram_hamming_weight: ClaimExpr<F> = VirtualPolynomial::RamHammingWeight.into();
        let ram_hamming_weight_squared = ram_hamming_weight.clone() * ram_hamming_weight.clone();

        let eq_r_stage1 = VerifierEvaluablePolynomial::Eq(CachedPointRef {
            opening: PolynomialId::Virtual(VirtualPolynomial::LookupOutput),
            sumcheck: SumcheckId::SpartanOuter,
            part: ChallengePart::Cycle,
        });

        InputOutputClaims {
            claims: vec![Claim {
                // NOTE: In this case, the input claim is 0, so this is just the sumcheck to
                // take r_cycle from.
                input_sumcheck_id: SumcheckId::SpartanOuter,
                input_claim_expr: F::zero().into(),
                batching_poly: eq_r_stage1,
                expected_output_claim_expr: ram_hamming_weight_squared - ram_hamming_weight,
            }],
            output_sumcheck_id: SumcheckId::RamHammingBooleanity,
        }
    }
}
