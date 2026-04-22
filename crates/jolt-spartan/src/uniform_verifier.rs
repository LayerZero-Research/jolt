//! Verifier for uniform (repeated-constraint) Spartan proofs (pure PIOP).
//!
//! Uses the [`UniformSpartanKey`] sparse representation to evaluate matrix
//! MLEs, avoiding the dense MLE storage required by the standard verifier.
//!
//! The caller is responsible for appending the witness commitment to the
//! transcript before calling verify, and for checking the witness opening
//! proof after verify returns.

use jolt_field::Field;
use jolt_poly::EqPolynomial;
use jolt_sumcheck::{SumcheckClaim, SumcheckVerifier};
use jolt_transcript::Transcript;
use jolt_verifier_backend::{eq_eval, FieldBackend};

use crate::error::SpartanError;
use crate::uniform_key::UniformSpartanKey;
use crate::uniform_prover::UniformSpartanProof;

/// Stateless uniform Spartan verifier.
pub struct UniformSpartanVerifier;

impl UniformSpartanVerifier {
    /// Verifies a uniform Spartan proof (PIOP only — no PCS checks).
    ///
    /// The caller must:
    /// 1. Append the witness commitment to the transcript before calling this.
    /// 2. Verify the witness opening proof at `r_y` after this returns.
    ///
    /// # Protocol
    ///
    /// 1. Sample $\tau$ (commitment already absorbed by caller).
    /// 2. Verify outer sumcheck → obtain $r_x$.
    /// 3. Check: $\widetilde{eq}(r_x, \tau) \cdot (Az \cdot Bz - Cz) = v$.
    /// 4. Absorb evaluation claims, sample $\rho_A, \rho_B, \rho_C$.
    /// 5. Verify inner sumcheck → obtain $r_y$.
    /// 6. Evaluate matrix MLEs via the sparse key at $(r_x, r_y)$.
    /// 7. Check: $M(r_x, r_y) \cdot z(r_y) = v_{\text{inner}}$.
    #[tracing::instrument(skip_all, name = "UniformSpartanVerifier::verify")]
    pub fn verify<F, T>(
        key: &UniformSpartanKey<F>,
        proof: &UniformSpartanProof<F>,
        transcript: &mut T,
    ) -> Result<(), SpartanError>
    where
        F: Field,
        T: Transcript<Challenge = F>,
    {
        Self::verify_with_challenges(key, proof, transcript).map(|_| ())
    }

