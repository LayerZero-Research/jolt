#!/bin/bash

# Script to count RISC-V A (Atomic) and C (Compressed) extension instructions in guest binaries
# Usage: ./count_riscv_instructions.sh [TARGET_DIR]
# Default TARGET_DIR: /tmp/jolt-guest-targets

set -euo pipefail

# Change to project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Default target directory
TARGET_DIR="${1:-/tmp/jolt-guest-targets}"

echo "Analyzing RISC-V A (Atomic) and C (Compressed) extension instructions in: $TARGET_DIR"

# Check if target directory exists
if [ ! -d "$TARGET_DIR" ]; then
    echo "Error: Target directory '$TARGET_DIR' does not exist"
    echo "Please ensure guest binaries have been built and are available in the target directory"
    exit 1
fi

# Function to count C extension instructions (16-bit compressed instructions)
count_c_extension_instructions() {
    local binary="$1"
    local count=$(riscv64-unknown-elf-objdump -d "$binary" | awk '$1 ~ /^[0-9a-f]+:$/ && $2 ~ /^[0-9a-f]{4}$/ {count++} END {print count+0}')
    echo "$count"
}

# Function to count A extension instructions (atomic instructions)
count_a_extension_instructions() {
    local binary="$1"
    local atomic_count=$(riscv64-unknown-elf-objdump -d "$binary" | grep -E "(amoadd|amoswap|amoor|amoand|amomin|amomax|amoxor|lr\.|sc\.)" | wc -l)
    echo "$atomic_count"
}

# Find all guest binaries and analyze them
found_binaries=false
while IFS= read -r -d '' binary; do
    found_binaries=true
    echo
    echo "Binary: $binary"
    echo "----------------------------------------"
    
    # Count C extension instructions (compressed)
    c_ext_count=$(count_c_extension_instructions "$binary")
    c_ext_count=$(echo "$c_ext_count" | tr -d '\n\r ')
    echo "C Extension (Compressed) instructions: $c_ext_count"
    
    # Count A extension instructions (atomic)
    a_ext_count=$(count_a_extension_instructions "$binary")
    a_ext_count=$(echo "$a_ext_count" | tr -d '\n\r ')
    echo "A Extension (Atomic) instructions: $a_ext_count"
    
    # Show compressed instruction details if any found
    if [ "$c_ext_count" -gt 0 ]; then
        echo
        echo "C Extension (Compressed) instruction details:"
        riscv64-unknown-elf-objdump -d "$binary" 2>/dev/null | awk '$1 ~ /^[0-9a-f]+:$/ && $2 ~ /^[0-9a-f]{4}$/ {print}' | head -10 2>/dev/null || true
        if [ "$c_ext_count" -gt 10 ]; then
            echo "... and $((c_ext_count - 10)) more compressed instructions"
        fi
    fi
    
    # Show atomic instruction details if any found
    if [ "$a_ext_count" -gt 0 ]; then
        echo
        echo "A Extension (Atomic) instruction details:"
        riscv64-unknown-elf-objdump -d "$binary" 2>/dev/null | grep -E "(amoadd|amoswap|amoor|amoand|amomin|amomax|amoxor|lr\.|sc\.)" | head -10 2>/dev/null || true
        if [ "$a_ext_count" -gt 10 ]; then
            echo "... and $((a_ext_count - 10)) more atomic instructions"
        fi
    fi
    
done < <(find "$TARGET_DIR" -type f -path '*/release/*-guest' -executable -print0)

if [ "$found_binaries" = false ]; then
    echo "No guest binaries found in $TARGET_DIR"
    echo "Expected path pattern: */release/*-guest"
fi
