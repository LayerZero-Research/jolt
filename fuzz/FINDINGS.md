# Findings

Findings of the Jolt + Akita liveness campaign on Jolt `main` (Akita pinned at
`252abb89`). Each entry states what was observed, whether the input is inside
the documented contract, how it was verified, and its status. Severity:
**High** (a supported statement cannot be proved, or soundness), **Medium**
(a documented-but-unbounded failure mode, or a public-API completeness gap
production does not reach), **Low** (limits far outside practical use, or
documentation).

"In contract" means every documented Jolt limit admits the input:
`log_T` in `12..=24` (K=16) or `12..=30` (K=256), advice capacities a power of
two with physical arity at most 34 (`ADVICE_MAX_PHYSICAL_VARS`), committed
programs with a power-of-two chunk count at most 256
(`MAX_COMMITTED_BYTECODE_CHUNK_COUNT`) and chunk/image arities at most 34
(`DIRECT_PROGRAM_MAX_PHYSICAL_VARS`). See `Shape::in_contract`.

Reproduce sweeps with (release, on the campaign host):

```bash
cd fuzz && cargo run --release -p jolt-fuzz-dev -- plan-sweep all plan.csv
```

## J-1 (High, liveness): committed programs with more than a few bytecode chunks cannot be preprocessed

Jolt documents committed programs with up to 256 bytecode chunks, but grouped
Akita planning (`provision_precommitted_for_k` -> `find_adapted_schedule`)
fails for most chunk counts:

- `Unsupported proof schedule: adapted precommit opening domain exceeds the
  maximum of 256 assignments` (21 713 of 38 232 swept committed shapes);
- `adapted planning supports at most 256 precommitted producers, got N` for
  256 chunks (256 chunks + the program image, plus advice, exceed
  `MAX_ADAPTED_PRECOMMIT_WIDTH = 256`; 3 672 shapes).

At `log_T = 12`, K=16 (identical at every other trace size), the committed
shapes that plan are:

| Advice | 1 chunk | 2 chunks | 4 chunks | 8 chunks | 16..128 chunks | 256 chunks |
|---|---|---|---|---|---|---|
| none | plans | plans | plans | only with image <= 8193 words and bytecode <= `2^8` (one isolated pass at bytecode `2^20`, image `2^26`) | fails (assignments) | fails (producers) |
| 4 KiB trusted + 4 KiB untrusted (SDK default) | plans | image <= 8193 words (<= `2^20` at bytecode `2^12`); never at bytecode `2^24` | only at bytecode `2^4`, image <= 8193 words | fails | fails | fails |

(Bytecode lengths swept: `2^4, 2^8, 2^12, 2^16, 2^20, 2^24`; image words
`1, 2^13, 2^13+1, 2^20, 2^26, 2^34`.)

Real guests reproduce it: the `program` lane fails `sha3-chain` committed with
8 chunks, and `interp-advice-large` committed with 2 chunks (large advice
capacities), at preprocessing:

```text
preprocess: Openings(InvalidSetup("Invalid or missing setup file: Unsupported
proof schedule: adapted precommit opening domain exceeds the maximum of 256
assignments"))
```

Cause: Akita enumerates the Cartesian product of per-group precommit-opening
assignments (with identical groups collapsed into multisets,
`packing_precommit_opening_products`,
`akita-planner/src/schedule_params/suffix_dp/candidates.rs`) and caps it at
256. Every bytecode chunk and each advice object is one group, so the product
grows combinatorially with the group count. The same code is on Akita `main`
(`candidates.rs:139`).

Status: open. The fix is either Jolt-side (bound the chunk count, or the
chunk-plus-advice group count, to what planning admits, and document it) or
Akita-side (a planner search that does not enumerate the product). Awaiting a
decision.

## J-2 (High, liveness): advice larger than the trace group fails at proving

When an advice object's physical arity exceeds the packed trace's final arity
(for example the smallest trace, `log_T = 12`, K=16, final arity 22, with a
trusted-advice capacity of 64 MiB, arity 23), preprocessing succeeds but the
stage-0 trace commitment fails:

```text
prove: Verifier(FinalOpeningVerificationFailed { reason: "commitment failed:
Invalid or missing setup file: schedule requires 781568 physical setup field
elements, but setup provides 264192" })
```

Verified end to end with the `interp-advice-large` guest (64 MiB trusted,
1 MiB untrusted capacity) and a near-empty program, and with
`jolt-fuzz-dev planning-case 12 p 12 20 - 26`. The same guest with a longer
trace (final arity above 23) proves and verifies.

