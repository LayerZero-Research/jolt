use allocative::Allocative;
use rayon::prelude::*;
use std::ops::Range;

use crate::field::JoltField;
use crate::poly::commitment::dory::{DoryGlobals, DoryLayout};
use crate::poly::multilinear_polynomial::{BindingOrder, MultilinearPolynomial, PolynomialBinding};
use crate::poly::opening_proof::{OpeningPoint, BIG_ENDIAN, LITTLE_ENDIAN};
use crate::poly::unipoly::UniPoly;
use crate::subprotocols::sumcheck_verifier::SumcheckInstanceParams;
use crate::utils::math::Math;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RoundScheduleData {
    cycle_phase_rounds: Vec<usize>,
    cycle_phase_col_round_count: usize,
    cycle_phase_total_rounds: usize,
    address_phase_rounds: Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Allocative)]
pub enum PrecommittedEmbeddingMode {
    DominantPrecommitted,
    EmbeddedPrecommitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Allocative)]
pub struct PrecommittedSchedulingReference {
    pub main_total_vars: usize,
    pub reference_total_vars: usize,
    pub cycle_alignment_rounds: usize,
    pub address_rounds: usize,
    pub joint_col_vars: usize,
}

#[derive(Debug, Clone, Allocative)]
pub struct PrecommittedClaimReduction<F: JoltField> {
    pub scheduling_reference: PrecommittedSchedulingReference,
    pub embedding_mode: PrecommittedEmbeddingMode,
    pub cycle_var_challenges: Vec<F::Challenge>,
    cycle_phase_rounds: Vec<usize>,
    cycle_phase_col_round_count: usize,
    cycle_phase_total_rounds: usize,
    #[allocative(skip)]
    address_phase_rounds: Range<usize>,
}

impl<F: JoltField> PrecommittedClaimReduction<F> {
    /// Compute shared scheduling dimensions from Main and precommitted candidates.
    ///
    /// `reference_total_vars` is the largest total var count across Main and candidates.
    pub fn scheduling_reference(
        main_total_vars: usize,
        candidates: &[usize],
    ) -> PrecommittedSchedulingReference {
        let address_rounds = DoryGlobals::main_k().log_2();
        let max_precommitted = candidates.iter().copied().max().unwrap_or(0);
        let reference_total_vars = std::cmp::max(main_total_vars, max_precommitted);
        let cycle_alignment_rounds = reference_total_vars.saturating_sub(address_rounds);
        let (reference_sigma, _) = DoryGlobals::balanced_sigma_nu(reference_total_vars);
        let joint_col_vars = std::cmp::max(
            DoryGlobals::configured_main_num_columns().log_2(),
            reference_sigma,
        );
        PrecommittedSchedulingReference {
            main_total_vars,
            reference_total_vars,
            cycle_alignment_rounds,
            address_rounds,
            joint_col_vars,
        }
    }

    #[inline]
    pub fn new(
        poly_total_vars: usize,
        poly_row_vars: usize,
        poly_col_vars: usize,
        scheduling_reference: PrecommittedSchedulingReference,
    ) -> Self {
        let (embedding_mode, round_schedule) = Self::embedding_mode_and_round_schedule_for_poly(
            poly_total_vars,
            poly_row_vars,
            poly_col_vars,
            &scheduling_reference,
        );
        Self {
            scheduling_reference,
            embedding_mode,
            cycle_var_challenges: vec![],
            cycle_phase_rounds: round_schedule.cycle_phase_rounds,
            cycle_phase_col_round_count: round_schedule.cycle_phase_col_round_count,
            cycle_phase_total_rounds: round_schedule.cycle_phase_total_rounds,
            address_phase_rounds: round_schedule.address_phase_rounds,
        }
    }

