# Spec: Akita Precommitted Advice and Main-Trace Batching

| Field | Value |
|-------|-------|
| Author(s) | Jolt contributors |
| Created | 2026-08-14 |
| Status | implemented and validated in the current working tree (2026-08-14) |
| PR | |

## Summary

The current Akita path commits `OneHotTrace`, trusted advice, and untrusted
advice as separate commitment objects and also proves and verifies each object
through a separate Akita PCS invocation. Dense advice made the advice object
small, but it did not fuse its final opening with the main packed trace.

Akita already supports the required distinction:

- any number of **precommitted groups** are committed independently, before the
  final group is known;
- one **final group** is committed later using a schedule selected from the
  exact ordered profiles of the precommitted groups; and
- all groups, even when their polynomial representations and opening points
  differ, are discharged by one `batched_prove` and one `batched_verify`.

Phase 1 applies that protocol to exactly these Akita groups, in this public
Akita group order:

1. `TrustedAdvice`: one dense polynomial, committed independently;
2. `OneHotTrace`: one streamed one-hot polynomial, committed as the final
   group using the trusted group's frozen profile.

The two commitments remain separate. The change is one joint opening proof and
one joint verification for those two commitments. A proof with no trusted
advice continues to use a one-group `OneHotTrace` opening. In Phase 1,
untrusted advice and committed-program objects remain auxiliary commitment
objects with their existing separate opening proofs. A later phase can add
untrusted advice as a second precommitted group.

The first checked-in grouped schedule is intentionally the concrete SHA2-chain
benchmark shape: 8 MiB of trusted advice (`2^20` `u64` words) and a `2^26`
K=256 trace. This produces a dense precommitted group `(20 variables,
1 polynomial)` and a final `OneHotTrace` group `(39 variables, 1 polynomial)`.

This is a protocol and wire-format change. It requires a new commitment-config
variant, new Fiat-Shamir domains, grouped statement validation, and a
multi-group pre-deserialization shape guard. It does not change the Jolt PIOP,
the dense advice encoding, or any advice sumcheck.

## Approval Gates

These are not interchangeable implementation details. The instruction to
proceed with implementation confirmed all three Phase-1 assumptions below.

### 1. Trusted-advice capacity

The requested sizes in the task are not equivalent:

| Requested spelling | `u64` words | Bytes | Dense group |
|---|---:|---:|---:|
| `2^20 u64` | `2^20` | `2^23` (8 MiB) | `(20, 1)` |
| `2^32 bytes` | `2^29` | `2^32` (4 GiB) | `(29, 1)` |

They differ by a factor of 512 and select different frozen profiles and
different grouped schedule rows. The existing benchmark
`sha2_chain_akita_trusted_advice_2pow20_words_nv26_20260813.md` uses the first
case.

**Phase-1 assumption:** `2^20` `u64` words, i.e. `2^23` bytes (8 MiB), is the
intended capacity. No implementation or schedule generation should begin if
the intended value is instead 4 GiB; this spec's concrete key must first be
changed from `(20, 1)` to `(29, 1)`.

### 2. Untrusted advice

One trusted-precommit schedule can prove `[TrustedAdvice, OneHotTrace]`. It
cannot also prove untrusted advice in the same Akita root. Doing so requires a
different checked-in schedule whose ordered groups are
`[UntrustedAdvice, TrustedAdvice, OneHotTrace]` and requires untrusted advice
to be committed before the main trace.

**Phase-1 assumption:** trusted advice is batched with the main trace;
untrusted advice remains a separate auxiliary PCS proof and verification. In
particular:

| Present objects | Phase-1 PCS proofs/verifications |
|---|---:|
| Main only | 1 |
| Trusted + main | 1 joint |
| Untrusted + main | 1 main + 1 untrusted |
| Untrusted + trusted + main | 1 trusted/main joint + 1 untrusted |

Thus Phase 1 removes the separate **trusted-advice** prove/verify path, not
every possible advice prove/verify path. If all advice must be fused now, the
two-precommit schedule is in scope now rather than later.

### 3. Final-group shape

The request fixes the advice capacity but does not independently state the
main packed trace shape. This spec associates it with the cited benchmark:

- `log_T = 26`;
- K=256, so `log_K = 8`;
- 32 fixed selector slots, so 5 selector variables; and
- `OneHotTrace` physical arity `26 + 8 + 5 = 39`.

That arithmetic is explanatory only. The implementation derives the
authoritative final shape with
`ONE_HOT_TRACE_LAYOUT.setup_shape(&OneHotTraceShape { ... })` and requires the
result to equal `(num_vars=39, num_polys=1)` for this fixed row. It must not
reimplement the layout calculation in scheduler or setup code.

**Phase-1 assumption:** the only new production grouped row is final `(39, 1)`
with trusted prefix `(20, 1)`. A present trusted input at any other main or
trusted shape must fail early as unsupported; it must not fall back silently
to runtime planning or to a separate trusted proof. No-trusted proofs may
continue to use existing scalar rows. Supporting every K=16/K=256 trace shape
requires a declared schedule matrix and is a later generalization.

## Intent

### Goals

- Preserve independent trusted-advice and main-trace commitments.
- Commit trusted advice without any knowledge of the later main trace.
- Freeze and retain the exact Akita profile selected by that independent
  trusted commitment.
- Select the final `OneHotTrace` schedule using that exact profile.
- Produce one Akita batched opening proof for trusted advice plus
  `OneHotTrace`, even though the groups use different representations,
  dimensions, and points.
- Verify those two groups with one Akita `batched_verify` invocation.
- Retain the streamed `TracePackedOneHot` implementation and dense advice
  representation without materializing a generic full main polynomial.
- Make grouped schedule selection, group ordering, setup capacity, transcript
  binding, and proof deserialization fail closed.
- Keep modular and legacy Akita proof bytes identical.
- Preserve no-advice proofs and the Phase-1 auxiliary paths for untrusted
  advice and committed-program objects.

### Non-goals

- Combining trusted advice and `OneHotTrace` into one commitment.
- Delaying the trusted commitment until the main trace exists.
- Batching untrusted advice in Phase 1.
- Batching committed bytecode or program-image objects in Phase 1.
- Generating grouped rows for every supported trace/advice shape.
- Changing dense advice's canonical little-endian `u64` encoding, prefix-floor
  embedding, range semantics, or final claim.
- Removing `RamValCheck`, `AdviceClaimReductionCyclePhase`, or
  `AdviceClaimReduction`.
- Changing Dory, BlindFold, or the non-Akita proof path.
- Adding `akita + zk` support.
- Using a runtime scheduler on the verifier.
- Attributing a fused PCS prove or verify time exactly between advice and main;
  the fused operation has no such additive boundary.

## Terminology and Protocol Roles

| Term | Meaning in this spec |
|---|---|
| Commitment group | One Akita commitment covering one or more polynomials at one group-local arity. Jolt Phase 1 additionally requires each group to be representation-homogeneous, and both groups contain one polynomial. |
| Independent/scalar row | The Akita schedule used when committing a group without prior groups. Trusted advice always uses this row. |
| Frozen profile | The complete `CommittedGroupProfile` returned by the independent commit: layout, block/live geometry, decomposition bases, and exact commitment matrices. It is more specific than arity and flavor. |
| Precommitted group | An independently committed group whose frozen profile is supplied when selecting and committing the final group. |
| Final group | The last group in Akita's public group ordering. In Jolt this is `OneHotTrace`. |
| Grouped row | A checked-in schedule keyed by the final layout and the exact ordered frozen profiles of all precommitted groups. |
| Joint PCS proof | One Akita proof over ordered group-local claims. It does not imply a joint commitment or one common point. |
| Outer Jolt order | Jolt's existing commitment absorption order: main, untrusted, trusted, then committed-program objects. |
| Akita group order | Public order required by the backend: all prefixes first, final group last. Phase 1 is trusted, then main. |

Akita's internal root implementation may process the final group before prefix
groups. Jolt must never expose or serialize that internal processing order.
All public claims, hints, commitments, points, role tags, and profiles use
`[precommitted groups..., final group]`.

