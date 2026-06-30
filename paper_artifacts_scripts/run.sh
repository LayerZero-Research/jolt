#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
exec bash paper_artifacts_scripts/reproduce_paper_experiments.sh "$@"
