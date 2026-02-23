//! Program-image (initial RAM) claim reduction.
//!
//! In committed bytecode mode, Stage 4 consumes prover-supplied scalar claims for the
//! program-image contribution to `Val_init(r_address)` without materializing the initial RAM.
//! This sumcheck binds those scalars to a trusted commitment to the program-image words polynomial.

use allocative::Allocative;
use std::cell::RefCell;
use std::cmp::{min, Ordering};
use std::ops::Range;

use rayon::prelude::*;

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
use crate::zkvm::claim_reductions::advice::ReductionPhase;
use crate::zkvm::config::ReadWriteConfig;
use crate::zkvm::ram::remap_address;
use crate::zkvm::witness::{CommittedPolynomial, VirtualPolynomial};
use tracer::JoltDevice;

const DEGREE_BOUND: usize = 2;

#[derive(Clone, Allocative)]
pub struct ProgramImageClaimReductionParams<F: JoltField> {
    pub phase: ReductionPhase,
    pub gamma: F,
    pub single_opening: bool,
    pub log_k_chunk: usize,
    pub log_t: usize,
    pub prog_col_vars: usize,
    pub prog_row_vars: usize,
    pub main_col_vars: usize,
    #[allocative(skip)]
    pub cycle_phase_row_rounds: Range<usize>,
    #[allocative(skip)]
    pub cycle_phase_col_rounds: Range<usize>,
    /// (little-endian) challenges for cycle-phase variables
    pub cycle_var_challenges: Vec<F::Challenge>,
    pub ram_num_vars: usize,
    pub start_index: usize,
    pub padded_len_words: usize,
    pub m: usize,
    pub r_addr_rw: Vec<F::Challenge>,
    pub r_addr_raf: Option<Vec<F::Challenge>>,
}

fn cycle_phase_round_schedule(
    log_t: usize,
    log_k_chunk: usize,
    main_col_vars: usize,
    prog_row_vars: usize,
    prog_col_vars: usize,
) -> (Range<usize>, Range<usize>) {
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => {
            let col_binding_rounds = 0..min(log_t, prog_col_vars);
            let row_binding_rounds =
                min(log_t, main_col_vars)..min(log_t, main_col_vars + prog_row_vars);
            (col_binding_rounds, row_binding_rounds)
        }
        DoryLayout::AddressMajor => {
            let col_binding_rounds = 0..prog_col_vars.saturating_sub(log_k_chunk);
            let row_binding_rounds = main_col_vars.saturating_sub(log_k_chunk)
                ..min(
                    log_t,
                    main_col_vars.saturating_sub(log_k_chunk) + prog_row_vars,
                );
            (col_binding_rounds, row_binding_rounds)
        }
    }
}

