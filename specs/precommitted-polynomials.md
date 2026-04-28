# Spec: Precommitted Polynomials — Advice, Bytecode, and Program Image

| Field       | Value                          |
|-------------|--------------------------------|
| Author(s)   | @0xAndoroid                    |
| Created     | 2026-04-08                     |
| Status      | proposed                       |
| PR          |                                |

## Summary

Jolt's verifier currently carries full program bytecode and initial memory tables in the clear. In recursive proof composition, this forces the recursive verifier to deserialize and evaluate claims over the full program data — cost that is linear in bytecode size and makes recursion prohibitively expensive for large programs. This PR replaces those cleartext tables with cryptographic commitments for both bytecode and the program image, batches prover-supplied auxiliary data (advice) into the same single-opening pipeline, and introduces a layout framework that confines the enlarged proof geometry to the stages where it is actually needed rather than padding the entire trace. On a SHA-256 benchmark, committed mode reduces the recursive verifier's preprocessing data by 97% and cuts deserialization cost by 94%. The layout framework reduces total proving time by up to 34% over a naive padding baseline.

## Intent

### Goal

Integrate three families of precommitted polynomials — trusted advice, untrusted advice, committed bytecode, and committed program image — into Jolt's single-opening pipeline (Stages 6b/7/8) so that: (1) the verifier no longer needs access to full program tables and instead checks openings against fixed-size commitments, and (2) precommitted polynomials of arbitrary size can coexist with the native trace layout without inflating earlier trace-domain stages.

### Invariants

1. **Prover/verifier consistency.** Every precommitted claim reduction produces an opening claim that the verifier can reconstruct identically from the proof transcript and public inputs. In standard (non-ZK) mode, `input_claim()` values must match exactly between prover and verifier; in ZK mode, `input_claim_constraint()` must encode the same formula.

2. **Sumcheck claim/constraint synchronization.** Each new sumcheck instance (`BytecodeClaimReduction`, `ProgramImageClaimReduction`, `AdviceClaimReduction`) implements `SumcheckInstanceParams` such that `input_claim()` and `input_claim_constraint()` stay in sync. Any change to one requires a matching update to the other.

3. **Top-left embedding correctness.** A precommitted polynomial committed in its own Dory matrix shape, when embedded in the larger joint matrix, must evaluate at the joint opening point to exactly `P(r_small) * selector_factor`, where `r_small` is the projection of the joint point onto the polynomial's own coordinates and the selector factor is the product of `(1 - r_i)` over the padded high coordinates.

4. **Layout non-propagation.** When a dominant precommitted polynomial forces an enlarged Stage 8 opening layout, the native trace geometry (Stages 1–6a) must remain unchanged. Only Stages 6b, 7, and 8 absorb the extra coordinates. Padded cycle rounds in Stages 1–6a must not increase.

5. **Two-phase round alignment.** Stage 6b binds exactly the cycle-derived coordinates of each embedded precommitted polynomial; Stage 7 binds exactly the remaining address-derived coordinates. Together they cover all row and column variables of the polynomial exactly once.

6. **Variable permutation correctness.** When the Dory opening point ordering differs from the polynomial's natural variable order, the polynomial's coefficient table is permuted by a bit-position permutation before sumcheck. After sumcheck, `normalize_opening_point()` maps challenges back to the original polynomial's opening point.

7. **`ProgramMode` toggle.** `ProgramMode::Full` (default) preserves the existing verifier behavior exactly — no regressions. `ProgramMode::Committed` activates committed bytecode/program-image paths. Both modes must produce valid proofs that verify correctly.

8. **Advice block alignment.** The larger (or equal-sized) advice block is placed first at the lowest RAM address, and the smaller block immediately after. This ensures each advice start index is aligned to its own block size, yielding a clean prefix-suffix factorization of the equality polynomial (no shifted-equality term needed for advice).

