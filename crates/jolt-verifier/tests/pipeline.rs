//! End-to-end pipeline tests for jolt-verifier.
//!
//! Tests the full S1→S2→S8 verification pipeline using uniform Spartan
//! with per-cycle constraints and config-driven stage descriptors.

use jolt_field::{Field, Fr};
use jolt_openings::mock::{MockCommitment, MockCommitmentScheme};
use jolt_openings::{CommitmentScheme, ProverClaim, VerifierClaim};
use jolt_poly::{EqPolynomial, Polynomial};
use jolt_spartan::{UniformSpartanKey, UniformSpartanProver};
use jolt_sumcheck::{BatchedSumcheckProver, SumcheckClaim};
use jolt_transcript::{AppendToTranscript, Blake2bTranscript, Transcript};
use jolt_verifier::config::ProverConfig;
use jolt_verifier::proof::SumcheckStageProof;
use jolt_verifier::stage::StageDescriptor;
use jolt_verifier::{
    verify, verify_openings, verify_spartan, verify_with_backend, JoltError, JoltProof,
    JoltVerifyingKey,
};
use jolt_verifier_backend::{replay_trace, Native, Tracing};
use num_traits::Zero;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

type MockPCS = MockCommitmentScheme<Fr>;

fn test_prover_config() -> ProverConfig {
    use jolt_verifier::config::{OneHotConfig, ReadWriteConfig};
    ProverConfig {
        trace_length: 2,
        ram_k: 16,
        one_hot_config: OneHotConfig::new(1),
        rw_config: ReadWriteConfig::new(1, 4),
    }
}

fn test_uniform_key(num_cycles: usize) -> UniformSpartanKey<Fr> {
    let one = Fr::from_u64(1);
    UniformSpartanKey::new(
        num_cycles,
        1,
        3,
        vec![vec![(1, one)]],
        vec![vec![(1, one)]],
        vec![vec![(2, one)]],
    )
}

fn make_cycle_witness(x: u64) -> Vec<Fr> {
    vec![Fr::from_u64(1), Fr::from_u64(x), Fr::from_u64(x * x)]
}

fn flatten_witnesses(key: &UniformSpartanKey<Fr>, cycle_witnesses: &[Vec<Fr>]) -> Vec<Fr> {
    let total_cols_padded = key.total_cols().next_power_of_two();
    let mut flat = vec![Fr::from_u64(0); total_cols_padded];
    for (c, w) in cycle_witnesses.iter().enumerate() {
        let base = c * key.num_vars_padded;
        for (v, &val) in w.iter().enumerate().take(key.num_vars) {
            flat[base + v] = val;
        }
    }
    flat
}

/// Commit witness and append to transcript. Returns the commitment.
fn commit_and_append<PCS: CommitmentScheme<Field = Fr>>(
    flat: &[Fr],
    setup: &PCS::ProverSetup,
    transcript: &mut Blake2bTranscript,
) -> PCS::Output {
    let (commitment, _) = PCS::commit(flat, setup);
    transcript.append_bytes(format!("{commitment:?}").as_bytes());
    commitment
}

use jolt_sumcheck::prover::SumcheckCompute;

struct EqGWitness {
    eq: Vec<Fr>,
    g: Vec<Fr>,
}

impl SumcheckCompute<Fr> for EqGWitness {
    fn round_polynomial(&self) -> jolt_poly::UnivariatePoly<Fr> {
        let half = self.eq.len() / 2;
        let mut evals = [Fr::zero(); 3];
        for j in 0..half {
            let e_lo = self.eq[j];
            let e_hi = self.eq[j + half];
            let g_lo = self.g[j];
            let g_hi = self.g[j + half];
            evals[0] += e_lo * g_lo;
            evals[1] += e_hi * g_hi;
            evals[2] += (e_hi + e_hi - e_lo) * (g_hi + g_hi - g_lo);
        }
        jolt_poly::UnivariatePoly::interpolate_over_integers(&evals)
    }

    fn bind(&mut self, c: Fr) {
        let half = self.eq.len() / 2;
        for j in 0..half {
            self.eq[j] = self.eq[j] + c * (self.eq[j + half] - self.eq[j]);
            self.g[j] = self.g[j] + c * (self.g[j + half] - self.g[j]);
        }
        self.eq.truncate(half);
        self.g.truncate(half);
    }
}

#[test]
fn verify_spartan_round_trip() {
    let key = test_uniform_key(2);
    let witnesses = vec![make_cycle_witness(3), make_cycle_witness(5)];
    let flat = flatten_witnesses(&key, &witnesses);

    let mut pt = Blake2bTranscript::new(b"verify-spartan-rt");
    let _ = commit_and_append::<MockPCS>(&flat, &(), &mut pt);
    let (proof, prover_r_x, prover_r_y) =
        UniformSpartanProver::prove_dense_with_challenges(&key, &flat, &mut pt)
            .expect("proving should succeed");

    let mut vt = Blake2bTranscript::new(b"verify-spartan-rt");
    let _ = commit_and_append::<MockPCS>(&flat, &(), &mut vt);
    let (verifier_r_x, verifier_r_y) =
        verify_spartan(&key, &proof, &mut vt).expect("verification should succeed");

    assert_eq!(prover_r_x, verifier_r_x);
    assert_eq!(prover_r_y, verifier_r_y);
}

