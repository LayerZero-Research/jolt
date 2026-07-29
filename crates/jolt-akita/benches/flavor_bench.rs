#![expect(
    clippy::expect_used,
    reason = "benchmarks should fail loudly if a setup or proof path is malformed"
)]
#![expect(clippy::print_stderr, reason = "the bench reports timings to stderr")]
#![expect(
    clippy::unimplemented,
    reason = "the bench stand-in exposes only the one-hot polynomial interface"
)]

//! Timed comparison of the two `OneHotTrace` commitment formats at production
//! shape: one sparse-unit union polynomial (`slots` slots x `2^(8+log_t)`
//! cells) versus one batched one-hot group (`slots` polynomials of `8+log_t`
//! variables each) — commit + batched open + verify for both.
//!
//! Not a Criterion harness: one iteration at trace shape already costs minutes,
//! and the interesting numbers are the individual phase timings rather than a
//! distribution. Run explicitly:
//!
//! ```text
//! cargo bench -p jolt-akita --bench flavor_bench
//! ```
//!
//! Environment knobs: `BENCH_LOG_T` and `BENCH_SLOTS` set the trace shape,
//! `BENCH_SKIP_ONEHOT` / `BENCH_SKIP_UNION` select one format, and
//! `BENCH_SETUP_SPLIT` (with `BENCH_VARS`, `BENCH_SLOTS`) instead reports the
//! per-flavor setup cost alone.

use std::time::Instant;

use akita_config::proof_optimized::fp128::D64Dense as AkitaDenseConfig;
use akita_pcs::AkitaCommitmentScheme;
use jolt_akita::configs::JoltD64OneHotK256;
use jolt_akita::{
    AkitaCommitment, AkitaField, AkitaNativeBatchPolynomials, AkitaNativeBatching, AkitaScheme,
    AkitaSetupParams,
};
use jolt_openings::{BatchOpeningScheme, CommitmentScheme, EvaluationClaim, VerifierOpeningClaim};
use jolt_poly::{MultilinearPoly, OneHotPolynomial};
use jolt_transcript::{Blake2bTranscript, Transcript};

type DenseBackendScheme = AkitaCommitmentScheme<AkitaDenseConfig>;
type OneHotK256BackendScheme = AkitaCommitmentScheme<JoltD64OneHotK256>;

const LOG_K: usize = 8;
const K: usize = 1 << LOG_K;

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Big-endian eq table: index bit of weight `2^(len-1-j)` pairs with
/// `point[j]`, matching Jolt's `MultilinearPoly::evaluate` convention (pinned
/// by `one_hot_evaluation_uses_big_endian_variable_order` in `src/scheme.rs`).
fn eq_table(point: &[AkitaField]) -> Vec<AkitaField> {
    let one = AkitaField::from_u64(1);
    let mut table = vec![one];
    for &p in point {
        let one_minus = one - p;
        let mut next = Vec::with_capacity(table.len() * 2);
        for &w in &table {
            next.push(w * one_minus);
            next.push(w * p);
        }
        table = next;
    }
    table
}

/// Split eq table: evaluating a sparse polynomial one hot index at a time
/// costs a single multiply instead of a full `2^n` table walk.
struct EqSplit {
    hi: Vec<AkitaField>,
    lo: Vec<AkitaField>,
    low_bits: usize,
    mask: usize,
}

impl EqSplit {
    fn new(point: &[AkitaField]) -> Self {
        let n = point.len();
        let low_bits = n / 2;
        Self {
            hi: eq_table(&point[..n - low_bits]),
            lo: eq_table(&point[n - low_bits..]),
            low_bits,
            mask: (1 << low_bits) - 1,
        }
    }

    fn weight(&self, index: usize) -> AkitaField {
        self.hi[index >> self.low_bits] * self.lo[index & self.mask]
    }
}

fn sparse_eval(poly: &dyn MultilinearPoly<AkitaField>, tables: &EqSplit) -> AkitaField {
    let mut acc = AkitaField::from_u64(0);
    poly.for_each_one(&mut |index| acc += tables.weight(index));
    acc
}

