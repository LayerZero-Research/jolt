use crate::field::JoltField;
use crate::poly::commitment::dory::{DoryGlobals, DoryLayout};
use crate::poly::opening_proof::{OpeningPoint, BIG_ENDIAN, LITTLE_ENDIAN};
use crate::utils::math::Math;
use std::ops::Range;

pub mod advice;
pub mod bytecode;
pub mod hamming_weight;
pub mod increments;
pub mod instruction_lookups;
pub mod program_image;
pub mod ram_ra;
pub mod registers;

pub use advice::{
    AdviceClaimReductionParams, AdviceClaimReductionProver, AdviceClaimReductionVerifier,
    AdviceKind,
};
pub use bytecode::{
    BytecodeClaimReductionParams, BytecodeClaimReductionProver, BytecodeClaimReductionVerifier,
};
pub use hamming_weight::{
    HammingWeightClaimReductionParams, HammingWeightClaimReductionProver,
    HammingWeightClaimReductionVerifier,
};
pub use increments::{
    IncClaimReductionSumcheckParams, IncClaimReductionSumcheckProver,
    IncClaimReductionSumcheckVerifier,
};
pub use instruction_lookups::{
    InstructionLookupsClaimReductionSumcheckParams, InstructionLookupsClaimReductionSumcheckProver,
    InstructionLookupsClaimReductionSumcheckVerifier,
};
pub use program_image::{
    ProgramImageClaimReductionParams, ProgramImageClaimReductionProver,
    ProgramImageClaimReductionVerifier,
};
pub use ram_ra::{
    RaReductionParams, RamRaClaimReductionSumcheckProver, RamRaClaimReductionSumcheckVerifier,
};
pub use registers::{
    RegistersClaimReductionSumcheckParams, RegistersClaimReductionSumcheckProver,
    RegistersClaimReductionSumcheckVerifier,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecommittedGeometry {
    pub main_cycle_rounds: usize,
    pub cycle_rounds: usize,
    pub address_rounds: usize,
    pub joint_col_vars: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwoPhaseRoundSchedule {
    pub cycle_phase_rounds: Vec<usize>,
    pub cycle_phase_col_round_count: usize,
    pub cycle_phase_total_rounds: usize,
    pub address_phase_rounds: Range<usize>,
}

pub struct PrecommittedClaimReduction;

impl PrecommittedClaimReduction {
    /// Geometry needed by two-phase claim reductions that bind cycle first, then address.
    ///
    /// Uses Main-context execution `T` for the native trace cycle segment and
    /// `main_log_embedding` to derive
    /// derive the largest embedding shape used by precommitted reductions.
    #[inline]
    pub fn precommitted_geometry() -> PrecommittedGeometry {
        let main_trace_t = DoryGlobals::get_T();
        let main_k = DoryGlobals::main_k();
        let main_log_embedding = DoryGlobals::get_main_log_embedding();
        debug_assert!(main_trace_t.is_power_of_two() && main_trace_t > 0);
        debug_assert!(main_k.is_power_of_two() && main_k > 0);
        let main_cycle_rounds = main_trace_t.log_2();
        let address_rounds = main_k.log_2();
        let cycle_rounds =
            std::cmp::max(main_cycle_rounds, main_log_embedding.saturating_sub(address_rounds));
        let joint_col_vars = DoryGlobals::balanced_sigma_nu(main_log_embedding).0;
        PrecommittedGeometry {
            main_cycle_rounds,
            cycle_rounds,
            address_rounds,
            joint_col_vars,
        }
    }

    /// AddressMajor column-bit partition for top-left embeddings:
    /// `(cycle-prefix, address, dense-cycle)`.
    #[inline]
    pub fn address_major_precommitted_col_bit_partition(
        poly_col_vars: usize,
    ) -> (usize, usize, usize) {
        let geom = Self::precommitted_geometry();
        let cycle_prefix_cols = geom.cycle_rounds.saturating_sub(geom.main_cycle_rounds);
        let prefix_bits = std::cmp::min(poly_col_vars, cycle_prefix_cols);
        let remaining = poly_col_vars.saturating_sub(prefix_bits);
        let address_bits = std::cmp::min(remaining, geom.address_rounds);
        let dense_bits = remaining.saturating_sub(address_bits);
        (prefix_bits, address_bits, dense_bits)
    }

    /// Shared Stage-6/7 schedule for top-left embedded polynomials in max-embedding context.
    ///
    /// Inputs are only the polynomial dimensions; cycle/address alignment is read from
    /// Main + main_log_embedding.
    pub fn precommitted_two_phase_round_schedule(
        poly_row_vars: usize,
        poly_col_vars: usize,
    ) -> TwoPhaseRoundSchedule {
        let geom = Self::precommitted_geometry();
        let (cycle_phase_col_rounds, cycle_phase_row_rounds) = match DoryGlobals::get_layout() {
            DoryLayout::CycleMajor => {
                let cycle_prefix = geom.cycle_rounds.saturating_sub(geom.main_cycle_rounds);
                let col_len = std::cmp::min(geom.main_cycle_rounds, poly_col_vars);
                let col_start = cycle_prefix;
                let col_end = std::cmp::min(geom.cycle_rounds, col_start + col_len);
                let col_binding_rounds = col_start..col_end;
                let row_start_base = std::cmp::min(
                    geom.main_cycle_rounds,
                    std::cmp::max(
                        std::cmp::min(geom.main_cycle_rounds, geom.joint_col_vars),
                        col_len,
                    ),
                );
                let row_start = std::cmp::min(geom.cycle_rounds, cycle_prefix + row_start_base);
                let row_end = std::cmp::min(geom.cycle_rounds, row_start + poly_row_vars);
                let row_binding_rounds = row_start..row_end;
                (col_binding_rounds, row_binding_rounds)
            }
            DoryLayout::AddressMajor => {
                let (prefix_col_bits, _address_col_bits, dense_col_bits) =
                    Self::address_major_precommitted_col_bit_partition(poly_col_vars);
                let col_end = std::cmp::min(
                    geom.cycle_rounds,
                    prefix_col_bits.saturating_add(dense_col_bits),
                );
                let col_binding_rounds = 0..col_end;
                let row_start_unclamped = geom.joint_col_vars.saturating_sub(geom.address_rounds);
                let row_start = std::cmp::min(
                    geom.cycle_rounds,
                    std::cmp::max(row_start_unclamped, col_end),
                );
                let row_end = std::cmp::min(geom.cycle_rounds, row_start + poly_row_vars);
                let row_binding_rounds = row_start..row_end;
                (col_binding_rounds, row_binding_rounds)
            }
        };
        let cycle_phase_total_rounds = if !cycle_phase_row_rounds.is_empty() {
            cycle_phase_row_rounds.end
        } else {
            cycle_phase_col_rounds.end
        };
        let mut cycle_phase_rounds =
            Vec::with_capacity(cycle_phase_col_rounds.len() + cycle_phase_row_rounds.len());
        cycle_phase_rounds.extend(cycle_phase_col_rounds.clone());
        cycle_phase_rounds.extend(cycle_phase_row_rounds.clone());
        let first_phase_bound_vars = cycle_phase_rounds.len();
        let total_poly_vars = poly_row_vars + poly_col_vars;
        let address_phase_rounds = 0..total_poly_vars.saturating_sub(first_phase_bound_vars);
        TwoPhaseRoundSchedule {
            cycle_phase_rounds,
            cycle_phase_col_round_count: cycle_phase_col_rounds.len(),
            cycle_phase_total_rounds,
            address_phase_rounds,
        }
    }

    #[inline]
    fn precommitted_cycle_phase_total_rounds(schedule: &TwoPhaseRoundSchedule) -> usize {
        schedule.cycle_phase_total_rounds
    }

    #[inline]
    pub fn precommitted_internal_dummy_gap_len(schedule: &TwoPhaseRoundSchedule) -> usize {
        Self::precommitted_cycle_phase_total_rounds(schedule) - schedule.cycle_phase_rounds.len()
    }

    #[inline]
    pub fn precommitted_num_rounds_for_phase(
        is_cycle_phase: bool,
        schedule: &TwoPhaseRoundSchedule,
    ) -> usize {
        if is_cycle_phase {
            Self::precommitted_cycle_phase_total_rounds(schedule)
        } else {
            schedule.address_phase_rounds.len()
        }
    }

    #[inline]
    pub fn precommitted_round_offset(
        is_cycle_phase: bool,
        max_num_rounds: usize,
        cycle_alignment_rounds: usize,
        num_rounds_for_phase: usize,
    ) -> usize {
        if is_cycle_phase {
            max_num_rounds.saturating_sub(cycle_alignment_rounds)
        } else {
            match DoryGlobals::get_layout() {
                DoryLayout::AddressMajor => 0,
                DoryLayout::CycleMajor => max_num_rounds.saturating_sub(num_rounds_for_phase),
            }
        }
    }

    pub fn normalize_precommitted_two_phase_opening_point<F: JoltField>(
        is_cycle_phase: bool,
        cycle_var_challenges: &[F::Challenge],
        schedule: &TwoPhaseRoundSchedule,
        challenges: &[F::Challenge],
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        if is_cycle_phase {
            let local_cycle_challenges: Vec<F::Challenge> = schedule
                .cycle_phase_rounds
                .iter()
                .map(|&idx| challenges[idx])
                .collect();
            return OpeningPoint::<LITTLE_ENDIAN, F>::new(local_cycle_challenges)
                .match_endianness();
        }

        match DoryGlobals::get_layout() {
            DoryLayout::CycleMajor => {
                OpeningPoint::<LITTLE_ENDIAN, F>::new([cycle_var_challenges, challenges].concat())
                    .match_endianness()
            }
            DoryLayout::AddressMajor => {
                let geom = Self::precommitted_geometry();
                let cycle_prefix_cols = geom.cycle_rounds.saturating_sub(geom.main_cycle_rounds);
                let col_cycle_len = schedule.cycle_phase_col_round_count;
                let (col_cycle, row_cycle) = cycle_var_challenges.split_at(col_cycle_len);
                let prefix_len = std::cmp::min(cycle_prefix_cols, col_cycle.len());
                let (prefix_cycle, dense_cycle) = col_cycle.split_at(prefix_len);
                OpeningPoint::<LITTLE_ENDIAN, F>::new(
                    [prefix_cycle, challenges, dense_cycle, row_cycle].concat(),
                )
                .match_endianness()
            }
        }
    }

    #[inline(always)]
    pub fn permute_precommitted_sumcheck_index_address_major(
        index: usize,
        poly_row_vars: usize,
        poly_col_vars: usize,
    ) -> usize {
        let (prefix_bits, address_bits, dense_bits) =
            Self::address_major_precommitted_col_bit_partition(poly_col_vars);
        let row_bits = poly_row_vars;

        let prefix_mask = if prefix_bits == 0 {
            0
        } else {
            (1usize << prefix_bits) - 1
        };
        let address_mask = if address_bits == 0 {
            0
        } else {
            (1usize << address_bits) - 1
        };
        let dense_mask = if dense_bits == 0 {
            0
        } else {
            (1usize << dense_bits) - 1
        };
        let row_mask = if row_bits == 0 {
            0
        } else {
            (1usize << row_bits) - 1
        };

        let prefix = index & prefix_mask;
        let address = (index >> prefix_bits) & address_mask;
        let dense = (index >> (prefix_bits + address_bits)) & dense_mask;
        let row = (index >> (prefix_bits + address_bits + dense_bits)) & row_mask;

        prefix
            | (dense << prefix_bits)
            | (row << (prefix_bits + dense_bits))
            | (address << (prefix_bits + dense_bits + row_bits))
    }

    /// Maps a top-left embedded polynomial index to `(address, cycle)` under CycleMajor max-embedding layout.
    #[inline(always)]
    pub fn cycle_major_top_left_index_to_address_cycle(
        index: usize,
        poly_col_vars: usize,
    ) -> (usize, usize) {
        let geom = Self::precommitted_geometry();
        let joint_cols = 1usize << geom.joint_col_vars;
        let poly_cols = 1usize << poly_col_vars;
        let row = index / poly_cols;
        let col = index % poly_cols;
        let global_index = row as u128 * joint_cols as u128 + col as u128;
        let main_trace_t = 1u128 << geom.main_cycle_rounds;
        let address = global_index / main_trace_t;
        let cycle = global_index % main_trace_t;
        (address as usize, cycle as usize)
    }
}
