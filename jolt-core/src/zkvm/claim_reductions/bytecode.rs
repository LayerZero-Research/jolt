//! Two-phase bytecode claim reduction (Stage 6b cycle -> Stage 7 address).

use std::cell::RefCell;
use std::ops::Range;

use allocative::Allocative;
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
use crate::zkvm::bytecode::chunks::committed_lanes;
use crate::zkvm::bytecode::read_raf_checking::BytecodeReadRafSumcheckParams;
use crate::zkvm::instruction::{
    CircuitFlags, InstructionFlags, NUM_CIRCUIT_FLAGS, NUM_INSTRUCTION_FLAGS,
};
use crate::zkvm::lookup_table::LookupTables;
use crate::zkvm::witness::{CommittedPolynomial, VirtualPolynomial};
use common::constants::{REGISTER_COUNT, XLEN};
use strum::EnumCount;

const DEGREE_BOUND: usize = 2;
const NUM_VAL_STAGES: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Allocative)]
pub enum PreCommitted {
    CycleVariables,
    AddressVariables,
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

#[derive(Clone, Allocative)]
pub struct BytecodeClaimReductionParams<F: JoltField> {
    pub phase: PreCommitted,
    pub cycle_var_challenges: Vec<F::Challenge>,
    pub eta: F,
    pub eta_powers: [F; NUM_VAL_STAGES],
    /// Eq weights over high bytecode address bits (one per committed chunk).
    pub chunk_rbc_weights: Vec<F>,
    pub bytecode_T: usize,
    pub main_log_k: usize,
    pub main_log_t: usize,
    pub log_t: usize,
    pub log_k_chunk: usize,
    /// Number of initial cycle rounds that must follow IncClaimReduction ordering.
    pub dense_cycle_prefix_vars: usize,
    pub bytecode_chunk_count: usize,
    pub bytecode_col_vars: usize,
    pub bytecode_row_vars: usize,
    pub joint_col_vars: usize,
    #[allocative(skip)]
    pub cycle_phase_row_rounds: Range<usize>,
    #[allocative(skip)]
    pub cycle_phase_col_rounds: Range<usize>,
    pub r_bc: OpeningPoint<BIG_ENDIAN, F>,
    pub lane_weights: Vec<F>,
}

impl<F: JoltField> BytecodeClaimReductionParams<F> {
    #[inline(always)]
    fn bytecode_exceeds_main_domain(&self) -> bool {
        self.total_poly_vars() > (self.main_log_t + self.main_log_k)
    }

