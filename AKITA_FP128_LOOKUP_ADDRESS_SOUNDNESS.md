# Soundness break: the instruction-lookup address is not injective over the Akita fp128 field

| | |
|---|---|
| **Component** | `jolt-prover-legacy` instruction lookups (`akita` feature) |
| **Severity** | Critical — arbitrary corruption of an accepted execution trace |
| **Affected** | Any proof produced over the Akita fp128 field (`--features akita`) |
| **Not affected** | The BN254 / Dory and HyperKZG paths |
| **Status** | **Confirmed by a working end-to-end exploit on `main` (`3ab638dd7`).** The unmodified verifier accepts two proofs for the same program and the same inputs with different public outputs. |
| **Reproduce** | `cargo nextest run -p jolt-prover-legacy --features host,akita,fp128-forgery-poc -E 'test(fp128_alias)' --no-capture` |

---

## 1. Summary

Jolt's instruction-lookup argument commits a **128-bit lookup address** `k` per cycle, as
`instruction_d` one-hot chunks. The only constraint tying that committed address to the
instruction's actual arithmetic is the read-address (RAF) leg,

```
Σ_k ra(k) · Identity(k)  =  RightLookupOperand
```

and `Identity` is the identity multilinear extension, whose value at a Boolean point is
literally the integer `k` **reduced mod p**:

```rust
// crates/jolt-prover-legacy/src/poly/identity_poly.rs
(0..len).map(|i| r[i].into().mul_u128(1 << (len - 1 - i))).sum()
```

So the address is pinned only up to its residue class. This is sound **if and only if the
map `[0, 2^LOG_K) → F_p` is injective**, i.e. `p ≥ 2^LOG_K`. With `LOG_K = XLEN · 2 = 128`
that means `p ≥ 2^128`.

BN254's scalar field satisfies this with 126 bits to spare. **The Akita fp128 field does
not:**

```
p = 2^128 − 2^32 + 22537        (exactly 128 bits, but strictly below 2^128)
2^128 − p = 4,294,944,759       ≈ 2^32 aliasing addresses
```

Every honest lookup index `s < 4,294,944,759` therefore has a **second committable
preimage** `s + p`, indistinguishable from `s` by every check in the protocol — while the
lookup *table* reads only the low `XLEN` address bits and so returns a different value. A
malicious prover swaps the address, keeps the operand sum honest, and the verifier accepts
an execution in which `ADD` returned `x + y − (2^128 − p)` instead of `x + y`.

The near-miss is stark: soundness needs `p ≥ 2^128`, and `p` falls short by exactly
`4,294,944,759` — a relative shortfall of about `2^-96`.

---

## 2. Provenance: what is old, what is new

This matters for triage, so it is stated precisely.

