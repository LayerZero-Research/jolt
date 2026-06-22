# Spec: Akita Fused Increment PIOP

| Field       | Value              |
|-------------|--------------------|
| Author(s)   | @quangvdao         |
| Created     | 2026-06-22         |
| Status      | accepted           |
| PR          |                    |

## Summary

The current `akita-pcs` path one-hot decomposes register increments and RAM increments separately.
That is correct, but it misses the RISC-V execution invariant that at any cycle only the RAM increment or the register increment can be nonzero.
This spec replaces the Akita-only split increment PIOP with one fused signed increment polynomial, one unsigned-offset decomposition of that fused increment, and an Akita-only intermediate sumcheck stage that proves the relationship to the existing RAM and register increment claims.
The `dory-pcs` PIOP remains unchanged.

## Intent

### Goal

Refactor the `akita-pcs` PIOP so it commits and opens one fused increment decomposition instead of separate RAM and register increment decompositions, while preserving the existing `dory-pcs` protocol and proof shape.

The protocol-level names are:

- `Inc(t)`: the signed fused increment at cycle `t`.
- `UnsignedInc(t) = Inc(t) + 2^64`: the nonnegative 65-bit value committed through the Akita one-hot path.
- `UnsignedIncChunk(j)`: the `j`-th lower chunk of `UnsignedInc`, where each chunk is a one-hot polynomial over `K = 2^log_k_chunk` values and the cycle domain.
- `UnsignedIncMsb`: the top bit of `UnsignedInc`, represented as a size-`T` boolean polynomial, not as a size-`K * T` one-hot polynomial.

Use `UnsignedIncChunk(j)` rather than `UnsignedIncByte(j)` in code and prose.
The chunk width is feature-configured by `log_k_chunk`, and is 4 bits for small traces and 8 bits for large traces.
`Byte` would be inaccurate for the 4-bit path.

### Invariants

- `dory-pcs` keeps the current PIOP: dense committed `RamInc` and `RdInc`, the current `IncClaimReduction`, current stage numbering, current Stage 8 dense increment openings, and current ZK behavior.
- `akita-pcs` is the only path that changes.
- The existing feature split is the boundary: `jolt-core/src/lib.rs:12-20` already makes `dory-pcs` and `akita-pcs` mutually exclusive and rejects `akita-pcs` with `zk`.
- In Akita mode, `RamInc` and `RdInc` are not PCS-committed increment polynomials.
- In Akita mode, `Inc(t)` is defined by:

```text
Inc(t) = RamInc(t) if Store(t) = 1
Inc(t) = RdInc(t)  if Store(t) = 0
```

- Equivalently, the protocol proves:

```text
RamInc(t) = Store(t) * Inc(t)
RdInc(t)  = (1 - Store(t)) * Inc(t)
```

- `Store(t)` is `CircuitFlags::Store`, the bytecode-backed RAM-write flag defined in `crates/jolt-riscv/src/flags.rs:24-34` and mirrored in `jolt-core/src/zkvm/instruction/mod.rs:105-115`.
- A load has `Store(t) = 0`; its RAM increment is zero and its register increment, if any, flows through `Inc(t)`.
- A store has `Store(t) = 1`; its register increment is zero and its RAM increment flows through `Inc(t)`.
- A zero-delta store is allowed: `Store(t) = 1` does not imply `Inc(t) != 0`.
- A non-store cycle with no register write is allowed: `Store(t) = 0` and `Inc(t) = 0`.
- `UnsignedInc(t)` is always `Inc(t) + 2^64`.
- `UnsignedInc(t)` is reconstructed as:

```text
UnsignedInc(t)
  = lower_chunks_value(t) + 2^64 * UnsignedIncMsb(t)
```

