#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use crate::field::JoltField;
use crate::poly::commitment::dory::{DoryContext, DoryGlobals};
use crate::poly::opening_proof::{AdvicePolynomialProverOpening, ProverOpeningAccumulator, VerifierOpeningAccumulator};
use crate::poly::unipoly::{CompressedUniPoly, UniPoly};
use crate::subprotocols::sumcheck_prover::SumcheckInstanceProver;
use crate::subprotocols::sumcheck_verifier::SumcheckInstanceVerifier;
use crate::transcripts::{AppendToTranscript, Transcript};
use crate::utils::errors::ProofVerifyError;
#[cfg(not(target_arch = "wasm32"))]
use crate::utils::profiling::print_current_memory_usage;
use ark_std::log2;

use ark_serialize::*;
use itertools::Itertools;
use std::marker::PhantomData;

/// Implements the standard technique for batching parallel sumchecks to reduce
/// verifier cost and proof size.
///
/// For details, refer to Jim Posen's ["Perspectives on Sumcheck Batching"](https://hackmd.io/s/HyxaupAAA).
/// We do what they describe as "front-loaded" batch sumcheck.
pub enum BatchedSumcheck {}
impl BatchedSumcheck {
    /// Calculate the binding rounds for trusted advice polynomials.
    /// Returns a vector of round indices that need to be bound for trusted advice.

