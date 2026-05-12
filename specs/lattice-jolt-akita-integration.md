# Spec: Lattice Jolt Akita Integration

| Field       | Value                      |
|-------------|----------------------------|
| Author(s)   | Quang Dao                  |
| Created     | 2026-05-11                 |
| Status      | draft                      |
| PR          |                            |

## Summary

Integrate Akita, formerly Hachi, as the lattice-based polynomial commitment backend for Jolt on top of the bytecode commitment branch.
The goal is to keep Jolt's zkVM relation and sumcheck stages intact while replacing the Dory-specific commitment, batching, and opening assumptions with an Akita-backed path that is transparent-setup and post-quantum.

This spec is intentionally a planning document.
The previous `jolt-hachi` branch is useful for identifying the shape of the work, but Akita has since been renamed, split into role-specific crates, and given newer proving and verification APIs.
The implementation must be redone against the latest Akita main branch rather than porting the old Hachi code mechanically.

## Intent

### Goal

Add an Akita PCS backend to Jolt that can commit to Jolt witness, bytecode, program-image, and advice polynomials, prove the Stage 8 opening obligations produced by the existing protocol, and verify those obligations through Akita's verifier-facing API.

The integration should introduce a clean PCS boundary for:

- Dory's current homomorphic random-linear-combination opening path.
- Akita's fused batched opening path over one or more commitment groups and opening points.
- Precommitted objects introduced by the bytecode commitment branch.
- Proof serialization and SDK surfaces that must be parameterized by the active PCS.

### Invariants

- The Akita path must prove the same Jolt execution relation as the Dory path.
- Stages 1 through 7 should remain protocol-identical unless a polynomial representation change requires a corresponding claim-reduction change.
- Stage 8 must bind every opening claim emitted by the earlier stages, including `RamInc`, `RdInc`, RA polynomials, advice polynomials, committed bytecode chunks, and committed program image openings.
- Prover and verifier must derive the same opening points, claim order, commitment grouping, transcript labels, and batching challenges.
- Akita integration must not rely on Dory-only APIs such as `combine_commitments`, `combine_hints`, `DoryGlobals`, Dory layouts, or homomorphic RLC construction.
- The bytecode commitment branch's committed program mode must remain sound: Akita proofs must bind committed bytecode chunks and program-image commitments to the same staged claims as Dory.
- If Akita uses a different scalar field than the current Jolt field, every conversion boundary must be explicit and transcript-bound.
- Any change to claim formulas must update the corresponding BlindFold or Akita ZK constraints when ZK mode is supported.
- The verifier path must not depend on Akita prover-only crates, polynomial backends, setup expansion, or planner search.

### Non-Goals

- Do not port the old `jolt-hachi` branch verbatim.
- Do not preserve old `hachi-*` names or compatibility aliases.
- Do not redesign Jolt's RISC-V semantics, bytecode semantics, RAM semantics, or instruction lookup semantics.
- Do not remove Dory from Jolt in this integration spec.
- Do not make Akita the default SDK backend until end-to-end correctness, serialization, and performance are measured.
- Do not require the initial implementation to solve recursive verifier performance.
  Native verifier correctness comes first, and guest-recursive verifier work can follow once the Akita verifier boundary is stable.

## Evaluation

### Acceptance Criteria

- [ ] Jolt has an Akita-backed PCS implementation or adapter that compiles against the latest Akita main branch.
- [ ] Akita setup, commitment, proof, verifier setup, and proof serialization are wired into Jolt's generic proof types without Dory-specific assumptions.
- [ ] Stage 8 can hand all Jolt opening obligations to Akita without using homomorphic commitment or hint combination.
- [ ] The Akita path supports committed bytecode chunks and committed program-image openings from the bytecode commitment branch.
- [ ] The Akita path handles all committed polynomial families required by a normal proof: dense trace polynomials, one-hot RA-style polynomials, increments, trusted advice, untrusted advice, bytecode chunks, and program image.
- [ ] Prover and verifier use one canonical claim grouping and opening-point ordering for Akita batched openings.
- [ ] Akita transcript integration is deterministic and domain-separated from Dory.
- [ ] Proof serialization roundtrips for Akita commitments, verifier setup, proofs, and Jolt proof envelopes.
- [ ] `muldiv` proves and verifies with Akita in standard mode.
- [ ] A committed-program test proves and verifies with Akita in standard mode.
- [ ] ZK mode support is explicitly decided: either Akita has a complete ZK opening path integrated with Jolt, or Akita is compile-time gated out of `host,zk` with a clear unsupported-mode error.
- [ ] `akita-verifier` remains prover-free and planner-free in the Jolt dependency graph.
- [ ] Dory tests continue passing after the abstraction changes.

