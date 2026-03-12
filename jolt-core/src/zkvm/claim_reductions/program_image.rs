//! Program-image (initial RAM) claim reduction.
//!
//! In committed bytecode mode, Stage 4 consumes prover-supplied scalar claims for the
//! program-image contribution to `Val_init(r_address)` without materializing the initial RAM.
//! This sumcheck binds those scalars to a trusted commitment to the program-image words polynomial.

use allocative::Allocative;
use std::cell::RefCell;

use rayon::prelude::*;

use crate::field::JoltField;
use crate::poly::commitment::dory::DoryGlobals;
use crate::poly::eq_poly::EqPolynomial;
use crate::poly::multilinear_polynomial::MultilinearPolynomial;
use crate::poly::opening_proof::{
    OpeningAccumulator, OpeningPoint, ProverOpeningAccumulator, SumcheckId,
    VerifierOpeningAccumulator, BIG_ENDIAN, LITTLE_ENDIAN,
};
use crate::poly::unipoly::UniPoly;
use crate::subprotocols::sumcheck_prover::SumcheckInstanceProver;
use crate::subprotocols::sumcheck_verifier::{SumcheckInstanceParams, SumcheckInstanceVerifier};
use crate::transcripts::Transcript;
use crate::utils::math::Math;
use crate::zkvm::claim_reductions::{
    permute_precommitted_polys, precommitted_eq_evals_with_scaling, precommitted_skip_round_scale,
    PrecomittedParams, PrecomittedProver, PrecommittedClaimReduction,
    PrecommittedSchedulingReference, TWO_PHASE_DEGREE_BOUND,
};
use crate::zkvm::config::ReadWriteConfig;
use crate::zkvm::ram::remap_address;
use crate::zkvm::witness::{CommittedPolynomial, VirtualPolynomial};
use tracer::JoltDevice;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Allocative)]
pub enum PreCommitted {
    CycleVariables,
    AddressVariables,
}

