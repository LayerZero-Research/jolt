#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

RUNS=10
DIR="/tmp/table_v_full"
rm -rf "$DIR"
mkdir -p "$DIR"

run_n() {
    local label="$1"; shift
    local prove_file="$DIR/${label}_prove.txt"
    local s16a_file="$DIR/${label}_s16a.txt"
    local s6b_file="$DIR/${label}_s6b.txt"
    local s7_file="$DIR/${label}_s7.txt"
    local s8_file="$DIR/${label}_s8.txt"
    echo -n "  $label: "
    for i in $(seq 1 $RUNS); do
        local out
        out=$("$@" 2>&1)
        echo "$out" | grep "Prover runtime" | sed 's/.*Prover runtime: //' | sed 's/ s//' >> "$prove_file"
        echo "$out" | grep "STAGE_TIMES" | sed 's/.*stages_1_6a=//' | sed 's/ms.*//' >> "$s16a_file"
        echo "$out" | grep "STAGE_TIMES" | sed 's/.*stage_6b=//' | sed 's/ms.*//' >> "$s6b_file"
        echo "$out" | grep "STAGE_TIMES" | sed 's/.*stage_7=//' | sed 's/ms.*//' >> "$s7_file"
        echo "$out" | grep "STAGE_TIMES" | sed 's/.*stage_8=//' | sed 's/ms.*//' >> "$s8_file"
        echo -n "$i "
    done
    echo ""
}

FIB=(cargo run --release -p fibonacci --)
SHA=(cargo run --release -p sha2-chain -- --iters 1)

export RUST_LOG=info

echo "Full Table V benchmark ($RUNS trials each, per-stage timing)"
echo ""

for LAY in CM AM; do
    export JOLT_LAYOUT=$LAY

    echo "=== Layout: $LAY ==="

    unset JOLT_NAIVE_PAD 2>/dev/null || true
    run_n "fib_${LAY}_F"     "${FIB[@]}"
    run_n "fib_${LAY}_C1"    "${FIB[@]}" --committed-bytecode 1
    run_n "fib_${LAY}_C16"   "${FIB[@]}" --committed-bytecode 16

    export JOLT_NAIVE_PAD=1
    run_n "fib_${LAY}_C1_naive"   "${FIB[@]}" --committed-bytecode 1
    run_n "fib_${LAY}_C16_naive"  "${FIB[@]}" --committed-bytecode 16
    unset JOLT_NAIVE_PAD

    run_n "sha2_${LAY}_F"     "${SHA[@]}"
    run_n "sha2_${LAY}_C1"    "${SHA[@]}" --committed-bytecode 1
    run_n "sha2_${LAY}_C16"   "${SHA[@]}" --committed-bytecode 16

    export JOLT_NAIVE_PAD=1
    run_n "sha2_${LAY}_C1_naive"   "${SHA[@]}" --committed-bytecode 1
    run_n "sha2_${LAY}_C16_naive"  "${SHA[@]}" --committed-bytecode 16
    unset JOLT_NAIVE_PAD

    echo ""
done

echo "Computing results..."
python3 << 'PYEOF'
import os

DIR = "/tmp/table_v_full"

def load(suffix):
    data = {}
    for fname in sorted(os.listdir(DIR)):
        if not fname.endswith(f'_{suffix}.txt'): continue
        label = fname.replace(f'_{suffix}.txt', '')
        with open(os.path.join(DIR, fname)) as f:
            times = sorted(float(l.strip()) for l in f if l.strip())
        trimmed = times[1:-1]
        data[label] = sum(trimmed) / len(trimmed)
    return data

prove = load('prove')
s16a = load('s16a')
s6b = load('s6b')
s7 = load('s7')
s8 = load('s8')

def g(d, label):
    return d.get(label, 0.0)

def pct(a, b):
    return (a / b - 1.0) * 100.0 if b else 0.0

def fmt_pct(v):
    return f'{v:+.1f}%'

paper_a = {
    ('fib','CM','C1'):  (80.8, -1.7, 138.8, 4.4, 157.1),
    ('fib','CM','C16'): (19.5, -0.5, 56.6, 6.2, 37.0),
    ('fib','AM','C1'):  (214.4, 1.9, 142.0, 5.7, 196.1),
    ('fib','AM','C16'): (42.7, -0.7, 61.1, 5.7, 48.3),
    ('sha2','CM','C1'):  (88.4, -0.7, 374.2, 8.1, 169.3),
    ('sha2','CM','C16'): (19.0, 3.7, 134.2, 14.0, 35.5),
    ('sha2','AM','C1'):  (303.7, 3.4, 378.9, 2.7, 266.8),
    ('sha2','AM','C16'): (63.8, 5.2, 149.4, 5.9, 60.5),
}

