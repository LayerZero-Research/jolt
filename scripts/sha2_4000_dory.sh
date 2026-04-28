#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

RUNS=5
DIR="/tmp/sha2_4000_dory"
rm -rf "$DIR"
mkdir -p "$DIR"

export RUST_LOG=info
export JOLT_LAYOUT=CM

echo "sha2-4000 F mode, $RUNS trials, capturing per-stage timing"
echo ""

for i in $(seq 1 $RUNS); do
    echo -n "  Run $i: "
    OUT=$(cargo run --release -p sha2-chain -- --iters 4000 2>&1)
    
    echo "$OUT" | grep "Prover runtime" | sed 's/.*Prover runtime: //' | sed 's/ s//' >> "$DIR/prove.txt"
    echo "$OUT" | grep "STAGE_TIMES" | sed 's/.*stages_1_6a=//' | sed 's/ms.*//' >> "$DIR/s16a.txt"
    echo "$OUT" | grep "STAGE_TIMES" | sed 's/.*stage_6b=//' | sed 's/ms.*//' >> "$DIR/s6b.txt"
    echo "$OUT" | grep "STAGE_TIMES" | sed 's/.*stage_7=//' | sed 's/ms.*//' >> "$DIR/s7.txt"
    echo "$OUT" | grep "STAGE_TIMES" | sed 's/.*stage_8=//' | sed 's/ms.*//' >> "$DIR/s8.txt"
    
    PROVE=$(tail -1 "$DIR/prove.txt")
    S8=$(tail -1 "$DIR/s8.txt")
    echo "prove=${PROVE}s stage8=${S8}ms"
done

echo ""
echo "Results (trimmed mean, drop fastest+slowest):"
python3 << 'PYEOF'
import os

DIR = "/tmp/sha2_4000_dory"

def load(name, scale=1.0):
    path = os.path.join(DIR, name + ".txt")
    if not os.path.exists(path):
        return 0.0
    with open(path) as f:
        vals = sorted(float(l.strip()) * scale for l in f if l.strip())
    trimmed = vals[1:-1] if len(vals) >= 3 else vals
    return sum(trimmed) / len(trimmed)

prove_ms = load("prove", 1000)
s16a = load("s16a")
s6b = load("s6b")
s7 = load("s7")
s8 = load("s8")
other = prove_ms - s16a - s6b - s7 - s8

print(f"{'Component':<35} {'Time (ms)':>10} {'Time (s)':>10} {'% of total':>12}")
print("=" * 70)
print(f"{'Total prove':<35} {prove_ms:>10.1f} {prove_ms/1000:>10.3f} {'100.0%':>12}")
print(f"{'  Witness gen + commit':<35} {other:>10.1f} {other/1000:>10.3f} {other/prove_ms*100:>11.1f}%")
print(f"{'  Stages 1-6a (sumchecks)':<35} {s16a:>10.1f} {s16a/1000:>10.3f} {s16a/prove_ms*100:>11.1f}%")
print(f"{'  Stage 6b (claim reduction)':<35} {s6b:>10.1f} {s6b/1000:>10.3f} {s6b/prove_ms*100:>11.1f}%")
print(f"{'  Stage 7 (hamming weight)':<35} {s7:>10.1f} {s7/1000:>10.3f} {s7/prove_ms*100:>11.1f}%")
print(f"{'  Stage 8 (Dory opening proof)':<35} {s8:>10.1f} {s8/1000:>10.3f} {s8/prove_ms*100:>11.1f}%")
print("=" * 70)
PYEOF
