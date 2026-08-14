# Spec: Dense Advice Commitment Objects on Akita

| Field | Value |
|-------|-------|
| Author(s) | Jolt contributors |
| Created | 2026-08-14 |
| Status | proposed |
| PR | |

## Summary

Akita currently commits trusted and untrusted advice as byte one-hot tables.
The rest of Jolt already reasons about advice as a multilinear polynomial of
little-endian `u64` words, so the Akita path runs extra reconstruction
sumchecks to prove that each final word claim decodes from its byte one-hot
commitment.

This change makes each Akita advice commitment bind the canonical word
polynomial directly. Trusted advice remains an independently precommitted
object, untrusted advice remains an independently per-proof object, and both
remain separate from the main `OneHotTrace` commitment. Stage 4 and the
stage-6b/stage-7 `AdviceClaimReduction` stay unchanged. Only the Akita-specific
advice byte tables, their one-hot validity/decode relations, and their
reconstruction-batch members are removed.

For an advice capacity containing `N` words, the committed logical domain
shrinks from `2^11 * N` byte-one-hot cells to `N` word values. Akita's checked-
in dense schedule has a 14-variable floor, so objects with `N < 2^14` use the
existing prefix-packing protocol to place the word polynomial in slot zero of
a `2^14`-coefficient physical polynomial; this physical padding does not
change the logical advice statement. Advice-only proofs no longer contain a
reconstruction sumcheck. Reconstruction remains a protocol phase for
committed bytecode and program-image objects.

This is also a statement change for untrusted advice: deleting its byte
booleanity/hamming proof removes Akita's global `< 2^64` guarantee. The new
verifier language, like the current Dory path, permits an arbitrary
field-valued logical advice vector; canonical `u64` packing remains an honest
SDK/prover invariant. Protocol review must explicitly approve that relaxation
or add a separate backend-neutral range argument before implementation.

## Intent

### Goal

Commit and open `TrustedAdvice` and `UntrustedAdvice` directly as independent,
dense, word-valued Akita polynomial objects, and eliminate every active
advice-specific byte-one-hot and reconstruction component without changing
Jolt's guest input API, memory layout, or base advice claim reductions.

### Terminology

| Term | Meaning in this spec |
|------|----------------------|
| Raw advice | The existing trusted or untrusted `Vec<u8>` produced by SDK serialization and stored in `JoltDevice`. |
| Dense advice polynomial | The full-domain MLE with one field coefficient per zero-padded little-endian `u64` word. “Dense” describes the committed statement, not necessarily its in-memory scalar type. |
| Physical advice polynomial | The Akita PCS object. It equals the logical word polynomial when its arity is at least 14; below that floor it is the canonical slot-zero extension defined in this spec. |
| Compact encoding | `jolt-witness::PolynomialEncoding::Compact`, which stores the dense logical word table as `u64` values and promotes them to the field when needed. It is not one-hot. |
| Advice object | One independently committed Akita object, with its own shape-exact transparent setup, commitment, opening hint, and opening proof. |
| Advice reconstruction | Only `TrustedAdviceReconstruction` and `UntrustedAdviceReconstruction`, which currently convert final word claims into byte-one-hot claims. It does not mean the base `AdviceClaimReduction`. |
| Runtime advice | The separate `#[jolt::advice]`/`AdviceTape` mechanism. It is unrelated to trusted and untrusted input categories in this spec. |

### Invariants

#### Canonical encoding

- The source of an advice polynomial remains the existing raw advice byte
  buffer. No new length, delimiter, or metadata coefficient is committed.
- Bytes are packed eight at a time as little-endian `u64` values. A partial
  final word and every unused capacity word are zero-padded.
- Actual byte length is not a separate committed value. As in the current
  memory model, buffers that differ only by trailing zero padding define the
  same advice polynomial.
- The logical polynomial domain is determined only by the public, aligned
  maximum advice capacity. Its canonical Akita physical arity is then derived
  deterministically from that logical domain and the public dense-schedule
  floor. Neither depends on actual byte length, values, trace length, or
  `OneHotTrace` configuration.
- In the honest canonical object, slot zero must have exactly the same
  coefficient table as the `TrustedAdvice`/`UntrustedAdvice` oracle used by
  Stage 4 and every other physical slot is zero. The bytes-to-words
  construction is a single shared implementation in `common`, not two
  definitions held together by tests. The verifier enforces this extension
  only at the protocol's random opening point, as quantified in the soundness
  section; it does not prove every padding coefficient is globally zero.
- `TrustedAdvice` and `UntrustedAdvice` remain logically dense word
  polynomials even if their witness-plane encoding remains `Compact<u64>`.
  Changing their `PolynomialEncoding` to `Dense` is not required by this
  protocol and must not be used as a proxy for completing the migration.

#### Commitment lifecycle and separation

- The main per-proof `OneHotTrace` commitment, untrusted advice commitment,
  trusted advice commitment, and committed-program objects remain distinct
  commitment objects.
- Trusted advice is committed out of band. The prover must possess the trusted
  bytes and opening material, while the verifier receives only the trusted
  commitment and its public shape/setup.
- Untrusted advice is committed during proof generation and its commitment
  remains in `JoltProof::untrusted_advice_commitment`.
- Trusted and untrusted advice are never combined with each other. This
  preserves trusted-commitment reuse and the different ownership/lifecycle of
  the two inputs.
- The commitment transcript order remains:

  1. `OneHotTrace`;
  2. untrusted advice, if present;
  3. trusted advice, if present;
  4. committed-program objects in their existing canonical order.

- The auxiliary opening-proof order remains untrusted advice, trusted advice,
  then committed-program objects. Code must not inherit the trusted-first walk
  order of the homomorphic `precommitted_final_openings` helper accidentally.

#### Claim binding

- Stage 4 continues to evaluate the dense word advice polynomial at the RAM
  initialization address subpoint and includes that value in `RamValCheck`.
- `AdviceClaimReductionCyclePhase` and `AdviceClaimReduction` remain for both
  advice kinds. They are not one-hot relations; they reduce the Stage-4 word
  claim to the advice object's final native opening point.
- A final advice claim may be completed in stage 6b or stage 7. Stage 8 must
  use the schedule-aware existing fallback rule and must not assume a stage-7
  output is always present.
- Stage 8 directly discharges the final logical claim
  `A_kind(r_kind) = v_kind` against that kind's dense advice commitment,
  using only the canonical prefix/floor claim reduction when `q < 14`.
  This reduction samples selector coordinates and is not a sumcheck.
- There is no advice byte polynomial between `AdviceClaimReduction` and the
  PCS opening, and there is no advice-specific booleanity, hamming, decode, or
  reconstruction sumcheck.
- Prover and verifier construct identical points, values, object order,
  layouts, transcript messages, and presence decisions. Point-order handling
  must be centralized. A `PrecommittedFinalOpening.point` is already in Jolt's
  canonical high-to-low protocol order. Constructing `PrefixPackedClaims`
  wraps/tags that logical vector once; `PrefixPackedLayout::reduce_claims`
  returns an `EvaluationClaim` whose physical point remains typed in that
  order. Code must not wrap it again or manually reverse coordinates. The
  Akita adapter alone performs the backend's required bit-order conversion.

#### Shape, setup, and presence

