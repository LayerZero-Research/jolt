# C FFI Guide for SHA-2 Virtual Sequence Builder

This guide explains how to call the SHA-2 virtual sequence builder from C using Foreign Function Interface (FFI).

## Overview

The `sha2_virtual_sequence_builder` function generates a sequence of RISC-V instructions for SHA-256 computation. Through FFI, this Rust function can be called directly from C code, returning a C-compatible array of instructions.

## Architecture

### Data Flow
```
Rust Function                    FFI Layer                      C Code
sha2_virtual_sequence_builder -> CInstructionArray         -> Process instructions
Vec<RV32IMInstruction>        -> CInstruction* + length    -> Free memory
```

### Memory Management
- **Allocation**: Rust allocates memory for the instruction array
- **Ownership**: C code receives ownership of the allocated memory
- **Deallocation**: C code must call `sha2_free_instructions()` to free memory

## Data Structures

### CInstruction
```c
typedef struct CInstruction {
    uint8_t type_id;                  // Instruction type (0-255)
    uint64_t address;                 // Memory address
    uint16_t registers[3];            // [rd, rs1, rs2]
    int64_t immediate;                // Immediate value
    uint16_t virtual_seq_remaining;   // Virtual sequence counter
    uint8_t inline_opcode;            // INLINE-specific
    uint8_t inline_funct3;            // INLINE-specific
    uint8_t inline_funct7;            // INLINE-specific
} CInstruction;
```

### CInstructionArray
```c
typedef struct CInstructionArray {
    CInstruction* data;    // Pointer to instruction array
    size_t length;         // Number of instructions
    size_t capacity;       // Allocated capacity
} CInstructionArray;
```

## API Functions

### Main Functions

#### sha2_virtual_sequence_builder_ffi
```c
CInstructionArray sha2_virtual_sequence_builder_ffi(
    uint64_t address,
    size_t rs1,
    size_t rs2
);
```
Generates SHA-256 instruction sequence using custom IV from rs2.

#### sha2_build_virtual_sequence
```c
CInstructionArray sha2_build_virtual_sequence(
    uint64_t address,
    size_t rs1,
    size_t rs2,
    bool use_initial_iv
);
```
Extended version with control over initial values.

#### sha2_free_instructions
```c
void sha2_free_instructions(CInstructionArray array);
```
**CRITICAL**: Must be called to free memory allocated by Rust.

### Helper Functions

#### sha2_get_instruction
```c
const CInstruction* sha2_get_instruction(
    const CInstructionArray* array,
    size_t index
);
```
Safe accessor for array elements.

#### sha2_instruction_type_to_string
```c
const char* sha2_instruction_type_to_string(uint8_t type_id);
```
Returns human-readable instruction name.

## Build Instructions

### 1. Build Rust Library

Add to `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

Build:
```bash
cargo build --release --features host
```

This creates:
- Linux: `target/release/libjolt_inlines_sha2.so`
- macOS: `target/release/libjolt_inlines_sha2.dylib`
- Windows: `target/release/jolt_inlines_sha2.dll`

### 2. Compile C Code

#### Using Makefile:
```bash
cd c
make        # Build everything
make run    # Build and run example
```

#### Manual compilation:

**Linux:**
```bash
gcc -o example example_ffi.c \
    -L../target/release -ljolt_inlines_sha2 \
    -Wl,-rpath,../target/release
```

**macOS:**
```bash
gcc -o example example_ffi.c \
    -L../target/release -ljolt_inlines_sha2 \
    -Wl,-rpath,@loader_path/../target/release
```

**Windows (MinGW):**
```bash
gcc -o example.exe example_ffi.c \
    -L../target/release -ljolt_inlines_sha2
