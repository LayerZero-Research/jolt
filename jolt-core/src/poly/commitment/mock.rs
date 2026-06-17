use std::borrow::Borrow;
use std::marker::PhantomData;

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use crate::{
    field::JoltField,
    poly::commitment::commitment_scheme::PolynomialBatchSource,
    poly::multilinear_polynomial::MultilinearPolynomial,
    transcripts::Transcript,
    utils::{errors::ProofVerifyError, small_scalar::SmallScalar},
};

use super::commitment_scheme::CommitmentScheme;

#[derive(Clone)]
pub struct MockCommitScheme<F: JoltField> {
    _marker: PhantomData<F>,
}

#[derive(Default, Debug, PartialEq, Clone, CanonicalDeserialize, CanonicalSerialize)]
pub struct MockCommitment<F: JoltField> {
    _field: PhantomData<F>,
}

#[derive(CanonicalSerialize, CanonicalDeserialize, Clone, Debug)]
pub struct MockProof<F: JoltField> {
    opening_point: Vec<F::Challenge>,
}

impl<F> CommitmentScheme for MockCommitScheme<F>
where
    F: JoltField,
{
    type Field = F;
    type Config = ();
    type ProverSetup = ();
    type VerifierSetup = ();
    type Commitment = MockCommitment<F>;
    type Proof = MockProof<F>;
    type BatchedProof = MockProof<F>;
    type OpeningProofHint = ();
    type BatchOpeningHint = Vec<()>;

    fn setup_prover(_num_vars: usize) -> Self::ProverSetup {}

    fn setup_verifier(_setup: &Self::ProverSetup) -> Self::VerifierSetup {}

    fn commit(
        _poly: &MultilinearPolynomial<Self::Field>,
        _setup: &Self::ProverSetup,
    ) -> (Self::Commitment, Self::OpeningProofHint) {
        (MockCommitment::default(), ())
    }

    fn batch_commit<P>(
        source: &P,
        gens: &Self::ProverSetup,
    ) -> (Vec<Self::Commitment>, Self::BatchOpeningHint)
    where
        P: PolynomialBatchSource<Self::Field>,
    {
        (0..source.num_polys())
            .map(|idx| {
                let poly = source
                    .get_poly(idx)
                    .expect("mock batch_commit requires materialized polynomials");
                Self::commit(poly, gens)
            })
            .unzip()
    }

    fn combine_commitments<C: Borrow<Self::Commitment>>(
        _commitments: &[C],
        _coeffs: &[Self::Field],
    ) -> Self::Commitment {
        MockCommitment::default()
    }

    fn combine_hints(
        _hints: Vec<Self::OpeningProofHint>,
        _coeffs: &[Self::Field],
    ) -> Self::OpeningProofHint {
    }

    fn prove<ProofTranscript: Transcript>(
        _setup: &Self::ProverSetup,
        _poly: &MultilinearPolynomial<Self::Field>,
        opening_point: &[<Self::Field as JoltField>::Challenge],
        _hint: Option<Self::OpeningProofHint>,
        _transcript: &mut ProofTranscript,
    ) -> (Self::Proof, Option<Self::Field>) {
        (
            MockProof {
                opening_point: opening_point.to_owned(),
            },
            None,
        )
    }

    fn verify<ProofTranscript: Transcript>(
        proof: &Self::Proof,
        _setup: &Self::VerifierSetup,
        _transcript: &mut ProofTranscript,
        opening_point: &[<Self::Field as JoltField>::Challenge],
        _opening: &Self::Field,
        _commitment: &Self::Commitment,
    ) -> Result<(), ProofVerifyError> {
        assert_eq!(proof.opening_point, opening_point);
        Ok(())
    }

    fn protocol_name() -> &'static [u8] {
        b"mock_commit"
    }

    fn split_batch_hint(batch_hint: &Self::BatchOpeningHint) -> Vec<Self::OpeningProofHint> {
        batch_hint.clone()
    }
}

impl<F> super::commitment_scheme::StreamingCommitmentScheme for MockCommitScheme<F>
where
    F: JoltField,
{
    type ChunkState = ();

    fn process_chunk<T: SmallScalar>(_setup: &Self::ProverSetup, _chunk: &[T]) -> Self::ChunkState {
    }

    fn process_chunk_onehot(
        _setup: &Self::ProverSetup,
        _onehot_k: usize,
        _chunk: &[Option<usize>],
    ) -> Self::ChunkState {
    }

    fn aggregate_chunks(
        _setup: &Self::ProverSetup,
        _onehot_k: Option<usize>,
        _tier1_commitments: &[Self::ChunkState],
    ) -> (Self::Commitment, Self::OpeningProofHint) {
        (MockCommitment::default(), ())
    }

    fn streaming_batch_hint(hints: Vec<Self::OpeningProofHint>) -> Self::BatchOpeningHint {
        hints
    }
}
