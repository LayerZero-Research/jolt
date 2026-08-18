# Spec: Preprocessing-Time Akita Schedule Generation for Trusted Advice

| Field | Value |
|-------|-------|
| Author(s) | Jolt contributors |
| Created | 2026-08-17 |
| Status | implemented on `omid/batch-advice` (2026-08-17); gates 1 and 2 approved |
| Branch | `omid/batch-advice` (child of `omid/dense-advice`) |
| Depends on | `specs/akita-precommitted-advice-batching.md` (Phase 1, implemented at `79d5feb20`) |

Upstream paths below are relative to the pinned Akita checkout,
`~/.cargo/git/checkouts/akita-967e7e5e12514df2/1d48114/crates/`, abbreviated `«A»/`.

## Summary

`omid/batch-advice` proves trusted advice and the main packed trace in one
heterogeneous Akita batch: trusted advice is an independently committed
**precommitted group**, the packed one-hot trace is the **final group**, and one
`batched_prove`/`batched_verify` discharges both. That works today for exactly
**one** grouped shape, baked into the checked-in K256 catalog at codegen time
(`crates/jolt-akita/src/schedules/mod.rs:129-133`):

```text
TRUSTED_ADVICE_GROUP            = PolynomialGroupLayout::new(20, 1)   // 8 MiB advice
TRUSTED_ADVICE_K256_FINAL_GROUP = PolynomialGroupLayout::new(39, 1)   // log_T = 26, K=256
```

Any other trusted-advice capacity, or any other trace length, fails as
unsupported (`specs/akita-precommitted-advice-batching.md`, Approval Gate 3).
That is the Phase-1 limitation this spec removes.

The enabling observation is an asymmetry in what is known when:

- **Trusted advice size is fixed per program and known at preprocessing.** It
  comes from `#[jolt::provable(max_trusted_advice_size = ...)]`, is aligned to a
  power of two in `MemoryLayout`, and is carried in preprocessing. It cannot
  change at prove time. So the trusted precommit layout — and therefore the
  frozen `CommittedGroupProfile` every grouped row is keyed on — is a **single
  known value** at preprocessing.
- **Trace length is not known at preprocessing.** The final `OneHotTrace` group
  arity depends on `log_T`, so it can be any value in the family's supported
  range.

Therefore: at preprocessing, for the program's one trusted precommit profile,
generate the grouped schedule row for **every** final-group arity in the
family's range — `K256_NUM_VARS = (12, 43)` and `K16_NUM_VARS = (12, 34)`
(`crates/jolt-akita/src/schedules/mod.rs:117-121`) — reusing any row the
checked-in catalog already provides, and planning only the missing ones. Register
those rows in a preprocessing-owned digest→row map, and let the existing
`batched_prove`/`batched_verify` path resolve against it unchanged.

This makes grouped trusted-advice batching work for **any** program advice size
and **any** trace length, without regenerating or shipping a checked-in catalog
per shape.

Scope is **trusted advice only**. Untrusted advice keeps its Phase-1 standalone
auxiliary path.

## Key architectural finding

The design does not need `GeneratedScheduleTable`, does not need to synthesize
catalog entries, and does not need any upstream change. Upstream already
demonstrates the exact mechanism in a test-only `CommitmentConfig` impl at
`«A»/akita-pcs/tests/support/mod.rs:187-241`: override the resolution methods to
plan at runtime and hand-build a `ResolvedScheduleRow` via `try_new`.

Everything required is public:

| Need | Location |
|---|---|
| `find_schedule(key, final_policy, precommitted_policies, policy, ring_cfg)` | `«A»/akita-planner/src/planner.rs:472-478` |
| `AkitaScheduleLookupKey { final_group, precommitteds }` — both fields `pub` | `«A»/akita-types/src/schedule/profiles.rs:237-243` |
| `CommittedGroupProfile::try_from_params(layout, params)` | `«A»/akita-types/src/schedule/profiles.rs` |
| `schedule_row_digest(&profiles, &schedule)` — pure, no catalog | `«A»/akita-types/src/schedule_selection.rs:52-78` |
| `ResolvedScheduleRow::try_new(selection, profiles, schedule, policy)` | `«A»/akita-schedules/src/resolve.rs:57-80` |
| `bounded_parallel_filter_map` / `offline_planning_worker_count` | `«A»/akita-planner/src/emit/mod.rs:59-124` |
| The four overridable resolution methods (default bodies, `Self`-only inputs) | `«A»/akita-config/src/lib.rs:325-406` |

The recipe is six lines, verbatim from `«A»/akita-pcs/tests/support/mod.rs:203-241`:

```rust
let schedule = find_schedule(key, final_pol, &pre_pols, &policy, ring_cfg)?.schedule;
let profiles = CommittedGroupBatchProfile {
    final_group: CommittedGroupProfile::try_from_params(
        key.final_group, &schedule.root.params.final_group.commitment)?,
    precommitteds: key.precommitteds.clone(),
};
let selection = OpeningScheduleSelection { row_digest: schedule_row_digest(&profiles, &schedule)? };
ResolvedScheduleRow::try_new(selection, profiles, schedule, &policy_of::<Cfg>())
```

