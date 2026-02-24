//! Two-phase bytecode claim reduction (Stage 6b cycle -> Stage 7 address),
//! implemented with the shared pre-committed reduction parent.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;

use allocative::Allocative;
use rayon::prelude::*;

use crate::field::JoltField;
use crate::poly::commitment::dory::{DoryGlobals, DoryLayout};
use crate::poly::eq_poly::EqPolynomial;
use crate::poly::multilinear_polynomial::{
    BindingOrder, MultilinearPolynomial, PolynomialBinding,
};
use crate::poly::opening_proof::{
    OpeningAccumulator, OpeningPoint, ProverOpeningAccumulator, SumcheckId,
    VerifierOpeningAccumulator, BIG_ENDIAN,
};
use crate::subprotocols::sumcheck_verifier::{SumcheckInstanceParams, SumcheckInstanceVerifier};
use crate::transcripts::Transcript;
use crate::utils::math::Math;
use crate::zkvm::bytecode::chunks::{build_committed_bytecode_polynomial_from_instructions, committed_lanes};
use crate::zkvm::bytecode::read_raf_checking::BytecodeReadRafSumcheckParams;
use crate::zkvm::claim_reductions::precommitted::sealed;
use crate::zkvm::claim_reductions::{
    cycle_phase_round_schedule, internal_dummy_gap_len, PreCommittedClaimReductionParams,
    PreCommittedPolyClaimReduction, PreCommittedPolyClaimReductionState,
    PreCommittedSumcheckInstanceParams, PreCommittedSumcheckInstanceProver,
};
use crate::zkvm::instruction::{
    CircuitFlags, InstructionFlags, NUM_CIRCUIT_FLAGS, NUM_INSTRUCTION_FLAGS,
};
use crate::zkvm::lookup_table::LookupTables;
use crate::zkvm::program::ProgramPreprocessing;
use crate::zkvm::witness::{CommittedPolynomial, VirtualPolynomial};
use common::constants::{REGISTER_COUNT, XLEN};
use strum::EnumCount;

const DEGREE_BOUND: usize = 2;
const NUM_VAL_STAGES: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Allocative)]
pub enum BytecodeReductionPhase {
    CycleVariables,
    AddressVariables,
}

#[derive(Clone, Allocative)]
pub struct BytecodeClaimReductionParams<F: JoltField> {
    pub reduction: PreCommittedPolyClaimReductionState<F, BytecodeReductionPhase>,
    pub eta: F,
    pub eta_powers: [F; NUM_VAL_STAGES],
    pub log_k: usize,
    pub log_t: usize,
    pub log_k_chunk: usize,
    pub bytecode_col_vars: usize,
    pub bytecode_row_vars: usize,
    pub main_col_vars: usize,
    pub main_row_vars: usize,
    #[allocative(skip)]
    pub cycle_phase_row_rounds: Range<usize>,
    #[allocative(skip)]
    pub cycle_phase_col_rounds: Range<usize>,
    pub r_bc: OpeningPoint<BIG_ENDIAN, F>,
    pub lane_weights: Vec<F>,
}