The upstream contract was audited at the Akita revision pinned by this branch,
`74c17ba4f33ed10edf8b137b6869aba199ba07fe`. Its canonical heterogeneous test
is `crates/akita-pcs/tests/akita_fp128_e2e/heterogeneous.rs` in that repository:
it independently commits one-hot and dense prefixes, commits a one-hot final
group with their ordered profiles, and performs one proof and one
verification. The implementation should follow that public flow rather than
depending on Akita's internal final-first processing order.

## Current and Target Architecture

### Current dense-advice path

```text
trusted bytes ──commit independently──> C_trusted, hint_trusted
trace rows    ──commit independently──> C_main,    hint_main

Stage 8:
  Akita prove(C_main,    r_main,    v_main)    -> pi_main
  Akita prove(C_trusted, r_trusted, v_trusted) -> pi_trusted

Verifier:
  Akita verify(pi_main)
  Akita verify(pi_trusted)
```

The two `prove` calls select unrelated scalar schedules. The main commitment
is currently created with
`GroupContext::scheduler_without_precommitted_groups()`.

### Phase-1 target

```text
trusted bytes
  └─ independent JoltDense commit
       -> C_trusted, hint_trusted, frozen_profile_trusted

frozen_profile_trusted + trace rows
  └─ grouped JoltOneHotK256 final commit
       -> C_main, hint_main

Stage 8 ordered groups:
  0: (C_trusted, r_trusted, [v_trusted], hint_trusted, Dense)
  1: (C_main,    r_main,    [v_main],    hint_main,    TraceOneHot) [final]

  one Akita batched_prove -> pi_trusted_and_main

Verifier:
  one Akita batched_verify(pi_trusted_and_main)
```

`C_trusted` and `C_main` are still produced by two distinct commit calls and
may have different commitment parameters. Only the opening proof is fused.

## Concrete Phase-1 Schedule

### Trusted precommit profile

For the assumed 8 MiB capacity:

```text
max_trusted_advice_bytes = 2^23
word_count               = 2^23 / 8 = 2^20
logical_word_vars        = 20
physical_vars            = max(20, dense_floor=14) = 20
polynomial_count         = 1
backend                   = JoltDense
```

The independent profile is obtained only through the checked-in scalar dense
row:

```text
trusted_layout  = PolynomialGroupLayout::new(20, 1)
trusted_profile = JoltDense::profile_without_precommitted_groups(trusted_layout)
```

The trusted commitment uses its dense shape-exact setup and:

```text
GroupContext::scheduler_without_precommitted_groups()
```

The exact returned profile must equal `trusted_profile`. Arity, flavor, and
layout digest alone are not a substitute for this equality.

### Final main profile

For SHA2-chain at padded trace length `2^26` with K=256:

```text
main_logical_vars = log_T + log_K = 26 + 8 = 34
selector_slots    = 32
selector_vars     = 5
main_physical_vars = ONE_HOT_TRACE_LAYOUT.setup_shape(...).num_vars = 39
polynomial_count   = 1
backend            = JoltOneHotK256 / TracePackedOneHot
```

The equality to 39 is an assertion on the canonical layout result, not a
second shape definition.

The new grouped lookup key is exactly:

```text
AkitaScheduleLookupKey {
    final_group: PolynomialGroupLayout::new(39, 1),
    precommitteds: vec![trusted_profile],
}
```

The precommitted honest-fold policy list passed to the planner is exactly:

```text
vec![JoltDense::root_honest_fold_policy()]
```

The final planner/config policy is `JoltOneHotK256`'s policy. The final trace
commit builds a nonempty `PrecommittedGroupProfiles` from the validated
trusted profile and calls:

```text
GroupContext::scheduler_with_precommitted_groups(&precommitted_profiles)
```

### Setup capacity

The main/final prover setup for this workload must cover the whole root:

```text
max_num_vars          = max(20, 39) = 39
max_total_batch_polys = 1 + 1 = 2
one_hot_k             = 256
```

Akita's public setup API currently calls its second parameter
`max_num_polys_per_commitment_group`, while grouped-key capacity validation
uses the **total polynomial count across all groups**. The Jolt adapter must
represent these as two distinct concepts:

```text
max_polys_in_one_group = 1
max_total_batch_polys  = 2
```

The second value is passed to the upstream setup-capacity API. The first is
used for Jolt group-local commitment validation. Setup capacity `2` must not
make a two-polynomial advice or main commitment acceptable. Do not rename or
reinterpret the generic group-local metadata method as total capacity; add an
explicit grouped-capacity field or method.

One verifier preprocessing object with the fixed 8 MiB trusted capacity must
support both:

- no trusted input: scalar final key `(39, 1)`; and
- trusted input present: grouped key `[(20, 1) dense] -> (39, 1) one-hot`.

The capacity hook retains the existing all-smaller-scalar scan and examines
every grouped catalog entry; matrix requirements are non-monotone. For each
grouped entry it first accounts for every individually fitting prefix group's
commit-only A/B footprint. It then accounts for the complete grouped effective
schedule only when
`AkitaScheduleLookupKey::fits_setup_capacity(max_num_vars,
max_total_batch_polys)` is true. Prefix accounting must occur before that
whole-key filter: an independently committable prefix can contribute setup
requirements even when the complete grouped key exceeds this setup's total
capacity. The final capacity is the component-wise maximum of all required
scalar, precommit, and complete grouped footprints. It must not assume the
scalar `(39, 2)` footprint dominates the grouped row.

The independently derived trusted setup `(20, 1)` remains available for
creating and validating the precommit, but it is no longer passed to a
separate trusted `verify_batch` call. The final/main verifier setup verifies
the complete grouped proof.

The standalone dense setup and final K256 setup must belong to the same
canonical Akita setup family: the same protocol/setup version, generator seed
or domain, field, and matrix-generation rules. Their extents need not be
equal. Instead, the final setup must cover a matrix prefix compatible with the
dense commitment's exact profile. Define a stable `SetupFamilyId` (or an
equivalent canonical descriptor digest) and separate extent metadata. Do not
treat profile equality as proof of setup identity: a profile describes
schedule parameters, not the concrete CRS matrices. The trusted artifact
records its family and extent, final preprocessing records its family and
extent, and Stage 0 requires family equality plus prefix/footprint coverage.
The verifier binds the canonical family identity and final extent in the new
protocol preamble. The current fixed Akita setup seed can supply this identity;
the implementation need not hash all matrices at runtime.

### Catalog generation

`crates/jolt-akita/src/schedules/mod.rs` already has the
`EmitSpec::group_batch_keys` and `regen_group_batch` machinery. Phase 1 adds
the single grouped key above to the K=256 family and regenerates the checked-in
table and catalog identity with:

```text
cargo run --release -p jolt-akita --bin gen_jolt_schedules -- \
  crates/jolt-akita/src/schedules k256
```

Runtime commit/profile resolution uses the checked-in dense scalar `(20, 1)`
row. Catalog generation must not construct the new K256 key by reading the old
dense catalog, because a full regeneration after planner-policy changes could
then embed a stale prefix. Add a Jolt equivalent of Akita's
`planned_profile_without_precommitted_groups::<JoltDense>` helper: run the
standalone dense planner directly, validate that scalar schedule, and derive
the prefix `CommittedGroupProfile` from its final-group parameters. If needed,
make `family_specs` fallible so profile construction does not unwrap.

The newly generated dense scalar row is the producer/equality oracle for that
planned prefix. Tests require every grouped prefix profile to have an exact
standalone generated producer, and the K256 grouped row must reproduce the
dense `(20, 1)` profile byte-for-byte.

The verifier resolves only the checked-in grouped row. Planner DP is an
offline generation tool, never a verification fallback. If the planner emits
a recursive setup-contribution schedule that the current shape guard cannot
validate, generation fails until that guard/setup path is deliberately
supported; it must not bypass the guard.

## Detailed Design

### 1. Preserve the precommit profile in prover state