The rejected alternative — building a runtime `GeneratedScheduleTable` — was
examined and is strictly worse: it needs `FoldSchedule → GeneratedFoldScheduleEntry`,
which exists at `«A»/akita-planner/src/emit/mod.rs:191` but is **private** (only
`emit_family_module`, which returns Rust source text, is exported). That would
force either an upstream pin bump or a ~110-line hand-port that must stay
byte-identical to the emitter forever, plus per-entry re-expansion through
`schedule_from_entry` on first lookup (`«A»/akita-schedules/src/resolve.rs:117-154`).
It buys nothing the direct route does not already provide.

## Approval Gates

### Gate 1 — planner DP becomes verifier-reachable at preprocessing

`specs/akita-precommitted-advice-batching.md` states as a hard requirement:

> The verifier resolves only the checked-in grouped row. Planner DP is an
> offline generation tool, never a verification fallback.

and as an acceptance criterion:

> - [x] The verifier never invokes planner DP and accepts only the checked-in
>       exact grouped row.

This spec **narrows, but does not preserve, that property**, and there is no way
to avoid it: the grouped row depends on the program's advice capacity, which is
not known when a catalog is emitted. The proposed replacement invariant:

```text
OLD: the verifier never runs planner DP.
NEW: the verifier never runs planner DP on the proof path.
     It runs planner DP exactly once, at preprocessing, over public program
     metadata (trusted advice capacity), producing a fixed row set. Per-proof
     verification remains identity-only `resolve_schedule_selection` against
     that fixed set.
```

This is forced by the resolution API. `resolve_schedule_selection`
(`«A»/akita-config/src/lib.rs:396-406`) takes **only a 32-byte digest** and must
return the row. Prover and verifier must therefore both hold the digest→row map.
Since the map cannot be checked in, the verifier must either build it (plan) or
be handed it.

Why planning at verifier preprocessing is defensible:

- The DP input is `max_trusted_advice_size` — program metadata that already flows
  through `JoltVerifierPreprocessing`, at the same trust level as the program
  view and `preprocessing_digest`. It is not proof-controlled.
- It happens once per (program, family), amortized over every proof, off the
  latency-critical path.
- The per-proof boundary is unchanged: the verifier reads the explicit
  `OpeningScheduleSelection` from the proof and resolves it by digest only. It
  never reconstructs a lookup key and never falls back to planning on a miss.

Why it still needs explicit sign-off:

- A planning verifier can be made to do unbounded work if its preprocessing input
  is ever attacker-influenced. The bound must be an explicit enforced cap
  (Design §5), not emergent.
- Prover and verifier must derive **identical** row sets, because the row digest
  is transcript-bound. Any DP nondeterminism across thread counts, platforms, or
  Akita revisions becomes a production verification failure.

**Alternative if rejected:** ship the planned row set as an attested
preprocessing artifact produced once by a trusted party. Strictly more work
(serialization format, attestation, distribution); note `FoldSchedule` is not
obviously `AkitaSerialize` and `canonical_descriptor_bytes()` is one-way, so this
is a real serialization project, not a `serde` derive. This spec assumes the gate
is accepted.

### Gate 2 — preprocessing cost, and it is worse than it first looks

The K256 range is 32 final arities. Each missing grouped row is one
`find_schedule` DP solve. Four findings compound:

1. **No memoization across calls.** `find_schedule` allocates a fresh
   `ScheduleMemo` per call (`«A»/akita-planner/src/planner.rs:527`), and
   `ScheduleMemo`/`SuffixCtx` are `pub(crate)` with no API to inject one. So 32
   sequential calls cost exactly 32× one call — there is **no** suffix sharing
   between `num_vars = 12` and `num_vars = 13` despite heavily overlapping DPs.
2. **Grouped keys are strictly more expensive than scalar keys.** Only scalar
   keys get the cheaper `direct_only_policy`; a non-empty `precommitteds` uses
   the full policy (`«A»/akita-planner/src/planner.rs:495-505`).
3. **`JoltOneHotK256` is `AdaptiveDimension`**, which sweeps candidate dimensions
   rather than using a fixed one — the expensive search branch
   (`«A»/akita-planner/src/planner.rs:487-494`).
4. **The known datapoint is scalar-dominated.** "Minutes" covers ~117 mostly
   scalar rows at 3-way parallelism (`crates/jolt-akita/tests/schedules.rs:227`,
   and its dedicated CI lane at `.github/workflows/rust.yml:640-647`). Naively
   that is single-digit seconds per scalar row; 32 grouped adaptive rows are
   plausibly **worse per row**, so the sweep could be several minutes.

