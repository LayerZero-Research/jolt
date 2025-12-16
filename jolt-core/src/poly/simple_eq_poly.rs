//! A simple (non-Gruen) Eq polynomial implementation with low-to-high binding.
//!
//! Eq(r_0, .., r_n, x_0, .., x_n) = ∏ (r_i * x_i + (1-r_i) * (1-x_i))
//!
//! This implementation stores all 2^n evaluations of Eq(r, x) for x in {0, 1}^n
//! and supports binding variables from low to high.

use allocative::Allocative;
use rayon::prelude::*;

use crate::{
    field::JoltField,
    poly::{eq_poly::EqPolynomial, unipoly::UniPoly},
};

/// A simple Eq polynomial that stores all evaluations and binds low-to-high.
///
/// After binding `k` variables, the polynomial has `2^(n-k)` evaluations.
#[derive(Debug, Clone, PartialEq, Allocative)]
pub struct SimpleEqPolynomial<F: JoltField> {
    /// The current evaluations of the Eq polynomial.
    /// After binding `k` variables with values (r_0, ..., r_{k-1}),
    /// this stores eq((r_0, ..., r_{k-1}, w_k, ..., w_{n-1}), (r_0, ..., r_{k-1}, x_k, ..., x_{n-1}))
    /// for all x in {0, 1}^{n-k}.
    pub evals: Vec<F>,
    /// The original point w = (w_0, ..., w_{n-1})
    pub w: Vec<F::Challenge>,
    /// Number of variables bound so far
    pub num_bound: usize,
}

impl<F: JoltField> SimpleEqPolynomial<F> {
    /// Create a new SimpleEqPolynomial from a point w.
    /// Computes eq(w, x) for all x in {0, 1}^n.
    #[tracing::instrument(skip_all, name = "SimpleEqPolynomial::new")]
    pub fn new(w: &[F::Challenge]) -> Self {
        let evals = EqPolynomial::evals(w);
        Self {
            evals,
            w: w.to_vec(),
            num_bound: 0,
        }
    }

    /// Create a new SimpleEqPolynomial with a scaling factor.
    #[tracing::instrument(skip_all, name = "SimpleEqPolynomial::new_with_scaling")]
    pub fn new_with_scaling(w: &[F::Challenge], scaling_factor: Option<F>) -> Self {
        let evals = EqPolynomial::evals_with_scaling(w, scaling_factor);
        Self {
            evals,
            w: w.to_vec(),
            num_bound: 0,
        }
    }

    /// Returns the number of variables in the original polynomial.
    pub fn get_num_vars(&self) -> usize {
        self.w.len()
    }

    /// Returns the current number of evaluations (2^remaining_vars).
    pub fn len(&self) -> usize {
        self.evals.len()
    }

    /// Returns true if there are no evaluations left.
    pub fn is_empty(&self) -> bool {
        self.evals.is_empty()
    }

    /// Returns the number of remaining (unbound) variables.
    pub fn remaining_vars(&self) -> usize {
        self.w.len() - self.num_bound
    }

    /// Bind the lowest unbound variable (low-to-high binding order).
    ///
    /// For each pair of adjacent evaluations (a, b), computes:
    ///   new_eval = a + r * (b - a) = (1-r)*a + r*b
    ///
    /// This is analogous to the binding in compact_polynomial.rs.
    #[tracing::instrument(skip_all, name = "SimpleEqPolynomial::bind")]
    pub fn bind(&mut self, r: F::Challenge) {
        let n = self.evals.len() / 2;
        
        // Low-to-high binding: adjacent pairs (2i, 2i+1)
        // new[i] = evals[2i] + r * (evals[2i+1] - evals[2i])
        //        = (1-r) * evals[2i] + r * evals[2i+1]
        self.evals = (0..n)
            .into_par_iter()
            .map(|i| {
                let a = self.evals[2 * i];
                let b = self.evals[2 * i + 1];
                a + (b - a) * r
            })
            .collect();
        
        self.num_bound += 1;
    }

    /// Get the evaluation at index `i`.
    pub fn get_eval(&self, i: usize) -> F {
        self.evals[i]
    }

    /// Get a reference to all current evaluations.
    pub fn evals(&self) -> &[F] {
        &self.evals
    }