- Each advice object has one logical dense word polynomial of arity `q =
  log2(N)`, one physical dense polynomial of arity `p = max(q, 14)`, and a
  nonzero, protocol-owned, kind-separated layout digest. The physical arity
  floor is part of this protocol version, not an adapter convenience.
- The transparent setup is derived from physical arity `p`, physical
  polynomial count `1`, and the new layout digest. It must select Akita's
  dense backend and a checked-in `JoltDense` schedule row.
- The verifier checks commitment and setup backend, arity, polynomial count,
  layout digest, kind, and presence before calling the PCS verifier.
- A present nonempty advice buffer containing only zero bytes still produces a
  commitment and an auxiliary opening proof. Presence is not inferred from
  coefficient values.
- Per-proof schedule presence controls the commitment, final opening, and
  auxiliary proof. An absent advice input produces none of those. A
  capacity-derived verifier setup slot may still exist and is ignored when
  that kind is absent; the same preprocessing must support proofs with and
  without advice. A scheduled advice kind requires its corresponding setup,
  commitment, final opening, and proof. An unscheduled commitment, final
  opening, or proof is rejected.
- Trusted and untrusted commitments are not substitutable even if their
  capacities and coefficient tables are identical.

#### Reconstruction phase

- With a full/public program, advice alone never causes the reconstruction
  phase to run.
- With committed program preprocessing, bytecode and program-image
  reconstruction remains unchanged and continues to control the presence of
  `reconstruction_sumcheck_proof`.
- In a combined advice plus committed-program proof, the reconstruction batch
  contains program members only; dense advice is opened independently after
  those members are verified.

#### Compatibility and unaffected modes

- The Dory/homomorphic clear and ZK protocols remain byte-for-byte unchanged.
- `zk + akita` remains unsupported and rejected at compile time.
- Existing Akita byte-one-hot advice commitments, setups, proofs, and cached
  preprocessing are incompatible with the new protocol and must fail closed.
- The proof self-description distinguishes the old packed protocol from the
  new dense-advice packed protocol before stage verification.
- Retired serialized enum discriminants are not reused for new meanings.

There is no current Akita/advice-specific `jolt-eval` invariant. The canonical
encoding invariant is small and deterministic, so it belongs in
`common`, `jolt-witness`, `jolt-claims`, and PCS object unit tests rather than
pulling the Akita stack into the generic invariant harness.

### Non-Goals

- Changing `TrustedAdvice`, `UntrustedAdvice`, or `PrivateInput` guest/host
  APIs, postcard serialization, `JoltDevice`, RAM addresses, maximum-size
  attributes, or tracer behavior.
- Changing runtime advice (`AdviceTape`).
- Merging advice into `OneHotTrace`, merging trusted and untrusted advice, or
  creating one joint commitment for all auxiliary objects.
- Removing the word-level Stage-4 advice evaluation or either phase of
  `AdviceClaimReduction`.
- Removing bytecode or program-image reconstruction, or changing their
  one-hot encodings.
- Changing OneHotTrace RA, increment, booleanity, or hamming-weight semantics.
- Changing the Dory commitment/opening path or its BlindFold constraints.
- Adding Akita ZK support.
- Proving a new global `u64` range property for advice coefficients. The
  precise soundness decision for this migration is stated below.
- Preserving verification of old Akita proof bytes or reusing old trusted
  advice commitments.

## Evaluation

### Acceptance Criteria

- [ ] The canonical advice coefficient table is defined once in `common` as
      zero-padded little-endian `u64` words and is called by the modular
      witness oracle plus both modular and legacy Akita object constructors.
- [ ] `TrustedAdvice` and `UntrustedAdvice` retain word arity and compact
      witness storage; the active `TrustedAdviceBytes` and
      `UntrustedAdviceBytes` oracle surfaces are gone.
- [ ] Akita commits each present advice kind as a singleton logical dense word
      object at exact word arity `q`, embedded canonically into one physical
      dense object of arity `max(q, 14)`, with a new kind-separated/versioned
      digest.
- [ ] Main, trusted, and untrusted commitments remain separate, with unchanged
      commitment labels and ordering.
- [ ] Stage 4, `AdviceClaimReductionCyclePhase`, and
      `AdviceClaimReduction` remain active and pass their existing tests.
- [ ] No prover or verifier path constructs or executes
      `UntrustedAdviceReconstruction` or `TrustedAdviceReconstruction`.
- [ ] No active final-opening map refers to
      `TrustedAdviceBytes`/`UntrustedAdviceBytes`.
- [ ] Stage 8 resolves direct advice finals from either stage 6b or stage 7 and
      obtains the logical `TrustedAdvice`/`UntrustedAdvice` word point/value,
      prefix-reduces it, and opens the physical object at the resulting
      `p`-coordinate point and scaled value.
- [ ] An advice-only full-program proof has
      `reconstruction_sumcheck_proof == None` and no reconstruction transcript
      activity.
- [ ] A committed-program proof still has the required reconstruction proof;
      adding advice does not add advice members to that batch.
- [ ] The number of auxiliary PCS proofs is unchanged for advice: zero, one,
      or two according to presence, followed by any program objects.
- [ ] Missing, extra, reordered, or cross-kind advice commitments/proofs are
      rejected; scheduled advice without its capacity-derived setup is
      rejected, while an unused setup slot is allowed for absent advice.
- [ ] Old byte-one-hot setup metadata or proof protocol configuration is
      rejected before a dense advice opening is accepted.
- [ ] The modular and legacy Akita provers emit byte-identical proofs for the
      existing parity fixtures after both are migrated.
- [ ] Dory clear and Dory ZK advice tests remain unchanged and pass.
- [ ] Fiat-Shamir source, challenge, and scope inventories reflect removal of
      advice reconstruction, addition of the Akita version marker, and
      retention of program reconstruction.
- [ ] No allocation proportional to `2^(word_vars + 11)` remains for advice;
      the physical coefficient-table domain is exactly
      `2^max(word_vars, 14)` (implicit zero storage is permitted).
- [ ] Documentation no longer describes advice as an active Akita byte
      one-hot object.
- [ ] Protocol/security review explicitly approves the untrusted-advice
      language change from globally byte-ranged words to an arbitrary
      field-valued logical vector, or this migration is blocked pending a
      separately specified range proof.
- [ ] The proof soundness budget includes the additional floor-selector term
      `(d_untrusted + d_trusted) / |F|` for present advice objects.

### Testing Strategy

#### Canonical encoding and witness tests

Add focused tests beside the shared helper in `common`, then extend
`crates/jolt-witness/src/backend/trace/tests.rs` and the modular/legacy Akita
object tests. Cover:

- disabled/zero capacity;
- empty input (the helper returns the zero logical table for a nonzero
  capacity, while top-level scheduling still treats the empty buffer as
  absent and creates no commitment/proof);
- nonempty all-zero input;
- lengths `1..=7`, exactly `8`, and `9` bytes;
- a word equal to `u64::MAX`;
- a partial final word;
- unused capacity words;
- exact configured capacity;
- oversize rejection;
- rejection of a non-word-aligned capacity at the helper/object-construction
  boundary, even though normal `MemoryLayout` construction already prevents
  it;
- rejection of any supplied nonzero capacity that is not a power of two. The
  object API must not align it or silently call `next_power_of_two` and
  commit a larger capacity than the public layout declares.

For both kinds, compare all of the following coefficient-by-coefficient:

1. `common`'s canonical `Vec<u64>`;
2. `TraceBackend::oracle_table(TrustedAdvice|UntrustedAdvice)`;
3. the first `N` entries of the polynomial retained by `DenseAdviceObject`,
   followed by the required zero physical slots;
4. the advice words installed into legacy Stage 4.

Evaluate the logical MLE at Boolean and random field points. For every test
shape also sample the physical selector point `s` and check directly that the
stored physical MLE evaluates to `eq(s, 0) * A(r)`. Feed that same resolved
claim through PCS open/verify to catch endianness, prefix, or point-order
mistakes.

#### Layout, setup, and PCS tests

Add tests in `jolt-claims` and `jolt-akita` for:

- a singleton logical plan containing the direct `TrustedAdvice` or
  `UntrustedAdvice` ID at word arity `q`;
- physical arity `p = max(q, 14)`, slot capacity `2^(p-q)`, one occupied slot,
  and one physical polynomial;
- zero selector variables when `q >= 14`, and the exact selector count `14-q`
  when `q < 14`;
- stable, nonzero, distinct trusted/untrusted digests;
- digests distinct from the retired byte-one-hot layout;
- exact dense-backend commitment/setup metadata;
- default 4 KiB and large 8 MiB advice shapes;
- small word arities that use the dense schedule's folded-only padding floor;
- wrong kind, arity, polynomial count, backend, and digest rejection;
- a direct commit/open/verify round trip obtained from an arbitrary logical
  word point through the prefix reduction.

The checked-in dense schedule catalog covers exact physical arities
`14..=34`; it does not pad an uncataloged opening automatically. Tests must
demonstrate the explicit slot-zero embedding at the default 4 KiB capacity
(`q = 9`, `p = 14`, slot capacity `32`) and the no-floor case at 8 MiB (`q =
20`, `p = 20`, slot capacity `1`). Reject any capacity whose derived physical
arity is outside the catalog. Regenerate the catalog only if a deliberate
reachable-key change alters its emitted rows; comment changes alone do not
justify replacing generated schedule data.

#### End-to-end presence matrix

Run both trace polynomial orders and both supported OneHotTrace chunk widths
where practical. The minimum matrix is:

| Program mode | Advice | Reconstruction | Advice auxiliary proofs |
|--------------|--------|----------------|-------------------------|
| Full | none | absent | 0 |
| Full | trusted only | absent | 1 trusted |
| Full | untrusted only | absent | 1 untrusted |
| Full | both | absent | untrusted, trusted |
| Committed | none | present, program only | 0 plus program objects |
| Committed | trusted/untrusted/both | present, program only | present advice in canonical order, then program objects |

Include cases whose advice reduction completes in stage 6b and cases that run
the stage-7 address phase. Include nonempty all-zero advice, partial-word
input, default capacity, exact capacity, and the largest supported benchmark
capacity. Reuse one trusted commitment across multiple valid traces/proofs.

#### Tamper and fail-closed tests

Update `crates/jolt-verifier/tests/soundness/tampering/akita.rs` and the tamper
manifest to cover:

- missing or changed external trusted commitment;
- changed proof-carried untrusted commitment;
- a commitment built for the other advice kind;
- changed final word claim or point;
- dropped, duplicated, or swapped auxiliary proof;
- an advice auxiliary proof moved across a program object;
- wrong dense setup arity/digest/backend;
- wrong logical arity, physical arity, or floor-slot capacity (deterministic
  rejection), plus a malicious object with a nonzero unused physical slot
  (rejection except with the quantified prefix-reduction soundness error);
- an old byte-one-hot setup/commitment under a dense protocol proof;
- an unexpected reconstruction proof in an advice-only/full-program proof;
- a missing reconstruction proof in committed-program mode;
- a changed bytecode/program-image reconstruction wire in combined mode;
- old `CommitmentConfig::Packed` versus the new protocol discriminator, and
  an old no-advice proof whose discriminator alone is relabeled to
  `PackedDenseAdvice` (the missing preamble marker must make it fail).

Also prove that a preprocessing object with capacity-derived trusted and
untrusted setup slots accepts a proof scheduling neither kind, while a proof
scheduling either kind fails if that setup is removed. Tombstoned byte-advice
oracle IDs must return the explicit retired/not-served error and must never
materialize a table.

The obsolete tests that mutate advice reconstruction output cells or drop an
advice-caused reconstruction proof are removed, not weakened. Their coverage
is replaced by direct final-claim and PCS-opening tampering.

#### Parity and regression suites

The legacy and modular Akita implementations must migrate atomically because
`akita_byte_diff` treats legacy as the proof-byte oracle. Update in-memory
verifier fixtures, proof component comparisons, and all one-hot terminology.
No checked-in trusted commitment or setup may be silently reused.

Representative validation commands after implementation are:

```bash
cargo nextest run -p common --cargo-quiet
cargo nextest run -p jolt-witness --cargo-quiet
cargo nextest run -p jolt-claims --features akita --cargo-quiet
cargo nextest run -p jolt-akita --cargo-quiet
cargo nextest run -p jolt-prover-legacy --features host,akita --cargo-quiet
cargo nextest run -p jolt-prover --features prover-fixtures,akita \
  --test-threads 1 --cargo-quiet
cargo nextest run -p jolt-verifier --features prover-fixtures,akita \
  --test-threads 1 --cargo-quiet
cargo nextest run -p jolt-verifier --features akita,fs-audit,prover-fixtures \
  --test-threads 1 --cargo-quiet

# Checked-in Akita schedule drift (CI profile, ignored regeneration test)
cargo nextest run --cargo-profile ci -p jolt-akita --run-ignored all \
  -E 'test(catalogs_match_planner_regeneration)' --cargo-quiet

# Unaffected protocol regressions
cargo nextest run -p jolt-prover --features prover-fixtures --cargo-quiet
cargo nextest run -p jolt-prover --features prover-fixtures,zk --cargo-quiet
cargo nextest run -p jolt-prover-legacy muldiv --cargo-quiet --features host
cargo nextest run -p jolt-prover-legacy muldiv --cargo-quiet --features host,zk

# Akita CI clippy matrix
cargo clippy -p jolt-verifier --all-targets --features akita
cargo clippy -p jolt-verifier --all-targets --features akita,prover-fixtures
cargo clippy -p jolt-prover-legacy --all-targets --features akita
cargo clippy -p jolt-prover --all-targets --features akita
cargo clippy -p jolt-prover --all-targets --features akita,prover-fixtures

cargo clippy --all --features host -q --all-targets -- -D warnings
cargo clippy --all --features host,zk -q --all-targets -- -D warnings
cargo fmt -q
```

### Performance

Let `N` be the logical word count, `q = log2(N)`, `p = max(q, 14)`, and `P =
2^p = max(N, 2^14)`. The old advice statement has `2^11 * N` logical cells and
`8N` sparse one positions; its reconstruction prover may materialize or fold
the full cell table. The new statement has `N` logical word coefficients, `P`
physical dense coefficients, and no advice reconstruction. Its object memory
must be `O(P)` and it must not retain either the old sparse positions or the
old cell table alongside the new word object. For `q >= 14`, this is `O(N)`;
for small capacities the fixed 14-variable PCS floor dominates.