#[test]
fn verify_openings_round_trip() {
    let num_vars = 4;
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    let point: Vec<Fr> = (0..num_vars).map(|_| Fr::random(&mut rng)).collect();

    let mut prover_claims = Vec::new();
    let mut verifier_claims = Vec::new();

    for _ in 0..3 {
        let poly = Polynomial::<Fr>::random(num_vars, &mut rng);
        let eval = poly.evaluate(&point);
        let commitment = MockPCS::commit(poly.evaluations(), &()).0;

        prover_claims.push(ProverClaim {
            evaluations: poly.evaluations().to_vec(),
            point: point.clone(),
            eval,
        });
        verifier_claims.push(VerifierClaim {
            commitment,
            point: point.clone(),
            eval,
        });
    }

    let mut pt = Blake2bTranscript::new(b"verify-openings-rt");
    use jolt_openings::{OpeningReduction, RlcReduction};
    let (reduced_prover, ()) =
        <RlcReduction as OpeningReduction<MockPCS>>::reduce_prover(prover_claims, &mut pt);
    let proofs: Vec<_> = reduced_prover
        .into_iter()
        .map(|claim| {
            let poly: <MockPCS as CommitmentScheme>::Polynomial = claim.evaluations.into();
            MockPCS::open(&poly, &claim.point, claim.eval, &(), None, &mut pt)
        })
        .collect();
    let mut vt = Blake2bTranscript::new(b"verify-openings-rt");
    verify_openings::<MockPCS, _>(verifier_claims, &proofs, &(), &mut vt)
        .expect("opening verification should succeed");
}

#[test]
fn verify_rejects_wrong_stage_count() {
    let key = test_uniform_key(2);
    let witnesses = vec![make_cycle_witness(3), make_cycle_witness(5)];
    let flat = flatten_witnesses(&key, &witnesses);

    let mut pt = Blake2bTranscript::new(b"stage-count");
    let witness_commitment = commit_and_append::<MockPCS>(&flat, &(), &mut pt);
    let (spartan_proof, _, _) =
        UniformSpartanProver::prove_dense_with_challenges(&key, &flat, &mut pt)
            .expect("proving should succeed");

    let proof = JoltProof::<Fr, MockPCS> {
        config: test_prover_config(),
        spartan_proof,
        stage_proofs: vec![],
        opening_proofs: vec![],
        witness_commitment,
        commitments: vec![],
    };

    let vk = JoltVerifyingKey {
        spartan_key: key,
        pcs_setup: (),
    };

    // Verify builds one descriptor but proof has 0 stage proofs → mismatch.
    let mut vt = Blake2bTranscript::new(b"stage-count");
    let result = verify::<MockPCS, _>(
        &proof,
        &vk,
        |_r_x, _r_y, _t| {
            vec![StageDescriptor::claim_reduction(
                vec![],
                vec![],
                Fr::zero(),
                vec![],
            )]
        },
        &mut vt,
    );
    assert!(
        matches!(result, Err(JoltError::InvalidProof(_))),
        "should reject stage count mismatch, got: {result:?}"
    );
}

#[test]
fn verify_rejects_bad_evaluation() {
    let mut rng = ChaCha20Rng::seed_from_u64(77);

    let key = test_uniform_key(2);
    let witnesses = vec![make_cycle_witness(3), make_cycle_witness(5)];
    let flat = flatten_witnesses(&key, &witnesses);

    let num_vars = 3;
    let n = 1usize << num_vars;
    let eq_point: Vec<Fr> = (0..num_vars).map(|_| Fr::random(&mut rng)).collect();
    let poly_table: Vec<Fr> = (0..n).map(|_| Fr::random(&mut rng)).collect();
    let commitment = MockPCS::commit(&poly_table, &()).0;

    let eq_table = EqPolynomial::new(eq_point.clone()).evaluations();
    let claimed_sum: Fr = eq_table
        .iter()
        .zip(poly_table.iter())
        .map(|(&e, &p)| e * p)
        .sum();

    let mut pt = Blake2bTranscript::new(b"bad-eval");
    let witness_commitment = commit_and_append::<MockPCS>(&flat, &(), &mut pt);
    let (spartan_proof, _, _) =
        UniformSpartanProver::prove_dense_with_challenges(&key, &flat, &mut pt)
            .expect("proving should succeed");

    let claim = SumcheckClaim {
        num_vars,
        degree: 2,
        claimed_sum,
    };

    let mut sc_witnesses: Vec<Box<dyn SumcheckCompute<Fr>>> = vec![Box::new(EqGWitness {
        eq: eq_table,
        g: poly_table.clone(),
    })];
    let sumcheck_proof = BatchedSumcheckProver::prove(&[claim], &mut sc_witnesses, &mut pt);

    let stage_proof = SumcheckStageProof {
        sumcheck_proof,
        evaluations: vec![Fr::from_u64(999)], // WRONG
    };

    let proof = JoltProof::<Fr, MockPCS> {
        config: test_prover_config(),
        spartan_proof,
        stage_proofs: vec![stage_proof],
        opening_proofs: vec![],
        witness_commitment,
        commitments: vec![commitment],
    };

    let vk = JoltVerifyingKey {
        spartan_key: key,
        pcs_setup: (),
    };

    let mut vt = Blake2bTranscript::new(b"bad-eval");
    let result = verify::<MockPCS, _>(
        &proof,
        &vk,
        |_r_x, _r_y, _t| {
            vec![StageDescriptor::claim_reduction(
                eq_point,
                vec![Fr::from_u64(1)],
                claimed_sum,
                vec![0],
            )]
        },
        &mut vt,
    );
    assert!(
        result.is_err(),
        "tampered evaluation should be rejected, got: {result:?}"
    );
}

