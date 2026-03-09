//! Two-phase advice claim reduction (Stage 6 cycle → Stage 7 address)
//!
//! This module generalizes the previous single-phase `AdviceClaimReduction` so that trusted and
//! untrusted advice can be committed as an arbitrary Dory matrix `2^{nu_a} x 2^{sigma_a}` (balanced
//! by default), while still keeping a **single Stage 8 Dory opening** at the unified Dory point.
//!
//! For an advice matrix embedded as the **top-left block** `2^{nu_a} x 2^{sigma_a}`, the *native*
//! advice evaluation point (in Dory order, LSB-first) is:
//! - `advice_cols = col_coords[0..sigma_a]`
//! - `advice_rows = row_coords[0..nu_a]`
//! - `advice_point = [advice_cols || advice_rows]`
//!
//! In our current pipeline, `cycle` coordinates come from Stage 6 and `addr` coordinates come from
//! Stage 7.
//! - **Phase 1 (Stage 6)**: bind the cycle-derived advice coordinates and output an intermediate
//!   scalar claim `C_mid`.
//! - **Phase 2 (Stage 7)**: resume from `C_mid`, bind the address-derived advice coordinates, and
//!   cache the final advice opening `AdviceMLE(advice_point)` for batching into Stage 8.
//!
//! ## Dummy-gap scaling (within Stage 6)
//! With cycle-major order, there may be a gap during the cycle phase where the cycle variables
//! being bound in the batched sumcheck do not appear in the advice polynommial.
//!
//! We handle this without modifying the generic batched sumcheck by treating those intervening
//! rounds as **dummy internal rounds** (constant univariates), and maintaining a running scaling
//! factor `2^{-dummy_done}` so the per-round univariates remain consistent.
//!
//! Trusted and untrusted advice run as **separate** sumcheck instances (each may have different
//! dimensions).
//!

use std::cell::RefCell;
use std::cmp::Ordering;

use crate::field::JoltField;
use crate::poly::commitment::dory::{DoryGlobals, DoryLayout};
use crate::poly::eq_poly::EqPolynomial;
use crate::poly::multilinear_polynomial::{BindingOrder, MultilinearPolynomial, PolynomialBinding};
use crate::poly::opening_proof::{
    OpeningAccumulator, OpeningPoint, ProverOpeningAccumulator, SumcheckId,
    VerifierOpeningAccumulator, BIG_ENDIAN, LITTLE_ENDIAN,
};
use crate::poly::unipoly::UniPoly;
use crate::subprotocols::sumcheck_prover::SumcheckInstanceProver;
use crate::subprotocols::sumcheck_verifier::{SumcheckInstanceParams, SumcheckInstanceVerifier};
use crate::transcripts::Transcript;
use crate::utils::math::Math;
use crate::zkvm::claim_reductions::{PrecommittedClaimReduction, TwoPhaseRoundSchedule};
use crate::zkvm::config::OneHotConfig;
use allocative::Allocative;
use common::jolt_device::MemoryLayout;
use rayon::prelude::*;

const DEGREE_BOUND: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Allocative)]
pub enum PreCommitted {
    CycleVariables,
    AddressVariables,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Allocative)]
pub enum AdviceKind {
    Trusted,
    Untrusted,
}

#[derive(Clone, Allocative)]
pub struct AdviceClaimReductionParams<F: JoltField> {
    pub kind: AdviceKind,
    pub phase: PreCommitted,
    pub cycle_var_challenges: Vec<F::Challenge>,
    pub gamma: F,
    pub single_opening: bool,
    pub log_k_chunk: usize,
    pub log_t: usize,
    pub cycle_alignment_rounds: usize,
    pub advice_col_vars: usize,
    pub advice_row_vars: usize,
    /// Number of column variables in the Stage-8 joint Dory matrix.
    pub joint_col_vars: usize,
    #[allocative(skip)]
    pub round_schedule: TwoPhaseRoundSchedule,
    pub r_val_eval: OpeningPoint<BIG_ENDIAN, F>,
    pub r_val_final: Option<OpeningPoint<BIG_ENDIAN, F>>,
}

