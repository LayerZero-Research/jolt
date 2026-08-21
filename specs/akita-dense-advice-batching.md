# Spec: Dense Advice and Preprocessing-Provisioned Akita Batching

| Field | Value |
|-------|-------|
| Author(s) | Omid Bodaghi, Jolt contributors |
| Created | 2026-08-21 |
| Status | implemented |
| Branch | `omid/batch-advice` |

## Summary

The Akita path commits trusted and untrusted advice directly as dense
polynomials of little-endian `u64` words. It no longer converts advice into a
byte one-hot table, proves that table's reconstruction, or opens advice through
a separate PCS proof.

Advice is still committed independently from the per-execution
`OneHotTrace`: trusted advice may be committed out of band, and untrusted
advice is committed before the trace during proving. Akita's heterogeneous
group batching then combines the independently committed dense advice groups
with the final one-hot trace group in one opening proof. The groups may use
different source representations, dimensions, setups, and evaluation points;
they remain separate commitments.

The grouped Akita schedule depends on both the final trace layout and the exact
profiles produced by the independent advice commitments. Advice capacity is
known during program preprocessing, but `log_T` is not. Preprocessing therefore
freezes the reachable advice profiles and provisions the grouped schedule rows
for every reachable final trace arity. Commit, prove, and verify only select and
resolve those rows; they never run the expensive schedule planner.

## Intent

### Goals

- Commit the word polynomial Jolt already consumes instead of a second,
  one-hot advice representation.
- Remove advice-specific byte reconstruction, booleanity, hamming-weight, and
  decode work from the Akita protocol.
- Preserve the independent commitment lifecycle of trusted advice, untrusted
  advice, and `OneHotTrace`.
- Discharge every present advice opening together with `OneHotTrace` through
  one heterogeneous Akita batch proof and verification.
- Generate advice-dependent grouped schedules once during preprocessing, after
  advice capacities are known and before setup sizing.
- Keep schedule planning out of commit, prove, verify, and proof
  deserialization.

### Non-goals

- Merging advice coefficients into the `OneHotTrace` commitment.
- Merging trusted and untrusted advice into one commitment object.
- Changing the guest advice API, RAM layout, maximum-size attributes, or
  runtime advice tape.
- Removing the word-level `RamValCheck` contribution or either phase of
  `AdviceClaimReduction`.
- Changing committed-bytecode or program-image reconstruction and opening.
- Changing the Dory path or adding `akita + zk` support.
- Pre-generating and checking in advice-dependent grouped schedule catalogs.

## Canonical Advice Object

For one advice kind with public maximum byte capacity `M`, a valid
`MemoryLayout` makes `M` either zero or an eight-byte-aligned power of two.
Define the canonical word vector by packing bytes eight at a time in
little-endian order and padding the unused capacity with zeroes:

```text
N = next_power_of_two(max(1, floor(M / 8)))

A[i] = sum_{j=0}^{7} byte[8*i + j] * 2^(8*j),  0 <= i < N
```

Bytes past the actual input length contribute zero. Inputs larger than `M` are
rejected. `common::advice::canonical_advice_words` is the shared definition
used by the witness and commitment paths.

The logical polynomial has one coefficient per word and
`q = log2(N)` variables. Akita's dense schedule family supports physical
arities `14..=34`, so the physical commitment uses

```text
p = max(q, 14)
```

variables. When `q < 14`, the logical polynomial occupies slot zero of the
larger prefix-packed object and the honest constructor fills every other slot
with zero. `PrefixPackedObjectPlan` performs the corresponding
logical-to-physical opening reduction. When `q >= 14`, there are no selector
coordinates and the logical and physical claims coincide.

Advice uses `JoltDenseBounded`, Akita's dense source configured for the signed
bit width needed to contain every `u64`. This is a commitment-source bound, not
a return to byte one-hot encoding: canonical honest advice coefficients are
still constructed as `u64`, and an out-of-range dense coefficient is rejected
instead of being truncated.

Trusted and untrusted advice use distinct semantic IDs and layout digests even
when their capacities and contents are identical.

## Protocol Architecture

### Before this change

```text
raw advice bytes
  +-- little-endian word polynomial --> RamValCheck --> AdviceClaimReduction
  |
  `-- byte one-hot polynomial -------> advice reconstruction --> PCS opening

OneHotTrace ----------------------------------------------------> PCS opening
```

This represented the same advice twice and required an advice-specific
sumcheck to connect the word claim to the byte one-hot commitment. Advice and
the trace also paid for separate PCS proofs and verifications.

### After this change

```text
raw advice bytes
  `-- canonical dense word polynomial
        |-- independent dense commitment (precommitted group)
        `-- RamValCheck --> AdviceClaimReduction --> final advice claim