impl<F: JoltField> ProgramImageClaimReductionParams<F> {
    pub fn num_address_phase_rounds(&self) -> usize {
        (self.prog_col_vars + self.prog_row_vars)
            - (self.cycle_phase_col_rounds.len() + self.cycle_phase_row_rounds.len())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        program_io: &JoltDevice,
        ram_min_bytecode_address: u64,
        padded_len_words: usize,
        ram_K: usize,
        trace_len: usize,
        log_k_chunk: usize,
        main_num_columns: usize,
        rw_config: &ReadWriteConfig,
        accumulator: &dyn OpeningAccumulator<F>,
        transcript: &mut impl Transcript,
    ) -> Self {
        let ram_num_vars = ram_K.log_2();
        let start_index =
            remap_address(ram_min_bytecode_address, &program_io.memory_layout).unwrap() as usize;
        let m = padded_len_words.log_2();
        debug_assert!(padded_len_words.is_power_of_two());
        debug_assert!(padded_len_words > 0);
        let log_t = trace_len.log_2();
        let (prog_col_vars, prog_row_vars) = DoryGlobals::balanced_sigma_nu(m);
        debug_assert!(main_num_columns.is_power_of_two());
        let main_col_vars = main_num_columns.log_2();
        let (cycle_phase_col_rounds, cycle_phase_row_rounds) = cycle_phase_round_schedule(
            log_t,
            log_k_chunk,
            main_col_vars,
            prog_row_vars,
            prog_col_vars,
        );

        // r_address_rw comes from RamVal/RamReadWriteChecking (Stage 2).
        let (r_rw, _) = accumulator.get_virtual_polynomial_opening(
            VirtualPolynomial::RamVal,
            SumcheckId::RamReadWriteChecking,
        );
        let (r_addr_rw, _) = r_rw.split_at(ram_num_vars);

        // r_address_raf comes from RamValFinal/RamOutputCheck (Stage 2), but may equal r_address_rw.
        let single_opening = rw_config.needs_single_advice_opening(log_t);
        let r_addr_raf = if single_opening {
            None
        } else {
            let (r_raf, _) = accumulator.get_virtual_polynomial_opening(
                VirtualPolynomial::RamValFinal,
                SumcheckId::RamOutputCheck,
            );
            let (r_addr_raf, _) = r_raf.split_at(ram_num_vars);
            Some(r_addr_raf.r)
        };

        // Sample gamma for combining rw + raf.
        let gamma: F = transcript.challenge_scalar();

        Self {
            phase: ReductionPhase::CycleVariables,
            gamma,
            single_opening,
            log_k_chunk,
            log_t,
            prog_col_vars,
            prog_row_vars,
            main_col_vars,
            cycle_phase_row_rounds,
            cycle_phase_col_rounds,
            cycle_var_challenges: vec![],
            ram_num_vars,
            start_index,
            padded_len_words,
            m,
            r_addr_rw: r_addr_rw.r,
            r_addr_raf,
        }
    }
}

impl<F: JoltField> SumcheckInstanceParams<F> for ProgramImageClaimReductionParams<F> {
    fn input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        match self.phase {
            ReductionPhase::CycleVariables => {
                // Scalar claims were staged in Stage 4 as virtual openings.
                let (_, c_rw) = accumulator.get_virtual_polynomial_opening(
                    VirtualPolynomial::ProgramImageInitContributionRw,
                    SumcheckId::RamValEvaluation,
                );
                if self.single_opening {
                    c_rw
                } else {
                    let (_, c_raf) = accumulator.get_virtual_polynomial_opening(
                        VirtualPolynomial::ProgramImageInitContributionRaf,
                        SumcheckId::RamValFinalEvaluation,
                    );
                    c_rw + self.gamma * c_raf
                }
            }
            ReductionPhase::AddressVariables => {
                accumulator
                    .get_committed_polynomial_opening(
                        CommittedPolynomial::ProgramImageInit,
                        SumcheckId::ProgramImageClaimReductionCyclePhase,
                    )
                    .1
            }
        }
    }

    fn degree(&self) -> usize {
        DEGREE_BOUND
    }

    fn num_rounds(&self) -> usize {
        match self.phase {
            ReductionPhase::CycleVariables => {
                if !self.cycle_phase_row_rounds.is_empty() {
                    self.cycle_phase_row_rounds.end - self.cycle_phase_col_rounds.start
                } else {
                    self.cycle_phase_col_rounds.len()
                }
            }
            ReductionPhase::AddressVariables => {
                let first_phase_rounds =
                    self.cycle_phase_col_rounds.len() + self.cycle_phase_row_rounds.len();
                self.m - first_phase_rounds
            }
        }
    }

    fn normalize_opening_point(
        &self,
        challenges: &[<F as JoltField>::Challenge],
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        if self.phase == ReductionPhase::CycleVariables {
            let mut prog_var_challenges: Vec<F::Challenge> =
                Vec::with_capacity(self.m - self.num_address_phase_rounds());
            prog_var_challenges.extend_from_slice(&challenges[self.cycle_phase_col_rounds.clone()]);
            prog_var_challenges.extend_from_slice(&challenges[self.cycle_phase_row_rounds.clone()]);
            return OpeningPoint::<LITTLE_ENDIAN, F>::new(prog_var_challenges).match_endianness();
        }

        match DoryGlobals::get_layout() {
            DoryLayout::CycleMajor => OpeningPoint::<LITTLE_ENDIAN, F>::new(
                [self.cycle_var_challenges.as_slice(), challenges].concat(),
            )
            .match_endianness(),
            DoryLayout::AddressMajor => OpeningPoint::<LITTLE_ENDIAN, F>::new(
                [challenges, self.cycle_var_challenges.as_slice()].concat(),
            )
            .match_endianness(),
        }
    }
}