impl<F: JoltField> AdviceClaimReductionParams<F> {
    pub fn new(
        kind: AdviceKind,
        memory_layout: &MemoryLayout,
        trace_len: usize,
        cycle_alignment_rounds: usize,
        accumulator: &dyn OpeningAccumulator<F>,
        transcript: &mut impl Transcript,
        single_opening: bool,
    ) -> Self {
        let max_advice_size_bytes = match kind {
            AdviceKind::Trusted => memory_layout.max_trusted_advice_size as usize,
            AdviceKind::Untrusted => memory_layout.max_untrusted_advice_size as usize,
        };

        let log_t = trace_len.log_2();
        let cycle_alignment_rounds = cycle_alignment_rounds.max(log_t);
        let log_k_chunk = OneHotConfig::new(log_t).log_k_chunk as usize;
        let joint_geometry = PrecommittedClaimReduction::precommitted_geometry();
        debug_assert_eq!(joint_geometry.main_cycle_rounds, log_t);
        debug_assert_eq!(joint_geometry.cycle_rounds, cycle_alignment_rounds);
        debug_assert_eq!(joint_geometry.address_rounds, log_k_chunk);
        let joint_col_vars = joint_geometry.joint_col_vars;

        let r_val_eval = accumulator
            .get_advice_opening(kind, SumcheckId::RamValEvaluation)
            .map(|(p, _)| p)
            .unwrap();
        let r_val_final = if single_opening {
            None
        } else {
            accumulator
                .get_advice_opening(kind, SumcheckId::RamValFinalEvaluation)
                .map(|(p, _)| p)
        };

        let gamma: F = transcript.challenge_scalar();

        let (advice_col_vars, advice_row_vars) =
            DoryGlobals::advice_sigma_nu_from_max_bytes(max_advice_size_bytes);
        let round_schedule = PrecommittedClaimReduction::precommitted_two_phase_round_schedule(
            advice_row_vars,
            advice_col_vars,
        );

        Self {
            kind,
            phase: PreCommitted::CycleVariables,
            cycle_var_challenges: vec![],
            gamma,
            advice_col_vars,
            advice_row_vars,
            single_opening,
            log_k_chunk,
            log_t,
            cycle_alignment_rounds,
            joint_col_vars,
            round_schedule,
            r_val_eval,
            r_val_final,
        }
    }

    /// (Total # advice variables) - (# variables bound during cycle phase)
    pub fn num_address_phase_rounds(&self) -> usize {
        self.round_schedule.address_phase_rounds.len()
    }
}

impl<F: JoltField> AdviceClaimReductionParams<F> {
    fn is_cycle_phase(&self) -> bool {
        self.phase == PreCommitted::CycleVariables
    }

    fn cycle_alignment_rounds(&self) -> usize {
        self.cycle_alignment_rounds
    }

    fn address_alignment_rounds(&self) -> usize {
        self.log_k_chunk
    }

    pub fn transition_to_address_phase(&mut self) {
        self.phase = PreCommitted::AddressVariables;
    }

    #[inline]
    fn is_cycle_phase_round(&self, round: usize) -> bool {
        self.round_schedule
            .cycle_phase_rounds
            .binary_search(&round)
            .is_ok()
    }

    pub fn round_offset(&self, max_num_rounds: usize) -> usize {
        PrecommittedClaimReduction::precommitted_round_offset(
            self.is_cycle_phase(),
            max_num_rounds,
            self.cycle_alignment_rounds(),
            self.num_rounds(),
        )
    }
}

