//! Program-image (initial RAM) claim reduction.
//!
//! In committed bytecode mode, Stage 4 consumes prover-supplied scalar claims for the
//! program-image contribution to `Val_init(r_address)` without materializing the initial RAM.
//! This sumcheck binds those scalars to a trusted commitment to the program-image words polynomial.

use allocative::Allocative;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::ops::Range;

use rayon::prelude::*;

use crate::field::JoltField;
use crate::poly::commitment::dory::{DoryGlobals, DoryLayout};
use crate::poly::eq_poly::EqPolynomial;
use crate::poly::multilinear_polynomial::MultilinearPolynomial;
use crate::poly::opening_proof::{
    OpeningAccumulator, ProverOpeningAccumulator, SumcheckId, VerifierOpeningAccumulator,
};
use crate::subprotocols::sumcheck_verifier::{SumcheckInstanceParams, SumcheckInstanceVerifier};
use crate::transcripts::Transcript;
use crate::utils::math::Math;
use crate::zkvm::claim_reductions::{
    cycle_phase_round_schedule, internal_dummy_gap_len, PreCommitted,
    PreCommittedClaimReductionParams, PreCommittedPolyClaimReduction,
    PreCommittedPolyClaimReductionState, PreCommittedPolyReductionCore,
    PreCommittedSumcheckInstanceParams,
    PreCommittedSumcheckInstanceProver,
};
use crate::zkvm::config::ReadWriteConfig;
use crate::zkvm::ram::remap_address;
use crate::zkvm::witness::{CommittedPolynomial, VirtualPolynomial};
use tracer::JoltDevice;

const DEGREE_BOUND: usize = 2;

#[derive(Clone, Allocative)]
pub struct ProgramImageClaimReductionParams<F: JoltField> {
    pub reduction: PreCommittedPolyClaimReductionState<F, PreCommitted>,
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
    pub ram_num_vars: usize,
    pub start_index: usize,
    pub padded_len_words: usize,
    pub m: usize,
    pub r_addr_rw: Vec<F::Challenge>,
    pub r_addr_raf: Option<Vec<F::Challenge>>,
    pub r_addr_rw_reduced: Vec<F::Challenge>,
    pub r_addr_raf_reduced: Option<Vec<F::Challenge>>,
    pub selector_rw: F,
    pub selector_raf: Option<F>,
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
        single_opening_log_t: usize,
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
        let single_opening = rw_config.needs_single_advice_opening(single_opening_log_t);
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
        let (r_addr_rw_reduced, selector_rw) = top_left_program_image_point_and_selector::<F>(
            &r_addr_rw.r,
            start_index,
            padded_len_words,
        );
        let (r_addr_raf_reduced, selector_raf) = if single_opening {
            (None, None)
        } else {
            let r_addr_raf_ref = r_addr_raf
                .as_ref()
                .expect("r_addr_raf must exist when !single_opening");
            let (reduced, selector) = top_left_program_image_point_and_selector::<F>(
                r_addr_raf_ref,
                start_index,
                padded_len_words,
            );
            (Some(reduced), Some(selector))
        };

        // Sample gamma for combining rw + raf.
        let gamma: F = transcript.challenge_scalar();

        Self {
            reduction: PreCommittedPolyClaimReductionState::new(PreCommitted::CycleVariables),
            gamma,
            single_opening,
            log_k_chunk,
            log_t,
            prog_col_vars,
            prog_row_vars,
            main_col_vars,
            cycle_phase_row_rounds,
            cycle_phase_col_rounds,
            ram_num_vars,
            start_index,
            padded_len_words,
            m,
            r_addr_rw: r_addr_rw.r,
            r_addr_raf,
            r_addr_rw_reduced,
            r_addr_raf_reduced,
            selector_rw,
            selector_raf,
        }
    }
}

impl<F: JoltField> crate::zkvm::claim_reductions::precommitted::sealed::Sealed
    for ProgramImageClaimReductionParams<F>
{
}

impl<F: JoltField> PreCommittedSumcheckInstanceParams<F> for ProgramImageClaimReductionParams<F> {
    fn precommitted_input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        match self.reduction.phase {
            PreCommitted::CycleVariables => {
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
            PreCommitted::AddressVariables => {
                accumulator
                    .get_committed_polynomial_opening(
                        CommittedPolynomial::ProgramImageInit,
                        SumcheckId::ProgramImageClaimReductionCyclePhase,
                    )
                    .1
            }
        }
    }

    fn precommitted_degree(&self) -> usize {
        DEGREE_BOUND
    }
}

impl<F: JoltField> PreCommittedClaimReductionParams<F> for ProgramImageClaimReductionParams<F> {
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
        self.m
    }

    fn cycle_alignment_rounds(&self) -> usize {
        self.log_t
    }

    fn address_alignment_rounds(&self) -> usize {
        self.log_k_chunk
    }
}

#[derive(Allocative)]
pub struct ProgramImageClaimReductionProver<F: JoltField> {
    core: PreCommittedPolyReductionCore<F, ProgramImageClaimReductionParams<F>>,
}

impl<F: JoltField> crate::zkvm::claim_reductions::precommitted::sealed::Sealed
    for ProgramImageClaimReductionProver<F>
{
}

