//! Backend-aware [`Expr`] evaluation.
//!
//! `jolt-ir` already ships an [`Expr::evaluate`](jolt_ir::Expr::evaluate)
//! that runs over a concrete [`jolt_field::Field`]. That is the right
//! shape for the prover and for everything that wants raw scalar output.
//!
//! The verifier-side caller wants to dispatch the same arithmetic through
//! a [`FieldBackend`] so the same [`Expr`] tree can be evaluated natively,
//! recorded into an AST (Tracing), or lowered into R1CS (R1CSGen). This
//! module supplies that dispatch as a single free function.
//!
//! # Why a free function in this crate (rather than `Expr::evaluate_with_backend`)
//!
//! - `jolt-ir` is intentionally backend-agnostic. Adding a method that
//!   names a verifier-only trait would couple the IR crate to the verifier
//!   stack.
//! - The visitor lives where the backend abstraction lives, so all of the
//!   "lift native scalars into the backend" plumbing stays in one place.

use jolt_ir::{Expr, ExprVisitor, Var};

use crate::backend::FieldBackend;

/// Evaluates `expr` through `backend` against pre-wrapped openings and
/// challenges.
///
/// `openings[i]` and `challenges[j]` are the backend's wrapped scalars for
/// `Var::Opening(i as u32)` and `Var::Challenge(j as u32)` respectively.
/// They must be wrapped via the backend ahead of time (typically with
/// [`FieldBackend::wrap_proof`] / [`FieldBackend::wrap_challenge`]) so that
/// provenance labels are recorded by Tracing / R1CSGen backends.
///
/// Each [`Expr`] node is dispatched through the corresponding
/// [`FieldBackend`] arithmetic op:
///
/// - constant `c` → [`FieldBackend::const_i128`]
/// - variable lookup → indexed read from `openings` / `challenges`
/// - `-x` → [`FieldBackend::neg`]
/// - `a + b`, `a - b`, `a * b` → [`FieldBackend::add`] / `sub` / `mul`
///
/// Evaluation uses [`Expr::visit`], so any expression that already
/// satisfies the IR layer's traversal contract works unchanged.
///
/// # Panics
///
/// Panics if `Var::Opening(i)` or `Var::Challenge(i)` indexes outside the
/// supplied slices (matching the panic semantics of
/// [`Expr::evaluate`](jolt_ir::Expr::evaluate)).
pub fn evaluate_expr<B: FieldBackend>(
    backend: &mut B,
    expr: &Expr,
    openings: &[B::Scalar],
    challenges: &[B::Scalar],
) -> B::Scalar {
    let mut visitor = BackendEvaluator {
        backend,
        openings,
        challenges,
    };
    expr.visit(&mut visitor)
}

/// `ExprVisitor` adapter that forwards every node into a [`FieldBackend`].
struct BackendEvaluator<'a, B: FieldBackend> {
    backend: &'a mut B,
    openings: &'a [B::Scalar],
    challenges: &'a [B::Scalar],
}