The logical-domain and reconstruction-work reduction is `2^11`. Do not claim
an unconditional `2^11` memory reduction: the current commitment input stores
only `8N` sparse hot positions, and the new default 4 KiB object is physically
padded from `N = 512` to `P = 16,384`. The physical-domain ratio is
`(2^11 * N) / P` (64x at 4 KiB and 2048x once `N >= 2^14`), while actual peak
memory must be measured because the representations differ.

Keep the profiling span name `commit_advice`; it is part of the profiling
taxonomy. Measure the existing modular Akita benchmark at least at 4 KiB and
8 MiB advice capacities, on reference and optimized backends, with three warm
samples. Record:

- trusted commitment time;
- stage-0 untrusted commitment time;
- total proving and verification time;
- reconstruction time and member count;
- peak RSS and advice-object allocation size;
- commitment, auxiliary opening, and total proof bytes;
- the no-advice baseline.

Acceptance requires:

- no repeatable regression above 3% for the no-advice path or committed-program
  reconstruction path;
- zero advice reconstruction sumcheck rounds, polynomials, or claims and no
  advice reconstruction transcript activity/span. Serialized `Option` and
  empty-aggregate tags may remain and must be reported rather than called a
  zero-byte payload;
- peak advice memory that scales with `P = max(N, 2^14)`, not `2^11 * N`;
- proof size and end-to-end results reported against the byte-one-hot baseline;
- an explanation and explicit approval for any repeatable advice-workload
  time or memory regression.

There is no existing Akita-advice `jolt-eval` objective. The current
`modular_benchmark` advice lane is the acceptance harness. A future Akita-aware
telemetry workload may expose `commit_advice`, prover time, verifier time, and
heap metrics, but creating that general integration is not required for this
protocol cutover.

## Design

### Architecture

#### Current and target dataflow

The canonical word polynomial already exists. The current Akita path creates a
second representation only for commitment:

```text
Current Akita

raw bytes
   ├── LE u64 word MLE ── Stage 4 ── AdviceClaimReduction (6b/7) ── word claim
   │                                                                    │
   └── (byte | place | word) one-hot MLE ── AdviceReconstruction ── byte claim
                                                                         │
                                                     independent Akita PCS open
```

The target removes the lower branch:

```text
Target Akita

raw bytes ── LE u64 word MLE ── Stage 4 ── AdviceClaimReduction (6b/7)
                                                            │
                                             final word point/value
                                                            │
                                  prefix/floor claim reduction
                                                            │
                                      independent dense Akita PCS open
```

The binding chain for each kind is therefore:

```text
RAM initial-value contribution
  -> Stage-4 advice opening
  -> cycle/address claim reduction
  -> (TrustedAdvice | UntrustedAdvice)(r) = v
  -> prefix reduction to tilde_A(s || r) = eq(s, 0) * v
  -> PCS::verify(commitment, s || r, eq(s, 0) * v)
```

No equality to a second encoding needs to be proven because the commitment now
binds the polynomial consumed by the PIOP itself.

#### Canonical dense advice polynomial

For a valid public `MemoryLayout`, maximum capacity `M` is either zero or an
8-byte-aligned power of two. A scheduled/present advice kind requires `M > 0`.
Define:

```text
N             = max(1, M / 8)
q             = log2(N)

A[i] = sum_{j=0}^{7} byte[8*i + j] * 2^(8*j),  0 <= i < N,
```

where an index beyond the actual byte buffer contributes zero. `A[i]` is first
formed as a `u64` and then canonically embedded into the PCS field. Normal
`MemoryLayout` construction aligns advice capacities and enforces the power-
of-two rule. A standalone helper/object constructor must nevertheless reject
`M % 8 != 0` and any nonzero non-power-of-two `M`; it must not round up,
truncate, or silently commit a larger logical capacity.

The Boolean index is only `word`; the logical point has `q` coordinates. The
retired `(byte | place | word)` layout and its fixed 11-coordinate prefix do
not appear anywhere in the dense statement.

Extract the capacity validation and zero-padding logic from
`crates/jolt-witness/src/backend/trace/advice.rs` into a no-PCS, no-field helper
in `common/src/advice.rs`. That helper must call (or subsume and re-export)
the existing `common::jolt_device::bytes_to_words_le` primitive, then resize
its output to exactly `N`; it must not introduce a second endian loop. Both
`jolt-witness` and the modular and legacy Akita constructors call the helper,
and legacy Stage-4 memory population consumes the same returned words. The
helper returns `Vec<u64>` and a small capacity/oversize error, so callers
promote to `F` only at a field/PCS boundary. This placement is intentional:
`jolt-prover-legacy` does not depend on `jolt-witness`, and the encoding must
not have two implementations held together only by tests.

#### Dense advice object and packing plan

Replace `AdviceOneHot<PCS>` with an object equivalent to:

```rust
pub struct DenseAdviceObject<PCS: CommitmentScheme> {
    pub plan: PrefixPackedObjectPlan,
    pub polynomial: Polynomial<PCS::Field>,
    pub commitment: PCS::Output,
    pub hint: PCS::OpeningHint,
    pub setup: PCS::ProverSetup,
    pub logical_word_vars: usize,
    pub physical_num_vars: usize,
}
```

The exact owned polynomial type may use a compact/lazy zero-extension wrapper
if the Akita adapter supports it. The semantic interface must still be a
general dense `MultilinearPoly`, must return `is_one_hot() == false`, and must
not materialize the retired byte cell table. A plain `Polynomial<F>` of `P =
2^max(q, 14)` entries is the baseline implementation. The canonical compact
`Vec<u64>` may remain the source representation, but its PCS-facing logical
length, zero extension, and evaluations must match the physical plan exactly.

Replace `commit_advice_one_hot` with `commit_advice_dense`:

1. validate actual length and capacity alignment;
2. build the canonical `N` words;
3. promote those words into slot zero of the canonical `P`-coefficient
   field/PCS polynomial and zero all remaining slots;
4. construct the canonical kind-specific plan;
5. derive the transparent shape-exact dense setup;
6. commit and retain the opening hint.

Keep `SparseUnitPolynomial` and `ProgramOneHot`; committed bytecode and program
image still use them.

Akita's generated `JoltDense` catalog admits physical arities `14..=34` and
performs exact schedule lookup. It does not automatically pad a nine-variable
object. Add the protocol constants `DENSE_ADVICE_MIN_PHYSICAL_VARS = 14` and
`DENSE_ADVICE_MAX_PHYSICAL_VARS = 34`, and this canonical plan in
`crates/jolt-claims/src/protocols/jolt/lattice/packing.rs`:

```text
advice_dense_packing_plan(kind, q):
    domain separator = "advice-dense-words-v2"
    logical columns  = [(advice_polynomial(kind), q)]
    p                = max(q, DENSE_ADVICE_MIN_PHYSICAL_VARS)
    require DENSE_ADVICE_MIN_PHYSICAL_VARS <= p <= DENSE_ADVICE_MAX_PHYSICAL_VARS
    slot capacity C  = 2^(p - q)
    selector vars d  = p - q
```

`advice_polynomial(Trusted) = TrustedAdvice` and
`advice_polynomial(Untrusted) = UntrustedAdvice`. Extend
`PrefixPackedObjectPlan` with an explicit-capacity constructor rather than
letting `columns.len().next_power_of_two()` force `C = 1`. It still contains
one semantic ID in slot zero; the other `C - 1` slots are protocol-mandated
zero padding for the canonical honest constructor. For `q >= 14`, `C = 1` and
no selector challenge is drawn. This
keeps all auxiliary objects on the existing prefix-layout transcript and
metadata path without requiring new Akita schedule rows below 14.