trace rows --> final one-hot commitment (conditioned on precommit profiles)

[advice claim(s) at their points, OneHotTrace claim at its point]
  `-- one heterogeneous Akita batch proof / verification
```

The base PIOP already works over advice words, so stages 4, 6b, and 7 retain
their existing semantics. Stage 8 resolves the final advice point and value
from whichever phase completed `AdviceClaimReduction`, reduces it through the
prefix-packed advice plan, and supplies it directly to the grouped PCS
statement.

Advice no longer participates in the reconstruction sumcheck. That phase is
present only when committed bytecode or program-image reconstruction requires
it.

### Commitment lifecycle

| Object | Commitment time | Representation | Opening |
|--------|-----------------|----------------|---------|
| Untrusted advice | Per proof, before `OneHotTrace` | One dense word polynomial | Joint main batch when present |
| Trusted advice | Out of band / preprocessing | One dense word polynomial | Joint main batch when present |
| `OneHotTrace` | Per proof, after all present advice precommits | One streamed one-hot polynomial | Final group of the joint batch |
| Committed-program objects | Program preprocessing | Existing one-hot representation | Existing auxiliary proofs |

Akita's public batch order is:

```text
[UntrustedAdvice?, TrustedAdvice?, OneHotTrace]
```

The precommitted roles are explicit and transcript-bound. Strict role ordering
rejects duplicates and permutations. The outer Jolt commitment absorption
order remains the protocol-defined order used by `absorb_packed_commitments`;
it must not be inferred from Akita's batch order.

## Parameter and Schedule Lifecycle

### Why the final parameters depend on advice

An advice commitment is created without knowing `log_T`. Its standalone dense
schedule and setup therefore depend only on its public capacity and canonical
packing plan. The commit produces an opening hint containing the exact frozen
`CommittedGroupProfile` selected for that dense group.

The final `OneHotTrace` commitment is different. Its grouped schedule is keyed
by:

```text
AkitaScheduleLookupKey {
    final_group: layout derived from OneHotTrace and log_T,
    precommitteds: exact ordered profiles of present advice groups,
}
```

Arity or flavor alone is insufficient: the full frozen profile fixes the
commitment matrices and folding parameters that the final group and joint
opening must use.

### Rejected: enumerate every advice size globally

A checked-in global catalog would need rows across:

```text
untrusted capacity x trusted capacity x final trace arity
```

and would grow again when other independently committed objects, such as
bytecode layouts, join the batch. Even if each dimension has a modest range,
their product makes the catalog and its generated parameter data unnecessarily
large. It also bakes guessed application capacities into the repository.

The checked-in Akita catalogs therefore contain only the bounded scalar dense
rows and scalar `OneHotTrace` rows. They contain no grouped advice rows.

### Chosen: provision the program-specific rows during preprocessing

Program preprocessing knows `max_untrusted_advice_size`,
`max_trusted_advice_size`, the one-hot family (`K=16` or `K=256`), and the
maximum padded trace length. It performs the following work before constructing
the main PCS setup:

1. Derive the physical dense layout for each advice kind whose configured
   capacity is nonzero.
2. Resolve each layout's standalone dense catalog row and freeze its exact
   precommit profile.
3. Enumerate every nonempty advice-presence combination reachable by a proof:
   `[untrusted]`, `[trusted]`, and `[untrusted, trusted]` when both capacities
   are configured. Equal profile sequences are deduplicated.
4. For each combination, plan one grouped row for every reachable final
   `OneHotTrace` arity in the selected family's declared range, capped by the
   program's maximum padded trace arity.
5. Validate each row and publish it to the runtime registry.
6. Size the main setup over both the checked-in scalar rows and every
   provisioned grouped row.

An arity for which Akita reports `UnsupportedSchedule` is skipped; a proof that
selects that arity fails shape validation. Every other planner error aborts
preprocessing. Provisioning is bounded by `MAX_REGISTERED_ROWS` and uses a
bounded planner worker count because planner solves are memory-intensive.

This design turns the advice-size dimensions into one program-specific choice.
Only the trace-arity dimension is swept. A proof later selects exactly one of
those precomputed rows once its actual `log_T` is known.

### Registry and resolution

`CommitmentConfig` schedule hooks are associated functions and cannot borrow a
preprocessing object. Provisioning therefore publishes rows into a process-wide
registry keyed by the commitment config and exact row digest/profile. The
registry is a union: programs with different advice capacities may coexist in
one process without reinterpreting each other's rows.