#[test]
fn sumcheck_eq_g_final_eval_matches() {
    let mut rng = ChaCha20Rng::seed_from_u64(42);
    let num_vars = 3;
    let n = 1usize << num_vars;

    let eq_point: Vec<Fr> = (0..num_vars).map(|_| Fr::random(&mut rng)).collect();
    let poly_table: Vec<Fr> = (0..n).map(|_| Fr::random(&mut rng)).collect();

    let eq_table = EqPolynomial::new(eq_point.clone()).evaluations();
    let claimed_sum: Fr = eq_table
        .iter()
        .zip(poly_table.iter())
        .map(|(&e, &p)| e * p)
        .sum();

    let claim = SumcheckClaim {
        num_vars,
        degree: 2,
        claimed_sum,
    };

    let mut pt = Blake2bTranscript::new(b"eq-g-test");
    let mut sc_witnesses: Vec<Box<dyn SumcheckCompute<Fr>>> = vec![Box::new(EqGWitness {
        eq: eq_table,
        g: poly_table.clone(),
    })];
    let proof = BatchedSumcheckProver::prove(&[claim.clone()], &mut sc_witnesses, &mut pt);

    let mut vt = Blake2bTranscript::new(b"eq-g-test");
    let (final_eval, challenges) =
        jolt_sumcheck::BatchedSumcheckVerifier::verify(&[claim], &proof, &mut vt)
            .expect("verification should succeed");

    let eq_eval = EqPolynomial::new(eq_point).evaluate(&challenges);
    let poly_eval = Polynomial::new(poly_table).evaluate(&challenges);
    let expected = eq_eval * poly_eval;

    assert_eq!(
        final_eval, expected,
        "final_eval from sumcheck should equal eq(r, challenges) * p(challenges)"
    );
}

