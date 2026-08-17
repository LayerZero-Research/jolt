# SHA2-chain: Akita trusted advice batched with the main trace

This run measures the Phase-1 precommitted-group protocol: trusted advice is
committed independently as one dense `(20, 1)` group, the packed main trace is
committed later as the contextual final `(39, 1)` group, and the two openings
are proved and verified together. Untrusted advice is not present in this
benchmark.

| Operation | Advice only | Main packed polynomial | Total |
|---|---:|---:|---:|
| Commit | 0.024482541 s | 15.899900834 s | 15.924383375 s |
| Prove | — (fused) | — (fused) | 30.277204624 s |
| Verify | — (fused) | — (fused) | 0.045383959 s |
| **Commit + prove + verify** | — | — | **46.246971958 s** |

The dashes are intentional. There is one Akita proof and one Akita
verification for `[TrustedAdvice, OneHotTrace]`; after batching, no exact
per-group advice/main prove or verify interval exists. The commit intervals
remain separate and additive because the two commitments remain separate.

The total prove row is the complete `jolt_prover::prove` interval with the
main commitment removed. It includes the Jolt PIOP, retained advice
reductions, and the one joint PCS opening. The verify row is the complete Jolt
verifier, including transcript and sumcheck checks around the single joint PCS
verification. Setup is reported separately below and is not included in these
totals.

## Exact PCS boundaries

| PCS operation | Advice | Main | Joint/total |
|---|---:|---:|---:|
| Independent/contextual commit | 0.024482541 s | 15.899900834 s | 15.924383375 s |
| Joint opening prove | — | — | 7.525781750 s |
| Joint opening verify | — | — | 0.039599708 s |

The joint wrapper spans include statement validation, transcript binding, and
backend dispatch. Inside them, the raw Akita `batched_prove` and
`batched_verify` calls took 7.515862333 s and 0.039409916 s, respectively.
There was exactly one backend prove call and one backend verify call for the
trusted/main groups.

## Main-only paired run

The same final binary was also run without advice:

| Operation | Main only |
|---|---:|
| Main commit | 16.300070625 s |
| Prove after removing main commit | 31.832814083 s |
| Full verify | 0.043234875 s |
| **Commit + prove + verify** | **48.176119583 s** |
| Raw main PCS prove | 7.731440667 s |
| Raw main PCS verify | 0.039917625 s |

The raw with-advice minus no-advice end-to-end delta is **-1.929147625 s**.
This is not an “advice cost”: the grouped run uses a different,
precommit-conditioned final schedule, and a single paired run also contains
ordinary system noise. Likewise, the joint PCS prove being 0.215578334 s
faster than the scalar main PCS prove must not be attributed to advice alone.

## Comparison with the separate-dense baseline

The baseline is the dense-advice rerun in
`sha2_chain_akita_trusted_advice_2pow20_words_nv26_20260813.md`, where trusted
advice and main used separate PCS proofs and verifications.

| Metric | Separate dense PCS | Grouped trusted + main | Change |
|---|---:|---:|---:|
| Commit | 16.259814167 s | 15.924383375 s | -0.335430792 s (-2.06%) |
| Prove | 30.107731666 s | 30.277204624 s | +0.169472958 s (+0.56%) |
| Verify | 0.046560250 s | 0.045383959 s | -0.001176291 s (-2.53%) |
| Commit + prove + verify | 46.414106083 s | 46.246971958 s | -0.167134125 s (-0.36%) |
| Proof payload | 189,048 bytes | 107,777 bytes | -81,271 bytes (-42.99%) |
| Peak RSS | 27.21 GiB | 28.40 GiB | +1.19 GiB (+4.37%) |

The timing changes are within the noise visible in the paired run. The clear
structural result is proof size: batching removes the standalone dense-advice
opening proof. The grouped proof is only 526 bytes larger than the 107,251-byte
no-advice proof, a 0.49% overhead.

For reference, compared with the original byte-one-hot advice run, the grouped
dense protocol reduces total time from 63.708256 s to 46.246972 s (-27.41%)
and proof payload from 194,232 bytes to 107,777 bytes (-44.51%).

## Setup and schedule identity

| Setup | Time |
|---|---:|
| Grouped-capacity main setup | 0.694358 s |
| Standalone dense trusted-advice setup | 0.542875 s |
| **Total setup** | **1.237233 s** |

Setup is transparent and amortizable; it is excluded from the operation table.
The no-advice paired run's grouped-capacity main setup took 0.750699 s.

- Setup-seed digest: `20750283dc25a4d22e0a8220ad9d0ff8bd4f54726741d1aa86bc91193a1dc688`
- K256 catalog digest: `d9008b3797464b14967a2521083a42107f4513ec74df604cbfda450ca209f5ac`
- Grouped `[trusted (20,1), main (39,1)]` row selection: `0509f22f85f3631b0b0881890a335a5b2646125875442790f53b7465d876104c`
- Main-only `(39,1)` row selection: `05abee25e3c65e027b294edaa4d5f290f5057e217b8be05c211f0fc32dac0d85`

The catalog digest is Blake2b-256 over the configured catalog identity and
every exact cryptographic row-selection digest. The grouped row selection is
proof-carried, transcript-bound, and resolved only against this configured
catalog.

## Workload and artifacts

- Date: 2026-08-14 (PDT)
- Code base: `cebacf912` plus the uncommitted batching implementation described here
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Backend: modular optimized Akita, K=256
- Input: 35 encoded bytes
- Padded trace: 67,108,864 rows (`2^26`)
- Trusted advice: 8,388,608 bytes (`2^23`) = 1,048,576 `u64` words (`2^20`)
- Grouped proof payload: 107,777 bytes
- Peak RSS: 28.40 GiB

Commands:

```text
cargo run --release -p jolt-prover --example modular_benchmark \
  --features prover-fixtures,akita -- \
  --name sha2-chain --scale 26 --backend optimized --format chrome

cargo run --release -p jolt-prover --example modular_benchmark \
  --features prover-fixtures,akita -- \
  --name sha2-chain --scale 26 --backend optimized --format chrome \
  --trusted-advice-bytes 8388608
```

Generated artifacts:

- `benchmark-runs/results/akita_sha2-chain_26_optimized_no_advice.csv`
- `benchmark-runs/results/akita_sha2-chain_26_optimized_trusted_advice_8388608.csv`
- `benchmark-runs/perfetto_traces/akita_sha2_chain_26_optimized_no_advice.json`
- `benchmark-runs/perfetto_traces/akita_sha2_chain_26_optimized_trusted_advice_8388608.json`