### Testing Strategy

Keep Dory as the regression baseline:

```bash
cargo nextest run -p jolt-core muldiv --cargo-quiet --features host
cargo nextest run -p jolt-core muldiv --cargo-quiet --features host,zk
cargo nextest run -p jolt-core muldiv_e2e_dory_committed_program_commitments --cargo-quiet --features host
cargo nextest run -p jolt-core muldiv_e2e_dory_committed_program_commitments --cargo-quiet --features host,zk
```

Add Akita-focused checks as the implementation lands:

```bash
cargo nextest run -p jolt-core muldiv_e2e_akita --cargo-quiet --features host
cargo nextest run -p jolt-core muldiv_e2e_akita_committed_program_commitments --cargo-quiet --features host
```

Targeted tests should cover:

- Jolt-to-Akita field and opening-point conversion.
- Transcript byte determinism for a minimal Akita opening flow.
- Dense polynomial commitments.
- One-hot polynomial commitments.
- Increment polynomial representation.
- Advice polynomial commitments and openings.
- Bytecode chunk commitment construction.
- Program-image commitment construction.
- Akita proof serialization within `JoltProof`.
- Verifier rejection for reordered claims, wrong opening points, wrong commitments, and wrong committed bytecode metadata.

Strict linting remains required:

```bash
cargo clippy --all --features host -q --all-targets -- -D warnings
cargo clippy --all --features host,zk -q --all-targets -- -D warnings
cargo fmt -q
```

### Performance

Akita is expected to change both prover and verifier cost models.
The first implementation should optimize for correctness and clean abstraction boundaries, then benchmark before tuning.

Measure at least:

- prover time for Dory full mode versus Dory committed mode versus Akita,
- verifier time for the same cases,
- proof size,
- setup size,
- serialized verifier preprocessing size,
- committed bytecode chunk count sensitivity,
- memory use during streaming witness commitment.

The Akita path should avoid materializing unnecessary verifier-side bytecode or program-image data in committed mode.
It should also avoid routing all polynomials through dense buffers when an Akita one-hot or packed representation can preserve Jolt's sparse structure.

## Design

### Architecture

The bytecode commitment branch already separates program data into full and committed modes.
Akita integration should sit behind the same high-level Jolt prover and verifier pipeline:

```text
Jolt trace and preprocessing
  -> witness polynomials and precommitted program objects
  -> sumcheck stages 1 through 7
  -> opening claims
  -> PCS-specific Stage 8 opening proof
```

Dory's Stage 8 path currently builds one homomorphic random linear combination and verifies one opening against a combined commitment.
Akita should instead consume the opening claims through its native grouped batched opening interface.
The important design task is to translate Jolt's opening accumulator into Akita commitment groups and opening-point groups without changing the earlier sumcheck semantics.

### Akita Crate Boundary

Latest Akita is no longer the old monolithic `hachi-pcs` crate.
The integration should depend on the smallest Akita crates needed for each role:

- `akita-types` for proof, setup, schedule, opening, and claim data shapes.
- `akita-verifier` for verifier replay.
- `akita-prover` for commitment, proving, polynomial backends, and prover hints.
- `akita-scheme` or `akita-pcs` only if the narrower role crates do not expose a stable enough integration surface.
- `akita-field`, `akita-algebra`, `akita-transcript`, and `akita-serialization` where Jolt needs direct adapters.

Verifier-only Jolt code must not depend on `akita-prover`, `akita-setup`, `akita-planner`, examples, benches, or profile harnesses.
If the first implementation must temporarily depend on an aggregate crate, it should be called out as a short-lived implementation debt and removed before the PR is considered complete.

### PCS Trait Shape

Jolt's current `CommitmentScheme` trait assumes single-polynomial commit/open methods and includes Dory-friendly homomorphic helpers.
Akita's current surface separates prover and verifier roles and supports fused batched openings over grouped commitments.

The implementation should avoid forcing Akita through the Dory trait shape.
Possible directions:

- Extend Jolt with a role-aware opening backend trait that can express both Dory RLC openings and Akita grouped batched openings.
- Split Dory-only homomorphic helpers into an explicit `AdditivelyHomomorphicPCS` extension trait.
- Keep `StreamingCommitmentScheme` only for PCS backends that truly stream tier-1 chunks, and give Akita either a direct adapter or a native streaming path if latest Akita supports it.
- Make proof serialization depend on PCS associated types rather than Dory layout fields.

The result should make the Dory path explicit as "homomorphic RLC" and the Akita path explicit as "grouped batched opening", rather than hiding both behind a misleading common method.

### Polynomial Representation

Akita should receive polynomial data in representations it can prove efficiently.
The implementation must map each Jolt committed polynomial family deliberately:

- Dense or compact trace polynomials can use an Akita dense backend or a small-scalar adapter.
- One-hot RA polynomials should use an Akita one-hot backend when supported.
- `RamInc` and `RdInc` need a representation decision.
  The old Hachi branch proposed offset-encoding signed increments and decomposing them into one-hot chunks.
  That idea should be reevaluated against latest Akita rather than assumed.
- Advice polynomials are precommitted and may have different dimensions from the main trace domain.
  They should use the same precommitted scheduling model as committed bytecode and program image where possible.
- Committed bytecode chunks should preserve the bytecode branch's lane layout and chunk metadata, but the coefficient ordering may need an Akita-specific adapter.
- The program-image polynomial should remain the padded initial RAM bytecode-word slice committed during preprocessing.

Any representation change that affects a claim value must be accompanied by the corresponding claim-reduction change.
For example, if increments become one-hot chunk polynomials, the increment claim reduction must prove equivalence to the old signed increment values.

### Opening Claim Grouping

The Jolt opening accumulator should expose a PCS-neutral list of opening obligations:

```text
opening obligation = {
  polynomial id,
  commitment id,
  opening point,
  claimed value,
  prover witness source,
}
```

Dory can continue to reduce these obligations into one RLC polynomial when using the Dory backend.
Akita should group them by the latest Akita batched proving API:

- one or more opening points,
- one or more committed polynomial groups per opening point,
- one commitment and hint per group,
- one fused batched proof.

This grouping must include the bytecode commitment branch's precommitted openings.
The design should avoid assuming that all polynomials share the same variable count or layout.

### Transcript And Fiat-Shamir Domains

Akita uses its own transcript convenience layer backed by Jolt transcript machinery in recent Akita code.
The Jolt integration needs a single adapter that makes the absorbed bytes and challenge derivation explicit.

Requirements:

- Dory and Akita transcript domains must be distinct.
- Akita proof and commitment objects must be absorbed in the same order by prover and verifier.
- Jolt's existing stage challenges must remain unchanged before the PCS-specific Stage 8 boundary unless the spec for Akita explicitly says otherwise.
- Akita-internal challenges must be derived by Akita's transcript code, not by hand-rolled Jolt wrappers.

### Field Boundary

Akita currently has its own field traits and concrete fields, including 128-bit prime fields used by the PCS.
Jolt's core proof system is generic over `JoltField`, with existing production use on BN254 scalar field.

The implementation must choose one of two explicit paths:

- Run the full Jolt proof over an Akita-compatible field, likely by adding or updating a Jolt field wrapper for Akita's field.
- Keep the Jolt relation field unchanged and add a sound, transcript-bound conversion layer only where Akita commits to and opens coefficients.

The first path is simpler to reason about for the PCS opening relation.
The second path needs a clear proof obligation for field conversion and should not be added casually.
The old `JoltFp128` work is a starting point, not a final design.

### ZK Mode

Current Jolt ZK mode relies on BlindFold constraints and PCS-specific hidden evaluation support.
Akita's ZK story has changed since the Hachi branch and may involve separate commitment-hiding or verifier-hiding machinery.

The integration should start by making the ZK decision explicit:

- If latest Akita has a production ZK opening path, wire it into Jolt's ZK proof surface and update BlindFold or replacement constraints accordingly.
- If not, compile-gate Akita to standard mode and make `host,zk` fail clearly for Akita rather than silently using an unsound fallback.

Do not fake Dory-style evaluation commitments for Akita.
Any hidden-claim path must be backed by the actual Akita protocol.

### Proof Serialization And SDK

Akita commitments, verifier setup, prover setup references, opening hints, and proofs must fit into Jolt's proof and preprocessing serialization.
This likely touches:

- `JoltProof` PCS associated types,
- proof deserialization and variant tags,
- committed program preprocessing serialization,
- SDK-generated preprocessing helpers,
- examples that choose a PCS backend,
- verifier-transpilation inputs if Akita guest verification is attempted.

