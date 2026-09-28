# Coverage inventory

Scope: Jolt `main` with the Akita revision it pins (`252abb89`), the three
schedule artifacts in `crates/jolt-akita/schedules/`, and the guests in
`src/programs.rs`. Properties: **L** liveness (every statement inside the
documented limits preprocesses, proves, and verifies), **S** soundness testing
(invalid proofs and statements are rejected), **R** robustness (no panic or
unbounded resource use on untrusted input).

## Status of this inventory

Honest summary of what has actually run (release builds, not the instrumented
campaign):

| Run | What | Result |
|---|---|---|
| `plan-sweep geometry` | 58 320 shapes | J-4 |
| `plan-sweep advice` | 73 872 shapes | J-3 (and J-2's shapes plan, then fail later) |
| `plan-sweep program` | 38 232 shapes | J-1 |
| `grid-sweep 12 22` | every K16, K256, dense row with 12..22 variables, 3 witnesses | one-hot rows pass (L2 margins 0.97, at most 2 attempts); dense rows fail on the zero polynomial (J-5) |
| `program` seeds | 54 seeds (18 guests, 3 option sets) | 51 pass, J-1 on one, 3 missing ELF (`stdlib`, since removed) |
| `smoke` | 40–300 random inputs per lane | J-1, J-5, and four harness bugs (fixed) |

The instrumented campaign (`jolt-fuzz run`) has **not run yet**: it is paused
pending the campaign host's security review (FINDINGS.md). The `grid-sweep`
zero-polynomial witness and the offloaded rows above 22 variables have not run
either.

## Targets

### `planning` (L)

- **Input:** a production preprocessing shape (`src/shape.rs`): `log_T` in
  `12..=30`, one-hot width (production rule, or forced K=16/K=256), bytecode
  length `2^0..2^26`, RAM domain `2^1..2^40` words, trusted and untrusted
  advice capacities (absent or `2^3..2^40` bytes), and an optional committed
  program (`2^0..2^8` chunks, image words), biased toward documented edges.
- **Path:** Jolt's own geometry (`one_hot_trace_setup_shape`,
  `advice_packing_plan`, `committed_program_packing_plan`) into the setup
  request `jolt_prover::akita::preprocessing::grouped_setup` builds, then
  `provision_precommitted_for_k`. Shapes whose opening has at most
  `JOLT_FUZZ_MAX_CASE_COEFFS` coefficients (default `2^23`) also run
  `AkitaScheme::setup`, precommitted dense commits, the streamed
  `commit_trace_one_hot`, `prove_batch`, verifier-setup and proof transport,
  and `verify_batch`.
- **Oracle:** `Shape::in_contract` states the documented limits with their
  sources; any failure of an in-contract shape is a finding. The streamed trace
  commitment must equal the materialized one-hot polynomial's
  (`commit_one_hot_group_owned*`), whose evaluation is jolt-poly's reference
  `evaluate`.
- **Sweep:** `plan-sweep` enumerates each stage's own inputs completely
  instead of their product (geometry: every chunking, `log_T`, bytecode, and
  RAM combination; advice: every `log_T` with every advice-capacity pair;
  program: every `log_T`, chunk count, and chunk/image arity edge, with and
  without advice).
- **Not covered:** the one mirrored step is the ten-line
  `PrecommittedScheduleParams` assembly of `grouped_setup` (private); the
  `program` target runs the real one. The smallest production opening is
  `2^22` coefficients, so under the default cap only the smallest traces are
  proved per iteration; larger ones only plan.

### `grid` (L)

- **Input:** a row of a loaded artifact (the lane fixes the family), a
  selector capacity, a layout digest, and a witness (`opening::Witness`: zero,
  maximal, random, or sparse values; column count; zero-row mask; point
  pattern).
- **Path:** one-hot 1-polynomial rows through the streamed `OneHotTrace`
  path, the same driver as `planning`; one-hot 2-polynomial and dense-bounded
  rows through `AkitaNativeBatching`'s same-point batch. Proof and verifier
  setup are transported through bincode before verification.
- **Oracle:** every catalog row is a supported statement; any failure is a
  finding. Rows above the cost cap run only in `grid-sweep` (release).
- **Not covered:** rows above 22 variables have not run yet, including every
  setup-offloaded row (K16 from 31 variables, K256 from 34); the 2-polynomial
  one-hot and dense standalone batches are public `jolt-akita` API, not the
  production prover's path.

### `program` (L, S)