    /// Compute the sumcheck polynomial by directly evaluating at points 0, 1, 2.
    ///
    /// For the lowest unbound variable X, computes:
    /// - s(0) = Σ f(0, x_rest) * eq(w, (0, x_rest))
    /// - s(1) = Σ f(1, x_rest) * eq(w, (1, x_rest))  
    /// - s(2) = Σ f(2, x_rest) * eq(w, (2, x_rest))  (via linear extrapolation)
    ///
    /// The function `f` maps index i to the polynomial evaluation at that index.
    pub fn compute_sumcheck_poly<G>(&self, f: G) -> UniPoly<F>
    where
        G: Fn(usize) -> F + Sync + Send,
    {
        let n = self.evals.len() / 2;
        
        // For each pair (2i, 2i+1), index 2i has X=0, index 2i+1 has X=1
        // We compute contributions to s(0), s(1), s(2)
        let (s_0, s_1, s_2): (F, F, F) = (0..n)
            .into_par_iter()
            .map(|i| {
                // eq values
                let eq_0 = self.evals[2 * i];     // eq when X=0
                let eq_1 = self.evals[2 * i + 1]; // eq when X=1
                let eq_2 = eq_1 + eq_1 - eq_0;    // eq when X=2 (linear extrapolation)
                
                // polynomial values
                let f_0 = f(2 * i);               // f when X=0
                let f_1 = f(2 * i + 1);           // f when X=1
                let f_2 = f_1 + f_1 - f_0;        // f when X=2 (linear extrapolation)
                
                (eq_0 * f_0, eq_1 * f_1, eq_2 * f_2)
            })
            .reduce(
                || (F::zero(), F::zero(), F::zero()),
                |(a0, a1, a2), (b0, b1, b2)| (a0 + b0, a1 + b1, a2 + b2)
            );
        
        UniPoly::from_evals(&[s_0, s_1, s_2])
    }
}

/// State wrapper for SimpleEqPolynomial to track binding progress in sumcheck.
/// This is analogous to `EqCycleState` but uses the simple (non-Gruen) Eq polynomial.
#[derive(Debug, Clone, Allocative)]
pub struct SimpleEqCycleState<F: JoltField> {
    /// The simple Eq polynomial storing eq(r', j)
    pub D: SimpleEqPolynomial<F>,
    /// The number of variables that have been bound during sumcheck so far
    pub num_variables_bound: usize,
}

impl<F: JoltField> SimpleEqCycleState<F> {
    /// Create a new SimpleEqCycleState from an opening point.
    #[tracing::instrument(skip_all, name = "SimpleEqCycleState::new")]
    pub fn new(r_cycle: &[F::Challenge]) -> Self {
        let D = SimpleEqPolynomial::new(r_cycle);
        Self {
            D,
            num_variables_bound: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poly::dense_mlpoly::DensePolynomial;
    use ark_bn254::Fr;
    use ark_ff::Zero;
    use ark_ff::One;
    use ark_std::test_rng;

    #[test]
    fn test_simple_eq_bind() {
        const NUM_VARS: usize = 10;
        let mut rng = test_rng();
        let w: Vec<<Fr as JoltField>::Challenge> =
            std::iter::repeat_with(|| <Fr as JoltField>::Challenge::random(&mut rng))
                .take(NUM_VARS)
                .collect();

        let mut regular_eq = DensePolynomial::<Fr>::new(EqPolynomial::evals(&w));
        let mut simple_eq = SimpleEqPolynomial::new(&w);
        
        // Verify they start equal
        assert_eq!(regular_eq.Z[..regular_eq.len()], simple_eq.evals[..]);

        // Bind all variables and verify equality after each bind
        for _ in 0..NUM_VARS {
            let r = <Fr as JoltField>::Challenge::random(&mut rng);
            regular_eq.bound_poly_var_bot(&r);
            simple_eq.bind(r);

            assert_eq!(regular_eq.Z[..regular_eq.len()], simple_eq.evals[..]);
        }
    }

    #[test]
    fn test_simple_eq_sumcheck_poly() {
        const NUM_VARS: usize = 6;
        let mut rng = test_rng();
        let w: Vec<<Fr as JoltField>::Challenge> =
            std::iter::repeat_with(|| <Fr as JoltField>::Challenge::random(&mut rng))
                .take(NUM_VARS)
                .collect();

        let simple_eq = SimpleEqPolynomial::<Fr>::new(&w);
        
        // Create a random polynomial to multiply with eq
        let poly_evals: Vec<Fr> = (0..(1 << NUM_VARS))
            .map(|_| Fr::random(&mut rng))
            .collect();
        
        // Compute the actual claim: sum of poly[i] * eq[i]
        let actual_claim: Fr = poly_evals
            .iter()
            .zip(simple_eq.evals.iter())
            .map(|(p, e)| *p * *e)
            .sum();
        
        // Compute the sumcheck polynomial
        let uni_poly = simple_eq.compute_sumcheck_poly(|i| poly_evals[i]);
        
        // Verify s(0) + s(1) = actual_claim
        let s_0 = uni_poly.evaluate(&Fr::zero());
        let s_1 = uni_poly.evaluate(&Fr::one());
        assert_eq!(s_0 + s_1, actual_claim);
    }
}