Parallelism is available but capped for a reason: each concurrent
`find_schedule` can hold up to 262,144 suffix-cache entries
(`«A»/akita-planner/src/schedule_params/suffix_dp/state.rs:111-118`), which is
why `DEFAULT_OFFLINE_PLANNING_WORKERS = 3` sits far below host parallelism.
**Raising `AKITA_SCHEDULE_GEN_JOBS` inside a proving process that already holds
the trace risks OOM** — the SHA2-chain benchmark already peaks at 28.92 GiB
(`specs/akita-advice-stack-handoff.md:264`).

`akita-setup`'s `disk-persistence` feature does **not** help: it caches the
public SIS matrix and the setup-prefix registry, not schedules
(`«A»/akita-setup/src/lib.rs:143-201`). Worse, under a runtime-resolving config
its registry cache key calls `resolve_catalog_row_for_key` just to compute a
filename (`«A»/akita-setup/src/lib.rs:157`), so it can trigger an *extra* solve
per process start unless the map is pre-populated.

**Measured (release, this branch).** The estimate above was pessimistic. One
grouped K256 solve at the `(20,1)` dense prefix:

| Final arity | Cataloged | Planner time |
|---|---|---:|
| 25 | no | 0.81 s |
| 39 | yes (reused) | 1.37 s |
| 43 | no | 1.40 s |

So the full 12..=43 sweep is roughly 30–40 s serial and ~15 s at the default 3
workers — a one-time preprocessing cost, comparable to the 4.6 s transparent
setup already in the path and far below the "several minutes" upper bound. Gate
2 is satisfied without reachable-range narrowing, which stays available as a
future optimization rather than a requirement.

Two cost properties fell out of implementation and are load-bearing:

- **Cataloged keys are free.** The sweep skips any key the checked-in catalog
  already covers, so the production `(20,1) → (39,1)` shape plans nothing, and
  the K16 fixture range plans nothing.
- **Each family sweeps its own catalog's range**, not the production grid. The
  fixture family uses `FIXTURE_K16_FINAL_NUM_VARS` (22..=26); sweeping the wider
  `K16_NUM_VARS` would plan 18 rows no fixture can reach.

## Intent

### Goals

- Derive the trusted-advice precommit layout and frozen profile from
  `max_trusted_advice_size` at preprocessing, for any supported capacity.
- Generate, at preprocessing, the grouped row for that one frozen profile against
  every final-group arity in the family's declared range.
- Reuse a checked-in row whenever the static catalog already covers the exact
  `(trusted_profile, final_layout)` key; plan only what is missing.
- Register the resulting rows once, in prover and verifier preprocessing, and
  resolve every commit/open/prove/verify against them through the **existing**
  `batched_prove`/`batched_verify` APIs — no new backend protocol, no wire change.
- Keep the per-proof verifier boundary identity-only.
- Remove the Phase-1 hard-coded `(20,1) → (39,1)` restriction.

### Non-goals

- Untrusted advice; bytecode / program-image objects. Unchanged Phase-1 paths.
- Deleting any checked-in catalog. Scalar rows remain the source of truth for
  no-advice proofs, auxiliary objects, and the standalone dense profile.
- Building a runtime `GeneratedScheduleTable` (see "Key architectural finding").
- Any upstream Akita change or pin bump.
- Changing the dense advice encoding, advice sumchecks, the PIOP, group ordering,
  point conventions, `akita + zk` rejection, Dory, or BlindFold.
- Runtime planning on the proof path, in either prover or verifier.

## Design

### 1. Trusted precommit shape and frozen profile

At preprocessing, from `memory_layout.max_trusted_advice_size`:

```text
word_capacity  = common::advice::advice_word_capacity(max_bytes)      // common/src/advice.rs:47
word_vars      = log2(word_capacity)
physical_vars  = max(word_vars, DENSE_ADVICE_MIN_PHYSICAL_VARS = 14)  // packing.rs:147
require physical_vars <= DENSE_ADVICE_MAX_PHYSICAL_VARS = 34          // packing.rs:148
trusted_layout = PolynomialGroupLayout::new(physical_vars, 1)
```

The frozen profile prefers the checked-in dense row and plans only on a miss:

1. `JoltDense::profile_without_precommitted_groups(trusted_layout)`
   (`«A»/akita-config/src/lib.rs:359-366`) — resolves the checked-in dense scalar
   row. The dense catalog covers 14..=34, so this hits for every legal capacity.
2. On a miss, plan directly: `find_schedule` with empty `precommitteds`, then
   `CommittedGroupProfile::try_from_params(layout, &schedule.root.params.final_group.commitment)`
   — the planner-direct form used by codegen at
   `crates/jolt-akita/src/schedules/mod.rs:166-173`.

Step 2 makes the design total but is unreachable under the current dense catalog.
A test must assert step 1 covers the whole legal capacity range — coverage the
dense catalog has never had.