impl<F: JoltField> BytecodeClaimReductionParams<F> {
    pub fn new(
        bytecode_read_raf_params: &BytecodeReadRafSumcheckParams<F>,
        accumulator: &dyn OpeningAccumulator<F>,
        transcript: &mut impl Transcript,
    ) -> Self {
        let log_k = bytecode_read_raf_params.log_K;
        let log_t = bytecode_read_raf_params.log_T;
        let log_k_chunk = bytecode_read_raf_params.one_hot_params.log_k_chunk;

        let eta: F = transcript.challenge_scalar();
        let mut eta_powers = [F::one(); NUM_VAL_STAGES];
        for i in 1..NUM_VAL_STAGES {
            eta_powers[i] = eta_powers[i - 1] * eta;
        }

        let (r_bc, _) = accumulator.get_virtual_polynomial_opening(
            VirtualPolynomial::BytecodeReadRafAddrClaim,
            SumcheckId::BytecodeReadRafAddressPhase,
        );

        let lane_weights = compute_lane_weights(bytecode_read_raf_params, accumulator, &eta_powers);

        let (main_col_vars, main_row_vars) = DoryGlobals::try_get_main_sigma_nu()
            .unwrap_or_else(|| DoryGlobals::main_sigma_nu(log_k_chunk, log_t));
        let total_vars = committed_lanes().log_2() + log_k;
        // Bytecode uses its own balanced dimensions (independent from Main).
        // In Stage 8 it is embedded as a top-left block in Main.
        let (bytecode_col_vars, bytecode_row_vars) = DoryGlobals::balanced_sigma_nu(total_vars);
        let (cycle_phase_col_rounds, cycle_phase_row_rounds) = cycle_phase_round_schedule(
            log_t,
            log_k_chunk,
            main_col_vars,
            bytecode_row_vars,
            bytecode_col_vars,
        );

        Self {
            reduction: PreCommittedPolyClaimReductionState::new(
                BytecodeReductionPhase::CycleVariables,
            ),
            eta,
            eta_powers,
            log_k,
            log_t,
            log_k_chunk,
            bytecode_col_vars,
            bytecode_row_vars,
            main_col_vars,
            main_row_vars,
            cycle_phase_row_rounds,
            cycle_phase_col_rounds,
            r_bc,
            lane_weights,
        }
    }

    pub fn num_address_phase_rounds(&self) -> usize {
        (self.bytecode_col_vars + self.bytecode_row_vars)
            - (self.cycle_phase_col_rounds.len() + self.cycle_phase_row_rounds.len())
    }
}

impl<F: JoltField> sealed::Sealed for BytecodeClaimReductionParams<F> {}

impl<F: JoltField> PreCommittedSumcheckInstanceParams<F> for BytecodeClaimReductionParams<F> {
    fn precommitted_input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        match self.reduction.phase {
            BytecodeReductionPhase::CycleVariables => (0..NUM_VAL_STAGES)
                .map(|stage| {
                    let (_, val_claim) = accumulator.get_virtual_polynomial_opening(
                        VirtualPolynomial::BytecodeValStage(stage),
                        SumcheckId::BytecodeReadRafAddressPhase,
                    );
                    self.eta_powers[stage] * val_claim
                })
                .sum(),
            BytecodeReductionPhase::AddressVariables => {
                accumulator
                    .get_virtual_polynomial_opening(
                        VirtualPolynomial::BytecodeClaimReductionIntermediate,
                        SumcheckId::BytecodeClaimReductionCyclePhase,
                    )
                    .1
            }
        }
    }

    fn precommitted_degree(&self) -> usize {
        DEGREE_BOUND
    }
}

impl<F: JoltField> PreCommittedClaimReductionParams<F> for BytecodeClaimReductionParams<F> {
    crate::zkvm::claim_reductions::precommitted::impl_standard_precommitted_claim_reduction_params!(
        BytecodeReductionPhase,
        BytecodeReductionPhase::CycleVariables,
        this => this.bytecode_col_vars + this.bytecode_row_vars
    );
}

#[derive(Allocative)]
pub struct BytecodeClaimReductionProver<F: JoltField> {
    pub params: BytecodeClaimReductionParams<F>,
    value_poly: MultilinearPolynomial<F>,
    eq_poly: MultilinearPolynomial<F>,
    scale: F,
}

impl<F: JoltField> sealed::Sealed for BytecodeClaimReductionProver<F> {}

impl<F: JoltField> BytecodeClaimReductionProver<F> {
    pub fn initialize(params: BytecodeClaimReductionParams<F>, program: Arc<ProgramPreprocessing>) -> Self {
        let raw_value_poly =
            build_committed_bytecode_polynomial_from_instructions::<F>(&program.instructions);
        let (value_poly, eq_poly) = build_permuted_value_and_eq_polys(&params, &raw_value_poly);

        Self {
            params,
            value_poly,
            eq_poly,
            scale: F::one(),
        }
    }
}

impl<F: JoltField> PreCommittedPolyClaimReduction<F> for BytecodeClaimReductionProver<F> {
    type Params = BytecodeClaimReductionParams<F>;

    fn params(&self) -> &Self::Params {
        &self.params
    }

    fn params_mut(&mut self) -> &mut Self::Params {
        &mut self.params
    }

    fn value_poly(&self) -> &MultilinearPolynomial<F> {
        &self.value_poly
    }

