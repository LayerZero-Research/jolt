#ifndef SHA2_FFI_H
#define SHA2_FFI_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * C-compatible representation of a RISC-V instruction
 */
typedef struct CInstruction {
    /** Instruction type ID (0-255) */
    uint8_t type_id;
    /** Memory address of instruction */
    uint64_t address;
    /** Register indices: rs1, rs2, rd */
    uint64_t rs1;
    uint64_t rs2;
    uint64_t rd;
    /** Immediate value (for I, S, B, U, J formats) */
    int64_t immediate;
    /** Virtual sequence remaining (0 if none) */
    uint64_t virtual_seq_remaining;
} CInstruction;

/**
 * Container for instruction array returned from Rust
 */
typedef struct CInstructionArray {
    /** Pointer to array of CInstruction */
    CInstruction* data;
    /** Number of instructions in the array */
    size_t length;
    /** Capacity of the allocated array (for memory management) */
    size_t capacity;
} CInstructionArray;

/**
 * Alternative wrapper matching the original function signature
 * Always uses custom IV from rs2 (use_initial_iv = false)
 * 
 * @param address Base address for the instruction sequence
 * @param rs1 Source register 1 index
 * @param rs2 Source register 2 index
 * @return Array of instructions. Must be freed with sha2_free_instructions()
 */
CInstructionArray sha2_virtual_sequence_builder_ffi(
    uint64_t address,
    size_t rs1,
    size_t rs2
);

/**
 * Free memory allocated by sha2_build_virtual_sequence
 * 
 * @param array The instruction array to free
 * 
 * IMPORTANT: After calling this function, the array and all its data become invalid
 */
void sha2_free_instructions(CInstructionArray array);



#ifdef __cplusplus
}
#endif

#endif /* SHA2_FFI_H */