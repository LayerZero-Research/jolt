# Paper Artifact Experiment Scripts

This folder contains scripts for reproducing the experiments in
`precommitted-geometry-and-dory-embedding.tex`.

## Prerequisites

Run from the repository root.

Required tools:

- `cargo`
- `rustc`
- `python3`
- `jolt`

If `jolt` is missing or stale, install it from this checkout:

```bash
cargo install --path . --locked
```

The script checks these tools before running. If a tool is missing, it exits with
a message describing what to install.

## Commands

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
│       └── *_run_*.log
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
- `table_v/raw/*_run_*.log`: full command output for each timing run.
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