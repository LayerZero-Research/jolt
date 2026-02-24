use crate::field::JoltField;
use crate::poly::commitment::dory::DoryGlobals;
use crate::poly::multilinear_polynomial::MultilinearPolynomial;
use crate::utils::thread::unsafe_allocate_zero_vec;
use crate::zkvm::instruction::{
    Flags, InstructionLookup, InterleavedBitsMarker, NUM_CIRCUIT_FLAGS, NUM_INSTRUCTION_FLAGS,
};
use crate::zkvm::lookup_table::LookupTables;
use common::constants::{REGISTER_COUNT, XLEN};
use tracer::instruction::Instruction;

/// Total number of "lanes" to commit bytecode fields
pub const fn total_lanes() -> usize {
    3 * (REGISTER_COUNT as usize) // rs1, rs2, rd one-hot lanes
        + 2 // unexpanded_pc, imm
        + NUM_CIRCUIT_FLAGS
        + NUM_INSTRUCTION_FLAGS
        + <LookupTables<XLEN> as strum::EnumCount>::COUNT
        + 1 // raf flag
}

/// Fixed committed bytecode lane capacity per opcode.
///
/// The canonical lane vector (`total_lanes`) occupies the prefix and the remaining
/// lanes are zero-padded in committed mode.
pub const COMMITTED_BYTECODE_LANE_CAPACITY: usize = total_lanes().next_power_of_two();

#[inline(always)]
pub const fn committed_lanes() -> usize {
    COMMITTED_BYTECODE_LANE_CAPACITY
}

/// Canonical lane layout for committed bytecode lanes.
///
/// The global lane order matches [`lane_value`] and the weights in
/// `claim_reductions/bytecode_single.rs::compute_lane_weights`.
#[derive(Clone, Copy, Debug)]
pub struct BytecodeLaneLayout {
    pub rs1_start: usize,
    pub rs2_start: usize,
    pub rd_start: usize,
    pub unexp_pc_idx: usize,
    pub imm_idx: usize,
    pub circuit_start: usize,
    pub instr_start: usize,
    pub lookup_start: usize,
    pub raf_flag_idx: usize,
}

