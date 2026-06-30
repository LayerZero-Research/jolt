#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

IMAGE="${IMAGE:-jolt-artifact}"
RESULTS="${RESULTS:-results}"
DORY_CACHE="${DORY_CACHE:-jolt-dory}"
COMMAND="${1:-}"

usage() {
    cat <<EOF
Usage: bash paper_artifacts_scripts/docker.sh <check|table-v|recursive|all>

Environment:
  IMAGE       Docker image name. Default: $IMAGE
  RESULTS     Host output directory. Default: $RESULTS
  DORY_CACHE  Docker volume for Dory setup cache. Default: $DORY_CACHE
  RUNS        Table V runs per row. Default is set by the artifact script.
EOF
}

case "$COMMAND" in
    check|table-v|recursive|all)
        ;;
    -h|--help|"")
        usage
        [[ "$COMMAND" == "" ]] && exit 1 || exit 0
        ;;
    *)
        echo "Unknown command: $COMMAND" >&2
        usage >&2
        exit 1
        ;;
esac

if ! docker info >/dev/null 2>&1; then
    cat >&2 <<'EOF'
ERROR: Docker is not available to this shell.

Start Docker Desktop, or on Ubuntu run:
  bash paper_artifacts_scripts/setup_ubuntu.sh docker
  sudo usermod -aG docker "$USER"
  newgrp docker
EOF
    exit 1
fi

mkdir -p "$RESULTS"
docker volume create "$DORY_CACHE" >/dev/null
docker build -f paper_artifacts_scripts/Dockerfile -t "$IMAGE" .
if ! docker run --rm \
    -e RUNS="${RUNS:-}" \
    -v "$(pwd)/$RESULTS:/tmp/jolt-paper-experiments" \
    -v "$DORY_CACHE:/root/.cache/dory" \
    "$IMAGE" "$COMMAND"; then
    cat >&2 <<EOF
ERROR: Docker artifact command failed.

If the error mentions Dory setup or a .urs file, clear the Dory cache with:
  docker volume rm $DORY_CACHE

Then rerun:
  bash paper_artifacts_scripts/docker.sh $COMMAND
EOF
    exit 1
fi