Upstream Akita's prover-side `CommittedGroup` contains both the frozen profile
and commitment payload. Jolt's public `AkitaCommitment` wrapper deliberately
stores the payload plus Jolt shape/flavor metadata and lets the verifier
reconstruct the authoritative profile from a checked-in row. That remains a
sound and smaller public boundary for grouped verification as long as the
profile is never reconstructed from arity alone.

The new trusted prover artifact must retain the actual upstream
`CommittedGroup`, including its exact profile. Before contextual main commit,
the prover resolves the approved standalone `JoltDense (20, 1)` row, compares
the actual profile for full equality, and passes that validated actual profile
to `PrecommittedGroupProfiles`.

The verifier does not receive or trust a proof-supplied profile. It derives:

- the trusted profile from the approved standalone `JoltDense (20, 1)` row;
- the final profile from the approved grouped row when trusted advice is
  present; and
- the final profile from the approved scalar row when trusted advice is
  absent.

It then reconstructs each backend `CommittedGroup` from that authoritative
profile and the checked public payload. Required checks are:

- the prover artifact's trusted profile equals the standalone profile;
- the prover hint's final profile equals the grouped/scalar final profile;
- flavor, one-hot K, arity, group-local polynomial count, layout digest, and
  commitment payload length agree with the expected profile and semantic
  role; and
- every backend hint's committed payload equals its public wrapper
  commitment.

The public trusted commitment encoding need not change merely to duplicate
the catalog-derived profile. If an implementation elects to carry a profile
or profile digest as redundant metadata, the verifier must equality-check it
against the catalog-derived value before using it. It must never use that
field as the source of schedule authority.

### 2. Trusted prover artifact

The public verifier input remains the trusted commitment only. The prover,
however, needs the trusted polynomial, backend commitment, opening hint, and
frozen profile for the joint proof.

Introduce an opaque prover-only `TrustedAdvicePrecommit<PCS>` (name
illustrative) containing:

- the public `PCS::Output` commitment;
- the canonical dense advice polynomial/source;
- its `PrefixPackedObjectPlan` and word arity;
- the Akita opening hint/backend committed group, which retains the exact
  frozen standalone profile; and
- the canonical Akita setup-family identity and the standalone setup extent
  used to create the commitment.

The trusted commit API returns this artifact. The modular and legacy packed
prover signature is exactly one optional artifact, not a loose
commitment/hint pair:

```text
prove(..., trusted_precommit: Option<&TrustedAdvicePrecommit<PCS>>, ...)
```

Stage 0 obtains the prover-side public commitment from that artifact. The
caller supplies the same extracted commitment as the verifier's independently
anchored trusted input. If an API layer also wants the prover to check a
separately supplied expected commitment, that is a second explicitly named
argument rather than hidden state.

Because the modular prover remains generic over `PCS`, the grouped opening
seam supplies an opaque associated profile/context accessor. Generic Jolt code
must not name `CommittedGroupProfile`; the Akita implementation extracts and
validates it from `PCS::OpeningHint`. Making the packed prover concrete over
`AkitaScheme` is an alternative, but mixing concrete Akita types into
`TrustedAdvicePrecommit<PCS>` is not.

This removes the modular prover's current prove-time trusted recommit, which
exists only to recover an opening hint. A precommitted group must be committed
once. Recomputing the canonical words for witness/claim consistency is
permitted; executing a second backend commitment is not.

The artifact is required prover state, not something recoverable from the
public commitment. Akita emits the opening hint only with the original commit
output. The in-process API therefore returns and retains the artifact until
all dependent proofs are complete. If an application needs a cross-process or
long-lived precommit, it must persist/transfer a versioned private artifact
containing the source, hint/backend group, plan, profile, and setup-family
metadata, with bounded decoding and the same validation as a freshly produced
artifact. This protocol does not define a hint-from-commitment reconstruction
operation. Losing the artifact requires creating and externally registering a
new precommit; Stage 0 never recommits merely to recover a hint.

Phase 1 does not add an Akita SDK surface. No Akita backend profile or hint
type is exposed through the existing guest SDK; a future SDK integration uses
the same opaque prover artifact and verifier-only public commitment.

### 3. Context-aware final trace commitment

Extend `TraceOneHotCommitment` or add an Akita-specific final-group commitment
trait with two explicit modes:

- `NoPrecommittedGroups`; and
- `WithPrecommittedGroups(ordered validated profiles)`.

For a present Phase-1 trusted artifact, Stage 0 validates the profile and
commits `TracePackedOneHot` under the grouped context. For absent trusted
advice it uses the scalar context.

The enforceable invariant is that the main hint/backend group has the exact
final profile resolved from the approved grouped row. `GroupContext` itself is
not serialized in an Akita commitment. The honest prover must use the grouped
context, and a scalar commitment whose profile differs from the grouped final
profile fails. If scalar and grouped scheduling ever produce byte-identical
final profiles, the resulting commitments are intentionally interchangeable:
there is no cryptographic provenance bit distinguishing which API call made
them. If product policy later requires path provenance even in that case, it
must add and bind an explicit Jolt context tag rather than infer provenance
from the payload.

Commit execution order is trusted first, main later. Jolt's outer commitment
absorption order may remain its existing versioned order
`[main, untrusted, trusted, program...]`; execution order and transcript order
need not be identical. The grouped Stage-8 statement separately binds Akita's
required `[trusted, main]` order.

### 4. Multi-group opening adapter

Do not overload the existing single-commitment semantics of
`CommitmentScheme::{open_batch, verify_batch}`. Add a true Akita grouped
batching seam, either as a second `BatchOpeningScheme` marker or a narrowly
scoped group-batching extension trait.

Its public statement is an ordered nonempty vector of groups. Every group
contains:

- a semantic role tag;
- one commitment;
- one canonical Jolt-order opening point;
- a nonempty ordered evaluation vector; and
- the expected flavor/layout metadata.

Its prover input contains one matching opening hint/source bundle per group.
Different groups may have different points and arities. Within a group, every
member must share the point. Phase 1 additionally imposes representation
homogeneity as a Jolt policy; upstream Akita can support mixed-representation
groups, but Jolt does not admit them in this protocol. Validation checks:

- group count and exact role order;
- one final group, always last;
- per-group public commitment equality with the hint's backend committed
  group, plus structural role/shape alignment of the source;
- point arity against that group, not the maximum arity;
- evaluation count equals that commitment's polynomial count;
- group-local representation homogeneity;
- maximum arity against setup capacity;
- sum of all group polynomial counts against total setup capacity; and
- exact batch profile resolves to one approved grouped row.

The adapter does not recommit or hash an entire source to pre-check that it
matches the hint. Source-to-commitment/evaluation consistency is enforced by
the PCS proof; a wrong but structurally valid source reaches proving failure
or produces a proof that verification rejects.

Phase-1 Jolt construction is deliberately typed to either `[main]` or
`[trusted, main]`; callers cannot provide arbitrary semantic roles. The
adapter internals may use a vector because upstream Akita is naturally
multi-group and Phase 2 will add another prefix.

For the two-group Phase-1 case, the adapter builds one
`OpeningClaims::from_groups`, one ordered hint list, and one ordered
polynomial-group list, then calls
`SelectedProverOpeningData::from_committed_claims::<JoltOneHotK256>` and one
`AkitaCommitmentScheme::<JoltOneHotK256>::batched_prove` with
`BasisMode::Lagrange`. A no-trusted, one-group proof continues to dispatch to
the main commitment's existing K16 or K256 configuration; it uses the same
versioned proof carrier but does not pretend that every scalar main row is
K256.

Verification builds the same ordered `OpeningClaims`, resolves the same public
selection, creates one `GroupBatchStatement`, and calls one
`batched_verify` with `BasisMode::Lagrange`.

The joint prove receives exactly one prover setup and one prove stack: the
final `JoltOneHotK256` grouped-capacity setup/stack. The independent
`JoltDense` setup/stack is used only when producing the trusted commitment; it
is not supplied again to joint prove. Joint verification likewise uses only
the final K256 verifier setup. Basis mode is part of the backend instance and
transcript descriptor, so prover and verifier must not rely on an implicit
default.

### 5. Preserve heterogeneous source performance