impl BytecodeLaneLayout {
    pub const fn new() -> Self {
        let reg_count = REGISTER_COUNT as usize;
        let rs1_start = 0usize;
        let rs2_start = rs1_start + reg_count;
        let rd_start = rs2_start + reg_count;
        let unexp_pc_idx = rd_start + reg_count;
        let imm_idx = unexp_pc_idx + 1;
        let circuit_start = imm_idx + 1;
        let instr_start = circuit_start + NUM_CIRCUIT_FLAGS;
        let lookup_start = instr_start + NUM_INSTRUCTION_FLAGS;
        let raf_flag_idx = lookup_start + <LookupTables<XLEN> as strum::EnumCount>::COUNT;
        Self {
            rs1_start,
            rs2_start,
            rd_start,
            unexp_pc_idx,
            imm_idx,
            circuit_start,
            instr_start,
            lookup_start,
            raf_flag_idx,
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub const fn total_lanes(&self) -> usize {
        self.raf_flag_idx + 1
    }

    /// True for all lanes except `unexpanded_pc` and `imm`.
    #[inline(always)]
    #[allow(dead_code)]
    pub const fn is_boolean_lane(&self, global_lane: usize) -> bool {
        global_lane != self.unexp_pc_idx && global_lane != self.imm_idx
    }
}

pub const BYTECODE_LANE_LAYOUT: BytecodeLaneLayout = BytecodeLaneLayout::new();

/// Active lane values for a single instruction.
///
/// Most lanes are boolean/one-hot, so we represent them as `One` to avoid
/// unnecessary field multiplications at call sites (e.g. Dory VMV).
#[derive(Clone, Copy, Debug)]
pub enum ActiveLaneValue<F: JoltField> {
    One,
    Scalar(F),
}

/// Enumerate the non-zero lanes for a single instruction in canonical global-lane order.
///
/// This is the sparse counterpart to [`lane_value`]: instead of scanning all lanes and
/// branching on zeros, we directly visit only lanes that are 1 (for boolean/one-hot lanes)
/// or have a non-zero scalar value (for `unexpanded_pc` and `imm`).
///
/// This is useful for:
/// - Streaming / VMV computations where the downstream logic needs to map lanes to matrix indices
/// - Any place where per-lane work dominates and the instruction lane vector is sparse
#[inline(always)]
pub fn for_each_active_lane_value<F: JoltField>(
    instr: &Instruction,
    mut visit: impl FnMut(usize, ActiveLaneValue<F>),
) {
    let l = BYTECODE_LANE_LAYOUT;

    let normalized = instr.normalize();
    let circuit_flags = <Instruction as Flags>::circuit_flags(instr);
    let instr_flags = <Instruction as Flags>::instruction_flags(instr);
    let lookup_idx = <Instruction as InstructionLookup<XLEN>>::lookup_table(instr)
        .map(|t| LookupTables::<XLEN>::enum_index(&t));
    let raf_flag = !InterleavedBitsMarker::is_interleaved_operands(&circuit_flags);

    // One-hot register lanes.
    if let Some(r) = normalized.operands.rs1 {
        visit(l.rs1_start + (r as usize), ActiveLaneValue::One);
    }
    if let Some(r) = normalized.operands.rs2 {
        visit(l.rs2_start + (r as usize), ActiveLaneValue::One);
    }
    if let Some(r) = normalized.operands.rd {
        visit(l.rd_start + (r as usize), ActiveLaneValue::One);
    }

    // Scalar lanes (skip if zero).
    let unexpanded_pc = F::from_u64(normalized.address as u64);
    if !unexpanded_pc.is_zero() {
        visit(l.unexp_pc_idx, ActiveLaneValue::Scalar(unexpanded_pc));
    }
    let imm = F::from_i128(normalized.operands.imm);
    if !imm.is_zero() {
        visit(l.imm_idx, ActiveLaneValue::Scalar(imm));
    }

    // Circuit flags.
    for i in 0..NUM_CIRCUIT_FLAGS {
        if circuit_flags[i] {
            visit(l.circuit_start + i, ActiveLaneValue::One);
        }
    }

    // Instruction flags.
    for i in 0..NUM_INSTRUCTION_FLAGS {
        if instr_flags[i] {
            visit(l.instr_start + i, ActiveLaneValue::One);
        }
    }

    // Lookup selector.
    if let Some(t) = lookup_idx {
        visit(l.lookup_start + t, ActiveLaneValue::One);
    }

    // RAF flag.
    if raf_flag {
        visit(l.raf_flag_idx, ActiveLaneValue::One);
    }
}


/// Coefficients are written at `(lane, cycle)` for all active canonical lanes, and all
/// remaining lane slots (`[total_lanes()..512)`) are zero-padded.
#[tracing::instrument(
    skip_all,
    name = "bytecode::build_committed_bytecode_polynomial_from_instructions"
)]
pub fn build_committed_bytecode_polynomial_from_instructions<F: JoltField>(
    instructions: &[Instruction],
) -> MultilinearPolynomial<F> {
    let bytecode_len = instructions.len();
    let lane_capacity = committed_lanes();
    let mut coeffs = unsafe_allocate_zero_vec(lane_capacity * bytecode_len);

    for (cycle, instr) in instructions.iter().enumerate() {
        for_each_active_lane_value::<F>(instr, |global_lane, lane_val| {
            debug_assert!(global_lane < total_lanes());
            let idx = DoryGlobals::get_layout().address_cycle_to_index(
                global_lane,
                cycle,
                lane_capacity,
                bytecode_len,
            );
            coeffs[idx] = match lane_val {
                ActiveLaneValue::One => F::one(),
                ActiveLaneValue::Scalar(v) => v,
            };
        });
    }

    MultilinearPolynomial::from(coeffs)
}