paper_b = {
    ('fib','CM','C1'):  (-27.6, -42.6, -35.8, -24.1, -0.7),
    ('fib','CM','C16'): (-9.4, -18.9, -1.6, -6.6, -0.3),
    ('fib','AM','C1'):  (-14.1, -41.0, -38.1, -24.5, -10.5),
    ('fib','AM','C16'): (-5.9, -20.1, -8.4, -9.4, -1.2),
    ('sha2','CM','C1'):  (-34.0, -51.2, -36.7, -40.2, -0.1),
    ('sha2','CM','C16'): (-6.2, -4.9, -5.4, -3.4, 1.2),
    ('sha2','AM','C1'):  (-16.0, -48.4, -41.3, -40.7, -12.3),
    ('sha2','AM','C16'): (0.7, -2.5, -8.7, -11.5, -0.2),
}

W = 140

print()
print('=' * W)
print('Panel (a): overhead of committed mode relative to F (same layout)')
print('=' * W)
h = f'{"Prog":<6} {"Lay":<4} {"Md":<4}  {"Total prove":>12} {"S1-6a":>10} {"S6b":>10} {"S7":>10} {"S8":>10}   {"F(s)":>6} {"Md(s)":>6}'
print(h)
print('-' * W)

for prog in ['fib', 'sha2']:
    for lay in ['CM', 'AM']:
        fk = f'{prog}_{lay}_F'
        for mode in ['C1', 'C16']:
            mk = f'{prog}_{lay}_{mode}'
            tp = pct(g(prove, mk), g(prove, fk))
            t16a = pct(g(s16a, mk), g(s16a, fk))
            t6b = pct(g(s6b, mk), g(s6b, fk))
            t7 = pct(g(s7, mk), g(s7, fk))
            t8 = pct(g(s8, mk), g(s8, fk))
            p = paper_a.get((prog, lay, mode))
            ps = f'  paper: {p[0]:+.1f}% {p[1]:+.1f}% {p[2]:+.1f}% {p[3]:+.1f}% {p[4]:+.1f}%' if p else ''
            print(f'{prog:<6} {lay:<4} {mode:<4}  {fmt_pct(tp):>12} {fmt_pct(t16a):>10} {fmt_pct(t6b):>10} {fmt_pct(t7):>10} {fmt_pct(t8):>10}   {g(prove,fk):>6.3f} {g(prove,mk):>6.3f}{ps}')

print()
print('=' * W)
print('Panel (b): savings of precommitted layout relative to naive trace padding')
print('=' * W)
h = f'{"Prog":<6} {"Lay":<4} {"Md":<4}  {"Total prove":>12} {"S1-6a":>10} {"S6b":>10} {"S7":>10} {"S8":>10}   {"Ours(s)":>7} {"Nv(s)":>7}'
print(h)
print('-' * W)

for prog in ['fib', 'sha2']:
    for lay in ['CM', 'AM']:
        for mode in ['C1', 'C16']:
            mk = f'{prog}_{lay}_{mode}'
            nk = f'{prog}_{lay}_{mode}_naive'
            tp = pct(g(prove, mk), g(prove, nk))
            t16a = pct(g(s16a, mk), g(s16a, nk))
            t6b = pct(g(s6b, mk), g(s6b, nk))
            t7 = pct(g(s7, mk), g(s7, nk))
            t8 = pct(g(s8, mk), g(s8, nk))
            p = paper_b.get((prog, lay, mode))
            ps = f'  paper: {p[0]:+.1f}% {p[1]:+.1f}% {p[2]:+.1f}% {p[3]:+.1f}% {p[4]:+.1f}%' if p else ''
            print(f'{prog:<6} {lay:<4} {mode:<4}  {fmt_pct(tp):>12} {fmt_pct(t16a):>10} {fmt_pct(t6b):>10} {fmt_pct(t7):>10} {fmt_pct(t8):>10}   {g(prove,mk):>7.3f} {g(prove,nk):>7.3f}{ps}')

print('=' * W)
PYEOF