**Runtime and codegen intentionally derive this profile differently, and must
agree byte-for-byte.** Codegen must *not* read the dense catalog it is
regenerating — `planned_profile_without_precommitted_groups`
(`crates/jolt-akita/src/schedules/mod.rs:166-173`) runs a fresh planner solve
precisely so a policy change cannot embed a stale prefix
(`specs/akita-precommitted-advice-batching.md:365-396`). Runtime does the
opposite: it reads the catalog, because the catalog row is what the independent
trusted commit will actually produce. A test must assert the two paths yield
byte-identical `CommittedGroupProfile`s for the same layout; a silent divergence
would produce grouped rows keyed on a prefix no commit can ever match.

**Invariant (unchanged from Phase 1):** the profile a grouped row is keyed on
must be *the* profile the independent trusted commit actually produces. The
authoritative value is lifted from the commit-time hint, not re-looked-up
(`AkitaScheme::trusted_precommitted_profiles`,
`crates/jolt-akita/src/scheme.rs:537-563`), and the prover still compares it for
full equality.

### 2. Grouped-row provisioning sweep

For the active family `Cfg` and its range `(min_vars, max_vars)`:

```text
for v in min_vars..=max_vars:
    key = AkitaScheduleLookupKey {
        final_group:   PolynomialGroupLayout::new(v, 1),
        precommitteds: vec![trusted_profile],
    }
    if the static catalog resolves this exact key:
        reuse that row                       // "already exists → do not regenerate"
    else:
        plan it (six-line recipe above), with
            final policy       = Cfg::root_honest_fold_policy()
            precommitted specs = vec![JoltDense::root_honest_fold_policy()]
        schedule.validate_structure()
        register (row_digest -> ResolvedScheduleRow)
```

`precommitted_honest_fold_policies` is positional and must have the same length
as `key.precommitteds` (`«A»/akita-planner/src/planner.rs:509-512`); the group's
geometry enters through the `CommittedGroupProfile` in the key, not the policy
spec. This is the same call shape as `emit::regen_group_batch`
(`crates/jolt-akita/src/schedules/mod.rs:151-164`) and
`k16_fixture_group_batch_keys` (`:189-206`) — the latter already sweeps a
final-arity range against one fixed trusted profile, offline. This spec moves
that loop to preprocessing and parameterizes the trusted shape.

Only `num_polys = 1` grouped rows are generated: every grouped row today
(production and fixture) is single-polynomial, and prefix packing yields one
physical polynomial (`crates/jolt-akita/src/schedules/mod.rs:111-114`).

The sweep runs through `bounded_parallel_filter_map` with
`offline_planning_worker_count` (`«A»/akita-planner/src/emit/mod.rs:59-124`) —
both public, order-preserving, and using a private scoped pool rather than the
workspace Rayon pool. Given Gate 2's memory finding, the worker count must be
**explicitly bounded and not raised via `AKITA_SCHEDULE_GEN_JOBS`** inside a
proving process.

**Not every arity in a family's range admits a grouped row.** Adding a precommit
raises the fold-count floor, so a final arity near the bottom of the scalar range
can have no multi-group schedule at all — `num_vars = 12` under K256 fails with
`no multi-group schedule with at least two folds`. The sweep treats
`AkitaError::UnsupportedSchedule` as "this arity has no row" and skips it,
logging the arity and reason; a proof at such an arity fails its own shape
validation. Every other planner error propagates, so a real failure is never
swallowed.

Two properties every planned row must satisfy, checked at registration:

- **Direct-only.** `shape_guard::validate_level_shape`
  (`crates/jolt-akita/src/shape_guard.rs:477-492`) rejects any schedule with
  `incoming_setup_prefix.is_some()` or a stage-3 payload — "Jolt's presets plan
  direct-only schedules." All Jolt presets set `recursive_setup_planning: false`,
  so this should hold by construction, but it must be asserted at registration
  rather than discovered at verification.
- **Exact layout.** `effective_batched_schedule`
  (`«A»/akita-config/src/schedule_selection.rs:33-37`) requires
  `resolved.profiles().opening_layout()` to equal the statement's opening batch
  exactly, so a row's final group must be `(setup.max_num_vars, 1)` and its
  prefix `(trusted physical vars, 1)` on the nose.

The registry additionally rejects duplicate row digests, mirroring the catalog
invariant at `«A»/akita-schedules/src/resolve.rs:145-152`.

### 3. Runtime row registry

A per-family, install-once registry mapping `ScheduleRowDigest → ResolvedScheduleRow`,
plus a `AkitaScheduleLookupKey → ResolvedScheduleRow` view for the honest-prover
lookups. Jolt's `delegate_preset!` (`crates/jolt-akita/src/configs.rs:169-171`)
gains overrides for the resolution methods:

| Method | Behavior |
|---|---|
| `resolve_catalog_row_for_key` | registry first, then the static catalog |
| `resolve_catalog_row_for_profiles` | registry first, then the static catalog |
| `resolve_schedule_selection` | registry first, then the static catalog |
| `resolve_catalog_row_for_opening`, `profile_without_precommitted_groups` | inherit (they delegate to the above) |