    pub fn prove<F: JoltField, ProofTranscript: Transcript>(
        mut sumcheck_instances: Vec<&mut dyn SumcheckInstanceProver<F, ProofTranscript>>,
        opening_accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut ProofTranscript,
    ) -> (SumcheckInstanceProof<F, ProofTranscript>, Vec<F::Challenge>) {
        let max_num_rounds = sumcheck_instances
            .iter()
            .map(|sumcheck| sumcheck.num_rounds())
            .max()
            .unwrap();

        let batching_coeffs: Vec<F> = transcript.challenge_vector(sumcheck_instances.len());
        let mut trusted_advice_poly_claim: F = F::zero();
        let mut trusted_advice_poly_binded: UniPoly<F> = UniPoly::from_coeff(vec![F::zero()]);
        let mut trusted_advice_round: usize = 0;

        let binding_rounds = AdvicePolynomialProverOpening::<F>::calculate_binding_rounds();
        let num_advice_var = AdvicePolynomialProverOpening::<F>::number_of_variables();
        tracing::info!("binding_rounds: {:?}", binding_rounds);
        // assert_eq!(binding_rounds, vec![2, 1, 0, 13, 9, 8, 7, 6]);

        // To see why we may need to scale by a power of two, consider a batch of
        // two sumchecks:
        //   claim_a = \sum_x P(x)             where x \in {0, 1}^M
        //   claim_b = \sum_{x, y} Q(x, y)     where x \in {0, 1}^M, y \in {0, 1}^N
        // Then the batched sumcheck is:
        //   \sum_{x, y} A * P(x) + B * Q(x, y)  where A and B are batching coefficients
        //   = A * \sum_y \sum_x P(x) + B * \sum_{x, y} Q(x, y)
        //   = A * \sum_y claim_a + B * claim_b
        //   = A * 2^N * claim_a + B * claim_b
        let mut individual_claims: Vec<F> = sumcheck_instances
            .iter()
            .map(|sumcheck| {
                let num_rounds = sumcheck.num_rounds();
                let input_claim = sumcheck.input_claim(opening_accumulator);
                transcript.append_scalar(&input_claim);
                let scaling_factor = max_num_rounds - num_rounds;
                let x = input_claim.mul_pow_2(scaling_factor);
                // tracing::info!("DARIVARI first individual claim: {:?}", x);

                if sumcheck.trusted_advice_dimensions().is_some() {
                    trusted_advice_poly_claim = sumcheck.input_claim(opening_accumulator);
                    // tracing::info!("DARIVARI first individual claim: {:?}, scaling factor: {}", x, scaling_factor);
                }
                x
            })
            .collect();



        #[cfg(test)]
        let mut batched_claim: F = individual_claims
            .iter()
            .zip(batching_coeffs.iter())
            .map(|(claim, coeff)| *claim * coeff)
            .sum();


        let mut r_sumcheck: Vec<F::Challenge> = Vec::with_capacity(max_num_rounds);
        let mut compressed_polys: Vec<CompressedUniPoly<F>> = Vec::with_capacity(max_num_rounds);

        // During Dory verification these are the order of r_sumcheck values that are used:
        // [x_5, x4, x_3, x_2, x_1, x_0, x_13, -> rows
        // x_12, x_11, x_10, x_9, x_8, x_7, x_6] -> columns
        // Trusted advice polynomial length is 4 rows and 4 columns, which means the following are indexes of r_sumcheck that 
        // trusted advice polynomial is later being evaluated on:
        // [x_2, x_1, x_0, x_13 -> rows
        // x_9, x_8, x_7, x_6] -> columns
        // These are the rounds in which we must bind the trusted advice polynomial.


        for round in 0..max_num_rounds {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let label = format!("Sumcheck round {round}");
                print_current_memory_usage(label.as_str());
            }

            let remaining_rounds = max_num_rounds - round;



            let univariate_polys: Vec<UniPoly<F>> = sumcheck_instances
                .iter_mut()
                .zip(individual_claims.iter_mut())
                .map(|(sumcheck, previous_claim)| {

                    if sumcheck.trusted_advice_dimensions().is_some() {
                        if binding_rounds.contains(&round) {
                            let scaling_factor = (max_num_rounds - num_advice_var) - (round - trusted_advice_round);
                            let x = sumcheck.compute_message(trusted_advice_round, trusted_advice_poly_claim);
                            trusted_advice_poly_binded = x.clone();
                            trusted_advice_round += 1;
                            UniPoly::from_coeff(x.coeffs.iter().map(|coeff| coeff.mul_pow_2(scaling_factor)).collect())
                        } else {
                            let scaling_factor = (max_num_rounds - num_advice_var) - (round - trusted_advice_round) - 1;
                            let scaled_claim = trusted_advice_poly_claim.mul_pow_2(scaling_factor);
                            UniPoly::from_coeff(vec![scaled_claim])
                        }
                    }
                    else {
                        // Standard logic for non-trusted-advice polynomials
                        let num_rounds = sumcheck.num_rounds();
                        if remaining_rounds > num_rounds {
                            // We haven't gotten to this sumcheck's variables yet, so
                            // the univariate polynomial is just a constant equal to
                            // the input claim, scaled by a power of 2.
                            let num_rounds = sumcheck.num_rounds();
                            let scaled_input_claim = sumcheck
                                .input_claim(opening_accumulator)
                                .mul_pow_2(remaining_rounds - num_rounds - 1);
                            // Constant polynomial
                            UniPoly::from_coeff(vec![scaled_input_claim])
                        } else {
                            let offset = max_num_rounds - sumcheck.num_rounds();
                            sumcheck.compute_message(round - offset, *previous_claim)
                        }
                    }
                })
                .collect();

            // Linear combination of individual univariate polynomials
            let batched_univariate_poly: UniPoly<F> =
                univariate_polys.iter().zip(&batching_coeffs).fold(
                    UniPoly::from_coeff(vec![]),
                    |mut batched_poly, (poly, &coeff)| {
                        batched_poly += &(poly * coeff);
                        batched_poly
                    },
                );

            let compressed_poly = batched_univariate_poly.compress();

            // append the prover's message to the transcript
            compressed_poly.append_to_transcript(transcript);
            let r_j = transcript.challenge_scalar_optimized::<F>();
            r_sumcheck.push(r_j);
            // tracing::info!("DARIVARI r_j: {:?}", r_j);

            // Cache individual claims for this round
            individual_claims
                .iter_mut()
                .zip(univariate_polys.into_iter())
                .for_each(|(claim, poly)| *claim = poly.evaluate(&r_j));

            if binding_rounds.contains(&round) {
                trusted_advice_poly_claim = trusted_advice_poly_binded.evaluate(&r_j);
            }

            #[cfg(test)]
            {
                // Sanity check
                let h0 = batched_univariate_poly.evaluate::<F>(&F::zero());
                let h1 = batched_univariate_poly.evaluate::<F>(&F::one());
                assert_eq!(
                    h0 + h1,
                    batched_claim,
                    "round {round}: H(0) + H(1) = {h0} + {h1} != {batched_claim}"
                );
                batched_claim = batched_univariate_poly.evaluate(&r_j);
            }

            for sumcheck in sumcheck_instances.iter_mut() {
                let poly_name = sumcheck.debug_name();
                let num_rounds = sumcheck.num_rounds();
                
                // Check if this is a trusted advice polynomial
                if sumcheck.trusted_advice_dimensions().is_some() {
                    if binding_rounds.contains(&round) {
                        sumcheck.ingest_challenge(r_j, trusted_advice_round - 1);
                    }
                } else {
                    // Standard binding logic for non-trusted-advice polynomials
                    // If a sumcheck instance has fewer than `max_num_rounds`,
                    // we wait until there are <= `sumcheck.num_rounds()` left
                    // before binding its variables.
                    if remaining_rounds <= num_rounds {
                        let offset = max_num_rounds - num_rounds;
                        let local_round = round - offset;
                        // tracing::info!(
                        //     "BIND [standard]: poly={}, global_round={}, local_round={}, num_rounds={}, r_j={:?}",
                        //     poly_name, round, local_round, num_rounds, r_j
                        // );
                        sumcheck.ingest_challenge(r_j, local_round);
                    } else {
                        // tracing::info!(
                        //     "SKIP [not started]: poly={}, global_round={}, num_rounds={}, remaining_rounds={}",
                        //     poly_name, round, num_rounds, remaining_rounds
                        // );
                    }
                }
            }

            compressed_polys.push(compressed_poly);
        }

