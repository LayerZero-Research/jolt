use core::panic::AssertUnwindSafe;
use std::panic;

use crate::emulator::cpu::Xlen;
use crate::emulator::test_harness::CpuTestHarness;
use crate::instruction::format::{InstructionFormat, InstructionRegisterState};
use crate::instruction::NormalizedInstruction;

use super::{
    addiw::ADDIW, addw::ADDW, amoaddd::AMOADDD, amoaddw::AMOADDW, amoandd::AMOANDD,
    amoandw::AMOANDW, amomaxd::AMOMAXD, amomaxud::AMOMAXUD, amomaxuw::AMOMAXUW, amomaxw::AMOMAXW,
    amomind::AMOMIND, amominud::AMOMINUD, amominuw::AMOMINUW, amominw::AMOMINW, amoord::AMOORD,
    amoorw::AMOORW, amoswapd::AMOSWAPD, amoswapw::AMOSWAPW, amoxord::AMOXORD, amoxorw::AMOXORW,
    div::DIV, divu::DIVU, divuw::DIVUW, divw::DIVW, lb::LB, lbu::LBU, lh::LH, lhu::LHU, lw::LW,
    lwu::LWU, mulh::MULH, mulhsu::MULHSU, mulw::MULW, rem::REM, remu::REMU, remuw::REMUW,
    remw::REMW, sb::SB, sh::SH, sll::SLL, slli::SLLI, slliw::SLLIW, sllw::SLLW, sra::SRA,
    srai::SRAI, sraiw::SRAIW, sraw::SRAW, srl::SRL, srli::SRLI, srliw::SRLIW, srlw::SRLW,
    subw::SUBW, sw::SW, RISCVInstruction, RISCVTrace,
};

use common::constants::RISCV_REGISTER_COUNT;

use rand::{rngs::StdRng, SeedableRng};

use super::{RISCVCycle, RV32IMCycle};

macro_rules! test_virtual_sequences_64 {
  ($( $instr:ty ),* $(,)?) => {
      $(
          paste::paste! {
              #[test]
              fn [<test_fuzz_rv64_ $instr:lower _virtual_sequence>]() {
                  virtual_sequence_trace_test_for_xlen::<$instr>(Xlen::Bit64);
              }
          }
      )*
  };
}

macro_rules! test_virtual_sequences_32 {
  ($( $instr:ty ),* $(,)?) => {
      $(
          paste::paste! {
              #[test]
              fn [<test_fuzz_rv32_ $instr:lower _virtual_sequence>]() {
                  virtual_sequence_trace_test_for_xlen::<$instr>(Xlen::Bit32);
              }
          }
      )*
  };
}

test_virtual_sequences_64!(
    ADDIW, ADDW, AMOADDD, AMOADDW, AMOANDD, AMOANDW, AMOMAXD, AMOMAXUD, AMOMAXUW, AMOMAXW, AMOMIND,
    AMOMINUD, AMOMINUW, AMOMINW, AMOORD, AMOORW, AMOSWAPD, AMOSWAPW, AMOXORD, AMOXORW, DIV, DIVU,
    DIVUW, DIVW, LB, LBU, LH, LHU, LW, LWU, MULH, MULHSU, MULW, REM, REMU, REMUW, REMW, SB, SH,
    SLL, SLLI, SLLIW, SLLW, SRA, SRAI, SRAIW, SRAW, SRL, SRLI, SRLIW, SRLW, SUBW, SW,
);

// RV32-only/valid instruction subset (exclude RV64-only ops: *W variants, LWU, and AMO*D)
test_virtual_sequences_32!(
    AMOADDW, AMOANDW, AMOORW, AMOXORW, AMOMINW, AMOMAXW, AMOMINUW, AMOMAXUW, DIV, DIVU, LB, LBU,
    LH, LHU, LW, MULH, MULHSU, REM, REMU, SB, SH, SLL, SLLI, SRA, SRAI, SRL, SRLI, SW,
);