    /// Verifies a uniform Spartan proof and returns the challenge vectors.
    ///
    /// Same as [`verify`](Self::verify) but returns `(r_x, r_y)` — the outer
    /// and inner sumcheck challenge points. Downstream stages need `r_x` for
    /// eq-weighted sumcheck claims, and `r_y` is the witness evaluation point
    /// that the caller uses for PCS verification.
    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip_all, name = "UniformSpartanVerifier::verify_with_challenges")]
    pub fn verify_with_challenges<F, T>(
        key: &UniformSpartanKey<F>,
        proof: &UniformSpartanProof<F>,
        transcript: &mut T,
    ) -> Result<(Vec<F>, Vec<F>), SpartanError>
    where
        F: Field,
        T: Transcript<Challenge = F>,
    {
        let total_rows_padded = key.total_rows().next_power_of_two();
        let total_cols_padded = key.total_cols().next_power_of_two();
        let num_row_vars = total_rows_padded.trailing_zeros() as usize;
        let num_col_vars = total_cols_padded.trailing_zeros() as usize;

        let tau: Vec<F> = (0..num_row_vars).map(|_| transcript.challenge()).collect();

        let outer_claim = SumcheckClaim {
            num_vars: num_row_vars,
            degree: 3,
            claimed_sum: F::zero(),
        };

        let (outer_final_eval, r_x) =
            SumcheckVerifier::verify(&outer_claim, &proof.outer_sumcheck_proof, transcript)?;

        let eq_eval = EqPolynomial::new(tau).evaluate(&r_x);
        let expected = eq_eval * (proof.az_eval * proof.bz_eval - proof.cz_eval);
        if expected != outer_final_eval {
            return Err(SpartanError::OuterEvaluationMismatch);
        }

        transcript.append(&proof.az_eval);
        transcript.append(&proof.bz_eval);
        transcript.append(&proof.cz_eval);

        let rho_a: F = transcript.challenge();
        let rho_b: F = transcript.challenge();
        let rho_c: F = transcript.challenge();

        let inner_claim = SumcheckClaim {
            num_vars: num_col_vars,
            degree: 2,
            claimed_sum: rho_a * proof.az_eval + rho_b * proof.bz_eval + rho_c * proof.cz_eval,
        };

        let (inner_final_eval, r_y) =
            SumcheckVerifier::verify(&inner_claim, &proof.inner_sumcheck_proof, transcript)?;

        let (a_eval, b_eval, c_eval) = key.evaluate_matrix_mles(&r_x, &r_y);
        let combined_matrix_eval = rho_a * a_eval + rho_b * b_eval + rho_c * c_eval;

        if combined_matrix_eval * proof.witness_eval != inner_final_eval {
            return Err(SpartanError::InnerEvaluationMismatch);
        }

        Ok((r_x, r_y))
    }

    /// Backend-aware sibling of [`verify_with_challenges`](Self::verify_with_challenges).
    ///
    /// Mirrors the standard verifier's protocol step-by-step, but every
    /// algebraic operation routes through `backend`:
    ///
    /// 1. Sample $\tau$ natively, wrap as challenge scalars.
    /// 2. Verify the outer sumcheck via [`SumcheckVerifier::verify_with_backend`]
    ///    starting from `running_sum = 0` (wrapped as a constant zero).
    /// 3. Wrap `(az, bz, cz)` from the proof as `Proof` scalars and assert
    ///    `eq(τ, r_x) · (az·bz - cz) == outer_final_eval` through the backend.
    /// 4. Sample `(ρ_A, ρ_B, ρ_C)` natively, wrap as challenges.
    /// 5. Compute `inner_claim = ρ_A·az + ρ_B·bz + ρ_C·cz` through the backend
    ///    and feed it to the inner sumcheck.
    /// 6. Compute matrix MLEs through
    ///    [`UniformSpartanKey::evaluate_matrix_mles_with_backend`].
    /// 7. Wrap `witness_eval` and assert
    ///    `(ρ_A·a + ρ_B·b + ρ_C·c) · witness_eval == inner_final_eval`.
    ///
    /// Returns `(r_x_w, r_y_w, r_x_f, r_y_f, witness_eval_w)`:
    ///
    /// - `r_x_w` / `r_y_w`: outer / inner challenge vectors, backend handles.
    /// - `r_x_f` / `r_y_f`: same, raw field elements (callers need them to drive
    ///   the rest of the verifier and downstream PCS).
    /// - `witness_eval_w`: wrapped witness evaluation, ready to be plumbed into
    ///   the opening RLC reduction.
    ///
    /// # Errors
    ///
    /// Returns [`SpartanError`] for sumcheck shape failures or backend assertion
    /// failures. Backend assertion failures are converted to
    /// [`SpartanError::OuterEvaluationMismatch`] /
    /// [`SpartanError::InnerEvaluationMismatch`] depending on which check failed.
    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip_all, name = "UniformSpartanVerifier::verify_with_backend")]
    pub fn verify_with_backend<F, B, T>(
        backend: &mut B,
        key: &UniformSpartanKey<F>,
        proof: &UniformSpartanProof<F>,
        transcript: &mut T,
    ) -> Result<(Vec<B::Scalar>, Vec<B::Scalar>, Vec<F>, Vec<F>, B::Scalar), SpartanError>
    where
        F: Field,
        B: FieldBackend<F = F>,
        T: Transcript<Challenge = F>,
    {
        let total_rows_padded = key.total_rows().next_power_of_two();
        let total_cols_padded = key.total_cols().next_power_of_two();
        let num_row_vars = total_rows_padded.trailing_zeros() as usize;
        let num_col_vars = total_cols_padded.trailing_zeros() as usize;

        let tau_f: Vec<F> = (0..num_row_vars).map(|_| transcript.challenge()).collect();
        let tau_w: Vec<B::Scalar> = tau_f
            .iter()
            .map(|t| backend.wrap_challenge(*t, "spartan_tau"))
            .collect();

        let outer_claim = SumcheckClaim {
            num_vars: num_row_vars,
            degree: 3,
            claimed_sum: F::zero(),
        };
        let zero_w = backend.const_zero();
        let (outer_final_w, r_x_w, r_x_f) = SumcheckVerifier::verify_with_backend(
            backend,
            &outer_claim,
            &proof.outer_sumcheck_proof,
            zero_w,
            transcript,
        )?;

        let az_w = backend.wrap_proof(proof.az_eval, "spartan_az");
        let bz_w = backend.wrap_proof(proof.bz_eval, "spartan_bz");
        let cz_w = backend.wrap_proof(proof.cz_eval, "spartan_cz");

        let eq_tau_rx = eq_eval(backend, &tau_w, &r_x_w);
        let az_bz = backend.mul(&az_w, &bz_w);
        let az_bz_minus_cz = backend.sub(&az_bz, &cz_w);
        let outer_expected = backend.mul(&eq_tau_rx, &az_bz_minus_cz);
        backend
            .assert_eq(&outer_expected, &outer_final_w, "spartan outer check")
            .map_err(|_| SpartanError::OuterEvaluationMismatch)?;

        // Native Fiat-Shamir mirror of the legacy verifier — must match exactly.
        transcript.append(&proof.az_eval);
        transcript.append(&proof.bz_eval);
        transcript.append(&proof.cz_eval);

        let rho_a_f: F = transcript.challenge();
        let rho_b_f: F = transcript.challenge();
        let rho_c_f: F = transcript.challenge();
        let rho_a_w = backend.wrap_challenge(rho_a_f, "spartan_rho_a");
        let rho_b_w = backend.wrap_challenge(rho_b_f, "spartan_rho_b");
        let rho_c_w = backend.wrap_challenge(rho_c_f, "spartan_rho_c");

        let ra_az = backend.mul(&rho_a_w, &az_w);
        let rb_bz = backend.mul(&rho_b_w, &bz_w);
        let rc_cz = backend.mul(&rho_c_w, &cz_w);
        let inner_claim_sum_w = backend.add(&ra_az, &rb_bz);
        let inner_claim_sum_w = backend.add(&inner_claim_sum_w, &rc_cz);

        let inner_claim = SumcheckClaim {
            num_vars: num_col_vars,
            degree: 2,
            claimed_sum: F::zero(),
        };
        let (inner_final_w, r_y_w, r_y_f) = SumcheckVerifier::verify_with_backend(
            backend,
            &inner_claim,
            &proof.inner_sumcheck_proof,
            inner_claim_sum_w,
            transcript,
        )?;

        let (a_w, b_w, c_w) = key.evaluate_matrix_mles_with_backend(backend, &r_x_w, &r_y_w);
        let ra_a = backend.mul(&rho_a_w, &a_w);
        let rb_b = backend.mul(&rho_b_w, &b_w);
        let rc_c = backend.mul(&rho_c_w, &c_w);
        let combined_m = backend.add(&ra_a, &rb_b);
        let combined_m = backend.add(&combined_m, &rc_c);

        let witness_eval_w = backend.wrap_proof(proof.witness_eval, "spartan_witness_eval");
        let inner_expected = backend.mul(&combined_m, &witness_eval_w);
        backend
            .assert_eq(&inner_expected, &inner_final_w, "spartan inner check")
            .map_err(|_| SpartanError::InnerEvaluationMismatch)?;

        Ok((r_x_w, r_y_w, r_x_f, r_y_f, witness_eval_w))
    }
}