9. **Shifted equality correctness (program image only).** The program-image start address is generally not aligned to its padded size. The carry-based dynamic program (`shifted_eq_coeffs` / `eval_shifted_eq_poly_at_opening_point`) must compute the exact multilinear extension of the shifted-equality slice in time logarithmic in the padded program-image length.

10. **Bytecode chunking soundness.** When committed bytecode is split into chunks, each chunk's claim reduction is an independent sumcheck instance. The verifier linearly combines chunk commitments and claimed evaluations to recover the full bytecode contribution. Chunk count must be a power of two.

### Non-Goals

- **Changing the underlying PCS.** This work assumes Dory and its additive homomorphism. Extending to non-homomorphic PCS is out of scope.
- **Recursion-specific Dory verification optimizations.** The remaining Dory verification bottleneck inside recursive proofs is addressed by companion work (`joltrecursion`), not this PR.
- **Streaming commitment for precommitted polynomials.** When a dominant precommitted polynomial forces a larger embedding, the current implementation falls back to non-streaming commitment. Optimizing this path is future work.
- **Automatic chunk count selection.** The bytecode chunk count is a manual configuration parameter (`DEFAULT_COMMITTED_BYTECODE_CHUNK_COUNT`). Automatic tuning based on bytecode size is not implemented.
- **Precommitted layout for HyperKZG.** Only the Dory PCS backend is supported.

## Evaluation

### Acceptance Criteria

- [ ] `cargo nextest run -p jolt-core muldiv --cargo-quiet --features host` passes (standard mode, full program).
- [ ] `cargo nextest run -p jolt-core muldiv --cargo-quiet --features host,zk` passes (ZK mode, full program).
- [ ] `cargo nextest run -p jolt-core muldiv --cargo-quiet --features host` passes with `ProgramMode::Committed` configured (standard mode, committed program).
- [ ] `cargo nextest run -p jolt-core muldiv --cargo-quiet --features host,zk` passes with `ProgramMode::Committed` configured (ZK mode, committed program).
- [ ] E2e tests with advice (`trusted_advice`, `untrusted_advice`) pass in both standard and ZK modes.
- [ ] Bytecode chunking (chunk count > 1) produces valid proofs that verify correctly.
- [ ] `ProgramMode::Full` produces bit-identical proof behavior to the pre-PR baseline (no regression in existing test suite).
- [ ] Clippy passes in both `--features host` and `--features host,zk` with zero warnings.
- [ ] `cargo fmt` produces no changes.
- [ ] The shifted-equality verifier-side evaluation (`eval_shifted_eq_poly_at_opening_point`) matches brute-force MLE evaluation on randomized test vectors.

### Testing Strategy

**Existing tests that must pass:**
- All e2e tests in `jolt-core` (`muldiv`, `fibonacci`, `sha2`, `stdlib`, `advice`) in both `--features host` and `--features host,zk`.
- All unit tests in `jolt-core`.

**New tests needed:**
- Unit tests for `shifted_eq_coeffs` and `eval_shifted_eq_poly_at_opening_point` verifying correctness against naive MLE evaluation.
- Unit tests for `PrecommittedClaimReduction` round alignment (verifying that cycle + address rounds cover all polynomial variables exactly once for both `CycleMajor` and `AddressMajor` layouts).
- Unit tests for bytecode chunk construction (`build_committed_bytecode_chunk_polynomials`) validating lane encoding.
- E2e test exercising `ProgramMode::Committed` with chunked bytecode (chunk count = 16).

**Mode coverage:**
- Every new sumcheck instance must be tested in both `--features host` (standard) and `--features host,zk` (ZK) modes. The ZK path exercises BlindFold constraint synchronization.

### Performance

