#!/bin/bash

# Build minimal libc for Jolt ZKVM
# This creates a static library that provides the bare minimum symbols

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/lib"
INCLUDE_DIR="$SCRIPT_DIR/include"

# Create output directories
mkdir -p "$OUTPUT_DIR"
mkdir -p "$INCLUDE_DIR"

# Build the library
echo "Building jolt-libc..."
riscv64-unknown-elf-gcc -c \
    -march=rv64imac \
    -mabi=lp64 \
    -mcmodel=medany \
    -fno-builtin \
    -nostdlib \
    -O2 \
    -g \
    "$SCRIPT_DIR/jolt_libc.c" \
    -o "$OUTPUT_DIR/jolt_libc.o"

echo "Building unwind stubs..."
rustc --target=riscv64imac-unknown-none-elf \
    --crate-type=staticlib \
    --emit=obj \
    -C opt-level=z \
    -C panic=abort \
    "$SCRIPT_DIR/unwind_stubs.rs" \
    -o "$OUTPUT_DIR/unwind_stubs.o"

# Create static libraries
riscv64-unknown-elf-ar rcs "$OUTPUT_DIR/libc.a" "$OUTPUT_DIR/jolt_libc.o"
riscv64-unknown-elf-ar rcs "$OUTPUT_DIR/libunwind.a" "$OUTPUT_DIR/unwind_stubs.o"

# Create minimal headers
mkdir -p "$INCLUDE_DIR/sys"
cat > "$INCLUDE_DIR/time.h" << 'EOF'
#ifndef _TIME_H
#define _TIME_H

typedef long time_t;
typedef int clockid_t;

time_t time(time_t* tloc);
int gettimeofday(void* tv, void* tz);

#endif
EOF

cat > "$INCLUDE_DIR/sys/ioctl.h" << 'EOF'
#ifndef _SYS_IOCTL_H
#define _SYS_IOCTL_H

typedef unsigned long ioctl_t;
#define _IOC_NRBITS     8
#define _IOC_TYPEBITS   8

#define _IOR(type,nr,size) (0)
#define _IOW(type,nr,size) (0)

#endif
EOF

echo "jolt-libc built successfully!"
echo "Library: $OUTPUT_DIR/libc.a"
echo "Unwind Library: $OUTPUT_DIR/libunwind.a"
echo "Headers: $INCLUDE_DIR/"