    fn value_poly_mut(&mut self) -> &mut MultilinearPolynomial<F> {
        &mut self.value_poly
    }

    fn eq_poly(&self) -> &MultilinearPolynomial<F> {
        &self.eq_poly
    }

    fn eq_poly_mut(&mut self) -> &mut MultilinearPolynomial<F> {
        &mut self.eq_poly
    }

    fn scale(&self) -> &F {
        &self.scale
    }

    fn scale_mut(&mut self) -> &mut F {
        &mut self.scale
    }
}

impl<F: JoltField, T: Transcript> PreCommittedSumcheckInstanceProver<F, T>
    for BytecodeClaimReductionProver<F>
{
    fn precommitted_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    fn precommitted_cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let opening_point = self.params.normalize_opening_point(sumcheck_challenges);

        if self.params.reduction.phase == BytecodeReductionPhase::CycleVariables {
            let c_mid = <Self as PreCommittedPolyClaimReduction<F>>::cycle_intermediate_claim(self);
            accumulator.append_virtual(
                transcript,
                VirtualPolynomial::BytecodeClaimReductionIntermediate,
                SumcheckId::BytecodeClaimReductionCyclePhase,
                opening_point.clone(),
                c_mid,
            );
        }

        if let Some(bytecode_claim) =
            <Self as PreCommittedPolyClaimReduction<F>>::final_claim_if_ready(self)
        {
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::Bytecode,
                SumcheckId::BytecodeClaimReduction,
                opening_point.r,
                bytecode_claim,
            );
        }
    }

    #[cfg(feature = "allocative")]
    fn precommitted_update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
        flamegraph.visit_root(self);
    }
}

pub struct BytecodeClaimReductionVerifier<F: JoltField> {
    pub params: RefCell<BytecodeClaimReductionParams<F>>,
    eq_poly: MultilinearPolynomial<F>,
}