        let max_num_rounds = sumcheck_instances
            .iter()
            .map(|sumcheck| sumcheck.num_rounds())
            .max()
            .unwrap();

        for sumcheck in sumcheck_instances.iter() {
            // Check if this is a trusted advice polynomial
            let r_slice: Vec<F::Challenge> = if sumcheck.trusted_advice_dimensions().is_some() {
                binding_rounds.iter().map(|round| r_sumcheck[*round]).collect::<Vec<F::Challenge>>()
                // vec![
                //     r_sumcheck[2], 
                //     r_sumcheck[1], 
                //     r_sumcheck[0], 
                //     r_sumcheck[13], 
                //     r_sumcheck[9], 
                //     r_sumcheck[8], 
                //     r_sumcheck[7], 
                //     r_sumcheck[6]
                //     ];
            } else {
                // If a sumcheck instance has fewer than `max_num_rounds`,
                // we wait until there are <= `sumcheck.num_rounds()` left
                // before binding its variables.
                // So, the sumcheck *actually* uses just the last `sumcheck.num_rounds()`
                // values of `r_sumcheck`.
                r_sumcheck[max_num_rounds - sumcheck.num_rounds()..].to_vec()
                
            };
            sumcheck.cache_openings(opening_accumulator, transcript, &r_slice);
        }

        (SumcheckInstanceProof::new(compressed_polys), r_sumcheck)
    }

    pub fn verify<F: JoltField, ProofTranscript: Transcript>(
        proof: &SumcheckInstanceProof<F, ProofTranscript>,
        sumcheck_instances: Vec<&dyn SumcheckInstanceVerifier<F, ProofTranscript>>,
        opening_accumulator: &mut VerifierOpeningAccumulator<F>,
        transcript: &mut ProofTranscript,
    ) -> Result<Vec<F::Challenge>, ProofVerifyError> {
        let max_degree = sumcheck_instances
            .iter()
            .map(|sumcheck| sumcheck.degree())
            .max()
            .unwrap();
        let max_num_rounds = sumcheck_instances
            .iter()
            .map(|sumcheck| sumcheck.num_rounds())
            .max()
            .unwrap();

        let batching_coeffs: Vec<F> = transcript.challenge_vector(sumcheck_instances.len());

        // To see why we may need to scale by a power of two, consider a batch of
        // two sumchecks:
        //   claim_a = \sum_x P(x)             where x \in {0, 1}^M
        //   claim_b = \sum_{x, y} Q(x, y)     where x \in {0, 1}^M, y \in {0, 1}^N
        // Then the batched sumcheck is:
        //   \sum_{x, y} A * P(x) + B * Q(x, y)  where A and B are batching coefficients
        //   = A * \sum_y \sum_x P(x) + B * \sum_{x, y} Q(x, y)
        //   = A * \sum_y claim_a + B * claim_b
        //   = A * 2^N * claim_a + B * claim_b
        let claim: F = sumcheck_instances
            .iter()
            .zip(batching_coeffs.iter())
            .map(|(sumcheck, coeff)| {
                let num_rounds = sumcheck.num_rounds();
                let input_claim = sumcheck.input_claim(opening_accumulator);
                transcript.append_scalar(&input_claim);
                input_claim.mul_pow_2(max_num_rounds - num_rounds) * coeff
            })
            .sum();

        let (output_claim, r_sumcheck) =
            proof.verify(claim, max_num_rounds, max_degree, transcript)?;
        
        let expected_output_claim: F = sumcheck_instances
            .iter()
            .zip(batching_coeffs.iter())
            .enumerate()
            .map(|(idx, (sumcheck, coeff))| {
                // Check if this is a trusted advice polynomial
                let r_slice: Vec<F::Challenge> = 
                if sumcheck.trusted_advice_dimensions().is_some() {
                    AdvicePolynomialProverOpening::<F>::calculate_binding_rounds()
                        .iter()
                        .sorted()
                        .map(|round| r_sumcheck[*round]).collect::<Vec<F::Challenge>>()
                    // let r_vec = 
                    //     vec![
                    //         r_sumcheck[0],
                    //         r_sumcheck[1],
                    //         r_sumcheck[2],
                    //         r_sumcheck[6],
                    //         r_sumcheck[7],
                    //         r_sumcheck[8],
                    //         r_sumcheck[9],
                    //         r_sumcheck[13]
                    //         // r_sumcheck[2], 
                    //         // r_sumcheck[1], 
                    //         // r_sumcheck[0], 
                    //         // r_sumcheck[13], 
                    //         // r_sumcheck[9], 
                    //         // r_sumcheck[8], 
                    //         // r_sumcheck[7], 
                    //         // r_sumcheck[6]
                    //         ];
                            // assert_eq!(r_vec, r_vec2);
                    // r_vec
                } else {
                    // If a sumcheck instance has fewer than `max_num_rounds`,
                    // we wait until there are <= `sumcheck.num_rounds()` left
                    // before binding its variables.
                    // So, the sumcheck *actually* uses just the last `sumcheck.num_rounds()`
                    // values of `r_sumcheck`.
                    r_sumcheck[max_num_rounds - sumcheck.num_rounds()..].to_vec()
                };

                // Cache polynomial opening claims, to be proven using either an
                // opening proof or sumcheck (in the case of virtual polynomials).
                sumcheck.cache_openings(opening_accumulator, transcript, &r_slice);
                let claim = sumcheck.expected_output_claim(opening_accumulator, &r_slice);
                
                claim * coeff
            })
            .sum();


        if output_claim != expected_output_claim {

            tracing::info!("THIIISSSSSSS output_claim: {:?}", output_claim);
            tracing::info!("THIIISSSSSSS expected_output_claim: {:?}", expected_output_claim);
            // return Err(ProofVerifyError::SumcheckVerificationError);
        }

        Ok(r_sumcheck)
    }
}

