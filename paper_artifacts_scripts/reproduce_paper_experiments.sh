#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

RUNS=10
COMMAND="${1:-}"

usage() {
    cat <<'EOF'
Usage: bash paper_artifacts_scripts/reproduce_paper_experiments.sh <table-v|recursive|all>

Reproduces the experiments from precommitted-geometry-and-dory-embedding.tex:
  - Table V proving-time benchmarks with 10-run trimmed means (paper default).
  - Layout dimension sanity table.
  - Recursive verifier bundle/cycle table.
  - CSV, JSON, Markdown reports and raw logs.

Commands:
  table-v      Run the non-recursive Table V and layout experiments.
  recursive    Run the recursive verifier experiments, including proof generation
               and inner verifier traces.
  all          Run both table-v and recursive experiments.

Environment:
  RUST_LOG is set to info by the script.
  The script builds the required release binaries before running benchmarks.

Examples:
  bash paper_artifacts_scripts/reproduce_paper_experiments.sh table-v
  bash paper_artifacts_scripts/reproduce_paper_experiments.sh recursive
  bash paper_artifacts_scripts/reproduce_paper_experiments.sh all
EOF
}

case "$COMMAND" in
    table-v|recursive|all)
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    "")
        usage >&2
        exit 1
        ;;
    *)
        echo "Unknown command: $COMMAND" >&2
        usage >&2
        exit 1
        ;;
esac

OUT_DIR="/tmp/jolt-paper-experiments/${COMMAND}-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$OUT_DIR"
export RUST_LOG=info

log() {
    printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*"
}

die() {
    echo "ERROR: $*" >&2
    exit 1
}

require_tool() {
    local tool="$1"
    local install_hint="$2"
    if ! command -v "$tool" >/dev/null 2>&1; then
        die "Required tool '$tool' is not installed or not on PATH. ${install_hint}"
    fi
}

check_prerequisites() {
    log "Checking prerequisites"
    require_tool cargo "Install Rust from https://rustup.rs/."
    require_tool rustc "Install Rust from https://rustup.rs/."
    require_tool python3 "Install Python 3 and ensure python3 is on PATH."
    require_tool jolt "Run: cargo install --path . --locked"

    if ! jolt --help >/dev/null 2>&1; then
        die "The 'jolt' command exists but failed to run. Reinstall it with: cargo install --path . --locked"
    fi

    if ! cargo metadata --no-deps --format-version 1 >/dev/null 2>&1; then
        die "cargo metadata failed. Run this script from a valid Jolt checkout with dependencies available."
    fi
}

build_required_binaries() {
    log "Building required host binaries"
    cargo build --release -p fibonacci -p sha2-chain -p recursion --bin recursion
}