    #[inline]
    fn embedding_mode_and_round_schedule_for_poly(
        poly_total_vars: usize,
        poly_row_vars: usize,
        poly_col_vars: usize,
        reference: &PrecommittedSchedulingReference,
    ) -> (PrecommittedEmbeddingMode, RoundScheduleData) {
        let has_precommitted_dominance = reference.reference_total_vars > reference.main_total_vars;
        let embedding_mode = if has_precommitted_dominance
            && poly_total_vars == reference.reference_total_vars
        {
            PrecommittedEmbeddingMode::DominantPrecommitted
        } else {
            PrecommittedEmbeddingMode::EmbeddedPrecommitted
        };
        if embedding_mode == PrecommittedEmbeddingMode::DominantPrecommitted {
            assert_eq!(poly_total_vars, reference.reference_total_vars);
        }
        let has_dominant_reference =
            has_precommitted_dominance && poly_total_vars > reference.main_total_vars;
        let round_schedule = Self::two_phase_round_schedule_for_mode(
            embedding_mode,
            reference,
            poly_row_vars,
            poly_col_vars,
            has_dominant_reference,
        );
        (embedding_mode, round_schedule)
    }

    /// AddressMajor column-bit partition for top-left embeddings:
    /// `(cycle-prefix, address, dense-cycle)`.
    #[inline]
    pub fn address_major_precommitted_col_bit_partition(
        poly_col_vars: usize,
        cycle_rounds: usize,
        address_rounds: usize,
    ) -> (usize, usize, usize) {
        let main_cycle_rounds = DoryGlobals::main_t().log_2();
        let cycle_prefix_cols = cycle_rounds.saturating_sub(main_cycle_rounds);
        let prefix_bits = std::cmp::min(poly_col_vars, cycle_prefix_cols);
        let remaining = poly_col_vars.saturating_sub(prefix_bits);
        let address_bits = std::cmp::min(remaining, address_rounds);
        let dense_bits = remaining.saturating_sub(address_bits);
        (prefix_bits, address_bits, dense_bits)
    }

    /// Shared Stage-6/7 schedule for top-left embedded polynomials in max-embedding context.
    ///
    /// Inputs are only the polynomial dimensions; cycle/address alignment is read from
    /// Main + main_log_embedding.
    fn precommitted_two_phase_round_schedule(
        cycle_rounds: usize,
        address_rounds: usize,
        joint_col_vars: usize,
        poly_row_vars: usize,
        poly_col_vars: usize,
    ) -> RoundScheduleData {
        let main_cycle_rounds = DoryGlobals::main_t().log_2();
        if DoryGlobals::get_layout() == DoryLayout::CycleMajor {
            // Build cycle/address selection from the exact row/column-tail projection of
            // the full `[c | k | T]` CycleMajor opening point, where `c` are dominant
            // cycle-prefix vars (if any), `k` are Stage-7 address vars, and `T` are native
            // main-cycle vars. This keeps embedded precommitted schedules aligned with a
            // dominant-bytecode anchor.
            let cycle_prefix_vars = cycle_rounds.saturating_sub(main_cycle_rounds);
            let address_vars = address_rounds;
            let total_full_vars = cycle_rounds + address_vars;
            let nu_full = total_full_vars.saturating_sub(joint_col_vars);
            let sigma_full = joint_col_vars;
            let row_start = nu_full.saturating_sub(poly_row_vars);
            let col_start = sigma_full.saturating_sub(poly_col_vars);

            let be_index_to_round = |be_index: usize| -> Option<usize> {
                if be_index < cycle_prefix_vars {
                    // Prefix block is `reverse(cycle_le[..c])`.
                    Some(cycle_prefix_vars.saturating_sub(1).saturating_sub(be_index))
                } else if be_index < cycle_prefix_vars + address_vars {
                    None
                } else {
                    // Dense block is `reverse(cycle_le[c..])`.
                    let dense_be_idx = be_index - (cycle_prefix_vars + address_vars);
                    Some(cycle_rounds.saturating_sub(1).saturating_sub(dense_be_idx))
                }
            };

            let mut col_rounds = Vec::new();
            for col_idx in col_start..sigma_full {
                let be_idx = nu_full + col_idx;
                if let Some(round) = be_index_to_round(be_idx) {
                    col_rounds.push(round);
                }
            }

            let mut row_rounds = Vec::new();
            for row_idx in row_start..nu_full {
                if let Some(round) = be_index_to_round(row_idx) {
                    row_rounds.push(round);
                }
            }

            col_rounds.sort_unstable();
            row_rounds.sort_unstable();
            assert!(
                col_rounds.last().copied().unwrap_or(0)
                    < row_rounds.first().copied().unwrap_or(usize::MAX)
                    || col_rounds.is_empty()
                    || row_rounds.is_empty(),
                "CycleMajor embedded precommitted schedule expects col rounds before row rounds"
            );

            let mut cycle_phase_rounds = Vec::with_capacity(col_rounds.len() + row_rounds.len());
            cycle_phase_rounds.extend(col_rounds.iter().copied());
            cycle_phase_rounds.extend(row_rounds.iter().copied());

            // This is the cycle-phase round *span* (global index domain), not the
            // number of active rounds. Gaps are dummy rounds that must still exist in
            // batched sumcheck timing.
            let cycle_phase_total_rounds = cycle_phase_rounds
                .iter()
                .copied()
                .max()
                .map(|r| r + 1)
                .unwrap_or(0);
            let first_phase_bound_vars = cycle_phase_rounds.len();
            let total_poly_vars = poly_row_vars + poly_col_vars;
            let address_phase_rounds = 0..total_poly_vars.saturating_sub(first_phase_bound_vars);

            return RoundScheduleData {
                cycle_phase_rounds,
                cycle_phase_col_round_count: col_rounds.len(),
                cycle_phase_total_rounds,
                address_phase_rounds,
            };
        }

        let (prefix_col_bits, _address_col_bits, dense_col_bits) =
            Self::address_major_precommitted_col_bit_partition(
                poly_col_vars,
                cycle_rounds,
                address_rounds,
            );
        let col_end = std::cmp::min(
            cycle_rounds,
            prefix_col_bits.saturating_add(dense_col_bits),
        );
        let cycle_phase_col_rounds = 0..col_end;
        let row_start_unclamped = joint_col_vars.saturating_sub(address_rounds);
        let row_start = std::cmp::min(
            cycle_rounds,
            std::cmp::max(row_start_unclamped, col_end),
        );
        let row_end = std::cmp::min(cycle_rounds, row_start + poly_row_vars);
        let cycle_phase_row_rounds = row_start..row_end;
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
        RoundScheduleData {
            cycle_phase_rounds,
            cycle_phase_col_round_count: cycle_phase_col_rounds.len(),
            cycle_phase_total_rounds,
            address_phase_rounds,
        }
    }