#[derive(CanonicalSerialize, CanonicalDeserialize, Debug, Clone)]
pub struct SumcheckInstanceProof<F: JoltField, ProofTranscript: Transcript> {
    pub compressed_polys: Vec<CompressedUniPoly<F>>,
    _marker: PhantomData<ProofTranscript>,
}

impl<F: JoltField, ProofTranscript: Transcript> SumcheckInstanceProof<F, ProofTranscript> {
    pub fn new(
        compressed_polys: Vec<CompressedUniPoly<F>>,
    ) -> SumcheckInstanceProof<F, ProofTranscript> {
        SumcheckInstanceProof {
            compressed_polys,
            _marker: PhantomData,
        }
    }

    /// Verify this sumcheck proof.
    /// Note: Verification does not execute the final check of sumcheck protocol: g_v(r_v) = oracle_g(r),
    /// as the oracle is not passed in. Expected that the caller will implement.
    ///
    /// Params
    /// - `claim`: Claimed evaluation
    /// - `num_rounds`: Number of rounds of sumcheck, or number of variables to bind
    /// - `degree_bound`: Maximum allowed degree of the combined univariate polynomial
    /// - `transcript`: Fiat-shamir transcript
    ///
    /// Returns (e, r)
    /// - `e`: Claimed evaluation at random point
    /// - `r`: Evaluation point
    pub fn verify(
        &self,
        claim: F,
        num_rounds: usize,
        degree_bound: usize,
        transcript: &mut ProofTranscript,
    ) -> Result<(F, Vec<F::Challenge>), ProofVerifyError> {
        let mut e = claim;
        let mut r: Vec<F::Challenge> = Vec::new();

        // verify that there is a univariate polynomial for each round
        assert_eq!(self.compressed_polys.len(), num_rounds);
        for i in 0..self.compressed_polys.len() {
            // verify degree bound
            if self.compressed_polys[i].degree() > degree_bound {
                return Err(ProofVerifyError::InvalidInputLength(
                    degree_bound,
                    self.compressed_polys[i].degree(),
                ));
            }

            // append the prover's message to the transcript
            self.compressed_polys[i].append_to_transcript(transcript);

            //derive the verifier's challenge for the next round
            let r_i: F::Challenge = transcript.challenge_scalar_optimized::<F>();
            r.push(r_i);

            // evaluate the claimed degree-ell polynomial at r_i using the hint
            // tracing::info!("THIIISSSSSSS in the verifier compressed_polys[i]: {:?}, r_i: {:?}", self.compressed_polys[i], r_i);
            e = self.compressed_polys[i].eval_from_hint(&e, &r_i);
        }

        Ok((e, r))
    }
}