These four cover all six resolution sites, two in Jolt and four inside upstream
Akita (which cannot be reached any other way):

| Site | Call | Location |
|---|---|---|
| Stage 0 pre-flight | `resolve_catalog_row_for_key` | `crates/jolt-akita/src/scheme.rs:160-186` |
| Commit with precommit | `resolve_catalog_row_for_key` | `«A»/akita-prover/src/api/commitment.rs:552` |
| Batch selection | `resolve_catalog_row_for_profiles` | `«A»/akita-prover/src/types/opening_data.rs:203` |
| Prover backend replay | `resolve_schedule_selection` | `«A»/akita-prover/src/protocol/core/prove.rs:91` |
| Verifier shape guard | `resolve_schedule_selection` | `crates/jolt-akita/src/shape_guard.rs:227` |
| Verifier backend replay | `resolve_schedule_selection` | `«A»/akita-verifier/src/protocol/core/verify.rs:303` |

**Ownership: preprocessing holds the rows; the ambient map only publishes them.**
The resolution hooks are `CommitmentConfig` associated functions with no
receiver, so they cannot read a preprocessing object. `JoltVerifierPreprocessing`
therefore owns a `trusted_advice_schedules: RegisteredRows` field, and
provisioning also publishes those rows into a process-wide union the hooks
consult. The field is `#[serde(skip)]` — `FoldSchedule` derives only
`Clone, Debug, PartialEq, Eq` (no serde, no `AkitaSerialize`), so rows cannot be
serialized at all. They are a pure function of the public advice capacity, so
`JoltVerifierPreprocessing::provision_akita_schedules` rebuilds them on the
deserialized side and returns the row-set digest for an explicit prover/verifier
comparison.

Registry rules:

- **The ambient map is a union, not one set per config.** Rows are addressed by
  exact row digest and exact committed profiles, so two programs with different
  advice capacities are simultaneously live without aliasing. This is what makes
  a long-lived multi-program prover process work; an earlier install-once rule
  rejected the second program outright.
- Re-publishing an identical row is a no-op (the normal prover-then-verifier
  case, and the post-deserialize rebuild).
- A digest carrying *different* profiles is a genuine collision and an error.
- **Nothing published ⇒ behavior byte-identical to today.** Every existing test
  must pass unchanged after this step alone.
- Registry misses fall through to the static catalog, never to the planner. This
  is what keeps "no DP on the proof path" mechanically true.

**Re-entrancy: the "already cataloged?" probe must bypass the registry.**
Because the overrides consult the registry first, using
`Cfg::resolve_catalog_row_for_key` to decide whether a key needs planning makes
provisioning non-idempotent: the second call sees the rows the first call
installed, skips them all, and installs an empty set — which then trips the
install-once check. The probe therefore goes through
`schedule_registry::catalog_only_row`, which calls
`resolve_generated_catalog_row_for_key` directly. `catalog_setup_capacity` uses
the same catalog-only path when iterating static entries, for the same reason.
This was caught by `reinstalling_a_different_row_set_is_rejected`, not by
inspection.

### 3b. Setup-capacity accounting must also see registry rows

This is the one place where the resolution-method overrides are *not* sufficient,
and it is easy to miss because it fails late.

`catalog_setup_capacity` (`crates/jolt-akita/src/configs.rs:40-80`) sizes the
prover setup by **iterating `table.entries` directly** — the static slice, not a
resolution call. For each entry it accounts the individually-fitting prefix
groups' commit-only footprint (`:57-68`, deliberately before the whole-key
`fits_setup_capacity` filter at `:70-72`), then the complete grouped schedule
(`:73-77`). Registry-resident rows are invisible to that loop, so a setup sized
under the current code would not cover a runtime-planned grouped row's matrices.

Two further constraints make this ordering-sensitive:

- Setup sizing runs **earlier than commit time**, so the registry must be
  populated before `setup_matrix_capacity` is first called — i.e. provisioning
  (§2) must precede setup construction in preprocessing, not merely precede
  proving.
- The existing accounting is non-monotone by design: matrix requirements do not
  follow from the scalar `(max_vars, total_polys)` footprint, which is why the
  loop exists at all (`specs/akita-precommitted-advice-batching.md:301-344`).

Required change: `catalog_setup_capacity` takes the component-wise maximum over
the static entries **and** the registered rows, using the same prefix-then-whole-key
accounting for both. The existing test
`grouped_setup_capacity_covers_precommit_and_complete_schedule`
(`crates/jolt-akita/tests/schedules.rs:94-124`) is the template: extend it to a
registry-provisioned row and assert `setup_matrix_capacity(v, 2)` covers both the
full grouped schedule and the precommit-only footprint, while `(v, 1)` still
covers the precommit footprint alone.

### 4. Preprocessing storage and the prover/verifier contract