impl<F: JoltField> BytecodeClaimReductionVerifier<F> {
    pub fn new(params: BytecodeClaimReductionParams<F>) -> Self {
        let eq_poly = build_permuted_eq_poly(&params);
        Self {
            params: RefCell::new(params),
            eq_poly,
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceVerifier<F, T>
    for BytecodeClaimReductionVerifier<F>
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
            BytecodeReductionPhase::CycleVariables => {
                accumulator
                    .get_virtual_polynomial_opening(
                        VirtualPolynomial::BytecodeClaimReductionIntermediate,
                        SumcheckId::BytecodeClaimReductionCyclePhase,
                    )
                    .1
            }
            BytecodeReductionPhase::AddressVariables => {
                let (_, bytecode_opening) = accumulator.get_committed_polynomial_opening(
                    CommittedPolynomial::Bytecode,
                    SumcheckId::BytecodeClaimReduction,
                );
                // Sumcheck binding order is always:
                //   1) cycle-phase variables, then
                //   2) address-phase variables.
                // This differs from AddressMajor opening-point serialization.
                let mut binding_challenges = params.reduction.cycle_var_challenges.clone();
                binding_challenges.extend_from_slice(sumcheck_challenges);
                let mut eq_poly = self.eq_poly.clone();
                for r_j in binding_challenges.iter() {
                    eq_poly.bind_parallel(*r_j, BindingOrder::LowToHigh);
                }
                let eq_eval = eq_poly.final_sumcheck_claim();

                let gap_len = internal_dummy_gap_len(
                    &params.cycle_phase_col_rounds,
                    &params.cycle_phase_row_rounds,
                );
                let two_inv = F::from_u64(2).inverse().unwrap();
                let scale = (0..gap_len).fold(F::one(), |acc, _| acc * two_inv);

                bytecode_opening * eq_eval * scale
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
        if params.reduction.phase == BytecodeReductionPhase::CycleVariables {
            let opening_point = params.normalize_opening_point(sumcheck_challenges);
            accumulator.append_virtual(
                transcript,
                VirtualPolynomial::BytecodeClaimReductionIntermediate,
                SumcheckId::BytecodeClaimReductionCyclePhase,
                opening_point.clone(),
            );
            params
                .reduction
                .set_cycle_challenges_from_opening_point(&opening_point);
        }

        if params.num_address_phase_rounds() == 0
            || params.reduction.phase == BytecodeReductionPhase::AddressVariables
        {
            let opening_point = params.normalize_opening_point(sumcheck_challenges);
            accumulator.append_dense(
                transcript,
                CommittedPolynomial::Bytecode,
                SumcheckId::BytecodeClaimReduction,
                opening_point.r,
            );
        }
    }
}

fn build_permuted_value_and_eq_polys<F: JoltField>(
    params: &BytecodeClaimReductionParams<F>,
    raw_value_poly: &MultilinearPolynomial<F>,
) -> (MultilinearPolynomial<F>, MultilinearPolynomial<F>) {
    let eq_cycle = EqPolynomial::<F>::evals(&params.r_bc.r);
    let mut indexed: Vec<(usize, (F, F))> = (0..raw_value_poly.len())
        .map(|idx| {
            let (lane, cycle) = native_index_to_lane_cycle(params, idx);
            let eq = params.lane_weights[lane] * eq_cycle[cycle];
            (idx, (raw_value_poly.get_coeff(idx), eq))
        })
        .collect();

    indexed.par_sort_by(|(a, _), (b, _)| {
        let (addr_a, cycle_a) = index_to_main_address_cycle(params, *a);
        let (addr_b, cycle_b) = index_to_main_address_cycle(params, *b);
        match addr_a.cmp(&addr_b) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => cycle_a.cmp(&cycle_b),
        }
    });

    let (value_coeffs, eq_coeffs): (Vec<F>, Vec<F>) = indexed.into_iter().map(|(_, p)| p).unzip();
    (value_coeffs.into(), eq_coeffs.into())
}

fn build_permuted_eq_poly<F: JoltField>(
    params: &BytecodeClaimReductionParams<F>,
) -> MultilinearPolynomial<F> {
    let eq_cycle = EqPolynomial::<F>::evals(&params.r_bc.r);
    let total_size = 1usize << (params.bytecode_col_vars + params.bytecode_row_vars);
    let mut indexed: Vec<(usize, F)> = (0..total_size)
        .map(|idx| {
            let (lane, cycle) = native_index_to_lane_cycle(params, idx);
            (idx, params.lane_weights[lane] * eq_cycle[cycle])
        })
        .collect();

    indexed.par_sort_by(|(a, _), (b, _)| {
        let (addr_a, cycle_a) = index_to_main_address_cycle(params, *a);
        let (addr_b, cycle_b) = index_to_main_address_cycle(params, *b);
        match addr_a.cmp(&addr_b) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => cycle_a.cmp(&cycle_b),
        }
    });

    let eq_coeffs: Vec<F> = indexed.into_iter().map(|(_, e)| e).collect();
    eq_coeffs.into()
}

#[inline(always)]
fn native_index_to_lane_cycle<F: JoltField>(
    params: &BytecodeClaimReductionParams<F>,
    index: usize,
) -> (usize, usize) {
    let bytecode_len = 1usize << params.log_k;
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => (index / bytecode_len, index % bytecode_len),
        DoryLayout::AddressMajor => (index % committed_lanes(), index / committed_lanes()),
    }
}

#[inline(always)]
fn index_to_main_address_cycle<F: JoltField>(
    params: &BytecodeClaimReductionParams<F>,
    index: usize,
) -> (usize, usize) {
    let bytecode_cols = 1usize << params.bytecode_col_vars;
    let row = index / bytecode_cols;
    let col = index % bytecode_cols;
    let main_cols = 1usize << params.main_col_vars;
    let global_index = row * main_cols + col;
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => {
            let address = global_index / (1usize << params.log_t);
            let cycle = global_index % (1usize << params.log_t);
            (address, cycle)
        }
        DoryLayout::AddressMajor => {
            let address = global_index % (1usize << params.log_k_chunk);
            let cycle = global_index / (1usize << params.log_k_chunk);
            (address, cycle)
        }
    }
}