#[test]
fn full_pipeline_spartan_plus_one_stage() {
    let mut rng = ChaCha20Rng::seed_from_u64(99);

    let key = test_uniform_key(2);
    let witnesses = vec![make_cycle_witness(3), make_cycle_witness(5)];
    let flat = flatten_witnesses(&key, &witnesses);

    let num_vars = 3;
    let num_polys = 2;
    let n = 1usize << num_vars;

    let eq_point: Vec<Fr> = (0..num_vars).map(|_| Fr::random(&mut rng)).collect();
    let coefficients: Vec<Fr> = (0..num_polys).map(|_| Fr::random(&mut rng)).collect();
    let poly_tables: Vec<Vec<Fr>> = (0..num_polys)
        .map(|_| (0..n).map(|_| Fr::random(&mut rng)).collect())
        .collect();

    let commitments: Vec<MockCommitment<Fr>> = poly_tables
        .iter()
        .map(|table| MockPCS::commit(table, &()).0)
        .collect();

    let eq_table = EqPolynomial::new(eq_point.clone()).evaluations();
    let mut g_table = vec![Fr::zero(); n];
    for (i, table) in poly_tables.iter().enumerate() {
        for (j, g) in g_table.iter_mut().enumerate() {
            *g += coefficients[i] * table[j];
        }
    }
    let claimed_sum: Fr = eq_table
        .iter()
        .zip(g_table.iter())
        .map(|(&e, &g)| e * g)
        .sum();

    let mut pt = Blake2bTranscript::new(b"full-pipeline");

    let witness_commitment = commit_and_append::<MockPCS>(&flat, &(), &mut pt);
    let (spartan_proof, _prover_r_x, _prover_r_y) =
        UniformSpartanProver::prove_dense_with_challenges(&key, &flat, &mut pt)
            .expect("spartan proving should succeed");

    let claim = SumcheckClaim {
        num_vars,
        degree: 2,
        claimed_sum,
    };
    let mut sc_witnesses: Vec<Box<dyn SumcheckCompute<Fr>>> = vec![Box::new(EqGWitness {
        eq: eq_table.clone(),
        g: g_table,
    })];
    let sumcheck_proof = BatchedSumcheckProver::prove(&[claim.clone()], &mut sc_witnesses, &mut pt);

    // Replay verifier transcript to extract challenges
    let mut vt_replay = Blake2bTranscript::new(b"full-pipeline");
    let _ = commit_and_append::<MockPCS>(&flat, &(), &mut vt_replay);
    let _ = verify_spartan(&key, &spartan_proof, &mut vt_replay).unwrap();
    let (_, s2_challenges) =
        jolt_sumcheck::BatchedSumcheckVerifier::verify(&[claim], &sumcheck_proof, &mut vt_replay)
            .expect("sumcheck replay should succeed");

    let evaluations: Vec<Fr> = poly_tables
        .iter()
        .map(|table| Polynomial::new(table.clone()).evaluate(&s2_challenges))
        .collect();

    // Fiat-Shamir: flush opening claim evals to transcript (matches verifier).
    for &eval in &evaluations {
        eval.append_to_transcript(&mut pt);
    }

    let stage_proof = SumcheckStageProof {
        sumcheck_proof,
        evaluations: evaluations.clone(),
    };

    // Opening claims: stage polys + witness
    let mut prover_claims: Vec<ProverClaim<Fr>> = poly_tables
        .iter()
        .zip(evaluations.iter())
        .map(|(table, &eval)| ProverClaim {
            evaluations: table.clone(),
            point: s2_challenges.clone(),
            eval,
        })
        .collect();

    // Witness opening claim — must be last to match verifier ordering.
    let witness_eval = spartan_proof.witness_eval;
    let spartan_r_y = _prover_r_y;
    prover_claims.push(ProverClaim {
        evaluations: flat.clone(),
        point: spartan_r_y.clone(),
        eval: witness_eval,
    });

    use jolt_openings::{OpeningReduction, RlcReduction};
    let (reduced, ()) =
        <RlcReduction as OpeningReduction<MockPCS>>::reduce_prover(prover_claims, &mut pt);
    let pcs_proofs: Vec<_> = reduced
        .into_iter()
        .map(|claim| {
            let poly: <MockPCS as CommitmentScheme>::Polynomial = claim.evaluations.into();
            MockPCS::open(&poly, &claim.point, claim.eval, &(), None, &mut pt)
        })
        .collect();
    let proof = JoltProof {
        config: test_prover_config(),
        spartan_proof,
        stage_proofs: vec![stage_proof],
        opening_proofs: pcs_proofs,
        witness_commitment,
        commitments: commitments.clone(),
    };

    let vk = JoltVerifyingKey {
        spartan_key: key,
        pcs_setup: (),
    };

    let mut vt = Blake2bTranscript::new(b"full-pipeline");
    let (_r_x, r_y) = verify::<MockPCS, _>(
        &proof,
        &vk,
        |_r_x, _r_y, _t| {
            vec![StageDescriptor::claim_reduction(
                eq_point,
                coefficients,
                claimed_sum,
                (0..num_polys).collect(),
            )]
        },
        &mut vt,
    )
    .expect("full pipeline verification should succeed");

    assert!(!r_y.is_empty());
}