Upstream Akita's stock heterogeneous polynomial wrapper supports ordinary
`DensePoly` and `OneHotPoly`. Jolt's main commitment uses the custom streamed
`TracePackedOneHot`, which is not a stock wrapper variant. Upstream grouped
prover data also requires one concrete root-source type across all groups.

Add a Jolt-owned heterogeneous root-source dispatch that supports at least:

- borrowed dense advice source; and
- borrowed/cheaply cloned `TracePackedOneHot` source.

It must implement/delegate the Akita root metadata, opening, tensor, and CPU
kernel capabilities needed by grouped proving. Group-local fast paths remain
homogeneous.

The implementation must not:

- materialize the `2^39` main physical polynomial;
- expand `TracePackedOneHot` into a generic padded `OneHotPoly` index vector;
- deep-clone the dense advice coefficients merely to erase their type; or
- regress the trace commitment's current streaming row ownership.

An upstream Akita API extension accepting heterogeneous prepared group
carriers would also solve this, but is not required for Phase 1. If used, it
must land and be pinned before Jolt consumes it.

### 6. Per-group point handling

Jolt semantic and prefix-reduced points remain in canonical high-to-low order.
The outer Jolt grouped statement binds those canonical points.

At the backend boundary only:

- dense trusted-advice point: unchanged;
- one-hot `OneHotTrace` point: reverse once using the existing adapter
  convention.

Conversion is performed independently per group on both prover and verifier.
There is no shared batch point and no global reversal. Tests compare direct
dense/one-hot MLE evaluations at each logical point with the claims passed to
the backend.

### 7. Stage-8 claim and transcript order

The PIOP and final leaf claims are unchanged. Stage 8 obtains:

- the main physical claim from `ONE_HOT_TRACE_LAYOUT.reduce_claims`; and
- the trusted physical claim from the existing dense advice
  `PrefixPackedObjectPlan.reduce_claims`.

For the new protocol version, the Fiat-Shamir-visible Stage-8 order is fixed:

1. derive and prefix-reduce the main `OneHotTrace` claim;
2. derive and prefix-reduce the untrusted dense claim, if present, but retain
   it for its later auxiliary proof;
3. derive and prefix-reduce the trusted dense claim, if present;
4. bind and prove the Akita group statement in public order
   `[TrustedAdvice, OneHotTrace]`, or `[OneHotTrace]` when absent;
5. prove the already reduced untrusted claim as the first auxiliary object, if
   present; and
6. process committed-program objects in their existing canonical order.

This preserves Jolt's existing main/untrusted/trusted prefix-challenge order.
The group order deliberately differs from the claim-reduction order. The
implementation must use named roles rather than positional inference when it
reorders the completed physical claims for Akita.

The existing helper that combines prefix reduction with an immediate PCS open
must be split or bypassed for both advice kinds: trusted is held for
`main_batch`, while untrusted is held until the later auxiliary step. No
standalone trusted proof is emitted.

### 8. Transcript binding and protocol version

Append a new `CommitmentConfig` variant after the existing serialized variants,
for example `PackedDenseAdviceBatched`. Retain `Packed` and
`PackedDenseAdvice` as rejected wire tombstones. Select the new variant only
after this feature is complete.

Use new transcript and encoding constants; do not reinterpret
`PACKED_DENSE_ADVICE_TRANSCRIPT_VERSION` in place. The grouped PCS domain is
also new, for example `jolt-akita/precommitted-group-batch/v1`.

Before bridging once into the Akita transcript, the outer Jolt transcript
binds:

1. protocol and grouped-batch version;
2. final/main verifier setup-family identity and extent, including maximum
   arity, total batch polynomial capacity, K, and main layout digest;
3. the explicit public `OpeningScheduleSelection` returned by the prover's
   selected opening data (its `row_digest`) and, separately, the fixed
   configured schedule-catalog `identity_digest`;
4. group count;
5. for each group in public Akita order:
   - group index and semantic role tag;
   - precommitted/final role;
   - backend flavor and K;
   - full validated frozen profile or its canonical digest;
   - Jolt layout digest, arity, and polynomial count;
   - full public commitment;
   - canonical Jolt-order point;
   - evaluation count and ordered values; and
6. one bridge challenge.

The backend proof is appended to Jolt's transcript once. A role tag is
required even when trusted and untrusted advice happen to have identical
shapes and values; shape alone cannot prevent semantic substitution.

The explicit selection is required public statement data, not an optional
optimization: upstream `GroupBatchStatement::new` requires it and the backend
proof does not contain it. Store it in the versioned Jolt Akita proof carrier
(for example, in `AkitaBatchProof`). The prover takes it from
`SelectedProverOpeningData::selection()`.

`OpeningScheduleSelection` identifies a row by `row_digest`; it does not carry
or “name” the generated catalog identity. The catalog `identity_digest` comes
from the fixed prover/verifier configuration and is absorbed separately. A
proof-carried catalog identity, if redundantly encoded, is never authoritative
and must equal that configured constant.

When trusted advice is present, the verifier calls
`JoltOneHotK256::resolve_schedule_selection` under the fixed grouped K256
catalog. When trusted advice is absent, it retains the validated main setup's
existing K16/K256 dispatch and resolves the scalar selection under that
configuration's catalog. It does not call the honest-prover
`select_schedule_for_profiles`, reconstruct a runtime lookup key to select a
row, or invoke planner search. After identity-only resolution it compares the
resolved row's exact prefix/final profiles and layouts with the
protocol-expected trusted/main statement. The row selection, identity of the
catalog actually selected by this presence/K dispatch, and expected group
descriptors are all bound before the bridge.

### 9. Batch-aware pre-deserialization guard

`crates/jolt-akita/src/shape_guard.rs` currently assumes a scalar key, one
commitment, final profile only, and compression-plan group index zero. Replace
that path for grouped proofs with two validation layers. First, replace
unbounded derived decoding of prover-controlled outer vectors with bounded
decoders/custom visitors. The Jolt proof visitor caps
`AkitaJointOpeningProof::auxiliary` element count before allocating its
wrappers, then checks the exact public expected count. Each Akita proof visitor
checks protocol absolute maxima for group count, selection bytes, commitment
payload bytes, proof-shape bytes, and proof body bytes before allocating those
vectors. Second, after that bounded outer decode and before any Akita backend
deserialization or shape-driven allocation, the grouped guard:

1. validates the bounded outer group count and Phase-1 roles;
2. reads the bounded explicit `OpeningScheduleSelection` and resolves its row
   digest under the presence/K-selected catalog: fixed grouped K256 when
   trusted is present, otherwise the validated scalar main K16/K256 catalog;
3. derives the expected standalone trusted profile and expected final layout
   from public protocol inputs;
4. requires the resolved row to equal the exact grouped descriptor containing
   that prefix and final layout (or the exact scalar descriptor when trusted
   advice is absent);
5. validates each wrapper's role/flavor/layout metadata against the
   corresponding resolved prefix/final profile and uses only that resolved
   profile to interpret the payload;
6. derives the full multi-group opening layout using each group's own arity
   and claim count;
7. validates each commitment coefficient count and byte length using its
   group-specific compression plan/profile;
8. deserializes each payload and reconstructs all backend `CommittedGroup`s;
9. validates the already bounded-decoded proof-shape blob against the complete
   grouped schedule; and
10. only then deserializes the backend proof body and permits backend
    shape-driven allocations.

Proof-shape byte caps, sumcheck-round caps, response-shape checks, and
direct-only setup-contribution restrictions remain in force. Every arithmetic
combination of group count, polynomial count, coefficient count, or byte count
uses checked arithmetic.

### 10. Proof model

Rename the first field of the packed joint-opening carrier to reflect its new
meaning:

```text
AkitaJointOpeningProof {
    main_batch: P,
    auxiliary: Vec<P>,
}
```

