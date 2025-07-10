use num_integer::div_floor;
use std::marker::PhantomData;

use crate::jolt::vm::stream::{FunctionStream, LagrangeBasisStream};
use crate::utils::math::Math;
use crate::utils::transcript::Transcript;
use crate::{
    field::JoltField,
    poly::{
        eq_poly::EqPolynomial,
        unipoly::{CompressedUniPoly, UniPoly},
    },
};

// Todoes:
// 1. Write more tests.
// 2. Write examples.
// 3. Write some benchmarks regarding the time and memory usages if you can.
// 4. Check for the API (should it be S1 and S2, Vector of S, or Vec<dyn S>)
// 5. Refactor eq function to make it faster.
// 6. Add documentation to the code.
// 7. Remove additional println functions, and if necessary, write some printerr.
// 8. Merge it with the current commit of the source code.

/// Holds the state and logic for the Product-sumcheck prover.
pub struct ProductSumcheckProver<F: JoltField, S1: FunctionStream<F>, S2: FunctionStream<F>> {
    first_stream: S1,
    second_stream: S2,
    num_rounds: usize,
    l: usize, // Time-constrained phase for rounds [1, l-1] and Space-constrained phase for the remaining rounds
    pub challenges: Vec<F>,
    pub compressed_polys: Vec<CompressedUniPoly<F>>,
    pub eval_points: Vec<(F, F, F)>,
    _phantom: PhantomData<F>,
}