- **Prover** (`crates/jolt-prover/src/preprocessing.rs:36-45`, and legacy
  `JoltProverPreprocessing`): store the resolved trusted precommit profile, a
  digest over the installed row set, and the trusted dense `ProverSetup` used for
  the precommit. This branch already separates `advice_dense_setup` from
  `commit_advice_dense_with_setup`
  (`crates/jolt-prover/src/akita/witness.rs:416-462`), so the setup is already a
  first-class value — it simply is not retained.
- **Verifier** (`crates/jolt-verifier/src/preprocessing.rs:158-182`): store the
  same row-set digest alongside the existing
  `trusted_advice_setup: Option<PCS::VerifierSetup>`.
- Both sides run §1–§3 over the same public input (`max_trusted_advice_size` +
  family). Determinism is the correctness requirement; the row-set digest is the
  check, and prover/verifier equality is enforced **before the transcript
  bridge**, so a mismatch surfaces as a clear error rather than an opaque
  verification failure.
- Serialization: prover preprocessing already excludes non-serializable Akita
  setups and rebuilds on deserialize
  (`crates/jolt-prover-legacy/src/zkvm/prover.rs:2529-2606`). Same rule here —
  the row-set digest is serialized; the rows are rebuilt.

The proof is unchanged: it still carries the explicit `OpeningScheduleSelection`
row digest, and the verifier still resolves it by digest.

### 5. Bounding and reducing the sweep

Required:

- A hard cap on planned rows per install (the family range width, ~32).
  Exceeding it is an error. This is what makes "the verifier plans" bounded.
- `max_trusted_advice_size` validated against 14..=34 **before** any planning, so
  an out-of-range capacity fails immediately rather than after 32 DP solves.
- An explicit worker bound for the sweep, independent of
  `AKITA_SCHEDULE_GEN_JOBS`, given the 262k-entries-per-worker memory profile.

Optional, pending Gate 2's measurement:

- **Reachable-range narrowing — the highest-leverage reduction.** 12..=43 is the
  family's declared span, not what a real trace produces. Final arity is
  `ONE_HOT_TRACE_LAYOUT.setup_shape(...)` = `log_T + log_K + selector_vars`, with
  a `2^12` trace floor already enforced. The existing
  `catalogs_cover_every_reachable_one_hot_trace_shape` test
  (`crates/jolt-akita/tests/schedules.rs`) already computes reachable shapes —
  reuse that computation rather than re-deriving it. Measure this first; it may
  cut the sweep by more than half.
- **On-demand planning** (plan the profile at preprocessing, defer grouped rows
  until `log_T` is known — one row instead of 32). Listed for completeness only:
  it puts DP on the proof path and contradicts Gate 1. **Not recommended.**

Explicitly *not* a lever: `disk-persistence` (Gate 2). If cross-process
amortization of the sweep is wanted, it is a separate serialization project.

### 6. The Phase-1 fixed-shape restriction

The draft called for deleting `TRUSTED_ADVICE_GROUP`,
`TRUSTED_ADVICE_K256_FINAL_GROUP`, and `k256_group_batch_keys`, and regenerating
the K256 catalog without its grouped row. Implementation showed that is wrong on
two counts, so **nothing is deleted**:

- **There is no explicit restriction to remove.** Grep shows those constants are
  referenced only by the offline emitter and by tests — never by prover or
  verifier code. `AkitaScheme::validate_trusted_trace_precommit`
  (`crates/jolt-akita/src/scheme.rs:144-196`) already builds its key from the
  actual committed profile and resolves it. The Phase-1 limit was *implicit* — a
  catalog miss — and provisioning removes it by supplying the missing row.
- **The checked-in grouped row is the reuse case.** Keeping it means the
  production `(20,1) → (39,1)` shape plans nothing, which is exactly the
  "already exists → do not regenerate" behavior this change is built around.
  Deleting it would force the production shape to be replanned on every process
  start.

The K16 fixture rows are likewise kept, and additionally serve as the static,
independently-generated oracle the runtime sweep is asserted equal against —
genuine independent ground truth, not a reimplementation oracle.

### 7. What was built

| Piece | Location |
|---|---|
| Registry: digest→row map, install-once, duplicate/direct-only rejection, row-set digest | `crates/jolt-akita/src/schedule_registry.rs` |
| Catalog-only probe (re-entrancy fix) | `schedule_registry::catalog_only_row` |
| Runtime trusted profile from the dense catalog | `schedule_registry::dense_precommit_profile` |
| Provisioning sweep, parallel + unschedulable-arity skip | `schedule_registry::provision` |
| Per-K family entry point | `schedule_registry::provision_trusted_advice_for_k` |
| Resolution overrides on all three hooks | `crates/jolt-akita/src/configs.rs` (`delegate_preset!`) |
| Setup capacity folding in registry rows | `configs::catalog_setup_capacity` + `fold_row_capacity` |
| Advice arity helper + provisioning wrapper | `crates/jolt-prover-legacy/src/zkvm/packed.rs` |
| Prover-side call (before setup sizing) | `AkitaPackedProver::one_hot_trace_setup_params` |
| Preprocessing-owned rows + rebuild-on-load | `JoltVerifierPreprocessing::{trusted_advice_schedules, provision_akita_schedules}` |
| Verifier-side call | `akita_verifier_preprocessing` |