Every contained versioned `AkitaBatchProof` carries the explicit
`OpeningScheduleSelection` used to construct its public backend statement, in
addition to the bridge, bounded proof shape, and proof body. Thus
`main_batch` carries the grouped/scalar final selection and every retained
auxiliary `P` carries its own scalar selection; using the same generic `P` is
unambiguous. Each object-specific transcript domain binds its selection and
the configured catalog identity before its bridge. The verifier resolves and
validates that selection under the semantic role's already validated backend
configuration before deserializing the latter two fields: grouped K256 for a
present-trusted `main_batch`, K16/K256 main config for scalar `main_batch`, and
the corresponding Dense/OneHot config for each auxiliary. Cross-catalog row
resolution and fallback are forbidden. An alternative implementation may use
distinct main/auxiliary proof wrapper types, but it may not leave auxiliary
selections implicit or verifier-chosen.

`main_batch` means:

- one-group main proof when trusted advice is absent; or
- two-group trusted-plus-main proof when trusted advice is present.

In Phase 1, `auxiliary` order is:

1. untrusted advice, if present;
2. committed-program bytecode object(s); and
3. committed program-image object, if present.

Trusted advice never appears in `auxiliary`. The protocol discriminator, not
the field name alone, distinguishes old proof bytes from new semantics.

### 11. Presence and supported-shape behavior

Presence is determined by Jolt's scheduled input, not by all-zero values.

- Present trusted advice requires the trusted prover artifact, fixed `(20, 1)`
  profile, and the grouped final commitment. The prover extracts the public
  commitment from the artifact; the verifier receives that commitment as its
  independently anchored input.
- A missing/malformed artifact fails before main commitment/proving, and a
  verifier input different from the artifact's extracted public commitment
  fails verification.
- Absent trusted advice uses the existing scalar final commit/opening even if
  preprocessing was provisioned to support the grouped row.
- An unused capacity-derived trusted setup/profile may remain in
  preprocessing when advice is absent.
- Present trusted advice with a capacity other than 8 MiB or a final group
  other than K256 `(39, 1)` fails early with an unsupported-configuration
  error in Phase 1.
- No silent separate-proof fallback is permitted for an unsupported trusted
  shape, because that would make the proof count depend on an undocumented
  scheduler miss.
- Untrusted presence retains its existing separate auxiliary behavior.

### 12. Unchanged advice semantics

Batching changes only PCS scheduling and final-opening discharge. The
following remain exactly as in the dense-advice protocol:

- canonical zero-padded little-endian `u64` advice coefficients;
- logical and physical dense advice layouts;
- Stage-4 advice contribution to `RamValCheck`;
- `AdviceClaimReductionCyclePhase`;
- `AdviceClaimReduction`;
- prefix-floor claim reduction for advice arities below the dense floor;
- arbitrary-field verifier language for untrusted dense advice; and
- trusted committer attestation assumptions.

No advice reconstruction, range, booleanity, or one-hot sumcheck is added or
removed by this feature.

## Component Changes

| Component | Required change |
|---|---|
| `crates/jolt-akita/src/schedules/mod.rs` | Add the fixed K256 grouped key and dense prefix honest-fold policy; keep scalar rows. |
| `crates/jolt-akita/src/schedules/jolt_fp128_onehot_k256.rs` | Regenerate with the grouped row and new catalog identity. |
| `crates/jolt-akita/src/bin/gen_jolt_schedules.rs` | Propagate a fallible family-spec/profile-generation path if the planner-derived prefix helper requires it. |
| `crates/jolt-akita/src/lib.rs` | Export the grouped adapter, statement/prover carriers, context handle, production scheme alias, and feature-gated fixture scheme. |
| `crates/jolt-akita/src/configs.rs` | Include approved grouped rows in setup-matrix capacity computation; distinguish total batch-polynomial capacity from group-local count; expose the canonical setup-family identity and extent. |
| `crates/jolt-akita/src/adapters.rs` | Add grouped statement/prover carriers, bounded outer decoding, grouped setup-family/extent and statement binding, and separate group-local/total-batch capacities. The public commitment may remain payload-plus-catalog-derived metadata. |
| `crates/jolt-akita/src/scheme.rs` | Add context-aware final trace commit using ordered precommit profiles; leave dense trusted commit independent. |
| `crates/jolt-akita/src/native_batching.rs` | Keep the scalar adapter or factor shared code, then add true multi-group prove/verify with independent points, hints, and representations. |
| `crates/jolt-akita/src/trace_onehot.rs` or a new adapter module | Add heterogeneous source/view/kernel dispatch for dense plus streamed trace without materialization or deep clones. |
| `crates/jolt-akita/src/shape_guard.rs` | After bounded outer decoding, resolve full grouped rows and validate every exact commitment/proof shape before backend deserialization. |
| `crates/jolt-openings/src/schemes.rs` | If needed, add an Akita-capable grouped extension seam and an explicit total-batch capacity method; preserve the existing group-local metadata and homomorphic batching semantics. |
| `crates/jolt-openings/src/lib.rs` | Re-export any new grouped extension trait and public capacity/context carrier. |
| `crates/jolt-verifier/src/config.rs` | Append/select the new packed-batched commitment variant and new transcript/encoding versions. |
| `crates/jolt-verifier/src/proof.rs` | Change the first packed proof field from main-only to `main_batch`, add a bounded outer visitor for auxiliary count, and preserve old protocol decoding as rejected/versioned. |
| `crates/jolt-verifier/src/preprocessing.rs` | Treat `pcs_setup` as the final/grouped setup; retain or replace trusted standalone setup metadata for precommit validation, not separate verification. |
| `crates/jolt-verifier/src/verifier.rs` | Absorb the new protocol preamble and audit the versioned outer commitment order. |
| `crates/jolt-verifier/src/stages/stage8/verify.rs` | Thread the grouped-capable packed verifier bounds and inputs into Stage 8. |
| `crates/jolt-verifier/src/stages/stage8/mod.rs` | Update Stage-8 setup metadata/bounds shared by packed verification. |
| `crates/jolt-prover/src/akita/stage0.rs` | Validate the trusted artifact/profile, select scalar vs grouped final context, and require total setup capacity two for the fixed grouped case. |
| `crates/jolt-prover/src/akita/stage8.rs` | Construct `[trusted, main]`, call one grouped prove, and remove trusted from auxiliary openings. |
| `crates/jolt-prover/src/akita/prover.rs` and public packed prover seam | Accept the opaque trusted precommit artifact and remove prove-time recommit. |
| `crates/jolt-prover/src/akita/mod.rs` | Change the public packed prover signature and document artifact ownership and verifier commitment extraction. |
| `crates/jolt-prover/src/akita/witness.rs` | Construct or convert the dense advice object into the retained trusted precommit artifact. |
| `crates/jolt-prover/src/preprocessing.rs` | Own the final grouped-capacity setup separately from any standalone dense precommit setup. |
| `crates/jolt-verifier/src/stages/stage8/packed.rs` | Construct the same groups, call one grouped verify, and leave only untrusted/program auxiliary verification. |
| `crates/jolt-prover-legacy/src/zkvm/packed.rs` | Mirror setup, final commit context, Stage-8 grouping, proof layout, and artifact flow through the shared adapter. |
| `crates/jolt-profiling/src/taxonomy.rs` | Replace obsolete standalone trusted-open/rebuild spans with stable precommit, contextual-main-commit, joint-prove, and joint-verify spans. |
| `crates/jolt-prover/examples/modular_benchmark.rs` | Build the grouped-capacity main setup, retain/pass the full trusted artifact, and report joint proof/verification honestly. |
| `crates/jolt-akita/tests/{native_batching,pathologies,schedules}.rs` | Cover heterogeneous grouping, guarded malformed inputs, generated rows, and setup capacity. |
| `crates/jolt-akita/tests/e2e.rs` | Update proof/setup carrier assumptions and exercise fixture-scheme dispatch where applicable. |
| `crates/jolt-prover/tests/{akita_e2e,akita_byte_diff}.rs` | Cover modular/legacy acceptance and byte parity through the injected small grouped configuration. |
| `crates/jolt-verifier/tests/fs_inventory/*`, `tests/fs_attacks.rs`, `tests/fs_obligations.rs`, and soundness tests | Update Fiat-Shamir inventories/obligations and add role/selection/group tampering cases. |
| `crates/jolt-verifier/tests/support/akita_fixtures.rs` | Construct fixture-scheme verifier data with the exact feature-gated row identities. |
| Akita-related crate `Cargo.toml` feature maps | Add `akita-test-schedules` as an explicitly nonproduction fixture feature and thread it through `prover-fixtures`; never enable it from `akita` itself. |
| `benchmark-runs/results/sha2_chain_akita_trusted_advice_2pow20_words_nv26_20280813.md` | Record the post-batching benchmark after acceptance passes. |