The physical table and opening reduction are exactly:

```text
P                         = 2^p = C * N
tilde_A[(0 << q) | i]     = A[i]              for 0 <= i < N
tilde_A[(slot << q) | i]  = 0                 for 1 <= slot < C

tilde_A(s || r)           = eq(s, 0) * A(r)
```

Given the final logical claim `A(r) = v`, `PrefixPackedLayout::reduce_claims`
absorbs the layout digest, `r`, and `v`, samples `s` with `d` coordinates, and
returns the physical claim `tilde_A(s || r) = eq(s, 0) * v`. For `d = 0`, the
empty-selector equality is one and the claim is unchanged. This is the only
logical-to-physical point/value transformation; no padding coordinate is
fixed manually and no coordinate vector is reversed.

The layout digest must use fresh `append_auxiliary_id` tags for the already-
existing direct word IDs; do not reuse the retired byte tags. Reserve the old
tags and assign tag `10` to `TrustedAdvice` and tag `11` to
`UntrustedAdvice`. These are digest tags, not new committed-polynomial enum
variants. The kind ID, logical arity `q`, physical arity `p`, slot capacity
`C`, and domain/version string are all digest inputs.

`TransparentObjectSetup` already creates dense-only singleton setups, and the
Akita commitment implementation already has a dense polynomial fallback. No
new PCS primitive or Akita proof type is required. Update comments that call
all auxiliary objects byte or sparse-unit objects.

#### Commitment creation and absorption

The stage-0/prover lifecycle remains:

| Object | Creation time | Commitment carried by | Opening witness available to |
|--------|---------------|-----------------------|------------------------------|
| `OneHotTrace` | Every proof | `JoltProof::commitments` | Prover stage 8 |
| Dense untrusted advice | Every proof where input is present | `JoltProof::untrusted_advice_commitment` | Prover stage 8 |
| Dense trusted advice | Preprocessing/out of band; rebuilt and checked by the current prover path if needed for its hint | Verifier/prover argument | Trusted committer and prover |
| Program objects | Committed-program preprocessing | Program preprocessing | Prover preprocessing |

`crates/jolt-prover/src/akita/stage0.rs` constructs the untrusted dense object
and absorbs its commitment in the existing position. The top-level Akita
prover reconstructs the trusted dense object from the retained trusted bytes,
checks that its commitment equals the supplied external commitment, and uses
its opening hint. This preserves the current API and fail-closed comparison;
an API that transports the hint directly may be considered separately.

`akita_verifier_preprocessing` continues deriving trusted and untrusted setup
capabilities from nonzero public maximum capacities, not from one proof's
actual presence schedule. Consequently a setup may legitimately be present
and unused. The prover/verifier must consult `PrecommittedSchedule` to decide
whether to absorb a commitment and consume a final claim/proof, and consult
the setup slot only after that decision. If scheduled, the setup's physical
arity, backend, polynomial count, and digest must match the canonical plan.

The existing commitment labels (`commitment`, `untrusted_advice`,
`trusted_advice`, and program labels) are not renamed. The new protocol
discriminator and layout digest, not label churn, distinguish versions.

#### Stages 4, 6b, and 7

These stages are representation-independent and remain unchanged:

- `crates/jolt-prover/src/stages/stage4.rs` evaluates the direct word oracle;
- `crates/jolt-claims/src/protocols/jolt/relations/ram/val_check.rs` includes
  trusted/untrusted contributions to `Val_init`;
- `crates/jolt-claims/src/protocols/jolt/geometry/claim_reductions/advice.rs`
  defines the two-phase shape and final opening IDs;
- the stage-6b cycle phase and stage-7 address phase reduce the claim to the
  advice object's native point;
- the reference kernels continue consuming the compact word table.

In particular, legacy BlindFold `input_claim`/constraint synchronization and
`RamValCheck` reconstruction of the full initial evaluation are unrelated to
byte-one-hot advice reconstruction and must not be removed.

#### Shared final-claim resolver

`crates/jolt-verifier/src/stages/stage8/precommitted.rs` already implements the
correct schedule-aware resolution:

- choose the stage-7 address-phase point/value when address rounds exist;
- otherwise choose the stage-6b cycle-phase terminus;
- fail when the scheduled source is absent or inconsistent.

Make this resolver, or an advice-only extraction from it, available in Akita
builds. Do not duplicate the current `completed_advice_claim` logic in
reconstruction. In clear Akita mode each resolved entry must contain:

```text
JoltCommittedPolynomial::{TrustedAdvice|UntrustedAdvice}
protocol-order point
clear field value
```

This point is the logical `q`-coordinate point. Building
`PrefixPackedClaims` tags that logical point as high-to-low once, and the
advice packing plan performs the transcript-derived selector reduction
described above. `PrefixPackedLayout::reduce_claims` already returns the
resulting `p`-coordinate `EvaluationClaim` in the same typed protocol order;
pass it to `PCS::open`/`PCS::verify` as-is. Do not unwrap/rewrap or reverse it.
The Akita adapter owns the backend bit-reversal, exactly as for other packed
objects.

The all-polynomial helper currently walks trusted advice before untrusted
advice for the homomorphic final batch. Treat its result as a keyed set.
Akita's explicit object-order function must emit untrusted before trusted.

#### Advice-free reconstruction batch

The relation cut is exact:

| Relation/stage | Decision | Reason |
|----------------|----------|--------|
| Stage-4 `RamValCheck` advice terms | Keep unchanged | Bind RAM initialization to the logical word oracle. |
| `AdviceClaimReductionCyclePhase` | Keep unchanged | Reduces the Stage-4 cycle side; independent of commitment representation. |
| `AdviceClaimReduction` | Keep unchanged | Completes the native logical word claim, including the address phase when scheduled. |
| `UntrustedAdviceReconstruction` | Remove from the active protocol | Its degree-3, `q+11`-round byte booleanity/hamming/decode batch exists only for the retired byte one-hot object. |
| `TrustedAdviceReconstruction` | Remove from the active protocol | Its degree-2, 11-round byte decode relation exists only for the retired byte one-hot object. |
| `BytecodeChunkReconstruction` | Keep unchanged | Committed bytecode remains byte one-hot. |
| `ProgramImageReconstruction` | Keep unchanged | Committed program image remains byte one-hot. |

Thus “remove advice reconstruction” never means removing either phase of the
base `AdviceClaimReduction`.

Delete the active symbolic relations in
`crates/jolt-claims/src/protocols/jolt/lattice/relations/advice_reconstruction.rs`
and their prover/verifier implementations. Specifically remove:

- untrusted byte booleanity;
- untrusted per-`(place, word)` hamming weight;
- untrusted byte-to-word decode;
- the untrusted reference-vector and gamma draws;
- trusted byte-to-word decode;
- byte-claim output carriers for both advice kinds.

`ReconstructionSumchecks` retains only:

```text
bytecode: Option<BytecodeChunkReconstructionInstance<_>>
program_image: Option<ProgramImageReconstructionInstance<_>>
```

`build_reconstruction_parts`, prover reconstruction orchestration, clear claim
projection, and proof-presence checks derive phase presence only from committed
bytecode/program-image layouts. Keep the module, `FsScope::Reconstruction`,
sumcheck proof field, and program relations.

