/// C FFI bindings for SHA-2 virtual sequence builder
use tracer::instruction::RV32IMInstruction;
use crate::trace_generator::sha2_virtual_sequence_builder;

/// C-compatible representation of an instruction
/// This is a simplified version that can be safely passed across FFI boundary
#[repr(C)]
pub struct CInstruction {
    /// Instruction type ID
    pub type_id: u8,
    /// Memory address of instruction
    pub address: u64,
    /// Normalized Operands
    pub rs1: u64,
    pub rs2: u64,
    pub rd: u64,
    pub imm: i64,
    /// Virtual sequence remaining (0 if none)
    pub virtual_seq_remaining: u64,
}

/// Container for returning instruction array to C
#[repr(C)]
pub struct CInstructionArray {
    /// Pointer to array of CInstruction
    pub data: *mut CInstruction,
    /// Number of instructions
    pub length: usize,
    /// Capacity (for memory management)
    pub capacity: usize,
}

impl CInstruction {
    /// Convert from RV32IMInstruction to CInstruction
    pub fn from_rv32im(instr: &RV32IMInstruction) -> Self {
        let normalized = instr.normalize();
        
        CInstruction {
            type_id: RV32IMInstruction::enum_index(instr),
            address: normalized.address as u64,
            rs1: normalized.operands.rs1 as u64,
            rs2: normalized.operands.rs2 as u64,
            rd: normalized.operands.rd as u64,
            imm: normalized.operands.imm,
            virtual_seq_remaining: instr.get_virtual_sequence_remaining().unwrap_or(0) as u64,
        }
    }
}

/// Main C FFI function - builds SHA-256 virtual sequence
/// 
/// # Safety
/// This function allocates memory that must be freed using `sha2_free_instructions`
#[no_mangle]
pub extern "C" fn sha2_virtual_sequence_builder_ffi(
    address: u64,
    rs1: usize,
    rs2: usize,
) -> CInstructionArray {
    let instructions = sha2_virtual_sequence_builder(address, rs1, rs2);
    for i in 0..100 {
        println!("instrucion {}: {:?}", i, instructions[i]);
    }
    println!("Instructions length is: {}", instructions.len());
    
    // Convert to C-compatible format
    let mut c_instructions: Vec<CInstruction> = instructions
        .iter()
        .map(CInstruction::from_rv32im)
        .collect();
    
    let length = c_instructions.len();
    let capacity = c_instructions.capacity();
    let data = c_instructions.as_mut_ptr();
    
    // Prevent Rust from deallocating the memory
    std::mem::forget(c_instructions);
    
    CInstructionArray {
        data,
        length,
        capacity,
    }
}

/// Free memory allocated by sha2_build_virtual_sequence
/// 
/// # Safety
/// - `array` must have been created by `sha2_build_virtual_sequence` or similar
/// - Must only be called once per array
/// - After calling, the array is invalid and must not be used
#[no_mangle]
pub extern "C" fn sha2_free_instructions(array: CInstructionArray) {
    if array.data.is_null() {
        return;
    }
    
    unsafe {
        // Reconstruct the Vec to properly deallocate
        let _ = Vec::from_raw_parts(array.data, array.length, array.capacity);
        // Vec is dropped here, freeing the memory
    }
}