    pub fn new(
        bytecode_read_raf_params: &BytecodeReadRafSumcheckParams<F>,
        main_log_t: usize,
        main_log_k: usize,
        dense_cycle_prefix_vars: usize,
        bytecode_chunk_count: usize,
        accumulator: &dyn OpeningAccumulator<F>,
        transcript: &mut impl Transcript,
    ) -> Self {
        let bytecode_t_full = bytecode_read_raf_params.bytecode_T;
        let full_bytecode_len = 1usize << bytecode_t_full;
        assert!(
            full_bytecode_len.is_multiple_of(bytecode_chunk_count),
            "bytecode chunk count ({bytecode_chunk_count}) must divide bytecode_len ({full_bytecode_len})"
        );
        let bytecode_t = (full_bytecode_len / bytecode_chunk_count).log_2();

        let eta: F = transcript.challenge_scalar();
        let mut eta_powers = [F::one(); NUM_VAL_STAGES];
        for i in 1..NUM_VAL_STAGES {
            eta_powers[i] = eta_powers[i - 1] * eta;
        }

        let (r_bc_full, _) = accumulator.get_virtual_polynomial_opening(
            VirtualPolynomial::BytecodeReadRafAddrClaim,
            SumcheckId::BytecodeReadRafAddressPhase,
        );
        debug_assert_eq!(r_bc_full.r.len(), bytecode_t_full);
        let dropped_bits = bytecode_t_full - bytecode_t;
        let chunk_rbc_weights = if dropped_bits == 0 {
            vec![F::one()]
        } else {
            EqPolynomial::<F>::evals(&r_bc_full.r[..dropped_bits])
        };
        debug_assert_eq!(chunk_rbc_weights.len(), bytecode_chunk_count);
        let r_bc = OpeningPoint::new(r_bc_full.r[dropped_bits..].to_vec());

        let lane_weights = compute_lane_weights(bytecode_read_raf_params, accumulator, &eta_powers);

        // bytecode_K is the committed lane capacity (already next-power-of-two padded).
        let bytecode_k = committed_lanes();
        let total_vars = bytecode_k.log_2() + bytecode_t;
        // Bytecode uses its own balanced dimensions (independent from Main).
        // In Stage 8 it is embedded as a top-left block in Joint.
        let (bytecode_col_vars, bytecode_row_vars) = DoryGlobals::balanced_sigma_nu(total_vars);
        // When bytecode is wider than the Main opening domain, anchor Stage-6 cycle alignment
        // on bytecode itself and leave Stage-7 at the shared address window.
        let cycle_alignment_rounds = if total_vars > (main_log_t + main_log_k) {
            total_vars.saturating_sub(main_log_k)
        } else {
            main_log_t
        };
        // Align pre-committed scheduling/permutation to Joint geometry and ensure width can embed
        // this bytecode block without column aliasing.
        let joint_col_vars = std::cmp::max(
            DoryGlobals::balanced_sigma_nu(cycle_alignment_rounds + main_log_k).0,
            bytecode_col_vars,
        );
        let (cycle_phase_col_rounds, cycle_phase_row_rounds) = cycle_phase_round_schedule(
            cycle_alignment_rounds,
            main_log_k,
            joint_col_vars,
            bytecode_row_vars,
            bytecode_col_vars,
        );

        Self {
            phase: PreCommitted::CycleVariables,
            cycle_var_challenges: vec![],
            eta,
            eta_powers,
            chunk_rbc_weights,
            bytecode_T: bytecode_t,
            main_log_k,
            main_log_t,
            log_t: main_log_t,
            log_k_chunk: main_log_k,
            dense_cycle_prefix_vars: dense_cycle_prefix_vars.min(cycle_alignment_rounds),
            bytecode_chunk_count,
            bytecode_col_vars,
            bytecode_row_vars,
            joint_col_vars,
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

impl<F: JoltField> BytecodeClaimReductionParams<F> {
    fn total_poly_vars(&self) -> usize {
        self.bytecode_col_vars + self.bytecode_row_vars
    }

    fn is_cycle_phase(&self) -> bool {
        self.phase == PreCommitted::CycleVariables
    }

    fn cycle_alignment_rounds(&self) -> usize {
        if self.bytecode_exceeds_main_domain() {
            self.total_poly_vars().saturating_sub(self.main_log_k)
        } else {
            self.main_log_t
        }
    }

    fn address_alignment_rounds(&self) -> usize {
        self.main_log_k
    }

    pub fn transition_to_address_phase(&mut self) {
        self.phase = PreCommitted::AddressVariables;
    }

    fn num_rounds_for_current_phase(&self) -> usize {
        if self.phase == PreCommitted::CycleVariables {
            if !self.cycle_phase_row_rounds.is_empty() {
                self.cycle_phase_row_rounds.end - self.cycle_phase_col_rounds.start
            } else {
                self.cycle_phase_col_rounds.len()
            }
        } else {
            let first_phase_rounds =
                self.cycle_phase_col_rounds.len() + self.cycle_phase_row_rounds.len();
            self.total_poly_vars() - first_phase_rounds
        }
    }

    pub fn round_offset(&self, max_num_rounds: usize) -> usize {
        if self.is_cycle_phase() {
            max_num_rounds.saturating_sub(self.cycle_alignment_rounds())
        } else {
            max_num_rounds.saturating_sub(self.num_rounds_for_current_phase())
        }
    }
}

impl<F: JoltField> SumcheckInstanceParams<F> for BytecodeClaimReductionParams<F> {
    fn input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        match self.phase {
            PreCommitted::CycleVariables => (0..NUM_VAL_STAGES)
                .map(|stage| {
                    let (_, val_claim) = accumulator.get_virtual_polynomial_opening(
                        VirtualPolynomial::BytecodeValStage(stage),
                        SumcheckId::BytecodeReadRafAddressPhase,
                    );
                    self.eta_powers[stage] * val_claim
                })
                .sum(),
            PreCommitted::AddressVariables => {
                accumulator
                    .get_virtual_polynomial_opening(
                        VirtualPolynomial::BytecodeClaimReductionIntermediate,
                        SumcheckId::BytecodeClaimReductionCyclePhase,
                    )
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
        if self.phase == PreCommitted::CycleVariables {
            let compact_offset = self.cycle_phase_col_rounds.start;
            let compact_col_rounds = 0..self.cycle_phase_col_rounds.len();
            let compact_row_rounds = self
                .cycle_phase_row_rounds
                .start
                .saturating_sub(compact_offset)
                ..self
                    .cycle_phase_row_rounds
                    .end
                    .saturating_sub(compact_offset);
            let mut cycle_var_challenges: Vec<F::Challenge> = Vec::with_capacity(
                self.cycle_phase_col_rounds.len() + self.cycle_phase_row_rounds.len(),
            );
            cycle_var_challenges.extend_from_slice(&challenges[compact_col_rounds]);
            if !self.cycle_phase_row_rounds.is_empty() {
                cycle_var_challenges.extend_from_slice(&challenges[compact_row_rounds]);
            }
            return OpeningPoint::<LITTLE_ENDIAN, F>::new(cycle_var_challenges).match_endianness();
        }

        match DoryGlobals::get_layout() {
            DoryLayout::CycleMajor => {
                // Address phase sumcheck binds `[cycle-vars || address-vars]` in LE round order.
                // After BE conversion this is `[rev(address) || rev(cycle)]`.
                // For Stage-8 embedding when bytecode dominates, we want:
                //   [rev(stage6 bytecode-only prefix) || rev(stage7 address) || rev(stage6 dense-suffix)].
                // With front-loaded batching in Stage-6b, non-bytecode instances are active in the
                // last `d` rounds, so BE order from normalized bytecode point is:
                //   [stage7 || dense-suffix || bytecode-prefix].
                // Reorder to:
                //   [bytecode-prefix || stage7 || dense-suffix].
                let opening_point_be: OpeningPoint<BIG_ENDIAN, F> =
                    OpeningPoint::<LITTLE_ENDIAN, F>::new(
                        [self.cycle_var_challenges.as_slice(), challenges].concat(),
                    )
                    .match_endianness();

                let total_rounds = opening_point_be.r.len();
                let stage7_rounds = self.address_alignment_rounds().min(total_rounds);
                let dense_rounds =
                    self.dense_cycle_prefix_vars.min(total_rounds.saturating_sub(stage7_rounds));
                let bytecode_prefix_rounds =
                    total_rounds.saturating_sub(stage7_rounds + dense_rounds);

                let stage7_start = 0;
                let stage7_end = stage7_start + stage7_rounds;
                let dense_start = stage7_end;
                let dense_end = dense_start + dense_rounds;
                let bytecode_prefix_start = dense_end;
                let bytecode_prefix_end = bytecode_prefix_start + bytecode_prefix_rounds;
                debug_assert_eq!(bytecode_prefix_end, total_rounds);

                let mut reordered = Vec::with_capacity(total_rounds);
                reordered.extend_from_slice(
                    &opening_point_be.r[bytecode_prefix_start..bytecode_prefix_end],
                );
                reordered.extend_from_slice(&opening_point_be.r[stage7_start..stage7_end]);
                // Keep the Stage-6b dense opening as a contiguous suffix so Stage-8
                // embedding places the full IncClaimReduction point at the end.
                reordered.extend_from_slice(&opening_point_be.r[dense_start..dense_end]);
                OpeningPoint::<BIG_ENDIAN, F>::new(reordered)
            }
            DoryLayout::AddressMajor => {
                // In AddressMajor, construct the Stage-8 anchor directly from Stage-6b/Stage-7
                // challenge vectors:
                //   [rev(last T vars of stage6b) || rev(stage7 vars) || rev(first b vars of stage6b)].
                // where:
                //   - stage6b vars are `cycle_var_challenges` in LE round order,
                //   - T = dense_cycle_prefix_vars (main dense width),
                //   - b = remaining bytecode-only Stage-6b vars.
                let stage6_rounds = self.cycle_var_challenges.len();
                let dense_rounds = self.dense_cycle_prefix_vars.min(stage6_rounds);
                let bytecode_prefix_rounds = stage6_rounds.saturating_sub(dense_rounds);

                let stage6_head = &self.cycle_var_challenges[..bytecode_prefix_rounds];
                let stage6_tail = &self.cycle_var_challenges[bytecode_prefix_rounds..];

                let mut reordered =
                    Vec::with_capacity(stage6_rounds + challenges.len());
                reordered.extend(stage6_tail.iter().rev().cloned());
                reordered.extend(challenges.iter().rev().cloned());
                reordered.extend(stage6_head.iter().rev().cloned());

                OpeningPoint::<BIG_ENDIAN, F>::new(reordered)
            }
        }
    }
}

#[derive(Allocative)]
pub struct BytecodeClaimReductionProver<F: JoltField> {
    params: BytecodeClaimReductionParams<F>,
    value_poly: MultilinearPolynomial<F>,
    eq_poly: MultilinearPolynomial<F>,
    scale: F,
    chunk_value_polys: Vec<MultilinearPolynomial<F>>,
}

impl<F: JoltField> BytecodeClaimReductionProver<F> {
    pub fn params(&self) -> &BytecodeClaimReductionParams<F> {
        &self.params
    }

    pub fn transition_to_address_phase(&mut self) {
        self.params.transition_to_address_phase();
    }

    pub fn initialize(
        params: BytecodeClaimReductionParams<F>,
        raw_chunk_polys: &[MultilinearPolynomial<F>],
    ) -> Self {
        let raw_value_coeffs: Vec<F> = (0..raw_chunk_polys[0].len())
            .into_par_iter()
            .map(|idx| {
                raw_chunk_polys
                    .iter()
                    .zip(params.chunk_rbc_weights.iter())
                    .map(|(poly, weight)| poly.get_coeff(idx) * *weight)
                    .sum::<F>()
            })
            .collect();
        let raw_value_poly: MultilinearPolynomial<F> = raw_value_coeffs.into();
        let (value_poly, eq_poly) = build_permuted_value_and_eq_polys(&params, &raw_value_poly);
        let chunk_value_polys = raw_chunk_polys
            .par_iter()
            .map(|raw_chunk_poly| build_permuted_value_and_eq_polys(&params, raw_chunk_poly).0)
            .collect();

        Self {
            params,
            value_poly,
            eq_poly,
            scale: F::one(),
            chunk_value_polys,
        }
    }

    fn bind_aux_polys(&mut self, r_j: F::Challenge) {
        for poly in self.chunk_value_polys.iter_mut() {
            poly.bind_parallel(r_j, BindingOrder::LowToHigh);
        }
    }

    fn compute_message_unscaled(&self, previous_claim_unscaled: F) -> UniPoly<F> {
        let half = self.value_poly.len() / 2;
        let evals: [F; DEGREE_BOUND] = (0..half)
            .into_par_iter()
            .map(|j| {
                let value_evals = self
                    .value_poly
                    .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                let eq_evals = self
                    .eq_poly
                    .sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                let mut out = [F::zero(); DEGREE_BOUND];
                for i in 0..DEGREE_BOUND {
                    out[i] = value_evals[i] * eq_evals[i];
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
        let len = self.value_poly.len();
        debug_assert_eq!(len, self.eq_poly.len());
        let mut sum = F::zero();
        for i in 0..len {
            sum += self.value_poly.get_bound_coeff(i) * self.eq_poly.get_bound_coeff(i);
        }
        sum * self.scale
    }

    fn final_claim_if_ready(&self) -> Option<F> {
        if self.value_poly.len() == 1 {
            Some(self.value_poly.final_sumcheck_claim())
        } else {
            None
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T> for BytecodeClaimReductionProver<F> {
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
        tracing::info!("BYTECODE: ingest_challenge round={} r_j={:?}, phase={:?}", round, r_j, self.params.phase);
        if self.params.is_cycle_phase() {
            let is_dummy_round = !self.params.cycle_phase_col_rounds.contains(&round)
                && !self.params.cycle_phase_row_rounds.contains(&round);
            if is_dummy_round {
                self.scale *= F::from_u64(2).inverse().unwrap();
            } else {
                self.value_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
                self.eq_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
                self.bind_aux_polys(r_j);
                self.params.cycle_var_challenges.push(r_j);
            }
            return;
        }
        self.value_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
        self.eq_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
        self.bind_aux_polys(r_j);
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
            accumulator.append_virtual(
                transcript,
                VirtualPolynomial::BytecodeClaimReductionIntermediate,
                SumcheckId::BytecodeClaimReductionCyclePhase,
                opening_point.clone(),
                c_mid,
            );
        }

        if let Some(bytecode_claim) = self.final_claim_if_ready() {
            let chunk_claims: Vec<F> = self
                .chunk_value_polys
                .iter()
                .map(|poly| poly.final_sumcheck_claim())
                .collect();
            let weighted_chunk_sum = chunk_claims
                .iter()
                .zip(params.chunk_rbc_weights.iter())
                .map(|(claim, weight)| *claim * *weight)
                .sum::<F>();
            debug_assert_eq!(weighted_chunk_sum, bytecode_claim);
            for (chunk_idx, claim) in chunk_claims.into_iter().enumerate() {
                accumulator.append_dense(
                    transcript,
                    CommittedPolynomial::BytecodeChunk(chunk_idx),
                    SumcheckId::BytecodeClaimReduction,
                    opening_point.r.clone(),
                    claim,
                );
            }
        }
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
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
        match params.phase {
            PreCommitted::CycleVariables => {
                accumulator
                    .get_virtual_polynomial_opening(
                        VirtualPolynomial::BytecodeClaimReductionIntermediate,
                        SumcheckId::BytecodeClaimReductionCyclePhase,
                    )
                    .1
            }
            PreCommitted::AddressVariables => {
                let bytecode_opening: F = (0..params.bytecode_chunk_count)
                    .map(|chunk_idx| {
                        params.chunk_rbc_weights[chunk_idx]
                            * accumulator
                                .get_committed_polynomial_opening(
                                    CommittedPolynomial::BytecodeChunk(chunk_idx),
                                    SumcheckId::BytecodeClaimReduction,
                                )
                                .1
                    })
                    .sum();
                // Sumcheck binding order is always cycle-phase variables first,
                // then address-phase variables (independent of opening-point serialization).
                let mut binding_challenges = params.cycle_var_challenges.clone();
                binding_challenges.extend_from_slice(sumcheck_challenges);
                let mut eq_poly = self.eq_poly.clone();
                for r_j in binding_challenges.iter() {
                    eq_poly.bind_parallel(*r_j, BindingOrder::LowToHigh);
                }
                let eq_eval = eq_poly.final_sumcheck_claim();

                let cycle_phase_total_rounds = if !params.cycle_phase_row_rounds.is_empty() {
                    params.cycle_phase_row_rounds.end - params.cycle_phase_col_rounds.start
                } else {
                    params.cycle_phase_col_rounds.len()
                };
                let gap_len = cycle_phase_total_rounds
                    - (params.cycle_phase_col_rounds.len() + params.cycle_phase_row_rounds.len());
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
        if params.phase == PreCommitted::CycleVariables {
            let opening_point = params.normalize_opening_point(sumcheck_challenges);
            accumulator.append_virtual(
                transcript,
                VirtualPolynomial::BytecodeClaimReductionIntermediate,
                SumcheckId::BytecodeClaimReductionCyclePhase,
                opening_point.clone(),
            );
            let opening_point_le: OpeningPoint<LITTLE_ENDIAN, F> = opening_point.match_endianness();
            params.cycle_var_challenges = opening_point_le.r;
        }

        if params.num_address_phase_rounds() == 0
            || params.phase == PreCommitted::AddressVariables
        {
            let opening_point = params.normalize_opening_point(sumcheck_challenges);
            for chunk_idx in 0..params.bytecode_chunk_count {
                accumulator.append_dense(
                    transcript,
                    CommittedPolynomial::BytecodeChunk(chunk_idx),
                    SumcheckId::BytecodeClaimReduction,
                    opening_point.r.clone(),
                );
            }
        }
    }
}

fn build_permuted_value_and_eq_polys<F: JoltField>(
    params: &BytecodeClaimReductionParams<F>,
    raw_value_poly: &MultilinearPolynomial<F>,
) -> (MultilinearPolynomial<F>, MultilinearPolynomial<F>) {
    let eq_cycle = EqPolynomial::<F>::evals(&params.r_bc.r);
    let value_coeffs: Vec<F> = (0..raw_value_poly.len())
        .map(|idx| raw_value_poly.get_coeff(idx))
        .collect();
    let eq_coeffs: Vec<F> = (0..raw_value_poly.len())
        .map(|idx| {
            let (lane, cycle) = native_index_to_lane_cycle(params, idx);
            params.lane_weights[lane] * eq_cycle[cycle]
        })
        .collect();
    let num_vars = params.total_poly_vars();
    if !needs_sumcheck_permutation(params) {
        return (value_coeffs.into(), eq_coeffs.into());
    }

    let mut permuted_values = vec![F::zero(); value_coeffs.len()];
    let mut permuted_eq = vec![F::zero(); eq_coeffs.len()];
    for old_idx in 0..value_coeffs.len() {
        let new_idx = permute_sumcheck_index(params, old_idx, num_vars);
        permuted_values[new_idx] = value_coeffs[old_idx];
        permuted_eq[new_idx] = eq_coeffs[old_idx];
    }
    (permuted_values.into(), permuted_eq.into())
}

fn build_permuted_eq_poly<F: JoltField>(
    params: &BytecodeClaimReductionParams<F>,
) -> MultilinearPolynomial<F> {
    let eq_cycle = EqPolynomial::<F>::evals(&params.r_bc.r);
    let total_size = 1usize << (params.bytecode_col_vars + params.bytecode_row_vars);
    let eq_coeffs: Vec<F> = (0..total_size)
        .map(|idx| {
            let (lane, cycle) = native_index_to_lane_cycle(params, idx);
            params.lane_weights[lane] * eq_cycle[cycle]
        })
        .collect();
    let num_vars = params.total_poly_vars();
    if !needs_sumcheck_permutation(params) {
        return eq_coeffs.into();
    }

    let mut permuted_eq = vec![F::zero(); eq_coeffs.len()];
    for old_idx in 0..eq_coeffs.len() {
        let new_idx = permute_sumcheck_index(params, old_idx, num_vars);
        permuted_eq[new_idx] = eq_coeffs[old_idx];
    }
    permuted_eq.into()
}

#[inline(always)]
fn native_index_to_lane_cycle<F: JoltField>(
    params: &BytecodeClaimReductionParams<F>,
    index: usize,
) -> (usize, usize) {
    let bytecode_len = 1usize << params.bytecode_T;
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => (index / bytecode_len, index % bytecode_len),
        DoryLayout::AddressMajor => (index % committed_lanes(), index / committed_lanes()),
    }
}

#[inline(always)]
fn needs_sumcheck_permutation<F: JoltField>(params: &BytecodeClaimReductionParams<F>) -> bool {
    let num_vars = params.total_poly_vars();
    match DoryGlobals::get_layout() {
        // CycleMajor: explicit block permutation that cancels normalize_opening_point's
        // [k || T || c] -> [c || k || T] opening-point reordering.
        DoryLayout::CycleMajor => {
            let k = params.main_log_k.min(num_vars);
            let t = params.dense_cycle_prefix_vars.min(num_vars.saturating_sub(k));
            let c = num_vars.saturating_sub(k + t);
            c > 0
        }
        // AddressMajor also needs an explicit block permutation to invert the
        // normalization impact on bytecode reduction coordinates.
        DoryLayout::AddressMajor => {
            // Use BE block semantics here (matching opening-point discussions):
            // raw committed chunk coordinates are `[T | k | c]`,
            // while bytecode sumcheck semantics are `[k | T | c]`.
            //
            // So we swap only the top BE blocks `T` and `k`, and keep `c` fixed.
            //
            // (Equivalent LE view: `[c | k | T] -> [c | T | k]`.)
            //
            // where:
            // - `c`: bytecode-only Stage-6 prefix vars,
            // - `k`: Stage-7 address vars,
            // - `T`: shared dense Stage-6 suffix vars.
            let k = params.address_alignment_rounds().min(num_vars);
            let t = params.dense_cycle_prefix_vars.min(num_vars.saturating_sub(k));
            k > 0 && t > 0
        }
    }
}

#[inline(always)]
fn permute_sumcheck_index<F: JoltField>(
    params: &BytecodeClaimReductionParams<F>,
    index: usize,
    num_vars: usize,
) -> usize {
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => {
            // BE block order conversion:
            //   raw:      [c | T | k]
            //   sumcheck: [T | k | c]
            //
            // (Equivalent LE view: `[k | T | c] -> [c | k | T]`.)
            let k = params.main_log_k.min(num_vars);
            let t = params.dense_cycle_prefix_vars.min(num_vars.saturating_sub(k));
            let c = num_vars.saturating_sub(k + t);
            if c == 0 {
                return index;
            }
            debug_assert!(num_vars < usize::BITS as usize);
            let c_mask = if c == 0 { 0 } else { (1usize << c) - 1 };
            let t_mask = if t == 0 { 0 } else { (1usize << t) - 1 };
            let k_mask = if k == 0 { 0 } else { (1usize << k) - 1 };

            // Parse raw `[c | T | k]` in BE block order.
            let c_bits = (index >> (t + k)) & c_mask;
            let t_bits = (index >> k) & t_mask;
            let k_bits = index & k_mask;

            // Emit sumcheck `[T | k | c]` in BE block order.
            (t_bits << (k + c)) | (k_bits << c) | c_bits
        }
        DoryLayout::AddressMajor => {
            // BE block order conversion:
            //   raw:      [T | k | c]
            //   sumcheck: [k | T | c]
            //
            // (Equivalent LE view: `[c | k | T] -> [c | T | k]`.)
            let k = params.address_alignment_rounds().min(num_vars);
            let t = params.dense_cycle_prefix_vars.min(num_vars.saturating_sub(k));
            let c = num_vars.saturating_sub(k + t);

            let t_mask = if t == 0 { 0 } else { (1usize << t) - 1 };
            let k_mask = if k == 0 { 0 } else { (1usize << k) - 1 };
            let c_mask = if c == 0 { 0 } else { (1usize << c) - 1 };

            // Parse raw `[T | k | c]` in BE block order.
            let t_bits = (index >> (k + c)) & t_mask;
            let k_bits = (index >> c) & k_mask;
            let c_bits = index & c_mask;

            // Emit sumcheck `[k | T | c]` in BE block order.
            (k_bits << (t + c)) | (t_bits << c) | c_bits
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
