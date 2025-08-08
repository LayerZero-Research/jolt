#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
#include "sha2_ffi.h"

int main() {
    printf("=== SHA-256 Virtual Sequence Builder FFI Example ===\n\n");
    
    // Parameters for the virtual sequence
    uint64_t address = 0x1000;  // Base address
    size_t rs1 = 5;              // Input data register
    size_t rs2 = 6;              // IV/output register
    
    // Build the instruction sequence using custom IV
    printf("Building SHA-256 virtual sequence...\n");
    printf("  Address: 0x%" PRIx64 "\n", address);
    printf("  RS1 (input): x%zu\n", rs1);
    printf("  RS2 (IV/output): x%zu\n", rs2);
    printf("\n");
    
    CInstructionArray instructions = sha2_virtual_sequence_builder_ffi(address, rs1, rs2);
    
    printf("Generated %zu instructions\n\n", instructions.length);
    
    // Display first 10 instructions
    size_t display_count = instructions.length < 10 ? instructions.length : 100;
    printf("=== First %zu Instructions ===\n", display_count);
    
    for (size_t i = 0; i < display_count; i++) {
        const CInstruction* inst = &instructions.data[i];
        printf("[%zu] Type: %u, Address: 0x%" PRIx64 ", ", 
               i, inst->type_id, inst->address);
        printf("rd: x%" PRIu64 ", rs1: x%" PRIu64 ", rs2: x%" PRIu64 ", ", 
               inst->rd, inst->rs1, inst->rs2);
        printf("imm: %" PRId64 ", ", inst->immediate);
        if (inst->virtual_seq_remaining > 0) {
            printf(", remaining: %" PRIu64, inst->virtual_seq_remaining);
        }
        printf("\n");
    }
    
    if (instructions.length > display_count) {
        printf("\n... and %zu more instructions\n", instructions.length - display_count);
    }
       
    // Clean up - IMPORTANT!
    printf("\nFreeing memory...\n");
    sha2_free_instructions(instructions);
    
    printf("\n=== Done ===\n");
    return 0;
}