fn compute_lane_weights<F: JoltField>(
    bytecode_read_raf_params: &BytecodeReadRafSumcheckParams<F>,
    accumulator: &dyn OpeningAccumulator<F>,
    eta_powers: &[F; NUM_VAL_STAGES],
) -> Vec<F> {
    let reg_count = REGISTER_COUNT as usize;
    let total = crate::zkvm::bytecode::chunks::total_lanes();

    let rs1_start = 0usize;
    let rs2_start = rs1_start + reg_count;
    let rd_start = rs2_start + reg_count;
    let unexp_pc_idx = rd_start + reg_count;
    let imm_idx = unexp_pc_idx + 1;
    let circuit_start = imm_idx + 1;
    let instr_start = circuit_start + NUM_CIRCUIT_FLAGS;
    let lookup_start = instr_start + NUM_INSTRUCTION_FLAGS;
    let raf_flag_idx = lookup_start + LookupTables::<XLEN>::COUNT;
    debug_assert_eq!(raf_flag_idx + 1, total);

    let log_reg = reg_count.log_2();
    let r_register_4 = accumulator
        .get_virtual_polynomial_opening(
            VirtualPolynomial::RdWa,
            SumcheckId::RegistersReadWriteChecking,
        )
        .0
        .r;
    let eq_r_register_4 = EqPolynomial::<F>::evals(&r_register_4[..log_reg]);

    let r_register_5 = accumulator
        .get_virtual_polynomial_opening(VirtualPolynomial::RdWa, SumcheckId::RegistersValEvaluation)
        .0
        .r;
    let eq_r_register_5 = EqPolynomial::<F>::evals(&r_register_5[..log_reg]);

    let mut weights = vec![F::zero(); committed_lanes()];

    {
        let coeff = eta_powers[0];
        let g = &bytecode_read_raf_params.stage1_gammas;
        weights[unexp_pc_idx] += coeff * g[0];
        weights[imm_idx] += coeff * g[1];
        for i in 0..NUM_CIRCUIT_FLAGS {
            weights[circuit_start + i] += coeff * g[2 + i];
        }
    }
    {
        let coeff = eta_powers[1];
        let g = &bytecode_read_raf_params.stage2_gammas;
        weights[circuit_start + (CircuitFlags::Jump as usize)] += coeff * g[0];
        weights[instr_start + (InstructionFlags::Branch as usize)] += coeff * g[1];
        weights[instr_start + (InstructionFlags::IsRdNotZero as usize)] += coeff * g[2];
        weights[circuit_start + (CircuitFlags::WriteLookupOutputToRD as usize)] += coeff * g[3];
    }
    {
        let coeff = eta_powers[2];
        let g = &bytecode_read_raf_params.stage3_gammas;
        weights[imm_idx] += coeff * g[0];
        weights[unexp_pc_idx] += coeff * g[1];
        weights[instr_start + (InstructionFlags::LeftOperandIsRs1Value as usize)] += coeff * g[2];
        weights[instr_start + (InstructionFlags::LeftOperandIsPC as usize)] += coeff * g[3];
        weights[instr_start + (InstructionFlags::RightOperandIsRs2Value as usize)] += coeff * g[4];
        weights[instr_start + (InstructionFlags::RightOperandIsImm as usize)] += coeff * g[5];
        weights[instr_start + (InstructionFlags::IsNoop as usize)] += coeff * g[6];
        weights[circuit_start + (CircuitFlags::VirtualInstruction as usize)] += coeff * g[7];
        weights[circuit_start + (CircuitFlags::IsFirstInSequence as usize)] += coeff * g[8];
    }
    {
        let coeff = eta_powers[3];
        let g = &bytecode_read_raf_params.stage4_gammas;
        for r in 0..reg_count {
            weights[rd_start + r] += coeff * g[0] * eq_r_register_4[r];
            weights[rs1_start + r] += coeff * g[1] * eq_r_register_4[r];
            weights[rs2_start + r] += coeff * g[2] * eq_r_register_4[r];
        }
    }
    {
        let coeff = eta_powers[4];
        let g = &bytecode_read_raf_params.stage5_gammas;
        for r in 0..reg_count {
            weights[rd_start + r] += coeff * g[0] * eq_r_register_5[r];
        }
        weights[raf_flag_idx] += coeff * g[1];
        for i in 0..LookupTables::<XLEN>::COUNT {
            weights[lookup_start + i] += coeff * g[2 + i];
        }
    }

    weights
}
