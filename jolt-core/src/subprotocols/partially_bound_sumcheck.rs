use crate::{field::JoltField, poly::{multilinear_polynomial::{BindingOrder, MultilinearPolynomial, PolynomialBinding}, unipoly::UniPoly}};



pub struct PartiallyBoundSumcheck<F: JoltField> {
    /// Multilinear polynomials over the *remaining* variables, in LowToHigh order.
    pub polys: Vec<MultilinearPolynomial<F>>,
    /// Degree bound in the current sumcheck stage (often 2 or 3 in Jolt).
    degree: usize,
    /// Expression being sumchecked, written in terms of the per-poly value at a point.
    /// Example: |v| v[0] * v[1] * (v[2] - v[3])
    expr: Box<dyn Fn(&[F]) -> F + Send + Sync>,
}
impl<F: JoltField> PartiallyBoundSumcheck<F> {
    pub fn new(
        remainder: Vec<Vec<F>>,
        degree: usize,
        expr: Box<dyn Fn(&[F]) -> F + Send + Sync>,
    ) -> Self {
        let polys = remainder
            .into_iter()
            .map(MultilinearPolynomial::<F>::from) // Dense (LargeScalars) is fine for small remainder
            .collect();
        Self { polys, degree, expr }
    }

    pub fn compute_message(&self, previous_claim: F) -> UniPoly<F> {
        assert!(!self.polys.is_empty());
        let len = self.polys[0].len();
        for p in &self.polys {
            assert_eq!(p.len(), len, "all remainder polys must have same length");
        }
        assert!(len.is_power_of_two());
        assert!(len >= 2, "need at least one variable left to sumcheck");

        // We’ll interpolate from evaluations at x = 0, 1, 2, ..., degree.
        // Enforce sumcheck constraint H(0)+H(1)=previous_claim by setting H(0)=previous_claim-H(1).
        let mut evals = vec![F::zero(); self.degree + 1];

        let half = len / 2;
        let mut vals = vec![F::zero(); self.polys.len()];

        // Compute H(t) for t=1..degree by summing over remaining vars except the current one.
        for i in 0..half {
            // For each poly, restrict current variable: f(t) = a + (b-a)*t
            // where a=f(0, rest), b=f(1, rest).
            let mut a = vec![F::zero(); self.polys.len()];
            let mut m = vec![F::zero(); self.polys.len()];
            for (j, p) in self.polys.iter().enumerate() {
                let f0 = p.get_bound_coeff(2 * i);
                let f1 = p.get_bound_coeff(2 * i + 1);
                a[j] = f0;
                m[j] = f1 - f0;
            }

            for t in 1..=self.degree {
                let tf = F::from_u64(t as u64);
                for j in 0..self.polys.len() {
                    vals[j] = a[j] + m[j] * tf;
                }
                evals[t] += (self.expr)(&vals);
            }
        }

        // Enforce the required consistency with the previous claim.
        evals[0] = previous_claim - evals[1];

        UniPoly::from_evals(&evals)
    }

    pub fn ingest_challenge(&mut self, r_j: F::Challenge) {
        for p in &mut self.polys {
            p.bind_parallel(r_j, BindingOrder::LowToHigh);
        }
    }

    pub fn final_poly_claims(&self) -> Vec<F> {
        // Useful for `cache_openings`: after all remaining rounds, each poly should have len==1.
        self.polys.iter().map(|p| p.final_sumcheck_claim()).collect()
    }
}
