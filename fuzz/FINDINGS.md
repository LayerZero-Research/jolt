# Findings

Findings of the Jolt + Akita liveness campaign on Jolt `main` (Akita pinned at
`252abb89`). Each entry states what was observed, whether the input is inside
the documented contract, how it was verified, and its status. Severity:
**High** (a supported statement cannot be proved, or soundness), **Medium**
(a documented-but-unbounded failure mode, or a public-API completeness gap
production does not reach), **Low** (limits far outside practical use, or
documentation).

Entries are numbered in discovery order. Triage order also weighs who can
trigger a finding:

| Priority | Finding | Severity | Triggered by | Fix |
|---|---|---|---|---|
| 1 | J-2 | High | Runtime inputs: an execution short enough to pad to a trace below the advice arity, on a guest with a large advice capacity | LayerZero-Research/jolt#49 |
| 2 | J-1 | High | Deployment choices only (chunk count, program size, advice kinds); fails at preprocessing, before any proof | LayerZero-Labs/akita#119 |
| 3 | J-3 | Medium | Deployment choices only (advice capacity at `log_T >= 21`) | none (by design) |
| 4 | J-4 | Low | Deployment choices only (very large bytecode and RAM domains) | none (by design) |
| – | J-5 | Medium | Not reachable from the Jolt prover; public `jolt-akita` APIs only, pinned Akita only | Akita bump |
| – | J-6 | Medium | Crafted verifier-preprocessing bytes (the schedule catalog they carry); not proof bytes | Akita #91 (Akita F-5); needs a pin at or after `cc1042c2` |

None of them lets a proof of a false statement verify; each fails with a clean
error.

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

Status: fix in Akita draft PR https://github.com/LayerZero-Labs/akita/pull/119
(branch `fix/adapted-precommit-uniform-fallback`, off Akita `main`): when the
multiset product exceeds the budget the adapted root searches one opening per
class of interchangeable groups, and `MAX_ADAPTED_PRECOMMIT_WIDTH` becomes 512
(a 256-chunk program with its image and both advice objects needs 259). Its
test adapts the full width of interchangeable producers; `akita-planner` and the
Akita workspace (CI feature set) pass. Jolt picks it up with its Akita bump;
re-run `plan-sweep program` then.

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

Status: draft PR https://github.com/LayerZero-Research/jolt/pull/49 (branch
`fix/akita-grouped-setup-capacity`, off `main`). The one-hot backend setup
capacity becomes the final arity raised to the largest precommitted group in
the setup's exact catalog (`AkitaVerifierSetup::one_hot_backend_num_vars`),
used by setup, verifier key re-derivation after transport, and the
grouped-statement arity check (consulted only for objects above the final
arity). Tested on the campaign host: `jolt-akita` 77/77 including the new
`grouped_capacity` test (final arity 16, trusted advice arity 22, in process and
serde-transported); `jolt-prover --features akita,prover-fixtures` 26/27, the
one failure being `e2e_matrix::akita::stdlib`, whose std-mode guest toolchain
cannot be built on that host (it passes in CI).

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

Status: by design, boundary undocumented. Adaptation freezes the selected
trace row's skeleton (recursive depth, per-level dimensions, and the
setup-offload topology) and fails closed when a precommitted group no longer
fits it, as the schedules README states; the setup field budget is not
involved (`setup_field_budget` is `None` for these configs). A very large
advice object enlarges the root output past what the frozen offloaded levels
admit. No code fix: the effective limit is the boundary above, and
`ADVICE_MAX_PHYSICAL_VARS = 34` overstates it for `log_T >= 21`.

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

Status: by design (the packed selector capacity is a fixed 64 or 32 columns);
the limit is undocumented but far outside ordinary layouts. No code fix.

## J-5 (Medium, pinned Akita only): openings of an identically zero polynomial fail verification

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

Status: does not reproduce on Akita `main`. Akita's own fuzzer, replaying its
all-zero inputs through `pcs_dense`, `pcs_onehot`, and `pcs_batch` (zero tables,
verified from the serialized proof bytes by `batched_verify(proof: &[u8], ..)`),
passes on `fuzzer` = `main` + fuzz. The typed proof layer the pinned revision
uses was replaced by the Spongefish proof stream (Akita #37, #67), which Jolt's
Akita bump (LayerZero-Research/jolt#39, pin `703d8580`) includes. No PR; until
the bump the harness replaces an identically zero final polynomial by one
nonzero entry and counts it (`known_zero_polynomial_adjusted`).

## J-6 (Medium, robustness, already fixed upstream): verifier panics on a crafted schedule catalog in the verifier preprocessing

Found by the `verifier` lane (campaign `dd8fbf74a114`, finding
`panic-verifier-a398686ab8d7`, reproducible). `AkitaVerifierSetup`
serializes the exact schedule catalog; a preprocessing whose catalog has a
zero `log_basis` makes the verifier panic in admission instead of returning an
error:

```text
panicked at akita-types/src/sis/decomposition_digits.rs:249:5: invalid log_basis
```

This is the Akita campaign's F-5 (second site), fixed upstream by Akita #91
(`cc1042c2`). Jolt `main` pins `252abb89`, before it, and so does the pin of
LayerZero-Research/jolt#39 (`703d8580`): the planned bump does not include the
fix; Jolt needs a pin at or after `cc1042c2`. Reachable only from verifier
preprocessing bytes, which deployments normally produce themselves; the effect
is a crash, never an accepted proof. No new PR (already addressed upstream).

## Operational note: endpoint detection on the campaign host (2026-09-28)

CrowdStrike Falcon on the development host killed shell commands that wrote
small binary fuzz inputs to `/tmp` with `printf "\x.."` and classified them as
malware staging. The campaign itself was not running. Fuzzing on that host is
paused until its security team confirms. Inputs are now produced only by
`jolt-fuzz-dev` into the campaign's own directories; a libFuzzer campaign
still writes many binary files, so its directories need the security team's
agreement before `run`.

## Harness corrections (not product defects)

- A `program@examples` input timed out under ASan (1 324 s against a
  1 200 s limit); it proves and verifies in 78 s in release. It used the
  reference prover backend (the naive test oracle), which the target now uses
  only for smaller shapes.

- The first campaign run reported three "malleability" findings in the
  `verifier` lane (non-canonical commitment and public-I/O bytes accepted).
  All three were bytes appended after a valid object, which the harness's
  decoder ignored; with trailing bytes rejected they replay cleanly.
- Seeds longer than a lane's `max_len` were stored untruncated, so a seed that
  hit a known finding was never quarantined (libFuzzer runs and names artifacts
  after the first `max_len` bytes) and its lane backed off instead of fuzzing.

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