/// Same shape as `full_pipeline_spartan_plus_one_stage`, but runs the
/// verifier through three independent paths and asserts they all agree:
///
/// 1. `verify` (legacy cleartext path) — establishes the ground truth.
/// 2. `verify_with_backend(Native)` — must produce identical output;
///    proves the new dispatch is zero-overhead and behaviour-preserving.
/// 3. `verify_with_backend(Tracing)` — records the full S2-S7 algebraic
///    DAG as an [`AstGraph`]. Replaying the graph against its own wrap
///    values must reproduce all values without firing any assertion.
///
/// Together these prove that the [`FieldBackend`] abstraction faithfully
/// captures the verifier's algebraic execution and that the resulting
/// trace can be re-evaluated, which is exactly what downstream consumers
/// (recursion lowering, R1CS emit, Lean export) need.
#[test]
fn backend_pipeline_native_and_tracing_match() {
    let mut rng = ChaCha20Rng::seed_from_u64(99);

    let key = test_uniform_key(2);
    let witnesses = vec![make_cycle_witness(3), make_cycle_witness(5)];
    let flat = flatten_witnesses(&key, &witnesses);

    let num_vars = 3;
    let num_polys = 2;
    let n = 1usize << num_vars;

    let eq_point: Vec<Fr> = (0..num_vars).map(|_| Fr::random(&mut rng)).collect();
    let coefficients: Vec<Fr> = (0..num_polys).map(|_| Fr::random(&mut rng)).collect();
    let poly_tables: Vec<Vec<Fr>> = (0..num_polys)
        .map(|_| (0..n).map(|_| Fr::random(&mut rng)).collect())
        .collect();

    let commitments: Vec<MockCommitment<Fr>> = poly_tables
        .iter()
        .map(|table| MockPCS::commit(table, &()).0)
        .collect();

    let eq_table = EqPolynomial::new(eq_point.clone()).evaluations();
    let mut g_table = vec![Fr::zero(); n];
    for (i, table) in poly_tables.iter().enumerate() {
        for (j, g) in g_table.iter_mut().enumerate() {
            *g += coefficients[i] * table[j];
        }
    }
    let claimed_sum: Fr = eq_table
        .iter()
        .zip(g_table.iter())
        .map(|(&e, &g)| e * g)
        .sum();

    let mut pt = Blake2bTranscript::new(b"backend-pipeline");
    let witness_commitment = commit_and_append::<MockPCS>(&flat, &(), &mut pt);
    let (spartan_proof, _prover_r_x, prover_r_y) =
        UniformSpartanProver::prove_dense_with_challenges(&key, &flat, &mut pt)
            .expect("spartan proving should succeed");

    let claim = SumcheckClaim {
        num_vars,
        degree: 2,
        claimed_sum,
    };
    let mut sc_witnesses: Vec<Box<dyn SumcheckCompute<Fr>>> = vec![Box::new(EqGWitness {
        eq: eq_table.clone(),
        g: g_table,
    })];
    let sumcheck_proof = BatchedSumcheckProver::prove(&[claim.clone()], &mut sc_witnesses, &mut pt);

    // Replay verifier transcript to extract challenges so we can compute
    // per-poly evaluations the verifier will check.
    let mut vt_replay = Blake2bTranscript::new(b"backend-pipeline");
    let _ = commit_and_append::<MockPCS>(&flat, &(), &mut vt_replay);
    let _ = verify_spartan(&key, &spartan_proof, &mut vt_replay).unwrap();
    let (_, s2_challenges) =
        jolt_sumcheck::BatchedSumcheckVerifier::verify(&[claim], &sumcheck_proof, &mut vt_replay)
            .expect("sumcheck replay should succeed");

    let evaluations: Vec<Fr> = poly_tables
        .iter()
        .map(|table| Polynomial::new(table.clone()).evaluate(&s2_challenges))
        .collect();

    for &eval in &evaluations {
        eval.append_to_transcript(&mut pt);
    }

    let stage_proof = SumcheckStageProof {
        sumcheck_proof,
        evaluations: evaluations.clone(),
    };

    let mut prover_claims: Vec<ProverClaim<Fr>> = poly_tables
        .iter()
        .zip(evaluations.iter())
        .map(|(table, &eval)| ProverClaim {
            evaluations: table.clone(),
            point: s2_challenges.clone(),
            eval,
        })
        .collect();
    prover_claims.push(ProverClaim {
        evaluations: flat.clone(),
        point: prover_r_y,
        eval: spartan_proof.witness_eval,
    });

    use jolt_openings::{OpeningReduction, RlcReduction};
    let (reduced, ()) =
        <RlcReduction as OpeningReduction<MockPCS>>::reduce_prover(prover_claims, &mut pt);
    let pcs_proofs: Vec<_> = reduced
        .into_iter()
        .map(|claim| {
            let poly: <MockPCS as CommitmentScheme>::Polynomial = claim.evaluations.into();
            MockPCS::open(&poly, &claim.point, claim.eval, &(), None, &mut pt)
        })
        .collect();
    let proof = JoltProof {
        config: test_prover_config(),
        spartan_proof,
        stage_proofs: vec![stage_proof],
        opening_proofs: pcs_proofs,
        witness_commitment,
        commitments: commitments.clone(),
    };

    let vk = JoltVerifyingKey {
        spartan_key: key,
        pcs_setup: (),
    };

    // Build a fresh descriptor closure for each call (the closure is FnOnce
    // and consumes `eq_point` / `coefficients` via `claim_reduction`).
    let make_descriptors = |eq_point: Vec<Fr>, coefficients: Vec<Fr>| {
        move |_r_x: &[Fr], _r_y: &[Fr], _t: &mut Blake2bTranscript| {
            vec![StageDescriptor::claim_reduction(
                eq_point,
                coefficients,
                claimed_sum,
                (0..num_polys).collect(),
            )]
        }
    };

    // 1. Cleartext baseline.
    let mut vt_legacy = Blake2bTranscript::new(b"backend-pipeline");
    let (legacy_r_x, legacy_r_y) = verify::<MockPCS, _>(
        &proof,
        &vk,
        make_descriptors(eq_point.clone(), coefficients.clone()),
        &mut vt_legacy,
    )
    .expect("legacy verify should succeed");

    // 2. Native backend — must match cleartext exactly.
    let mut vt_native = Blake2bTranscript::new(b"backend-pipeline");
    let mut native = Native::<Fr>::new();
    let (native_r_x, native_r_y) = verify_with_backend::<_, MockPCS, _>(
        &mut native,
        &proof,
        &vk,
        make_descriptors(eq_point.clone(), coefficients.clone()),
        &mut vt_native,
    )
    .expect("Native backend verify should succeed");
    assert_eq!(legacy_r_x, native_r_x, "Native must match cleartext r_x");
    assert_eq!(legacy_r_y, native_r_y, "Native must match cleartext r_y");

    // 3. Tracing backend — must produce a non-trivial graph that replays
    //    cleanly against its captured wrap values.
    let mut vt_tracing = Blake2bTranscript::new(b"backend-pipeline");
    let mut tracer = Tracing::<Fr>::new();
    let (tracing_r_x, tracing_r_y) = verify_with_backend::<_, MockPCS, _>(
        &mut tracer,
        &proof,
        &vk,
        make_descriptors(eq_point, coefficients),
        &mut vt_tracing,
    )
    .expect("Tracing backend verify should succeed");
    assert_eq!(
        legacy_r_x, tracing_r_x,
        "Tracing must produce same Fiat-Shamir r_x as cleartext"
    );
    assert_eq!(
        legacy_r_y, tracing_r_y,
        "Tracing must produce same Fiat-Shamir r_y as cleartext"
    );

    let graph = tracer.snapshot();
    let wraps = tracer.wrap_values();
    assert!(
        graph.node_count() > 50,
        "tracing graph should be non-trivial, got {} nodes",
        graph.node_count()
    );
    assert!(
        graph.assertion_count() >= 1,
        "tracing graph should record at least one stage assertion"
    );

    let _values = replay_trace(&graph, &wraps).expect("tracing replay should succeed");
}