#[derive(Allocative)]
pub struct ProgramImageClaimReductionProver<F: JoltField> {
    pub params: ProgramImageClaimReductionParams<F>,
    program_word: MultilinearPolynomial<F>,
    eq_slice: MultilinearPolynomial<F>,
    /// Internal scaling for dummy rounds in the cycle phase.
    scale: F,
}

fn build_eq_slice_table<F: JoltField>(
    r_addr: &[F::Challenge],
    start_index: usize,
    len: usize,
) -> Vec<F> {
    debug_assert!(len.is_power_of_two());
    let mut out = Vec::with_capacity(len);
    let mut idx = start_index;
    let mut off = 0usize;
    while off < len {
        let remaining = len - off;
        let (block_size, block_evals) =
            EqPolynomial::<F>::evals_for_max_aligned_block(r_addr, idx, remaining);
        out.extend_from_slice(&block_evals);
        idx += block_size;
        off += block_size;
    }
    debug_assert_eq!(out.len(), len);
    out
}

#[inline]
fn row_col_to_address_cycle(
    row: usize,
    col: usize,
    main_col_vars: usize,
    log_t: usize,
    log_k_chunk: usize,
    layout: DoryLayout,
) -> (usize, usize) {
    let main_cols = 1usize << main_col_vars;
    let global_index = row as u128 * main_cols as u128 + col as u128;
    match layout {
        DoryLayout::CycleMajor => {
            let address = global_index / (1u128 << log_t);
            let cycle = global_index % (1u128 << log_t);
            (address as usize, cycle as usize)
        }
        DoryLayout::AddressMajor => {
            let address = global_index % (1u128 << log_k_chunk);
            let cycle = global_index / (1u128 << log_k_chunk);
            (address as usize, cycle as usize)
        }
    }
}

fn permute_program_image_coeffs_by_address_cycle<F: JoltField>(
    coeffs: Vec<F>,
    prog_col_vars: usize,
    main_col_vars: usize,
    log_t: usize,
    log_k_chunk: usize,
    layout: DoryLayout,
) -> Vec<F> {
    let prog_cols = 1usize << prog_col_vars;
    let mut indexed_coeffs: Vec<(usize, F)> = coeffs.into_par_iter().enumerate().collect();
    indexed_coeffs.par_sort_by(|&(index_a, _), &(index_b, _)| {
        let (address_a, cycle_a) = row_col_to_address_cycle(
            index_a / prog_cols,
            index_a % prog_cols,
            main_col_vars,
            log_t,
            log_k_chunk,
            layout,
        );
        let (address_b, cycle_b) = row_col_to_address_cycle(
            index_b / prog_cols,
            index_b % prog_cols,
            main_col_vars,
            log_t,
            log_k_chunk,
            layout,
        );
        match address_a.cmp(&address_b) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => cycle_a.cmp(&cycle_b),
        }
    });
    indexed_coeffs
        .into_par_iter()
        .map(|(_, coeff)| coeff)
        .collect()
}

