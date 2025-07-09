# mem_ctr: Memory Operation Counter for RISC-V Hash Functions

Benchmark and inspect load/store usage of various hash-function implementations on RISC-V (32-bit & 64-bit) targets.

---

## Table of Contents
1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Hash Function Selection](#hash-function-selection)
4. [Building](#building)
5. [Generating the Assembly Listing](#generating-the-assembly-listing)
6. [Counting Memory Operations](#counting-memory-operations)
7. [Project Layout](#project-layout)
8. [License](#license)

---

## Overview
This project provides a tiny harness that compiles a hash-function implementation to bare-metal RISC-V and then counts the number of **load** and **store** instructions executed inside its compression function.

---

## Prerequisites
The tool-chain requirements are fairly small and entirely Rust-based:

```bash
# 1. Install the required target (32-bit RISC-V bare metal)
rustup target add riscv32im-unknown-none-elf

# 2. Install the LLVM tools that match your Rust version
rustup component add llvm-tools-preview

# 3. Install the handy Cargo wrappers for objdump/size/etc.
cargo install cargo-binutils
```

> **Tip** All commands can be executed from the repository root.  The examples below assume a *bash/zsh* shell.

---

## Hash Function Selection
The project supports both SHA2 and SHA3 hash functions through compilation features. **Exactly one** must be enabled:

### Available Features:
- `sha2_hasher` - Use SHA-256 (SHA2 family)
- `sha3_hasher` - Use SHA3-256 (SHA3 family) [**DEFAULT**]

### Examples:
```bash
# Use SHA3-256 (default)
cargo build --release --target riscv32im-unknown-none-elf

# Use SHA2-256 explicitly
cargo build --release --target riscv32im-unknown-none-elf --features sha2_hasher --no-default-features

# Use SHA3-256 explicitly
cargo build --release --target riscv32im-unknown-none-elf --features sha3_hasher --no-default-features
```

> **Note**: The build will fail if both features are enabled or if neither is enabled.

---

## Building
Compile the code in **release** mode for the `riscv32im-unknown-none-elf` target:

```bash
# Default build (SHA3-256)
cargo build --release --target riscv32im-unknown-none-elf

# SHA2-256 build
cargo build --release --target riscv32im-unknown-none-elf --features sha2_hasher --no-default-features
```

The resulting ELF binary is stored at:
`mem_ctr/target/riscv32im-unknown-none-elf/release/mem_ctr`

---

## Generating the Assembly Listing
In order to analyse the generated code we disassemble the ELF using `cargo objdump` and redirect the output to a file:

```bash
# For SHA3-256 (default)
cargo objdump --release --target riscv32im-unknown-none-elf \
             -- -d --no-show-raw-insn --print-imm-hex \
             > target/riscv32im-unknown-none-elf/release/mem_ctr_sha3.asm

# For SHA2-256
cargo objdump --release --target riscv32im-unknown-none-elf --features sha2_hasher --no-default-features \
             -- -d --no-show-raw-insn --print-imm-hex \
             > target/riscv32im-unknown-none-elf/release/mem_ctr_sha2.asm
```

Flag overview:
* `-d` – disassemble **all** text sections.
* `--no-show-raw-insn` – hide raw instruction bytes for a cleaner listing.
* `--print-imm-hex` – print immediates in hexadecimal (handy for addresses).

---

## Counting Memory Operations
For the SHA-256 and SHA3-256 the compression routine reside <sha2::sha256::compress256::hb1135b3174355bb5> and <keccak::keccak_p::h5227b0be3b6b58de> of the generated listing, respectively.  To count the memory traffic inside that window you can run:

```bash
# generic pattern ----------------------------------------------------
sed -n '<start>,<end>p' mem_ctr_<algo>.asm | \
  grep -E '\s(lb|lh|lw|lbu|lhu|sb|sh|sw)\s' | wc -l
# -------------------------------------------------------------------
```

### SHA-256 example
The compression function sits roughly in lines *244 – 4492* of the `mem_ctr_sha2.asm` listing:

```bash
sed -n '244,4492p' mem_ctr_sha2.asm | \
  grep -E '\s(lb|lh|lw|lbu|lhu|sb|sh|sw)\s' | wc -l
# → 666 / 4 248 instructions access memory
```

### SHA3-256 example
For the SHA-3 implementation it is lines *249 – 1121* of `mem_ctr_sha3.asm`:

```bash
sed -n '249,1121p' mem_ctr_sha3.asm | \
  grep -E '\s(lb|lh|lw|lbu|lhu|sb|sh|sw)\s' | wc -l
# → 422 / 879 instructions access memory
```

The regular expression matches the canonical RISC-V load (`lb`, `lh`, `lw`, `lbu`, `lhu`) and store (`sb`, `sh`, `sw`) mnemonics. Feel free to tweak the line range or regex when analysing other algorithms or optimisation experiments.

---