Cause: `jolt_prover::akita::preprocessing::grouped_setup` passes the trace's
exact final arity as the Akita setup's `max_num_vars`. Akita sizes the setup
from the catalog rows whose groups all fit `max_num_vars`
(`SetupRequirements::from_catalog`), so a grouped row with a larger
precommitted group contributes nothing to the setup. `jolt-akita` also rejects
such statements outright (`native_batching.rs`, "arity exceeds grouped setup
capacity"). The setup capacity and the exact final dimension are the same
field (`AkitaSetupParams::max_num_vars`).

Status: fix written, not yet tested: branch `fix/akita-grouped-setup-capacity`
(off `main`). The one-hot backend setup capacity becomes the final arity
raised to the largest precommitted group in the setup's exact catalog
(`AkitaVerifierSetup::one_hot_backend_num_vars`), used by setup, verifier key
re-derivation after transport, and the grouped-statement arity check. Its test
(`crates/jolt-akita/tests/grouped_capacity.rs`: final arity 16, trusted advice
arity 22, in process and serde-transported) must run on the campaign host
before a PR opens.

## J-3 (Medium, liveness): large advice fails to plan on setup-offloaded trace rows

Traces with `log_T >= 21` use setup-offloaded schedules. With advice arities
from about 28 upward (capacities of 2 GiB and more), grouped planning fails:

```text
Unsupported proof schedule: no multi-group schedule in the audited fold domain for num_vars=N
```

(3 593 of 73 872 swept advice shapes; every failure is at `log_T >= 21`; every
direct row, `log_T <= 20`, plans every advice arity up to 34.) The smallest
failing arity per row is 28 to 31 depending on `log_T` and on the pair of
advice arities. The README documents that grouped preprocessing "fails closed"
when the frozen skeleton cannot admit the profiles, but not where; the
documented advice limit is 34.

Status: open (documented failure mode, undocumented boundary). Awaiting a
decision.

## J-4 (Low, liveness): the packed trace's selector capacity bounds bytecode and RAM together

`OneHotTrace` has 64 selector columns at K=16 and 32 at K=256. Large bytecode
and RAM domains need more RA columns than that, and Jolt's own trace geometry
(`one_hot_trace_setup_shape`) rejects the shape:

```text
final opening batch construction failed: invalid batch opening: OneHotTrace has N columns, exceeding the K=N^N packed capacity N
```

Independent of `log_T`. Smallest failing RAM domain by bytecode length:

| K | Bytecode | Fails from `ram_K` |
|---|---|---|
| 16 | `2^21..2^24` | `2^37` |
| 16 | `2^25`, `2^26` | `2^33` |
| 256 | `2^17..2^24` | `2^33` |
| 256 | `2^25`, `2^26` | `2^25` |

`ram_K` counts 8-byte words, so `2^33` is a 64 GiB address span; default
layouts use about `2^22`. Not documented as a limit.

Status: open (limit far outside ordinary layouts).

## J-5 (Medium, under triage): openings of an identically zero polynomial fail verification

An honest opening whose committed polynomial is identically zero fails
verification with `Akita payload has trailing bytes after deserialization`:

- every dense-bounded catalog row (14..22 variables, 1 and 2 polynomials)
  through `AkitaNativeBatching`'s same-point batch;
- one-hot rows through `commit_trace_one_hot` + `prove_batch` with no
  precommitted groups when every column is empty.

The backend proof is longer than the canonical shape the verifier derives from
the schedule (56 153 bytes produced, 41 977 consumed for dense 14:1). Random
and maximal witnesses of the same rows verify. Production's grouped opening
with an all-zero advice object verifies (the `advice-consumer` guest with zero
advice), and a production `OneHotTrace` is never empty (RAM columns commit
every row), so the Jolt prover does not reach this; the affected entry points
are public `jolt-akita` APIs.

Status: under triage. Being re-checked against Akita `main`, whose proof
stream replaced the typed proof layer the pinned revision uses.

## Harness corrections (not product defects)

- The reference prover backend refuses shapes whose dense address-by-cycle
  grid exceeds 32 GiB ("a test oracle sized for small traces"); the `program`
  target now selects it only for small shapes.
- Committed programs whose chunk arity exceeds 34 are outside the documented
  contract (`DIRECT_PROGRAM_MAX_PHYSICAL_VARS`); `Shape::in_contract` now says
  so.
- A reference trace commitment without precommitted groups must use
  `commit_one_hot_group_owned`; the `_with_precommitted` variant requires at
  least one group.
- One-hot selected rows were reduced modulo `k as u8`, which is 0 for K=256.
