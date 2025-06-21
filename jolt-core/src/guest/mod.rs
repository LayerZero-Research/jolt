use common::jolt_device::{JoltDevice, MemoryConfig};
use tracer::{
    self,
    instruction::{RV32IMCycle, RV32IMInstruction, VirtualInstructionSequence},
};

use rayon::prelude::*;

pub mod prover;
pub mod verifier;

/// Runtime configuration for proof generation and verification
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    /// Maximum input size in bytes
    pub max_input_size: u64,
    /// Maximum output size in bytes
    pub max_output_size: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_input_size: 4 * 1024,  // 4KB
            max_output_size: 4 * 1024, // 4KB
        }
    }
}

/// Program represents a decoded RISC-V program
#[derive(Clone)]
pub struct Program {
    elf_contents: Vec<u8>,
}

impl Program {
    /// Create a new program from ELF contents
    pub fn new(elf: impl Into<Vec<u8>>) -> Self {
        Self {
            elf_contents: elf.into(),
        }
    }

    /// Decode the ELF file into instructions and memory initialization
    pub fn decode(&self) -> (Vec<RV32IMInstruction>, Vec<(u64, u8)>) {
        let (bytecode, memory_init) = tracer::decode(&self.elf_contents);
        let bytecode = Self::expand(bytecode);
        (bytecode, memory_init)
    }

    /// Expand virtual sequences into actual instructions
    pub fn expand(instructions: Vec<RV32IMInstruction>) -> Vec<RV32IMInstruction> {
        instructions
            .into_par_iter()
            .flat_map_iter(|instr| match instr {
                RV32IMInstruction::DIV(div) => div.virtual_sequence(),
                RV32IMInstruction::DIVU(divu) => divu.virtual_sequence(),
                RV32IMInstruction::LB(lb) => lb.virtual_sequence(),
                RV32IMInstruction::LBU(lbu) => lbu.virtual_sequence(),
                RV32IMInstruction::LH(lh) => lh.virtual_sequence(),
                RV32IMInstruction::LHU(lhu) => lhu.virtual_sequence(),
                RV32IMInstruction::MULH(mulh) => mulh.virtual_sequence(),
                RV32IMInstruction::MULHSU(mulhsu) => mulhsu.virtual_sequence(),
                RV32IMInstruction::REM(rem) => rem.virtual_sequence(),
                RV32IMInstruction::REMU(remu) => remu.virtual_sequence(),
                RV32IMInstruction::SB(sb) => sb.virtual_sequence(),
                RV32IMInstruction::SH(sh) => sh.virtual_sequence(),
                RV32IMInstruction::SLL(sll) => sll.virtual_sequence(),
                RV32IMInstruction::SLLI(slli) => slli.virtual_sequence(),
                RV32IMInstruction::SRA(sra) => sra.virtual_sequence(),
                RV32IMInstruction::SRAI(srai) => srai.virtual_sequence(),
                RV32IMInstruction::SRL(srl) => srl.virtual_sequence(),
                RV32IMInstruction::SRLI(srli) => srli.virtual_sequence(),
                RV32IMInstruction::SHA256(sha256) => sha256.virtual_sequence(),
                RV32IMInstruction::SHA256INIT(sha256init) => sha256init.virtual_sequence(),
                _ => vec![instr],
            })
            .collect()
    }

    /// Generate execution trace for the program
    pub fn trace(&self, inputs: &[u8], config: &RuntimeConfig) -> (JoltDevice, Vec<RV32IMCycle>) {
        let memory_config = MemoryConfig {
            max_input_size: config.max_input_size,
            max_output_size: config.max_output_size,
            ..Default::default() // Stack size and memory size will be set by host
        };

        let (trace, io_device) = tracer::trace(self.elf_contents.clone(), inputs, &memory_config);
        (io_device, trace)
    }
}
