# Spec: Akita Committed Bytecode Policy

| Field       | Value                                              |
|-------------|----------------------------------------------------|
| Author(s)   | Quang Dao                                          |
| Created     | 2026-06-15                                         |
| Status      | approved                                           |
| PR          | LayerZero-Research/jolt#22 (`lz/integrate-akita`) |

## Summary

Akita PCS integration must eventually support **committed bytecode and program-image openings**, the same verifier-efficiency mode defined in `specs/1344-committed-bytecode-program-image.md`.
The initial Akita milestone targets `ProgramMode::Full` only, but the codebase and specs must **preserve** committed-mode semantics, types, and extension points rather than deleting them to simplify a merge.

Temporary removal of Dory committed-mode code during an integration merge is acceptable only when tracked as **debt to restore**, not as a permanent product decision.

## Intent

### Goal

Jolt with Akita PCS will support both:

- **`ProgramMode::Full`**: verifier holds full bytecode / program-image preprocessing (first milestone).
- **`ProgramMode::Committed`**: verifier holds commitments to bytecode chunks and program image; prover proves openings through the same claim-reduction + Stage 8 pipeline (required future milestone).

Dory is the reference implementation for committed-mode protocol semantics today.
Akita adoption reuses those semantics and replaces only the PCS layer (mega-polynomial batch opening instead of homomorphic Dory RLC).

### Invariants

- **Protocol parity:** Full and Committed modes must prove the same guest execution relation for a given PCS family, matching spec 1344.
- **No silent amputation:** Do not delete `ProgramMode::Committed`, `BytecodeChunk`, `ProgramImageInit`, precommitted scheduling types, or modular-verifier formula coverage unless replacing them with an equivalent on the same branch in the same PR.
- **Preserve main alignment:** When merging `origin/main`, prefer **compile-time retention** (`feature = "dory-committed"` or equivalent) over excising committed-subsystem files.
  Port layout-sensitive code to `MatrixLayout` rather than dropping it.
- **Enum and wire-format stability:** `CommittedPolynomial` tags, `SumcheckId`s, and proof serialization for committed objects stay compatible with `jolt-core` and `jolt-verifier` compat layers unless a versioned wire break is explicitly spec'd.
- **Akita-specific PCS constraint:** Ajtai commitments are not homomorphic; Akita Committed mode will use the mega-polynomial / selector-batch opening design (see `AKITA_INTEGRATION.md` Architecture), not Dory's `combine_commitments` path.
  This is a PCS implementation difference, not a reason to remove committed-mode protocol types from the tree.

### Non-Goals (this policy)

- Implementing Akita Committed mode in the first integration PR.
- Supporting committed bytecode on Dory and Akita in the same binary invocation (one PCS per proof).
- Backward-compatibility shims for pre-1344 proof formats.

## Evaluation

### Acceptance Criteria (policy adoption)

- [x] This spec exists and is linked from `AKITA_INTEGRATION.md`.
- [x] `AKITA_INTEGRATION.md` no longer describes committed-subsystem excision as a permanent "locked" product decision.
- [x] `specs/1344-committed-bytecode-program-image.md` states that Akita PCS is in scope for Committed mode eventually.
- [ ] Follow-up engineering tracks restoration of precommitted / committed-program modules on the Akita branch (or on `main` first, then merge).

### Engineering principles (effective immediately)

1. **Do not excise to save merge time** if the same files will be re-added for Akita Committed mode.
   Prefer `cfg`/feature gating and `MatrixLayout` ports.
2. **Do not delete zombie types without a plan:** if `ProgramMode::Committed` or `BytecodeChunk` remain in the enum, either wire them (Dory) or gate them explicitly; do not leave `unreachable!` stubs indefinitely.
3. **Specs lead code:** when integration tradeoffs arise, update this spec and spec 1344 before deleting protocol surface area.
4. **Modular verifier is the long-term home** for precommitted scheduling (`jolt-claims`, `jolt-verifier`), per upstream #1610/#1616.
   Akita integration must converge with that direction, not fork a second advice scheduling model without justification.

## Design

### Phased delivery

| Phase | Scope | Committed bytecode |
|-------|--------|-------------------|
| **A (now)** | Akita PCS, Full mode, inc one-hot, Dory default | Dory Committed unchanged on `main`; on Akita branch restore/gate rather than delete |
| **B** | `MatrixLayout` on shared trunk; precommitted on `MatrixLayout` | Dory Committed e2e green on integration branch |
| **C** | Akita Committed | Mega-polynomial Stage 8; committed chunk + program-image claim reductions on Fp128 |

Phase C depends on Phase B protocol types and tests remaining on the tree after Phase A.

### Relationship to other specs

- **`specs/1344-committed-bytecode-program-image.md`**: canonical protocol definition (Dory-first).
- **`AKITA_INTEGRATION.md`**: integration tracker; must align with this policy.
- **`specs/jolt-verifier-model-crate.md`**: modular verifier owns precommitted schedule and Stage 8 opening order long term.

### Known branch debt (`lz/integrate-akita`)

The first main-sync merge **temporarily removed** `claim_reductions/{precommitted,program_image,bytecode}.rs` and reverted advice scheduling to the pre-main path.
That state is **merge debt**, not the target architecture.
Restoration work is tracked as follow-up on the integration branch or via landing layout + precommitted on `main` before the next Akita tranche.
