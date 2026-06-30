# Paper Artifact Experiment Scripts

This folder contains scripts for reproducing the experiments in
`precommitted-geometry-and-dory-embedding.tex`.

The exact artifact source is the `paper-experiments` branch of this repository.
The repository is dual licensed under `LICENSE-MIT` and `LICENSE-APACHE` at the
repository root.

## Supported Reproduction Paths

The artifact supports two documented paths:

- Docker: reproducible Ubuntu 22.04 environment with dependencies installed in
  the image. This path works on Linux, macOS, and Windows hosts with Docker.
- Native Ubuntu: direct execution on an Ubuntu 22.04 host, recommended when
  collecting paper-quality timing measurements on an otherwise idle machine.

Both paths run the same `paper_artifacts_scripts/reproduce_paper_experiments.sh`
script. Native runs write reports under `/tmp/jolt-paper-experiments`; Docker
runs write to that path inside the container and preserve the files on the host
through the mounted `results` directory.

## Quick Start

After installing Docker, run:

```bash
bash paper_artifacts_scripts/docker.sh check
bash paper_artifacts_scripts/docker.sh table-v
bash paper_artifacts_scripts/docker.sh recursive
```

On Windows PowerShell, use:

```powershell
.\paper_artifacts_scripts\docker.ps1 check
.\paper_artifacts_scripts\docker.ps1 table-v
.\paper_artifacts_scripts\docker.ps1 recursive
```

Generated tables are written to `results/` on the host. For native Ubuntu runs,
install prerequisites with `bash paper_artifacts_scripts/setup_ubuntu.sh native`
and replace `docker.sh` with `run.sh`.

## Install Docker Prerequisites

Use this section for the Docker path. If you use the native Ubuntu path, skip to
`Native Ubuntu Path`.

### Ubuntu 22.04

Install Git and Docker Engine:

```bash
bash paper_artifacts_scripts/setup_ubuntu.sh docker
```

The wrapper scripts call `docker` without `sudo`. The setup script adds your user
to the `docker` group; start a new shell or run:

```bash
newgrp docker
docker run hello-world
```

### macOS

Install Git and Docker Desktop:

```bash
xcode-select --install
```

Then install Docker Desktop from <https://www.docker.com/products/docker-desktop/>
and start it once from `/Applications`. Verify:

```bash
docker version
docker run hello-world
```

If you use Homebrew, this is also fine:

```bash
brew install git
brew install --cask docker
```

### Windows

Install:

- Git for Windows: <https://git-scm.com/download/win>
- Docker Desktop with WSL 2 backend: <https://www.docker.com/products/docker-desktop/>

Start Docker Desktop and verify from PowerShell:

```powershell
docker version
docker run hello-world
```

## Docker Path

Run Docker commands from the repository root. The wrapper builds the image,
creates the Dory cache volume, mounts the host result directory, and runs the
artifact command.

Linux/macOS Bash or zsh:

```bash
bash paper_artifacts_scripts/docker.sh check
bash paper_artifacts_scripts/docker.sh table-v
bash paper_artifacts_scripts/docker.sh recursive
```

Windows PowerShell:

```powershell
.\paper_artifacts_scripts\docker.ps1 check
.\paper_artifacts_scripts\docker.ps1 table-v
.\paper_artifacts_scripts\docker.ps1 recursive
```

To run both experiment groups in one command, replace `table-v` or `recursive`
with `all`.

The Docker wrapper uses short default names:

- Image: `jolt-artifact`
- Dory setup volume: `jolt-dory`
- Host result directory: `results`

Override them with `IMAGE`, `DORY_CACHE`, or `RESULTS` if needed. For example:

```bash
RESULTS=/tmp/jolt-results bash paper_artifacts_scripts/docker.sh table-v
```

The artifact uses two Dory setup caches by default:

- The `jolt-dory` Docker volume is mounted at `/root/.cache/dory`, so generated
  URS files persist across `check`, `table-v`, and `recursive` containers.
- Jolt keeps deserialized Dory prover setup in-process, so repeated samples in
  one table row do not reload the same setup from disk.

If Docker fails with a Dory setup or `.urs` error, clear the cache and rerun:

```bash
docker volume rm jolt-dory
bash paper_artifacts_scripts/docker.sh check
```

PowerShell:

```powershell
docker volume rm jolt-dory
.\paper_artifacts_scripts\docker.ps1 check
```

The Docker build installs Ubuntu packages, installs the pinned Rust toolchain and
RISC-V targets, runs `cargo install --path . --locked`, and prebuilds the release
binaries used by the artifact script. This verifies that all Rust dependencies
are publicly resolvable, including `dory-pcs = 0.3.0`. Repeated Docker runs reuse
the cached image layers unless the source checkout changes.

