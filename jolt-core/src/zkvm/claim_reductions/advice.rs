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
use crate::poly::multilinear_polynomial::MultilinearPolynomial;
use crate::poly::opening_proof::{
    OpeningAccumulator, OpeningPoint, ProverOpeningAccumulator, SumcheckId,
    VerifierOpeningAccumulator, BIG_ENDIAN,
};
use crate::subprotocols::sumcheck_verifier::{SumcheckInstanceParams, SumcheckInstanceVerifier};
use crate::transcripts::Transcript;
use crate::utils::math::Math;
use crate::zkvm::claim_reductions::{
    cycle_phase_round_schedule, internal_dummy_gap_len, PreCommittedClaimReductionParams,
    PreCommitted, PreCommittedPolyClaimReduction, PreCommittedPolyClaimReductionState,
    PreCommittedPolyReductionCore,
    PreCommittedSumcheckInstanceParams, PreCommittedSumcheckInstanceProver,
};
use crate::zkvm::config::OneHotConfig;
use allocative::Allocative;
use common::jolt_device::MemoryLayout;
use rayon::prelude::*;

const DEGREE_BOUND: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Allocative)]
pub enum AdviceKind {
    Trusted,
    Untrusted,
}

#[derive(Clone, Allocative)]
pub struct AdviceClaimReductionParams<F: JoltField> {
    pub kind: AdviceKind,
    pub reduction: PreCommittedPolyClaimReductionState<F, PreCommitted>,
    pub gamma: F,
    pub single_opening: bool,
    pub log_k_chunk: usize,
    pub log_t: usize,
    pub advice_col_vars: usize,
    pub advice_row_vars: usize,
    /// Number of column variables in the main Dory matrix
    pub main_col_vars: usize,
    /// Number of row variables in the main Dory matrix
    pub main_row_vars: usize,
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
        accumulator: &dyn OpeningAccumulator<F>,
        transcript: &mut impl Transcript,
        single_opening: bool,
    ) -> Self {
        let max_advice_size_bytes = match kind {
            AdviceKind::Trusted => memory_layout.max_trusted_advice_size as usize,
            AdviceKind::Untrusted => memory_layout.max_untrusted_advice_size as usize,
        };

        let log_t = trace_len.log_2();
        let log_k_chunk = OneHotConfig::new(log_t).log_k_chunk as usize;
        let (main_col_vars, main_row_vars) = DoryGlobals::try_get_main_sigma_nu()
            .unwrap_or_else(|| DoryGlobals::main_sigma_nu(log_k_chunk, log_t));

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
            log_t,
            log_k_chunk,
            main_col_vars,
            advice_row_vars,
            advice_col_vars,
        );

        Self {
            kind,
            reduction: PreCommittedPolyClaimReductionState::new(PreCommitted::CycleVariables),
            gamma,
            advice_col_vars,
            advice_row_vars,
            single_opening,
            log_k_chunk,
            log_t,
            main_col_vars,
            main_row_vars,
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

impl<F: JoltField> crate::zkvm::claim_reductions::precommitted::sealed::Sealed
    for AdviceClaimReductionParams<F>
{
}

impl<F: JoltField> PreCommittedSumcheckInstanceParams<F> for AdviceClaimReductionParams<F> {
    fn precommitted_input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        match self.reduction.phase {
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

    fn precommitted_degree(&self) -> usize {
        DEGREE_BOUND
    }
}

impl<F: JoltField> PreCommittedClaimReductionParams<F> for AdviceClaimReductionParams<F> {
    fn reduction(&self) -> &PreCommittedPolyClaimReductionState<F, PreCommitted> {
        &self.reduction
    }

    fn reduction_mut(&mut self) -> &mut PreCommittedPolyClaimReductionState<F, PreCommitted> {
        &mut self.reduction
    }

    fn cycle_phase_col_rounds(&self) -> &Range<usize> {
        &self.cycle_phase_col_rounds
    }

    fn cycle_phase_row_rounds(&self) -> &Range<usize> {
        &self.cycle_phase_row_rounds
    }

    fn total_poly_vars(&self) -> usize {
        self.advice_col_vars + self.advice_row_vars
    }

    fn cycle_alignment_rounds(&self) -> usize {
        self.log_t
    }

    fn address_alignment_rounds(&self) -> usize {
        self.log_k_chunk
    }
}

#[derive(Allocative)]
pub struct AdviceClaimReductionProver<F: JoltField> {
    core: PreCommittedPolyReductionCore<F, AdviceClaimReductionParams<F>>,
}

impl<F: JoltField> crate::zkvm::claim_reductions::precommitted::sealed::Sealed
    for AdviceClaimReductionProver<F>
{
}

impl<F: JoltField> AdviceClaimReductionProver<F> {
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

        let main_cols = 1 << params.main_col_vars;
        // Maps a (row, col) position in the Dory matrix layout to its
        // implied (address, cycle).
        let row_col_to_address_cycle = |row: usize, col: usize| -> (usize, usize) {
            match DoryGlobals::get_layout() {
                DoryLayout::CycleMajor => {
                    let global_index = row as u128 * main_cols + col as u128;
                    let address = global_index / (1 << params.log_t);
                    let cycle = global_index % (1 << params.log_t);
                    (address as usize, cycle as usize)
                }
                DoryLayout::AddressMajor => {
                    let global_index = row as u128 * main_cols + col as u128;
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
            core: PreCommittedPolyReductionCore::new(params, advice_poly, eq_poly),
        }
    }
}

impl<F: JoltField> PreCommittedPolyClaimReduction<F> for AdviceClaimReductionProver<F> {
    type Params = AdviceClaimReductionParams<F>;

    fn precommitted_core(&self) -> &PreCommittedPolyReductionCore<F, Self::Params> {
        &self.core
    }

    fn precommitted_core_mut(&mut self) -> &mut PreCommittedPolyReductionCore<F, Self::Params> {
        &mut self.core
    }
}

impl<F: JoltField, T: Transcript> PreCommittedSumcheckInstanceProver<F, T>
    for AdviceClaimReductionProver<F>
{
    fn precommitted_cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let params = self.params();
        let opening_point = params.normalize_opening_point(sumcheck_challenges);
        if params.reduction.phase == PreCommitted::CycleVariables {
            let c_mid = <Self as PreCommittedPolyClaimReduction<F>>::cycle_intermediate_claim(self);

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

        if let Some(advice_claim) =
            <Self as PreCommittedPolyClaimReduction<F>>::final_claim_if_ready(self)
        {
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
    fn precommitted_update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
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
        accumulator: &VerifierOpeningAccumulator<F>,
        transcript: &mut impl Transcript,
        single_opening: bool,
    ) -> Self {
        let params = AdviceClaimReductionParams::new(
            kind,
            memory_layout,
            trace_len,
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
        match params.reduction.phase {
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
        if params.reduction.phase == PreCommitted::CycleVariables {
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
            params
                .reduction
                .set_cycle_challenges_from_opening_point(&opening_point);
        }

        if params.num_address_phase_rounds() == 0
            || params.reduction.phase == PreCommitted::AddressVariables
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
