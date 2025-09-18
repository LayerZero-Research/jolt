#!/usr/bin/env bash
# run_benchmarks.sh — Orchestrate machine prep, optional pinning, and benchmark runs
#
# Usage:
#   ./scripts/benchmark/run_benchmarks.sh [flags]
#
# Flags:
#   --prep | --no-prep              Run machine prep (Linux only, requires sudo). Default: --prep on Linux, --no-prep on macOS
#   --pin  | --no-pin               Run benchmarks under NUMA pinning. Default: --no-pin
#   --nodes NODES                   NUMA nodes to pin to (e.g. 0, 0-3, 0,2). Default: env PIN_NODES or 0
#   --policy bind|interleave        NUMA memory policy. Default: env PIN_POLICY or bind
#   --max-scale N                   Max scale exponent (passed to jolt_runner.sh as first arg)
#   --min-scale M                   Min scale exponent (passed as second arg)
#   -h | --help                     Show this help
#
# Examples:
#   # Local macOS (no prep/pin)
#   ./scripts/benchmark/run_benchmarks.sh --max-scale 27 --min-scale 20
#
#   # Linux AWS: default does prep; pinned run on nodes 0-3
#   ./scripts/benchmark/run_benchmarks.sh --pin --nodes 0-3 --policy bind --max-scale 27 --min-scale 20
#
# Notes:
#   - On macOS, prep and pin are skipped automatically.
#   - This script forwards only max/min scale to jolt_runner.sh. Edit that script for bench list.

set -euo pipefail

show_help() {
  sed -n '2,40p' "$0"
}

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

# Ensure we can find the project root (for validation)
PROJECT_ROOT=$(find_project_root)
if [[ -z "$PROJECT_ROOT" ]]; then
    exit 1
fi

OS_NAME=$(uname -s)
IS_LINUX=0
if [[ "$OS_NAME" == "Linux" ]]; then
  IS_LINUX=1
fi

# Defaults (can be overridden by flags or env)
# Prep by default on Linux; skip on macOS
if (( IS_LINUX )); then
  PREP=1
else
  PREP=0
fi
PIN=0
PIN_NODES_DEFAULT=${PIN_NODES:-0}
PIN_POLICY_DEFAULT=${PIN_POLICY:-bind}
MAX_SCALE=""
MIN_SCALE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prep) PREP=1 ;;
    --no-prep) PREP=0 ;;
    --pin) PIN=1 ;;
    --no-pin) PIN=0 ;;
    --nodes)
      [[ $# -lt 2 ]] && { echo "--nodes requires a value"; exit 1; }
      PIN_NODES_DEFAULT="$2"; shift ;;
    --policy)
      [[ $# -lt 2 ]] && { echo "--policy requires a value (bind|interleave)"; exit 1; }
      PIN_POLICY_DEFAULT="$2"; shift ;;
    --max-scale)
      [[ $# -lt 2 ]] && { echo "--max-scale requires a value"; exit 1; }
      MAX_SCALE="$2"; shift ;;
    --min-scale)
      [[ $# -lt 2 ]] && { echo "--min-scale requires a value"; exit 1; }
      MIN_SCALE="$2"; shift ;;
    -h|--help) show_help; exit 0 ;;
    --) shift; break ;;
    *)
      # Positional numeric convenience: first number -> max, second -> min
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        if [[ -z "$MAX_SCALE" ]]; then
          MAX_SCALE="$1"
        elif [[ -z "$MIN_SCALE" ]]; then
          MIN_SCALE="$1"
        else
          echo "Ignoring extra positional argument: $1"
        fi
      else
        echo "Unknown flag/arg: $1"; echo; show_help; exit 1
      fi
      ;;
  esac
  shift
done

# Apply defaults if not provided
MAX_SCALE=${MAX_SCALE:-27}
MIN_SCALE=${MIN_SCALE:-20}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "OS: $OS_NAME | prep=$PREP pin=$PIN | nodes=$PIN_NODES_DEFAULT policy=$PIN_POLICY_DEFAULT"

# 1) Prep machine (Linux only)
if (( PREP )); then
  if (( IS_LINUX )); then
    echo "==> Running machine prep (requires sudo)"
    "$SCRIPT_DIR/tune_linux.sh"
  else
    echo "==> Skipping prep on $OS_NAME (Linux-only)"
  fi
fi

# 2) Run benchmarks, optionally pinned (Linux only)
BENCH_SCRIPT="$SCRIPT_DIR/jolt_runner.sh"
if [[ ! -x "$BENCH_SCRIPT" ]]; then
  # Try to make it executable; ignore failure
  chmod +x "$BENCH_SCRIPT" 2>/dev/null || true
fi

if (( PIN )) && (( IS_LINUX )); then
  echo "==> Running benchmarks with NUMA pinning"
  export PIN_NODES="$PIN_NODES_DEFAULT"
  export PIN_POLICY="$PIN_POLICY_DEFAULT"
  "$SCRIPT_DIR/pin_numa.sh" "$BENCH_SCRIPT" "$MAX_SCALE" "$MIN_SCALE"
else
  if (( PIN )) && (( ! IS_LINUX )); then
    echo "==> Skipping pin on $OS_NAME (Linux-only). Running benchmarks directly."
  else
    echo "==> Running benchmarks without pinning"
  fi
  "$BENCH_SCRIPT" "$MAX_SCALE" "$MIN_SCALE"
fi

echo "All benchmarks completed."