run_table_v() {
    local raw="$OUT_DIR/table_v/raw"
    mkdir -p "$raw"
    rm -f "$raw"/*.txt "$raw"/*.log 2>/dev/null || true

    local -a fib=(cargo run --release -p fibonacci --)
    local -a sha=(cargo run --release -p sha2-chain -- --iters 1)

    run_n() {
        local label="$1"
        local envvars="$2"
        shift 2
        local -a cmd=("$@")
        local prove_file="$raw/${label}_prove.txt"
        local s16a_file="$raw/${label}_s16a.txt"
        local s6b_file="$raw/${label}_s6b.txt"
        local s7_file="$raw/${label}_s7.txt"
        local s8_file="$raw/${label}_s8.txt"

        printf '  %-20s ' "$label"
        for i in $(seq 1 "$RUNS"); do
            local out log_file
            log_file="$raw/${label}_run_${i}.log"
            if [[ -n "$envvars" ]]; then
                out=$(env $envvars "${cmd[@]}" 2>&1)
            else
                out=$("${cmd[@]}" 2>&1)
            fi
            printf '%s\n' "$out" > "$log_file"

            if ! grep -q "valid: true" "$log_file"; then
                echo
                echo "ERROR: $label run $i did not verify. See $log_file" >&2
                exit 1
            fi

            grep "Prover runtime" "$log_file" | sed 's/.*Prover runtime: //' | sed 's/ s//' >> "$prove_file"
            local stage_line
            stage_line=$(grep "stages_1_6a=" "$log_file")
            echo "$stage_line" | sed 's/.*stages_1_6a=//' | sed 's/ms.*//' >> "$s16a_file"
            echo "$stage_line" | sed 's/.*stage_6b=//' | sed 's/ms.*//' >> "$s6b_file"
            echo "$stage_line" | sed 's/.*stage_7=//' | sed 's/ms.*//' >> "$s7_file"
            echo "$stage_line" | sed 's/.*stage_8=//' | sed 's/ms.*//' >> "$s8_file"
            printf '%s ' "$i"
        done
        echo
    }

    log "Running Table V benchmark: RUNS=$RUNS"
    for layout in CM AM; do
        log "Layout $layout"
        run_n "fib_${layout}_F"          "JOLT_LAYOUT=$layout RUST_LOG=info" "${fib[@]}"
        run_n "fib_${layout}_C1"         "JOLT_LAYOUT=$layout RUST_LOG=info" "${fib[@]}" --committed-bytecode 1
        run_n "fib_${layout}_C16"        "JOLT_LAYOUT=$layout RUST_LOG=info" "${fib[@]}" --committed-bytecode 16
        run_n "fib_${layout}_C1_naive"   "JOLT_LAYOUT=$layout JOLT_NAIVE_PAD=1 RUST_LOG=info" "${fib[@]}" --committed-bytecode 1
        run_n "fib_${layout}_C16_naive"  "JOLT_LAYOUT=$layout JOLT_NAIVE_PAD=1 RUST_LOG=info" "${fib[@]}" --committed-bytecode 16

        run_n "sha2_${layout}_F"         "JOLT_LAYOUT=$layout RUST_LOG=info" "${sha[@]}"
        run_n "sha2_${layout}_C1"        "JOLT_LAYOUT=$layout RUST_LOG=info" "${sha[@]}" --committed-bytecode 1
        run_n "sha2_${layout}_C16"       "JOLT_LAYOUT=$layout RUST_LOG=info" "${sha[@]}" --committed-bytecode 16
        run_n "sha2_${layout}_C1_naive"  "JOLT_LAYOUT=$layout JOLT_NAIVE_PAD=1 RUST_LOG=info" "${sha[@]}" --committed-bytecode 1
        run_n "sha2_${layout}_C16_naive" "JOLT_LAYOUT=$layout JOLT_NAIVE_PAD=1 RUST_LOG=info" "${sha[@]}" --committed-bytecode 16
    done

    OUT_DIR="$OUT_DIR" RUNS="$RUNS" python3 <<'PY'
import json
import os
import re
import csv
from pathlib import Path

out_dir = Path(os.environ["OUT_DIR"])
raw = out_dir / "table_v" / "raw"
report = out_dir / "table_v"
report.mkdir(parents=True, exist_ok=True)
runs = int(os.environ["RUNS"])

def load(label, suffix):
    path = raw / f"{label}_{suffix}.txt"
    vals = sorted(float(line.strip()) for line in path.read_text().splitlines() if line.strip())
    trimmed = vals[1:-1] if len(vals) >= 3 else vals
    return {
        "mean": sum(trimmed) / len(trimmed),
        "all": vals,
        "trimmed": trimmed,
    }

def metric(label):
    return {
        "prove": load(label, "prove"),
        "s16a": load(label, "s16a"),
        "s6b": load(label, "s6b"),
        "s7": load(label, "s7"),
        "s8": load(label, "s8"),
    }

labels = []
for layout in ["CM", "AM"]:
    for program in ["fib", "sha2"]:
        labels.extend([
            f"{program}_{layout}_F",
            f"{program}_{layout}_C1",
            f"{program}_{layout}_C16",
            f"{program}_{layout}_C1_naive",
            f"{program}_{layout}_C16_naive",
        ])

data = {label: metric(label) for label in labels}

def mean(label, key):
    return data[label][key]["mean"]

def pct(a, b):
    return (a / b - 1.0) * 100.0

def fmt_s(x):
    return f"{x:.3f}"

def fmt_ms_as_s(x):
    return f"{x/1000.0:.3f}"

def cell(label, base, key, scale=1.0):
    a = mean(label, key) * scale
    b = mean(base, key) * scale
    return f"{a:.3f}/{b:.3f} ({pct(a, b):+.0f}%)"

def dims_from_log(label):
    log = raw / f"{label}_run_1.log"
    text = log.read_text()
    m = re.search(r"DIMENSIONS: padded_trace=(\d+) \(log_T=(\d+)\), K=(\d+) \(log_K=(\d+)\), native_vars=(\d+), candidates=\[([^\]]*)\], main_total_vars=(\d+)", text)
    if not m:
        return None
    return {
        "padded_trace": int(m.group(1)),
        "log_T": int(m.group(2)),
        "K": int(m.group(3)),
        "log_K": int(m.group(4)),
        "native_vars": int(m.group(5)),
        "candidates": m.group(6),
        "main_total_vars": int(m.group(7)),
    }

