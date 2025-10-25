use std::{cell::RefCell, rc::Rc};

use num_traits::Zero;

use crate::{
    field::{JoltField, MulTrunc},
    poly::{
        commitment::commitment_scheme::CommitmentScheme,
        multilinear_polynomial::{BindingOrder, MultilinearPolynomial, PolynomialBinding},
        opening_proof::{
            OpeningAccumulator, OpeningPoint, ProverOpeningAccumulator, SumcheckId,
            VerifierOpeningAccumulator, BIG_ENDIAN,
        },
    },
    subprotocols::sumcheck::SumcheckInstance,
    transcripts::Transcript,
    utils::math::Math,
    zkvm::{
        dag::state_manager::StateManager,
        witness::{CommittedPolynomial, VirtualPolynomial},
    },
};
use allocative::Allocative;
#[cfg(feature = "allocative")]
use allocative::FlameGraphBuilder;
use rayon::prelude::*;

#[derive(Allocative)]
pub struct HammingWeightProverState<F: JoltField> {
    ra: Vec<MultilinearPolynomial<F>>,
}

#[derive(Allocative)]
pub struct HammingWeightSumcheck<F: JoltField> {
    gamma: Vec<F>,
    log_K_chunk: usize,
    d: usize,
    prover_state: Option<HammingWeightProverState<F>>,
}

impl<F: JoltField> HammingWeightSumcheck<F> {
    #[tracing::instrument(skip_all, name = "BytecodeHammingWeightSumcheck::new_prover")]
    pub fn new_prover(
        sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
        F: Vec<Vec<F>>,
    ) -> Self {
        let d = sm.get_prover_data().0.shared.bytecode.d;
        let gamma: F = sm.transcript.borrow_mut().challenge_scalar();
        let mut gamma_powers = vec![F::one(); d];
        for i in 1..d {
            gamma_powers[i] = gamma_powers[i - 1] * gamma;
        }
        let log_K = sm.get_bytecode().len().log_2();
        let log_K_chunk = log_K.div_ceil(d);
        let ra = F
            .into_iter()
            .map(MultilinearPolynomial::from)
            .collect::<Vec<_>>();
        // tracing::debug!("BytecodeHammingWeightSumcheck MLP: {ra:?}");
        Self {
            gamma: gamma_powers,
            log_K_chunk,
            d,
            prover_state: Some(HammingWeightProverState { ra }),
        }
    }