mod dory {
    use super::*;
    use jolt_dory::{DoryCommitment, DoryScheme};
    use jolt_openings::{OpeningReduction, RlcReduction};

    #[test]
    fn dory_verify_openings_round_trip() {
        let num_vars = 4;
        let mut rng = ChaCha20Rng::seed_from_u64(42);

        let prover_setup = DoryScheme::setup_prover(num_vars);
        let verifier_setup = DoryScheme::setup_verifier(num_vars);

        let point: Vec<Fr> = (0..num_vars).map(|_| Fr::random(&mut rng)).collect();

        let mut prover_claims = Vec::new();
        let mut verifier_claims = Vec::new();

        for _ in 0..3 {
            let poly = Polynomial::<Fr>::random(num_vars, &mut rng);
            let eval = poly.evaluate(&point);
            let (commitment, _hint) = DoryScheme::commit(poly.evaluations(), &prover_setup);

            prover_claims.push(ProverClaim {
                evaluations: poly.evaluations().to_vec(),
                point: point.clone(),
                eval,
            });
            verifier_claims.push(VerifierClaim {
                commitment,
                point: point.clone(),
                eval,
            });
        }

        let mut pt = Blake2bTranscript::new(b"dory-openings-rt");
        let (reduced_prover, ()) =
            <RlcReduction as OpeningReduction<DoryScheme>>::reduce_prover(prover_claims, &mut pt);
        let proofs: Vec<_> = reduced_prover
            .into_iter()
            .map(|claim| {
                let poly: <DoryScheme as CommitmentScheme>::Polynomial = claim.evaluations.into();
                DoryScheme::open(
                    &poly,
                    &claim.point,
                    claim.eval,
                    &prover_setup,
                    None,
                    &mut pt,
                )
            })
            .collect();
        let mut vt = Blake2bTranscript::new(b"dory-openings-rt");
        verify_openings::<DoryScheme, _>(verifier_claims, &proofs, &verifier_setup, &mut vt)
            .expect("Dory opening verification should succeed");
    }