Resolution order is:

```text
provisioned registry row -> checked-in scalar catalog -> error
```

A miss never invokes the planner. Consequently:

- advice and final commits resolve by exact lookup key/profile;
- the prover opening replays the selected row by identity;
- the verifier resolves the proof's public row selection by digest; and
- no expensive generation occurs during commit, prove, verify, or decode.

Publishing the same row again is a no-op. Reusing a digest for different
profiles is rejected as a collision. The standalone dense profile used to key
grouped rows is read from the same checked-in catalog that the independent
advice commit uses; a drift test asserts that this profile matches a fresh
planner solve.

### Setup ordering invariant

Provisioning must happen before main setup construction. Akita setup matrix
requirements are not monotone in arity or polynomial count, so a setup sized
only from the largest scalar row does not necessarily cover a smaller grouped
row. `catalog_setup_capacity` explicitly folds every provisioned row into the
capacity calculation, including the commit-only footprint of each precommit.

The main setup reserves one batch polynomial for `OneHotTrace` plus one for
each advice kind the program may carry:

```text
max_total_batch_polys = 1
    + (max_untrusted_advice_size > 0)
    + (max_trusted_advice_size > 0)
```

This is total batch capacity; every individual group still contains exactly
one polynomial.

## Prover and Verifier Flow

### Preprocessing

- Derive the trusted and untrusted advice setups from their public capacities.
- Provision the grouped rows for the selected one-hot family and reachable
  final arities.
- Include those rows when sizing the final `OneHotTrace` setup.
- Rebuild the same deterministic rows in a verifier process from the public
  capacities; rows are not proof-controlled.

### Stage 0

- Construct and independently commit untrusted advice when present.
- Accept the independently committed trusted advice object when present and
  validate its shape, setup, commitment/hint agreement, and equality with the
  public trusted bytes available to the prover.
- Put present precommits in canonical role order.
- Resolve the exact grouped row before materializing the expensive trace
  source.
- Commit `OneHotTrace` as the final group under the ordered frozen precommit
  profiles.
- Absorb all commitments through the shared prover/verifier transcript helper.

### Stage 8

- Resolve each final advice claim from stage 6b or stage 7.
- Reduce each logical advice claim through its prefix-packed physical layout.
- Reduce the `OneHotTrace` leaves to its one physical claim.
- If advice is present, call the precommitted-trace batch prover once over all
  advice groups plus the trace. If no advice is present, retain the ordinary
  single-group `open_batch_from_hint` path.
- Open committed-program objects through their existing auxiliary path.

### Verification

- Reconstruct advice presence and layouts from public preprocessing and proof
  structure.
- Validate every commitment and setup's backend flavor, arity, polynomial
  count, layout digest, role, and presence.
- Reconstruct the same group-local points and evaluations.
- Resolve the selected grouped row by digest and validate its exact profiles.
- Run one heterogeneous batch verification for present advice plus
  `OneHotTrace`; use ordinary single-group verification when advice is absent.
- Reject missing, extra, reordered, or shape-inconsistent groups.

## Invariants and Failure Behavior

- Advice bytes, the witness oracle, and the committed polynomial use the same
  canonical word table.
- Actual input length affects presence but never commitment arity; configured
  capacity fixes the shape.
- A present all-zero advice input still has a commitment and batch member.
- The trusted and untrusted roles are not interchangeable.
- The final trace commit is conditioned on the exact profiles of every present
  advice precommit.
- Prover and verifier use identical presence combinations, group order,
  layouts, points, evaluations, row selection, and transcript messages.
- Advice final claims come directly from `AdviceClaimReduction`; there is no
  advice reconstruction relation or advice byte opening.
- The bounded dense source must cover every canonical `u64` coefficient. A
  future dense object with a wider coefficient domain requires a different
  commitment config rather than silently reusing `JoltDenseBounded`.
- No grouped row is accepted unless it was deterministically provisioned from
  public preprocessing inputs and is covered by the setup matrices.
- Planner DP is preprocessing-only. A runtime registry or catalog miss is an
  error, never an invitation to generate parameters on demand.

The verifier fails closed on, at minimum:

- advice exceeding its configured capacity;
- advice commitment presence disagreeing with the scheduled input;
- incorrect dense/one-hot flavor, arity, count, layout digest, or one-hot `K`;
- commitment and opening hint disagreement;
- duplicate or noncanonical precommit roles;
- an unknown row digest or a row whose profiles do not match the statement;
- a grouped schedule outside setup capacity; or
- a missing, extra, or invalid joint opening proof.

## Evaluation

### Acceptance Criteria