    fn two_phase_round_schedule_for_mode(
        mode: PrecommittedEmbeddingMode,
        reference: &PrecommittedSchedulingReference,
        poly_row_vars: usize,
        poly_col_vars: usize,
        has_dominant_reference: bool,
    ) -> RoundScheduleData {
        if has_dominant_reference {
            return dominant_precommitted_two_phase_round_schedule(
                reference.cycle_alignment_rounds,
                reference.address_rounds,
                reference.joint_col_vars,
                poly_row_vars,
                poly_col_vars,
            );
        }
        match mode {
            PrecommittedEmbeddingMode::DominantPrecommitted => {
                dominant_precommitted_two_phase_round_schedule(
                    reference.cycle_alignment_rounds,
                    reference.address_rounds,
                    reference.joint_col_vars,
                    poly_row_vars,
                    poly_col_vars,
                )
            }
            PrecommittedEmbeddingMode::EmbeddedPrecommitted => {
                Self::precommitted_two_phase_round_schedule(
                    reference.cycle_alignment_rounds,
                    reference.address_rounds,
                    reference.joint_col_vars,
                    poly_row_vars,
                    poly_col_vars,
                )
            }
        }
    }

    #[inline]
    pub fn precommitted_round_offset(
        is_cycle_phase: bool,
        max_num_rounds: usize,
        cycle_alignment_rounds: usize,
        _num_rounds_for_phase: usize,
    ) -> usize {
        if is_cycle_phase {
            max_num_rounds.saturating_sub(cycle_alignment_rounds)
        } else {
            match DoryGlobals::get_layout() {
                DoryLayout::AddressMajor => 0,
                DoryLayout::CycleMajor => 0,
            }
        }
    }

