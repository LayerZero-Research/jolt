#!/bin/bash

# Script to build sha2-ex with different SHA256 implementations
# Usage: ./build_sha2_variants.sh [jolt|sha2_crate]

set -e

if [ $# -eq 0 ]; then
    echo "Usage: $0 [jolt|sha2_crate]"
    echo ""
    echo "Options:"
    echo "  jolt        - Use Jolt's optimized inline SHA256 implementation"
    echo "  sha2_crate  - Use standard sha2 crate implementation"
    exit 1
fi

IMPLEMENTATION=$1

case $IMPLEMENTATION in
    "jolt")
        echo "Building sha2-ex with Jolt's inline SHA256 implementation..."
        cargo build --release --features jolt_sha256 --no-default-features
        ;;
    "sha2_crate")
        echo "Building sha2-ex with sha2 crate implementation..."
        cargo build --release --features sha2_crate --no-default-features
        ;;
    *)
        echo "Error: Unknown implementation '$IMPLEMENTATION'"
        echo "Valid options: jolt, sha2_crate"
        exit 1
        ;;
esac

echo "✅ Build completed successfully with $IMPLEMENTATION implementation"
echo ""
echo "To run with different implementations:"
echo "  For Jolt:      cargo run --release --features jolt_sha256 --no-default-features"
echo "  For sha2 crate: cargo run --release --features sha2_crate --no-default-features" 