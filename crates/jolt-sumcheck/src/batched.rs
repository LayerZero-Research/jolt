//! Batched sumcheck: reduces multiple claims into one via random linear
//! combination.
//!
//! Supports claims with **different** `num_vars` and `degree` bounds via
//! front-loaded batching: shorter instances are active only in the last
//! `num_vars` rounds and are padded with constant dummy polynomials in
//! earlier rounds. Each claim is scaled by $2^{N - n_i}$ where $N$ is the
//! maximum `num_vars` across all claims.

use jolt_field::Field;
use jolt_poly::{UnivariatePoly, UnivariatePolynomial};
use jolt_transcript::{AppendToTranscript, Transcript};
use jolt_verifier_backend::{helpers::univariate_horner, FieldBackend};

use crate::claim::SumcheckClaim;
use crate::error::SumcheckError;
use crate::handler::{ClearRoundHandler, ClearRoundVerifier, RoundHandler, RoundVerifier};
use crate::proof::SumcheckProof;
use crate::prover::SumcheckCompute;

/// Batched sumcheck prover that combines $m$ independent claims.
///
/// Given claims $C_0, \ldots, C_{m-1}$ over polynomials $g_0, \ldots, g_{m-1}$,
/// draws a batching coefficient $\alpha$ from the transcript and proves the
/// combined claim:
///
/// $$\sum_{j=0}^{m-1} \alpha^j \cdot 2^{N - n_j} \cdot C_j$$
///
/// where $N = \max_j n_j$ is the maximum number of variables. Claims with
/// fewer variables are front-padded with constant dummy rounds.
pub struct BatchedSumcheckProver;

impl BatchedSumcheckProver {
    /// Proves a batch of sumcheck claims with a pluggable round handler.
    ///
    /// The handler controls how combined round polynomials are absorbed
    /// into the transcript and what proof artifact is produced.
    ///
    /// # Panics
    ///
    /// Panics if `claims` is empty or if `claims` and `witnesses` have
    /// different lengths.
    #[tracing::instrument(skip_all, name = "BatchedSumcheckProver::prove")]
    pub fn prove_with_handler<F, T, H>(
        claims: &[SumcheckClaim<F>],
        witnesses: &mut [Box<dyn SumcheckCompute<F>>],
        transcript: &mut T,
        mut handler: H,
    ) -> H::Proof
    where
        F: Field,
        T: Transcript<Challenge = F>,
        H: RoundHandler<F>,
    {
        assert!(!claims.is_empty(), "must have at least one claim");
        assert_eq!(
            claims.len(),
            witnesses.len(),
            "claims and witnesses must have the same length"
        );

        let max_num_vars = claims.iter().map(|c| c.num_vars).max().unwrap();
        let max_degree = claims.iter().map(|c| c.degree).max().unwrap();

        // Fiat-Shamir: bind each claimed sum before deriving the batching
        // coefficient. This ensures alpha depends on all input claims.
        for claim in claims {
            claim.claimed_sum.append_to_transcript(transcript);
        }

        let alpha: F = transcript.challenge();

        let offsets: Vec<usize> = claims.iter().map(|c| max_num_vars - c.num_vars).collect();

        let mut individual_claims: Vec<F> = claims
            .iter()
            .zip(offsets.iter())
            .map(|(c, &offset)| c.claimed_sum.mul_pow_2(offset))
            .collect();

        let two_inv = (F::one() + F::one())
            .inverse()
            .expect("2 is invertible in any prime field of order > 2");

        for round in 0..max_num_vars {
            // Provide running claims to active witnesses before computing round polys.
            for (i, witness) in witnesses.iter_mut().enumerate() {
                let active = round >= offsets[i] && round < offsets[i] + claims[i].num_vars;
                if active {
                    witness.set_claim(individual_claims[i]);
                }
            }

            let instance_polys: Vec<UnivariatePoly<F>> = witnesses
                .iter()
                .enumerate()
                .map(|(i, witness)| {
                    let active = round >= offsets[i] && round < offsets[i] + claims[i].num_vars;
                    if active {
                        if round == offsets[i] {
                            witness
                                .first_round_polynomial()
                                .unwrap_or_else(|| witness.round_polynomial())
                        } else {
                            witness.round_polynomial()
                        }
                    } else {
                        UnivariatePoly::new(vec![individual_claims[i] * two_inv])
                    }
                })
                .collect();

            // Combine evaluations at points 0, 1, ..., max_degree with alpha weights.
            let num_points = max_degree + 1;
            let mut combined_evals = vec![F::zero(); num_points];
            let mut alpha_power = F::one();
            for poly in &instance_polys {
                for (t, combined) in combined_evals.iter_mut().enumerate() {
                    *combined += alpha_power * poly.evaluate(F::from_u64(t as u64));
                }
                alpha_power *= alpha;
            }

            let points: Vec<(F, F)> = combined_evals
                .into_iter()
                .enumerate()
                .map(|(t, y)| (F::from_u64(t as u64), y))
                .collect();
            let combined_poly = UnivariatePoly::interpolate(&points);

            handler.absorb_round_poly(&combined_poly, transcript);
            let challenge: F = transcript.challenge();
            handler.on_challenge(challenge);

            for (i, poly) in instance_polys.iter().enumerate() {
                individual_claims[i] = poly.evaluate(challenge);
            }

            for (i, witness) in witnesses.iter_mut().enumerate() {
                let active = round >= offsets[i] && round < offsets[i] + claims[i].num_vars;
                if active {
                    witness.bind(challenge);
                }
            }
        }

        handler.finalize()
    }