fn test_rng() -> StdRng {
    let seed = [0u8; 32];
    StdRng::from_seed(seed)
}

pub fn virtual_sequence_trace_test_for_xlen<I: RISCVInstruction + RISCVTrace + Copy>(xlen: Xlen)
where
    RV32IMCycle: From<RISCVCycle<I>>,
{
    let mut rng = test_rng();

    for _ in 0..1000 {
        let instruction = I::random(&mut rng);
        let instr: NormalizedInstruction = instruction.into();
        let register_state =
            <<I::Format as InstructionFormat>::RegisterState as InstructionRegisterState>::random(
                &mut rng,
            );

        let mut original = match xlen {
            Xlen::Bit32 => CpuTestHarness::new_32(),
            Xlen::Bit64 => CpuTestHarness::new(),
        };
        let mut virtual_ = match xlen {
            Xlen::Bit32 => CpuTestHarness::new_32(),
            Xlen::Bit64 => CpuTestHarness::new(),
        };

        if instr.operands.rs1 != 0 {
            original.write_register(instr.operands.rs1, register_state.rs1_value());
            virtual_.write_register(instr.operands.rs1, register_state.rs1_value());
        }
        if instr.operands.rs2 != 0 {
            original.write_register(instr.operands.rs2, register_state.rs2_value());
            virtual_.write_register(instr.operands.rs2, register_state.rs2_value());
        }

        let mut ram_access = Default::default();

        let res = panic::catch_unwind(AssertUnwindSafe(|| {
            instruction.execute(&mut original.cpu, &mut ram_access);
        }));
        if res.is_err() {
            continue;
        }

        let mut trace_vec = Vec::new();
        instruction.trace(&mut virtual_.cpu, Some(&mut trace_vec));

        let mut mismatches: Vec<String> = Vec::new();

        if original.cpu.pc != virtual_.cpu.pc {
            mismatches.push(format!(
                "PC mismatch: orig=0x{:x} virt=0x{:x}",
                original.cpu.pc, virtual_.cpu.pc
            ));
        }

        for i in 0..RISCV_REGISTER_COUNT {
            let o = original.cpu.x[i as usize];
            let v = virtual_.cpu.x[i as usize];
            if o != v {
                mismatches.push(format!(
                    "x{:02}: orig=0x{:016x} ({}) virt=0x{:016x} ({})",
                    i, o as u64, o, v as u64, v
                ));
            }
        }

        if !mismatches.is_empty() {
            // Build high-signal context for debugging
            let rs1_idx = instr.operands.rs1;
            let rs2_idx = instr.operands.rs2;
            let rd_idx = instr.operands.rd;
            let rs1_val = if rs1_idx == 0 {
                0
            } else {
                register_state.rs1_value()
            };
            let rs2_val = if rs2_idx == 0 {
                0
            } else {
                register_state.rs2_value()
            };

            let header = format!(
                "Exec vs Trace mismatch (xlen={:?})\n  instr={:?}\n  rs1=x{} val=0x{:016x}\n  rs2=x{} val=0x{:016x}\n  rd=x{}\n  trace_len={}\n  diffs ({}):",
                xlen,
                instruction,
                rs1_idx,
                rs1_val,
                rs2_idx,
                rs2_val,
                rd_idx,
                trace_vec.len(),
                mismatches.len()
            );

            let details = mismatches
                .iter()
                .take(32)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n    ");

            assert!(
                false,
                "{}\n    {}{}",
                header,
                details,
                if mismatches.len() > 32 {
                    format!("\n    (+{} more)", mismatches.len() - 32)
                } else {
                    String::new()
                }
            );
        }
    }
}

// Backward-compat: keep a default RV64 test entry point if older test names invoke it.
#[allow(dead_code)]
pub fn virtual_sequence_trace_test<I: RISCVInstruction + RISCVTrace + Copy>()
where
    RV32IMCycle: From<RISCVCycle<I>>,
{
    virtual_sequence_trace_test_for_xlen::<I>(Xlen::Bit64)
}