    fn round_offset_for_mode(
        mode: PrecommittedEmbeddingMode,
        is_cycle_phase: bool,
        max_num_rounds: usize,
        cycle_alignment_rounds: usize,
        num_rounds_for_phase: usize,
    ) -> usize {
        match mode {
            PrecommittedEmbeddingMode::DominantPrecommitted => {
                if is_cycle_phase {
                    max_num_rounds.saturating_sub(cycle_alignment_rounds)
                } else {
                    max_num_rounds.saturating_sub(num_rounds_for_phase)
                }
            }
            PrecommittedEmbeddingMode::EmbeddedPrecommitted => Self::precommitted_round_offset(
                is_cycle_phase,
                max_num_rounds,
                cycle_alignment_rounds,
                num_rounds_for_phase,
            ),
        }
    }

    fn normalize_precommitted_two_phase_opening_point(
        is_cycle_phase: bool,
        cycle_var_challenges: &[<F as JoltField>::Challenge],
        schedule: &RoundScheduleData,
        challenges: &[<F as JoltField>::Challenge],
        scheduling_reference: &PrecommittedSchedulingReference,
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        if is_cycle_phase {
            let local_cycle_challenges: Vec<F::Challenge> = schedule
                .cycle_phase_rounds
                .iter()
                .map(|&global_round| {
                    let local_round = global_round;
                    assert!(
                        local_round < challenges.len(),
                        "cycle round index out of local bounds: global_round={} local_len={}",
                        global_round,
                        challenges.len()
                    );
                    challenges[local_round]
                })
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
                let main_cycle_rounds = DoryGlobals::main_t().log_2();
                let cycle_rounds = scheduling_reference.cycle_alignment_rounds;
                let cycle_prefix_cols = cycle_rounds.saturating_sub(main_cycle_rounds);
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

    fn normalize_two_phase_opening_point_for_mode(
        mode: PrecommittedEmbeddingMode,
        is_cycle_phase: bool,
        cycle_var_challenges: &[<F as JoltField>::Challenge],
        schedule: &RoundScheduleData,
        challenges: &[<F as JoltField>::Challenge],
        scheduling_reference: &PrecommittedSchedulingReference,
        dense_cycle_prefix_rounds: usize,
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        match mode {
            PrecommittedEmbeddingMode::DominantPrecommitted => {
                if is_cycle_phase {
                    Self::normalize_precommitted_two_phase_opening_point(
                        is_cycle_phase,
                        cycle_var_challenges,
                        schedule,
                        challenges,
                        scheduling_reference,
                    )
                } else {
                    normalize_dominant_precommitted_opening_point(
                        cycle_var_challenges,
                        challenges,
                        scheduling_reference.address_rounds,
                        dense_cycle_prefix_rounds,
                    )
                }
            }
            PrecommittedEmbeddingMode::EmbeddedPrecommitted => {
                Self::normalize_precommitted_two_phase_opening_point(
                    is_cycle_phase,
                    cycle_var_challenges,
                    schedule,
                    challenges,
                    scheduling_reference,
                )
            }
        }
    }

    #[inline(always)]
    pub fn permute_precommitted_sumcheck_index_address_major(
        index: usize,
        poly_row_vars: usize,
        poly_col_vars: usize,
        scheduling_reference: &PrecommittedSchedulingReference,
    ) -> usize {
        let (prefix_bits, address_bits, dense_bits) =
            Self::address_major_precommitted_col_bit_partition(
                poly_col_vars,
                scheduling_reference.cycle_alignment_rounds,
                scheduling_reference.address_rounds,
            );
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
        let joint_cols = DoryGlobals::configured_main_num_columns();
        let poly_cols = 1usize << poly_col_vars;
        let row = index / poly_cols;
        let col = index % poly_cols;
        let global_index = row as u128 * joint_cols as u128 + col as u128;
        let main_trace_t = DoryGlobals::main_t() as u128;
        let address = global_index / main_trace_t;
        let cycle = global_index % main_trace_t;
        (address as usize, cycle as usize)
    }

    #[inline]
    pub fn num_address_phase_rounds(&self) -> usize {
        self.address_phase_rounds.len()
    }

    #[inline]
    pub fn is_cycle_phase_round(&self, round: usize) -> bool {
        self.cycle_phase_rounds.iter().any(|&scheduled| scheduled == round)
    }

    #[inline]
    pub fn cycle_alignment_rounds(&self) -> usize {
        self.scheduling_reference.cycle_alignment_rounds
    }

    #[inline]
    pub fn address_alignment_rounds(&self) -> usize {
        self.scheduling_reference.address_rounds
    }

    #[inline]
    pub fn num_rounds_for_phase(&self, is_cycle_phase: bool) -> usize {
        if is_cycle_phase {
            self.cycle_phase_total_rounds
        } else {
            self.address_phase_rounds.len()
        }
    }

    pub fn round_offset(&self, is_cycle_phase: bool, max_num_rounds: usize) -> usize {
        Self::round_offset_for_mode(
            self.embedding_mode,
            is_cycle_phase,
            max_num_rounds,
            self.cycle_alignment_rounds(),
            self.num_rounds_for_phase(is_cycle_phase),
        )
    }

    pub fn normalize_opening_point(
        &self,
        is_cycle_phase: bool,
        challenges: &[F::Challenge],
        dense_cycle_prefix_rounds: usize,
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        Self::normalize_two_phase_opening_point_for_mode(
            self.embedding_mode,
            is_cycle_phase,
            &self.cycle_var_challenges,
            &RoundScheduleData {
                cycle_phase_rounds: self.cycle_phase_rounds.clone(),
                cycle_phase_col_round_count: self.cycle_phase_col_round_count,
                cycle_phase_total_rounds: self.cycle_phase_total_rounds,
                address_phase_rounds: self.address_phase_rounds.clone(),
            },
            challenges,
            &self.scheduling_reference,
            dense_cycle_prefix_rounds,
        )
    }

    #[inline]
    pub fn record_cycle_challenge(&mut self, challenge: F::Challenge) {
        self.cycle_var_challenges.push(challenge);
    }

    #[inline]
    pub fn set_cycle_var_challenges(&mut self, challenges: Vec<F::Challenge>) {
        self.cycle_var_challenges = challenges;
    }
}

fn dominant_precommitted_two_phase_round_schedule(
    // Number of cycle-phase rounds available in the aligned domain
    // (can exceed the native trace log-length in dominant embeddings).
    cycle_round_limit: usize,
    address_alignment_rounds: usize,
    joint_col_vars: usize,
    poly_row_vars: usize,
    poly_col_vars: usize,
) -> RoundScheduleData {
    let (cycle_phase_col_rounds, cycle_phase_row_rounds) = match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => {
            let col_end = std::cmp::min(cycle_round_limit, poly_col_vars);
            let col_binding_rounds = 0..col_end;
            let row_start = std::cmp::min(
                cycle_round_limit,
                std::cmp::max(
                    std::cmp::min(cycle_round_limit, joint_col_vars),
                    col_end,
                ),
            );
            let row_end = std::cmp::min(cycle_round_limit, row_start + poly_row_vars);
            let row_binding_rounds = row_start..row_end;
            (col_binding_rounds, row_binding_rounds)
        }
        DoryLayout::AddressMajor => {
            let col_end = std::cmp::min(
                cycle_round_limit,
                poly_col_vars.saturating_sub(address_alignment_rounds),
            );
            let col_binding_rounds = 0..col_end;
            let row_start_unclamped = joint_col_vars.saturating_sub(address_alignment_rounds);
            let row_start = std::cmp::min(
                cycle_round_limit,
                std::cmp::max(row_start_unclamped, col_end),
            );
            let row_end = std::cmp::min(cycle_round_limit, row_start + poly_row_vars);
            let row_binding_rounds = row_start..row_end;
            (col_binding_rounds, row_binding_rounds)
        }
    };
    let cycle_phase_total_rounds = if !cycle_phase_row_rounds.is_empty() {
        cycle_phase_row_rounds
            .end
            .saturating_sub(cycle_phase_col_rounds.start)
    } else {
        cycle_phase_col_rounds.len()
    };
    let mut cycle_phase_rounds =
        Vec::with_capacity(cycle_phase_col_rounds.len() + cycle_phase_row_rounds.len());
    cycle_phase_rounds.extend(cycle_phase_col_rounds.clone());
    cycle_phase_rounds.extend(cycle_phase_row_rounds.clone());
    let total_poly_vars = poly_row_vars + poly_col_vars;
    let first_phase_bound_vars = cycle_phase_rounds.len();
    let address_phase_rounds = 0..total_poly_vars.saturating_sub(first_phase_bound_vars);
    RoundScheduleData {
        cycle_phase_rounds,
        cycle_phase_col_round_count: cycle_phase_col_rounds.len(),
        cycle_phase_total_rounds,
        address_phase_rounds,
    }
}

pub fn permute_precommitted_value_and_eq_coeffs<V: Copy + Send + Sync, F: JoltField>(
    value_coeffs: Vec<V>,
    eq_coeffs: Vec<F>,
    embedding_mode: PrecommittedEmbeddingMode,
    poly_row_vars: usize,
    poly_col_vars: usize,
    scheduling_reference: &PrecommittedSchedulingReference,
    dense_cycle_prefix_vars: usize,
) -> (Vec<V>, Vec<F>) {
    assert_eq!(value_coeffs.len(), eq_coeffs.len());
    match embedding_mode {
        PrecommittedEmbeddingMode::EmbeddedPrecommitted => match DoryGlobals::get_layout() {
            DoryLayout::AddressMajor => {
                let mut permuted_values = value_coeffs.clone();
                let mut permuted_eq = vec![F::zero(); eq_coeffs.len()];
                for old_idx in 0..value_coeffs.len() {
                    let new_idx =
                        PrecommittedClaimReduction::<F>::permute_precommitted_sumcheck_index_address_major(
                            old_idx,
                            poly_row_vars,
                            poly_col_vars,
                            scheduling_reference,
                        );
                    permuted_values[new_idx] = value_coeffs[old_idx];
                    permuted_eq[new_idx] = eq_coeffs[old_idx];
                }
                (permuted_values, permuted_eq)
            }
            DoryLayout::CycleMajor => {
                let mut permuted_coeffs: Vec<(usize, (V, F))> = value_coeffs
                    .into_par_iter()
                    .zip(eq_coeffs.into_par_iter())
                    .enumerate()
                    .collect();
                permuted_coeffs.par_sort_by(|&(idx_a, _), &(idx_b, _)| {
                    let (address_a, cycle_a) =
                        PrecommittedClaimReduction::<F>::cycle_major_top_left_index_to_address_cycle(
                            idx_a,
                            poly_col_vars,
                        );
                    let (address_b, cycle_b) =
                        PrecommittedClaimReduction::<F>::cycle_major_top_left_index_to_address_cycle(
                            idx_b,
                            poly_col_vars,
                        );
                    address_a
                        .cmp(&address_b)
                        .then_with(|| cycle_a.cmp(&cycle_b))
                });
                permuted_coeffs
                    .into_par_iter()
                    .map(|(_, coeffs)| coeffs)
                    .unzip()
            }
        },
        PrecommittedEmbeddingMode::DominantPrecommitted => {
            let num_vars = poly_row_vars + poly_col_vars;
            if !dominant_needs_sumcheck_permutation(
                num_vars,
                scheduling_reference.address_rounds,
                dense_cycle_prefix_vars,
            ) {
                return (value_coeffs, eq_coeffs);
            }

            let mut permuted_values = value_coeffs.clone();
            let mut permuted_eq = vec![F::zero(); eq_coeffs.len()];
            for old_idx in 0..value_coeffs.len() {
                let new_idx = dominant_permute_sumcheck_index(
                    old_idx,
                    num_vars,
                    scheduling_reference.address_rounds,
                    dense_cycle_prefix_vars,
                );
                permuted_values[new_idx] = value_coeffs[old_idx];
                permuted_eq[new_idx] = eq_coeffs[old_idx];
            }
            (permuted_values, permuted_eq)
        }
    }
}

pub fn build_permuted_precommitted_polys<V: Copy + Send + Sync, F: JoltField>(
    value_coeffs: Vec<V>,
    eq_coeffs: Vec<F>,
    embedding_mode: PrecommittedEmbeddingMode,
    poly_row_vars: usize,
    poly_col_vars: usize,
    scheduling_reference: &PrecommittedSchedulingReference,
    dense_cycle_prefix_vars: usize,
) -> (MultilinearPolynomial<F>, MultilinearPolynomial<F>)
where
    MultilinearPolynomial<F>: From<Vec<V>>,
{
    let (permuted_values, permuted_eq) = permute_precommitted_value_and_eq_coeffs(
        value_coeffs,
        eq_coeffs,
        embedding_mode,
        poly_row_vars,
        poly_col_vars,
        scheduling_reference,
        dense_cycle_prefix_vars,
    );
    let value_poly: MultilinearPolynomial<F> = permuted_values.into();
    let eq_poly: MultilinearPolynomial<F> = permuted_eq.into();
    (value_poly, eq_poly)
}

pub fn normalize_dominant_precommitted_opening_point<F: JoltField>(
    cycle_var_challenges: &[F::Challenge],
    challenges: &[F::Challenge],
    address_alignment_rounds: usize,
    dense_cycle_prefix_vars: usize,
) -> OpeningPoint<BIG_ENDIAN, F> {
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => {
            let opening_point_be: OpeningPoint<BIG_ENDIAN, F> =
                OpeningPoint::<LITTLE_ENDIAN, F>::new([cycle_var_challenges, challenges].concat())
                    .match_endianness();

            let total_rounds = opening_point_be.r.len();
            let stage7_rounds = address_alignment_rounds.min(total_rounds);
            let dense_rounds =
                dense_cycle_prefix_vars.min(total_rounds.saturating_sub(stage7_rounds));
            let precommitted_prefix_rounds =
                total_rounds.saturating_sub(stage7_rounds + dense_rounds);

            let stage7_start = 0;
            let stage7_end = stage7_start + stage7_rounds;
            let dense_start = stage7_end;
            let dense_end = dense_start + dense_rounds;
            let precommitted_prefix_start = dense_end;
            let precommitted_prefix_end = precommitted_prefix_start + precommitted_prefix_rounds;
            assert_eq!(precommitted_prefix_end, total_rounds);

            let mut reordered = Vec::with_capacity(total_rounds);
            reordered.extend_from_slice(
                &opening_point_be.r[precommitted_prefix_start..precommitted_prefix_end],
            );
            reordered.extend_from_slice(&opening_point_be.r[stage7_start..stage7_end]);
            reordered.extend_from_slice(&opening_point_be.r[dense_start..dense_end]);
            OpeningPoint::<BIG_ENDIAN, F>::new(reordered)
        }
        DoryLayout::AddressMajor => {
            let stage6_rounds = cycle_var_challenges.len();
            let dense_rounds = dense_cycle_prefix_vars.min(stage6_rounds);
            let precommitted_prefix_rounds = stage6_rounds.saturating_sub(dense_rounds);

            let stage6_head = &cycle_var_challenges[..precommitted_prefix_rounds];
            let stage6_tail = &cycle_var_challenges[precommitted_prefix_rounds..];

            let mut reordered = Vec::with_capacity(stage6_rounds + challenges.len());
            reordered.extend(stage6_tail.iter().rev().cloned());
            reordered.extend(challenges.iter().rev().cloned());
            reordered.extend(stage6_head.iter().rev().cloned());

            OpeningPoint::<BIG_ENDIAN, F>::new(reordered)
        }
    }
}