impl<F: JoltField> SumcheckInstanceParams<F> for AdviceClaimReductionParams<F> {
    fn input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        match self.phase {
            PreCommitted::CycleVariables => {
                let mut claim = F::zero();
                if let Some((_, eval)) =
                    accumulator.get_advice_opening(self.kind, SumcheckId::RamValEvaluation)
                {
                    claim += eval;
                }
                if !self.single_opening {
                    if let Some((_, final_eval)) =
                        accumulator.get_advice_opening(self.kind, SumcheckId::RamValFinalEvaluation)
                    {
                        claim += self.gamma * final_eval;
                    }
                }
                claim
            }
            PreCommitted::AddressVariables => {
                // Address phase starts from the cycle phase intermediate claim.
                accumulator
                    .get_advice_opening(self.kind, SumcheckId::AdviceClaimReductionCyclePhase)
                    .expect("Cycle phase intermediate claim not found")
                    .1
            }
        }
    }

    fn degree(&self) -> usize {
        DEGREE_BOUND
    }

    fn num_rounds(&self) -> usize {
        PrecommittedClaimReduction::precommitted_num_rounds_for_phase(
            self.is_cycle_phase(),
            &self.round_schedule,
        )
    }

    fn normalize_opening_point(&self, challenges: &[F::Challenge]) -> OpeningPoint<BIG_ENDIAN, F> {
        PrecommittedClaimReduction::normalize_precommitted_two_phase_opening_point(
            self.is_cycle_phase(),
            &self.cycle_var_challenges,
            &self.round_schedule,
            challenges,
        )
    }
}

#[derive(Allocative)]
pub struct AdviceClaimReductionProver<F: JoltField> {
    params: AdviceClaimReductionParams<F>,
    advice_poly: MultilinearPolynomial<F>,
    eq_poly: MultilinearPolynomial<F>,
    scale: F,
}

impl<F: JoltField> AdviceClaimReductionProver<F> {
    pub fn params(&self) -> &AdviceClaimReductionParams<F> {
        &self.params
    }

    pub fn transition_to_address_phase(&mut self) {
        self.params.transition_to_address_phase();
    }

    pub fn initialize(
        params: AdviceClaimReductionParams<F>,
        advice_poly: MultilinearPolynomial<F>,
    ) -> Self {
        let eq_evals = if params.single_opening {
            EqPolynomial::evals(&params.r_val_eval.r)
        } else {
            let evals = EqPolynomial::evals(&params.r_val_eval.r);
            let r_final = params
                .r_val_final
                .as_ref()
                .expect("r_val_final must exist when !single_opening");
            let eq_final = EqPolynomial::evals_with_scaling(&r_final.r, Some(params.gamma));
            evals
                .par_iter()
                .zip(eq_final.par_iter())
                .map(|(e1, e2)| *e1 + e2)
                .collect()
        };
        let (advice_poly, eq_poly): (MultilinearPolynomial<F>, MultilinearPolynomial<F>) =
            match DoryGlobals::get_layout() {
                DoryLayout::AddressMajor => {
                    let MultilinearPolynomial::U64Scalars(poly) = advice_poly else {
                        panic!("Advice should have u64 coefficients");
                    };
                    let mut permuted_advice = vec![0u64; poly.coeffs.len()];
                    let mut permuted_eq = vec![F::zero(); eq_evals.len()];
                    for old_idx in 0..poly.coeffs.len() {
                        let new_idx =
                            PrecommittedClaimReduction::permute_precommitted_sumcheck_index_address_major(
                                old_idx,
                                params.advice_row_vars,
                                params.advice_col_vars,
                            );
                        permuted_advice[new_idx] = poly.coeffs[old_idx];
                        permuted_eq[new_idx] = eq_evals[old_idx];
                    }
                    (permuted_advice.into(), permuted_eq.into())
                }
                DoryLayout::CycleMajor => {
                    let mut permuted_coeffs: Vec<(usize, (u64, F))> = match advice_poly {
                        MultilinearPolynomial::U64Scalars(poly) => poly
                            .coeffs
                            .into_par_iter()
                            .zip(eq_evals.into_par_iter())
                            .enumerate()
                            .collect(),
                        _ => panic!("Advice should have u64 coefficients"),
                    };
                    // Sort the advice and EQ polynomial coefficients by (address, cycle).
                    // By sorting this way, binding the resulting polynomials in low-to-high
                    // order is equivalent to binding the original polynomials' "cycle" variables
                    // low-to-high, then their "address" variables low-to-high.
                    permuted_coeffs.par_sort_by(|&(index_a, _), &(index_b, _)| {
                        let (address_a, cycle_a) =
                            PrecommittedClaimReduction::cycle_major_top_left_index_to_address_cycle(
                                index_a,
                                params.advice_col_vars,
                            );
                        let (address_b, cycle_b) =
                            PrecommittedClaimReduction::cycle_major_top_left_index_to_address_cycle(
                                index_b,
                                params.advice_col_vars,
                            );
                        match address_a.cmp(&address_b) {
                            Ordering::Less => Ordering::Less,
                            Ordering::Greater => Ordering::Greater,
                            Ordering::Equal => cycle_a.cmp(&cycle_b),
                        }
                    });

                    let (advice_coeffs, eq_coeffs): (Vec<_>, Vec<_>) = permuted_coeffs
                        .into_par_iter()
                        .map(|(_, coeffs)| coeffs)
                        .unzip();
                    (advice_coeffs.into(), eq_coeffs.into())
                }
            };

        Self {
            params,
            advice_poly,
            eq_poly,
            scale: F::one(),
        }
    }