The proof's reconstruction claims aggregate removes trusted/untrusted advice
fields. An empty aggregate may still be represented in `ClearProofClaims`, but
it causes no proof or transcript operation.

#### Stage-8 statement assembly

Refactor the current `leaf_claims` construction into a fallible, shared final
leaf map used by prover and verifier. It contains:

- OneHotTrace leaves from stage 7;
- direct dense advice leaves from the shared stage-6b/stage-7 resolver;
- committed-program byte/image leaves from reconstruction.

Insertion must reject duplicates instead of silently replacing a prior map
entry. Every canonical object plan must find exactly its required IDs with
the expected point arity.

Stage 8 then performs, in order:

1. the existing native same-point `OneHotTrace` opening;
2. direct dense untrusted advice opening, if present;
3. direct dense trusted advice opening, if present;
4. existing program prefix-object openings.

`open_auxiliary` remains suitable. For advice it receives
`DenseAdviceObject::polynomial` and the direct `TrustedAdvice` or
`UntrustedAdvice` final leaf, rather than a sparse byte column and a
reconstruction leaf.

The verifier constructs advice plans from the public scheduled word arity,
not by subtracting 11 from an untrusted proof point. It requires schedule,
commitment, final leaf, and auxiliary proof presence to agree exactly and, for
a scheduled kind, requires the capacity-derived setup to match. An unused
setup is permitted for an unscheduled kind.

#### Formal soundness statement and range semantics

For floor-padded objects (`d = p-q > 0`), define the logical polynomial bound
by a physical commitment to be its slot-zero restriction:

```text
A(x) := tilde_A(0^d || x).
```

The honest constructor commits the canonical zero extension, but the verifier
does not inspect every unused coefficient. Conditional on the final logical
point `r`, prefix reduction checks the multilinear selector polynomial

```text
g(s) = tilde_A(s || r) - eq(s, 0) * v
```

at a transcript-random `s` sampled after the commitment and semantic claim are
absorbed. If the slot-zero claim is wrong, or an unused slot evaluates
nonzero at `r`, then `g` is nonzero and the check accepts with probability at
most `d / |F|` by Schwartz-Zippel. It does not prove that every unused-slot
coefficient is globally zero; global zero padding is a canonical honest-
constructor invariant and makes the physical object deterministic. The base
sumchecks/claim reduction supply their existing randomness for `r` and retain
their existing soundness analysis.

For two present advice objects, the additional floor-reduction error is at
most `(d_untrusted + d_trusted) / |F|` by a union bound; absent kinds contribute
zero, and `d = 0` contributes no additional error. For the default 4 KiB
capacity, `d = 5` per present object.

The old untrusted reconstruction proved more than equivalence between two
representations: byte booleanity plus hamming weight implied that every
decoded coefficient was a canonical value below `2^64`.

This spec deliberately changes the Akita committed-advice statement to match
the current homomorphic/Dory statement:

- the verifier language permits the untrusted logical advice polynomial to be
  an arbitrary vector in `F^N`, defined as the slot-zero restriction of the
  committed physical object;
- the honest SDK/prover constructor accepts bytes and embeds canonical `u64`
  words, but that packing is an honest-prover/API invariant rather than a
  cryptographically enforced property;
- `RamValCheck`, the two-phase `AdviceClaimReduction`, the prefix reduction,
  and the PCS opening bind execution to that same field-valued vector;
- there is no independent global proof that every committed coefficient lies
  in `[0, 2^64)`.

This decision must be called out in protocol review. It must not be described
as preserving the old Akita range claim, and merely sharing a byte-packing
helper does not establish it. If guest/RAM semantics require every advice
coefficient to be a canonical `u64`, then this proposal is not semantics-
preserving and must not ship without a separately specified backend-neutral
range argument or another explicit protocol feature. Keeping a hidden subset
of the byte-one-hot reconstruction is not part of this migration.

Trusted advice does not require an in-protocol range proof under the existing
trust model: its committer attests to the canonical bytes/word construction.

#### Fiat-Shamir delta

Commitment absorption labels/order and the Stage-1-through-Stage-7 draw
schedule remain unchanged. Their challenge values naturally change because
the advice commitments and preamble change. The cutover's transcript delta is:

- append the new Akita protocol-version scalar at the end of the preamble,
  including on no-advice proofs;
- remove the untrusted reconstruction reference vector, gamma, relation-batch
  coefficients, sumcheck rounds, and output claim absorption;
- remove the trusted reconstruction relation-batch coefficients, sumcheck
  rounds, and output claim absorption;
- retain all reconstruction scope/messages required by bytecode or program
  image members, with their batch formed only from those members;
- for each present dense advice object, absorb the existing
  `prefix_packed_claim` statement using logical `q`, floor-slot capacity `C`,
  fresh layout digest, logical point `r`, and value `v`, then draw `p-q`
  selector coordinates. This draw count is zero when `q >= 14`.

No new transcript method or scope is required. The static prefix-reduction
challenge call site already exists for packed auxiliary objects, but runtime
draw counts and all subsequent challenges change. Modular and legacy code
must therefore share the same object-order loop and be byte-diff tested. FS
source/absorb/challenge inventories remove the advice-specific reconstruction
source/challenge entries, add the Akita preamble marker, and keep the generic
reconstruction absorb sites, `FsScope::Reconstruction`, and prefix-layout
sites.

#### Protocol version, proof shape, and serialization

As a deliberate fail-early compatibility boundary, append this new
commitment-axis variant after every existing serde variant:

```rust
CommitmentConfig::PackedDenseAdvice
```

The `akita` build selects the new variant. Update both construction sites:
`SELECTED_COMMITMENT_CONFIG` in `crates/jolt-verifier/src/config.rs` and the
legacy packed prover's literal `JoltProtocolConfig` in
`crates/jolt-prover-legacy/src/zkvm/packed.rs`. Keep `Packed` as a
legacy/reserved value selected by no current build so a deserialized old
configuration cannot be mistaken for the new statement.
`validate_proof_config` rejects it before stage verification. A new variant is
not mathematically required merely because proof shape changes; it is the
chosen operational policy for clear version diagnostics and cache invalidation.

Bind the cutover in the transcript even when no advice is present. Define
`PACKED_DENSE_ADVICE_TRANSCRIPT_VERSION: u64 = 1` and
`PACKED_DENSE_ADVICE_ENCODING: u64 = 1`, and append them under the labels
`b"akita_protocol_version"` and `b"akita_advice_encoding"` immediately after
the existing `dory_layout` preamble scalar and before the first commitment. Both verifier
preamble entry points do this under `cfg(feature = "akita")`; the modular
Akita prover inherits that shared helper, and the legacy packed prover appends
the same label/value immediately after its common `fiat_shamir_preamble`
call. The second scalar explicitly binds the advice representation independently
of the broader packed protocol version. Homomorphic/Dory builds append nothing, so their proof bytes remain
unchanged.

Exact config validation, this preamble marker, and the fresh advice layout
digest are all mandatory. Consequently an old no-advice proof cannot be made
valid merely by changing its serialized `Packed` discriminator, and an
advice-bearing proof additionally binds the kind-specific dense layout. Add
the new preamble absorption to the FS source/absorb inventories and parity
fixtures.

The high-level proof shape otherwise remains:

- `JoltProof::commitments` is `OneHotTrace`;
- `JoltProof::untrusted_advice_commitment` is the separate dense untrusted
  object;
