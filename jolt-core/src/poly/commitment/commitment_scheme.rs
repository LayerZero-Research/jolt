use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::Zero;
use common::constants::ONEHOT_CHUNK_THRESHOLD_LOG_T;
use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::sync::Arc;

use crate::transcripts::Transcript;
use crate::{
    curve::JoltCurve,
    field::JoltField,
    poly::multilinear_polynomial::MultilinearPolynomial,
    poly::opening_proof::BatchOpeningState,
    poly::rlc_polynomial::{RLCStreamingData, TraceSource},
    utils::{errors::ProofVerifyError, small_scalar::SmallScalar},
    zkvm::{
        claim_reductions::PrecommittedPolynomial, config::OneHotParams,
        witness::CommittedPolynomial,
    },
};

pub trait PolynomialBatchSource<F: JoltField>: Sync {
    fn num_polys(&self) -> usize;

    fn get_poly(&self, _idx: usize) -> Option<&MultilinearPolynomial<F>> {
        None
    }

    fn onehot_index(&self, _cycle_idx: usize, _poly_idx: usize) -> Option<u8> {
        None
    }

    fn batch_onehot_indices(&self, cycle_idx: usize, poly_start: usize, buf: &mut [Option<u8>]) {
        for (i, slot) in buf.iter_mut().enumerate() {
            *slot = self.onehot_index(cycle_idx, poly_start + i);
        }
    }

    fn num_cycles(&self) -> Option<usize> {
        None
    }

    fn onehot_k(&self) -> Option<usize> {
        None
    }
}

impl<F: JoltField, U: Borrow<MultilinearPolynomial<F>> + Sync> PolynomialBatchSource<F> for [U] {
    fn num_polys(&self) -> usize {
        self.len()
    }

    fn get_poly(&self, idx: usize) -> Option<&MultilinearPolynomial<F>> {
        Some(self[idx].borrow())
    }
}

impl<F: JoltField, U: Borrow<MultilinearPolynomial<F>> + Sync> PolynomialBatchSource<F> for Vec<U> {
    fn num_polys(&self) -> usize {
        self.len()
    }

    fn get_poly(&self, idx: usize) -> Option<&MultilinearPolynomial<F>> {
        Some(self[idx].borrow())
    }
}