fn eval_program_image_eq_selector_at_bound_point<F: JoltField>(
    r_addr: &[F::Challenge],
    start_index: usize,
    padded_len_words: usize,
    prog_col_vars: usize,
    main_col_vars: usize,
    log_t: usize,
    log_k_chunk: usize,
    layout: DoryLayout,
    cycle_var_challenges: &[F::Challenge],
    address_var_challenges: &[F::Challenge],
) -> F {
    let eq_slice = build_eq_slice_table::<F>(r_addr, start_index, padded_len_words);
    let eq_coeffs = permute_program_image_coeffs_by_address_cycle(
        eq_slice,
        prog_col_vars,
        main_col_vars,
        log_t,
        log_k_chunk,
        layout,
    );
    let mut eq_poly = MultilinearPolynomial::from(eq_coeffs);
    for &r in cycle_var_challenges {
        eq_poly.bind_parallel(r, BindingOrder::LowToHigh);
    }
    for &r in address_var_challenges {
        eq_poly.bind_parallel(r, BindingOrder::LowToHigh);
    }
    eq_poly.final_sumcheck_claim()
}

#[inline]
fn cycle_phase_internal_gap_len(
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

impl<F: JoltField> ProgramImageClaimReductionProver<F> {
    #[tracing::instrument(skip_all, name = "ProgramImageClaimReductionProver::initialize")]
    pub fn initialize(
        params: ProgramImageClaimReductionParams<F>,
        program_image_words_padded: Vec<u64>,
    ) -> Self {
        debug_assert_eq!(program_image_words_padded.len(), params.padded_len_words);
        debug_assert_eq!(params.padded_len_words, 1usize << params.m);

        let eq_rw = build_eq_slice_table::<F>(
            &params.r_addr_rw,
            params.start_index,
            params.padded_len_words,
        );
        let mut eq_comb = eq_rw;
        if !params.single_opening {
            let r_raf = params.r_addr_raf.as_ref().expect("missing raf address");
            let eq_raf =
                build_eq_slice_table::<F>(r_raf, params.start_index, params.padded_len_words);
            for (c, e) in eq_comb.iter_mut().zip(eq_raf.iter()) {
                *c += params.gamma * *e;
            }
        }
        // Permute ProgramWord and eq_slice by (address, cycle) so binding low-to-high is
        // equivalent to binding cycle vars first, then address vars.
        let main_cols = 1usize << params.main_col_vars;
        let row_col_to_address_cycle = |row: usize, col: usize| -> (usize, usize) {
            match DoryGlobals::get_layout() {
                DoryLayout::CycleMajor => {
                    let global_index = row as u128 * main_cols as u128 + col as u128;
                    let address = global_index / (1u128 << params.log_t);
                    let cycle = global_index % (1u128 << params.log_t);
                    (address as usize, cycle as usize)
                }
                DoryLayout::AddressMajor => {
                    let global_index = row as u128 * main_cols as u128 + col as u128;
                    let address = global_index % (1u128 << params.log_k_chunk);
                    let cycle = global_index / (1u128 << params.log_k_chunk);
                    (address as usize, cycle as usize)
                }
            }
        };
        let prog_cols = 1usize << params.prog_col_vars;
        let prog_index_to_address_cycle = |index: usize| -> (usize, usize) {
            let row = index / prog_cols;
            let col = index % prog_cols;
            row_col_to_address_cycle(row, col)
        };

        let mut permuted_coeffs: Vec<(usize, (u64, F))> = program_image_words_padded
            .into_par_iter()
            .zip(eq_comb.into_par_iter())
            .enumerate()
            .collect();
        permuted_coeffs.par_sort_by(|&(index_a, _), &(index_b, _)| {
            let (address_a, cycle_a) = prog_index_to_address_cycle(index_a);
            let (address_b, cycle_b) = prog_index_to_address_cycle(index_b);
            match address_a.cmp(&address_b) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => cycle_a.cmp(&cycle_b),
            }
        });
        let (program_word_coeffs, eq_coeffs): (Vec<_>, Vec<_>) = permuted_coeffs
            .into_par_iter()
            .map(|(_, coeffs)| coeffs)
            .unzip();
        let program_word: MultilinearPolynomial<F> =
            MultilinearPolynomial::from(program_word_coeffs);
        let eq_slice: MultilinearPolynomial<F> = MultilinearPolynomial::from(eq_coeffs);

        Self {
            params,
            program_word,
            eq_slice,
            scale: F::one(),
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T>
    for ProgramImageClaimReductionProver<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        match self.params.phase {
            ReductionPhase::CycleVariables => max_num_rounds.saturating_sub(self.params.log_t),
            ReductionPhase::AddressVariables => 0,
        }
    }

    fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
        if self.params.phase == ReductionPhase::CycleVariables
            && !self.params.cycle_phase_col_rounds.contains(&round)
            && !self.params.cycle_phase_row_rounds.contains(&round)
        {
            UniPoly::from_coeff(vec![previous_claim * F::from_u64(2).inverse().unwrap()])
        } else {
            let num_trailing_variables = match self.params.phase {
                ReductionPhase::CycleVariables => {
                    self.params.log_t.saturating_sub(self.params.num_rounds())
                }
                ReductionPhase::AddressVariables => self
                    .params
                    .log_k_chunk
                    .saturating_sub(self.params.num_rounds()),
            };
            let scaling_factor = self.scale * F::one().mul_pow_2(num_trailing_variables);
            let prev_unscaled = previous_claim * scaling_factor.inverse().unwrap();
            let poly_unscaled = self.compute_message_unscaled(prev_unscaled);
            poly_unscaled * scaling_factor
        }
    }

    #[tracing::instrument(skip_all, name = "ProgramImageClaimReductionProver::ingest_challenge")]
    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        match self.params.phase {
            ReductionPhase::CycleVariables => {
                if !self.params.cycle_phase_col_rounds.contains(&round)
                    && !self.params.cycle_phase_row_rounds.contains(&round)
                {
                    self.scale *= F::from_u64(2).inverse().unwrap();
                } else {
                    self.program_word
                        .bind_parallel(r_j, BindingOrder::LowToHigh);
                    self.eq_slice.bind_parallel(r_j, BindingOrder::LowToHigh);
                    self.params.cycle_var_challenges.push(r_j);
                }
            }
            ReductionPhase::AddressVariables => {
                self.program_word
                    .bind_parallel(r_j, BindingOrder::LowToHigh);
                self.eq_slice.bind_parallel(r_j, BindingOrder::LowToHigh);
            }
        }
    }

    fn cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let opening_point = self.params.normalize_opening_point(sumcheck_challenges);
        if self.params.phase == ReductionPhase::CycleVariables {
            let len = self.program_word.len();
            debug_assert_eq!(len, self.eq_slice.len());

            let mut sum = F::zero();
            for i in 0..len {
                sum += self.program_word.get_bound_coeff(i) * self.eq_slice.get_bound_coeff(i);
            }
            let c_mid = sum * self.scale;
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::ProgramImageInit,
                SumcheckId::ProgramImageClaimReductionCyclePhase,
                opening_point.r.clone(),
                c_mid,
            );
        }

        if self.program_word.len() == 1 {
            let claim = self.program_word.final_sumcheck_claim();
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::ProgramImageInit,
                SumcheckId::ProgramImageClaimReduction,
                opening_point.r,
                claim,
            );
        }
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
        flamegraph.visit_root(self);
    }
}