```

## Usage Example

### Basic Usage
```c
#include "sha2_ffi.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Generate instruction sequence
    CInstructionArray instructions = sha2_virtual_sequence_builder_ffi(
        0x1000,  // address
        5,       // rs1 (input register)
        6        // rs2 (IV/output register)
    );
    
    printf("Generated %zu instructions\n", instructions.length);
    
    // Process instructions
    for (size_t i = 0; i < instructions.length; i++) {
        const CInstruction* inst = sha2_get_instruction(&instructions, i);
        if (inst) {
            printf("Instruction %zu: %s\n", 
                   i, sha2_instruction_type_to_string(inst->type_id));
        }
    }
    
    // IMPORTANT: Free memory
    sha2_free_instructions(instructions);
    
    return 0;
}
```

### Advanced: Processing Specific Instruction Types
```c
void process_instructions(const CInstructionArray* array) {
    for (size_t i = 0; i < array->length; i++) {
        const CInstruction* inst = sha2_get_instruction(array, i);
        if (!inst) continue;
        
        switch (inst->type_id) {
            case INST_ADD:
                printf("ADD x%d, x%d, x%d\n",
                       inst->registers[0],  // rd
                       inst->registers[1],  // rs1
                       inst->registers[2]); // rs2
                break;
                
            case INST_LW:
                printf("LW x%d, %ld(x%d)\n",
                       inst->registers[0],  // rd
                       inst->immediate,     // offset
                       inst->registers[1]); // rs1
                break;
                
            case INST_SW:
                printf("SW x%d, %ld(x%d)\n",
                       inst->registers[2],  // rs2 (value)
                       inst->immediate,     // offset
                       inst->registers[1]); // rs1 (base)
                break;
                
            case INST_INLINE:
                printf("INLINE opcode=%02x funct3=%d funct7=%d\n",
                       inst->inline_opcode,
                       inst->inline_funct3,
                       inst->inline_funct7);
                break;
        }
    }
}
```

## Memory Safety Guidelines

### DO:
- ✅ Always call `sha2_free_instructions()` when done
- ✅ Use `sha2_get_instruction()` for safe array access
- ✅ Check for NULL pointers
- ✅ Keep array alive while using instruction pointers

### DON'T:
- ❌ Free individual instructions
- ❌ Use instruction pointers after freeing array
- ❌ Modify the data pointer directly
- ❌ Call `sha2_free_instructions()` twice on same array

## Error Handling

```c
CInstructionArray result = sha2_virtual_sequence_builder_ffi(addr, rs1, rs2);

if (result.data == NULL || result.length == 0) {
    fprintf(stderr, "Failed to generate instructions\n");
    return -1;
}

// Safe element access
const CInstruction* inst = sha2_get_instruction(&result, index);
if (inst == NULL) {
    fprintf(stderr, "Invalid index: %zu\n", index);
}

// Always free, even on error
sha2_free_instructions(result);
```

## Performance Considerations

1. **Batch Processing**: Process all instructions before freeing the array
2. **Memory Locality**: Instructions are stored contiguously in memory
3. **Zero-Copy**: No copying between Rust and C (direct memory transfer)
4. **Allocation**: Single allocation for entire instruction array

## Troubleshooting

### Common Issues

#### "undefined reference to `sha2_virtual_sequence_builder_ffi`"
- Ensure Rust library is built with `--features host`
- Check library path in linker flags
- Verify library name matches platform convention

#### Segmentation fault
- Ensure `sha2_free_instructions()` is called only once
- Don't use instruction pointers after freeing array
- Check array bounds when accessing elements

#### "cannot find -ljolt_inlines_sha2"
- Build Rust library first: `cargo build --release --features host`
- Check that library exists in `target/release/`
- Use correct library name for platform

### Platform-Specific Notes

**Linux**: 
- May need to set `LD_LIBRARY_PATH`:
  ```bash
  export LD_LIBRARY_PATH=../target/release:$LD_LIBRARY_PATH
  ```

**macOS**:
- May need to adjust `@rpath`:
  ```bash
  install_name_tool -add_rpath ../target/release example
  ```

**Windows**:
- Ensure DLL is in PATH or same directory as executable
- Use `.lib` import library for linking

## Complete Working Example

See `c/example_ffi.c` for a complete working example that demonstrates:
- Building instruction sequences
- Processing different instruction types
- Analyzing instruction distribution
- Proper memory management
- Error handling

## Integration with Existing C Projects

### CMake
```cmake
# Find Rust library
find_library(JOLT_SHA2_LIB 
    NAMES jolt_inlines_sha2
    PATHS ${CMAKE_CURRENT_SOURCE_DIR}/../target/release
)

# Add to target
target_link_libraries(your_target ${JOLT_SHA2_LIB})
```

### pkg-config
Create `jolt-sha2.pc`:
```
prefix=/usr/local
libdir=${prefix}/lib
includedir=${prefix}/include

Name: jolt-sha2
Description: SHA-2 FFI for Jolt
Version: 0.1.0
Libs: -L${libdir} -ljolt_inlines_sha2
Cflags: -I${includedir}
```

## License

This FFI interface follows the same license as the Jolt project.