impl<B: FieldBackend> ExprVisitor for BackendEvaluator<'_, B> {
    type Output = B::Scalar;

    #[inline]
    fn visit_constant(&mut self, val: i128) -> B::Scalar {
        self.backend.const_i128(val)
    }

    #[inline]
    fn visit_var(&mut self, var: Var) -> B::Scalar {
        match var {
            Var::Opening(id) => self.openings[id as usize].clone(),
            Var::Challenge(id) => self.challenges[id as usize].clone(),
        }
    }

    #[inline]
    fn visit_neg(&mut self, inner: B::Scalar) -> B::Scalar {
        self.backend.neg(&inner)
    }

    #[inline]
    fn visit_add(&mut self, lhs: B::Scalar, rhs: B::Scalar) -> B::Scalar {
        self.backend.add(&lhs, &rhs)
    }

    #[inline]
    fn visit_sub(&mut self, lhs: B::Scalar, rhs: B::Scalar) -> B::Scalar {
        self.backend.sub(&lhs, &rhs)
    }

    #[inline]
    fn visit_mul(&mut self, lhs: B::Scalar, rhs: B::Scalar) -> B::Scalar {
        self.backend.mul(&lhs, &rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::Native;
    use jolt_field::{Field, Fr};
    use jolt_ir::ExprBuilder;
    use rand_chacha::ChaCha8Rng;
    use rand_core::SeedableRng;

    fn wrap_slice<B: FieldBackend>(backend: &mut B, xs: &[B::F]) -> Vec<B::Scalar> {
        xs.iter().map(|x| backend.wrap_proof(*x, "x")).collect()
    }

    /// `evaluate_expr` over `Native` must agree with `Expr::evaluate` on every
    /// expression shape. This is the contract that lets us substitute backends
    /// without changing observable verifier behaviour.
    #[test]
    fn matches_expr_evaluate_for_canonical_shapes() {
        let mut rng = ChaCha8Rng::seed_from_u64(0xdead);
        for (name, expr, n_openings, n_challenges) in canonical_shapes() {
            for _ in 0..16 {
                let openings: Vec<Fr> = (0..n_openings).map(|_| Fr::random(&mut rng)).collect();
                let challenges: Vec<Fr> = (0..n_challenges).map(|_| Fr::random(&mut rng)).collect();

                let direct: Fr = expr.evaluate(&openings, &challenges);

                let mut backend = Native::<Fr>::new();
                let openings_w = wrap_slice(&mut backend, &openings);
                let challenges_w = wrap_slice(&mut backend, &challenges);
                let via_backend = evaluate_expr(&mut backend, &expr, &openings_w, &challenges_w);

                assert_eq!(direct, via_backend, "mismatch on {name}");
            }
        }
    }

    /// Constants and integer-literal arithmetic flow through `const_i128` —
    /// guard against any future backend that interprets `const_i128` differently
    /// from `Field::from_i128`.
    #[test]
    fn integer_literal_arithmetic() {
        let b = ExprBuilder::new();
        let h = b.opening(0);
        let expr = b.build(2i128 * h + 1);

        let mut backend = Native::<Fr>::new();
        let h_w = backend.wrap_proof(Fr::from_u64(10), "h");
        let result = evaluate_expr(&mut backend, &expr, &[h_w], &[]);
        assert_eq!(result, Fr::from_u64(21));
    }

    /// Sanity check: `evaluate_expr` honours `Var::Challenge` provenance
    /// (would silently swap with openings if the visitor mis-routed).
    #[test]
    fn opening_and_challenge_are_distinct() {
        let b = ExprBuilder::new();
        let o = b.opening(0);
        let c = b.challenge(0);
        let expr = b.build(o * o + c);

        let mut backend = Native::<Fr>::new();
        let o_w = backend.wrap_proof(Fr::from_u64(7), "o");
        let c_w = backend.wrap_challenge(Fr::from_u64(11), "c");
        let result = evaluate_expr(&mut backend, &expr, &[o_w], &[c_w]);
        assert_eq!(result, Fr::from_u64(7 * 7 + 11));
    }

    fn canonical_shapes() -> Vec<(&'static str, jolt_ir::Expr, usize, usize)> {
        vec![
            {
                let b = ExprBuilder::new();
                let c = b.constant(42);
                ("constant", b.build(c), 0, 0)
            },
            {
                let b = ExprBuilder::new();
                let h = b.opening(0);
                let g = b.challenge(0);
                ("booleanity", b.build(g * (h * h - h)), 1, 1)
            },
            {
                let b = ExprBuilder::new();
                let a = b.opening(0);
                let bv = b.opening(1);
                let c = b.opening(2);
                let d = b.opening(3);
                ("foil", b.build((a + bv) * (c + d)), 4, 0)
            },
            {
                let b = ExprBuilder::new();
                let alpha = b.challenge(0);
                let beta = b.challenge(1);
                let a = b.opening(0);
                let bv = b.opening(1);
                ("weighted_sum", b.build(alpha * a + beta * bv), 2, 2)
            },
            {
                let b = ExprBuilder::new();
                let a = b.opening(0);
                let bv = b.opening(1);
                ("neg_product", b.build(-(a * bv)), 2, 0)
            },
            {
                let b = ExprBuilder::new();
                let a = b.opening(0);
                let bv = b.opening(1);
                let c = b.opening(2);
                let d = b.opening(3);
                let e = b.opening(4);
                let f = b.opening(5);
                ("deep_nested", b.build(((a + bv) * c - d) * (e + f)), 6, 0)
            },
            {
                let b = ExprBuilder::new();
                let h = b.opening(0);
                ("integer_literals", b.build(2i128 * h + 1), 1, 0)
            },
        ]
    }
}