After a Docker run, list the generated result directories on the host:

Linux/macOS Bash or zsh:

```bash
ls -R results
```

Windows PowerShell:

```powershell
Get-ChildItem -Recurse results
```

The main tables are:

- `results/table-v-*/table_v/table_v_summary.md`
- `results/table-v-*/table_v/table_v_panel_a.csv`
- `results/table-v-*/table_v/table_v_panel_b.csv`
- `results/table-v-*/table_v/layout_comparison.csv`
- `results/recursive-*/recursive/recursive_summary.md`
- `results/recursive-*/recursive/recursive_summary.csv`
- `results/all-*/README.md` when using `all`

## Native Ubuntu Path

Run all native commands from the repository root.

The artifact uses the Rust toolchain pinned in `rust-toolchain.toml` and resolves
the Dory PCS dependency from the public `dory-pcs = 0.3.0` crate. It should not
require any author-local dependency paths.

On a clean Ubuntu 22.04 machine, install the system prerequisites and Rust with:

```bash
bash paper_artifacts_scripts/setup_ubuntu.sh native
```

The native reproduction script checks these tools before running:

- `cargo`
- `rustc`
- `python3`
- `jolt`

Run the native wrapper from the repository root:

```bash
bash paper_artifacts_scripts/run.sh check
bash paper_artifacts_scripts/run.sh table-v
bash paper_artifacts_scripts/run.sh recursive
```

To run both experiment groups in one command, replace `table-v` or `recursive`
with `all`. Native execution avoids container overhead and is preferable for
benchmark numbers intended for direct comparison with the paper.

## Underlying Script Commands

The Docker and native wrappers above call this script. These direct commands are
listed for users who want to bypass the wrappers:

```bash
bash paper_artifacts_scripts/reproduce_paper_experiments.sh check
```

Checks prerequisites and builds the release binaries used by the script without
running the long benchmarks.

```bash
bash paper_artifacts_scripts/reproduce_paper_experiments.sh table-v
```

Runs the non-recursive Table V and layout experiments.

```bash
bash paper_artifacts_scripts/reproduce_paper_experiments.sh recursive
```

Runs the recursive verifier experiments. This generates the inner proof bundles
and then traces the recursive verifier guest for each row.

```bash
bash paper_artifacts_scripts/reproduce_paper_experiments.sh all
```

Runs both groups.

All commands use the paper settings:

- Table V proving-time experiments with `RUNS=10`.
- Trimmed means drop the fastest and slowest runs.
- Layout dimension checks.
- Recursive verifier bundle/cycle experiments.

This can take hours and uses a lot of CPU. Recursive trace files can be tens of
GB, so the script deletes `/tmp/*-guest.trace` files after parsing them.

## Output Layout

Each command writes to a timestamped directory:

```text
/tmp/jolt-paper-experiments/<command>-YYYYMMDD-HHMMSS/
```

For `all`, the output layout is:

```text
/tmp/jolt-paper-experiments/all-YYYYMMDD-HHMMSS/
├── README.md
├── table_v/
│   ├── layout_comparison.csv
│   ├── table_v_panel_a.csv
│   ├── table_v_panel_b.csv
│   ├── table_v_summary.json
│   ├── table_v_summary.md
│   └── raw/
│       ├── *_prove.txt
│       ├── *_s16a.txt
│       ├── *_s6b.txt
│       ├── *_s7.txt
│       ├── *_s8.txt
│       └── *_runs.log
└── recursive/
    ├── recursive_summary.csv
    ├── recursive_summary.json
    ├── recursive_summary.md
    ├── *_generate.log
    ├── *_trace.log
    └── work/
        └── <program>_<mode>/
            └── *_proofs.bin
```

## Output Formats

- `*.csv`: spreadsheet-friendly paper tables.
- `*.json`: structured raw summary for scripts.
- `*.md`: human-readable tables.
- `table_v/raw/*.txt`: raw timing values used for trimmed means.
- `table_v/raw/*_runs.log`: full command output for each table row. The
  examples run all `RUNS` samples in one process so Dory preprocessing is reused
  instead of reloaded from disk for every sample.
- `recursive/*_generate.log`: recursive proof bundle generation logs.
- `recursive/*_trace.log`: recursive verifier trace logs.

## Trace Files

Trace files are deleted immediately after parsing because they can be very large.
During a recursive run, temporary trace files are written by the harness to paths
like:

```text
/tmp/fibonacci-guest.trace
/tmp/sha2-chain-guest.trace
```