- `UnsignedIncMsb` is boolean over the cycle domain.
- `UnsignedIncMsb` must have exactly `T` coefficients and must not be packed as a one-hot size-`K * T` polynomial.
- Each `UnsignedIncChunk(j)` is one-hot over the address chunk domain for every cycle.
- The lower chunks and `UnsignedIncMsb` together encode the same `UnsignedInc` value used by the fused increment virtualization relation.
- Every Akita prover claim formula and verifier claim formula must use the same opening points and batching coefficients.
- If Akita ZK support is added later, the BlindFold constraints for the new Akita stage must match the non-ZK claim formulas exactly.
  That work is out of scope for this spec because `akita-pcs` currently rejects `zk`.

New `jolt-eval` invariants are useful follow-up work but are not required for the first implementation.
Good candidates are:

- `akita_fused_inc_trace_equivalence`: for generated trace cycles, check that the fused `Inc`, `UnsignedInc`, `UnsignedIncChunk`, and `UnsignedIncMsb` witnesses reconstruct the old `RamInc` and `RdInc` witnesses.
- `akita_fused_inc_stage_claims`: compare a direct evaluator for the fused increment virtualization relation against the prover and verifier parameter formulas.

### Security Argument

This is a relative soundness argument for the Akita PIOP change, not a standalone proof of the full Jolt zkVM.
It assumes the existing Jolt PIOP is sound for the Dory dense-increment relation, the sumcheck protocol is sound under Fiat-Shamir, bytecode read-RAF correctly binds bytecode-derived virtual claims to the committed bytecode table, the one-hot booleanity and hamming-weight reductions are sound, and the PCS is binding for all committed polynomial openings.

The target statement is:

```text
If an Akita proof verifies, then every accepted opening claim about the old
dense increment witnesses is consistent with one signed Inc polynomial and
one bytecode-backed Store selector, and every accepted opening claim about
the committed unsigned increment chunks reconstructs UnsignedInc = Inc + 2^64.
```

The argument proceeds in four steps.

First, `IncVirtualization` preserves the old RAM and register increment semantics at all verifier-challenged points.
Its input claim is the same batched scalar that the current Akita path obtains from the old dense `RamInc` and `RdInc` claims.
The new relation expands those dense claims as:

```text
RamInc(t) = Store(t) * Inc(t)
RdInc(t)  = (1 - Store(t)) * Inc(t)
```

By sumcheck soundness, a prover that changes this batched claim without changing the corresponding multilinear relation can succeed only with negligible probability.
At the final challenge `r_cycle_inc`, the verifier records `Inc(r_cycle_inc)` and `Store(r_cycle_inc)`, and the expected output claim is the multilinear evaluation of the same relation at that point.
Thus the old increment claims consumed by stages 2 through 5 are reduced to claims about `Inc` and `Store`.

Second, bytecode read-RAF binds the selector to the instruction table.
`Store(r_cycle_inc)` is not trusted as an unconstrained witness value.
Stage 6 adds a bytecode Val claim for `CircuitFlags::Store` at `r_cycle_inc`, and bytecode read-RAF proves that this claim is the value in the committed bytecode row selected by the execution trace.
Under the existing bytecode read-RAF soundness assumption, the prover cannot choose `Store` independently of the bytecode flag table.
Therefore the selector used in `IncVirtualization` is the same store flag enforced by the R1CS and bytecode constraints.

Third, `UnsignedIncClaimReduction` preserves the offset relation between signed and unsigned increment views.
Its input claim is:

```text
UnsignedInc(r_cycle_inc) = Inc(r_cycle_inc) + 2^64
```

and its sumcheck relation is:

```text
sum_t eq(r_cycle_inc, t) * UnsignedInc(t).
```

By sumcheck soundness, the accepted output claim `UnsignedInc(r_cycle_6)` is consistent with the same committed unsigned-increment witness polynomial used by the later chunk checks.
This is the bridge from the signed virtual relation to the unsigned committed representation.

Fourth, Stage 6 and Stage 7 bind the unsigned representation to the committed chunks.
Stage 6 booleanity constrains each lower `UnsignedIncChunk(j)` to be boolean over the address-plus-cycle domain, and Stage 7 hamming checks constrain each lower chunk to be one-hot over the address chunk domain at `r_cycle_6`.
Stage 6b constrains `UnsignedIncMsb` to be boolean over the cycle domain.
Stage 7 then proves:

```text
UnsignedInc(r_cycle_6) - 2^64 * UnsignedIncMsb(r_cycle_6)
  = sum_j weight_j * sum_a identity(a) * UnsignedIncChunk(j)(a, r_cycle_6).
```

Because the lower chunks are one-hot and boolean, the right-hand side is exactly the lower base-`K` value encoded by the chunk witnesses.
Because `UnsignedIncMsb` is boolean and opened as a size-`T` polynomial, the left-hand side is exactly the lower 64-bit portion of the accepted unsigned increment claim.
The Stage 8 PCS opening proof then binds these values to the committed Akita witness polynomials.

Together, these reductions show that replacing separate Akita `RamInc` and `RdInc` one-hot decompositions with one fused `UnsignedInc` decomposition does not give the prover a new degree of freedom, except with the same negligible soundness error as the underlying sumchecks, Fiat-Shamir challenges, bytecode read-RAF, and PCS openings.

The remaining proof gaps are outside this spec:

- There is no fresh formal proof here of the entire Jolt PIOP or zkVM soundness theorem.
- This spec assumes the existing RAM and register read-write checks correctly produce the old dense `RamInc` and `RdInc` claims consumed by `IncVirtualization`.
- This spec assumes bytecode read-RAF already soundly binds bytecode Val claims to the final bytecode table.
- This spec assumes the existing batched-sumcheck suffix convention for shorter instances is sound and implemented symmetrically by prover and verifier.
- This spec does not prove Akita PCS binding from first principles.
- This spec does not cover Akita ZK mode because the repository currently rejects `akita-pcs` with `zk`.

### Non-Goals

- Do not change the `dory-pcs` PIOP.
- Do not change Dory proof serialization.
- Do not change Dory Stage 6, Stage 7, Stage 8, or BlindFold behavior.
- Do not add a compatibility shim for the old Akita split increment decomposition.
  The Akita path should fully cut over to fused increments.
- Do not port the Markos modular verifier fused Stage 6 relations into `jolt-core`.
- Do not change RISC-V instruction semantics, `CircuitFlags`, RAM access semantics, or register write semantics.
- Do not require Akita ZK support in this work.

## Evaluation

### Acceptance Criteria

- [ ] With `dory-pcs`, `all_committed_polynomials` still includes dense `RdInc` and `RamInc` and does not include `UnsignedIncChunk` or `UnsignedIncMsb`.
- [ ] With `akita-pcs`, `all_committed_polynomials` includes `UnsignedIncChunk(0..lower_chunk_count)` and `UnsignedIncMsb`, and does not include `RdIncRa`, `RdIncMsb`, `RamIncRa`, or `RamIncMsb`.
- [ ] With `akita-pcs`, no Stage 8 opening is requested for dense `RamInc` or dense `RdInc`.
- [ ] With `akita-pcs`, Stage 5 remains responsible for instruction read-RAF and register val evaluation.
- [ ] With `akita-pcs`, `RamRaClaimReduction` moves from Stage 5 into the new Akita intermediate stage.
- [ ] With `akita-pcs`, the new Akita intermediate stage proves the fused increment virtualization relation and outputs claims for `Inc(r_cycle_inc)` and `Store(r_cycle_inc)`.
- [ ] With `akita-pcs`, bytecode read-RAF validates `Store(r_cycle_inc)` against the committed bytecode row's `CircuitFlags::Store` value before later stages rely on it.
- [ ] With `akita-pcs`, Stage 6b reduces `Inc(r_cycle_inc) + 2^64` to `UnsignedInc(r_cycle_6)`.
- [ ] With `akita-pcs`, lower `UnsignedIncChunk(j)` polynomials are boolean-constrained in Stage 6a/6b and one-hot-constrained over the address chunk domain in Stage 7.
- [ ] With `akita-pcs`, `UnsignedIncMsb` booleanity is enforced inside the Booleanity cycle phase at `r_cycle_6`, not as a separate Stage 6b sumcheck instance.
- [ ] With `akita-pcs`, Stage 7 reconstructs `UnsignedInc(r_cycle_6)` from lower `UnsignedIncChunk(j)` openings and the size-`T` `UnsignedIncMsb(r_cycle_6)` opening.
- [ ] With `akita-pcs`, `UnsignedIncMsb` is opened as a size-`T` polynomial at the Stage 6b cycle point, not as an address-plus-cycle one-hot polynomial.
- [ ] Existing Dory standard and Dory ZK `muldiv` tests keep passing.
- [ ] Akita `muldiv_e2e_akita` keeps passing with the fused increment PIOP.

