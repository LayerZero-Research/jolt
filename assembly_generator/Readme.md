# mem_ctr: Memory Operation Counter for RISC-V Hash Functions

Benchmark and inspect load/store usage of various hash-function implementations on RISC-V (32-bit & 64-bit) targets.

---

## Table of Contents
1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Hash Function Selection](#hash-function-selection)
4. [Automated Script Usage](#automated-script-usage)
5. [Manual Building](#manual-building)
6. [Generating the Assembly Listing](#generating-the-assembly-listing)
7. [Counting Memory Operations](#counting-memory-operations)
8. [Project Layout](#project-layout)
9. [License](#license)

---

## Overview
This project provides a tiny harness that compiles a hash-function implementation to bare-metal RISC-V instructions.

---

## Prerequisites
The tool-chain requirements are fairly small and entirely Rust-based:

```bash
# 1. Install the required targets (32-bit & 64-bit RISC-V bare metal)
rustup target add riscv32im-unknown-none-elf
rustup target add riscv64gc-unknown-none-elf

# 2. Install the LLVM tools that match your Rust version
rustup component add llvm-tools-preview

# 3. Install the handy Cargo wrappers for objdump/size/etc.
cargo install cargo-binutils
```

> **Tip** All commands can be executed from the repository root.  The examples below assume a *bash/zsh* shell.

---

## Hash Function Selection
The project supports SHA2, SHA3, and inline SHA2 hash functions through compilation features. **Exactly one** must be enabled:

### Available Features:
- `sha2_hasher` - Use SHA-256 (SHA2 family)
- `sha3_hasher` - Use SHA3-256 (SHA3 family)
- `inline_sha2_hasher` - Use inline SHA2-256 implementation [**DEFAULT**]

### Supported Combinations:
- **Default + SHA3**: Both RV32 and RV64
- **Default + SHA2**: Both RV32 and RV64  
- **Inline + SHA2**: RV32 only

---

## Automated Script Usage
The project includes a `build_and_analyze.sh` script that automates the build and analysis process for different configurations.

### Usage:
```bash
./build_and_analyze.sh <hash_function> <architecture> <implementation>
```

### Arguments:
- `hash_function`: `sha2`, `sha3`
- `architecture`: `rv32`, `rv64`
- `implementation`: `default`, `inline`

### Examples:
```bash
# Default SHA2 on RV32
./build_and_analyze.sh sha2 rv32 default

# Default SHA3 on RV64
./build_and_analyze.sh sha3 rv64 default

# Inline SHA2 on RV32
./build_and_analyze.sh sha2 rv32 inline
```

The script will:
1. Build the project with the specified configuration
2. Generate assembly listings in the `assembly_results/` directory
3. Provide commands for analyzing memory operations
4. Display build information and file statistics

---

## Manual Building
Alternatively, you can build manually for specific configurations:

### 32-bit RISC-V Examples:
```bash
# Default build (inline SHA2-256)
cargo build --release --target riscv32im-unknown-none-elf

# SHA2-256 build
cargo build --release --target riscv32im-unknown-none-elf --features sha2_hasher --no-default-features

# SHA3-256 build
cargo build --release --target riscv32im-unknown-none-elf --features sha3_hasher --no-default-features
```

### 64-bit RISC-V Examples:
```bash
# SHA2-256 build
cargo build --release --target riscv64gc-unknown-none-elf --features sha2_hasher --no-default-features

# SHA3-256 build
cargo build --release --target riscv64gc-unknown-none-elf --features sha3_hasher --no-default-features
```

The resulting ELF binaries are stored at:
- RV32: `target/riscv32im-unknown-none-elf/release/inline_benchmark`
- RV64: `target/riscv64gc-unknown-none-elf/release/inline_benchmark`

---

## Generating the Assembly Listing
### Using the Automated Script
The script automatically generates assembly listings. Output files are named as:
`assembly_results/mem_ctr_<implementation>_<hash_function>_<architecture>.asm`

### Manual Generation
To manually generate assembly listings:

#### 32-bit RISC-V:
```bash
# For SHA3-256
cargo objdump --release --target riscv32im-unknown-none-elf --features sha3_hasher --no-default-features \
             -- -d --print-imm-hex \
             > assembly_results/mem_ctr_default_sha3_rv32.asm

# For SHA2-256
cargo objdump --release --target riscv32im-unknown-none-elf --features sha2_hasher --no-default-features \
             -- -d --print-imm-hex \
             > assembly_results/mem_ctr_default_sha2_rv32.asm

# For inline SHA2-256 (default)
cargo objdump --release --target riscv32im-unknown-none-elf \
             -- -d --print-imm-hex \
             > assembly_results/mem_ctr_inline_sha2_rv32.asm
```

#### 64-bit RISC-V:
```bash
# For SHA3-256
cargo objdump --release --target riscv64gc-unknown-none-elf --features sha3_hasher --no-default-features \
             -- -d --print-imm-hex \
             > assembly_results/mem_ctr_default_sha3_rv64.asm

# For SHA2-256
cargo objdump --release --target riscv64gc-unknown-none-elf --features sha2_hasher --no-default-features \
             -- -d --print-imm-hex \
             > assembly_results/mem_ctr_default_sha2_rv64.asm
```

Flag overview:
* `-d` – disassemble **all** text sections.
* `--print-imm-hex` – print immediates in hexadecimal (handy for addresses).

---

## Counting Memory Operations
To count memory operations in the generated assembly files, use the following pattern:

```bash
# Generic pattern
grep -E '\s(lb|lh|lw|lbu|lhu|sb|sh|sw)\s' <assembly_file> | wc -l
```

### Examples:
```bash
# Count memory operations in inline SHA2 RV32
grep -E '\s(lb|lh|lw|lbu|lhu|sb|sh|sw)\s' assembly_results/mem_ctr_inline_sha2_rv32.asm | wc -l

# Count memory operations in default SHA3 RV64
grep -E '\s(lb|lh|lw|lbu|lhu|sb|sh|sw)\s' assembly_results/mem_ctr_default_sha3_rv64.asm | wc -l
```

The automated script also provides the appropriate command after generating each assembly file.

The regular expression matches the canonical RISC-V load (`lb`, `lh`, `lw`, `lbu`, `lhu`) and store (`sb`, `sh`, `sw`) mnemonics. Feel free to tweak the regex when analyzing other algorithms or optimization experiments.

---