- **No regression in `ProgramMode::Full`.** This is the default mode and must not incur any overhead from the new code paths (all committed-mode logic is behind `ProgramMode::Committed` branches).
- **Committed mode overhead is concentrated in Stages 6b, 7, and 8.** Stages 1–6a should remain within ±5% of the `ProgramMode::Full` baseline.
- **Layout savings.** Against a naive trace-padding baseline, the precommitted layout should reduce total proving time by 14%–34% for `ProgramMode::Committed` with chunk count = 1, depending on layout and program.
- **Recursion improvement.** Embedded verifier preprocessing size should decrease by ~97% for programs with substantial bytecode (e.g., SHA-256 chains). Preprocessing deserialization cycles should decrease by ~94%.

## Design

### Architecture

**New types and enums:**

- `ProgramMode` (`config.rs`): `Full` | `Committed` — gates bytecode/program-image commitment paths.
- `PrecommittedPolynomial<F>` (`claim_reductions/precommitted.rs`): `Dense` | `BytecodeChunk` | `ProgramImage` — distinguishes precommitted object types for layout and claim reduction.
- `PrecommittedPhase` (`claim_reductions/precommitted.rs`): `CycleVariables` | `AddressVariables` — tracks which phase of the two-phase claim reduction is active.
- `PrecommittedSchedulingReference` (`claim_reductions/precommitted.rs`): Shared scheduling dimensions (main total vars, reference total vars, cycle alignment rounds, address rounds, joint col vars).
- `PrecommittedClaimReduction<F>` (`claim_reductions/precommitted.rs`): Core scheduling and round-alignment logic, variable permutation, active-round computation for embedded and dominant regimes.

**New claim reduction modules:**

- `claim_reductions/bytecode.rs`: `BytecodeClaimReductionParams` — two-phase bytecode claim reduction. Combines five staged value claims via random challenge η, reduces to opening claims for each bytecode chunk polynomial.
- `claim_reductions/program_image.rs`: `ProgramImageClaimReductionParams` — two-phase program-image claim reduction with shifted-equality evaluation. Uses carry-based DP for verifier-side `eq(r_addr, a_0 + y)` evaluation.
- `claim_reductions/advice.rs`: `AdviceClaimReductionParams` — two-phase advice claim reduction (one instance each for trusted and untrusted advice).

**Modified modules:**

- `zkvm/prover.rs`: Stage 6b adds precommitted claim reductions (advice, bytecode, program image) in committed mode. Stage 7 resumes address-phase binding.
- `zkvm/verifier.rs`: Mirrors prover changes. In committed mode, constructs verifier-side claim reduction params from proof transcript. Reconstructs full `init_eval` for standard mode consistency.
- `zkvm/bytecode/read_raf_checking.rs`: `ProgramMode`-aware dispatch. In committed mode, stages bytecode value claims for the bytecode claim reduction rather than relying on verifier-held tables.
- `zkvm/bytecode/chunks.rs`: Bytecode chunk construction — lane encoding, chunk sizing, polynomial building.
- `zkvm/bytecode/mod.rs`: Chunk count configuration, `TrustedBytecodeCommitments`, `TrustedBytecodeHints`.
- `zkvm/program.rs`: `TrustedProgramHints` and `program_image_commitment` for committed program image preprocessing.
- `zkvm/ram/mod.rs`: `prover_accumulate_program_image` / `verifier_accumulate_program_image` for program-image contributions.
- `zkvm/ram/val_check.rs`: Splits `init_eval` into public portion + advice/program-image contributions in committed mode.
- `poly/rlc_polynomial.rs`: `vmp_precommitted_contribution` — adds precommitted polynomials to the joint RLC polynomial as top-left embedded blocks.
- `poly/opening_proof.rs`: Joint opening builds with `precommitted_polys: HashMap<CommittedPolynomial, PrecommittedPolynomial<F>>`.
- `poly/commitment/dory/dory_globals.rs`: `MAIN_LOG_EMBEDDING`, `initialize_main_with_log_embedding` — global precommitted geometry initialization when committed objects exceed native trace dimensions.
- `zkvm/config.rs`: `ProgramMode` enum with serialization support.
- `zkvm/mod.rs`: `stage8_opening_ids` — adds `BytecodeChunk(i)` and `ProgramImageInit` opening IDs in committed mode.
- `zkvm/witness.rs`: `CommittedPolynomial::BytecodeChunk(usize)`, `CommittedPolynomial::ProgramImageInit` variants.

