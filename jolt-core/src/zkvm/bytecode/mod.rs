use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use tracer::instruction::Cycle;

use crate::poly::commitment::commitment_scheme::CommitmentScheme;

pub mod chunks;
pub mod read_raf_checking;

pub use jolt_program::preprocess::{BytecodePCMapper, BytecodePreprocessing, PreprocessingError};

#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct TrustedBytecodeCommitments<PCS: CommitmentScheme> {
    pub commitments: Vec<PCS::Commitment>,
    pub num_columns: usize,
    pub log_k_chunk: u8,
    pub bytecode_chunk_count: usize,
    pub bytecode_len: usize,
    pub bytecode_T: usize,
}

pub fn get_pc_for_cycle(bytecode: &BytecodePreprocessing, cycle: &Cycle) -> usize {
    if matches!(cycle, Cycle::NoOp) {
        return 0;
    }
    let instruction = cycle
        .instruction()
        .try_jolt_instruction_row()
        .expect("trace cycle must be a final Jolt instruction row");
    match bytecode.get_pc(&instruction) {
        Some(pc) => pc,
        None => panic!(
            "bytecode preprocessing is missing PC mapping for instruction at address {:#x} with virtual_sequence_remaining {:?}",
            instruction.address, instruction.virtual_sequence_remaining
        ),
    }
}

pub fn entry_bytecode_index(bytecode: &BytecodePreprocessing) -> usize {
    match bytecode.entry_bytecode_index() {
        Some(pc) => pc,
        None => panic!(
            "bytecode preprocessing is missing entry mapping for address {:#x}",
            bytecode.entry_address
        ),
    }
}