    /// Proves a batch of sumcheck claims with cleartext round handling.
    ///
    /// Convenience wrapper using [`ClearRoundHandler`].
    ///
    /// # Panics
    ///
    /// Panics if `claims` is empty or if `claims` and `witnesses` have
    /// different lengths.
    pub fn prove<F, T>(
        claims: &[SumcheckClaim<F>],
        witnesses: &mut [Box<dyn SumcheckCompute<F>>],
        transcript: &mut T,
    ) -> SumcheckProof<F>
    where
        F: Field,
        T: Transcript<Challenge = F>,
    {
        let max_num_vars = claims.iter().map(|c| c.num_vars).max().unwrap_or(0);
        Self::prove_with_handler(
            claims,
            witnesses,
            transcript,
            ClearRoundHandler::with_capacity(max_num_vars),
        )
    }
}

/// Batched sumcheck verifier.
///
/// Recomputes the combined claim with the same scaling and batching
/// coefficients as the prover, then delegates to the single-instance
/// verifier.
pub struct BatchedSumcheckVerifier;

impl BatchedSumcheckVerifier {
    /// Verifies a batched sumcheck proof with a pluggable round verifier.
    ///
    /// Returns `(v, r)` on success, where `v` is the combined final
    /// evaluation and `r` is the full challenge vector of length
    /// `max(num_vars)`.
    ///
    /// # Errors
    ///
    /// Returns [`SumcheckError`] if verification fails.
    #[tracing::instrument(skip_all, name = "BatchedSumcheckVerifier::verify")]
    pub fn verify_with_handler<F, T, V>(
        claims: &[SumcheckClaim<F>],
        round_proofs: &[V::RoundProof],
        transcript: &mut T,
        verifier: &V,
    ) -> Result<(F, Vec<F>), SumcheckError>
    where
        F: Field,
        T: Transcript<Challenge = F>,
        V: RoundVerifier<F>,
    {
        assert!(!claims.is_empty(), "must have at least one claim");

        let max_num_vars = claims.iter().map(|c| c.num_vars).max().unwrap();
        let max_degree = claims.iter().map(|c| c.degree).max().unwrap();

        // Fiat-Shamir: absorb claimed sums (must match prover).
        for claim in claims {
            claim.claimed_sum.append_to_transcript(transcript);
        }

        let alpha: F = transcript.challenge();

        let combined_sum: F = claims
            .iter()
            .enumerate()
            .fold(F::zero(), |acc, (j, claim)| {
                let scaled = claim.claimed_sum.mul_pow_2(max_num_vars - claim.num_vars);
                acc + pow(alpha, j) * scaled
            });

        let combined_claim = SumcheckClaim {
            num_vars: max_num_vars,
            degree: max_degree,
            claimed_sum: combined_sum,
        };

        crate::verifier::SumcheckVerifier::verify_with_handler(
            &combined_claim,
            round_proofs,
            transcript,
            verifier,
        )
    }

    /// Verifies a batched sumcheck proof with cleartext verification.
    ///
    /// Convenience wrapper using [`ClearRoundVerifier`].
    ///
    /// # Errors
    ///
    /// Returns [`SumcheckError`] if verification fails.
    pub fn verify<F, T>(
        claims: &[SumcheckClaim<F>],
        proof: &SumcheckProof<F>,
        transcript: &mut T,
    ) -> Result<(F, Vec<F>), SumcheckError>
    where
        F: Field,
        T: Transcript<Challenge = F>,
    {
        Self::verify_with_handler(
            claims,
            &proof.round_polynomials,
            transcript,
            &ClearRoundVerifier,
        )
    }