fn top_left_program_image_point_and_selector<F: JoltField>(
    r_addr: &[F::Challenge],
    start_index: usize,
    padded_len_words: usize,
) -> (Vec<F::Challenge>, F) {
    assert!(
        padded_len_words.is_power_of_two() && padded_len_words > 0,
        "padded_len_words must be a non-zero power of two"
    );
    assert_eq!(
        start_index % padded_len_words,
        0,
        "program-image block must be aligned to padded_len_words for top-left embedding"
    );

    let m = padded_len_words.log_2();
    assert!(
        m <= r_addr.len(),
        "program-image variable count exceeds RAM address variable count"
    );
    let prefix_len = r_addr.len() - m;
    let start_prefix = start_index / padded_len_words;

    let mut selector = F::one();
    for (i, r_i) in r_addr[..prefix_len].iter().enumerate() {
        let bit_index = prefix_len - 1 - i;
        let prefix_bit = (start_prefix >> bit_index) & 1;
        let r_i_f: F = (*r_i).into();
        selector *= if prefix_bit == 1 {
            r_i_f
        } else {
            F::one() - r_i_f
        };
    }

    (r_addr[prefix_len..].to_vec(), selector)
}

impl<F: JoltField> ProgramImageClaimReductionProver<F> {
    #[tracing::instrument(skip_all, name = "ProgramImageClaimReductionProver::initialize")]
    pub fn initialize(
        params: ProgramImageClaimReductionParams<F>,
        program_image_words_padded: Vec<u64>,
    ) -> Self {
        debug_assert_eq!(program_image_words_padded.len(), params.padded_len_words);
        debug_assert_eq!(params.padded_len_words, 1usize << params.m);

        let eq_evals = if params.single_opening {
            EqPolynomial::evals_with_scaling(
                &params.r_addr_rw_reduced,
                Some(params.selector_rw.clone()),
            )
        } else {
            let evals = EqPolynomial::evals_with_scaling(
                &params.r_addr_rw_reduced,
                Some(params.selector_rw.clone()),
            );
            let r_addr_raf_reduced = params
                .r_addr_raf_reduced
                .as_ref()
                .expect("missing reduced raf address");
            let selector_raf = params
                .selector_raf
                .as_ref()
                .expect("missing reduced raf selector")
                .clone();
            let eq_final = EqPolynomial::evals_with_scaling(r_addr_raf_reduced, Some(selector_raf));
            evals
                .par_iter()
                .zip(eq_final.par_iter())
                .map(|(e1, e2)| *e1 + params.gamma * *e2)
                .collect()
        };

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
            .zip(eq_evals.into_par_iter())
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
            core: PreCommittedPolyReductionCore::new(params, program_word, eq_slice),
        }
    }
}

impl<F: JoltField> PreCommittedPolyClaimReduction<F> for ProgramImageClaimReductionProver<F> {
    type Params = ProgramImageClaimReductionParams<F>;

    fn precommitted_core(&self) -> &PreCommittedPolyReductionCore<F, Self::Params> {
        &self.core
    }

    fn precommitted_core_mut(&mut self) -> &mut PreCommittedPolyReductionCore<F, Self::Params> {
        &mut self.core
    }
}

impl<F: JoltField, T: Transcript> PreCommittedSumcheckInstanceProver<F, T>
    for ProgramImageClaimReductionProver<F>
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
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::ProgramImageInit,
                SumcheckId::ProgramImageClaimReductionCyclePhase,
                opening_point.r.clone(),
                c_mid,
            );
        }

        if let Some(claim) = <Self as PreCommittedPolyClaimReduction<F>>::final_claim_if_ready(self)
        {
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
    fn precommitted_update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
        flamegraph.visit_root(self);
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
        params.round_offset(max_num_rounds)
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
                    .get_committed_polynomial_opening(
                        CommittedPolynomial::ProgramImageInit,
                        SumcheckId::ProgramImageClaimReductionCyclePhase,
                    )
                    .1
            }
            PreCommitted::AddressVariables => {
                let opening_point = params.normalize_opening_point(sumcheck_challenges);
                debug_assert_eq!(opening_point.len(), params.m);
                let pw_eval = accumulator
                    .get_committed_polynomial_opening(
                        CommittedPolynomial::ProgramImageInit,
                        SumcheckId::ProgramImageClaimReduction,
                    )
                    .1;
                let eq_eval = params.selector_rw.clone()
                    * EqPolynomial::mle(&opening_point.r, &params.r_addr_rw_reduced);
                let eq_combined = if params.single_opening {
                    eq_eval
                } else {
                    let r_final = params
                        .r_addr_raf_reduced
                        .as_ref()
                        .expect("r_addr_raf_reduced must exist when !single_opening");
                    let selector_raf = params
                        .selector_raf
                        .as_ref()
                        .expect("selector_raf must exist when !single_opening")
                        .clone();
                    let eq_final = selector_raf * EqPolynomial::mle(&opening_point.r, r_final);
                    eq_eval + params.gamma * eq_final
                };

                let gap_len = internal_dummy_gap_len(
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
        if params.reduction.phase == PreCommitted::CycleVariables {
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::ProgramImageInit,
                SumcheckId::ProgramImageClaimReductionCyclePhase,
                opening_point.r.clone(),
            );
            params
                .reduction
                .set_cycle_challenges_from_opening_point(&opening_point);
        }

        if params.reduction.phase == PreCommitted::AddressVariables
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