    fn compute_message_unscaled(&self, previous_claim_unscaled: F) -> UniPoly<F> {
        let half = self.advice_poly.len() / 2;
        let evals: [F; DEGREE_BOUND] = (0..half)
            .into_par_iter()
            .map(|j| {
                let a_evals = self
                    .advice_poly
                    .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                let eq_evals = self
                    .eq_poly
                    .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);

                let mut out = [F::zero(); DEGREE_BOUND];
                for i in 0..DEGREE_BOUND {
                    out[i] = a_evals[i] * eq_evals[i];
                }
                out
            })
            .reduce(
                || [F::zero(); DEGREE_BOUND],
                |mut acc, arr| {
                    acc.iter_mut().zip(arr.iter()).for_each(|(a, b)| *a += *b);
                    acc
                },
            );
        UniPoly::from_evals_and_hint(previous_claim_unscaled, &evals)
    }

    fn cycle_intermediate_claim(&self) -> F {
        let len = self.advice_poly.len();
        debug_assert_eq!(len, self.eq_poly.len());

        let mut sum = F::zero();
        for i in 0..len {
            sum += self.advice_poly.get_bound_coeff(i) * self.eq_poly.get_bound_coeff(i);
        }
        sum * self.scale
    }

    fn final_claim_if_ready(&self) -> Option<F> {
        if self.advice_poly.len() == 1 {
            Some(self.advice_poly.final_sumcheck_claim())
        } else {
            None
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T> for AdviceClaimReductionProver<F> {
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        self.params.round_offset(max_num_rounds)
    }

    fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
        if self.params.is_cycle_phase() && !self.params.is_cycle_phase_round(round) {
            return UniPoly::from_coeff(vec![previous_claim * F::from_u64(2).inverse().unwrap()]);
        }

        let trailing_cap = if self.params.is_cycle_phase() {
            self.params.cycle_alignment_rounds()
        } else {
            self.params.address_alignment_rounds()
        };
        let num_trailing_variables = trailing_cap.saturating_sub(self.params.num_rounds());
        let scaling_factor = self.scale * F::one().mul_pow_2(num_trailing_variables);
        let prev_unscaled = previous_claim * scaling_factor.inverse().unwrap();
        let poly_unscaled = self.compute_message_unscaled(prev_unscaled);
        poly_unscaled * scaling_factor
    }

    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        if self.params.is_cycle_phase() {
            let is_dummy_round = !self.params.is_cycle_phase_round(round);
            if is_dummy_round {
                self.scale *= F::from_u64(2).inverse().unwrap();
            } else {
                self.advice_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
                self.eq_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
                self.params.cycle_var_challenges.push(r_j);
            }
            return;
        }

        self.advice_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
        self.eq_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
    }

    fn cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let params = &self.params;
        let opening_point = params.normalize_opening_point(sumcheck_challenges);
        if params.phase == PreCommitted::CycleVariables {
            let c_mid = self.cycle_intermediate_claim();

            match params.kind {
                AdviceKind::Trusted => accumulator.append_trusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReductionCyclePhase,
                    opening_point.clone(),
                    c_mid,
                ),
                AdviceKind::Untrusted => accumulator.append_untrusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReductionCyclePhase,
                    opening_point.clone(),
                    c_mid,
                ),
            }
        }

        if let Some(advice_claim) = self.final_claim_if_ready() {
            match params.kind {
                AdviceKind::Trusted => accumulator.append_trusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReduction,
                    opening_point,
                    advice_claim,
                ),
                AdviceKind::Untrusted => accumulator.append_untrusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReduction,
                    opening_point,
                    advice_claim,
                ),
            }
        }
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
        flamegraph.visit_root(self);
    }
}