### Testing Strategy

Run Dory correctness and Dory ZK regression tests:

```bash
cargo nextest run -p jolt-core muldiv --cargo-quiet --features host
cargo nextest run -p jolt-core muldiv --cargo-quiet --features host,zk
```

Run Akita correctness:

```bash
cargo nextest run -p jolt-core muldiv_e2e_akita --cargo-quiet --no-default-features --features host,akita-pcs
```

Run strict linting for the supported feature combinations:

```bash
cargo clippy --all --features host -q --all-targets -- -D warnings
cargo clippy --all --features host,zk -q --all-targets -- -D warnings
cargo clippy -p jolt-core --no-default-features --features host,akita-pcs -q --all-targets -- -D warnings
cargo fmt -q
```

Add targeted unit tests for:

- signed `Inc` witness derivation from trace cycles,
- `UnsignedInc = Inc + 2^64`,
- lower chunk extraction for `log_k_chunk = 4` and `log_k_chunk = 8`,
- `UnsignedIncMsb` extraction as a single cycle-domain bit,
- reconstruction of `UnsignedInc` from chunks plus `UnsignedIncMsb`,
- zero-delta stores,
- loads with register increments and zero RAM increments,
- non-store cycles with zero register increments.

### Performance

The expected large-trace Akita packed-polynomial count changes from separate RAM/register increment decompositions to one fused decomposition.
The committed-polynomial enumeration order in `all_committed_polynomials` is normative for the Akita packed layout and must be exactly:

```text
InstructionRa
UnsignedIncChunk
RamRa
BytecodeRa
UnsignedIncMsb
rest (trusted/untrusted advice, bytecode chunks, program image)
```

For `log_k_chunk = 8` this is concretely:

```text
16 InstructionRa
 8 UnsignedIncChunk
at most 4 RamRa
at most 3 BytecodeRa
 1 shared slot for UnsignedIncMsb plus smaller advice/bytecode material
```

`UnsignedIncMsb` is a size-`T` dense polynomial, so it occupies the shared trailing slot rather than a full one-hot lane.
The trailing `rest` material is not exercised by the Akita correctness tests today and may live in separate commitments, so the reorder only has to be correct for the leading one-hot families.
The target is to fit the dominant Akita packed main commitment into 32 size-`256 * T` lanes for large traces.
`UnsignedIncMsb` must not consume a full size-`256 * T` lane by itself.
The RAM and bytecode counts above are practical upper estimates for the current presets, not protocol constants.
If a future parameter choice exceeds those estimates, the Akita layout may add padding lanes.

The new intermediate stage adds proof bytes and transcript work.
That cost is acceptable if it removes the duplicate increment one-hot family and reduces Stage 5 memory pressure by moving `RamRaClaimReduction`.

Useful follow-up `jolt-eval` objectives:

- an Akita prover-time benchmark for `muldiv_e2e_akita`,
- an Akita main commitment lane-count objective,
- an Akita peak-memory objective around stages 5 through 7.

## Design

### Architecture

The Dory path remains structurally unchanged:

```text
dory-pcs
  Stage 5: InstructionReadRaf + RamRaClaimReduction + RegistersValEvaluation
  Stage 6b: existing IncClaimReduction over dense RamInc/RdInc
  Stage 7: existing HammingWeightClaimReduction
  Stage 8: opens dense RamInc/RdInc from IncClaimReduction
```

