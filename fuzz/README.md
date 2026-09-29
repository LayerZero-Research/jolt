# Jolt + Akita liveness campaign

A standalone, resumable fuzzing campaign for Jolt with the Akita lattice PCS,
built on cargo-fuzz/libFuzzer. Its first goal is **liveness**: every statement
inside Jolt's documented limits (program, inputs, advice, prover options) must
preprocess, prove, and verify. A clean error on a provable input is a finding,
not an expected rejection. Soundness and robustness checks are secondary.

- What is tested, how, and what is not: [`COVERAGE.md`](COVERAGE.md)
- Findings: [`FINDINGS.md`](FINDINGS.md)
- Target registry (single source of truth): [`campaign/targets.toml`](campaign/targets.toml)

Everything is Rust:

| Crate | Path | Role |
|---|---|---|
| `jolt-akita-fuzz` | `src/` | Targets, generators, and oracles, independent of the engine |
| `jolt-fuzz` (runner) | `runner/` | Builds and runs the campaign; does not link Jolt or Akita |
| `jolt-fuzz-dev` | `devtool/` | Uninstrumented commands: sweeps, smoke, replay, guest and bundle builds |
| `jolt-fuzz-interp-guest` | `guests/interp/` | A guest whose input is a program for a small interpreter |

The runner is a copy of the Akita campaign's runner (`LayerZero-Labs/akita`,
branch `fuzzer`, `fuzz/runner`), parametrized for this repository. A copy
rather than a git dependency because the runner locates its source tree through
`CARGO_MANIFEST_DIR`, hardcodes crate, binary, and environment names, and keys
crash signatures on crate-name frames; none of that works from a git checkout
under `~/.cargo/git`.

This workspace lives at the repository root on purpose. `scripts/fuzz.py`
discovers `crates/*/fuzz` and `*/fuzz` workspaces, not `./fuzz`, so this
campaign does not join the CI fuzz matrix.

## What it fuzzes

Jolt `main` with its pinned Akita (`252abb89`, the root `Cargo.toml`
`akita-*` revision). The harness depends on the same revision with
`response-model-diagnostics`, which adds fold-response reports and changes no
proof bytes.

| Target | Lanes | Kind | What one input is |
|---|---|---|---|
| `planning` | 1 | end to end | A production preprocessing shape (trace length, one-hot width, bytecode and RAM domains, advice capacities, committed program and chunk count) mapped through Jolt's own geometry into grouped Akita planning; small shapes also commit, prove, and verify |
| `grid` | `k16`, `k256`, `dense` | end to end | One row of a shipped schedule artifact, with a generated witness and point, through Jolt's Akita setup, commit, prove, transport, and verify |
| `program` | `interp`, `examples` | end to end | A guest ELF, its arguments and advice, and the prover's free options, through the full Jolt preprocess, prove, and verify |
| `verifier` | 1 | boundary | Edits to one region (proof, verifier preprocessing, public I/O, commitment) of an honest proof bundle, then decode and verify |

Every proving target runs under the fold-margin liveness oracle
(`src/liveness.rs`). See COVERAGE.md for the oracles.

## Quick start

On a Linux x86_64 machine with `rustup`, `git`, and a C/C++ toolchain:

```bash
git clone https://github.com/LayerZero-Research/jolt && cd jolt
git checkout fuzzer
cargo install --path . --locked               # the `jolt` CLI, used to build guests
cd fuzz
rustup show                                    # installs nightly-2026-08-24 (fuzz/rust-toolchain.toml)
cargo install cargo-fuzz --version 0.13.2 --locked
cargo run --release -p jolt-fuzz-runner -- prepare --out ../../jolt-fuzz-dist   # about 40 min
../../jolt-fuzz-dist/jolt-fuzz run --output ../../jolt-fuzz-out
```

`run` does not return on its own unless `--duration-hours` is given. Run it
under `tmux`, `screen`, `nohup`, or a systemd user service, and stop it with
Ctrl-C or `SIGTERM`. Rerun the same command to resume.

### Hosts with endpoint security

A fuzzing campaign writes many small binary files (corpus entries, crash
artifacts, generated seeds) and runs sanitizer-instrumented binaries. Endpoint
detection has flagged similar activity as malware staging (see the 2026-09-28
note in FINDINGS.md). Agree the campaign's directories with the host's security
team before running it, and generate inputs with `jolt-fuzz-dev` (never by
writing bytes from a shell).

## 1. Preparation

`prepare` is the only step that needs Cargo and the network. It:

1. checks that `campaign/targets.toml` and `jolt_akita_fuzz::targets::ALL`
   name the same targets;
2. builds the instrumented binary: `cargo fuzz build --release
   --debug-assertions --sanitizer address --target <host> fuzz_all`. One binary
   (`fuzz_all`) links every target and `JOLT_FUZZ_TARGET` selects one;
3. builds every guest in `src/programs.rs` with the `jolt` CLI
   (`jolt-fuzz-dev build-guests`), from the repository root for the example
   guests and from `fuzz/guests/` for the interpreter guest;
4. proves honest verifier bundles from those guests (`jolt-fuzz-dev bundles`);
5. generates seeds (`jolt-fuzz-dev seeds`);
6. copies everything into `--out`:

```
jolt-fuzz                 the runner (uninstrumented)
bin/fuzz_all              the instrumented libFuzzer binary
bin/llvm-symbolizer       if one was found on the build host
artifacts/schedules/*.aks the three schedule artifacts (crates/jolt-akita/schedules)
artifacts/guests/*.elf    guest ELFs
artifacts/bundles/*.bundle honest proof bundles for the verifier target
seeds/<target>/           seed corpora
campaign/targets.toml, README.md, BUILD-INFO.json, MANIFEST.sha256
```

The fuzz profile keeps `debug-assertions`, `overflow-checks`, and line tables
and turns fat LTO off. Options: `--sanitizer none` builds without ASan,
`--skip-build` repackages an existing build, `--sequential` builds without
Rayon. `jolt-fuzz validate` re-checks a distribution against its manifest.