- **Input:** a guest (lane `interp`: the interpreter guest in four memory
  layouts; lane `examples`: 13 e2e example guests), its postcard arguments
  and advice from a per-guest generator, and the prover's options: full or
  committed program with `2^0..2^3` chunks, padded-trace bound slack
  (`x1`, `x2`, `x4`), forced K=256, address-first RAM and register binding,
  optimized or reference backend.
- **Interpreter guest:** its input is a program for a small VM that runs the
  exact RISC-V instructions for 32 ALU operations (division and remainder by
  zero, `i64::MIN / -1`, `mulh*`, `W` forms, out-of-range shifts), loads and
  stores of every width over a heap span the program sets (the RAM domain),
  32- and 64-bit atomics, advice reads, bounded nested loops (trace length),
  and a requested panic. Layouts: default (4 KiB advice), no advice, large
  advice (64 MiB trusted, 1 MiB untrusted), 256 MiB heap.
- **Path:** the e2e sequence: `jolt_host::Program` on the prebuilt ELF,
  `JoltProgramPreprocessing`, the modular tracer, `ProverConfig::derive_compact`,
  `preprocess_full_with_advice` or `preprocess_committed_with_advice`,
  `commit_trusted_advice`, `akita::prove` under the liveness oracle, and
  `jolt_verifier::verify`.
- **Oracle:** any failure after tracing is a finding (guest panics are proved
  too). After an accepted proof: bincode transport of the proof and the
  verifier preprocessing must verify, and a flipped public output byte or panic
  flag must be rejected.
- **Not covered:** traces above `2^18` (`JOLT_FUZZ_MAX_LOG_T`), so the K=256
  production width (`log_T >= 25`) is reached only by forcing it; guest memory
  layouts are fixed per guest (attributes are compile-time), so advice and heap
  capacities vary only across the prebuilt layouts; `stdlib` (std mode) is
  excluded because its musl cross toolchain could not be built on the campaign
  host without root; ECDSA, `modinv` (compute-advice builds), and
  `sig-recovery` guests are not included; ZK and Dory modes are out of scope.

### `verifier` (R, S)

- **Input:** an honest bundle (6, proved by `jolt-fuzz-dev bundles`: muldiv
  default, committed with 2 chunks, forced K=256 with address-first binding;
  advice-consumer; interp; interp with large advice), a region, and up to 16
  edits (xor, set, insert, delete, truncate, small delta, splice from another
  bundle's same region).
- **Oracle:** decoding and verification never panic; libFuzzer's malloc limit
  bounds what a decoded verifier setup may allocate while re-deriving keys; a
  bundle that verifies must re-encode to an honest bundle (otherwise
  soundness), byte-identically (otherwise malleability).
- **Not covered:** edits touch one region per input; no cheating prover
  builds a consistent transcript for a false statement, so acceptance checks
  show binding and canonical decoding, not soundness in general.

## Liveness oracle

`src/liveness.rs`, ported from the Akita campaign. A tracing subscriber
captures Akita's fold-response reports (`response-model-diagnostics`) during
every proof and checks, for every fold: the grind never ran out (4096
nonces); the accepted response is within its cap; the L2 margin
`40·mean/(39·cap)` is at most 1; the L-infinity margin under the planner's
Gaussian model is at most 1; attempts stay at or below 1024. Margins and
attempts are also reported to libFuzzer as coverage.

Differences from the Akita campaign:

- capture is process-global, not thread-local, because `jolt-akita` proves
  inside its own backend Rayon pool;
- errors are matched by message, because Jolt wraps `AkitaError` in
  `OpeningsError` text;
- the pinned Akita has no per-probe events and no fault-injection feature, so
  per-probe L-infinity margins and joint multi-group margins (D-1 in the Akita
  findings) are not measured, and no fault-injection target exists.

## What the checks establish, and what they do not

- A passing liveness check covers the sampled statements, not every
  admissible one. The sweeps are exhaustive only over the parameters they
  enumerate, and only for planning; proving runs for sampled shapes.
- The streamed-versus-materialized trace check uses production's own
  materialization order (`TracePackedOneHot` segments). If both shared a
  layout bug, the check would pass.
- Rejection of a changed public output shows binding of the public I/O, not
  algebraic soundness.
- Kernels selected by CPU features run only for the campaign host's CPU
  (AVX-512 on the current host).
- Akita findings already addressed upstream (Akita F-1..F-6, D-1) are not
  re-reported; any Akita finding here is re-checked against Akita `main`
  first.
