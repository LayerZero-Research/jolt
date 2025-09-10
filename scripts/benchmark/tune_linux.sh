#!/usr/bin/env bash
# Fast system-level tuning for zkVM benchmarks on Ubuntu 24.
# 
# Usage:
#   ./tune_linux.sh           # Full system setup (run once after boot)
#   ./tune_linux.sh --cache-only  # Per-run prep (page cache drop + ulimits)
#
# Full setup: performance governor, THP, reduces I/O jitter, tames background services
# Per-run prep: drops page cache and sets ulimits for clean benchmark runs

set -euo pipefail

CACHE_ONLY=false
if [[ "${1:-}" == "--cache-only" ]]; then
    CACHE_ONLY=true
fi

need_sudo() {
  if [[ "$EUID" -ne 0 ]]; then
    echo "Re-running with sudo..."
    exec sudo --preserve-env=PATH "$0" "$@"
  fi
}
need_sudo "$@"

if [[ "$CACHE_ONLY" == "true" ]]; then
    echo "==> Running per-run prep (cache drop + ulimits)"
else
    echo "==> Running full system setup"
    echo "==> Setting CPU to performance mode"
fi

# Per-run prep: always run these
if [[ "$CACHE_ONLY" == "true" ]] || [[ "$CACHE_ONLY" == "false" ]]; then
    echo "==> Increasing file descriptor & memlock limits for current shell"
    ulimit -n 1048576 || true
    ulimit -l unlimited 2>/dev/null || true

    echo "==> Dropping page cache for clean benchmark runs"
    sync
    echo 3 > /proc/sys/vm/drop_caches
fi

# Skip persistent settings if cache-only mode
if [[ "$CACHE_ONLY" == "true" ]]; then
    exit 0
fi

echo "==> Setting CPU to performance mode"
if command -v powerprofilesctl >/dev/null 2>&1; then
  powerprofilesctl set performance || true
fi

# Try amd-pstate first; fall back to per-core governor
if [[ -d /sys/devices/system/cpu/amd_pstate ]]; then
  echo 1 > /sys/devices/system/cpu/amd_pstate/status 2>/dev/null || true
fi
if [[ -d /sys/devices/system/cpu/cpu0/cpufreq ]]; then
  for g in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    [[ -f "$g" ]] && echo performance > "$g" || true
  done
fi

echo "==> Enabling Transparent Huge Pages (THP) and defrag"
for f in /sys/kernel/mm/transparent_hugepage/enabled /sys/kernel/mm/transparent_hugepage/defrag; do
  [[ -f "$f" ]] && echo always > "$f" || true
done

echo "==> Disabling automatic NUMA balancing (we'll pin manually)"
if [[ -f /proc/sys/kernel/numa_balancing ]]; then
  echo 0 > /proc/sys/kernel/numa_balancing
fi

echo "==> Reducing I/O writeback jitter"
sysctl -q vm.dirty_ratio=10
sysctl -q vm.dirty_background_ratio=5

echo "==> Turning swap off (reversible with 'swapon -a')"
swapoff -a || true

echo "==> Stopping irqbalance to reduce cross-core interrupts (optional)"
if systemctl list-unit-files | grep -q '^irqbalance\.service'; then
  systemctl stop irqbalance || true
fi

echo "==> Detecting NUMA topology and preparing core list on node 0"
if ! command -v lscpu >/dev/null 2>&1; then
  apt-get update && apt-get install -y util-linux
fi

# Build a comma-separated list of CPU IDs that belong to NUMA node 0.
CORES_NODE0=$(lscpu -e=CPU,NODE | awk '$2 == 0 {print $1}' | paste -sd, -)
if [[ -z "${CORES_NODE0}" ]]; then
  echo "Could not determine cores for NUMA node 0; using all cores."
  CORES_NODE0=$(lscpu -e=CPU | awk 'NR>1{print $1}' | paste -sd, -)
fi
echo "Cores on node 0: ${CORES_NODE0}"

echo "==> Example: run your binary pinned to node 0 + those cores, high priority"
echo
cat <<'EOF'

# Replace ./target/release/your-binary and <args> as needed.
# chrt -f uses FIFO scheduling; requires root. If you prefer normal policy, drop chrt.

numactl --cpunodebind=0 --membind=0 \
taskset -c __CORES_NODE0__ \
chrt -f 80 \
./target/release/your-binary <args>

EOF
echo

# Print a ready-to-paste command with actual cores filled in:
echo "Ready-to-paste command with detected cores:"
echo "numactl --cpunodebind=0 --membind=0 taskset -c ${CORES_NODE0} chrt -f 80 ./target/release/your-binary <args>"

echo
echo "All set. Run the command above to benchmark."