    pub fn new_verifier(
        sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
    ) -> Self {
        let d = sm.get_verifier_data().0.shared.bytecode.d;
        let gamma: F = sm.transcript.borrow_mut().challenge_scalar();
        let mut gamma_powers = vec![F::one(); d];
        for i in 1..d {
            gamma_powers[i] = gamma_powers[i - 1] * gamma;
        }
        let log_K = sm.get_bytecode().len().log_2();
        let log_K_chunk = log_K.div_ceil(d);
        Self {
            gamma: gamma_powers,
            log_K_chunk,
            d,
            prover_state: None,
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstance<F, T> for HammingWeightSumcheck<F> {
    fn degree(&self) -> usize {
        1
    }

    fn num_rounds(&self) -> usize {
        self.log_K_chunk
    }

    fn input_claim(&self, _acc: Option<&RefCell<dyn OpeningAccumulator<F>>>) -> F {
        self.gamma.iter().sum()
    }

    #[tracing::instrument(skip_all, name = "BytecodeHammingWeight::compute_prover_message")]
    fn compute_prover_message(&mut self, _round: usize, _previous_claim: F) -> Vec<F> {
        let ps = self.prover_state.as_ref().unwrap();

        let prover_msg = ps
            .ra
            .par_iter()
            .zip(self.gamma.par_iter())
            .map(|(ra, gamma)| {
                let ra_sum = (0..ra.len() / 2)
                    .into_par_iter()
                    .map(|i| ra.get_bound_coeff(2 * i))
                    .fold_with(F::Unreduced::<5>::zero(), |running, new| {
                        running + new.as_unreduced_ref()
                    })
                    .reduce(F::Unreduced::zero, |running, new| running + new);
                ra_sum.mul_trunc::<4, 9>(gamma.as_unreduced_ref())
            })
            .reduce(F::Unreduced::zero, |running, new| running + new);

        vec![F::from_montgomery_reduce(prover_msg)]
    }

    #[tracing::instrument(skip_all, name = "BytecodeHammingWeight::bind")]
    fn bind(&mut self, r_j: F::Challenge, _round: usize) {
        self.prover_state
            .as_mut()
            .unwrap()
            .ra
            .par_iter_mut()
            .for_each(|ra| ra.bind_parallel(r_j, BindingOrder::LowToHigh))
    }

    fn expected_output_claim(
        &self,
        opening_accumulator: Option<Rc<RefCell<VerifierOpeningAccumulator<F>>>>,
        _r: &[F::Challenge],
    ) -> F {
        let opening_accumulator = opening_accumulator.as_ref().unwrap();
        self.gamma
            .iter()
            .enumerate()
            .map(|(i, gamma)| {
                let ra = opening_accumulator
                    .borrow()
                    .get_committed_polynomial_opening(
                        CommittedPolynomial::BytecodeRa(i),
                        SumcheckId::BytecodeHammingWeight,
                    )
                    .1;
                ra * gamma
            })
            .sum()
    }

    fn normalize_opening_point(
        &self,
        opening_point: &[F::Challenge],
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        OpeningPoint::new(opening_point.iter().rev().copied().collect())
    }

    fn cache_openings_prover(
        &self,
        accumulator: Rc<RefCell<ProverOpeningAccumulator<F>>>,
        transcript: &mut T,
        opening_point: OpeningPoint<BIG_ENDIAN, F>,
    ) {
        let ps = self.prover_state.as_ref().unwrap();
        let r_cycle = accumulator
            .borrow()
            .get_virtual_polynomial_opening(
                VirtualPolynomial::LookupOutput,
                SumcheckId::SpartanOuter,
            )
            .0
            .r
            .clone();
        let ra_claims = ps
            .ra
            .iter()
            .map(|ra| ra.final_sumcheck_claim())
            .collect::<Vec<F>>();
        accumulator.borrow_mut().append_sparse(
            transcript,
            (0..self.d).map(CommittedPolynomial::BytecodeRa).collect(),
            SumcheckId::BytecodeHammingWeight,
            opening_point.r.to_vec(),
            r_cycle,
            ra_claims,
        );
    }

    fn cache_openings_verifier(
        &self,
        accumulator: Rc<RefCell<VerifierOpeningAccumulator<F>>>,
        transcript: &mut T,
        opening_point: OpeningPoint<BIG_ENDIAN, F>,
    ) {
        let r_cycle = accumulator
            .borrow()
            .get_virtual_polynomial_opening(
                VirtualPolynomial::LookupOutput,
                SumcheckId::SpartanOuter,
            )
            .0
            .r
            .clone();
        let r = opening_point
            .r
            .iter()
            .cloned()
            .chain(r_cycle.iter().cloned())
            .collect::<Vec<_>>();
        accumulator.borrow_mut().append_sparse(
            transcript,
            (0..self.d).map(CommittedPolynomial::BytecodeRa).collect(),
            SumcheckId::BytecodeHammingWeight,
            r,
        );
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut FlameGraphBuilder) {
        flamegraph.visit_root(self);
    }

    fn dump_mlp(&self) {
        use ark_serialize::Compress;
        use std::fs::File;
        use std::io::Write;

        // Extract all BN254 values from the MLP polynomials
        let mut all_values: Vec<F> = Vec::new();

        if let Some(prover_state) = &self.prover_state {
            for poly in &prover_state.ra {
                // Extract values based on the polynomial type
                match poly {
                    MultilinearPolynomial::LargeScalars(dense_poly) => {
                        // Add all values from the dense polynomial
                        all_values.extend(&dense_poly.Z);
                    }
                    // TODO: Handle other polynomial types if needed [[memory:2757483]]
                    // For now, we only handle LargeScalars which contains DensePolynomial
                    _ => {
                        tracing::warn!("Unhandled polynomial type in dump_mlp: {:?}", poly);
                    }
                }
            }
        }

        // Get the count before using the values
        let value_count = all_values.len();

        // Create the output file
        std::fs::create_dir_all("dumps").ok();
        let filename = "dumps/1_mlp_bytecode_hamming_weight.txt";
        let mut file = match File::create(filename) {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Failed to create MLP dump file: {}", e);
                return;
            }
        };

        // Write the count of BN254 values (in decimal)
        if let Err(e) = writeln!(file, "{}", value_count) {
            tracing::error!("Failed to write to MLP dump file: {}", e);
            return;
        }

        // Write each BN254 value as hex (most significant byte first)
        for value in all_values {
            // Serialize the field element to bytes
            let mut bytes = Vec::new();
            if let Err(e) = value.serialize_with_mode(&mut bytes, Compress::No) {
                tracing::error!("Failed to serialize field element: {}", e);
                continue;
            }

            // BN254 field elements are 32 bytes (256 bits)
            // Convert to hex string with most significant byte first
            // The serialization is little-endian, so we need to reverse
            bytes.reverse();
            let hex_string = bytes
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<String>();

            if let Err(e) = writeln!(file, "{}", hex_string) {
                tracing::error!("Failed to write to MLP dump file: {}", e);
                return;
            }
        }

        tracing::info!("MLP dump written to {} ({} values)", filename, value_count);
    }
}