pub trait CommitmentScheme: Clone + Sync + Send + 'static {
    type Field: JoltField + Sized;
    type Config: Clone
        + Sync
        + Send
        + Debug
        + Default
        + PartialEq
        + CanonicalSerialize
        + CanonicalDeserialize;
    type ProverSetup: Clone + Sync + Send + Debug + CanonicalSerialize + CanonicalDeserialize;
    type VerifierSetup: Clone + Sync + Send + Debug + CanonicalSerialize + CanonicalDeserialize;
    type Commitment: Default
        + Debug
        + Sync
        + Send
        + PartialEq
        + CanonicalSerialize
        + CanonicalDeserialize
        + Clone;
    type Proof: Sync + Send + CanonicalSerialize + CanonicalDeserialize + Clone + Debug;
    type BatchedProof: Sync + Send + CanonicalSerialize + CanonicalDeserialize;
    /// A hint that helps the prover compute an opening proof. Typically some byproduct of
    /// the commitment computation, e.g. for Dory the Pedersen commitments to the rows can be
    /// used as a hint for the opening proof.
    type OpeningProofHint: Sync
        + Send
        + Clone
        + Debug
        + PartialEq
        + CanonicalSerialize
        + CanonicalDeserialize;
    type BatchOpeningHint: Sync + Send + Clone + Debug;

    /// Generates the prover setup for this PCS. `max_num_vars` is the maximum number of
    /// variables of any polynomial that will be committed using this setup.
    fn setup_prover(max_num_vars: usize) -> Self::ProverSetup;

    fn setup_prover_from_shape(
        max_log_t: usize,
        max_log_k: usize,
        log_packed: Option<usize>,
    ) -> Self::ProverSetup {
        let max_num_vars = max_log_t
            .checked_add(max_log_k)
            .and_then(|n| n.checked_add(log_packed.unwrap_or(0)))
            .expect("setup_prover_from_shape max_num_vars overflow");
        Self::setup_prover(max_num_vars)
    }

    /// Generates the verifier setup from the prover setup.
    fn setup_verifier(setup: &Self::ProverSetup) -> Self::VerifierSetup;

    fn active_config() -> Self::Config {
        Self::Config::default()
    }

    fn append_pcs_config_to_transcript<ProofTranscript: Transcript>(
        config: &Self::Config,
        transcript: &mut ProofTranscript,
    ) {
        transcript.append_serializable(b"pcs_config", config);
    }

    fn dory_layout(_config: &Self::Config) -> Option<crate::poly::commitment::dory::DoryLayout> {
        None
    }

    fn supports_committed_program() -> bool {
        true
    }

    fn prove_batch_opening<ProofTranscript: Transcript>(
        state: &BatchOpeningState<Self::Field>,
        context: BatchOpeningProverContext<'_, Self::Field, Self>,
        transcript: &mut ProofTranscript,
    ) -> (Self::Proof, Option<Self::Field>)
    where
        Self: Sized,
    {
        let (joint_poly, hint) = state.build_streaming_rlc::<Self>(
            context.one_hot_params,
            context.trace_source,
            context.rlc_streaming_data,
            context.opening_hints,
            context.precommitted_polys,
        );
        Self::prove(
            context.setup,
            &joint_poly,
            &state.opening_point,
            Some(hint),
            transcript,
        )
    }

    fn combine_batch_commitments(
        state: &BatchOpeningState<Self::Field>,
        commitment_map: &mut HashMap<CommittedPolynomial, Self::Commitment>,
    ) -> Result<Self::Commitment, ProofVerifyError> {
        let mut rlc_map = BTreeMap::new();
        for (gamma, (poly, _claim)) in state
            .gamma_powers
            .iter()
            .zip(state.polynomial_claims.iter())
        {
            *rlc_map.entry(*poly).or_insert(Self::Field::zero()) += *gamma;
        }

        let (coeffs, commitments): (Vec<Self::Field>, Vec<Self::Commitment>) = rlc_map
            .into_iter()
            .map(|(polynomial, coefficient)| {
                commitment_map
                    .remove(&polynomial)
                    .map(|commitment| (coefficient, commitment))
                    .ok_or_else(|| {
                        ProofVerifyError::DoryError(format!(
                            "missing commitment for Stage 8 polynomial {:?}",
                            polynomial
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();

        Ok(Self::combine_commitments(&commitments, &coeffs))
    }

    fn verify_batch_opening<ProofTranscript: Transcript>(
        proof: &Self::Proof,
        setup: &Self::VerifierSetup,
        transcript: &mut ProofTranscript,
        state: &BatchOpeningState<Self::Field>,
        commitment_map: &mut HashMap<CommittedPolynomial, Self::Commitment>,
        joint_claim: &Self::Field,
    ) -> Result<(), ProofVerifyError> {
        let joint_commitment = Self::combine_batch_commitments(state, commitment_map)?;
        Self::verify(
            proof,
            setup,
            transcript,
            &state.opening_point,
            joint_claim,
            &joint_commitment,
        )
    }

    /// Commits to a multilinear polynomial using the provided setup.
    ///
    /// # Arguments
    /// * `poly` - The multilinear polynomial to commit to
    /// * `setup` - The prover setup for the commitment scheme
    ///
    /// # Returns
    /// A tuple containing the commitment to the polynomial and a hint that can be used
    /// to optimize opening proof generation
    fn commit(
        poly: &MultilinearPolynomial<Self::Field>,
        setup: &Self::ProverSetup,
    ) -> (Self::Commitment, Self::OpeningProofHint);

    /// Commits to multiple multilinear polynomials in batch.
    ///
    /// # Arguments
    /// * `polys` - A slice of multilinear polynomials to commit to
    /// * `gens` - The prover setup for the commitment scheme
    ///
    /// # Returns
    /// A vector of commitments, one for each input polynomial
    fn batch_commit<S>(
        source: &S,
        gens: &Self::ProverSetup,
    ) -> (Vec<Self::Commitment>, Self::BatchOpeningHint)
    where
        S: PolynomialBatchSource<Self::Field>;

    /// Homomorphically combines multiple commitments into a single commitment, computed as a
    /// linear combination with the given coefficients.
    fn combine_commitments<C: Borrow<Self::Commitment>>(
        _commitments: &[C],
        _coeffs: &[Self::Field],
    ) -> Self::Commitment {
        todo!("`combine_commitments` should be on a separate `AdditivelyHomomorphic` trait")
    }

    /// Homomorphically combines multiple opening proof hints into a single hint, computed as a
    /// linear combination with the given coefficients.
    fn combine_hints(
        _hints: Vec<Self::OpeningProofHint>,
        _coeffs: &[Self::Field],
    ) -> Self::OpeningProofHint {
        unimplemented!()
    }

    /// Generates a proof of evaluation for a polynomial at a specific point.
    ///
    /// # Arguments
    /// * `setup` - The prover setup for the commitment scheme
    /// * `poly` - The multilinear polynomial being proved
    /// * `opening_point` - The point at which the polynomial is evaluated
    /// * `hint` - An optional hint that helps optimize the proof generation.
    ///   When `None`, implementations should compute the hint internally if needed.
    /// * `transcript` - The transcript for Fiat-Shamir transformation
    ///
    /// # Returns
    /// A tuple containing:
    /// - The proof of the polynomial evaluation at the specified point
    /// - An optional ZK blinding factor (y_blinding) for use in BlindFold; None for non-ZK schemes
    fn prove<ProofTranscript: Transcript>(
        setup: &Self::ProverSetup,
        poly: &MultilinearPolynomial<Self::Field>,
        opening_point: &[<Self::Field as JoltField>::Challenge],
        hint: Option<Self::OpeningProofHint>,
        transcript: &mut ProofTranscript,
    ) -> (Self::Proof, Option<Self::Field>);

    /// Verifies a proof of polynomial evaluation at a specific point.
    ///
    /// # Arguments
    /// * `proof` - The proof to be verified
    /// * `setup` - The verifier setup for the commitment scheme
    /// * `transcript` - The transcript for Fiat-Shamir transformation
    /// * `opening_point` - The point at which the polynomial is evaluated
    /// * `opening` - The claimed evaluation value of the polynomial at the opening point
    /// * `commitment` - The commitment to the polynomial
    ///
    /// # Returns
    /// Ok(()) if the proof is valid, otherwise a ProofVerifyError
    fn verify<ProofTranscript: Transcript>(
        proof: &Self::Proof,
        setup: &Self::VerifierSetup,
        transcript: &mut ProofTranscript,
        opening_point: &[<Self::Field as JoltField>::Challenge],
        opening: &Self::Field,
        commitment: &Self::Commitment,
    ) -> Result<(), ProofVerifyError>;

    fn protocol_name() -> &'static [u8];

    fn split_batch_hint(batch_hint: &Self::BatchOpeningHint) -> Vec<Self::OpeningProofHint>;

    fn packed_main_commitment_arity() -> Option<usize> {
        None
    }

    fn uses_onehot_inc() -> bool {
        false
    }

    fn log_k_chunk_for_trace(log_T: usize) -> usize {
        if log_T >= ONEHOT_CHUNK_THRESHOLD_LOG_T {
            8
        } else {
            4
        }
    }

    fn supported_log_k_chunks(max_log_k: usize) -> Vec<usize> {
        vec![max_log_k]
    }

    fn validate_batch_proof_shape(
        _proof: &Self::BatchedProof,
        _one_hot_log_k_chunk: usize,
    ) -> Result<(), ProofVerifyError> {
        Ok(())
    }
}

pub trait ZkEvalCommitment<C: JoltCurve>: CommitmentScheme {
    /// Returns the evaluation commitment (e.g. y_com) if present in the proof.
    fn eval_commitment(proof: &Self::Proof) -> Option<C::G1>;

    /// Returns the generators used for evaluation commitments in the prover setup.
    fn eval_commitment_gens(setup: &Self::ProverSetup) -> Option<(C::G1, C::G1)>;

    /// Returns the generators used for evaluation commitments in the verifier setup.
    fn eval_commitment_gens_verifier(setup: &Self::VerifierSetup) -> Option<(C::G1, C::G1)>;

    /// Extracts G1 generators and blinding generator from the prover setup for Pedersen commitments.
    /// Returns None for PCS that don't support ZK Pedersen commitments.
    #[cfg(feature = "zk")]
    fn zk_generators(_setup: &Self::ProverSetup, _count: usize) -> Option<(Vec<C::G1>, C::G1)> {
        None
    }
}

pub struct BatchOpeningProverContext<'a, F, PCS>
where
    F: JoltField,
    PCS: CommitmentScheme<Field = F>,
{
    pub setup: &'a PCS::ProverSetup,
    pub one_hot_params: OneHotParams,
    pub trace_source: TraceSource,
    pub rlc_streaming_data: Arc<RLCStreamingData>,
    pub batch_hint: PCS::BatchOpeningHint,
    pub opening_hints: HashMap<CommittedPolynomial, PCS::OpeningProofHint>,
    pub precommitted_polys: HashMap<CommittedPolynomial, PrecommittedPolynomial<F>>,
}

pub trait StreamingCommitmentScheme: CommitmentScheme {
    /// The type representing chunk state (tier 1 commitments)
    type ChunkState: Send + Sync + Clone + PartialEq + Debug;

    /// Compute tier 1 commitment for a chunk of small scalar values
    fn process_chunk<T: SmallScalar>(setup: &Self::ProverSetup, chunk: &[T]) -> Self::ChunkState;

    /// Compute tier 1 commitment for a chunk of one-hot values
    fn process_chunk_onehot(
        setup: &Self::ProverSetup,
        onehot_k: usize,
        chunk: &[Option<usize>],
    ) -> Self::ChunkState;

    /// Compute tier 2 commitment from accumulated tier 1 commitments
    fn aggregate_chunks(
        setup: &Self::ProverSetup,
        onehot_k: Option<usize>,
        tier1_commitments: &[Self::ChunkState],
    ) -> (Self::Commitment, Self::OpeningProofHint);

    fn aggregate_streaming_batch(
        _setup: &Self::ProverSetup,
        _onehot_ks: &[Option<usize>],
        _tier1_per_poly: &[Vec<Self::ChunkState>],
    ) -> Option<(Vec<Self::Commitment>, Self::BatchOpeningHint)> {
        None
    }

    fn streaming_batch_hint(hints: Vec<Self::OpeningProofHint>) -> Self::BatchOpeningHint;
}