    /// Verifies a batched sumcheck proof through a [`FieldBackend`].
    ///
    /// Returns the same `(final_eval, challenges)` pair as [`verify`], but
    /// every field operation is routed through `backend`. The transcript is
    /// driven natively (Fiat-Shamir bytes are deterministic), but every
    /// arithmetic check, every polynomial evaluation, and every assertion is
    /// recorded by the backend.
    ///
    /// `Native` is a zero-overhead pass-through. `Tracing` records the entire
    /// verifier into an [`AstGraph`](jolt_verifier_backend::AstGraph) for
    /// recursion / Lean export / fuzzing. R1CS lowering will use the same
    /// trace in a future commit.
    ///
    /// The returned `final_eval`/`challenges` carry [`B::Scalar`] handles
    /// (concrete in `Native`, AST node ids in `Tracing`). The corresponding
    /// raw field values from the transcript are also returned so the caller
    /// can drive downstream PCS verification (which currently still uses the
    /// native group operations).
    ///
    /// # Errors
    ///
    /// Returns [`SumcheckError`] for round-shape errors. Backend assertion
    /// failures are bubbled up as [`SumcheckError::RoundCheckFailed`].
    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip_all, name = "BatchedSumcheckVerifier::verify_with_backend")]
    pub fn verify_with_backend<B, T>(
        backend: &mut B,
        claims: &[SumcheckClaim<B::F>],
        proof: &SumcheckProof<B::F>,
        transcript: &mut T,
    ) -> Result<(B::Scalar, Vec<B::Scalar>, Vec<B::F>), SumcheckError>
    where
        B: FieldBackend,
        T: Transcript<Challenge = B::F>,
    {
        assert!(!claims.is_empty(), "must have at least one claim");

        let max_num_vars = claims.iter().map(|c| c.num_vars).max().unwrap();
        let max_degree = claims.iter().map(|c| c.degree).max().unwrap();

        for claim in claims {
            claim.claimed_sum.append_to_transcript(transcript);
        }

        let alpha_f: B::F = transcript.challenge();
        let alpha_w = backend.wrap_challenge(alpha_f, "sumcheck_alpha");

        // combined = sum_j alpha^j * (claim_j.claimed_sum * 2^offset_j)
        let mut combined_w = backend.const_zero();
        let mut alpha_pow_w = backend.const_one();
        for claim in claims {
            let scaled_f = claim.claimed_sum.mul_pow_2(max_num_vars - claim.num_vars);
            let scaled_w = backend.wrap_public(scaled_f, "claimed_sum_scaled");
            let term = backend.mul(&alpha_pow_w, &scaled_w);
            combined_w = backend.add(&combined_w, &term);
            alpha_pow_w = backend.mul(&alpha_pow_w, &alpha_w);
        }

        if proof.round_polynomials.len() != max_num_vars {
            return Err(SumcheckError::WrongNumberOfRounds {
                expected: max_num_vars,
                got: proof.round_polynomials.len(),
            });
        }

        let zero_w = backend.const_zero();
        let one_w = backend.const_one();

        let mut running_sum_w = combined_w;
        let mut challenges_w: Vec<B::Scalar> = Vec::with_capacity(max_num_vars);
        let mut challenges_f: Vec<B::F> = Vec::with_capacity(max_num_vars);

        for (round, round_proof) in proof.round_polynomials.iter().enumerate() {
            if round_proof.degree() > max_degree {
                return Err(SumcheckError::DegreeBoundExceeded {
                    got: round_proof.degree(),
                    max: max_degree,
                });
            }

            let coeffs_w: Vec<B::Scalar> = round_proof
                .coefficients()
                .iter()
                .map(|c| backend.wrap_proof(*c, "round_poly_coeff"))
                .collect();

            let s_at_zero = univariate_horner(backend, &coeffs_w, &zero_w);
            let s_at_one = univariate_horner(backend, &coeffs_w, &one_w);
            let sum_w = backend.add(&s_at_zero, &s_at_one);

            backend
                .assert_eq(&sum_w, &running_sum_w, "sumcheck round consistency")
                .map_err(|e| SumcheckError::RoundCheckFailed {
                    round,
                    expected: format!("running_sum (round {round})"),
                    actual: e.to_string(),
                })?;

            for coeff in round_proof.coefficients() {
                coeff.append_to_transcript(transcript);
            }

            let r_f: B::F = transcript.challenge();
            let r_w = backend.wrap_challenge(r_f, "sumcheck_r");

            running_sum_w = univariate_horner(backend, &coeffs_w, &r_w);
            challenges_w.push(r_w);
            challenges_f.push(r_f);
        }

        Ok((running_sum_w, challenges_w, challenges_f))
    }
}

/// Computes $\text{base}^{\text{exp}}$ by repeated squaring.
#[inline]
fn pow<F: Field>(base: F, exp: usize) -> F {
    if exp == 0 {
        return F::one();
    }
    let mut result = F::one();
    let mut b = base;
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 {
            result *= b;
        }
        b = b.square();
        e >>= 1;
    }
    result
}
