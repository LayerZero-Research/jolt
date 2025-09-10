#!/usr/bin/env bash
# Usage: ./jolt_runner.sh [MAX_TRACE_LENGTH] [MIN_TRACE_LENGTH] [--resume]
set -euo pipefail

MIN_TRACE_LENGTH=${2:-21}
MAX_TRACE_LENGTH=${1:-27}
RESUME_MODE=false
BENCH_LIST="fibonacci sha2-chain sha3-chain btreemap"

# Check for --resume flag
for arg in "$@"; do
    if [[ "$arg" == "--resume" ]]; then
        RESUME_MODE=true
        break
    fi
done

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

echo "Running benchmarks with TRACE_LENGTH=$MIN_TRACE_LENGTH..$MAX_TRACE_LENGTH"

# if [ -d "perfetto_traces" ]; then
#     echo "Clearing existing perfetto_traces directory..."
#     rm -rf perfetto_traces
# fi

mkdir -p benchmark-runs/perfetto_traces benchmark-runs/results

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

if [ ! -f "benchmark-runs/results/timings.csv" ]; then
    echo "benchmark_name,scale,prover_time_s,trace_length,proving_hz" >benchmark-runs/results/timings.csv
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

# Clean up existing SRS files (skip if resuming)
if [ "$RESUME_MODE" = false ]; then
    echo "Cleaning up existing .srs files..."
    rm -f *.srs
else
    echo "Resume mode: preserving existing .srs files"
fi

for scale in $(seq $MIN_TRACE_LENGTH $MAX_TRACE_LENGTH); do
    echo ">>> Running benchmarks at scale 2^$scale"
    
    for bench in $BENCH_LIST; do
        # Check if this benchmark is already completed (resume logic)
        result_file="benchmark-runs/results/${bench}_${scale}.csv"
        if [ "$RESUME_MODE" = true ] && [ -f "$result_file" ]; then
            echo "> $bench at scale 2^$scale: SKIPPED (already completed)"
            continue
        fi
        
        echo "> $bench at scale 2^$scale"
        if [ ${#TIME_CMD[@]} -gt 0 ] && [ "$TIME_WITH_GB_CONVERSION" = true ]; then
            # Capture time output to convert KB to GB
            TIME_OUTPUT_FILE=$(mktemp)
            echo "RUST_MIN_STACK=$RUST_MIN_STACK ${TIME_CMD[*]} -o \"$TIME_OUTPUT_FILE\" $JOLT_BIN benchmark --name \"$bench\" --scale \"$scale\" --format chrome"
            "${TIME_CMD[@]}" -o "$TIME_OUTPUT_FILE" "$JOLT_BIN" benchmark --name "$bench" --scale "$scale" --format chrome
            
            # Parse and display with GB conversion
            if [[ -f "$TIME_OUTPUT_FILE" ]]; then
                IFS=, read -r elapsed_time max_rss_kb cpu_pct < "$TIME_OUTPUT_FILE"
                max_rss_gb=$(python3 -c "print(f'{float('$max_rss_kb') / 1024 / 1024:.2f}')")
                
                # Convert M:SS.SS to seconds
                if [[ "$elapsed_time" == *:* ]]; then
                    elapsed_seconds=$(echo "$elapsed_time" | awk -F: '{print $1*60 + $2}')
                    echo "Elapsed: ${elapsed_time} (${elapsed_seconds}s) | Max RSS: ${max_rss_gb} GB | CPU: ${cpu_pct}"
                else
                    echo "Elapsed: ${elapsed_time} | Max RSS: ${max_rss_gb} GB | CPU: ${cpu_pct}"
                fi
                rm -f "$TIME_OUTPUT_FILE"
            fi
        elif [ ${#TIME_CMD[@]} -gt 0 ]; then
            echo "RUST_MIN_STACK=$RUST_MIN_STACK ${TIME_CMD[*]} $JOLT_BIN benchmark --name \"$bench\" --scale \"$scale\" --format chrome"
            "${TIME_CMD[@]}" "$JOLT_BIN" benchmark --name "$bench" --scale "$scale" --format chrome
        else
            echo "RUST_MIN_STACK=$RUST_MIN_STACK $JOLT_BIN benchmark --name \"$bench\" --scale \"$scale\" --format chrome"
            "$JOLT_BIN" benchmark --name "$bench" --scale "$scale" --format chrome
        fi
    done
done

echo "Creating final consolidated results..."
echo "benchmark_name,scale,prover_time_s,trace_length,proving_hz" > benchmark-runs/results/timings.csv
cat benchmark-runs/results/*_*.csv >> benchmark-runs/results/timings.csv || true
echo "Results consolidated in benchmark-runs/results/timings.csv"