pub struct AdviceClaimReductionVerifier<F: JoltField> {
    pub params: RefCell<AdviceClaimReductionParams<F>>,
}

impl<F: JoltField> AdviceClaimReductionVerifier<F> {
    pub fn new(
        kind: AdviceKind,
        memory_layout: &MemoryLayout,
        trace_len: usize,
        cycle_alignment_rounds: usize,
        accumulator: &VerifierOpeningAccumulator<F>,
        transcript: &mut impl Transcript,
        single_opening: bool,
    ) -> Self {
        let params = AdviceClaimReductionParams::new(
            kind,
            memory_layout,
            trace_len,
            cycle_alignment_rounds,
            accumulator,
            transcript,
            single_opening,
        );

        Self {
            params: RefCell::new(params),
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceVerifier<F, T>
    for AdviceClaimReductionVerifier<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        unsafe { &*self.params.as_ptr() }
    }

    fn expected_output_claim(
        &self,
        accumulator: &VerifierOpeningAccumulator<F>,
        sumcheck_challenges: &[F::Challenge],
    ) -> F {
        let params = self.params.borrow();
        match params.phase {
            PreCommitted::CycleVariables => {
                accumulator
                    .get_advice_opening(params.kind, SumcheckId::AdviceClaimReductionCyclePhase)
                    .unwrap_or_else(|| panic!("Cycle phase intermediate claim not found",))
                    .1
            }
            PreCommitted::AddressVariables => {
                let opening_point = params.normalize_opening_point(sumcheck_challenges);
                let advice_claim = accumulator
                    .get_advice_opening(params.kind, SumcheckId::AdviceClaimReduction)
                    .expect("Final advice claim not found")
                    .1;

                let eq_eval = EqPolynomial::mle(&opening_point.r, &params.r_val_eval.r);
                let eq_combined = if params.single_opening {
                    eq_eval
                } else {
                    let r_final = params
                        .r_val_final
                        .as_ref()
                        .expect("r_val_final must exist when !single_opening");
                    let eq_final = EqPolynomial::mle(&opening_point.r, &r_final.r);
                    eq_eval + params.gamma * eq_final
                };

                let gap_len =
                    PrecommittedClaimReduction::precommitted_internal_dummy_gap_len(
                        &params.round_schedule,
                    );
                let two_inv = F::from_u64(2).inverse().unwrap();
                let scale = (0..gap_len).fold(F::one(), |acc, _| acc * two_inv);

                // Account for Phase 1's internal dummy-gap traversal via constant scaling.
                advice_claim * eq_combined * scale
            }
        }
    }

    fn cache_openings(
        &self,
        accumulator: &mut VerifierOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let mut params = self.params.borrow_mut();
        if params.phase == PreCommitted::CycleVariables {
            let opening_point = params.normalize_opening_point(sumcheck_challenges);
            match params.kind {
                AdviceKind::Trusted => accumulator.append_trusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReductionCyclePhase,
                    opening_point.clone(),
                ),
                AdviceKind::Untrusted => accumulator.append_untrusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReductionCyclePhase,
                    opening_point.clone(),
                ),
            }
            let opening_point_le: OpeningPoint<LITTLE_ENDIAN, F> = opening_point.match_endianness();
            params.cycle_var_challenges = opening_point_le.r;
        }

        if params.num_address_phase_rounds() == 0 || params.phase == PreCommitted::AddressVariables
        {
            let opening_point = params.normalize_opening_point(sumcheck_challenges);
            match params.kind {
                AdviceKind::Trusted => accumulator.append_trusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReduction,
                    opening_point,
                ),
                AdviceKind::Untrusted => accumulator.append_untrusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReduction,
                    opening_point,
                ),
            }
        }
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        let params = self.params.borrow();
        params.round_offset(max_num_rounds)
    }
}