| Ingredient | Where it comes from |
|---|---|
| The committed lookup address spans `[0, 2^128)` as chunked one-hot `ra` | Inherited a16z design. `read_raf_checking.rs` predates all lattice work (`efa345165`, 2025-07-29); `IdentityPolynomial` dates to `e3aea366b` (2023-06-29). Last touched only by the `072998fb9` refactor (2026-06-24). |
| The address is tied to arithmetic **only** through `Identity(k) = k mod p` | Same — inherited, unchanged. |
| **`p < 2^128`** | `04d39d602` — *feat: Akita lattice PCS integration* (#1676, 2026-07-23), which introduced `crates/jolt-prover-legacy/src/field/akita.rs`. |

So the *mechanism* is old and is not a defect on its own: over BN254 it is perfectly
sound, because the field is far larger than the address domain. What the mechanism carries
is an **undocumented, unasserted precondition** — `p ≥ 2^(2·XLEN)` — that nothing in the
codebase states or checks. Introducing a 128-bit field silently violated it.

Two things this is **not**:

- **It is not caused by batching, one-hot packing, or the fused multi-group opening.** This
  branch's baseline (`main` at `3ab638dd7`) contains none of the fused multi-group opening
  machinery — no `fused_stage8_open_eligible`, no `AKITA_MIN_MULTI_GROUP_VARS`, no
  `multi_group_prove_one_hot` — and the exploit reproduces there verbatim. The chunked
  one-hot address encoding it exploits is the inherited design, not an artefact of packing
  claims together.
- **It is not a challenge-space or Schwartz–Zippel issue.** No sumcheck is fooled and no
  low-probability event is needed. The forged witness satisfies every constraint
  *exactly*, with probability 1.

The commit that introduced the fp128 field also introduced the packed prover, which is why
the two are easy to conflate. The field is the cause; the packing is a bystander.

---

## 3. Background: three facts that combine

**(a) The address domain is `2^128`, and that is not reducible.** `LOG_K = XLEN * 2 = 128`
because two 64-bit operands must be addressable jointly. Interleaved-operand tables
(`AND`, `OR`, `XOR`, comparisons) index by `interleave_bits(x, y) -> u128` and their MLEs
read `r[2i]`/`r[2i+1]` as the two operands' bits; `MUL`'s index is the full product `x·y`,
up to `(2^64 − 1)^2`. So the domain genuinely needs 128 variables. **The fix cannot shrink
the domain — it has to repair the tie between address and arithmetic.**

**(b) For non-interleaved instructions the RAF leg uses full-width `Identity`.** For
`AddOperands`, `SubtractOperands`, `MultiplyOperands` and advice,
`is_interleaved_operands()` is false, and the RAF term is `γ · Identity(k)` over all 128
variables. (The interleaved path decomposes into two 64-bit halves and *is* injective on
`[0, 2^128)`; it is not the vulnerable path.)

**(c) The table MLE ignores the high address half.** `RangeCheckTable` — the table `ADD`
selects — is

```rust
// crates/jolt-prover-legacy/src/zkvm/lookup_table/range_check.rs
fn materialize_entry(&self, index: u128) -> u64 { index as u64 }          // low XLEN bits
fn evaluate_mle(&self, r: &[F]) -> F { /* reads only r[XLEN..2*XLEN] */ }
```

So the *read* leg distinguishes `s` from `s + p` while the *address* leg cannot. That
asymmetry is the exploit.

---

## 4. The invariant that breaks

The protocol's implicit claim is:

> The committed one-hot address determines a unique integer in `[0, 2^LOG_K)`, and the RAF
> leg pins that integer to the instruction's operands.

The second clause holds only up to residues. Injectivity of `[0, 2^LOG_K) → F_p` requires
`p ≥ 2^LOG_K`. Over fp128 the map is `4,294,944,759`-to-... precisely, `2^128 − p` values
have exactly two preimages each and the rest have one (there is no third, since `2p >
2^128`).

### 4.1 Objection: isn't the field large enough to hold every honest value?

This is the natural first reaction, and it is the wrong comparison. The honest lookup index
for `ADD` is at most `2^65`, comfortably inside a 128-bit field — so "the field can hold
every honest value" is true and irrelevant.

The attack does not need a value the field cannot hold. It needs **a second preimage inside
the committed domain**, and the committed domain is `2^128` cells — *larger than the
field*. The prover is free to place its one-hot `ra` on any of those cells. The relevant
comparison is therefore:

> **field size vs. DOMAIN size — not field size vs. value size.**

The domain is fixed combinatorially by the protocol (`LOG_K` address variables, hence
`2^LOG_K` cells) and does not grow with the field. Concretely:

| | address variables available | `k + p` needs | aliases in `[0, 2^128)` |
|---|---|---|---|
| BN254 Fr | 128 | 254 | **0** |
| Akita fp128 | 128 | 128 | **4,294,944,759** |

Over BN254 the forgery is not *rejected* so much as **inexpressible**: committing
`k + p_bn254 ≈ 2^254` would need 254 address variables and only 128 exist. Making a field
*larger* never helps an attacker here; the surplus is unaddressable, and that surplus is
exactly the safety margin.

A related non-issue, for completeness: field *elements* have no representation freedom —
`s` and `s + p` are literally the same element, not a collision. The "two objects, one
field image" pattern requires a pre-field domain with more members than the field, where
the protocol pins only the image. The one-hot address is such a domain; field-valued
witnesses like `RightLookupOperand` are not.

---

## 5. Why no other check catches it

| Check | Why the aliased address survives |
|---|---|
| One-hot **booleanity** (`B² − B`) | `s + p` is an ordinary 128-bit integer; each chunk's hot lane is a legal lane index. Nothing is out of range. |
| **Hamming weight 1** per chunk | Exactly one hot lane per chunk, as required. |
| **RAF leg** `Σ_k ra(k)·Identity(k)` | `Identity(s + p) = Identity(s)` in `F_p`. Identical claim. |
| R1CS `RightLookupOperand == x + y` | `to_lookup_operands()` is left **honest** — only the committed *address* moves — so this constraint is untouched. |
| **Read leg** / `RdWrite == LookupOutput` | The prover claims the table entry at the *forged* address, `low64(s + p)`, and the emulator writes that same value to `rd`. Consistent. |
| Register / RAM read-write checking | The whole trace is fabricated around the corrupted `rd`, so every later read agrees. |
| Range checks / canonicity | **None exist** on the committed address. Nothing restricts it to `[0, p)`. |
| Commitment binding, Fiat–Shamir | Irrelevant. The prover commits honestly to a witness it is *entitled* to choose; no binding is broken. |

There is no probabilistic step. The forged witness satisfies the constraint system exactly.

---

## 6. Worked example (from the committed test)

Guest — the entire program under attack is one register-register `ADD` whose result is the
public output:

```rust
// examples/alias-forgery/guest/src/lib.rs
#[jolt::provable(heap_size = 32768, max_trace_length = 65536)]
fn settle(debit: u64, credit: u64) -> u64 {
    debit.wrapping_add(credit)
}
```

Inputs `debit = 1_234_567_891`, `credit = 2_000_000_011`. Honest lookup index
`s = 3_234_567_902`, inside the alias window. Both operands are below `RAM_START_ADDRESS`
(`0x8000_0000`) and far from any loop counter, so the pair cannot collide with pointer
arithmetic elsewhere in the trace.

| | honest | forged |
|---|---|---|
| committed address | `0x…c0cb_eee7` (= `s`) | `0xffff_ffff_ffff_ffff_ffff_ffff_c0cb_eee7` (= `s + p`) |
| `Identity(addr)` | `3234567902` | **`3234567902`** — same field element |
| `RightLookupOperand` | `3234567902` | `3234567902` — untouched |
| table entry (low 64 bits) | `3234567902` | `18446744072649174759` |
| `rd` | `3234567902` | `18446744072649174759` |
| verifier | **accept** | **accept** |

The corruption is exactly `−(2^128 − p) mod 2^64`. Because `ADD`'s honest index is at most
`2^65`, the attacker can shift *any* `ADD`/`SUB`/advice result whose untruncated operand
sum lands below `2^32`-ish — which in practice is most integer arithmetic in a real guest.

---

## 7. The exploit in this branch

All exploit code sits behind the `fp128-forgery-poc` cargo feature, which no default,
release, or CI configuration enables.

| File | Role |
|---|---|
| `examples/alias-forgery/guest/` | the target guest (above) |
| `tracer/src/instruction/fp128_forgery.rs` | forgery parameters, runtime arming flag, corrupted-cycle counter |
| `tracer/src/instruction/add.rs` | for the target cycle, writes `low64(s + p)` to `rd`, so registers/RAM/outputs are all self-consistent with the corrupted value |
| `crates/jolt-prover-legacy/src/zkvm/instruction/add.rs` | `to_lookup_index()` returns `s + p`; `to_lookup_operands()` stays honest; `to_lookup_output()` returns the aliased table entry |
| `crates/jolt-prover-legacy/src/zkvm/fp128_forgery_poc.rs` | the driver: proves and verifies the same guest on the same inputs, honestly and forged |

**Nothing in `jolt-verifier` is touched.** The hook only changes witness values a prover is
free to choose. A real attacker fabricates the trace directly; driving the emulator is
merely the cheapest way to produce the same bytes.

### The break

```
$ cargo nextest run -p jolt-prover-legacy \
    --features host,akita,fp128-forgery-poc -E 'test(fp128_alias)' --no-capture

  fp128 lookup-address alias — end-to-end forgery [fp128 / Akita]
  guest                  : alias-forgery-guest  (settle(debit, credit) -> debit + credit)
  inputs                 : debit=1234567891, credit=2000000011
  alias window (2^128-p) : 4294944759
  honest : output=3234567902           accepted=true  forged_cycles=0
  forged : output=18446744072649174759 accepted=true  forged_cycles=1
```

Both proofs accepted, same program, same inputs, **different public outputs**, differing by
exactly `2^128 − p`. The test asserts the honest baseline forged nothing
(`forged_cycles = 0`) and the forged run corrupted exactly one cycle
(`forged_cycles = 1`), so it cannot pass by silently measuring two honest runs.

### The BN254 control

Same guest, same hook, same corrupted trace — the only variable is the field:

```
$ cargo nextest run -p jolt-prover-legacy \
    --features host,fp128-forgery-poc -E 'test(fp128_alias)' --no-capture

  fp128 lookup-address alias — end-to-end forgery [BN254 / Dory (control)]
  honest : output=3234567902           accepted=true  forged_cycles=0
  forged : output=18446744072649174759 accepted=false forged_cycles=1
  forged rejected with   : StageClaimOutputMismatch { stage: 5 }
```

Stage 5 is the instruction read-RAF sumcheck. Over BN254, `s + p_fp128` is a genuinely
different field element, so the RAF leg no longer matches `RightLookupOperand`. **The check
that fp128 lets slip is the same check that catches it on BN254** — which localises the
cause to the modulus and exonerates the encoding, the packing, and the opening.

A third test, `fp128_alias_window_is_a_field_property`, states the adoption criterion
directly and reads BN254's modulus off `ark_bn254::Fr` rather than hardcoding it, so the
control's premise is checked rather than asserted.

---

## 8. Remediation

The domain cannot shrink (§3a), so the tie between address and arithmetic must be repaired.
Options, in increasing order of robustness:

**Option 1 — range-check the committed address (partial; NOT sufficient alone).**
Constrain the top address bits to zero for the affected instructions. Note the honest
ranges differ per instruction:

| Instruction class | honest index range | top bits provably zero? |
|---|---|---|
| `ADD` / `SUB` / advice | `< 2^65` | yes — 63 high bits |
| `MUL` family | up to `(2^64 − 1)^2`, i.e. `< 2^128` | **no** |

So a `< 2^65` range check fixes `ADD`/`SUB`/advice but does nothing for `MUL`, whose honest
index legitimately fills the domain. Any fix that only bounds the address leaves `MUL`
exposed.

**Option 2 — limb-split the RAF binding (preferred).** Replace the single full-width
`Identity` term with two 64-bit limbs, `Identity_hi` and `Identity_lo`, each bound
separately, and reconstruct `hi · 2^64 + lo` in R1CS. Each limb ranges over `[0, 2^64)`,
which *is* injective into fp128, so the address is pinned exactly — for `MUL` as well. This
is the same decomposition the interleaved path already uses, generalised to the
non-interleaved one.

**Option 3 — require `p ≥ 2^(2·XLEN)`.** Assert this as an explicit, tested backend
precondition. Note the check is on the modulus **value**, not its bit length: fp128's
modulus is *exactly* 128 bits and still fails, so "128-bit field, 128-bit address domain"
is precisely the broken case. `fp128_alias_window_is_a_field_property` encodes this check.
On its own it disqualifies fp128 rather than fixing it, so it is best paired with Option 2
as a guard against future backends.

Whatever is chosen, the precondition should be documented at the definition of `LOG_K` and
at `IdentityPolynomial`, since that is where a future reader will need it.

---

## 9. Verification status

**Verified by reading the source and confirmed numerically:**

- `LOG_K = XLEN * 2 = 128`, `XLEN = 64`.
- `p = 2^128 − 2^32 + 22537`, exactly 128 bits, so `2^128 > p`; alias window `4,294,944,759`.
- `IdentityPolynomial::evaluate` computes `Σ r[i]·2^(len−1−i)`, i.e. `k mod p` at Boolean points.
- `is_interleaved_operands()` is false for `AddOperands` / `SubtractOperands` / `MultiplyOperands` / `Advice`, routing them to the `γ · Identity` RAF term.
- `ADD::to_lookup_index()` is the untruncated 65-bit sum; `RangeCheckTable::materialize_entry` truncates to the low `XLEN` bits; `RangeCheckTable::evaluate_mle` reads only `r[XLEN..2*XLEN]`.
- No booleanity, Hamming-weight, range, or canonicity check restricts the committed address to `[0, p)`.

**Demonstrated end to end**, on `main` at `3ab638dd7`, with a dedicated guest program and a
BN254 control (§7).

**Falsification target for reviewers:** none remaining at the protocol level — the exploit
is realised. What is still worth review is whether the hook is a faithful model of a
malicious prover, i.e. that it only changes witness values a prover controls and never
weakens a verifier check.

---

## Appendix — code reference index

| Claim | Location |
|---|---|
| `LOG_K = XLEN * 2` | `crates/jolt-prover-legacy/src/zkvm/instruction_lookups/mod.rs` |
| `XLEN = 64` | `common/src/constants.rs` |
| Field modulus | `crates/jolt-prover-legacy/src/field/akita.rs` |
| `Identity(k) = k mod p` | `crates/jolt-prover-legacy/src/poly/identity_poly.rs` |
| Interleaved-path halves (safe) | `crates/jolt-prover-legacy/src/poly/identity_poly.rs` |
| RAF leg definition | `crates/jolt-prover-legacy/src/zkvm/instruction_lookups/read_raf_checking.rs` |
| Prover-side identity/operand split | `crates/jolt-prover-legacy/src/poly/prefix_suffix.rs::init_Q_raf` |
| `raf_flag` instruction set | `crates/jolt-prover-legacy/src/zkvm/instruction/mod.rs` |
| ADD lookup index = untruncated sum | `crates/jolt-prover-legacy/src/zkvm/instruction/add.rs` |
| ADD selects `RangeCheckTable` | `crates/jolt-prover-legacy/src/zkvm/instruction/add.rs` |
| Table truncates / MLE ignores high half | `crates/jolt-prover-legacy/src/zkvm/lookup_table/range_check.rs` |
| `right_lookup` is `u128` | `crates/jolt-prover-legacy/src/zkvm/r1cs/inputs.rs` |