The Akita path cuts over to fused increments:

```text
akita-pcs
  Stage 5:   InstructionReadRaf + RegistersValEvaluation
  Stage 5i:  RamRaClaimReduction + IncVirtualization
  Stage 6a:  BytecodeReadRafAddressPhase + BooleanityAddressPhase
             including lower UnsignedIncChunk
  Stage 6b:  BytecodeReadRaf cycle phase validates Store(r_cycle_inc)
             + Booleanity cycle phase including lower UnsignedIncChunk
             + UnsignedIncMsb booleanity (folded into Booleanity cycle phase)
             + UnsignedIncClaimReduction
             + existing non-increment reductions
  Stage 7:   HammingWeightClaimReduction over all one-hot RA families,
             now including lower UnsignedIncChunk, plus a single
             increment value-link claim
  Stage 8:   opens lower UnsignedIncChunk at Stage 7 points
             + opens UnsignedIncMsb at Stage 6b cycle point
```

In prose, call the inserted step the increment virtualization stage.
Use `Stage 5i` only as a compact stage label and avoid `5.5` in code identifiers.

The Akita proof adds one `stage5_inc_sumcheck_proof` field under `#[cfg(feature = "akita-pcs")]`.
Dory proof serialization must not gain this field.

### Existing Code Anchors

The current Akita split increment path is concentrated in:

- `jolt-core/src/zkvm/witness.rs:80-107`, where `RdIncRa`, `RdIncMsb`, `RamIncRa`, and `RamIncMsb` are added for one-hot increment mode.
- `jolt-core/src/zkvm/witness.rs:181-211`, where those split increment chunks are streamed into commitments.
- `jolt-core/src/zkvm/witness.rs:302-344`, where those split increment chunks are materialized as witness polynomials.
- `jolt-core/src/zkvm/claim_reductions/increments.rs:76-88`, where the existing increment reduction still models two dense increment polynomials.
- `jolt-core/src/zkvm/claim_reductions/increments.rs:130-217`, where the current input and output claim formulas batch two `RamInc` openings and two `RdInc` openings.
- `jolt-core/src/zkvm/claim_reductions/hamming_weight.rs:207-240`, where stage 7 appends both register and RAM increment one-hot families.
- `jolt-core/src/zkvm/claim_reductions/hamming_weight.rs:322-333`, where stage 7 reads dense `RdInc` and `RamInc` claims and adds the unsigned shift separately for each.
- `jolt-core/src/zkvm/prover.rs:1354-1421`, where current Stage 5 batches `InstructionReadRaf`, `RamRaClaimReduction`, and `RegistersValEvaluation`.
- `jolt-core/src/zkvm/prover.rs:1496-1717`, where current Stage 6b includes `IncClaimReduction`.
- `jolt-core/src/zkvm/prover.rs:2323-2347`, where current Akita Stage 8 opens all four split increment families.
- `jolt-core/src/zkvm/verifier.rs:1798-1834`, where the verifier mirrors those Akita Stage 8 openings.

### Witness And Commitment Surface

In Akita mode, replace these committed-polynomial variants:

```text
RdIncRa(j)
RdIncMsb
RamIncRa(j)
RamIncMsb
```

with:

```text
UnsignedIncChunk(j)
UnsignedIncMsb
```

For each cycle:

```text
store = CircuitFlags::Store(cycle)
ram_inc = post_ram - pre_ram if RAMAccess::Write else 0
rd_inc = post_rd - pre_rd if rd_write exists else 0
Inc = store * ram_inc + (1 - store) * rd_inc
UnsignedInc = Inc + 2^64
UnsignedIncChunk(j) = j-th lower chunk of UnsignedInc
UnsignedIncMsb = bit 64 of UnsignedInc
```

The implementation should derive `store` from the final proof-facing instruction row, not from ad hoc tracer-side RAM access shape.
The witness helper may use trace RAM/register values for `ram_inc` and `rd_inc`, but the protocol binds the selector to bytecode through Stage 6 bytecode read-RAF.