/// Bench stand-in for the packed union polynomial: unit-sparse over the
/// slot-prefixed cell domain, exposing only the one-hot interface the
/// sparse-unit commit path consumes.
struct UnionSparse {
    num_vars: usize,
    ones: Vec<usize>,
}

impl MultilinearPoly<AkitaField> for UnionSparse {
    fn num_vars(&self) -> usize {
        self.num_vars
    }

    fn evaluate(&self, point: &[AkitaField]) -> AkitaField {
        let tables = EqSplit::new(point);
        let mut acc = AkitaField::from_u64(0);
        for &one in &self.ones {
            acc += tables.weight(one);
        }
        acc
    }

    fn for_each_row(&self, _sigma: usize, _f: &mut dyn FnMut(usize, &[AkitaField])) {
        unimplemented!("bench union polynomial exposes only the one-hot interface")
    }

    fn is_one_hot(&self) -> bool {
        true
    }

    fn for_each_one(&self, f: &mut dyn FnMut(usize)) {
        for &one in &self.ones {
            f(one);
        }
    }
}

/// Setup cost alone, per backend flavor, at one shape.
fn setup_cost_split_by_flavor() {
    let num_vars = env_usize("BENCH_VARS", 28);
    let polys = env_usize("BENCH_SLOTS", 30);
    let start = Instant::now();
    let dense = DenseBackendScheme::setup_prover(num_vars, polys).expect("dense setup");
    eprintln!("dense setup ({num_vars},{polys}): {:.2?}", start.elapsed());
    drop(dense);
    let start = Instant::now();
    let one_hot = OneHotK256BackendScheme::setup_prover(num_vars, polys).expect("one-hot setup");
    eprintln!(
        "one-hot setup ({num_vars},{polys}): {:.2?}",
        start.elapsed()
    );
    drop(one_hot);
}

