# Akita PCS Integration into Jolt

Tracking document for replacing Dory (pairing-based) with Akita (lattice-based) PCS in Jolt.

Branch: `lz/integrate-akita`

---

## Syncing with upstream `main`

Operational policy + conflict ledger for merging `origin/main` into the Akita branch.
Merge tooling: `rerere.enabled=true`, `merge.conflictStyle=zdiff3`. Resolve on the
throwaway branch `merge/main-sync-scratch` first, then replay onto `lz/integrate-akita`.

### Decision (locked): Akita runs `ProgramMode::Full` only — committed subsystem **excised**

The committed-bytecode / program-image path that `main` introduced is **Dory-only** and
is **excised** from this merge (not carried compile-only).

- **Why excised, not compile-only:** main generalized advice into a `precommitted`
  polynomial subsystem (`PrecommittedPolynomial`, `claim_reductions/{precommitted,program_image}.rs`)
  that threads `precommitted_polys` through `rlc::new_streaming` / `build_streaming_rlc`.
  That path calls `OneHotPolynomial` **without** a layout parameter, whereas Akita's
  layout abstraction (chosen below) requires one. The two are mutually exclusive in the
  same `rlc`, so keeping the committed path compiling would force a full union of two
  ~1000-line `rlc` versions with no test baseline. Since Akita can't use committed
  bytecode anyway (Ajtai commitments are non-homomorphic), we drop it.
- **What is removed:** the two new files `claim_reductions/precommitted.rs` and
  `claim_reductions/program_image.rs`, their `mod.rs` wiring, and all `precommitted_polys`
  threading in `opening_proof.rs` / `rlc_polynomial.rs` / `prover.rs` / `witness.rs`.
  Akita keeps its pre-main `advice_polys` (raw `MultilinearPolynomial`) path, which carries
  the same advice data minus main's commitment-reuse wrapper (an optimization, not a
  correctness requirement).
- **Re-introduce later:** when Akita supports committed bytecode, re-sync this subsystem
  on top of the layout abstraction as a dedicated workstream.

### Decision (locked): `JoltSharedPreprocessing` — keep Akita's non-generic struct, excise main's committed-program preprocessing

`main` deeply refactored preprocessing around a committed-program subsystem:
`JoltSharedPreprocessing<PCS>` with `program: ProgramPreprocessing<PCS>` (a `Full`/`Committed`
enum that absorbs `ram: RAMPreprocessing` inside `FullProgramPreprocessing`),
`program_meta: ProgramMetadata`, and `bytecode_chunk_count`. It also binds a
`preprocessing_digest` into Fiat-Shamir.

We **keep Akita's** non-generic struct and **excise** main's committed-program preprocessing:

```
pub struct JoltSharedPreprocessing {
    pub bytecode: Arc<BytecodePreprocessing>,
    pub ram: RAMPreprocessing,
    pub memory_layout: MemoryLayout,
    pub max_padded_trace_length: usize,
}
```

- **Why keep Akita's:** the merged `fiat_shamir_preamble` already settled to Akita's
  **5-arg** form (`program_io, ram_K, trace_length, entry_address, transcript`) — no
  `program_meta`/digest/config binding. Akita's verification path is therefore
  self-consistent *without* main's committed-program metadata. Akita's own code accesses
  `shared.bytecode` / `shared.ram` (matching this struct), so this is the **take-HEAD**
  side for preprocessing hunks — Akita code stays untouched; only main's additions are
  dropped.
- **What is removed:** `ProgramPreprocessing` enum, `FullProgramPreprocessing`,
  `ProgramMetadata`, `bytecode_chunk_count`, the `<PCS>` generic on the struct, the
  `committed_program_prover_data` field on `JoltProverPreprocessing`, the
  `preprocessing_digest` Fiat-Shamir binding, and all `ProgramMode::Committed` branches
  that consume them. `entry_address` comes from `shared.bytecode.entry_address` (Akita),
  not `shared.program_meta.entry_address` (main).
- **Resolution rule:** for any `prover.rs` / `verifier.rs` / `transpilable_verifier.rs`
  hunk that is committed-program preprocessing machinery → take Akita's HEAD side / drop
  main's addition. `ProgramMode` enum itself stays; only `Committed` wiring is excised.

### Decision (locked): layout convention — Option A (keep Akita's `MatrixLayout`)

