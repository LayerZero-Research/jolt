//! Sumcheck verifier: checks round polynomials against the claimed sum.

use jolt_field::Field;
use jolt_poly::UnivariatePolynomial;
use jolt_transcript::{AppendToTranscript, Transcript};
use jolt_verifier_backend::{helpers::univariate_horner, FieldBackend};

use crate::claim::SumcheckClaim;
use crate::error::SumcheckError;
use crate::handler::{ClearRoundVerifier, RoundVerifier};
use crate::proof::SumcheckProof;

/// Stateless sumcheck verifier engine.
///
/// Replays the Fiat-Shamir transcript and checks each round against
/// the running sum, ultimately producing the final evaluation point
/// and expected value for an oracle query.
pub struct SumcheckVerifier;

impl SumcheckVerifier {
    /// Verifies a sumcheck proof using a pluggable round verifier.
    ///
    /// The verifier handler controls how per-round proof data is absorbed
    /// into the transcript and whether consistency checks are performed.
    /// Use [`ClearRoundVerifier`] for standard proofs or a committed
    /// verifier (from `jolt-blindfold`) for ZK proofs.
    ///
    /// On success, returns `(v, r)` where `v` is the final evaluation
    /// and `r = (r_1, ..., r_n)` is the challenge vector.
    ///
    /// # Errors
    ///
    /// Returns [`SumcheckError`] if the handler's consistency checks fail
    /// or the proof has the wrong number of rounds.
    #[tracing::instrument(skip_all, name = "SumcheckVerifier::verify")]
    pub fn verify_with_handler<F, T, V>(
        claim: &SumcheckClaim<F>,
        round_proofs: &[V::RoundProof],
        transcript: &mut T,
        verifier: &V,
    ) -> Result<(F, Vec<F>), SumcheckError>
    where
        F: Field,
        T: Transcript<Challenge = F>,
        V: RoundVerifier<F>,
    {
        if round_proofs.len() != claim.num_vars {
            return Err(SumcheckError::WrongNumberOfRounds {
                expected: claim.num_vars,
                got: round_proofs.len(),
            });
        }

        let mut running_sum = claim.claimed_sum;
        let mut challenges = Vec::with_capacity(claim.num_vars);

        for (round, round_proof) in round_proofs.iter().enumerate() {
            verifier.absorb_and_check(round_proof, running_sum, claim.degree, round, transcript)?;
            let r: F = transcript.challenge();
            running_sum = verifier.next_running_sum(round_proof, r);
            challenges.push(r);
        }

        Ok((running_sum, challenges))
    }

    /// Verifies a cleartext sumcheck proof.
    ///
    /// Convenience wrapper around [`verify_with_handler`](Self::verify_with_handler)
    /// using [`ClearRoundVerifier`].
    ///
    /// For each round $i = 0, \ldots, n-1$:
    /// 1. Checks that $\deg(s_i) \le d$ (the claim's degree bound).
    /// 2. Checks that $s_i(0) + s_i(1)$ equals the running sum.
    /// 3. Absorbs $s_i$ into the transcript and squeezes challenge $r_i$.
    /// 4. Sets the running sum to $s_i(r_i)$.
    ///
    /// On success, returns `(v, \mathbf{r})$ where $v = s_n(r_n)$ is the
    /// final evaluation and $\mathbf{r} = (r_1, \ldots, r_n)$ is the
    /// challenge vector.
    ///
    /// # Errors
    ///
    /// Returns [`SumcheckError`] if any round check fails, a degree bound
    /// is exceeded, or the proof has the wrong number of rounds.
    pub fn verify<F, T>(
        claim: &SumcheckClaim<F>,
        proof: &SumcheckProof<F>,
        transcript: &mut T,
    ) -> Result<(F, Vec<F>), SumcheckError>
    where
        F: Field,
        T: Transcript<Challenge = F>,
    {
        Self::verify_with_handler(
            claim,
            &proof.round_polynomials,
            transcript,
            &ClearRoundVerifier,
        )
    }

    /// Verifies a single-instance cleartext sumcheck proof through a [`FieldBackend`].
    ///
    /// Returns `(final_eval, challenges_w, challenges_f)`:
    ///
    /// - `final_eval` is the running sum after the last round, expressed as a
    ///   backend scalar.
    /// - `challenges_w` is the round-by-round challenge vector wrapped through
    ///   the backend (one [`B::Scalar`] per round).
    /// - `challenges_f` is the same vector, but as raw [`B::F`] values.
    ///
    /// `Native` carries the same data twice for free; `Tracing` needs both —
    /// the wrapped values feed downstream backend ops, while the raw values are
    /// needed by callers that drive the Fiat-Shamir transcript or feed legacy
    /// (non-backend) helpers.
    ///
    /// # Errors
    ///
    /// Mirrors [`verify`](Self::verify) for round-shape errors. Backend
    /// assertion failures bubble up as
    /// [`SumcheckError::RoundCheckFailed`].
    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip_all, name = "SumcheckVerifier::verify_with_backend")]
    pub fn verify_with_backend<B, T>(
        backend: &mut B,
        claim: &SumcheckClaim<B::F>,
        proof: &SumcheckProof<B::F>,
        running_sum_w: B::Scalar,
        transcript: &mut T,
    ) -> Result<(B::Scalar, Vec<B::Scalar>, Vec<B::F>), SumcheckError>
    where
        B: FieldBackend,
        T: Transcript<Challenge = B::F>,
    {
        if proof.round_polynomials.len() != claim.num_vars {
            return Err(SumcheckError::WrongNumberOfRounds {
                expected: claim.num_vars,
                got: proof.round_polynomials.len(),
            });
        }

        let zero_w = backend.const_zero();
        let one_w = backend.const_one();

        let mut running_w = running_sum_w;
        let mut challenges_w: Vec<B::Scalar> = Vec::with_capacity(claim.num_vars);
        let mut challenges_f: Vec<B::F> = Vec::with_capacity(claim.num_vars);

        for (round, round_proof) in proof.round_polynomials.iter().enumerate() {
            if round_proof.degree() > claim.degree {
                return Err(SumcheckError::DegreeBoundExceeded {
                    got: round_proof.degree(),
                    max: claim.degree,
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
                .assert_eq(&sum_w, &running_w, "sumcheck round consistency")
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

            running_w = univariate_horner(backend, &coeffs_w, &r_w);
            challenges_w.push(r_w);
            challenges_f.push(r_f);
        }

        Ok((running_w, challenges_w, challenges_f))
    }
}