### Increment Virtualization Stage

Add an Akita-only stage between current Stage 5 and Stage 6a.
Call the relation `IncVirtualization`.

This stage is a batched sumcheck with `max_rounds = log_T`.

The stage contains:

- `RamRaClaimReduction`, moved from Stage 5 for Akita only.
- `IncVirtualization`, a degree-3 cycle-only relation.

The fused increment input claim is:

```text
v1 + gamma * v2 + gamma^2 * w1 + gamma^3 * w2
```

where:

```text
v1 = RamInc(r_cycle_stage2)
v2 = RamInc(r_cycle_stage4)
w1 = RdInc(s_cycle_stage4)
w2 = RdInc(s_cycle_stage5)
```

The sumcheck relation is:

```text
sum_t Inc(t) * (
    (eq(r_cycle_stage2, t) + gamma * eq(r_cycle_stage4, t)) * Store(t)
  + gamma^2 * (eq(s_cycle_stage4, t) + gamma * eq(s_cycle_stage5, t)) * (1 - Store(t))
)
```

At the final cycle point `r_cycle_inc`, this stage caches:

```text
Inc(r_cycle_inc)
Store(r_cycle_inc)
```

The output claim for `IncVirtualization` is computed from the cached `Inc(r_cycle_inc)` and `Store(r_cycle_inc)` claims:

```text
Inc(r_cycle_inc) * (
    (eq(r_cycle_stage2, r_cycle_inc) + gamma * eq(r_cycle_stage4, r_cycle_inc)) * Store(r_cycle_inc)
  + gamma^2 * (eq(s_cycle_stage4, r_cycle_inc) + gamma * eq(s_cycle_stage5, r_cycle_inc)) * (1 - Store(r_cycle_inc))
)
```

### Stage 6 Changes For Akita

Stage 6 must validate the `Store(r_cycle_inc)` claim produced by the increment virtualization stage and reduce the signed increment claim into an unsigned increment claim at the Stage 6b cycle point.

In Akita mode only:

- Extend bytecode read-RAF parameter generation to include one additional staged bytecode claim for `CircuitFlags::Store`.
- The additional bytecode Val polynomial is:

```text
Val_inc(k) = CircuitFlags::Store(k)
```

- Its opening point is the increment virtualization stage cycle point `r_cycle_inc`.
- Its claimed value is `Store(r_cycle_inc)`.
- The Stage 6 bytecode read-RAF input claim includes this new staged claim with a fresh batching coefficient.
- The prover computes this Val column from the final bytecode row's circuit flags.
- The verifier computes the same claim formula from the opening accumulator and the same transcript-derived batching coefficient.

Stage 6a and Stage 6b booleanity include the lower `UnsignedIncChunk(j)` polynomials.
They use the same split address/cycle booleanity structure as the current one-hot RA families:

```text
0 = sum_{a,t} eq(r_addr_bool, a) * eq(r_cycle_6, t)
      * sum_j gamma_j * (UnsignedIncChunk(j)(a,t)^2 - UnsignedIncChunk(j)(a,t))
```

Stage 6b folds `UnsignedIncMsb` booleanity into the existing Booleanity cycle phase.
It is not a separate Stage 6b sumcheck instance.
The cycle relation adds a batched term over the same `r_cycle_6` challenges as the lower-chunk booleanity cycle pass:

```text
0 = sum_t eq(r_cycle_6, t)
      * (UnsignedIncMsb(t)^2 - UnsignedIncMsb(t))
```

The Booleanity cycle implementation should reuse the materialized `UnsignedIncMsb` witness column from fused increment generation and avoid a second trace pass.
Because `UnsignedIncMsb` is already a size-`T` boolean polynomial, the prover may use specialized handling that exploits the `{0,1}` coefficient domain; a generic dense booleanity term is also acceptable if it is simpler.

Stage 6b replaces the current Akita use of `IncClaimReduction` with `UnsignedIncClaimReduction`.
Its input claim is:

```text
UnsignedInc(r_cycle_inc) = Inc(r_cycle_inc) + 2^64
```