#[inline(always)]
fn dominant_needs_sumcheck_permutation(
    num_vars: usize,
    address_alignment_rounds: usize,
    dense_cycle_prefix_vars: usize,
) -> bool {
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => {
            let k = address_alignment_rounds.min(num_vars);
            let t = dense_cycle_prefix_vars.min(num_vars.saturating_sub(k));
            let c = num_vars.saturating_sub(k + t);
            c > 0
        }
        DoryLayout::AddressMajor => {
            let k = address_alignment_rounds.min(num_vars);
            let t = dense_cycle_prefix_vars.min(num_vars.saturating_sub(k));
            k > 0 && t > 0
        }
    }
}

#[inline(always)]
fn dominant_permute_sumcheck_index(
    index: usize,
    num_vars: usize,
    address_alignment_rounds: usize,
    dense_cycle_prefix_vars: usize,
) -> usize {
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => {
            let k = address_alignment_rounds.min(num_vars);
            let t = dense_cycle_prefix_vars.min(num_vars.saturating_sub(k));
            let c = num_vars.saturating_sub(k + t);
            if c == 0 {
                return index;
            }
            assert!(num_vars < usize::BITS as usize);
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
            let k = address_alignment_rounds.min(num_vars);
            let t = dense_cycle_prefix_vars.min(num_vars.saturating_sub(k));
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

pub const TWO_PHASE_DEGREE_BOUND: usize = 2;

pub trait PrecomittedParams<F: JoltField>: SumcheckInstanceParams<F> {
    fn is_cycle_phase(&self) -> bool;
    fn is_cycle_phase_round(&self, round: usize) -> bool;
    fn cycle_alignment_rounds(&self) -> usize;
    fn address_alignment_rounds(&self) -> usize;
    fn record_cycle_challenge(&mut self, challenge: F::Challenge);
}

#[derive(Allocative)]
pub struct PrecomittedProver<F: JoltField, P: PrecomittedParams<F>> {
    params: P,
    value_poly: MultilinearPolynomial<F>,
    eq_poly: MultilinearPolynomial<F>,
    scale: F,
}

impl<F: JoltField, P: PrecomittedParams<F>> PrecomittedProver<F, P> {
    pub fn new(
        params: P,
        value_poly: MultilinearPolynomial<F>,
        eq_poly: MultilinearPolynomial<F>,
    ) -> Self {
        Self {
            params,
            value_poly,
            eq_poly,
            scale: F::one(),
        }
    }

    pub fn params(&self) -> &P {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut P {
        &mut self.params
    }

    fn compute_message_unscaled(&self, previous_claim_unscaled: F) -> UniPoly<F> {
        let half = self.value_poly.len() / 2;
        let value_poly = &self.value_poly;
        let eq_poly = &self.eq_poly;
        let evals: [F; TWO_PHASE_DEGREE_BOUND] = (0..half)
            .into_par_iter()
            .map(|j| {
                let value_evals = value_poly
                    .sumcheck_evals_array::<TWO_PHASE_DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                let eq_evals = eq_poly
                    .sumcheck_evals_array::<TWO_PHASE_DEGREE_BOUND>(j, BindingOrder::LowToHigh);

                let mut out = [F::zero(); TWO_PHASE_DEGREE_BOUND];
                for i in 0..TWO_PHASE_DEGREE_BOUND {
                    out[i] = value_evals[i] * eq_evals[i];
                }
                out
            })
            .reduce(
                || [F::zero(); TWO_PHASE_DEGREE_BOUND],
                |mut acc, arr| {
                    acc.iter_mut().zip(arr.iter()).for_each(|(a, b)| *a += *b);
                    acc
                },
            );
        UniPoly::from_evals_and_hint(previous_claim_unscaled, &evals)
    }

    pub fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
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

    pub fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        if self.params.is_cycle_phase() {
            let is_dummy_round = !self.params.is_cycle_phase_round(round);
            if is_dummy_round {
                self.scale *= F::from_u64(2).inverse().unwrap();
            } else {
                self.value_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
                self.eq_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
                self.params.record_cycle_challenge(r_j);
            }
            return;
        }

        self.value_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
        self.eq_poly.bind_parallel(r_j, BindingOrder::LowToHigh);
    }

    pub fn cycle_intermediate_claim(&self) -> F {
        let len = self.value_poly.len();
        assert_eq!(len, self.eq_poly.len());

        let mut sum = F::zero();
        for i in 0..len {
            sum += self.value_poly.get_bound_coeff(i) * self.eq_poly.get_bound_coeff(i);
        }
        sum * self.scale
    }

    pub fn final_claim_if_ready(&self) -> Option<F> {
        if self.value_poly.len() == 1 {
            Some(self.value_poly.get_bound_coeff(0))
        } else {
            None
        }
    }
}

pub fn precommitted_dummy_round_scale<F: JoltField>(precommitted: &PrecommittedClaimReduction<F>) -> F {
    let gap_len = precommitted.cycle_phase_total_rounds - precommitted.cycle_phase_rounds.len();
    let two_inv = F::from_u64(2).inverse().unwrap();
    (0..gap_len).fold(F::one(), |acc, _| acc * two_inv)
}
