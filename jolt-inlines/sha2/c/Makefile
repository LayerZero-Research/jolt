# Makefile for building SHA-2 FFI example

# Compiler settings
CC = gcc
CFLAGS = -Wall -Wextra -O2 -std=c11
LDFLAGS = -L../../../target/release -ljolt_inlines_sha2
INCLUDES = -I.

# Platform-specific settings
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    LDFLAGS += -Wl,-rpath,../../../target/release
    LIB_EXT = so
endif
ifeq ($(UNAME_S),Darwin)
    LDFLAGS += -Wl,-rpath,@loader_path/../../../target/release
    LIB_EXT = dylib
endif

# Build targets
TARGET = example_ffi
RUST_LIB = ../../../target/release/libjolt_inlines_sha2.$(LIB_EXT)

.PHONY: all clean rust-lib run

all: $(TARGET)

# Build Rust library with C FFI support
rust-lib:
	@echo "Building Rust library with FFI support..."
	cd .. && cargo build --release --features host
	@echo "Rust library built: $(RUST_LIB)"

# Build C example
$(TARGET): example_ffi.c sha2_ffi.h rust-lib
	@echo "Building C example..."
	$(CC) $(CFLAGS) $(INCLUDES) example_ffi.c -o $(TARGET) $(LDFLAGS)
	@echo "Build complete: $(TARGET)"

# Run the example
run: $(TARGET)
	@echo "Running FFI example..."
	./$(TARGET)

# Clean build artifacts
clean:
	rm -f $(TARGET)
	@echo "Cleaned C build artifacts"

# Clean everything including Rust artifacts
clean-all: clean
	cd .. && cargo clean
	@echo "Cleaned all build artifacts"

# Help
help:
	@echo "SHA-2 FFI Build System"
	@echo ""
	@echo "Targets:"
	@echo "  make          - Build everything (Rust library and C example)"
	@echo "  make run      - Build and run the example"
	@echo "  make clean    - Clean C build artifacts"
	@echo "  make clean-all - Clean all artifacts (C and Rust)"
	@echo ""
	@echo "The Rust library will be built at: $(RUST_LIB)"
	@echo "The C example will be built as: $(TARGET)"