`one_hot_polynomial.rs` / `rlc_polynomial.rs` carry an Akita abstraction (`poly/matrix_layout.rs`)
that threads an explicit `MatrixLayout` through `num_rows`/`from_indices`/`commit_rows`/
`vector_matrix_product`/`new_streaming`, bridged from Dory via `DoryGlobals::matrix_layout()`.
We **keep** this (vs main's `DoryGlobals`-global convention).

- **Verified equivalent for Dory:** `MatrixLayout::address_cycle_to_index` gives
  AddressMajor = `cycle*K + addr`, CycleMajor = `addr*T + cycle`; main's general path
  (Full mode, `extra_vars=0`) computes the same indices. So Akita's matrix matches main's,
  which is why main's **no-reorder** Dory `prove`/`verify` is consistent with Akita's
  indexing (we removed `reorder_opening_point_for_layout`, an Akita shim slated for removal).
- main's only divergent one_hot logic is the AddressMajor embedding strides
  (`dense_stride`/`one_hot_stride`) — a committed-mode feature that is a no-op in Full mode.

### Dependency resolution: `spongefish` / `sha3` (decided — Option A)

The merge created a real version fork. `jolt-core` ended up needing two incompatible
`sha3` versions at once:

- direct `sha3.workspace` pin `=0.11.0-rc.9` (inherited from `main`), and
- `sha3 0.11.0` transitively via `akita-transcript → spongefish 0.7.0`.

Cargo will not unify a release (`0.11.0`) with its own pre-release (`0.11.0-rc.9`), so
the workspace must collapse to one version.

- `main`'s workspace `spongefish 0.6.1` (which forces `sha3 0.11.0-rc.9`) is used **only**
  by the parallel crates `crates/jolt-transcript` and `jolt-eval` — **not** by `jolt-core`.
- Akita's external `akita-transcript` crate pins `spongefish 0.7.0`
  (`git rev d2d190b1329d35ac9577438d05aed4f17a57b9f9`), which forces `sha3 0.11.0`.

**Decision (Option A):** unify the whole workspace on `spongefish 0.7.0` (akita's git rev)
and `sha3 = "=0.11.0"`. Implemented in root `Cargo.toml`. Result: every `spongefish`
consumer agrees on one `sha3`; `jolt-core` resolves and builds.

**Deferred cost:** `crates/jolt-transcript` and `jolt-eval` were written for the
`spongefish 0.6.1` API and may need small `0.6.1 → 0.7.0` fixups. Neither is a `jolt-core`
dependency, so this does not block the `muldiv` gate — fix as a follow-up.

### Resolution rule

- Any conflict hunk that is purely committed-bytecode / program-image / `precommitted`
  machinery → **excise** (drop main's addition; take Akita's pre-main shape). Do not
  carry `PrecommittedPolynomial` / `precommitted_polys` into the Akita tree.
- `ProgramMode` itself stays (Full is upstream default); only the `Committed` wiring and
  the precommitted subsystem are removed.
- Akita-specific logic lives in the `else { /* Full */ }` branch, the streaming /
  mega-commitment path, the one-hot inc path, and the `MatrixLayout` threading — that is
  where real reconciliation happens.
- Where the layout convention conflicts, keep Akita's `MatrixLayout` API (Option A).

### Conflict ledger (scratch branch, 23 files)

Resolve bottom-up; `prover.rs` / `verifier.rs` last. Classes: **MECH** mechanical ·
**LEAF** take-incoming · **PCS** PCS/trait reconcile · **WIT** witness/claim-layout
reconcile · **ORCH** orchestration (resolve last).

| Ord | File | Hunks | Class | Action |
|----|------|------|------|--------|
| 1 | `.config/nextest.toml` | 1 | MECH | Union both sides |
| 1 | `Cargo.lock` | 3 | MECH | Regenerate via `cargo` after `Cargo.toml` settles |
| 2 | `tracer/src/emulator/cpu.rs` | 1 | LEAF | Take incoming (correctness fix) |
| 2 | `tracer/src/instruction/divuw.rs` | 1 | LEAF | Take incoming |
| 2 | `tracer/src/instruction/remuw.rs` | 1 | LEAF | Take incoming |
| 3 | `poly/commitment/dory/mod.rs` | 1 | LEAF | Take incoming; keep Akita re-exports |
| 3 | `poly/commitment/dory/tests.rs` | 1 | LEAF | Take incoming |
| 3 | `poly/commitment/dory/dory_globals.rs` | 1 | PCS | Take incoming; preserve Akita layout hooks |
| 3 | `subprotocols/booleanity.rs` | 1 | LEAF | Take incoming (main rewrite; Akita delta trivial) |
| 6 | `zkvm/transpilable_verifier.rs` | 3 | ORCH | **Reclassified from LEAF.** Coupled to verifier.rs: advice ctor signature + `fiat_shamir_preamble` (`program_meta.entry_address` + dropped config args). Resolve with verifier.rs |
| 5 | `zkvm/claim_reductions/advice.rs` | 5 | WIT | **Reclassified from LEAF — Akita-critical.** Ours threads explicit `log_k_chunk` (dynamic chunk policy); main *rewrote* advice via `precommitted_scheduling_reference` + `max_advice_size`. Must reconcile both, preserving Akita `log_k_chunk` + the `input_claim` reconstruction invariant |
| 4 | `poly/opening_proof.rs` | 1 | PCS | Reconcile; keep Akita accumulator additions |
| 4 | `poly/commitment/commitment_scheme.rs` | 1 | PCS | Keep Akita trait API + main's `CanonicalSerialize` bound |
| 4 | `poly/commitment/dory/commitment_scheme.rs` | 5 | PCS | Keep Akita hooks + main Dory ZK/mixed-mode fixes |
| 4 | `poly/one_hot_polynomial.rs` | 6 | PCS | Akita layout abstraction vs main; Akita-critical |
| 4 | `poly/rlc_polynomial.rs` | 4 | PCS | **Take Akita (`--ours`).** advice+layout API; excise main's `precommitted`. Forced by Akita one_hot (layout param) being incompatible with main rlc |
| 4 | `zkvm/config.rs` | 1 | PCS | Reconcile `OneHotConfig`/`ReadWriteConfig` (not `ProgramMode`) |
| 5 | `zkvm/witness.rs` | 6 | WIT | Keep **both** variant sets; gate by `onehot_inc` / Full; Akita-critical |
| 5 | `zkvm/claim_reductions/hamming_weight.rs` | 2 | WIT | Akita inc fusion vs main; Akita-critical |
| 5 | `zkvm/mod.rs` | 2 | WIT | Entry/preprocess wiring; ensure Akita selects Full |
| 5 | `zkvm/proof_serialization.rs` | 3 | WIT | Reconcile `JoltProof` shape (both sides) |
| 6 | `zkvm/prover.rs` | 19 | ORCH | Committed hunks take-incoming; Full / streaming / mega / inc reconcile |
| 6 | `zkvm/verifier.rs` | 13 | ORCH | Same as prover |

**Excision impact (committed subsystem removed):** delete `claim_reductions/precommitted.rs`
and `claim_reductions/program_image.rs`; drop their decls/re-exports from
`claim_reductions/mod.rs`; remove `precommitted_polys` threading from `opening_proof.rs`,
`rlc_polynomial.rs`, `prover.rs`, `witness.rs`. `advice.rs` reverts to Akita's `log_k_chunk`
path (main's `precommitted_scheduling_reference` rewrite is dropped).

**PCS tier resolved so far:** `commitment_scheme.rs`, `dory/commitment_scheme.rs`,
`one_hot_polynomial.rs` (Akita), `opening_proof.rs` (being re-fixed to advice+layout).

Validation gates after each layer: `cargo clippy -p jolt-core` in `host` and `host,zk`;
then `muldiv` e2e in both modes; watch `advice` tests for the input-claim invariant.

### Post-merge correctness fixes (after conflict resolution)

Two runtime bugs surfaced once the tree compiled. Both stem from mixing main's evolved
protocol structure with Akita's reverted helpers; neither is caught by `clippy`.

1. **ZK Fiat-Shamir mismatch — verifier missing `bind_opening_inputs`** (`zkvm/verifier.rs`,
   `verify_stage8`). The prover binds the joint opening to the transcript after `batch_prove`
   (`bind_opening_inputs_zk` with `y_com` in ZK, `bind_opening_inputs` with `joint_claim`
   otherwise). Akita's verifier never mirrored this. In **standard** mode the append is
   harmless (no later squeeze reads it, so `muldiv --features host` passes), but in **ZK**
   mode the very first BlindFold squeeze (the Nova fold challenge `r`) derives from that
   state — so the verifier computed a different `r` and `BatchedSumcheck`/BlindFold replay
   panicked with "Fiat-Shamir transcript mismatch" at the first BlindFold round. Fix: after
   `batch_verify`, mirror the prover's bind (cfg-gated: `bind_opening_inputs_zk` / recompute
   `joint_claim = Σ γ^i · claimᵢ` then `bind_opening_inputs`). main's verifier had this; the
   resolution had kept Akita's bind-less `verify_stage8`.

2. **Advice `round_offset` underflow under the Stage 6 split** (`claim_reductions/advice.rs`).
   Akita's two-phase advice reduction (kept over main's `precommitted` rewrite) aligns its
   `CycleVariables` phase to "the start of Booleanity's cycle segment" via
   `(max_num_rounds − (log_k_chunk + log_t)) + log_k_chunk`. main split Booleanity into an
   address phase (Stage 6a) and a cycle phase (Stage 6b); in the 6b batch Booleanity now
   contributes only `log_t` cycle rounds, so `max_num_rounds < log_k_chunk + log_t` and the
   intermediate subtraction underflowed (`attempt to subtract with overflow`). The intended
   result is unchanged (`max_num_rounds − log_t`); fix computes it directly in both the
   prover and verifier `round_offset`, avoiding the underflow. Caught by `advice` e2e tests.

### Workspace-wide sweep (beyond `jolt-core`)

`jolt-core` compiled and passed first, but `cargo clippy --all` surfaced breakage in other
workspace members that the committed-program excision and spongefish re-pin hadn't reached:

3. **Committed-program excision into the SDK / examples / tooling.** `JoltSharedPreprocessing`
   was reverted to Akita's non-generic, Full-only struct (`bytecode, ram, memory_layout,
   max_padded_trace_length`) and `CommittedProgramProverData` / `new_committed` were removed,
   but several crates still referenced them:
   - `jolt-sdk/src/host_utils.rs` re-exported `CommittedProgramProverData`; the
     `#[jolt::provable]` macro (`jolt-sdk/macros`) generated `preprocess_shared_committed_*` /
     `preprocess_committed_*` and a stale 3-arg `JoltSharedPreprocessing::new`. Dropped the
     committed generators and rewired `make_preprocess_shared_func` to the 5-arg
     `new(bytecode, memory_layout, memory_init, max_trace_length, e_entry)?`.
   - `examples/{muldiv,fibonacci,recursion}` had a `--committed-bytecode` path; collapsed to
     the Full path and removed the threaded `bytecode_chunk_count`.
   - `src/build_wasm.rs` and `transpiler/src/main.rs` used the old `ProgramPreprocessing::Full/
     Committed` enum and generic `JoltSharedPreprocessing<PCS>`; since Akita's struct is
     non-generic and PCS-independent, the transpiler's symbolic preprocessing is now a clone
     of the real `.shared` and bytecode words read from `.shared.ram.bytecode_words`.

4. **`jolt-eval` spongefish/arkworks version skew** (`invariant/transcript_symmetry.rs`). The
   merge re-pinned `spongefish` to akita-transcript's rev (0.7.0), whose workspace pins
   arkworks `^0.6`, while Jolt runs the 0.5 `dev/twist-shout` fork via `[replace]`. spongefish's
   blanket `impl Encoding/Decoding for ark_ff::Fp` is therefore over a *different* `Fp` than
   Jolt's `Fr`, so `prover.public_message(&ArkFr)` / `verifier_message::<ArkFr>()` no longer
   resolved. Fix mirrors how `jolt-transcript` actually feeds the sponge in production: absorb
   scalars as canonical LE bytes via `BytesMsg` (`CanonicalBytes::to_bytes_le_vec`) and squeeze
   challenges as `[u8; 32]` → `Fr::from_le_bytes_mod_order`. The prover/verifier consistency
   property is unchanged; all `transcript_prover_verifier_consistency_*` invariants pass.

Post-sweep validation: `cargo clippy --all` clean with `-D warnings` in **both** `host` and
`host,zk`; `jolt-eval` 94/94; `muldiv` and `advice` e2e pass in both modes; `tracer` / `common`
/ `jolt-sdk` / `jolt-platform` 128/129 (the lone failure, `jolt-sdk verify_proof`, depends on
gitignored `tests/fixtures/` and fails on any clean checkout); the `muldiv` example runs the
full guest→preprocess→prove→verify pipeline (`valid: true`).

---

## Architecture

### Mega-polynomial approach

All main witness polynomials are committed as a single Akita commitment using `log₂(N)` selector variables. Given N polynomials P_0, ..., P_{N-1}, the mega-polynomial is:

```
P*(x, y) = Σ_i eq(y, i) · P_i(x)    where y ∈ {0,1}^{log₂ N}
```

**Batch opening** reduces to a single Akita opening: after Stages 1–7 produce claims v_i = P_i(r), the verifier samples ρ ∈ F^{log₂ N} and the combined claim is `P*(r, ρ) = Σ_i eq(ρ, i) · v_i`. One Akita proof suffices.

This eliminates the need for homomorphic batching (`combine_commitments` / `combine_hints`), which is impossible for Ajtai commitments due to nonlinear gadget decomposition G^{-1}.

### Increment → one-hot conversion

RamInc and RdInc (currently dense i128 polynomials) are converted to one-hot format via offset encoding: `inc + 2^64` maps the signed range to unsigned, then decomposed into `d_inc = ⌈65/8⌉ = 9` chunks of 8-bit one-hot polynomials per increment type (18 new committed RA polynomials total).

The IncClaimReduction sumcheck is replaced by a fused RAF-style evaluation sumcheck:
- **Stage 6 (cycle phase):** produces pushforward polynomials G_inc_l(k) for each chunk
- **Stage 7 (address phase):** evaluates `Σ_k G_inc_l(k) · unmap_inc_l(k)` fused with HammingWeight

This eliminates the `lagrange_factor` workaround in Stage 8.

### Advice handling (deferred)

Advice polynomials (TrustedAdvice, UntrustedAdvice) remain on separate Dory commitments for the first pass. They have a different lifecycle (committed outside streaming pipeline) and can be opened via a separate proof. Converting advice to one-hot and joining the mega-commitment is a future workstream.

---

## Status

### Phase 0 — Trait refactoring (DONE)

- [x] `CommitmentScheme` trait: `&self` instance methods, `Config` associated type, `from_proof()`, `config()`, `Default` supertrait
- [x] All implementations updated: Dory, Mock, HyperKZG
- [x] All call sites updated: prover, verifier, witness, SDK macros, benches
- [x] `DoryBatchedProof` wraps `ArkDoryProof` + `DoryLayout`
- [x] `balanced_sigma_nu` extracted as standalone function
- [x] Partial DoryGlobals removal from `commit`, `prove`, `process_chunk`, `aggregate_chunks`
- [x] Merged into `lz/integrate-akita`, clippy clean

### Phase 1 — Prerequisites

- [ ] Abstract layout out of `OneHotPolynomial`
  - Replace `DoryGlobals::get_num_columns()`, `get_layout()`, `get_T()` with explicit parameters
  - `commit_rows`, `vector_matrix_product`, `num_rows` need layout parameter
  - ~15 DoryGlobals call sites in `one_hot_polynomial.rs`
- [ ] Abstract layout out of `RLCPolynomial`
  - ~11 DoryGlobals call sites
  - Akita path won't use RLCPolynomial (no homomorphic RLC), but need clean separation
- [ ] Add `streaming_layout()` to `StreamingCommitmentScheme` trait
  - Returns chunk size, alignment, num_chunks for a given polynomial length
  - Replaces `DoryGlobals::get_num_columns()` in `prover.rs:604`
- [ ] Move `dory_layout` from `JoltProof` into `PCS::Config`
  - Already `type Config = DoryLayout` for Dory
  - Makes proof serialization PCS-agnostic
- [ ] Remove DoryGlobals from prover.rs streaming loop
  - `DoryGlobals::initialize_context` → PCS config
  - `DoryLayout::AddressMajor` branching → PCS-level decision

### Phase 2 — Akita streaming commitment

- [ ] Implement `StreamingCommitmentScheme` for `AkitaCommitmentScheme`
  - `ChunkState`: `(partial_u, s, t_hat, ring_coeffs)` — partial outer Ajtai contribution + hint material
  - `process_chunk`: field → ring packing + inner Ajtai per block
  - `process_chunk_onehot`: sparse ring construction + `inner_ajtai_onehot`
  - `aggregate_chunks`: sum partial_u vectors + assemble `AkitaCommitmentHint`
- [ ] Small-scalar path for `process_chunk` (generic over SmallScalar or upcast)
- [ ] Implement `JoltToAkitaTranscript` adapter (mirror `JoltToDoryTranscript`)
- [ ] Create `jolt-core/src/poly/commitment/akita/` module
  - `mod.rs`, `commitment_scheme.rs` implementing Jolt's `CommitmentScheme` trait
  - Delegates to `akita-pcs` crate

### Phase 3 — Increment → one-hot

- [ ] Design `UnmapIncPolynomial` family (scaled `IdentityPolynomial` with offset)
- [ ] Modify witness generation to produce one-hot Inc polynomials
  - `RamInc` → 9 `RamIncRa(d)` one-hot polynomials
  - `RdInc` → 9 `RdIncRa(d)` one-hot polynomials
  - Offset encoding: `inc + 2^64` → unsigned → 8-bit chunks
- [ ] Modify Stage 6 `IncClaimReduction` to produce pushforward G_inc_l(k)
- [ ] Extend Stage 7 `HammingWeight` fusion to include Inc RA polynomials (+18 G polys)
- [ ] Remove `lagrange_factor` workaround in Stage 8 (`prover.rs:1392-1395`, `verifier.rs:582-585`)

### Phase 4 — Batch opening redesign

- [ ] Design mega-polynomial coefficient layout
  - Group by size class (dense T, one-hot K·T)
  - Zero-pad shorter polynomials
  - Determine selector variable ordering
- [ ] Implement `AkitaCommitmentScheme::batch_prove` / `batch_verify`
  - Selector sumcheck (~6 rounds)
  - Phase 0 inner evaluation (α = log₂ D rounds)
  - Standard Akita opening
- [ ] Adapt `ProverOpeningAccumulator` / `VerifierOpeningAccumulator`
  - Selector challenge sampling
  - Combined claim computation: `Σ eq(ρ, i) · v_i`
- [ ] Remove Dory-specific batch infrastructure from Akita path
  - `BatchPolynomialSource`, `StreamingBatchSource` (keep for Dory)
  - `RLCPolynomial` streaming RLC (keep for Dory)

### Phase 5 — End-to-end + cleanup

- [ ] Wire up `JoltProof` generic over PCS
- [ ] Run `muldiv` e2e test with Akita PCS
- [ ] Advice → one-hot conversion (deferred from Phase 3)
- [ ] Full DoryGlobals removal (~79 remaining call sites across 8 files)

---

## Key design parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| Ring degree D | 1024 | α = log₂ D = 10 inner evaluation rounds |
| Field | Fp128 (q = 2^128 - 275) | Solinas prime, `JoltFp128` wrapper exists |
| k_chunk (one-hot) | 256 | 4 cycles per ring element (4 × 256 = 1024 = D) |
| d_inc (increment chunks) | 9 | ⌈65/8⌉ for 65-bit signed range |
| Selector variables | ~7 | log₂(~82 polynomials) |
| Extra Stage 8 rounds | ~17 | 7 selector + 10 inner evaluation |

## Key files

| File | Role |
|------|------|
| `jolt-core/src/poly/commitment/commitment_scheme.rs` | `CommitmentScheme` + `StreamingCommitmentScheme` traits |
| `jolt-core/src/poly/commitment/dory/commitment_scheme.rs` | Dory impl (reference for Akita impl) |
| `jolt-core/src/poly/one_hot_polynomial.rs` | OneHotPolynomial (needs layout abstraction) |
| `jolt-core/src/poly/opening_proof.rs` | Accumulator, batch source, lagrange factors |
| `jolt-core/src/zkvm/prover.rs` | Streaming commit orchestration, Stage 8 |
| `jolt-core/src/zkvm/verifier.rs` | Stage 8 batch verify |
| `jolt-core/src/zkvm/witness.rs` | CommittedPolynomial enum, streaming witness gen |
| `jolt-core/src/zkvm/claim_reductions/increments.rs` | IncClaimReduction (to be replaced) |
| `jolt-core/src/zkvm/claim_reductions/advice.rs` | Advice claim reduction (deferred) |
| `jolt-core/src/zkvm/ram/raf_evaluation.rs` | RAF evaluation sumcheck (template for fused Inc) |
| `jolt-core/src/field/fp128.rs` | JoltFp128 wrapper over akita's Prime128M8M4M1M0 |
| `../akita/src/protocol/commitment_scheme.rs` | Akita's AkitaCommitmentScheme |
| `../akita/src/protocol/commitment/commit.rs` | Akita commit_coeffs, commit_onehot |
| `../akita/docs/AKITA_FOR_JOLT.md` | Integration design doc |

## Non-goals (explicit)

- Full DoryGlobals removal: deferred, not blocking Akita integration
- Akita recursion: out of scope for initial integration
- DA layer / Data Proof migration: separate workstream (see `../akita/docs/DATA_PROOF_AKITA_MIGRATION.md`)
- Backward compatibility shims: full cutover, no dual-PCS runtime