The initial SDK surface can be explicit and experimental.
It should not hide bytecode chunk count, Akita config, or unsupported ZK behavior.

### Relationship To Existing Bytecode Commitment Work

The bytecode commitment branch makes committed bytecode and program image part of Jolt's opening proof.
Akita integration should build on that rather than introducing a separate program commitment path.

Specifically:

- `ProgramMode::Committed` should work with Akita.
- Bytecode chunk commitments should become Akita commitments in Akita mode.
- Program-image commitments should become Akita commitments in Akita mode.
- Stage 6a, Stage 6b, and Stage 7 claim reductions should continue producing the same logical claims.
- Stage 8 should route those claims through Akita grouped openings instead of Dory RLC openings.

### Alternatives Considered

One option is to preserve the old Hachi branch's mega-polynomial design exactly.
That branch bundled all main witness polynomials into a selector-indexed mega-polynomial because Ajtai commitments do not support Dory-style homomorphic batching.
Latest Akita now exposes grouped batched opening APIs, so the implementation should first evaluate whether native batching subsumes the mega-polynomial layer.

Another option is to keep Jolt's Dory RLC abstraction and emulate it for Akita.
This is rejected because it hides the real protocol boundary and risks incorrect assumptions about commitment linearity and hint combination.

A third option is to defer committed bytecode and program-image support until after Akita proves ordinary trace polynomials.
This is rejected for this branch because the integration target is explicitly on top of the bytecode commitment branch.
The initial implementation can stage the work, but the spec target must include committed program mode.

## Documentation

Update the Jolt book after the implementation stabilizes:

- explain Akita as the lattice PCS backend and Dory as the existing pairing-based backend,
- describe which PCS backends support full mode, committed program mode, and ZK mode,
- document Akita configuration choices that affect proof size and verifier cost,
- document committed bytecode chunking when Akita is selected,
- add a short note that Akita descends from Hachi and that old Hachi branch details are historical.

Developer documentation should also record:

- how to update the Akita dependency,
- which Akita crates Jolt is allowed to depend on in prover and verifier paths,
- the verification command matrix,
- any known unsupported combinations.

## Execution

Suggested implementation order:

1. Pin or vendor the latest Akita dependency and record the exact commit used for the first integration attempt.
2. Add the minimal Jolt field and transcript adapters needed to compile an Akita proof in isolation.
3. Introduce a PCS-neutral opening-obligation representation in Jolt.
4. Refactor the Dory Stage 8 path to consume that representation without behavior changes.
5. Add an Akita Stage 8 path that groups obligations for Akita's batched proving and verification API.
6. Add Akita commitment support for dense and one-hot main witness polynomials.
7. Decide and implement the increment representation.
8. Add trusted and untrusted advice commitment support.
9. Add committed bytecode chunk and program-image commitment support.
10. Wire proof serialization and preprocessing serialization.
11. Add standard-mode e2e tests for `muldiv` and committed program mode.
12. Decide and implement or explicitly gate ZK mode.
13. Add SDK and example entrypoints.
14. Run Dory regression tests, Akita e2e tests, clippy in both feature modes, and proof-size/performance smoke benchmarks.

The implementation should make each boundary compile and test before moving to the next family of polynomials.
If latest Akita's API differs materially from the assumptions here, update this spec first rather than forcing the implementation through stale names.

## References

- Bytecode commitment spec: `specs/1344-committed-bytecode-program-image.md`
- Jolt PCS trait: `jolt-core/src/poly/commitment/commitment_scheme.rs`
- Jolt opening accumulator and Stage 8 support: `jolt-core/src/poly/opening_proof.rs`
- Jolt proof serialization: `jolt-core/src/zkvm/proof_serialization.rs`
- Jolt prover Stage 8 path: `jolt-core/src/zkvm/prover.rs`
- Jolt verifier Stage 8 path: `jolt-core/src/zkvm/verifier.rs`
- Akita crate graph: `/Users/quang.dao/Documents/SNARKs/akita/docs/crate-graph.md`
- Akita Jolt trait integration spec: `/Users/quang.dao/Documents/SNARKs/akita/specs/akita-crate-followup-jolt-integration.md`
- Historical Hachi integration notes: `/Users/quang.dao/Documents/SNARKs/jolt-hachi/HACHI_INTEGRATION.md`
