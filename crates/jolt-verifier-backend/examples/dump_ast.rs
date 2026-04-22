//! Trace a representative slice of the verifier and dump the AST.
//!
//! Runs a 2-round sumcheck verification (degree-2 round polynomial each)
//! plus a final `eq(r, ρ) * P(ρ)` consistency check, all through the
//! [`Tracing`] backend. Writes the resulting [`AstGraph`] as
//! `/tmp/jolt-ast.dot` (Graphviz) and `/tmp/jolt-ast.mmd` (Mermaid).
//!
//! Usage:
//!   cargo run -p jolt-verifier-backend --example dump_ast
//!   dot -Tpng /tmp/jolt-ast.dot -o /tmp/jolt-ast.png

#![expect(
    clippy::print_stderr,
    reason = "diagnostic output is the entire point of this example"
)]
#![expect(
    clippy::expect_used,
    reason = "example: panics on I/O or replay errors are fine"
)]

use std::fs;

use jolt_field::{Field, Fr};
use jolt_verifier_backend::helpers::{eq_eval, univariate_horner};
use jolt_verifier_backend::{replay_trace, to_dot, to_mermaid, FieldBackend, Tracing};

fn horner(coeffs: &[Fr], r: Fr) -> Fr {
    coeffs
        .iter()
        .rev()
        .fold(Fr::from_u64(0), |acc, &c| acc * r + c)
}

fn eq_native(a: &[Fr], b: &[Fr]) -> Fr {
    a.iter().zip(b).fold(Fr::from_u64(1), |acc, (a_i, b_i)| {
        let one = Fr::from_u64(1);
        acc * (*a_i * *b_i + (one - *a_i) * (one - *b_i))
    })
}

fn main() {
    let coeffs0 = [Fr::from_u64(9), Fr::from_u64(11), Fr::from_u64(13)];
    let coeffs1 = [Fr::from_u64(1959), Fr::from_u64(17), Fr::from_u64(18)];

    let claim0 = coeffs0[0] + coeffs0.iter().copied().sum::<Fr>();

    let r0 = Fr::from_u64(17);
    let claim1 = horner(&coeffs0, r0);

    let r1 = Fr::from_u64(36);
    let claim2 = horner(&coeffs1, r1);

    let rho = [Fr::from_u64(2), Fr::from_u64(3)];
    let challenges_native = [r0, r1];
    let eq_v = eq_native(&challenges_native, &rho);

    let p_at_rho = claim2 * eq_v.inverse().expect("nonzero eq");

    debug_assert_eq!(
        coeffs1[0] + coeffs1.iter().copied().sum::<Fr>(),
        claim1,
        "round-1 consistency must hold"
    );

    let mut t = Tracing::<Fr>::new();

    let mut claim = t.wrap_proof(claim0, "S₀ claim");
    let mut challenges: Vec<<Tracing<Fr> as FieldBackend>::Scalar> = Vec::new();

    let labels = [
        ("p₀.c0", "p₀.c1", "p₀.c2", "r₀"),
        ("p₁.c0", "p₁.c1", "p₁.c2", "r₁"),
    ];
    let coeff_sets = [coeffs0, coeffs1];
    let challenge_values = [r0, r1];

    for (round, ((c0_v, c1_v, c2_v), &r_v)) in coeff_sets
        .iter()
        .map(|c| (c[0], c[1], c[2]))
        .zip(challenge_values.iter())
        .enumerate()
    {
        let (l0, l1, l2, lr) = labels[round];
        let c0 = t.wrap_proof(c0_v, l0);
        let c1 = t.wrap_proof(c1_v, l1);
        let c2 = t.wrap_proof(c2_v, l2);

        let c0_plus_c1 = t.add(&c0, &c1);
        let p_at_1 = t.add(&c0_plus_c1, &c2);

        let consistency = t.add(&c0, &p_at_1);
        let _ = t.assert_eq(&consistency, &claim, "round consistency");

        let r = t.wrap_challenge(r_v, lr);
        claim = univariate_horner(&mut t, &[c0, c1, c2], &r);
        challenges.push(r);
    }

    let p_at_rho_w = t.wrap_proof(p_at_rho, "P(ρ)");
    let rho_a = t.wrap_public(rho[0], "ρ₀");
    let rho_b = t.wrap_public(rho[1], "ρ₁");

    let eq_value = eq_eval(&mut t, &challenges, &[rho_a, rho_b]);
    let final_lhs = t.mul(&eq_value, &p_at_rho_w);
    let _ = t.assert_eq(&final_lhs, &claim, "final eq*g check");

    let graph = t.snapshot();
    let wraps = t.wrap_values();

    eprintln!(
        "graph: {} nodes, {} assertions, {} wraps",
        graph.node_count(),
        graph.assertion_count(),
        wraps.len()
    );

    let dot_path = "/tmp/jolt-ast.dot";
    let mmd_path = "/tmp/jolt-ast.mmd";
    fs::write(dot_path, to_dot(&graph)).expect("write dot");
    fs::write(mmd_path, to_mermaid(&graph)).expect("write mermaid");
    eprintln!("wrote {dot_path} and {mmd_path}");

    let _ = replay_trace(&graph, &wraps).expect("trace must replay");
    eprintln!("replay OK");
}