Deferred, and not required for correctness: comparing the row-set digest before
the transcript bridge. `provision_akita_schedules` returns it, but no caller
asserts prover/verifier equality yet. Both sides derive from the same public
advice capacity, so a divergence would surface as a selection-resolution
failure rather than a named preprocessing mismatch.

Also deferred (see "Later Phases"): provisioning K=16 and K=256 together so
preprocessing is self-contained before the trace fixes K. Today each call site
provisions the K it already knows.

## Component Changes

| Component | Change |
|---|---|
| `crates/jolt-akita/src/schedule_registry.rs` (new) | Per-family install-once digest→row registry; planned-row cap; worker bound |
| `crates/jolt-akita/src/schedules/mod.rs` | Extract trusted-profile derivation + grouped sweep so codegen and runtime share one implementation; delete `TRUSTED_ADVICE_GROUP` / `TRUSTED_ADVICE_K256_FINAL_GROUP` / `k256_group_batch_keys` |
| `crates/jolt-akita/src/schedules/jolt_fp128_onehot_k256.rs` | Regenerate without the fixed grouped row |
| `crates/jolt-akita/src/configs.rs` | `delegate_preset!` overrides the four resolution methods to consult the registry then the static catalog; `catalog_setup_capacity` (`:40-80`) additionally folds in registered rows (§3b) |
| `crates/jolt-akita/src/lib.rs` | Export the provisioning entry point and registry |
| `crates/jolt-akita/src/shape_guard.rs` | Unchanged in mechanism; verify it resolves through the registry-backed path |
| `crates/jolt-prover/src/preprocessing.rs` | Store trusted profile, row-set digest, trusted dense prover setup |
| `crates/jolt-prover/src/akita/{prover,stage0,stage8}.rs` | Read the retained trusted setup/profile; drop fixed-shape guards |
| `crates/jolt-verifier/src/preprocessing.rs` | Store the row-set digest next to `trusted_advice_setup` |
| `crates/jolt-verifier/src/stages/stage8/packed.rs` | Row-set digest equality check before the bridge |
| `crates/jolt-prover-legacy/src/zkvm/{prover,packed}.rs` | Mirror preprocessing storage and provisioning; `akita_verifier_preprocessing` derives the verifier row-set digest from the same result |
| `crates/jolt-prover/examples/modular_benchmark.rs` | Report provisioning time as its own metric; drop the fixed-shape assumption |
| `crates/jolt-akita/tests/schedules.rs` | Runtime-vs-fixture equality; dense coverage; cap, install-once, and fall-through tests |
| `crates/jolt-prover/tests/{akita_e2e,akita_byte_diff}.rs` | Multiple advice capacities × multiple trace lengths |

## Testing Plan

Independent-ground-truth tests (not reimplementation oracles):

- **Runtime ≡ checked-in fixture.** Provision the fixture family at
  `FIXTURE_TRUSTED_ADVICE_GROUP` over `FIXTURE_K16_FINAL_NUM_VARS`; assert every
  planned row's digest equals the corresponding checked-in fixture row's digest.
  This is why §6 keeps the fixture rows.
- **Reuse, not regenerate.** With the fixture catalog present, assert the sweep
  plans zero rows.
- **Determinism.** Provision twice in separate processes and at different worker
  counts; assert equal row-set digests.
- **Dense coverage.** `profile_without_precommitted_groups` succeeds for every
  `physical_vars` in 14..=34 — coverage the dense catalog has never had.
- **Runtime profile ≡ codegen profile.** For every legal `physical_vars`, the
  catalog-read profile (runtime) equals the planner-solved profile (codegen)
  byte-for-byte (§1).
- **Registry discipline.** Cap exceeded ⇒ error; second install with a different
  row set ⇒ error; identical ⇒ no-op; duplicate row digest ⇒ error; a planned row
  that is not direct-only ⇒ error; nothing installed ⇒ byte-identical behavior.
- **Setup capacity over registry rows.** Extending
  `grouped_setup_capacity_covers_precommit_and_complete_schedule`
  (`crates/jolt-akita/tests/schedules.rs:94-124`) to a registry-provisioned row:
  `(v, 2)` covers the full grouped schedule and the precommit-only footprint;
  `(v, 1)` covers the precommit footprint but rejects the grouped key.
- **Capacity/trace matrix e2e.** Modular and legacy proofs accept and are
  byte-identical across ≥2 trusted capacities × ≥2 trace lengths — the case
  Phase 1 could not express at all.
- **Verifier stays identity-only.** A counter or poisoned test hook asserting no
  `find_schedule` call occurs on the verification path, so Gate 1's narrowed
  invariant is enforced mechanically rather than by convention.
- **Tamper.** A selection digest absent from the registry rejects; a verifier
  preprocessed at a different advice capacity rejects on row-set digest mismatch
  before the bridge.