layout_rows = []
for program in ["fib", "sha2"]:
    display = "sha2-1" if program == "sha2" else "fib"
    for mode in ["C1", "C16"]:
        ours = dims_from_log(f"{program}_CM_{mode}")
        naive = dims_from_log(f"{program}_CM_{mode}_naive")
        layout_rows.append({
            "program": display,
            "mode": mode,
            "native_trace": f"{ours['padded_trace']} -> {naive['padded_trace']}",
            "native_vars": f"{ours['native_vars']} -> {naive['native_vars']}",
            "final_vars": ours["main_total_vars"],
        })

panel_a = []
for program in ["fib", "sha2"]:
    display = "sha2-1" if program == "sha2" else "fib"
    for layout in ["CM", "AM"]:
        base = f"{program}_{layout}_F"
        for mode in ["C1", "C16"]:
            label = f"{program}_{layout}_{mode}"
            panel_a.append({
                "program": display,
                "layout": layout,
                "mode": mode,
                "total": cell(label, base, "prove"),
                "s16a": cell(label, base, "s16a", 0.001),
                "s6b": cell(label, base, "s6b", 0.001),
                "s7": cell(label, base, "s7", 0.001),
                "s8": cell(label, base, "s8", 0.001),
            })

panel_b = []
for program in ["fib", "sha2"]:
    display = "sha2-1" if program == "sha2" else "fib"
    for layout in ["CM", "AM"]:
        for mode in ["C1", "C16"]:
            ours = f"{program}_{layout}_{mode}"
            naive = f"{program}_{layout}_{mode}_naive"
            panel_b.append({
                "program": display,
                "layout": layout,
                "mode": mode,
                "total": cell(ours, naive, "prove"),
                "s16a": cell(ours, naive, "s16a", 0.001),
                "s6b": cell(ours, naive, "s6b", 0.001),
                "s7": cell(ours, naive, "s7", 0.001),
                "s8": cell(ours, naive, "s8", 0.001),
            })

summary = {
    "runs": runs,
    "raw": data,
    "layout": layout_rows,
    "panel_a": panel_a,
    "panel_b": panel_b,
}
(report / "table_v_summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True))

with (report / "layout_comparison.csv").open("w", newline="") as f:
    writer = csv.DictWriter(
        f,
        fieldnames=["program", "mode", "native_trace", "native_vars", "final_vars"],
    )
    writer.writeheader()
    writer.writerows(layout_rows)

with (report / "table_v_panel_a.csv").open("w", newline="") as f:
    writer = csv.DictWriter(
        f,
        fieldnames=["program", "layout", "mode", "total", "s16a", "s6b", "s7", "s8"],
    )
    writer.writeheader()
    writer.writerows(panel_a)

with (report / "table_v_panel_b.csv").open("w", newline="") as f:
    writer = csv.DictWriter(
        f,
        fieldnames=["program", "layout", "mode", "total", "s16a", "s6b", "s7", "s8"],
    )
    writer.writeheader()
    writer.writerows(panel_b)

def md_table(headers, rows):
    out = ["| " + " | ".join(headers) + " |", "| " + " | ".join(["---"] * len(headers)) + " |"]
    for row in rows:
        out.append("| " + " | ".join(str(x) for x in row) + " |")
    return "\n".join(out)

md = [
    "# Table V Reproduction",
    "",
    f"Runs per row: {runs}. Trimmed mean drops fastest and slowest when runs >= 3.",
    "",
    "## Layout Comparison",
    md_table(["Program", "Mode", "Native trace len", "Native vars", "Final vars"], [
        [r["program"], r["mode"], r["native_trace"], r["native_vars"], r["final_vars"]]
        for r in layout_rows
    ]),
    "",
    "## Panel (a): Committed mode vs F",
    md_table(["Program", "Layout", "Mode", "Total prove", "Stages 1-6a", "Stage 6b", "Stage 7", "Stage 8"], [
        [r["program"], r["layout"], r["mode"], r["total"], r["s16a"], r["s6b"], r["s7"], r["s8"]]
        for r in panel_a
    ]),
    "",
    "## Panel (b): Precommitted layout vs naive trace padding",
    md_table(["Program", "Layout", "Mode", "Total prove", "Stages 1-6a", "Stage 6b", "Stage 7", "Stage 8"], [
        [r["program"], r["layout"], r["mode"], r["total"], r["s16a"], r["s6b"], r["s7"], r["s8"]]
        for r in panel_b
    ]),
    "",
]
(report / "table_v_summary.md").write_text("\n".join(md))
print(f"Wrote {report / 'table_v_summary.md'}")
print(f"Wrote {report / 'table_v_summary.json'}")
print(f"Wrote {report / 'layout_comparison.csv'}")
print(f"Wrote {report / 'table_v_panel_a.csv'}")
print(f"Wrote {report / 'table_v_panel_b.csv'}")
PY
}