impl<F: JoltField> ProgramImageClaimReductionProver<F> {
    #[tracing::instrument(skip_all, name = "ProgramImageClaimReductionProver::compute_message")]
    fn compute_message_unscaled(&mut self, previous_claim_unscaled: F) -> UniPoly<F> {
        let half = self.program_word.len() / 2;
        let program_word = &self.program_word;
        let eq_slice = &self.eq_slice;
        let evals: [F; DEGREE_BOUND] = (0..half)
            .into_par_iter()
            .map(|j| {
                let pw =
                    program_word.sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                let eq = eq_slice.sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                let mut out = [F::zero(); DEGREE_BOUND];
                for i in 0..DEGREE_BOUND {
                    out[i] = pw[i] * eq[i];
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
}

pub struct ProgramImageClaimReductionVerifier<F: JoltField> {
    pub params: RefCell<ProgramImageClaimReductionParams<F>>,
}

impl<F: JoltField, T: Transcript> SumcheckInstanceVerifier<F, T>
    for ProgramImageClaimReductionVerifier<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        unsafe { &*self.params.as_ptr() }
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        let params = self.params.borrow();
        match params.phase {
            ReductionPhase::CycleVariables => max_num_rounds.saturating_sub(params.log_t),
            ReductionPhase::AddressVariables => 0,
        }
    }

    fn expected_output_claim(
        &self,
        accumulator: &VerifierOpeningAccumulator<F>,
        sumcheck_challenges: &[F::Challenge],
    ) -> F {
        let params = self.params.borrow();
        match params.phase {
            ReductionPhase::CycleVariables => {
                accumulator
                    .get_committed_polynomial_opening(
                        CommittedPolynomial::ProgramImageInit,
                        SumcheckId::ProgramImageClaimReductionCyclePhase,
                    )
                    .1
            }
            ReductionPhase::AddressVariables => {
                let opening_point = params.normalize_opening_point(sumcheck_challenges);
                debug_assert_eq!(opening_point.len(), params.m);
                let pw_eval = accumulator
                    .get_committed_polynomial_opening(
                        CommittedPolynomial::ProgramImageInit,
                        SumcheckId::ProgramImageClaimReduction,
                    )
                    .1;
                let layout = DoryGlobals::get_layout();

                let eq_eval = eval_program_image_eq_selector_at_bound_point::<F>(
                    &params.r_addr_rw,
                    params.start_index,
                    params.padded_len_words,
                    params.prog_col_vars,
                    params.main_col_vars,
                    params.log_t,
                    params.log_k_chunk,
                    layout,
                    &params.cycle_var_challenges,
                    sumcheck_challenges,
                );
                let eq_combined = if params.single_opening {
                    eq_eval
                } else {
                    let r_final = params
                        .r_addr_raf
                        .as_ref()
                        .expect("r_addr_raf must exist when !single_opening");
                    let eq_final = eval_program_image_eq_selector_at_bound_point::<F>(
                        r_final,
                        params.start_index,
                        params.padded_len_words,
                        params.prog_col_vars,
                        params.main_col_vars,
                        params.log_t,
                        params.log_k_chunk,
                        layout,
                        &params.cycle_var_challenges,
                        sumcheck_challenges,
                    );
                    eq_eval + params.gamma * eq_final
                };

                let gap_len = cycle_phase_internal_gap_len(
                    &params.cycle_phase_col_rounds,
                    &params.cycle_phase_row_rounds,
                );
                let two_inv = F::from_u64(2).inverse().unwrap();
                let scale = (0..gap_len).fold(F::one(), |acc, _| acc * two_inv);
                pw_eval * eq_combined * scale
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
        let opening_point = params.normalize_opening_point(sumcheck_challenges);
        if params.phase == ReductionPhase::CycleVariables {
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::ProgramImageInit,
                SumcheckId::ProgramImageClaimReductionCyclePhase,
                opening_point.r.clone(),
            );
            let opening_point_le: OpeningPoint<LITTLE_ENDIAN, F> = opening_point.match_endianness();
            params.cycle_var_challenges = opening_point_le.r;
        }

        if params.phase == ReductionPhase::AddressVariables
            || params.num_address_phase_rounds() == 0
        {
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::ProgramImageInit,
                SumcheckId::ProgramImageClaimReduction,
                opening_point.r,
            );
        }
    }
}