**Stage pipeline changes:**

| Stage | Change in Committed Mode |
|-------|--------------------------|
| 1–6a | Unchanged (native trace geometry preserved) |
| 6b | Extended with `lB` extra rounds for precommitted cycle-variable binding. Advice, bytecode, and program-image claim reductions run here (cycle phase). |
| 7 | Extended with precommitted address-variable binding. Advice, bytecode, and program-image claim reductions complete here (address phase). |
| 8 | Enlarged Dory opening point of length `lT + lK + lB`. Joint matrix includes precommitted polynomials via top-left embedding or dominant-regime placement. |

### Alternatives Considered

1. **Naive trace padding.** Pad the native trace layout to match the dominant precommitted polynomial size and run all stages on the padded trace. Rejected because it propagates the cost of one large committed object backward through the entire prover pipeline, inflating Stages 1–6a by up to 64x for large programs. Our precommitted layout avoids this propagation.

2. **Separate Dory opening proofs per precommitted polynomial.** Each precommitted polynomial gets its own independent Dory opening proof. Rejected because proving and verification costs would grow linearly with the number of precommitted objects, defeating the purpose of Jolt's single-opening architecture.

3. **AIR-style preprocessed columns.** Add dedicated preprocessed columns for each committed object (like SP1/RISC Zero). Not applicable to Jolt's sumcheck-based pipeline, which has no notion of fixed witness columns — every polynomial must be explicitly reduced to the common opening point.

## Documentation

The Jolt book (`book/`) should be updated with:
- A new section explaining committed bytecode mode and how to enable it.
- An updated batched-openings page reflecting the precommitted layout (embedded and dominant regimes).
- Documentation of the `ProgramMode::Committed` API for SDK users.

These documentation updates are deferred until the implementation is merged.

## Execution

The implementation follows the paper's structure:

1. **Foundation**: `ProgramMode` enum, `PrecommittedPolynomial`, `PrecommittedSchedulingReference`, and `PrecommittedClaimReduction` (the general round-alignment and variable-permutation machinery).
2. **Advice claim reduction**: Two-phase reduction with clean prefix-suffix eq factorization (no shifted-equality needed due to block alignment).
3. **Bytecode claim reduction**: Lane-weighted reduction combining five staged value claims via random challenge η. Bytecode chunk construction and committed bytecode preprocessing.
4. **Program-image claim reduction**: Shifted-equality evaluation via carry-based DP. `shifted_eq_coeffs` materializes the Boolean table; `eval_shifted_eq_poly_at_opening_point` evaluates the MLE in O(log P) time.
5. **Dory geometry**: `initialize_main_with_log_embedding` sets the global embedding when a precommitted polynomial dominates. `vmp_precommitted_contribution` places precommitted polynomials in the joint RLC matrix.
6. **Prover/verifier wiring**: Stage 6b/7/8 modifications, `ProgramMode`-aware dispatch in `read_raf_checking`, RAM val-check decomposition.
7. **BlindFold synchronization**: `input_claim_constraint()` and `output_claim_constraint()` implementations for all new sumcheck instances.

## References

- [Jolt paper](https://eprint.iacr.org/2023/1217): Base Jolt zkVM architecture.
- [Dory](https://eprint.iacr.org/2020/1274): Polynomial commitment scheme.
- [Precommitted Geometry paper](precommitted-geometry-and-dory-embedding.tex): Full technical treatment of the framework developed in this PR.
- [JoltBook: Batched Openings](https://jolt.a16zcrypto.com/how/optimizations/batched-openings.html): Existing batched-opening documentation.
- [PR #1344](https://github.com/a16z/jolt/pull/1344): Implementation PR targeting main Jolt repository.
