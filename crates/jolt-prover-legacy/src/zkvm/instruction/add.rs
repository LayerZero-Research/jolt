use tracer::instruction::{add::ADD, RISCVCycle};

use crate::zkvm::instruction::{InstructionFlags, NUM_INSTRUCTION_FLAGS};
use crate::zkvm::lookup_table::{range_check::RangeCheckTable, LookupTables};

use super::{CircuitFlags, Flags, InstructionLookup, LookupQuery, NUM_CIRCUIT_FLAGS};

impl<const XLEN: usize> InstructionLookup<XLEN> for ADD {
    fn lookup_table(&self) -> Option<LookupTables<XLEN>> {
        Some(RangeCheckTable.into())
    }
}

impl Flags for ADD {
    fn circuit_flags(&self) -> [bool; NUM_CIRCUIT_FLAGS] {
        let mut flags = [false; NUM_CIRCUIT_FLAGS];
        flags[CircuitFlags::AddOperands] = true;
        flags[CircuitFlags::WriteLookupOutputToRD] = true;
        flags[CircuitFlags::VirtualInstruction] = self.virtual_sequence_remaining.is_some();
        flags[CircuitFlags::DoNotUpdateUnexpandedPC] =
            self.virtual_sequence_remaining.unwrap_or(0) != 0;
        flags[CircuitFlags::IsFirstInSequence] = self.is_first_in_sequence;
        flags[CircuitFlags::IsCompressed] = self.is_compressed;
        flags
    }

    fn instruction_flags(&self) -> [bool; NUM_INSTRUCTION_FLAGS] {
        let mut flags = [false; NUM_INSTRUCTION_FLAGS];
        flags[InstructionFlags::LeftOperandIsRs1Value] = true;
        flags[InstructionFlags::RightOperandIsRs2Value] = true;
        flags
    }
}

impl<const XLEN: usize> LookupQuery<XLEN> for RISCVCycle<ADD> {
    fn to_lookup_operands(&self) -> (u64, u128) {
        let (x, y) = LookupQuery::<XLEN>::to_instruction_inputs(self);
        (0, x as u128 + y as u64 as u128)
    }

    fn to_lookup_index(&self) -> u128 {
        let honest = LookupQuery::<XLEN>::to_lookup_operands(self).1;
        // PoC only (`fp128-forgery-poc`): commit the aliased address `s + p`.
        // `to_lookup_operands` stays honest on purpose — `RightLookupOperand`
        // must remain `s` so the R1CS constraint `RightLookupOperand == x + y`
        // still holds. Only the committed *address* moves.
        #[cfg(feature = "fp128-forgery-poc")]
        {
            use tracer::instruction::fp128_forgery;
            let (x, y) = LookupQuery::<XLEN>::to_instruction_inputs(self);
            if fp128_forgery::is_target(x, y as u64) {
                return fp128_forgery::forged_lookup_index(honest);
            }
        }
        honest
    }

    fn to_instruction_inputs(&self) -> (u64, i128) {
        match XLEN {
            #[cfg(test)]
            8 => (
                self.register_state.rs1 as u8 as u64,
                self.register_state.rs2 as u8 as i128,
            ),
            32 => (
                self.register_state.rs1 as u32 as u64,
                self.register_state.rs2 as u32 as i128,
            ),
            64 => (self.register_state.rs1, self.register_state.rs2 as i128),
            _ => panic!("{XLEN}-bit word size is unsupported"),
        }
    }

    fn to_lookup_output(&self) -> u64 {
        let (x, y) = LookupQuery::<XLEN>::to_instruction_inputs(self);
        // PoC only (`fp128-forgery-poc`): the table entry at the aliased address
        // is its low 64 bits. This is what the read leg proves and what the
        // emulator wrote to `rd`, so `RdWrite == LookupOutput` still holds.
        #[cfg(feature = "fp128-forgery-poc")]
        {
            use tracer::instruction::fp128_forgery;
            if fp128_forgery::is_target(x, y as u64) {
                return fp128_forgery::forged_output(x, y as u64);
            }
        }
        match XLEN {
            #[cfg(test)]
            8 => (x as u8).overflowing_add(y as u8).0.into(),
            32 => (x as u32).overflowing_add(y as u32).0.into(),
            64 => x.overflowing_add(y as u64).0,
            _ => panic!("{XLEN}-bit word size is unsupported"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::zkvm::instruction::test::{
        lookup_output_matches_trace_test, materialize_entry_test,
    };

    use super::*;
    use ark_bn254::Fr;

    #[test]
    fn materialize_entry() {
        materialize_entry_test::<Fr, ADD>();
    }

    #[test]
    fn lookup_output_matches_trace() {
        lookup_output_matches_trace_test::<ADD>();
    }
}