- the trusted commitment is a verifier argument;
- `AkitaJointOpeningProof::{one_hot_trace, auxiliary}` is unchanged;
- `reconstruction_sumcheck_proof` remains optional for committed program
  mode;
- reconstruction clear claims retain program fields only.

Removing advice reconstruction changes:

- reconstruction batching coefficients and round challenges;
- all later Fiat-Shamir state;
- setup arities and layout digests;
- proof bytes and preprocessing cache keys.

All Akita prover and verifier components land atomically. Regenerate in-memory
fixtures and any external cached preprocessing/trusted commitments.

Several ID enums are serialized positionally and later variants were appended
for codec stability. Do not reuse or reorder the retired slots. Retain the
following as clearly documented, unreachable tombstones with their existing
payload/type representations unless every affected positional codec is first
explicitly versioned:

- `JoltRelationId::{UntrustedAdviceReconstruction,
  TrustedAdviceReconstruction}`;
- `JoltCommittedPolynomial::{UntrustedAdviceBytes, TrustedAdviceBytes}`;
- `UntrustedAdviceReconstructionChallenge` and both reconstruction public-ID
  payload enums, plus their enclosing `JoltChallengeId`/`JoltDerivedId`
  variants;
- legacy `SumcheckId` reconstruction variants used by its compact opening-ID
  codec.

The reconstructed output-claim carrier fields are not ID discriminants and
are removed from the active proof projection. Tombstoned IDs have no relation
implementation, packing-plan constructor, stage member, or accepted final-
opening path. `TraceBackend` retains exhaustive arms for the byte-polynomial
IDs, but those arms return an explicit retired/not-served error rather than
materializing an oracle. New code must not emit any tombstone. If the
implementation instead removes one, it must version every affected positional
codec and add old/new round-trip and mismatch tests; silently shifting a
discriminant or changing a retained variant's payload representation is
forbidden.

#### Failure rules

The verifier rejects all of the following before or during Stage 8:

- scheduled trusted advice without a trusted commitment or trusted setup;
- scheduled untrusted advice without the proof-carried commitment or setup;
- an advice commitment when that kind is absent from `PrecommittedSchedule`;
- a final advice claim with the wrong polynomial ID or point length;
- a one-hot backend commitment for advice;
- a dense commitment/setup with the wrong arity, physical count, or digest;
- a trusted object in the untrusted transcript/proof position or vice versa;
- too few or too many auxiliary proofs;
- a reconstruction proof when no program reconstruction member exists;
- no reconstruction proof when a committed-program member exists;
- legacy packed protocol configuration or byte-advice metadata.

#### Component impact

| Area | Components | Required change |
|------|------------|-----------------|
| Protocol configuration/preamble | `crates/jolt-verifier/src/{config.rs,verifier.rs}` and `crates/jolt-prover-legacy/src/zkvm/{mod.rs,packed.rs}` | Add/select the dense-advice packed discriminator, retain old `Packed` as legacy/reserved, and absorb the exact Akita version marker after `dory_layout`. |
| Claim IDs and exports | `crates/jolt-claims/src/protocols/jolt/{ids.rs,mod.rs}` | Retire advice-byte relation/derived/challenge usage; preserve positional slots or explicitly version codecs. Keep direct advice IDs and base claim-reduction IDs. |
| Akita packing | `crates/jolt-claims/src/protocols/jolt/lattice/{mod.rs,packing.rs}` | Replace `advice_bytes_packing_plan` with the direct singleton dense-word plan, explicit floor-slot capacity, and fresh digest tags/domain. Own the 14-variable protocol floor here. |
| Advice relations | `crates/jolt-claims/src/protocols/jolt/lattice/relations/{mod.rs,advice_reconstruction.rs}` | Remove active advice reconstruction relation code and tests; keep bytecode/program-image relations. |
| Final-opening map | `crates/jolt-claims/src/protocols/jolt/geometry/committed_openings.rs` | Remove active byte-advice final mappings; retain direct `TrustedAdvice`/`UntrustedAdvice -> AdviceClaimReduction`. |
| Shared word encoding | `common/src/{lib.rs,advice.rs,jolt_device.rs}` | Own the no-field canonical capacity validation and zero-padding helper used by modular and legacy code; reuse or subsume the existing `bytes_to_words_le` endian primitive. |
| Witness materialization | `crates/jolt-witness/src/backend/trace/{advice.rs,oracle.rs,tests.rs}` | Call the common word helper; delete byte-table materializers; retain compact direct advice shapes. Tombstoned byte IDs return retired/not-served. |
| Akita object support | `crates/jolt-akita/src/{adapters.rs,scheme.rs,configs.rs,shape_guard.rs,schedules/mod.rs,schedules/jolt_fp128_dense.rs}` and `src/bin/gen_jolt_schedules.rs` | Update dense-object terminology, validate physical rather than logical arity, and assert the claims-layer floor agrees with catalog coverage. PCS algorithms do not change; `schedules/jolt_fp128_dense.rs` is expected to remain untouched unless a deliberate floor/catalog audit changes reachable rows. |
| Opening abstraction | `crates/jolt-openings/src/{prefix.rs,schemes.rs}` | Tests/docs only: exercise the already-existing explicit-capacity prefix reduction and update wording that assumes auxiliary objects are sparse/byte objects. No functional change or new PCS primitive. |
| Modular stage 0 | `crates/jolt-prover/src/akita/{witness.rs,stage0.rs}` | Introduce/commit `DenseAdviceObject`; preserve presence and absorb order. |
| Modular reconstruction | `crates/jolt-prover/src/akita/reconstruction.rs` | Remove advice kernels, folding, publics, and members; keep program reconstruction. |
| Modular stage 8 | `crates/jolt-prover/src/akita/{prover.rs,stage8.rs,mod.rs}` | Rebuild/check trusted dense object, resolve direct finals using stage 6b/7, and open word polynomials in canonical auxiliary order. Update comments and unreachable stub wording. |
| Verifier reconstruction | `crates/jolt-verifier/src/stages/stage8/reconstruction.rs` | Remove advice instances, inputs, outputs, and presence cause; keep program members and scope. |
| Verifier Stage 8 | `crates/jolt-verifier/src/stages/stage8/{mod.rs,precommitted.rs,packed.rs,verify.rs}` | Share the final advice resolver across features; build direct leaves and validate/open dense objects. |
| Proof/preprocessing | `crates/jolt-verifier/src/{proof.rs,preprocessing.rs,verifier.rs}` | Change reconstruction field semantics/docs, keep separate setup/commitment fields, preserve commitment order, and permit unused capacity-derived advice setups. |
| Legacy packed prover | `crates/jolt-prover-legacy/src/zkvm/{packed.rs,clear_claims.rs,proof.rs,prover.rs}` | Mirror dense objects/direct openings, remove advice reconstruction orchestration/projection, update setup construction and helper callsites. |
| Legacy relations/IDs | `crates/jolt-prover-legacy/src/zkvm/claim_reductions/{mod.rs,advice_bytes.rs}`, `crates/jolt-prover-legacy/src/poly/opening_proof.rs`, `crates/jolt-prover-legacy/src/zkvm/proof_parts.rs` | Delete the active advice-byte module; tombstone or version the two legacy `SumcheckId`s, preserve `SumcheckId::COUNT`/compact opening-ID behavior, and add codec tests. Keep `claim_reductions/advice.rs`. |
| Tests and FS inventories | `crates/jolt-prover/tests/{akita_e2e.rs,akita_byte_diff.rs}`, `crates/jolt-verifier/tests/{completeness/akita.rs,support,soundness,fs_inventory,fs_obligations.rs}`, legacy packed tests | Rewrite advice expectations/tampers and preserve modular/legacy parity. Remove advice-specific reconstruction source/challenge entries, add the Akita preamble marker, and retain generic reconstruction absorbs, prefix-layout sites, and `FsScope::Reconstruction` for committed program. |
| Benchmark/profiling | `crates/jolt-prover/examples/modular_benchmark.rs`, `crates/jolt-profiling/src/taxonomy.rs` | Benchmark dense advice; retain the stable `commit_advice` span name. |
| Feature/docs cleanup | affected `Cargo.toml` files and `jolt-openings/src/schemes.rs` | Update Akita feature comments and stale claims that all auxiliary/precommitted objects are byte one-hot. Do not change feature behavior. |
| Guest/SDK/tracer | `tracer`, `jolt-sdk`, examples | No semantic or API change. Only stale one-hot wording may change. |