Its relation is:

```text
sum_t eq(r_cycle_inc, t) * UnsignedInc(t)
```

where:

```text
UnsignedInc(t) = Inc(t) + 2^64
```

The relation is cycle-only and active on the Stage 6b cycle rounds.
At the final Stage 6b cycle point `r_cycle_6`, it caches:

```text
UnsignedInc(r_cycle_6)
UnsignedIncMsb(r_cycle_6)
UnsignedIncChunk(j)(r_addr_bool, r_cycle_6) for each lower chunk j
```

Stage 6 still handles the existing bytecode read-RAF, RAM hamming booleanity, RAM RA virtualization, instruction RA virtualization, advice reductions, bytecode reductions, and program-image reductions.
The only Stage 6 changes are the added `Store(r_cycle_inc)` bytecode claim, lower `UnsignedIncChunk(j)` booleanity, `UnsignedIncMsb` booleanity, and `UnsignedIncClaimReduction`.

### Stage 7 Changes For Akita

Because `UnsignedIncClaimReduction` runs inside the Stage 6b batched sumcheck alongside Booleanity, its final cycle point is the same `r_cycle_6` shared by every one-hot RA family.
The lower `UnsignedIncChunk(j)` polynomials therefore live at exactly the same cycle point as `InstructionRa`, `RamRa`, and `BytecodeRa`, so they fold into the single existing `HammingWeightClaimReduction` sumcheck rather than a separate increment-only sumcheck.
This matches how the current Akita code already folds `RdIncRa` and `RamIncRa` into `HammingWeightClaimReduction`; the change is that the two families collapse to one `UnsignedIncChunk` family and the two old value-link claims collapse to one.

Concretely, in `HammingWeightClaimReduction`:

- The polynomial list replaces `RdIncRa(j)`, `RamIncRa(j)`, `RdIncMsb`, `RamIncMsb` with lower `UnsignedIncChunk(j)` only.
- Each lower `UnsignedIncChunk(j)` keeps the existing per-family hamming, booleanity, and address-virtualization batched terms, with hamming claim public value `1`:

```text
sum_a UnsignedIncChunk(j)(a, r_cycle_6) = 1
```

- The two old per-family value-link claims (`rd_inc_claim + 2^64`, `ram_inc_claim + 2^64`) are replaced by a single value-link claim:

```text
lower_value_claim
  = UnsignedInc(r_cycle_6) - 2^64 * UnsignedIncMsb(r_cycle_6)
```

reconstructed as:

```text
lower_value_claim
  = sum_j weight_j * sum_a identity(a) * UnsignedIncChunk(j)(a, r_cycle_6)
```

where `weight_j` is the positional base-`K` weight for lower chunk `j`.
The fused increment chunk weights drop the old `2^64` MSB weight, because the MSB contribution is now subtracted explicitly on the left-hand side rather than carried as a top chunk weight.

After this sumcheck, each lower `UnsignedIncChunk(j)` has a single opening at `(r_addr_stage7, r_cycle_6)`, the same address-plus-cycle point as the other RA families.

`UnsignedIncMsb` does not participate in Stage 7 address reduction.
It is already a size-`T` polynomial and is opened directly at `r_cycle_6` in Stage 8.

### Stage 8 Changes For Akita

In Akita mode, Stage 8 opens:

- every non-increment committed polynomial required by the existing protocol,
- every lower `UnsignedIncChunk(j)` at its Stage 7 address-plus-cycle point,
- `UnsignedIncMsb` at the Stage 6b cycle point with `num_vars = log_T`.

In Akita mode, Stage 8 must not open:

- dense `RamInc`,
- dense `RdInc`,
- `RdIncRa(j)`,
- `RdIncMsb`,
- `RamIncRa(j)`,
- `RamIncMsb`.

In Dory mode, Stage 8 remains unchanged and still opens dense `RamInc` and dense `RdInc` from the existing increment claim reduction.

### Proof Serialization

Add an Akita-only proof field:

```rust
#[cfg(feature = "akita-pcs")]
pub stage5_inc_sumcheck_proof: SumcheckInstanceProof<F, C, FS>,
```

Do not add this field to Dory proofs.

In Akita mode, prover orchestration is:

```text
stage1
stage2
stage3
stage4
stage5
stage5_inc
stage6a
stage6b
stage7
stage8
```

In Dory mode, prover orchestration remains:

```text
stage1
stage2
stage3
stage4
stage5
stage6a
stage6b
stage7
stage8
```

The verifier must mirror the same feature-gated order.

### Alternatives Considered

The first alternative was to push the virtualization directly into Stage 5 by substituting `RdInc = (1 - Store) * Inc` inside register val evaluation and adding separate virtualization sumchecks for earlier `RamInc` and `RdInc` claims.
That avoids a new stage, but it makes Stage 5 higher degree, increases surgery in the current highest-memory stage, and makes the bytecode flag dependency harder to isolate.

The second alternative was to keep the current split Akita increment decomposition and rely on sparsity.
That is correct but leaves the packed polynomial layout in an awkward 38-40 lane range for large traces.

The chosen design adds an Akita-only intermediate stage.
It is easier to specify and review, isolates the bytecode-backed `Store` dependency, and cuts over the Akita commitment layout to one fused increment decomposition.

## Documentation

No Jolt book update is required for the first implementation because this is an internal Akita PCS PIOP change and Akita is not the default SDK path.

Update or add developer-facing documentation under `specs/` if later work exposes Akita PCS as a supported public proving mode.

## Execution

Implement in this order:

1. Rename and refactor the Akita increment witness surface from split `RdInc*` and `RamInc*` families to `UnsignedIncChunk(j)` and `UnsignedIncMsb`.
2. Keep Dory committed polynomial enumeration and Dory Stage 8 opening logic unchanged.
3. Add the Akita-only `stage5_inc_sumcheck_proof` field and prover/verifier orchestration.
4. Move `RamRaClaimReduction` out of Stage 5 only under `akita-pcs`.
5. Implement `IncVirtualization`.
6. Extend Stage 6 booleanity with lower `UnsignedIncChunk(j)` and fold `UnsignedIncMsb` booleanity into the Booleanity cycle phase.
7. Extend Akita bytecode read-RAF to validate `Store(r_cycle_inc)`.
8. Replace Akita's old dense increment reduction in Stage 6b with `UnsignedIncClaimReduction`.
9. Update Stage 7 so lower increment chunks use `r_cycle_6` and `UnsignedIncMsb` is excluded from address reduction.
10. Update Akita Stage 8 opening collection for lower chunks and size-`T` `UnsignedIncMsb`.
11. Add focused witness and claim-formula tests before running e2e tests.

## References

- `jolt-core/src/lib.rs:12-20`: feature gating for `dory-pcs`, `akita-pcs`, and Akita ZK rejection.
- `jolt-core/src/zkvm/mod.rs:71-82`: backend-specific `F` and `PCS` aliases.
- `jolt-core/src/poly/commitment/commitment_scheme.rs:331-337`: default `uses_onehot_inc() = false`.
- `jolt-core/src/poly/commitment/akita/commitment_scheme.rs:1240-1246`: Akita `uses_onehot_inc() = true`.
- `jolt-core/src/zkvm/witness.rs:25-41`: current unsigned-offset helpers for split RAM/register increments.
- `jolt-core/src/zkvm/witness.rs:80-107`: current committed polynomial enumeration for split Akita increments.
- `jolt-core/src/zkvm/claim_reductions/increments.rs:1-47`: current dense two-increment reduction documentation.
- `jolt-core/src/zkvm/claim_reductions/hamming_weight.rs:120-149`: current increment chunk weights.
- `jolt-core/src/zkvm/prover.rs:710-727`: current prover stage order.
- `jolt-core/src/zkvm/verifier.rs:491-514`: current verifier stage order.
- `crates/jolt-riscv/src/flags.rs:24-34`: `CircuitFlags::Store`.
