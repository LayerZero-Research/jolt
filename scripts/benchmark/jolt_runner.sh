#!/usr/bin/env bash
# Usage: ./jolt_runner.sh [MAX_TRACE_LENGTH] [MIN_TRACE_LENGTH]
set -euox pipefail

# Mitigate stack overflow
export RUST_MIN_STACK=33554432

# Find the project root by looking for Cargo.toml
find_project_root() {
    local current_dir="$PWD"
    while [[ "$current_dir" != "/" ]]; do
        if [[ -f "$current_dir/Cargo.toml" ]] && [[ -f "$current_dir/rust-toolchain.toml" ]]; then
            echo "$current_dir"
            return 0
        fi
        current_dir="$(dirname "$current_dir")"
    done
    echo "Error: Could not find project root (looking for Cargo.toml and rust-toolchain.toml)" >&2
    return 1
}

# Change to project root to ensure all relative paths work correctly
PROJECT_ROOT=$(find_project_root)
if [[ -z "$PROJECT_ROOT" ]]; then
    exit 1
fi
cd "$PROJECT_ROOT"

MIN_TRACE_LENGTH=${2:-21}
MAX_TRACE_LENGTH=${1:-27}
BENCH_LIST="fibonacci sha2-chain sha3-chain btreemap"
echo "Running benchmarks with TRACE_LENGTH=$MIN_TRACE_LENGTH..$MAX_TRACE_LENGTH"

# if [ -d "perfetto_traces" ]; then
#     echo "Clearing existing perfetto_traces directory..."
#     rm -rf perfetto_traces
# fi

mkdir -p perfetto_traces

# Detect GNU time (gtime on macOS, /usr/bin/time GNU on Linux)
TIME_CMD=()
TIME_WITH_GB_CONVERSION=false
if command -v gtime >/dev/null 2>&1; then
    if gtime --version >/dev/null 2>&1; then
        TIME_CMD=(gtime -f "%E,%M,%P")
        TIME_WITH_GB_CONVERSION=true
    fi
elif [[ -x /usr/bin/time ]] && /usr/bin/time --version 2>&1 | grep -qi 'GNU'; then
    TIME_CMD=(/usr/bin/time -f "%E,%M,%P")
    TIME_WITH_GB_CONVERSION=true
fi

if [ ! -f "perfetto_traces/timings.csv" ]; then
    echo "benchmark_name,scale,prover_time_s,trace_length,proving_hz" >perfetto_traces/timings.csv
fi

# Build once, then run the binary for all iterations
if [ "${SKIP_BUILD:-0}" != "1" ]; then
    echo "Building jolt-core (release)..."
    cargo build --release -p jolt-core
fi
JOLT_BIN="./target/release/jolt-core"
if [ ! -x "$JOLT_BIN" ]; then
    echo "Error: built binary not found at $JOLT_BIN"
    exit 1
fi

# Clean up existing SRM files
rm *.srs || true

for scale in $(seq $MIN_TRACE_LENGTH $MAX_TRACE_LENGTH); do
    echo ">>> Running benchmarks at scale 2^$scale"
    
    for bench in $BENCH_LIST; do
        echo "> $bench at scale 2^$scale"
        if [ ${#TIME_CMD[@]} -gt 0 ] && [ "$TIME_WITH_GB_CONVERSION" = true ]; then
            # Capture time output to convert KB to GB
            TIME_OUTPUT_FILE=$(mktemp)
            "${TIME_CMD[@]}" -o "$TIME_OUTPUT_FILE" "$JOLT_BIN" benchmark --name "$bench" --scale "$scale" --format chrome
            
            # Parse and display with GB conversion
            if [[ -f "$TIME_OUTPUT_FILE" ]]; then
                IFS=, read -r elapsed_time max_rss_kb cpu_pct < "$TIME_OUTPUT_FILE"
                max_rss_gb=$(python3 -c "print(f'{float('$max_rss_kb') / 1024 / 1024:.2f}')")
                echo "Elapsed: ${elapsed_time} | Max RSS: ${max_rss_gb} GB | CPU: ${cpu_pct}"
                rm -f "$TIME_OUTPUT_FILE"
            fi
        elif [ ${#TIME_CMD[@]} -gt 0 ]; then
            "${TIME_CMD[@]}" "$JOLT_BIN" benchmark --name "$bench" --scale "$scale" --format chrome
        else
            "$JOLT_BIN" benchmark --name "$bench" --scale "$scale" --format chrome
        fi
    done
done