run_recursive() {
    local recursive_dir="$OUT_DIR/recursive"
    mkdir -p "$recursive_dir"

    OUT_DIR="$OUT_DIR" python3 <<'PY'
import json
import os
import re
import subprocess
import sys
import time
import csv
from pathlib import Path

root = Path.cwd()
out_dir = Path(os.environ["OUT_DIR"])
recursive_dir = out_dir / "recursive"
work_root = recursive_dir / "work"
work_root.mkdir(parents=True, exist_ok=True)
env = os.environ.copy()
env["RUST_LOG"] = "info"
env["JOLT_LAYOUT"] = "CM"

configs = [
    ("fib", "F", "fibonacci", [], "/tmp/fibonacci-guest.trace", "fibonacci-guest_proofs.bin"),
    ("fib", "C1", "fibonacci", ["--committed-bytecode", "--bytecode-chunk", "1"], "/tmp/fibonacci-guest.trace", "fibonacci-guest_proofs.bin"),
    ("fib", "C16", "fibonacci", ["--committed-bytecode", "--bytecode-chunk", "16"], "/tmp/fibonacci-guest.trace", "fibonacci-guest_proofs.bin"),
    ("sha2-4000", "F", "sha2-chain", [], "/tmp/sha2-chain-guest.trace", "sha2-chain-guest_proofs.bin"),
    ("sha2-4000", "C1", "sha2-chain", ["--committed-bytecode", "--bytecode-chunk", "1"], "/tmp/sha2-chain-guest.trace", "sha2-chain-guest_proofs.bin"),
    ("sha2-4000", "C16", "sha2-chain", ["--committed-bytecode", "--bytecode-chunk", "16"], "/tmp/sha2-chain-guest.trace", "sha2-chain-guest_proofs.bin"),
]

def run(cmd, log_path):
    started = time.time()
    proc = subprocess.run(cmd, cwd=root, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    log_path.write_text(proc.stdout)
    return proc.returncode, time.time() - started, proc.stdout

rows = []
for program, mode, example, flags, trace_path, proof_name in configs:
    label = f"{program}_{mode}"
    workdir = work_root / label
    workdir.mkdir(parents=True, exist_ok=True)
    proof_path = workdir / proof_name

    print(f"=== {label} generate ===", flush=True)
    cmd = ["cargo", "run", "--release", "-p", "recursion", "--bin", "recursion", "--"] + flags + [
        "generate", "--example", example, "--workdir", str(workdir)
    ]
    rc, elapsed, out = run(cmd, recursive_dir / f"{label}_generate.log")
    print(f"GENERATE_DONE {label} rc={rc} elapsed={elapsed:.1f}s", flush=True)
    if rc != 0:
        print(out[-6000:], flush=True)
        sys.exit(rc)
    bundle = proof_path.stat().st_size

    trace_file = Path(trace_path)
    if trace_file.exists():
        trace_file.unlink()

    print(f"=== {label} trace ===", flush=True)
    cmd = ["cargo", "run", "--release", "-p", "recursion", "--bin", "recursion", "--"] + flags + [
        "trace", "--example", example, "--workdir", str(workdir), "--embed", "--disk"
    ]
    rc, elapsed, out = run(cmd, recursive_dir / f"{label}_trace.log")
    print(f"TRACE_DONE {label} rc={rc} elapsed={elapsed:.1f}s", flush=True)

    markers = {
        m[0]: int(m[3])
        for m in re.findall(r'"([^"]+)": (\d+) RV64IMAC cycles \+ (\d+) virtual instructions = (\d+) total cycles', out)
    }
    trace_vals = re.findall(r"trace length: (\d+) cycles", out)
    output_vals = re.findall(r"Recursion output \(trace-only\): (\d+)", out)
    row = {
        "program": program,
        "mode": mode,
        "bundle": bundle,
        "prep_deser": markers.get("deserialize preprocessing"),
        "proof_deser": markers.get("deserialize proof"),
        "device_deser": markers.get("deserialize device"),
        "verification": markers.get("verification"),
        "final_trace": int(trace_vals[-1]) if trace_vals else None,
        "output": int(output_vals[-1]) if output_vals else None,
        "trace_rc": rc,
        "serialize_buffer_full": "SerializeBufferFull" in out,
    }
    rows.append(row)
    print("ROW_JSON " + json.dumps(row, sort_keys=True), flush=True)

    if trace_file.exists():
        size = trace_file.stat().st_size
        trace_file.unlink()
        print(f"Deleted {trace_file} ({size} bytes)", flush=True)

    if row["output"] != 1 or any(row[k] is None for k in ["prep_deser", "proof_deser", "device_deser", "verification", "final_trace"]):
        print(f"ERROR: incomplete recursive row {label}. See {recursive_dir / f'{label}_trace.log'}", flush=True)
        sys.exit(1)

(recursive_dir / "recursive_summary.json").write_text(json.dumps(rows, indent=2, sort_keys=True))

with (recursive_dir / "recursive_summary.csv").open("w", newline="") as f:
    writer = csv.DictWriter(
        f,
        fieldnames=[
            "program",
            "mode",
            "bundle",
            "prep_deser",
            "proof_deser",
            "device_deser",
            "verification",
            "final_trace",
            "output",
            "trace_rc",
            "serialize_buffer_full",
        ],
    )
    writer.writeheader()
    writer.writerows(rows)

def fmt(n):
    return f"{n:,}" if isinstance(n, int) else ""

def md_table(headers, rows_):
    lines = ["| " + " | ".join(headers) + " |", "| " + " | ".join(["---"] * len(headers)) + " |"]
    for row in rows_:
        lines.append("| " + " | ".join(str(x) for x in row) + " |")
    return "\n".join(lines)

md = [
    "# Recursive Verification Reproduction",
    "",
    "All cycle counts are RV64IMAC + virtual instructions. Trace files are deleted after parsing.",
    "",
    md_table(
        ["Program", "Mode", "Bundle (B)", "Prep. deser.", "Proof deser.", "Device deser.", "Verification", "Final trace"],
        [
            [r["program"], r["mode"], fmt(r["bundle"]), fmt(r["prep_deser"]), fmt(r["proof_deser"]), fmt(r["device_deser"]), fmt(r["verification"]), fmt(r["final_trace"])]
            for r in rows
        ],
    ),
    "",
]
(recursive_dir / "recursive_summary.md").write_text("\n".join(md))
print(f"Wrote {recursive_dir / 'recursive_summary.md'}")
print(f"Wrote {recursive_dir / 'recursive_summary.json'}")
print(f"Wrote {recursive_dir / 'recursive_summary.csv'}")
PY
}

write_index() {
    OUT_DIR="$OUT_DIR" python3 <<'PY'
import os
from pathlib import Path

out = Path(os.environ["OUT_DIR"])
parts = ["# Paper Experiment Reproduction", ""]
if (out / "table_v" / "table_v_summary.md").exists():
    parts += ["- Table V: `table_v/table_v_summary.md`", "- Table V raw logs: `table_v/raw/`"]
    parts += ["- Table V CSVs: `table_v/layout_comparison.csv`, `table_v/table_v_panel_a.csv`, `table_v/table_v_panel_b.csv`"]
if (out / "recursive" / "recursive_summary.md").exists():
    parts += ["- Recursive verification: `recursive/recursive_summary.md`", "- Recursive raw logs: `recursive/*.log`"]
    parts += ["- Recursive CSV: `recursive/recursive_summary.csv`"]
parts += ["", f"Output directory: `{out}`", ""]
(out / "README.md").write_text("\n".join(parts))
print(f"Wrote {out / 'README.md'}")
PY
}

check_prerequisites
build_required_binaries

if [[ "$COMMAND" == "all" || "$COMMAND" == "table-v" ]]; then
    run_table_v
fi

if [[ "$COMMAND" == "all" || "$COMMAND" == "recursive" ]]; then
    run_recursive
fi

write_index
log "Done. Reports are in $OUT_DIR"