    #[test]
    fn dory_full_pipeline_spartan_plus_one_stage() {
        let mut rng = ChaCha20Rng::seed_from_u64(99);

        let num_vars = 3;
        let num_polys = 2;
        let n = 1usize << num_vars;

        let prover_setup = DoryScheme::setup_prover(num_vars);
        let verifier_setup = DoryScheme::setup_verifier(num_vars);

        let key = test_uniform_key(2);
        let witnesses = vec![make_cycle_witness(3), make_cycle_witness(5)];
        let flat = flatten_witnesses(&key, &witnesses);

        let eq_point: Vec<Fr> = (0..num_vars).map(|_| Fr::random(&mut rng)).collect();
        let coefficients: Vec<Fr> = (0..num_polys).map(|_| Fr::random(&mut rng)).collect();
        let poly_tables: Vec<Vec<Fr>> = (0..num_polys)
            .map(|_| (0..n).map(|_| Fr::random(&mut rng)).collect())
            .collect();

        let commitments: Vec<DoryCommitment> = poly_tables
            .iter()
            .map(|table| DoryScheme::commit(table, &prover_setup).0)
            .collect();

        let eq_table = EqPolynomial::new(eq_point.clone()).evaluations();
        let mut g_table = vec![Fr::zero(); n];
        for (i, table) in poly_tables.iter().enumerate() {
            for (j, g) in g_table.iter_mut().enumerate() {
                *g += coefficients[i] * table[j];
            }
        }
        let claimed_sum: Fr = eq_table
            .iter()
            .zip(g_table.iter())
            .map(|(&e, &g)| e * g)
            .sum();

        let mut pt = Blake2bTranscript::new(b"dory-full-pipeline");

        let witness_commitment = commit_and_append::<DoryScheme>(&flat, &prover_setup, &mut pt);
        let (spartan_proof, _prover_r_x, prover_r_y) =
            UniformSpartanProver::prove_dense_with_challenges(&key, &flat, &mut pt)
                .expect("spartan proving should succeed");

        let claim = SumcheckClaim {
            num_vars,
            degree: 2,
            claimed_sum,
        };
        let mut sc_witnesses: Vec<Box<dyn SumcheckCompute<Fr>>> = vec![Box::new(EqGWitness {
            eq: eq_table.clone(),
            g: g_table,
        })];
        let sumcheck_proof =
            BatchedSumcheckProver::prove(&[claim.clone()], &mut sc_witnesses, &mut pt);

        let mut vt_replay = Blake2bTranscript::new(b"dory-full-pipeline");
        let _ = commit_and_append::<DoryScheme>(&flat, &prover_setup, &mut vt_replay);
        let _ = verify_spartan(&key, &spartan_proof, &mut vt_replay).unwrap();
        let (_, s2_challenges) = jolt_sumcheck::BatchedSumcheckVerifier::verify(
            &[claim],
            &sumcheck_proof,
            &mut vt_replay,
        )
        .expect("sumcheck replay should succeed");

        let evaluations: Vec<Fr> = poly_tables
            .iter()
            .map(|table| Polynomial::new(table.clone()).evaluate(&s2_challenges))
            .collect();

        // Fiat-Shamir: flush opening claim evals to transcript (matches verifier).
        for &eval in &evaluations {
            eval.append_to_transcript(&mut pt);
        }

        let stage_proof = SumcheckStageProof {
            sumcheck_proof,
            evaluations: evaluations.clone(),
        };

        let mut prover_claims: Vec<ProverClaim<Fr>> = poly_tables
            .iter()
            .zip(evaluations.iter())
            .map(|(table, &eval)| ProverClaim {
                evaluations: table.clone(),
                point: s2_challenges.clone(),
                eval,
            })
            .collect();

        // Witness opening claim — last.
        prover_claims.push(ProverClaim {
            evaluations: flat.clone(),
            point: prover_r_y,
            eval: spartan_proof.witness_eval,
        });

        let (reduced, ()) =
            <RlcReduction as OpeningReduction<DoryScheme>>::reduce_prover(prover_claims, &mut pt);
        let pcs_proofs: Vec<_> = reduced
            .into_iter()
            .map(|claim| {
                let poly: <DoryScheme as CommitmentScheme>::Polynomial = claim.evaluations.into();
                DoryScheme::open(
                    &poly,
                    &claim.point,
                    claim.eval,
                    &prover_setup,
                    None,
                    &mut pt,
                )
            })
            .collect();
        let proof = JoltProof {
            config: test_prover_config(),
            spartan_proof,
            stage_proofs: vec![stage_proof],
            opening_proofs: pcs_proofs,
            witness_commitment,
            commitments: commitments.clone(),
        };

        let vk = JoltVerifyingKey {
            spartan_key: key,
            pcs_setup: verifier_setup,
        };

        let mut vt = Blake2bTranscript::new(b"dory-full-pipeline");
        let (_r_x, r_y) = verify::<DoryScheme, _>(
            &proof,
            &vk,
            |_r_x, _r_y, _t| {
                vec![StageDescriptor::claim_reduction(
                    eq_point,
                    coefficients,
                    claimed_sum,
                    (0..num_polys).collect(),
                )]
            },
            &mut vt,
        )
        .expect("Dory full pipeline verification should succeed");

        assert!(!r_y.is_empty());
    }

