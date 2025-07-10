#!/bin/bash

# Script to build and analyze inline benchmark for different configurations
# Usage: ./build_and_analyze.sh <hash_function> <architecture> <implementation>
# 
# Arguments:
#   hash_function: sha2, sha3
#   architecture: rv32, rv64
#   implementation: default, inline
#
# Supported combinations:
#   - default + sha3 + rv32/rv64
#   - default + sha2 + rv32/rv64  
#   - inline + sha2 + rv32

set -e  # Exit on any error

# Function to display usage
usage() {
    echo "Usage: $0 <hash_function> <architecture> <implementation>"
    echo ""
    echo "Arguments:"
    echo "  hash_function: sha2, sha3"
    echo "  architecture: rv32, rv64"
    echo "  implementation: default, inline"
    echo ""
    echo "Supported combinations:"
    echo "  - default + sha3 + rv32/rv64"
    echo "  - default + sha2 + rv32/rv64"
    echo "  - inline + sha2 + rv32"
    echo ""
    echo "Examples:"
    echo "  $0 sha2 rv32 default"
    echo "  $0 sha3 rv64 default"
    echo "  $0 sha2 rv32 inline"
    exit 1
}

# Check if correct number of arguments provided
if [ $# -ne 3 ]; then
    echo "Error: Expected 3 arguments, got $#"
    usage
fi

HASH_FUNCTION=$1
ARCHITECTURE=$2
IMPLEMENTATION=$3

# Validate arguments
case $HASH_FUNCTION in
    sha2|sha3) ;;
    *) echo "Error: Invalid hash function '$HASH_FUNCTION'. Must be 'sha2' or 'sha3'"; usage ;;
esac

case $ARCHITECTURE in
    rv32|rv64) ;;
    *) echo "Error: Invalid architecture '$ARCHITECTURE'. Must be 'rv32' or 'rv64'"; usage ;;
esac

case $IMPLEMENTATION in
    default|inline) ;;
    *) echo "Error: Invalid implementation '$IMPLEMENTATION'. Must be 'default' or 'inline'"; usage ;;
esac

# Validate supported combinations
case "$IMPLEMENTATION-$HASH_FUNCTION-$ARCHITECTURE" in
    default-sha3-rv32|default-sha3-rv64|default-sha2-rv32|default-sha2-rv64|inline-sha2-rv32) ;;
    *) echo "Error: Unsupported combination: $IMPLEMENTATION + $HASH_FUNCTION + $ARCHITECTURE"; usage ;;
esac

# Set target based on architecture
case $ARCHITECTURE in
    rv32) TARGET="riscv32im-unknown-none-elf" ;;
    rv64) TARGET="riscv64gc-unknown-none-elf" ;;
esac

# Set features based on implementation and hash function
case "$IMPLEMENTATION-$HASH_FUNCTION" in
    default-sha2) FEATURES="sha2_hasher" ;;
    default-sha3) FEATURES="sha3_hasher" ;;
    inline-sha2) FEATURES="inline_sha2_hasher" ;;
esac

# Create output directory
OUTPUT_DIR="assembly_results"
mkdir -p "$OUTPUT_DIR"

# Generate output filename
OUTPUT_FILE="$OUTPUT_DIR/mem_ctr_${IMPLEMENTATION}_${HASH_FUNCTION}_${ARCHITECTURE}.asm"

echo "Building configuration:"
echo "  Hash Function: $HASH_FUNCTION"
echo "  Architecture: $ARCHITECTURE ($TARGET)"
echo "  Implementation: $IMPLEMENTATION"
echo "  Features: $FEATURES"
echo "  Output: $OUTPUT_FILE"
echo ""

# Build the project
echo "Building project..."
if [ "$IMPLEMENTATION" == "inline" ] && [ "$HASH_FUNCTION" == "sha2" ]; then
    # For inline SHA2, use the default feature (which is now inline_sha2_hasher)
    cargo build --release --target "$TARGET"
else
    # For default implementations, explicitly set features
    cargo build --release --target "$TARGET" --features "$FEATURES" --no-default-features
fi

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    exit 1
fi

# Generate assembly listing
echo "Generating assembly listing..."
if [ "$IMPLEMENTATION" == "inline" ] && [ "$HASH_FUNCTION" == "sha2" ]; then
    # For inline SHA2, use default features
    cargo objdump --release --target "$TARGET" \
                 -- -d --print-imm-hex \
                 > "$OUTPUT_FILE"
else
    # For default implementations, use explicit features
    cargo objdump --release --target "$TARGET" --features "$FEATURES" --no-default-features \
                 -- -d --print-imm-hex \
                 > "$OUTPUT_FILE"
fi

if [ $? -eq 0 ]; then
    echo "✅ Assembly listing generated: $OUTPUT_FILE"
else
    echo "❌ Assembly generation failed!"
    exit 1
fi

# Display file info
echo ""
echo "Assembly file information:"
echo "  Size: $(du -h "$OUTPUT_FILE" | cut -f1)"
echo "  Lines: $(wc -l < "$OUTPUT_FILE")"
echo ""
echo "✅ Process completed successfully!"
echo ""
echo "To analyze memory operations, you can use:"
echo "  grep -E '\\s(lb|lh|lw|lbu|lhu|sb|sh|sw)\\s' '$OUTPUT_FILE' | wc -l" 