`common`, `jolt-witness`, the guest/tracer interfaces, RAM layout, and all
non-Akita code require no semantic changes.

## End-to-End Flow

### Preprocessing / trusted precommit

1. Validate the public trusted capacity and derive the canonical dense plan.
2. For Phase 1, require `word_vars = physical_vars = 20` when trusted advice
   may be present.
3. Resolve the standalone `JoltDense (20, 1)` row.
4. Build the independent dense setup.
5. Commit trusted advice with no precommitted groups.
6. Retain the complete prover artifact and expose only its public commitment
   to the verifier/caller.
7. Check the returned frozen profile against the standalone catalog profile.
8. Provision the final K256 setup for arity 39 and total batch count 2, including
   both the scalar and grouped row footprints.

### Prover Stage 0

1. Validate trusted input presence against the single prover artifact and
   extract its public commitment. The verifier caller separately anchors that
   same extracted commitment.
2. Validate the artifact's kind, layout digest, capacity, profile, payload,
   setup-family identity, and setup extent.
3. When trusted is present, build `PrecommittedGroupProfiles` from its exact
   profile; otherwise choose the empty/scalar context.
4. Assemble streamed `OneHotTrace` rows.
5. Commit the final group using the selected context.
6. Check the final profile is the expected grouped/scalar catalog profile.
7. Commit untrusted advice separately as today.
8. Absorb all outer Jolt commitments in the versioned canonical order.

### Prover Stage 8

1. Resolve and prefix-reduce the main claim.
2. Resolve and prefix-reduce the untrusted claim when present, retaining it
   without opening yet.
3. Resolve and prefix-reduce the trusted claim when present.
4. Create `[TrustedAdvice, OneHotTrace]` group claims and matching hint/source
   groups, or main-only when absent.
5. Convert each point to backend order according to that group's flavor.
6. Obtain the explicit row selection from `SelectedProverOpeningData`.
7. Bind the selection/grouped statement and bridge the transcript once.
8. Execute one final-config `batched_prove` in Lagrange basis and append one
   proof (`JoltOneHotK256` is mandatory for the two-group Phase-1 row).
9. Open the retained untrusted claim and committed-program objects through
   their remaining auxiliary paths.

### Verifier Stage 8

1. Recompute main, untrusted, and trusted physical claims, as present, in that
   fixed prefix-challenge order.
2. Read the explicit selection, resolve it by catalog identity only, and check
   the resolved scalar/grouped row against expected presence and roles.
3. Run the multi-group shape guard before deserializing backend payloads.
4. Build `[TrustedAdvice, OneHotTrace]` or main-only in public group order.
5. Apply per-group point conversion.
6. Bind the identical outer statement and bridge once.
7. Execute one Lagrange-basis final-config `batched_verify` for `main_batch`
   using only the final/main verifier setup (`JoltOneHotK256` for the
   two-group Phase-1 row).
8. Verify only remaining untrusted/program auxiliary proofs.
9. Reject any missing, extra, or reordered proof/group.

## Security and Soundness Requirements

### External trusted-commitment anchor

The trusted commitment is not proof-selected. It remains an external verifier
input associated with the trusted data publisher. Both the Jolt outer
transcript and the grouped Akita statement absorb it. Substituting a different
trusted commitment changes the statement and must fail unless the caller
explicitly supplies that different commitment.

### Profile and schedule binding

The schedule-authoritative profile is part of commitment interpretation. The
final commitment must be bound to the exact trusted profile used by the joint
proof. The prover requires:

```text
hint trusted profile == approved standalone profile
hint final profile   == approved grouped-row final profile
```

The verifier derives those approved profiles from the scalar/grouped catalog
rows, validates wrapper metadata and payload lengths against them, reconstructs
the backend groups with them, and verifies the proof under that resolved
schedule. It cannot compare an untransmitted “actual profile,” and it must not
accept a prover-selected replacement. No prover-side comparison or
verifier-side derivation may be weakened to `(num_vars, poly_count, flavor)`.

### Role and order binding

Phase-1 group order is exactly `[TrustedAdvice, OneHotTrace]`. Semantic role
tags and kind-separated Jolt layout digests prevent a same-shape commitment
from being substituted into another role. Duplicated, omitted, swapped, or
unknown roles are rejected before the backend verifier.

### Claim binding

Each group binds its own point and ordered values. Akita may batch different
points across groups; it does not allow different points within one
commitment group. The trusted value remains the output of the existing advice
claim reduction, and the main value remains the prefix reduction of the
existing final trace claims.

### Setup capacity

The verifier setup must cover the selected grouped schedule's matrices and
total polynomial count. Setup success for a scalar row is not evidence that a
grouped row fits. A dedicated test materializes or checks every required setup
prefix slot and matrix footprint for the exact selected grouped row.

The trusted and final setups must also share the canonical setup-family
identity, and the final setup's matrices must be prefix-compatible with the
standalone trusted commitment for its resolved profile. A matching profile
under a different seed/domain is not compatible. The prover checks artifact
metadata early; the joint proof remains the cryptographic check that the
public payload opens under the verifier's canonical setup.

### Deserialization safety

Every prover-controlled outer length is checked against a protocol hard cap
before allocating its corresponding byte vector. After bounded outer decoding,
every shape is checked against the trusted resolved schedule before backend
deserialization or schedule-driven allocation. This includes both commitment
payloads, not only the final group. Correct-length coefficient/value tampering
is not expected to fail at this layer; it is rejected during payload parsing or
PCS verification. Existing caps remain defense in depth and do not replace
exact validation.

### Soundness accounting

This feature delegates to Akita's existing heterogeneous multi-group protocol
and does not add a Jolt-side random-linear combination of the two PCS proofs.
The soundness statement is therefore one Akita group-batch statement over two
independently committed polynomials. Existing dense-advice prefix-floor error
and arbitrary-field advice language are unchanged; the feature adds no new
advice range claim. In particular, it inherits the formal statement in
`specs/akita-dense-advice.md`: a floor-padded advice object with
`d = physical_vars - logical_vars > 0` contributes at most `d / |F|`
additional prefix-reduction error, union-bounded across present objects. The
fixed trusted `(20, 1)` object has `d = 0`.

## Failure Behavior

The prover returns an early unsupported/invariant error, and the verifier
returns verification failure, for any of the following:

- `2^32` bytes supplied under the `(20, 1)` protocol profile;
- present trusted advice with any capacity other than 8 MiB in Phase 1;
- present trusted advice with a final group other than K256 `(39, 1)`;
- absent/malformed trusted prover artifact or a verifier input that does not
  equal the public commitment extracted from that artifact;
- trusted commitment created under a non-dense or wrong standalone profile;
- final commitment whose actual final profile does not equal the required
  grouped-row final profile;
- grouped schedule missing from the checked-in catalog;
- planner/catalog/profile drift;
- trusted/final setup-family mismatch or insufficient compatible setup extent;
- insufficient maximum arity, total polynomial capacity, or matrix footprint;
- wrong group count, role, order, flavor, K, arity, digest, or claim count;
- mismatched public commitment and hint/backend group, or a structurally
  mismatched source;
- wrong per-group point arity or point-order conversion;
- altered evaluation, point, commitment, or profile;
- forged commitment coefficient/byte length;
- forged or oversized proof shape/body;
- extra or missing `main_batch`/auxiliary proofs; or
- an old packed-dense outer proof, or a scalar-context main commitment whose
  final profile is incompatible with the new grouped protocol.

There is no fallback to runtime planning, a main commitment under an
incompatible final profile, or a separate trusted proof.