### Alternatives Considered

#### Keep byte one-hot advice

This preserves the current Akita-specific range statement and exploits sparse
one-hot commitment input, but retains a `2^11` larger logical domain and two
extra relations whose only purpose is to bridge to the word polynomial used
by Jolt. It does not meet the requested simplification.

#### Commit one dense coefficient per byte

An eight-bit field value per byte is smaller than a byte one-hot table, but
Jolt still consumes words. A byte-to-word reduction would remain, so this does
not eliminate reconstruction.

#### Commit dense words and retain a separate range proof

This can preserve the old untrusted `< 2^64` guarantee, but it is a distinct
protocol feature. It requires a range-check design, cost model, and
backend-neutral soundness story. It is intentionally not hidden inside this
migration.

#### Merge advice into `OneHotTrace`

Advice has a different arity, point, ownership, and lifetime. Trusted advice
must be reusable across proofs, while `OneHotTrace` is trace-specific. Merging
would lose that property and complicate Akita's native same-point opening.

#### Merge trusted and untrusted advice into one auxiliary object

The two kinds have different commitment times and trust boundaries. A joint
object would force trusted advice to be recommitted per proof or would allow
untrusted data into a trusted precommitment. Separate objects are required.

#### Bypass `PrefixPackedObjectPlan` for singleton advice

A raw direct opening would work mathematically. Reusing the singleton prefix
object path is preferred because it already binds layout metadata, normalizes
transcript assembly, validates all auxiliary objects consistently, and gives
small advice polynomials the exact slot-zero reduction needed by Akita's
14-variable floor. It draws no selector challenge once `q >= 14`; below the
floor it draws exactly `14-q`.

#### Reuse `advice_bytes` digest tags for word advice

Even though arity usually differs, tag reuse creates unnecessary ambiguity and
could collide at unusual shapes. A new domain/version and new direct-ID tags
make old setup/commitment reuse fail closed.

## Documentation

This spec supersedes only the advice-byte portions of
[`lattice-claims.md`](./lattice-claims.md): the all-unstructured-data-one-hot
rationale, advice byte object geometry, advice reconstruction relations,
advice reconstruction scheduling, and related open question. During
implementation, add a supersession note there and update its active protocol
tables; do not alter its OneHotTrace or committed-program design.

Add a similar supersession note to the advice/program-reconstruction
engineering-debt item in
[`akita-optimal-committed-data.md`](./akita-optimal-committed-data.md).

The public Jolt book's guest, trusted-advice, and untrusted-advice APIs do not
change. Optionally add one sentence to the RAM architecture page stating that
the committed advice polynomial contains zero-padded little-endian 64-bit
words. Internal module docs in `jolt-claims`, `jolt-prover`, `jolt-verifier`,
`jolt-openings`, and `jolt-akita` must stop describing advice as a byte
one-hot/sparse-unit object.

## Execution

Land the migration as one protocol change across the modular prover, legacy
parity prover, and verifier. A recommended order is:

1. **Freeze the statement and version.** Add the new packed protocol
   discriminator and transcript marker, reserve old serialized tags, define
   the direct advice plan, 14-variable slot-zero embedding, domain separator,
   ID tags, object order, and formal range semantics.
2. **Single-source word construction.** Extract canonical advice word packing
   into `common`, point the existing trace oracle and both Akita provers at it,
   remove byte-table oracle service, and add coefficient parity tests before
   changing proving code.
3. **Introduce dense advice objects.** Replace modular and legacy one-hot
   constructors, derive physical-arity setups from the canonical logical plan,
   validate dense metadata, and update trusted preprocessing/untrusted stage-0
   commitment callsites.
4. **Share final advice resolution.** Ungate or refactor the existing
   precommitted final-opening resolver so both Dory and Akita obtain stage-6b
   fallback/stage-7 final claims through one implementation.
5. **Open advice directly.** Update modular prover/verifier Stage 8 and the
   legacy packed prover to build direct advice leaves and independently open
   untrusted then trusted dense objects.
6. **Remove advice reconstruction.** Delete symbolic relations and active IDs,
   modular kernels, legacy `advice_bytes` provers, verifier instances, clear
   claim fields, and proof-presence causes. “Active IDs” means all emission and
   dispatch uses; retain the serialized enum variants as tombstones. Keep the
   program reconstruction phase intact.
7. **Update wire checks and inventories.** Change proof docs/presence rules,
   protocol validation, setup/cache invalidation, Fiat-Shamir inventories, and
   serialization/tombstone tests.
8. **Restore parity and soundness coverage.** Update modular/legacy byte-diff,
   e2e, fixture, and tamper suites using the full presence matrix.
9. **Benchmark and document.** Run the 4 KiB/8 MiB comparison, record proof
   size/time/RSS, update superseded specs and module comments, and run all
   acceptance commands.

Do not split steps 3–7 into independently mergeable prover/verifier changes:
the transcript, proof claims, layout digest, and setup shape change together.

## References

- [`lattice-claims.md`](./lattice-claims.md) — current Akita protocol; this
  spec supersedes its advice-byte sections.
- [`akita-upstream-split.md`](./akita-upstream-split.md) — ownership boundaries,
  compatibility policy, parity, and performance acceptance for Akita.
- [`akita-optimal-committed-data.md`](./akita-optimal-committed-data.md) — prior
  committed-data optimization analysis.
- [`1344-committed-bytecode-program-image.md`](./1344-committed-bytecode-program-image.md)
  — base precommitted scheduling and direct word advice lifecycle.
- `crates/jolt-witness/src/backend/trace/advice.rs` — current canonical word
  materialization and retired byte table.
- `crates/jolt-claims/src/protocols/jolt/geometry/claim_reductions/advice.rs`
  — retained two-phase advice claim reduction.
- `crates/jolt-claims/src/protocols/jolt/lattice/relations/advice_reconstruction.rs`
  — relations removed by this design.
- `crates/jolt-verifier/src/stages/stage8/precommitted.rs` — existing final
  precommitted opening resolver to reuse.
- `crates/jolt-prover/src/akita/witness.rs` and
  `crates/jolt-prover-legacy/src/zkvm/packed.rs` — modular and legacy object
  constructors to migrate.