The toolchain is pinned in `fuzz/rust-toolchain.toml` (the same nightly as the
repository's other fuzz workspaces) and dependency versions in
`fuzz/Cargo.lock`.

## 2. Running

```bash
dist/jolt-fuzz run --output DIR [options] [-- extra libFuzzer flags]
```

| Option | Default | Meaning |
|---|---|---|
| `--cpus N` | usable CPUs minus a reserve | CPU slots for workers |
| `--reserve-cpus R` | max(1, CPUs/16) | CPUs left to the system |
| `--memory-mb M` | 80% of available | Memory budget for all workers |
| `--targets a,b` / `--exclude a,b` | all | Restrict lanes |
| `--slice-minutes M` | 60 | libFuzzer process lifetime before rotation |
| `--duration-hours H` | unbounded | Stop after H hours |
| `--no-hard-memory-limit` | off | Skip cgroup memory scopes |
| `--skip-baseline` | off | Skip the startup seed execution |
| `--no-replay` | off | Do not re-run new findings in a fresh process |
| `--adopt` | off | Resume an output directory created on another machine |

At startup the runner validates the distribution, takes an exclusive lock on
the output directory, records the machine identity, sizes the CPU and memory
budget, copies seeds into the corpus, and runs every lane once over its shipped
seeds (the honest baseline). Each lane starts fuzzing when its own baseline
passes; a baseline failure is recorded as a finding first.

Scheduling, limits, and failure handling are the Akita runner's:

- **Lanes and scheduling.** A lane is a target or one of its variants
  (`grid@k256`). Lanes are chosen by weighted deficit (least CPU time per unit
  `weight` runs next). A lane reserves `threads` CPU slots
  (`JOLT_FUZZ_THREADS`, which sizes the harness's global Rayon pool) and an
  estimate of its memory.
- **Limits.** Per libFuzzer process: `-timeout`, `-rss_limit_mb`,
  `-malloc_limit_mb`, and `-max_len` from the registry. When
  `systemd-run --user --scope` works, each worker also runs in its own cgroup
  with `MemoryMax = rss_limit_mb + 1024 MiB`. A wall-clock watchdog kills jobs
  that outlive their slice.
- **Failures.** A failing worker is classified (`panic`, `asan`, `timeout`,
  `oom`, `leak`, `signal`, `crash`) and given a signature (panic location and
  message shape, sanitizer summary, or top `jolt`/`akita` frames). The raw
  artifact and the last 400 output lines go under
  `findings/<kind>-<target>-<hash>/`, the input is quarantined out of the
  corpus, and a new signature is replayed once in a fresh process.
- **Corpus compaction.** libFuzzer `-merge=1` from a hard-linked snapshot
  once a corpus reaches 200 inputs and has doubled; dropped inputs move to
  `corpus-archive/`.
- **Shutdown and resume.** `SIGINT`/`SIGTERM` stops workers and saves state;
  rerunning resumes corpora, findings, quarantine, and per-lane totals.

Environment the runner sets for each worker: `TMPDIR` (the output
directory's `tmp/`, so libFuzzer's merge files and every other temporary file
stay inside the campaign directory), `JOLT_FUZZ_TARGET`,
`JOLT_FUZZ_THREADS`, `JOLT_FUZZ_ARTIFACTS`, `JOLT_FUZZ_GUESTS`,
`JOLT_FUZZ_BUNDLES`, `JOLT_FUZZ_STATS_FILE`, and lane variant variables
(`JOLT_FUZZ_GRID_FAMILY`, `JOLT_FUZZ_GUEST_SET`). Harness limits read from the
environment: `JOLT_FUZZ_MAX_CASE_COEFFS` (default `2^23` committed
coefficients per `planning`/`grid` iteration) and `JOLT_FUZZ_MAX_LOG_T`
(default 18, the largest trace the `program` target proves).

## 3. Inspecting, exporting, reproducing

```bash
dist/jolt-fuzz status --output DIR [--json]     # budget, lanes, coverage, findings, reach counters
dist/jolt-fuzz export --output DIR [--to PATH] [--with-logs]
dist/jolt-fuzz reproduce --output DIR <finding-id>
dist/jolt-fuzz reproduce --output DIR program@interp ./input.bin
dist/jolt-fuzz minimize --output DIR <finding-id> [--seconds 300]
```

Reach counters in `status` show what each target actually exercised, for
example `honest_verified`, `in_contract_ok`, `out_of_contract_rejected`,
`guest_panicked`, `binding_rejected`, `verify_reject`, and the liveness
buckets (`liveness_attempts_*`, `liveness_margin_*`).

## Developer commands

Run from `fuzz/` with `cargo run --release -p jolt-fuzz-dev -- <command>`:

| Command | Purpose |
|---|---|
| `list` | Library target names |
| `smoke TARGET [N] [SEED]` | N pseudo-random inputs without libFuzzer; failing inputs are saved to `smoke-failures/` and the run continues |
| `replay TARGET FILE...` | Run inputs once |
| `seeds DIR` | Regenerate seed corpora |
| `build-guests DIR` | Build every guest ELF with the `jolt` CLI |
| `bundles DIR` | Prove honest verifier bundles (needs `JOLT_FUZZ_GUESTS`) |
| `plan-sweep [PHASE] [CSV]` | Near-exhaustive preprocessing-planning sweep; phases `geometry`, `advice`, `program`, `all`; nonzero exit if any in-contract shape fails |
| `planning-case LOG_T K LOG_BYTECODE LOG_RAM_K U T [CHUNKS IMAGE]` | Plan, commit, prove, and verify one shape with no size cap |
| `grid-sweep [MIN] [MAX] [FAMILY]` | Every catalog row with MIN..=MAX variables, four witnesses each, printing fold margins and attempts |

`JOLT_FUZZ_LIVENESS_LOG=1` prints every fold report;
`JOLT_FUZZ_OPENING_LOG=1` prints each opening's geometry.

Targets and variants the lanes use can be replayed with the same variables the
runner sets, for example
`JOLT_FUZZ_GUEST_SET=interp JOLT_FUZZ_GUESTS=... jolt-fuzz-dev replay program input.bin`.

## Adding a target

1. Implement `pub fn run(data: &[u8])` in `src/targets/`, list it in
   `targets::ALL`, and give it seeds in `targets::seeds`.
2. Add `[target.<name>]` to `campaign/targets.toml`.

`prepare` refuses to package if the registry and the library disagree.

## Adding a guest

Add an entry to `GUESTS` in `src/programs.rs` with its package, function,
memory attributes (they must match its `#[jolt::provable]` attributes, because
the host-side layout must match the ELF), and an argument generator. Guests
outside the root workspace go under `fuzz/guests/`.

## Output directory layout

```
campaign.json      identity: campaign id, machine, build ids
state.json         per-lane totals (every 15 s and at shutdown)
live.json          running jobs (while running)
events.log         runner events (capped)
corpus/<target>/          shared, resumable corpora
findings/<id>/            meta.json, sample-N.input, sample-N.txt, replay.txt
quarantine/<target>/      corpus inputs removed because they crash
corpus-archive/<target>/  inputs dropped by compaction
logs/<lane>/              per-job libFuzzer output (capped)
tmp/                      worker TMPDIR and reproduce scratch
stats/<lane>/             reach counters of running jobs
exports/                  archives written by `export`
```
