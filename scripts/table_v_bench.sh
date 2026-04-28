#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

RUNS=10
RESULTS_DIR="/tmp/table_v_results"
rm -rf "$RESULTS_DIR"
mkdir -p "$RESULTS_DIR"

run_config() {
    local label="$1"
    local cmd="$2"
    local envvars="${3:-}"
    local outfile="$RESULTS_DIR/${label}.txt"

    echo -n "  $label: "
    for i in $(seq 1 $RUNS); do
        local t
        t=$(env $envvars RUST_LOG=info $cmd 2>&1 | grep "Prover runtime" | sed 's/.*Prover runtime: //' | sed 's/ s//')
        echo "$t" >> "$outfile"
        echo -n "$i "
    done
    echo ""
}

echo "Running each config ${RUNS} times..."
echo ""

echo "=== fibonacci ==="
run_config "fib_F"          "cargo run --release -p fibonacci"
run_config "fib_C1"         "cargo run --release -p fibonacci -- --committed-bytecode 1"
run_config "fib_C16"        "cargo run --release -p fibonacci -- --committed-bytecode 16"
run_config "fib_C1_naive"   "cargo run --release -p fibonacci -- --committed-bytecode 1"   "JOLT_NAIVE_PAD=1"
run_config "fib_C16_naive"  "cargo run --release -p fibonacci -- --committed-bytecode 16"  "JOLT_NAIVE_PAD=1"

echo ""
echo "=== sha2-chain (1 iter) ==="
run_config "sha2_F"          "cargo run --release -p sha2-chain -- --iters 1"
run_config "sha2_C1"         "cargo run --release -p sha2-chain -- --iters 1 --committed-bytecode 1"
run_config "sha2_C16"        "cargo run --release -p sha2-chain -- --iters 1 --committed-bytecode 16"
run_config "sha2_C1_naive"   "cargo run --release -p sha2-chain -- --iters 1 --committed-bytecode 1"   "JOLT_NAIVE_PAD=1"
run_config "sha2_C16_naive"  "cargo run --release -p sha2-chain -- --iters 1 --committed-bytecode 16"  "JOLT_NAIVE_PAD=1"

echo ""
echo "=== Raw timings (all $RUNS runs per config) ==="
for f in "$RESULTS_DIR"/*.txt; do
    label=$(basename "$f" .txt)
    printf "%-18s " "$label"
    cat "$f" | tr '\n' ' '
    echo ""
done

echo ""
echo "=== Trimmed mean (drop fastest & slowest, average remaining $((RUNS-2))) ==="
python3 -c "
import os, sys

results_dir = '$RESULTS_DIR'
data = {}
for fname in sorted(os.listdir(results_dir)):
    if not fname.endswith('.txt'):
        continue
    label = fname.replace('.txt', '')
    with open(os.path.join(results_dir, fname)) as f:
        times = [float(line.strip()) for line in f if line.strip()]
    times.sort()
    trimmed = times[1:-1]  # drop fastest and slowest
    avg = sum(trimmed) / len(trimmed)
    data[label] = (avg, times)

# Print raw trimmed means
print(f'{\"Config\":<18} {\"Mean(s)\":>10} {\"Min\":>8} {\"Max\":>8} {\"All runs (sorted)\"}')
print('-' * 90)
for label in sorted(data.keys()):
    avg, times = data[label]
    trimmed = times[1:-1]
    print(f'{label:<18} {avg:>10.4f} {min(trimmed):>8.4f} {max(trimmed):>8.4f}   {\" \".join(f\"{t:.4f}\" for t in times)}')

print()
print('=' * 110)
print('Panel (a): Committed mode overhead vs Full (precommitted layout)')
print('=' * 110)
print(f'{\"Program\":<10} {\"Mode\":<6} {\"F (s)\":>10} {\"Mode (s)\":>10} {\"Prove Δ%\":>12}   Paper Δ%')
print('-' * 110)

paper_a = {
    ('fib','C1'): 80.8, ('fib','C16'): 19.5,
    ('sha2','C1'): 88.4, ('sha2','C16'): 19.0,
}

for prog in ['fib', 'sha2']:
    f_key = f'{prog}_F'
    if f_key not in data:
        continue
    f_avg = data[f_key][0]
    for mode in ['C1', 'C16']:
        m_key = f'{prog}_{mode}'
        if m_key not in data:
            continue
        m_avg = data[m_key][0]
        delta = (m_avg / f_avg - 1.0) * 100.0
        paper = paper_a.get((prog, mode), None)
        paper_str = f'{paper:+.1f}%' if paper else 'N/A'
        print(f'{prog:<10} {mode:<6} {f_avg:>10.4f} {m_avg:>10.4f} {delta:>+11.1f}%   {paper_str}')

print()
print('=' * 110)
print('Panel (b): Precommitted layout savings vs naive trace padding')
print('=' * 110)
print(f'{\"Program\":<10} {\"Mode\":<6} {\"Ours (s)\":>10} {\"Naive (s)\":>10} {\"Savings\":>12}   Paper')
print('-' * 110)

paper_b = {
    ('fib','C1'): -27.6, ('fib','C16'): -9.4,
    ('sha2','C1'): -34.0, ('sha2','C16'): -6.2,
}

for prog in ['fib', 'sha2']:
    for mode in ['C1', 'C16']:
        ours_key = f'{prog}_{mode}'
        naive_key = f'{prog}_{mode}_naive'
        if ours_key not in data or naive_key not in data:
            continue
        ours_avg = data[ours_key][0]
        naive_avg = data[naive_key][0]
        savings = (ours_avg / naive_avg - 1.0) * 100.0
        paper = paper_b.get((prog, mode), None)
        paper_str = f'{paper:+.1f}%' if paper else 'N/A'
        print(f'{prog:<10} {mode:<6} {ours_avg:>10.4f} {naive_avg:>10.4f} {savings:>+11.1f}%   {paper_str}')

print()
print('=' * 110)
"