## Compatibility

- Dory clear and Dory ZK proof bytes remain unchanged.
- `akita + zk` remains rejected.
- Old `Packed` and `PackedDenseAdvice` outer proofs are not accepted under the
  new config, and an old single-group opening proof cannot substitute for
  `main_batch`.
- Enum discriminants are appended, never reused.
- Positional codecs retain tombstoned payload representations or receive an
  explicit outer version; a field rename alone is not considered versioning.
- A standalone trusted commitment generated under the identical approved
  dense `(20, 1)` row and canonical setup family remains a valid public input
  only when its matching original prover artifact/hint is retained. A public
  commitment alone cannot recreate that state, and this does not require
  changing the public commitment encoding.
- A prior scalar-context main commitment is reusable only if its retained
  backend profile is exactly the grouped row's final profile and it has a
  matching hint/source under the canonical setup family. Normally the grouped
  scheduler selects a different final profile, in which case it must be
  recomputed. Verification enforces exact profile/proof compatibility, not
  unobservable API-call provenance.
- No-advice Akita proofs use the new outer protocol version but the existing
  scalar backend row.
- Untrusted-only and committed-program paths retain their Phase-1 auxiliary
  PCS behavior under the new outer transcript version.

## Testing Plan

### Schedule and setup tests

- The standalone dense `(20, 1)` profile resolves and validates.
- The exact grouped key `[(20, 1) dense] -> (39, 1) K256` resolves.
- Its prefix profile exactly equals the standalone dense profile.
- No reordered, changed, or extra prefix key resolves.
- The grouped row survives generated-table drift regeneration.
- Setup capacity `(39, total=2)` covers scalar and grouped footprints.
- Capacity `(39, total=1)` is rejected for the grouped row.
- A dense precommit and final setup from the canonical setup family are
  compatible; changing the setup seed/domain is rejected even with identical
  schedule metadata.
- The checked-in schedule satisfies the shape guard's supported direct-only
  constraints.

### `jolt-akita` adapter tests

- Using a small test-only grouped catalog/config, independently commit a dense
  prefix, contextually commit streamed `TracePackedOneHot`, then produce one
  proof and one verification at distinct points. The same test must exercise
  the production source types and dispatch, not substitute an ordinary
  `OneHotPoly` for the streamed trace.
- Separately resolve and inspect the production `(20, 1) -> (39, 1)` row
  without allocating either production witness.
- Count backend invocations and assert one `batched_prove` and one
  `batched_verify` for the two groups.
- Preserve the streaming trace source; add an allocation/regression assertion
  that no `2^39` table or generic padded one-hot index vector is built.
- Verify main-only through the scalar row using the same grouped-capacity setup.
- Tamper every role, order, profile, commitment, point, value, hint, K, digest,
  group count, selection, proof shape, and payload length.
- Assert oversized outer vectors fail during bounded outer decoding and exact
  shape/length mismatches fail before backend deserialization or
  schedule-driven allocation. Correct-length content tampering may proceed to
  parsing/verification but must still reject.
- In particular, encode an auxiliary-count length prefix above the protocol
  cap and assert rejection occurs before allocating any `P` wrappers.

### Jolt end-to-end tests

- Trusted-only modular proof accepts and uses one PCS proof/verification under
  a test-only grouped row with the same role/profile protocol.
- Trusted-only legacy proof accepts under that row and is byte-identical to
  modular.
- No-advice proof accepts through the scalar final row.
- Untrusted-only proof has main plus one untrusted auxiliary proof.
- Trusted+untrusted proof has one trusted/main joint proof plus one untrusted
  auxiliary proof.
- Committed-program proofs retain their auxiliary program proofs in canonical
  order.
- Trusted commitment substitution, omission, and artifact mismatch reject.
- A scalar-context main commitment with a different final profile rejects in
  a trusted statement; an intentionally equal-profile case documents the
  absence of unobservable context provenance.
- Unsupported trusted or main capacity rejects before the expensive commit.
- Old/new protocol codec and config mismatches reject.
- Fiat-Shamir source/scope/challenge inventories are updated and tamper tests
  cover group-role and schedule-selection binding.

The production `(20,1)->(39,1)` end-to-end case is too large for an ordinary
unit-test matrix. Small synthetic adapter and Jolt end-to-end tests may use a
test-only config/row, but production acceptance must also include the real
benchmark command. A test-only row must not be compiled into or become an
undocumented fallback for the production verifier.

Concretely, add an alternate concrete `AkitaFixtureScheme` behind a
nonproduction Cargo feature such as `akita-test-schedules` (enabled by the
existing `prover-fixtures` test lane). Do not use only `#[cfg(test)]`: Rust
integration tests link the library without that cfg. Parameterize the shared
grouped adapter and the minimum modular/legacy packed entry seams over the
scheme/config provider, while preserving `AkitaScheme` as the fixed production
alias. Both fixture provers and the verifier instantiate the same alternate
scheme; the provider is trusted build configuration, never proof data or a
runtime planner.

The fixture catalog is finite and explicit. Default 4 KiB trusted capacity is
`2^9` words; the dense floor/zero-prefix embedding makes one physical
polynomial with 14 variables and 32 selector slots, so the group layout is
`(14, 1)`, not 32 polynomials. Generate K16 grouped rows with that exact prefix
and final singleton arities `p = 22..=26`, corresponding to the Akita
fixture-supported `log_T = 12..=16` under the canonical
`ONE_HOT_TRACE_LAYOUT` (K16 reserves 64 selector slots, and both modular and
legacy Akita enforce a `2^12` trace floor). Total batch capacity
remains 2. Each fixture asserts its derived final shape and resolves only the
matching checked-in/generated fixture row. These rows and their identities are
absent from `AkitaScheme`'s production catalog. The existing trusted
`akita_e2e` and `akita_byte_diff` cases must run through this seam; an
adapter-only synthetic test is not a substitute for modular/legacy byte
parity.

### Required validation commands after implementation

```text
cargo fmt -q
cargo clippy --all --features host -q --all-targets -- -D warnings
cargo clippy --all --features host,zk -q --all-targets -- -D warnings
cargo clippy --all --features host,akita -q --all-targets -- -D warnings
cargo nextest run -p jolt-akita --features akita-test-schedules --cargo-quiet
cargo nextest run -p jolt-prover --features prover-fixtures,akita --cargo-quiet
cargo nextest run -p jolt-verifier --features prover-fixtures,akita --cargo-quiet
cargo nextest run -p jolt-prover-legacy --features host,akita,akita-test-schedules --cargo-quiet
cargo nextest run -p jolt-prover-legacy muldiv --cargo-quiet --features host
cargo nextest run -p jolt-prover-legacy muldiv --cargo-quiet --features host,zk
```

Run the existing non-Akita modular clear/ZK suites as regression coverage.

## Benchmark and Instrumentation Contract

The target benchmark remains SHA2-chain, scale 26, K256, with 8 MiB trusted
advice. Use the existing result document as the before-batching baseline.

Commit timings remain exactly separable because the commits remain separate:

| Metric | Boundary |
|---|---|
| Trusted advice commit | Independent dense precommit only. |
| Main commit | Stream assembly, contextual final commit, and post-commit residency release. |
| Total commit | Sum of the two non-overlapping commit intervals. |

Prove and verify are intentionally fused and must be reported as such:

| Metric | Boundary |
|---|---|
| Joint trusted+main PCS prove | The single Akita grouped `batched_prove` call. |
| Joint trusted+main PCS verify | The single Akita grouped `batched_verify` call. |
| Whole Jolt prove/verify | Existing end-to-end boundaries. |

Internal preparation spans may report trusted claim reduction, main claim
reduction, statement assembly, serialization, or backend phases, but those are
not additive “advice PCS prove” and “main PCS prove” values. Once the backend
fuses the groups, exact per-group prove/verify times do not exist. A paired
no-advice delta is an empirical comparison containing interaction and run
noise, not an exact advice component.

Record at least:

- separate trusted and main commitment time;
- one joint PCS prove and verify time;
- total Jolt prove and verify time;
- proof bytes, including `main_batch` and remaining auxiliaries;
- peak RSS;
- setup/schedule preparation time and whether it is amortized;
- exact catalog row/selection identity; and
- comparison against the current separate-dense baseline.

Suggested stable spans are:

```text
akita_trusted_advice_precommit
akita_main_commit_with_precommitted
akita_trusted_main_batched_prove
akita_trusted_main_batched_verify
```

Remove or redefine spans that imply a standalone trusted PCS opening after it
no longer exists.

## Implementation Order

1. Confirm the three approval gates.
2. Define the fixed trusted profile and Phase-1 role/order constants.
3. Add the grouped schedule request, regenerate the K256 table, and fix setup
   capacity accounting.
4. Version the outer protocol, transcript domains, and proof carrier; change
   public commitment encoding only if redundant profile metadata is chosen.
5. Retain and validate the actual committed-group profiles in prover hints and
   derive verifier profiles from approved catalog rows.
6. Add the context-aware final trace commit.
7. Add the heterogeneous streaming source adapter and grouped prove/verify
   seam.
8. Generalize the pre-deserialization guard and transcript statement binding.
9. Introduce the trusted prover artifact and remove trusted recommit.
10. Convert modular Stage 0 and Stage 8.
11. Convert verifier Stage 8.
12. Port legacy through the same shared adapter and restore byte parity.
13. Add schedule, setup, adapter, pathology, e2e, codec, and FS tests.
14. Run the required validation suites.
15. Rerun and append the production benchmark with joint timing labels.

## Acceptance Criteria

- [x] Approval explicitly confirms 8 MiB / `2^20` words, or the entire fixed
      key is updated to the literal 4 GiB case before code changes.
- [x] Approval explicitly confirms that Phase 1 leaves untrusted advice as an
      auxiliary proof.
- [x] Approval explicitly confirms the fixed K256 final `(39, 1)` benchmark
      row rather than a full shape matrix.
- [x] Trusted advice is committed once, independently, under the standalone
      `JoltDense (20, 1)` profile.
- [x] The exact frozen profile is preserved, independently validated, and
      supplied to the final commit context.
- [x] The trusted artifact and final setup share the canonical setup-family
      identity, and the final extent covers the trusted profile's matrices.
- [x] `OneHotTrace` is committed under the approved grouped row whenever
      trusted advice is present and under the scalar row when absent.
- [x] Main and trusted commitments remain distinct public objects.
- [x] Stage 8 emits one `main_batch` proof and the verifier performs one Akita
      verification for trusted plus main.
- [x] Trusted advice never appears in the auxiliary proof list.
- [x] Untrusted advice remains a separately committed/proved/verified Phase-1
      auxiliary object with explicit tests for untrusted-only and both-present
      cases.
- [x] Main setup capacity covers arity 39, total polynomial count 2, and the
      maximum footprint of both approved scalar/grouped rows.
- [x] The verifier never invokes planner DP and accepts only the checked-in
      exact grouped row.
- [x] Group roles, order, profiles, commitments, points, and values are bound
      into the transcript before one bridge challenge.
- [x] Dense points remain unchanged and main one-hot points reverse exactly
      once at the backend boundary.
- [x] Outer byte vectors are hard-capped before allocation, and every exact
      commitment/proof shape is checked before backend deserialization or
      schedule-driven allocation.
- [x] No full `2^39` main table, generic expanded one-hot trace, or deep dense
      clone is introduced.
- [x] Unsupported trusted/main shapes fail early and never silently fall back
      to a separate trusted proof.
- [x] Dense advice PIOP semantics and all retained advice reductions are
      unchanged.
- [x] Modular and legacy Akita proofs are byte-identical.
- [x] Old packed-dense outer proofs and wrong-profile scalar main commitments
      fail closed; an identical standalone trusted commitment remains usable
      only with its retained matching artifact and canonical setup family.
- [x] Dory clear/ZK behavior remains unchanged.
- [x] Instrumentation reports separate commits and joint prove/verify without
      inventing per-group PCS timings.
- [x] The SHA2-chain scale-26 8 MiB benchmark is rerun and recorded only after
      correctness, tamper, lint, and regression suites pass.

## Phase 2: Untrusted Advice in the Joint Batch (Implemented)

Phase 2 generalizes the batch from one precommitted group to an ordered list,
and puts untrusted advice in it. The canonical public group order is:

```text
[UntrustedAdvice, TrustedAdvice, OneHotTrace]
```

Both advice objects and the trace are now discharged by **one** joint opening
proof and one joint verification. Untrusted advice is no longer an auxiliary
opening; only committed-program objects remain auxiliary.

### Why untrusted advice can be precommitted

"Precommitted" is a batch-structure property, not a trust property: it means the
group is committed before the final group, so the final commit can be
conditioned on its frozen profile. Untrusted advice qualifies because its
**shape** is known at preprocessing (`max_untrusted_advice_size`) even though its
**value** is only committed during proving. The transparent object setup and the
dense precommit profile both derive from the max size alone
(`advice_dense_setup`, `dense_precommit_profile`), so the grouped schedule row is
plannable at preprocessing exactly as the trusted row is.

Trust is unchanged by batching: the trusted commitment still arrives
out-of-band under its committer's attestation, and untrusted advice is still
bound only by the execution argument and the PCS. In particular, batching does
**not** range-prove untrusted coefficients — canonical `u64` word packing remains
an honest-prover/API invariant, as in `akita-dense-advice.md`.

### Group order and presence

Order is explicit, not positional. Each precommitted group carries a
`PrecommittedRole`, whose variant order defines the canonical batch order and
whose label is bound into the statement transcript. A list that is empty,
duplicated, or out of ascending role order is rejected fail-closed
(`validate_precommitted_order`).

Which advice objects a proof carries is a runtime fact, so all four presence
combinations are reachable and each has its own grouped row:

| Untrusted | Trusted | Groups in the batch |
|---|---|---|
| absent | absent | `[OneHotTrace]` (scalar, non-grouped opening) |
| present | absent | `[UntrustedAdvice, OneHotTrace]` |
| absent | present | `[TrustedAdvice, OneHotTrace]` |
| present | present | `[UntrustedAdvice, TrustedAdvice, OneHotTrace]` |

Preprocessing provisions rows for every combination reachable from the
program's public capacities (`AdvicePrecommitLayouts::precommit_combinations`),
so a program whose advice happens to be empty at proving time cannot land on a
missing row. The offline emitter enumerates the same combinations
(`emit::advice_precommit_combinations`); the two must agree or a reachable proof
finds no row.

The combination list is **deduplicated by profile sequence**, because a grouped
row is keyed on the frozen dense profiles alone and never on which advice kind
produced them. When both advice capacities have the same physical arity — as
both the production (`20`) and fixture (`14`) layouts do — the untrusted-only and
trusted-only cases resolve to the *same* single-precommit row, and emitting it
under both names would be a duplicate row identity. Distinguishing the two cases
is the statement transcript's job, not the schedule's: the verifier derives the
roles from its own reduction schedule, so a proof cannot substitute one advice
kind for the other even though they share a schedule row.

### Ordering change in the prover

Both advice objects are now committed **before** the trace, since the final
commit is conditioned on every precommitted hint. Commit order and Fiat-Shamir
absorb order are decoupled: the absorb sequence remains the canonical
`OneHotTrace`, untrusted advice, trusted advice, `ProgramOneHot`. In the legacy
prover this required splitting the untrusted commit from its absorb.

### Capacity and versioning

Total batch polynomial capacity is now derived rather than fixed at 2: one group
per advice object the program can precommit, plus the trace
(`grouped_batch_poly_capacity`), so 3 when both capacities are nonzero.

The statement domain moves to `akita_precommit_batch_v3` /
`jolt-akita/precommitted-group-batch/v3`, the group count is bound explicitly,
and the commitment config becomes `PackedAllAdviceBatched` (version 3).
`PackedDenseAdviceBatched` is retained only as a wire tombstone: v2 proofs are
rejected rather than verified through a compatibility fallback, matching the
Phase-1 break policy.
