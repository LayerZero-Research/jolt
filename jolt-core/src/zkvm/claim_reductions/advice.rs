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
use std::ops::Range;

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

#[inline]
fn internal_dummy_gap_len(
    cycle_phase_col_rounds: &Range<usize>,
    cycle_phase_row_rounds: &Range<usize>,
) -> usize {
    let cycle_phase_total_rounds = if !cycle_phase_row_rounds.is_empty() {
        cycle_phase_row_rounds.end - cycle_phase_col_rounds.start
    } else {
        cycle_phase_col_rounds.len()
    };
    cycle_phase_total_rounds - (cycle_phase_col_rounds.len() + cycle_phase_row_rounds.len())
}

fn cycle_phase_round_schedule(
    log_t: usize,
    log_k_chunk: usize,
    joint_col_vars: usize,
    poly_row_vars: usize,
    poly_col_vars: usize,
) -> (Range<usize>, Range<usize>) {
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => {
            let col_end = std::cmp::min(log_t, poly_col_vars);
            let col_binding_rounds = 0..col_end;
            let row_start = std::cmp::min(
                log_t,
                std::cmp::max(std::cmp::min(log_t, joint_col_vars), col_end),
            );
            let row_end = std::cmp::min(log_t, row_start + poly_row_vars);
            let row_binding_rounds = row_start..row_end;
            (col_binding_rounds, row_binding_rounds)
        }
        DoryLayout::AddressMajor => {
            let col_end = std::cmp::min(log_t, poly_col_vars.saturating_sub(log_k_chunk));
            let col_binding_rounds = 0..col_end;
            let row_start_unclamped = joint_col_vars.saturating_sub(log_k_chunk);
            let row_start = std::cmp::min(log_t, std::cmp::max(row_start_unclamped, col_end));
            let row_end = std::cmp::min(log_t, row_start + poly_row_vars);
            let row_binding_rounds = row_start..row_end;
            (col_binding_rounds, row_binding_rounds)
        }
    }
}

#[inline]
fn joint_col_vars_for_precommitted(cycle_rounds: usize, address_rounds: usize) -> usize {
    DoryGlobals::balanced_sigma_nu(cycle_rounds + address_rounds).0
}

fn precommitted_num_rounds<Phase: Copy + Eq>(
    phase: Phase,
    cycle_phase: Phase,
    cycle_phase_col_rounds: &Range<usize>,
    cycle_phase_row_rounds: &Range<usize>,
    total_poly_vars: usize,
) -> usize {
    if phase == cycle_phase {
        if !cycle_phase_row_rounds.is_empty() {
            cycle_phase_row_rounds.end - cycle_phase_col_rounds.start
        } else {
            cycle_phase_col_rounds.len()
        }
    } else {
        let first_phase_rounds = cycle_phase_col_rounds.len() + cycle_phase_row_rounds.len();
        total_poly_vars - first_phase_rounds
    }
}

fn normalize_two_phase_opening_point<F: JoltField, Phase: Copy + Eq>(
    phase: Phase,
    cycle_var_challenges: &[F::Challenge],
    cycle_phase: Phase,
    cycle_phase_col_rounds: &Range<usize>,
    cycle_phase_row_rounds: &Range<usize>,
    challenges: &[F::Challenge],
) -> OpeningPoint<BIG_ENDIAN, F> {
    if phase == cycle_phase {
        let compact_offset = cycle_phase_col_rounds.start;
        let compact_col_rounds = 0..cycle_phase_col_rounds.len();
        let compact_row_rounds = cycle_phase_row_rounds.start.saturating_sub(compact_offset)
            ..cycle_phase_row_rounds.end.saturating_sub(compact_offset);
        let mut cycle_var_challenges: Vec<F::Challenge> =
            Vec::with_capacity(cycle_phase_col_rounds.len() + cycle_phase_row_rounds.len());
        cycle_var_challenges.extend_from_slice(&challenges[compact_col_rounds]);
        if !cycle_phase_row_rounds.is_empty() {
            cycle_var_challenges.extend_from_slice(&challenges[compact_row_rounds]);
        }
        return OpeningPoint::<LITTLE_ENDIAN, F>::new(cycle_var_challenges).match_endianness();
    }

    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => OpeningPoint::<LITTLE_ENDIAN, F>::new(
            [cycle_var_challenges, challenges].concat(),
        )
        .match_endianness(),
        DoryLayout::AddressMajor => OpeningPoint::<LITTLE_ENDIAN, F>::new(
            [challenges, cycle_var_challenges].concat(),
        )
        .match_endianness(),
    }
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
    pub cycle_phase_row_rounds: Range<usize>,
    #[allocative(skip)]
    pub cycle_phase_col_rounds: Range<usize>,
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
        let joint_col_vars = joint_col_vars_for_precommitted(cycle_alignment_rounds, log_k_chunk);

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
        let (col_binding_rounds, row_binding_rounds) = cycle_phase_round_schedule(
            cycle_alignment_rounds,
            log_k_chunk,
            joint_col_vars,
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
            cycle_phase_row_rounds: row_binding_rounds,
            cycle_phase_col_rounds: col_binding_rounds,
            r_val_eval,
            r_val_final,
        }
    }

    /// (Total # advice variables) - (# variables bound during cycle phase)
    pub fn num_address_phase_rounds(&self) -> usize {
        (self.advice_col_vars + self.advice_row_vars)
            - (self.cycle_phase_col_rounds.len() + self.cycle_phase_row_rounds.len())
    }
}