    /// Same shape as `dory_full_pipeline_spartan_plus_one_stage` but
    /// asserts Native + Tracing parity against the legacy verifier on a
    /// real PCS (Dory). This proves the [`FieldBackend`] abstraction
    /// composes with non-mock group operations: the Spartan + opening
    /// stages still flow through the native group code, while the S2
    /// algebraic checks route through the backend.
    #[test]
    fn dory_backend_pipeline_native_and_tracing_match() {
        let mut rng = ChaCha20Rng::seed_from_u64(99);

        let num_vars = 3;
        let num_polys = 2;
        let n = 1usize << num_vars;

        let prover_setup = DoryScheme::setup_prover(num_vars);
        let verifier_setup = DoryScheme::setup_verifier(num_vars);

        let key = test_uniform_key(2);
        let witnesses = vec![make_cycle_witness(3), make_cycle_witness(5)];
        let flat = flatten_witnesses(&key, &witnesses);

        let eq_point: Vec<Fr> = (0..num_vars).map(|_| Fr::random(&mut rng)).collect();
        let coefficients: Vec<Fr> = (0..num_polys).map(|_| Fr::random(&mut rng)).collect();
        let poly_tables: Vec<Vec<Fr>> = (0..num_polys)
            .map(|_| (0..n).map(|_| Fr::random(&mut rng)).collect())
            .collect();

        let commitments: Vec<DoryCommitment> = poly_tables
            .iter()
            .map(|table| DoryScheme::commit(table, &prover_setup).0)
            .collect();

        let eq_table = EqPolynomial::new(eq_point.clone()).evaluations();
        let mut g_table = vec![Fr::zero(); n];
        for (i, table) in poly_tables.iter().enumerate() {
            for (j, g) in g_table.iter_mut().enumerate() {
                *g += coefficients[i] * table[j];
            }
        }
        let claimed_sum: Fr = eq_table
            .iter()
            .zip(g_table.iter())
            .map(|(&e, &g)| e * g)
            .sum();

        let mut pt = Blake2bTranscript::new(b"dory-backend-pipeline");
        let witness_commitment = commit_and_append::<DoryScheme>(&flat, &prover_setup, &mut pt);
        let (spartan_proof, _prover_r_x, prover_r_y) =
            UniformSpartanProver::prove_dense_with_challenges(&key, &flat, &mut pt)
                .expect("spartan proving should succeed");

        let claim = SumcheckClaim {
            num_vars,
            degree: 2,
            claimed_sum,
        };
        let mut sc_witnesses: Vec<Box<dyn SumcheckCompute<Fr>>> = vec![Box::new(EqGWitness {
            eq: eq_table.clone(),
            g: g_table,
        })];
        let sumcheck_proof =
            BatchedSumcheckProver::prove(&[claim.clone()], &mut sc_witnesses, &mut pt);

        let mut vt_replay = Blake2bTranscript::new(b"dory-backend-pipeline");
        let _ = commit_and_append::<DoryScheme>(&flat, &prover_setup, &mut vt_replay);
        let _ = verify_spartan(&key, &spartan_proof, &mut vt_replay).unwrap();
        let (_, s2_challenges) = jolt_sumcheck::BatchedSumcheckVerifier::verify(
            &[claim],
            &sumcheck_proof,
            &mut vt_replay,
        )
        .expect("sumcheck replay should succeed");

        let evaluations: Vec<Fr> = poly_tables
            .iter()
            .map(|table| Polynomial::new(table.clone()).evaluate(&s2_challenges))
            .collect();

        for &eval in &evaluations {
            eval.append_to_transcript(&mut pt);
        }

        let stage_proof = SumcheckStageProof {
            sumcheck_proof,
            evaluations: evaluations.clone(),
        };

        let mut prover_claims: Vec<ProverClaim<Fr>> = poly_tables
            .iter()
            .zip(evaluations.iter())
            .map(|(table, &eval)| ProverClaim {
                evaluations: table.clone(),
                point: s2_challenges.clone(),
                eval,
            })
            .collect();
        prover_claims.push(ProverClaim {
            evaluations: flat.clone(),
            point: prover_r_y,
            eval: spartan_proof.witness_eval,
        });

        let (reduced, ()) =
            <RlcReduction as OpeningReduction<DoryScheme>>::reduce_prover(prover_claims, &mut pt);
        let pcs_proofs: Vec<_> = reduced
            .into_iter()
            .map(|claim| {
                let poly: <DoryScheme as CommitmentScheme>::Polynomial = claim.evaluations.into();
                DoryScheme::open(
                    &poly,
                    &claim.point,
                    claim.eval,
                    &prover_setup,
                    None,
                    &mut pt,
                )
            })
            .collect();
        let proof = JoltProof {
            config: test_prover_config(),
            spartan_proof,
            stage_proofs: vec![stage_proof],
            opening_proofs: pcs_proofs,
            witness_commitment,
            commitments: commitments.clone(),
        };

        let vk = JoltVerifyingKey {
            spartan_key: key,
            pcs_setup: verifier_setup,
        };

        let make_descriptors = |eq_point: Vec<Fr>, coefficients: Vec<Fr>| {
            move |_r_x: &[Fr], _r_y: &[Fr], _t: &mut Blake2bTranscript| {
                vec![StageDescriptor::claim_reduction(
                    eq_point,
                    coefficients,
                    claimed_sum,
                    (0..num_polys).collect(),
                )]
            }
        };

        let mut vt_legacy = Blake2bTranscript::new(b"dory-backend-pipeline");
        let (legacy_r_x, legacy_r_y) = verify::<DoryScheme, _>(
            &proof,
            &vk,
            make_descriptors(eq_point.clone(), coefficients.clone()),
            &mut vt_legacy,
        )
        .expect("Dory legacy verify should succeed");

        let mut vt_native = Blake2bTranscript::new(b"dory-backend-pipeline");
        let mut native = Native::<Fr>::new();
        let (native_r_x, native_r_y) = verify_with_backend::<_, DoryScheme, _>(
            &mut native,
            &proof,
            &vk,
            make_descriptors(eq_point.clone(), coefficients.clone()),
            &mut vt_native,
        )
        .expect("Dory Native backend verify should succeed");
        assert_eq!(legacy_r_x, native_r_x);
        assert_eq!(legacy_r_y, native_r_y);

        let mut vt_tracing = Blake2bTranscript::new(b"dory-backend-pipeline");
        let mut tracer = Tracing::<Fr>::new();
        let (tracing_r_x, tracing_r_y) = verify_with_backend::<_, DoryScheme, _>(
            &mut tracer,
            &proof,
            &vk,
            make_descriptors(eq_point, coefficients),
            &mut vt_tracing,
        )
        .expect("Dory Tracing backend verify should succeed");
        assert_eq!(legacy_r_x, tracing_r_x);
        assert_eq!(legacy_r_y, tracing_r_y);

        let graph = tracer.snapshot();
        let wraps = tracer.wrap_values();
        assert!(graph.node_count() > 50);
        assert!(graph.assertion_count() >= 1);
        let _ = replay_trace(&graph, &wraps).expect("Dory tracing replay should succeed");
    }
}