fn flavor_bench_sparse_union_vs_batched_one_hot() {
    let log_t = env_usize("BENCH_LOG_T", 20);
    let slots = env_usize("BENCH_SLOTS", 16);
    let t = 1usize << log_t;
    let cell_vars = LOG_K + log_t;
    let union_vars = cell_vars + slots.next_power_of_two().ilog2() as usize;
    let mut state = 0x1234_5678;

    // Per-slot hot lanes; the last slot mimics the msb column (lanes {0, 1}).
    let slot_indices: Vec<Vec<Option<u8>>> = (0..slots)
        .map(|slot| {
            (0..t)
                .map(|_| {
                    let r = splitmix(&mut state);
                    if slot == slots - 1 {
                        Some((r & 1) as u8)
                    } else {
                        Some((r & 0xFF) as u8)
                    }
                })
                .collect()
        })
        .collect();

    // Batched one-hot group.
    if std::env::var("BENCH_SKIP_ONEHOT").is_err() {
        let start = Instant::now();
        let (prover_setup, verifier_setup) =
            AkitaScheme::setup(AkitaSetupParams::one_hot_only(cell_vars, slots, [1; 32], K))
                .expect("one-hot setup");
        eprintln!(
            "one-hot setup ({cell_vars} vars, {slots} polys): {:.2?}",
            start.elapsed()
        );
        let polys: Vec<OneHotPolynomial> = slot_indices
            .iter()
            .map(|indices| OneHotPolynomial::new(K, indices.clone()))
            .collect();
        let start = Instant::now();
        let (commitment, hint) = AkitaScheme::commit_one_hot_group(&prover_setup, [2; 32], &polys)
            .expect("one-hot group commit");
        eprintln!("one-hot commit: {:.2?}", start.elapsed());

        let point: Vec<AkitaField> = (0..cell_vars)
            .map(|_| AkitaField::from_u64(splitmix(&mut state)))
            .collect();
        let tables = EqSplit::new(&point);
        let statement: Vec<VerifierOpeningClaim<AkitaField, AkitaCommitment>> = polys
            .iter()
            .map(|poly| VerifierOpeningClaim {
                commitment: commitment.clone(),
                evaluation: EvaluationClaim::new(point.clone(), sparse_eval(poly, &tables)),
            })
            .collect();
        let poly_refs: AkitaNativeBatchPolynomials<'_> = polys
            .iter()
            .map(|poly| poly as &dyn MultilinearPoly<AkitaField>)
            .collect();
        let mut prover_transcript = Blake2bTranscript::<AkitaField>::new(b"flavor-bench");
        let start = Instant::now();
        let proof = <AkitaNativeBatching as BatchOpeningScheme>::prove_batch(
            &prover_setup,
            statement.clone(),
            poly_refs,
            hint,
            &mut prover_transcript,
        )
        .expect("one-hot batched open");
        eprintln!("one-hot batched open: {:.2?}", start.elapsed());
        let mut verifier_transcript = Blake2bTranscript::<AkitaField>::new(b"flavor-bench");
        let start = Instant::now();
        <AkitaNativeBatching as BatchOpeningScheme>::verify_batch(
            &verifier_setup,
            &statement,
            &proof,
            &mut verifier_transcript,
        )
        .expect("one-hot verify");
        eprintln!("one-hot verify: {:.2?}", start.elapsed());
        assert_eq!(prover_transcript.state(), verifier_transcript.state());
    }

    if std::env::var("BENCH_SKIP_UNION").is_ok() {
        return;
    }

    // Sparse-unit union of the same content.
    let start = Instant::now();
    let (prover_setup, verifier_setup) =
        AkitaScheme::setup(AkitaSetupParams::new(union_vars, 1, [1; 32])).expect("union setup");
    eprintln!("union setup ({union_vars} vars): {:.2?}", start.elapsed());
    let mut ones = Vec::with_capacity(slots * t);
    for (slot, indices) in slot_indices.iter().enumerate() {
        for (cycle, &lane) in indices.iter().enumerate() {
            let lane = lane.expect("every bench slot cycle is hot") as usize;
            ones.push((slot << cell_vars) | (lane << log_t) | cycle);
        }
    }
    ones.sort_unstable();
    let union = UnionSparse {
        num_vars: union_vars,
        ones,
    };
    let start = Instant::now();
    let (commitment, hint) =
        <AkitaScheme as CommitmentScheme>::commit(&union, &prover_setup).expect("union commit");
    eprintln!("union commit: {:.2?}", start.elapsed());

    let point: Vec<AkitaField> = (0..union_vars)
        .map(|_| AkitaField::from_u64(splitmix(&mut state)))
        .collect();
    let value = union.evaluate(&point);
    let statement = vec![VerifierOpeningClaim {
        commitment: commitment.clone(),
        evaluation: EvaluationClaim::new(point.clone(), value),
    }];
    let mut prover_transcript = Blake2bTranscript::<AkitaField>::new(b"flavor-bench");
    let start = Instant::now();
    let proof = <AkitaNativeBatching as BatchOpeningScheme>::prove_batch(
        &prover_setup,
        statement.clone(),
        vec![&union as &dyn MultilinearPoly<AkitaField>],
        hint,
        &mut prover_transcript,
    )
    .expect("union open");
    eprintln!("union open: {:.2?}", start.elapsed());
    let mut verifier_transcript = Blake2bTranscript::<AkitaField>::new(b"flavor-bench");
    let start = Instant::now();
    <AkitaNativeBatching as BatchOpeningScheme>::verify_batch(
        &verifier_setup,
        &statement,
        &proof,
        &mut verifier_transcript,
    )
    .expect("union verify");
    eprintln!("union verify: {:.2?}", start.elapsed());
    assert_eq!(prover_transcript.state(), verifier_transcript.state());
}

fn main() {
    if std::env::var("BENCH_SETUP_SPLIT").is_ok() {
        setup_cost_split_by_flavor();
        return;
    }
    flavor_bench_sparse_union_vs_batched_one_hot();
}
