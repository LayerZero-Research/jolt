use serde::{Deserialize, Serialize};

use crate::{declare_riscv_instr, emulator::cpu::Cpu};

use super::{format::format_r::FormatR, RISCVInstruction, RISCVTrace};

declare_riscv_instr!(
    name   = ADD,
    mask   = 0xfe00707f,
    match  = 0x00000033,
    format = FormatR,
    ram    = ()
);

impl ADD {
    fn exec(&self, cpu: &mut Cpu, _: &mut <ADD as RISCVInstruction>::RAMAccess) {
        // PoC only (`fp128-forgery-poc`): fabricate the one cycle whose lookup
        // address the prover aliases, so the register file agrees with the
        // corrupted lookup output. See `super::fp128_forgery`.
        #[cfg(feature = "fp128-forgery-poc")]
        {
            let x = cpu.x[self.operands.rs1 as usize] as u64;
            let y = cpu.x[self.operands.rs2 as usize] as u64;
            if super::fp128_forgery::is_target(x, y) {
                super::fp128_forgery::note_forged_cycle();
                cpu.write_register(
                    self.operands.rd as usize,
                    cpu.sign_extend(super::fp128_forgery::forged_output(x, y) as i64),
                );
                return;
            }
        }

        cpu.write_register(
            self.operands.rd as usize,
            cpu.sign_extend(
                cpu.x[self.operands.rs1 as usize].wrapping_add(cpu.x[self.operands.rs2 as usize]),
            ),
        );
    }
}

impl RISCVTrace for ADD {}