Validation commands:

```bash
cargo fmt -q
cargo clippy --all --features host,akita -q --all-targets -- -D warnings
cargo nextest run -p jolt-akita --features akita-test-schedules --cargo-quiet
cargo nextest run -p jolt-prover --features prover-fixtures,akita --cargo-quiet
cargo nextest run -p jolt-verifier --features prover-fixtures,akita --cargo-quiet
cargo nextest run -p jolt-prover-legacy --features host,akita,akita-test-schedules --cargo-quiet
# non-Akita regression
cargo nextest run -p jolt-prover-legacy muldiv --cargo-quiet --features host
cargo nextest run -p jolt-prover-legacy muldiv --cargo-quiet --features host,zk
```

Benchmark: rerun SHA2-chain scale 26, K256, 8 MiB trusted advice
(`specs/akita-advice-stack-handoff.md:220-232`) and report provisioning time as a
distinct metric. Commit/prove/verify must be unchanged within noise — this moves
work into preprocessing, it does not alter the proving path.

## Acceptance Criteria

- [x] Gate 1 approved: "no DP on the proof path" replaces "no DP on the verifier".
- [x] Gate 2 approved: measured at ~1.2 s per grouped solve, ~15–40 s per sweep.
- [x] Trusted precommit profile derived from `max_trusted_advice_size` for any
      capacity in 14..=34 physical vars, with dense coverage tested
      (`dense_catalog_covers_every_legal_advice_arity`).
- [x] Runtime (catalog-read) and codegen (planner-solved) dense profiles agree
      byte-for-byte across the whole range
      (`runtime_and_generated_dense_profiles_agree`).
- [x] Grouped rows provisioned once at preprocessing across the family range,
      reusing checked-in rows and planning only the missing ones
      (`provisioning_reuses_cataloged_rows_instead_of_planning_them`).
- [x] A previously unseen advice capacity provisions a complete row set, and
      every row resolves through the public-selection boundary
      (`a_new_advice_capacity_provisions_and_resolves_every_final_arity`).
- [x] Runtime-planned rows reproduce the checked-in rows byte-for-byte
      (`planned_rows_match_the_checked_in_fixture_rows`).
- [x] Per-proof verification remains identity-only resolution; registry misses
      fall through to the static catalog, never to the planner.
- [x] Setup-capacity accounting covers registry rows, and provisioning is ordered
      before setup construction (`setup_capacity_covers_a_provisioned_row`).
- [x] Every planned row is direct-only; duplicate digests carrying different
      profiles are rejected (`a_digest_collision_with_different_profiles_is_rejected`),
      and re-publishing an identical set is a no-op
      (`republishing_an_identical_row_set_is_a_no_op`).
- [x] Two programs with different advice capacities are live in one process and
      neither resolves the other's row
      (`two_advice_capacities_coexist_without_aliasing`).
- [x] Preprocessing owns the rows and rebuilds them after a serde roundtrip
      (`JoltVerifierPreprocessing::provision_akita_schedules`).
- [x] `batched_prove` / `batched_verify` used unchanged — no new backend protocol,
      no wire-format change, no upstream Akita change or pin bump.
- [x] Untrusted advice, bytecode, and program-image paths behaviorally unchanged.
- [x] With nothing installed, behavior is byte-identical to today (whole
      pre-existing suite passes: 54 jolt-akita, 18 jolt-prover, 90 jolt-verifier,
      456 jolt-prover-legacy, plus clear/ZK Dory muldiv).
- [ ] Row-set digest stored in preprocessing and compared before the bridge
      (deferred, §7).
- [ ] SHA2-chain scale-26 benchmark rerun with provisioning reported separately.

## Later Phases (not in scope)

0. **Both-K provisioning.** Provision K=16 and K=256 in one pass so
   preprocessing is self-contained before the trace fixes K — roughly 55 solves
   (~20 s at 3 workers) instead of ~32.

   Note what this phase is *not*: dropping the checked-in scalar catalogs.
   `jolt_fp128_onehot_k16` (46 rows), `jolt_fp128_onehot_k256` (64) and
   `jolt_fp128_dense` (42) are one-dimensional in arity with no advice
   dimension, so they never grow with advice sizes — the combinatorial blowup
   this spec avoids is `#trusted × #untrusted × #arity`, and it never
   materializes. Planning those ~152 scalar rows at every preprocessing would
   cost roughly three minutes to delete files that are already bounded.

1. **Untrusted advice** as a second precommitted group: two-prefix keys, group
   order `[UntrustedAdvice, TrustedAdvice, OneHotTrace]`, total setup capacity 3.
   Untrusted capacity is also preprocessing-fixed, so the same mechanism applies —
   but the sweep becomes a product over two frozen profiles unless both are
   pinned together. Given Gate 2's cost findings, that product is the question to
   answer before starting.
2. **Bytecode / program-image objects** through the same provisioning path. Their
   shapes are likewise preprocessing-known.