impl<F: JoltField> AdviceClaimReductionParams<F> {
    fn total_poly_vars(&self) -> usize {
        self.advice_col_vars + self.advice_row_vars
    }

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

    fn num_rounds_for_current_phase(&self) -> usize {
        precommitted_num_rounds(
            self.phase,
            PreCommitted::CycleVariables,
            &self.cycle_phase_col_rounds,
            &self.cycle_phase_row_rounds,
            self.total_poly_vars(),
        )
    }

    pub fn round_offset(&self, max_num_rounds: usize) -> usize {
        if self.is_cycle_phase() {
            max_num_rounds.saturating_sub(self.cycle_alignment_rounds())
        } else {
            max_num_rounds.saturating_sub(self.num_rounds_for_current_phase())
        }
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
        self.num_rounds_for_current_phase()
    }

    fn normalize_opening_point(&self, challenges: &[F::Challenge]) -> OpeningPoint<BIG_ENDIAN, F> {
        normalize_two_phase_opening_point(
            self.phase,
            &self.cycle_var_challenges,
            PreCommitted::CycleVariables,
            &self.cycle_phase_col_rounds,
            &self.cycle_phase_row_rounds,
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

        let joint_cols = 1 << params.joint_col_vars;
        // Maps a (row, col) position in the Dory matrix layout to its
        // implied (address, cycle).
        let row_col_to_address_cycle = |row: usize, col: usize| -> (usize, usize) {
            match DoryGlobals::get_layout() {
                DoryLayout::CycleMajor => {
                    let global_index = row as u128 * joint_cols + col as u128;
                    let address = global_index / (1 << params.log_t);
                    let cycle = global_index % (1 << params.log_t);
                    (address as usize, cycle as usize)
                }
                DoryLayout::AddressMajor => {
                    let global_index = row as u128 * joint_cols + col as u128;
                    let address = global_index % (1 << params.log_k_chunk);
                    let cycle = global_index / (1 << params.log_k_chunk);
                    (address as usize, cycle as usize)
                }
            }
        };

        let advice_cols = 1 << params.advice_col_vars;
        // Maps an index in the advice vector to its implied (address, cycle), based
        // on the position the index maps to in the Dory matrix layout.
        let advice_index_to_address_cycle = |index: usize| -> (usize, usize) {
            let row = index / advice_cols;
            let col = index % advice_cols;
            row_col_to_address_cycle(row, col)
        };

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
            let (address_a, cycle_a) = advice_index_to_address_cycle(index_a);
            let (address_b, cycle_b) = advice_index_to_address_cycle(index_b);
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
        let advice_poly: MultilinearPolynomial<F> = advice_coeffs.into();
        let eq_poly: MultilinearPolynomial<F> = eq_coeffs.into();

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
        if self.params.is_cycle_phase()
            && !self.params.cycle_phase_col_rounds.contains(&round)
            && !self.params.cycle_phase_row_rounds.contains(&round)
        {
            return UniPoly::from_coeff(vec![previous_claim * F::from_u64(2).inverse().unwrap()]);
        }

        let trailing_cap = if self.params.is_cycle_phase() {
            self.params.cycle_alignment_rounds()
        } else {
            self.params.address_alignment_rounds()
        };
        let num_trailing_variables =
            trailing_cap.saturating_sub(self.params.num_rounds_for_current_phase());
        let scaling_factor = self.scale * F::one().mul_pow_2(num_trailing_variables);
        let prev_unscaled = previous_claim * scaling_factor.inverse().unwrap();
        let poly_unscaled = self.compute_message_unscaled(prev_unscaled);
        poly_unscaled * scaling_factor
    }

    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        if self.params.is_cycle_phase() {
            let is_dummy_round = !self.params.cycle_phase_col_rounds.contains(&round)
                && !self.params.cycle_phase_row_rounds.contains(&round);
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

                let gap_len = internal_dummy_gap_len(
                    &params.cycle_phase_col_rounds,
                    &params.cycle_phase_row_rounds,
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

        if params.num_address_phase_rounds() == 0
            || params.phase == PreCommitted::AddressVariables
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