- [x] Trusted and untrusted advice are committed as canonical dense word
      polynomials, not byte one-hot tables.
- [x] Advice-specific reconstruction relations and proof fields are removed;
      bytecode/program reconstruction remains intact.
- [x] Both advice kinds, in every reachable presence combination, join
      `OneHotTrace` in one heterogeneous Akita opening proof.
- [x] Advice commitments remain independent and trusted advice remains reusable.
- [x] Grouped advice rows are absent from checked-in catalogs and generated
      during preprocessing from actual configured capacities.
- [x] Provisioned rows cover the reachable final-arity range and are included
      in main setup sizing before any commitment.
- [x] Commit, prove, verify, and deserialization perform lookup only and never
      invoke planner DP.
- [x] Different program capacities coexist without row aliasing; identical
      provisioning is idempotent.
- [x] Modular and legacy Akita paths agree on the commitment and proof protocol.
- [x] Advice-free proofs retain the scalar `OneHotTrace` path.

### Testing Strategy

The permanent tests should cover independent protocol properties rather than
reimplementing the retired byte-one-hot path as an oracle:

- `common::advice` tests canonical little-endian packing, zero padding, and
  oversize rejection.
- `jolt-akita` schedule tests assert that grouped advice rows are not cataloged,
  that one- and two-precommit rows provision and resolve, and that setup
  capacity covers both the grouped schedule and precommit footprint.
- Registry tests cover new capacities, all resolution hooks, idempotent
  publication, digest collision rejection, and coexistence of different
  program capacities.
- Akita end-to-end tests cover no advice, trusted only, untrusted only, both
  kinds, and tampered commitments/openings.
- The modular prover acceptance suite and legacy packed tests cover byte parity
  and the full pipeline.
- Standard Dory and ZK suites continue passing as non-Akita regressions.

Relevant validation commands are:

```bash
cargo nextest run -p jolt-akita --cargo-quiet
cargo nextest run -p jolt-prover --features prover-fixtures,akita --cargo-quiet
cargo nextest run -p jolt-prover-legacy --features host,akita --cargo-quiet
cargo clippy --all --features host -q --all-targets -- -D warnings
cargo fmt -q
```

### Performance

The proof path should improve by removing advice byte-one-hot materialization,
advice reconstruction sumchecks, and separate advice opening proofs. Parameter
generation cost moves to preprocessing and is bounded by the program's actual
advice profiles and reachable trace-arity range. Profiling must report this
preprocessing cost separately from per-proof commit/prove/verify time.

The stable advice spans remain `commit_advice` and
`AdviceOpeningEvaluation::evaluate`. No new `jolt-eval` invariant or objective
is required for this internal Akita protocol change.

## Alternatives Considered

### Keep byte one-hot advice

This retains a representation optimized for one-hot commitments but keeps a
much larger polynomial and an advice-only reconstruction protocol even though
Jolt's PIOP already consumes dense words.

### Open dense advice separately

This removes reconstruction but leaves one PCS prove/verify per advice object.
Akita's heterogeneous batching supports independent dense precommits and a
one-hot final group at different points, so separate openings are unnecessary.

### Generate every grouped parameter row ahead of time

This makes the repository catalog scale with the Cartesian product of trusted
capacity, untrusted capacity, and trace arity, and repeats the problem for each
future precommitted object. Most rows would never be used by a given program.

### Generate the selected row during proving or verification

This generates only one row, but places an expensive, memory-intensive planner
on latency-sensitive and verifier-reachable paths. It also occurs too late for
correct setup matrix sizing.

### Preprocessing provision over reachable trace arities

This is the selected design. Preprocessing collapses each advice-size dimension
to the program's configured capacity, plans the bounded set of possible trace
rows before setup construction, and leaves all proof-path operations as exact
lookups.

## References

- [`lattice-claims.md`](./lattice-claims.md) for the wider Akita protocol and
  the historical advice update.
- [`akita-optimal-committed-data.md`](./akita-optimal-committed-data.md) for the
  committed-data performance background.
- `common/src/advice.rs` for canonical word encoding.
- `crates/jolt-claims/src/protocols/jolt/lattice/packing.rs` for advice and
  `OneHotTrace` packing plans.
- `crates/jolt-akita/src/schedule_registry.rs` for preprocessing provisioning
  and runtime row resolution.
- `crates/jolt-akita/src/native_batching.rs` and `scheme.rs` for heterogeneous
  batch proving and verification.
- `crates/jolt-prover/src/akita/{stage0.rs,stage8.rs,witness.rs}` and
  `crates/jolt-verifier/src/stages/stage8/packed.rs` for the protocol wiring.
