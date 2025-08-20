use core::panic::AssertUnwindSafe;
use std::panic;

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

macro_rules! test_virtual_sequences {
  ($( $instr:ty ),* $(,)?) => {
      $(
          paste::paste! {
              #[test]
              fn [<test_ $instr:lower _virtual_sequence>]() {
                  virtual_sequence_trace_test::<$instr>();
              }
          }
      )*
  };
}

test_virtual_sequences!(
    ADDIW, ADDW, AMOADDD, AMOADDW, AMOANDD, AMOANDW, AMOMAXD, AMOMAXUD, AMOMAXUW, AMOMAXW, AMOMIND,
    AMOMINUD, AMOMINUW, AMOMINW, AMOORD, AMOORW, AMOSWAPD, AMOSWAPW, AMOXORD, AMOXORW, DIV, DIVU,
    DIVUW, DIVW, LB, LBU, LH, LHU, LW, LWU, MULH, MULHSU, MULW, REM, REMU, REMUW, REMW, SB, SH,
    SLL, SLLI, SLLIW, SLLW, SRA, SRAI, SRAIW, SRAW, SRL, SRLI, SRLIW, SRLW, SUBW, SW,
);

fn test_rng() -> StdRng {
    let seed = [0u8; 32];
    StdRng::from_seed(seed)
}

pub fn virtual_sequence_trace_test<I: RISCVInstruction + RISCVTrace + Copy>()
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

        let mut original = CpuTestHarness::new();
        let mut virtual_ = CpuTestHarness::new();

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

        assert_eq!(
            original.cpu.pc, virtual_.cpu.pc,
            "PC register has different values after execution"
        );

        for i in 0..RISCV_REGISTER_COUNT {
            assert_eq!(
                original.cpu.x[i as usize], virtual_.cpu.x[i as usize],
                "Register {} has different values after execution. Original: {:?}, Virtual: {:?}",
                i, original.cpu.x[i as usize], virtual_.cpu.x[i as usize]
            );
        }
    }
}