impl<F, S1, S2> ProductSumcheckProver<F, S1, S2>
where
    F: JoltField,
    S1: FunctionStream<F>,
    S2: FunctionStream<F>,
{
    /// Creates a new Product-sumcheck prover.
    ///
    /// # Arguments
    ///
    /// * `k`: An arbitrary value for space and time tradeoff.
    /// * `n`: Number of variables in p and q (for now we assume they have the same number of variables).
    /// * `first_stream`: A stream for the multi-linear polynomial on the left-hand side (p in the paper).
    /// * `second_stream`: A stream for the multilinear polynomial on the right-hand side (q in the paper).
    pub fn new(k: usize, n: usize, first_stream: S1, second_stream: S2) -> Self {
        let l = div_floor(n, 2 * k); // As per step 1: l' = floor(n / 2k), assuming n is the total size and k=1 here.
        Self {
            first_stream,
            second_stream,
            num_rounds: n,
            l,
            challenges: Vec::new(),
            compressed_polys: Vec::new(),
            eval_points: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Executes the sumcheck protocol and generates the proof.
    pub fn prove<ProofTranscript: Transcript>(
        &mut self,
        external_randomness: Vec<F>,
        lc_external_randomness: F,
    ) {
        let (mut p, mut s, mut j_prime, mut w_j_prime);
        let mut new_state = Vec::new();
        for j in 1..=self.num_rounds {
            p = false;
            s = j.log_2();
            if j <= self.l - 1 {
                if (1 << s) == j {
                    p = true;
                }
                j_prime = 1 << s;
                w_j_prime = std::cmp::min(1 << s, self.num_rounds + 1 - j_prime);
                println!(
                    "Round {}: Time-constrained phase, P={}, j'={}, w(j')={}",
                    j, p, j_prime, w_j_prime
                );
            } else {
                if j % self.l == 0 {
                    p = true;
                }
                j_prime = self.l * div_floor(j, self.l);
                w_j_prime = std::cmp::min(self.l, self.num_rounds - j_prime + 1);
                println!(
                    "Round {}: Space-constrained phase, P={}, j'={}, w(j')={}",
                    j, p, j_prime, w_j_prime
                );
            }

            if p == true {
                new_state = self.compute_state(j, j_prime, w_j_prime, lc_external_randomness);
            }

            let (f_0, f_1, f_z) = self.compute_round_polynomials(j, j_prime, w_j_prime, &new_state);
            self.eval_points.push((f_0, f_1, f_z));
            let univariate_poly = UniPoly::from_evals(&[f_0, f_1, f_z]);
            let compressed_poly = univariate_poly.compress();
            // compressed_poly.append_to_transcript(transcript);
            self.compressed_polys.push(compressed_poly);
            // let r_j = transcript.challenge_scalar::<F>();
            let r_j = external_randomness[j - 1];
            self.challenges.push(r_j);
        }
    }

    /// Computes state M_j' when P = 1 (step e of ProductSC algorithm)
    fn compute_state(
        &mut self,
        j: usize,
        j_prime: usize,
        w_j_prime: usize,
        lc_external_randomness: F,
    ) -> Vec<Vec<F>> {
        assert_eq!(j, j_prime);
        println!(
            "Computing M_j' table for round j={}, j'={}, w(j')={}  -> ",
            j, j_prime, w_j_prime
        );
        // Let the challenges received so far be r ∈ F^(j'-1)
        // Your original slice
        let mut copied_challenge = self.challenges.clone();
        let r_challenges_mut: &mut [F] = &mut copied_challenge[0..j_prime - 1];
        r_challenges_mut.reverse();
        let reversed_slice: &[F] = r_challenges_mut;

        // Dimension of the state table
        let state_dimension = 1 << w_j_prime; // 2^w(j')
                                              // Initialize M_j'[β₁, β₂] table
        let mut m_j_prime_table: Vec<Vec<F>> =
            vec![vec![F::zero(); state_dimension]; state_dimension];

        let b_space_size = 1 << (self.num_rounds + 1 - j_prime - w_j_prime); // 2^(n-j'-w(j')+1)
        let beta_space_size = 1 << (w_j_prime); // 2^(w(j'))
        let x_space_size = 1 << (j_prime - 1); // 2^(w(j'));

        // Compute characteristic function χ_x(r)
        // let eq_r: Vec<F> = EqPolynomial::evals(&reversed_slice);
        let mut lagrange_basis = LagrangeBasisStream::new(&reversed_slice);

        // Step i: For each b ∈ {0, 1}^(n-j'-w(j')+1)
        for _ in 0..b_space_size {
            // Temporary storage for X[β] and Y[β]
            let mut x_values = vec![F::zero(); beta_space_size];
            let mut y_values = vec![F::zero(); beta_space_size];

            // Step ii: For each β ∈ {0, 1}^w(j')
            for beta in 0..beta_space_size {
                // Compute X[β] := p(r, β, b) = Σ_{x∈{0,1}^(j'-1)} χ_x(r) p(x, β, b)
                // Compute Y[β] := q(r, β, b) = Σ_{x∈{0,1}^(j'-1)} χ_x(r) q(x, β, b)
                let mut x_sum = F::zero();
                let mut y_sum = F::zero();
                for _ in 0..x_space_size {
                    let p_value = self.first_stream.next().unwrap();
                    let lagrange_base = lagrange_basis.next().unwrap();
                    x_sum += lagrange_base * p_value;

                    let q_value = self.second_stream.next().unwrap();
                    y_sum += lagrange_base * q_value;
                }
                lagrange_basis.reset();
                x_values[beta] = x_sum;
                y_values[beta] = y_sum;
            }

            // Step vi & vii: For each β₁, β₂ ∈ {0, 1}^w(j'), update M_j'[β₁, β₂]
            for beta1 in 0..beta_space_size {
                for beta2 in 0..beta_space_size {
                    m_j_prime_table[beta1][beta2] +=
                        x_values[beta1] * (lc_external_randomness + y_values[beta2]);
                }
            }
        }
        self.first_stream.reset();
        self.second_stream.reset();
        return m_j_prime_table;
    }

    /// Computes round polynomial evaluations f_j(0), f_j(1), f_j(z) (step g of ProductSC algorithm)
    fn compute_round_polynomials(
        &mut self,
        j: usize,
        j_prime: usize,
        w_j_prime: usize,
        m_j_prime_table: &[Vec<F>],
    ) -> (F, F, F) {
        println!(
            "Computing round polynomials for j={}, j'={}, w(j')={}",
            j, j_prime, w_j_prime
        );
        let r_prime = &self.challenges[j_prime - 1..j - 1]; // r' ∈ F^{j-j'}

        // Compute characteristic functions for r'
        let eq_r_prime: Vec<F> = EqPolynomial::evals(r_prime);

        // Dimensions for the computation
        let b1_b2_space_size = 1 << (j - j_prime); // 2^{j-j'} for b1, b2
        let b_space_size = 1 << (w_j_prime + j_prime - j - 1); // 2^{w(j')-j+j'-1} for b

        let mut f_0 = F::zero();
        let mut f_1 = F::zero();
        let mut f_z = F::zero();
        let z = F::from_u64(2);

        // Step (g): Compute round polynomial evaluations
        for b1 in 0..b1_b2_space_size {
            for b2 in 0..b1_b2_space_size {
                // Compute χ_{b1}(r') and χ_{b2}(r')
                let chi_product = eq_r_prime[b1] * eq_r_prime[b2];

                for b in 0..b_space_size {
                    // Compute indices: b1,0 := (b1, 0, b), b1,1 := (b1, 1, b), etc.
                    let b1_0 = self.concatenate_bits(
                        b,
                        w_j_prime + j_prime - j - 1,
                        0,
                        1,
                        b1,
                        j - j_prime,
                    );
                    let b1_1 = self.concatenate_bits(
                        b,
                        w_j_prime + j_prime - j - 1,
                        1,
                        1,
                        b1,
                        j - j_prime,
                    );
                    let b2_0 = self.concatenate_bits(
                        b,
                        w_j_prime + j_prime - j - 1,
                        0,
                        1,
                        b2,
                        j - j_prime,
                    );
                    let b2_1 = self.concatenate_bits(
                        b,
                        w_j_prime + j_prime - j - 1,
                        1,
                        1,
                        b2,
                        j - j_prime,
                    );

                    // Get M_j' values
                    let m_b1_0_b2_0 = m_j_prime_table[b1_0][b2_0];
                    let m_b1_0_b2_1 = m_j_prime_table[b1_0][b2_1];
                    let m_b1_1_b2_0 = m_j_prime_table[b1_1][b2_0];
                    let m_b1_1_b2_1 = m_j_prime_table[b1_1][b2_1];

                    // f_j(0) = Σ_{b1,b2} χ_{b1}(r') χ_{b2}(r') Σ_b M_{j'}[b1,0, b2,0]
                    f_0 += chi_product * m_b1_0_b2_0;

                    // f_j(1) = Σ_{b1,b2} χ_{b1}(r') χ_{b2}(r') Σ_b M_{j'}[b1,1, b2,1]
                    f_1 += chi_product * m_b1_1_b2_1;

                    // f_j(z) coefficients for: (1-z)^2, z(1-z), z(1-z), z^2
                    f_z += chi_product
                        * (m_b1_0_b2_0 * (F::one() - z).mul(F::one() - z)
                            + m_b1_0_b2_1 * (z.mul(F::one() - z))
                            + m_b1_1_b2_0 * (z.mul(F::one() - z))
                            + m_b1_1_b2_1 * (z.mul(z)));
                }
            }
        }
        (f_0, f_1, f_z)
    }

    /// Helper function to concatenate bit strings: (bits1, bits2, bits3) -> single index
    fn concatenate_bits(
        &self,
        bits1: usize,
        _len1: usize,
        bits2: usize,
        len2: usize,
        bits3: usize,
        len3: usize,
    ) -> usize {
        // Concatenate: bits1 || bits2 || bits3
        // bits1 takes the most significant positions, bits3 takes the least significant
        (bits1 << (len2 + len3)) | (bits2 << len3) | bits3
    }
}

// #[cfg(test)]
// mod tests {
//     use dory::arithmetic::Field;

//     use crate::jolt::vm::rv32i_vm::ProofTranscript;

//     use super::*;

//     #[test]
//     fn test_correct_computation_first_round() {
//         use ark_bn254::Fr;
//         let p_vec: Vec<Fr> = vec![2, 3, 1, 4, 3, 5, 2, 1, 4, 3, 1, 2, 2, 4, 5, 6]
//             .into_iter()
//             .map(Fr::from)
//             .collect();
//         let q_vec: Vec<Fr> = vec![1, 0, 4, 2, 0, 1, 2, 5, 3, 2, 0, 1, 1, 2, 4, 1]
//             .into_iter()
//             .map(Fr::from)
//             .collect();
//         let claim: Fr = p_vec.iter().zip(&q_vec).map(|(x, y)| *x * *y).sum();
//         assert_eq!(claim, Fr::from(84));
//         let mut product_sumcheck = ProductSumcheckProver::new(1, 4, p_vec, q_vec);
//         let state = product_sumcheck.compute_state(1, 1, 1, Fr::from(0));
//         assert_eq!(state.len(), 2);
//         assert_eq!((state[0].len(), state[1].len()), (2, 2));
//         assert_eq!(
//             (state[0][0], state[0][1], state[1][0], state[1][1]),
//             (Fr::from(44), Fr::from(33), Fr::from(58), Fr::from(40))
//         );

//         let polys = product_sumcheck.compute_round_polynomials(1, 1, 1, &state);
//         assert_eq!((polys.0, polys.1), (Fr::from(44), Fr::from(40)));
//     }

//     #[test]
//     fn test_correct_state_computation() {
//         use ark_bn254::Fr;
//         let p_vec: Vec<Fr> = vec![2, 3, 1, 4, 3, 5, 2, 1, 4, 3, 1, 2, 2, 4, 5, 6]
//             .into_iter()
//             .map(Fr::from)
//             .collect();
//         let q_vec: Vec<Fr> = vec![1, 0, 4, 2, 0, 1, 2, 5, 3, 2, 0, 1, 1, 2, 4, 1]
//             .into_iter()
//             .map(Fr::from)
//             .collect();
//         let claim: Fr = p_vec.iter().zip(&q_vec).map(|(x, y)| *x * *y).sum();
//         assert_eq!(claim, Fr::from(84));
//         let mut product_sumcheck = ProductSumcheckProver::new(1, 4, p_vec, q_vec);
//         product_sumcheck.challenges = vec![Fr::from(10), Fr::from(5), Fr::from(6), Fr::from(2)];
//         let state1 = product_sumcheck.compute_state(1, 1, 1, Fr::from(0));
//         assert_eq!(state1.len(), 2);
//         assert_eq!((state1[0].len(), state1[1].len()), (2, 2));
//         assert_eq!(
//             (state1[0][0], state1[0][1], state1[1][0], state1[1][1]),
//             (Fr::from(44), Fr::from(33), Fr::from(58), Fr::from(40))
//         );

//         let state2 = product_sumcheck.compute_state(2, 2, 2, Fr::from(0));
//         assert_eq!(state2.len(), 4);
//         assert_eq!((state2[0].len(), state2[1].len()), (4, 4));
//         assert_eq!(
//             (state2[0][0], state2[0][1], state2[1][0], state2[1][1]),
//             (Fr::from(44), Fr::from(33), Fr::from(58), Fr::from(40)),
//             "Failed at state 2"
//         );

//         let state3 = product_sumcheck.compute_state(3, 2, 2, Fr::from(0));
//         assert_eq!(state3.len(), 2);
//         assert_eq!((state3[0].len(), state3[1].len()), (2, 2));
//         assert_eq!(
//             (state3[0][0], state3[0][1], state3[1][0], state3[1][1]),
//             (Fr::from(44), Fr::from(33), Fr::from(58), Fr::from(40)),
//             "Failed at state 3"
//         );

//         let state4 = product_sumcheck.compute_state(4, 4, 1, Fr::from(0));
//         assert_eq!(state4.len(), 2);
//         assert_eq!((state4[0].len(), state4[1].len()), (2, 2));
//         assert_eq!(
//             (state4[0][0], state4[0][1], state4[1][0], state4[1][1]),
//             (Fr::from(44), Fr::from(33), Fr::from(58), Fr::from(40)),
//             "Failed at state 4"
//         );
//     }

//     #[test]
//     fn test_correct_computation() {
//         use ark_bn254::Fr;
//         let p_vec: Vec<Fr> = vec![2, 3, 1, 4, 3, 5, 2, 1, 4, 3, 1, 2, 2, 4, 5, 6]
//             .into_iter()
//             .map(Fr::from)
//             .collect();
//         let q_vec: Vec<Fr> = vec![1, 0, 4, 2, 0, 1, 2, 5, 3, 2, 0, 1, 1, 2, 4, 1]
//             .into_iter()
//             .map(Fr::from)
//             .collect();
//         let claim: Fr = p_vec.iter().zip(&q_vec).map(|(x, y)| *x * *y).sum();
//         assert_eq!(claim, Fr::from(84));
//         let mut product_sumcheck = ProductSumcheckProver::new(1, 4, p_vec, q_vec);
//         product_sumcheck.prove::<ProofTranscript>(vec![Fr::from(10), Fr::from(5), Fr::from(6), Fr::from(2)], Fr::zero());
//         assert_eq!(product_sumcheck.eval_points[0], (Fr::from(44), Fr::from(40), Fr::from(22)));
//         dbg!("eval points 1 is: {}", product_sumcheck.eval_points[1]);
//         dbg!("eval points 2 is: {}", product_sumcheck.eval_points[2]);
//         dbg!("eval points 3 is: {}", product_sumcheck.eval_points[3]);
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{field::JoltField, utils::transcript::KeccakTranscript};
//     use ark_bn254::Fr;
//     use std::collections::VecDeque;

//     /// Mock stream that returns predefined values for testing
//     #[derive(Clone)]
//     struct MockStream<F: JoltField> {
//         values: VecDeque<F>,
//     }

//     impl<F: JoltField> MockStream<F> {
//         fn new(values: Vec<F>) -> Self {
//             Self {
//                 values: values.into(),
//             }
//         }
//     }

//     impl<F: JoltField> FunctionStream<F> for MockStream<F> {
//         fn next(&mut self) -> Option<F> {
//             self.values.pop_front()
//         }

//         fn reset(&mut self) {
//             // For simplicity, we don't implement reset in this mock
//         }

//         fn into_vec(self) -> Vec<F> {
//             self.values.into()
//         }
//     }

//     #[test]
//     fn test_product_sumcheck_prover_basic() {
//         // Test parameters
//         let k = 1;
//         let n = 4; // 4 variables

//         // Create mock streams with simple values
//         let p_values: Vec<Fr> = (1..=16).map(|i| Fr::from(i)).collect();
//         let q_values: Vec<Fr> = (1..=16).map(|i| Fr::from(i * 2)).collect();

//         let p_stream = MockStream::new(p_values);
//         let q_stream = MockStream::new(q_values);

//         // Create prover
//         let mut prover = ProductSumcheckProver::new(k, n, p_stream, q_stream);

//         // Test initial state
//         assert_eq!(prover.num_rounds, n);
//         assert_eq!(prover.l, 2); // floor(4 / (2*1)) = 2
//         assert_eq!(prover.challenges.len(), 0);
//         assert_eq!(prover.compressed_polys.len(), 0);

//         // Add some mock challenges to test the polynomial computation
//         prover.challenges.push(Fr::from(3));
//         prover.challenges.push(Fr::from(5));
//         prover.challenges.push(Fr::from(7));

//         let mut transcript = KeccakTranscript::new(b"hello");

//         // Test prove function (this will generate prover messages)
//         prover.prove(&mut transcript);

//         // Check that prover messages were generated
//         assert!(prover.compressed_polys.len() > 0);
//         println!("Generated {} prover messages", prover.compressed_polys.len());
//     }

//     #[test]
//     fn test_compute_state_function() {
//         let k = 1;
//         let n = 3;

//         // Create simple mock streams
//         let p_values: Vec<Fr> = (1..=8).map(|i| Fr::from(i)).collect();
//         let q_values: Vec<Fr> = (1..=8).map(|i| Fr::from(i + 10)).collect();

//         let p_stream = MockStream::new(p_values);
//         let q_stream = MockStream::new(q_values);

//         let mut prover = ProductSumcheckProver::new(k, n, p_stream, q_stream);

//         // Add mock challenges
//         prover.challenges.push(Fr::from(2));

//         // Test compute_state
//         let j = 2;
//         let j_prime = 2;
//         let w_j_prime = 1;

//         let state = prover.compute_state(j, j_prime, w_j_prime);

//         // Check that state table has correct dimensions
//         let expected_dim = 1 << w_j_prime; // 2^w_j_prime
//         assert_eq!(state.len(), expected_dim);
//         assert_eq!(state[0].len(), expected_dim);

//         // Check that state values are not all zero (assuming our mock values produce non-zero results)
//         let has_non_zero = state
//             .iter()
//             .any(|row| row.iter().any(|&val| val != Fr::from(0)));
//         assert!(has_non_zero, "State should contain non-zero values");
//     }

//     #[test]
//     fn test_compute_round_polynomials() {
//         let k = 1;
//         let n = 3;

//         let p_stream = MockStream::new(vec![Fr::from(1); 8]);
//         let q_stream = MockStream::new(vec![Fr::from(2); 8]);

//         let mut prover = ProductSumcheckProver::new(k, n, p_stream, q_stream);

//         // Add mock challenges
//         prover.challenges.push(Fr::from(3));
//         prover.challenges.push(Fr::from(5));

//         // Create a mock state table
//         let state_dim = 4; // 2^2
//         let mock_state: Vec<Vec<Fr>> = (0..state_dim)
//             .map(|i| {
//                 (0..state_dim)
//                     .map(|j| Fr::from((i + j + 1) as u64))
//                     .collect()
//             })
//             .collect();

//         // Test compute_round_polynomials
//         let j = 3;
//         let j_prime = 2;
//         let w_j_prime = 2;

//         let (f_0, f_1, f_z) = prover.compute_round_polynomials(j, j_prime, w_j_prime, &mock_state);

//         // Check that polynomials are computed (non-zero values expected)
//         println!("f_0: {:?}, f_1: {:?}, f_z: {:?}", f_0, f_1, f_z);

//         // Basic sanity check - they should be finite field elements
//         // assert!(f_0.is_finite());
//         // assert!(f_1.is_finite());
//         // assert!(f_z.is_finite());
//     }

//     #[test]
//     fn test_concatenate_bits() {
//         let k = 1;
//         let n = 4;

//         let p_stream = MockStream::new(vec![Fr::from(1); 1]);
//         let q_stream = MockStream::new(vec![Fr::from(1); 1]);

//         let prover = ProductSumcheckProver::new(k, n, p_stream, q_stream);

//         // Test concatenate_bits function
//         let result = prover.concatenate_bits(5, 3, 3, 2, 9, 4);
//         assert_eq!(result, 377);

//         let result2 = prover.concatenate_bits(1, 1, 0, 1, 3, 2);
//         assert_eq!(result2, 11);
//     }

//     #[test]
//     fn test_time_vs_space_constrained_phases() {
//         let k = 2;
//         let n = 8;

//         let p_values: Vec<Fr> = (1..=256).map(|i| Fr::from(i)).collect();
//         let q_values: Vec<Fr> = (1..=256).map(|i| Fr::from(i * 3)).collect();

//         let p_stream = MockStream::new(p_values);
//         let q_stream = MockStream::new(q_values);

//         let mut prover = ProductSumcheckProver::new(k, n, p_stream, q_stream);

//         // l = floor(8 / (2*2)) = 2
//         assert_eq!(prover.l, 2);

//         // Add challenges for testing
//         for i in 1..=n {
//             prover.challenges.push(Fr::from(i as u64));
//         }

//         // Test that the phases are correctly determined
//         // Time-constrained: j <= l-1 = 1, so only j=1
//         // Space-constrained: j > l-1, so j=2,3,4,5,6,7,8

//         let mut transcript = KeccakTranscript::new(b"hello");
//         // This is tested implicitly in the prove() function
//         prover.prove(&mut transcript);

//         // Should have generated messages for all rounds
//         assert_eq!(prover.compressed_polys.len(), n);
//     }

//     #[test]
//     fn test_parameter_calculation() {
//         // Test different k and n values
//         let test_cases = vec![
//             (1, 4, 2),  // k=1, n=4, expected l=2
//             (2, 8, 2),  // k=2, n=8, expected l=2
//             (1, 6, 3),  // k=1, n=6, expected l=3
//             (3, 12, 2), // k=3, n=12, expected l=2
//         ];

//         for (k, n, expected_l) in test_cases {
//             let p_stream = MockStream::new(vec![Fr::from(1); 10]);
//             let q_stream = MockStream::new(vec![Fr::from(2); 10]);
//             let prover = ProductSumcheckProver::new(k, n, p_stream, q_stream);

//             assert_eq!(prover.l, expected_l, "Failed for k={}, n={}", k, n);
//             assert_eq!(prover.num_rounds, n);
//         }
//     }
// }
