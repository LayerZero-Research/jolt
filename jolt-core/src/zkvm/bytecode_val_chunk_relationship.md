# Bytecode Val Polys vs Committed Bytecode Polynomial

This note records the exact relationship between:

- Stage-6a bytecode `Val` polynomials (from `bytecode/read_raf_checking.rs`)
- the single committed bytecode polynomial `CommittedPolynomial::Bytecode`
- the two-phase bytecode claim reduction (`claim_reductions/bytecode_single.rs`)

It is focused on `ProgramMode::Committed`.

## 1) What is committed

Committed bytecode uses one polynomial over a fixed 512-lane address domain:

- lane domain size = `512`
- cycle domain size = `bytecode_len`
- total coefficients = `512 * bytecode_len`

For each cycle `k`, canonical bytecode lanes occupy the prefix:

- real lanes: `[0, total_lanes())` (`total_lanes() = 448`)
- padded lanes: `[total_lanes(), 512)` are zero

In code this lives in:

- `TrustedProgramCommitments::bytecode_commitment` in `zkvm/program.rs`
- `CommittedPolynomial::Bytecode` in `zkvm/witness.rs`

## 2) Lane-basis definition

Define:

`B(l, k)` = committed bytecode value at lane `l` and cycle `k`.

By construction:

- for `l < total_lanes()`: `B(l, k)` equals canonical lane value of instruction `k`
- for `l >= total_lanes()`: `B(l, k) = 0`

## 3) Val polynomials are linear forms over the same lane basis

In Stage 6a, five val polynomials are built (`Val_0` .. `Val_4`).
For each stage `s`, there exist sparse lane weights `w_s(l)` such that:

`Val_s(k) = sum_{l=0..511} w_s(l) * B(l, k)`.

The reduction folds these with challenge `eta`:

`W_eta(l) = sum_{s=0..4} eta^s * w_s(l)`.

`W_eta` is stored as `lane_weights` in `BytecodeClaimReductionParams`.

## 4) Exact claim transformation

Input claim at Stage 6b:

`C_in = sum_{s=0..4} eta^s * Val_s(r_bc)`.

Define:

`S(k) = sum_{l=0..511} W_eta(l) * B(l, k)`.

Then:

`C_in = sum_k Eq(r_bc, k) * S(k)`.

### 4.1 Cycle phase

The cycle-phase sumcheck proves the equation above and binds cycle vars to `r_cycle`,
yielding:

`C_mid = Eq(r_cycle, r_bc) * S(r_cycle)`.

This is cached as `VirtualPolynomial::BytecodeClaimReductionIntermediate`.

### 4.2 Address phase

Let:

- `BytecodeOpen := B(r_addr, r_cycle)` (opening of committed bytecode polynomial)
- `W_eta(r_addr) := sum_l W_eta(l) * Eq(r_addr, l)`

Then verifier checks:

`C_mid = Eq(r_cycle, r_bc) * BytecodeOpen * W_eta(r_addr)`.

This is exactly `BytecodeClaimReductionVerifier::expected_output_claim` in address phase.

## 5) Stage 8 batching

Stage 8 uses independent `gamma` powers to batch many openings.
These are unrelated to `W_eta` and only affect joint opening batching.

## 6) Where to look in code

- Val construction: `zkvm/bytecode/read_raf_checking.rs`
- Committed bytecode polynomial builder: `zkvm/bytecode/chunks.rs`
- Bytecode claim reduction: `zkvm/claim_reductions/bytecode_single.rs`
- Committed-bytecode commitment derivation: `zkvm/program.rs`