#[derive(Clone, Allocative)]
pub struct ProgramImageClaimReductionParams<F: JoltField> {
    pub phase: PreCommitted,
    pub precommitted: PrecommittedClaimReduction<F>,
    pub gamma: F,
    pub single_opening: bool,
    pub log_t: usize,
    pub prog_col_vars: usize,
    pub prog_row_vars: usize,
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
        self.precommitted.num_address_phase_rounds()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        program_io: &JoltDevice,
        ram_min_bytecode_address: u64,
        padded_len_words: usize,
        ram_K: usize,
        rw_config: &ReadWriteConfig,
        scheduling_reference: PrecommittedSchedulingReference,
        accumulator: &dyn OpeningAccumulator<F>,
        transcript: &mut impl Transcript,
    ) -> Self {
        let ram_num_vars = ram_K.log_2();
        let start_index =
            remap_address(ram_min_bytecode_address, &program_io.memory_layout).unwrap() as usize;
        let m = padded_len_words.log_2();
        debug_assert!(padded_len_words.is_power_of_two());
        debug_assert!(padded_len_words > 0);
        let (prog_col_vars, prog_row_vars) = DoryGlobals::balanced_sigma_nu(m);
        let log_t = DoryGlobals::main_t().log_2();
        let total_vars = prog_row_vars + prog_col_vars;
        let precommitted = PrecommittedClaimReduction::new(
            total_vars,
            prog_row_vars,
            prog_col_vars,
            scheduling_reference,
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
            phase: PreCommitted::CycleVariables,
            precommitted,
            gamma,
            single_opening,
            log_t,
            prog_col_vars,
            prog_row_vars,
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

impl<F: JoltField> ProgramImageClaimReductionParams<F> {
    fn is_cycle_phase(&self) -> bool {
        self.phase == PreCommitted::CycleVariables
    }

    pub fn transition_to_address_phase(&mut self) {
        self.phase = PreCommitted::AddressVariables;
    }

    pub fn round_offset(&self, max_num_rounds: usize) -> usize {
        self.precommitted
            .round_offset(self.is_cycle_phase(), max_num_rounds)
    }
}

impl<F: JoltField> SumcheckInstanceParams<F> for ProgramImageClaimReductionParams<F> {
    fn input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        match self.phase {
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

    fn degree(&self) -> usize {
        TWO_PHASE_DEGREE_BOUND
    }

    fn num_rounds(&self) -> usize {
        self.precommitted
            .num_rounds_for_phase(self.is_cycle_phase())
    }

    fn normalize_opening_point(&self, challenges: &[F::Challenge]) -> OpeningPoint<BIG_ENDIAN, F> {
        self.precommitted
            .normalize_opening_point(self.is_cycle_phase(), challenges, self.log_t)
    }
}

impl<F: JoltField> PrecomittedParams<F> for ProgramImageClaimReductionParams<F> {
    fn is_cycle_phase(&self) -> bool {
        self.phase == PreCommitted::CycleVariables
    }

    fn is_cycle_phase_round(&self, round: usize) -> bool {
        self.precommitted.is_cycle_phase_round(round)
    }

    fn is_address_phase_round(&self, round: usize) -> bool {
        self.precommitted.is_address_phase_round(round)
    }

    fn cycle_alignment_rounds(&self) -> usize {
        self.precommitted.cycle_alignment_rounds()
    }

    fn address_alignment_rounds(&self) -> usize {
        self.precommitted.address_alignment_rounds()
    }

    fn record_cycle_challenge(&mut self, challenge: F::Challenge) {
        self.precommitted.record_cycle_challenge(challenge);
    }
}

#[derive(Allocative)]
pub struct ProgramImageClaimReductionProver<F: JoltField> {
    core: PrecomittedProver<F, ProgramImageClaimReductionParams<F>>,
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
    pub fn params(&self) -> &ProgramImageClaimReductionParams<F> {
        self.core.params()
    }

    pub fn transition_to_address_phase(&mut self) {
        self.core.params_mut().transition_to_address_phase();
    }

    #[tracing::instrument(skip_all, name = "ProgramImageClaimReductionProver::initialize")]
    pub fn initialize(
        params: ProgramImageClaimReductionParams<F>,
        program_image_words_padded: Vec<u64>,
    ) -> Self {
        debug_assert_eq!(program_image_words_padded.len(), params.padded_len_words);
        debug_assert_eq!(params.padded_len_words, 1usize << params.m);

        let eq_evals = if params.single_opening {
            precommitted_eq_evals_with_scaling(
                &params.r_addr_rw_reduced,
                Some(params.selector_rw.clone()),
                &params.precommitted,
            )
        } else {
            let evals = precommitted_eq_evals_with_scaling(
                &params.r_addr_rw_reduced,
                Some(params.selector_rw.clone()),
                &params.precommitted,
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
            let eq_final = precommitted_eq_evals_with_scaling(
                r_addr_raf_reduced,
                Some(selector_raf),
                &params.precommitted,
            );
            evals
                .par_iter()
                .zip(eq_final.par_iter())
                .map(|(e1, e2)| *e1 + params.gamma * *e2)
                .collect()
        };

        // Permute ProgramWord and eq_slice so low-to-high binding follows the two-phase
        // schedule while preserving top-left projection semantics against the joint point.
        let (program_word, eq_slice): (MultilinearPolynomial<F>, MultilinearPolynomial<F>) = {
            let mut permuted =
                permute_precommitted_polys(vec![program_image_words_padded], &params.precommitted)
                    .into_iter();
            let program_word = permuted
                .next()
                .expect("expected one permuted program image polynomial");
            let eq_slice = eq_evals.into();
            (program_word, eq_slice)
        };

        Self {
            core: PrecomittedProver::new(params, program_word, eq_slice),
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T>
    for ProgramImageClaimReductionProver<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        self.core.params()
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        self.core.params().round_offset(max_num_rounds)
    }

    fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
        self.core.compute_message(round, previous_claim)
    }

    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        self.core.ingest_challenge(r_j, round);
    }

    fn cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let params = self.core.params();
        let opening_point = params.normalize_opening_point(sumcheck_challenges);
        if params.phase == PreCommitted::CycleVariables {
            let c_mid = self.core.cycle_intermediate_claim();
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::ProgramImageInit,
                SumcheckId::ProgramImageClaimReductionCyclePhase,
                opening_point.r.clone(),
                c_mid,
            );
        }

        if let Some(claim) = self.core.final_claim_if_ready() {
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
        match params.phase {
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

                let scale: F = precommitted_skip_round_scale(&params.precommitted);
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
        if params.phase == PreCommitted::CycleVariables {
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::ProgramImageInit,
                SumcheckId::ProgramImageClaimReductionCyclePhase,
                opening_point.r.clone(),
            );
            let opening_point_le: OpeningPoint<LITTLE_ENDIAN, F> = opening_point.match_endianness();
            params
                .precommitted
                .set_cycle_var_challenges(opening_point_le.r);
        }

        if params.phase == PreCommitted::AddressVariables || params.num_address_phase_rounds() == 0
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
