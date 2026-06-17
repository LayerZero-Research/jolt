use std::borrow::Borrow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::slice::from_ref;

use akita_algebra::CyclotomicRing;
use akita_config::proof_optimized::fp128::D128Full;
use akita_config::CommitmentConfig;
use akita_field::CanonicalField;
use akita_pcs::AkitaCommitmentScheme;
use akita_prover::{
    AkitaPolyOps, CommitmentProver as AkitaCommitmentProver, CommittedPolynomials,
    ComputeBackendSetup, CpuBackend, DensePoly,
};
use akita_types::{
    CommitmentVerifier as AkitaCommitmentVerifier, CommittedOpenings, RingCommitment,
    SetupContributionMode,
};
use ark_ff::biginteger::S128;
use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use rayon::prelude::*;

use super::wrappers::{
    jolt_to_akita, AkitaProof, AkitaProverSetup, AkitaVerifierSetup, ArkBridge, Fp128,
    JoltToAkitaTranscript,
};
use crate::curve::JoltCurve;
use crate::field::fp128::JoltFp128;
use crate::field::JoltField;
use crate::poly::commitment::commitment_scheme::{
    BatchOpeningProverContext, CommitmentScheme, StreamingCommitmentScheme, ZkEvalCommitment,
};
use crate::poly::multilinear_polynomial::MultilinearPolynomial;
use crate::poly::opening_proof::BatchOpeningState;
use crate::transcripts::Transcript;
use crate::utils::errors::ProofVerifyError;
use crate::utils::small_scalar::SmallScalar;
use crate::zkvm::witness::CommittedPolynomial;

pub type Fp128Dense128Config = D128Full;

#[derive(Clone, Default)]
pub struct JoltAkitaCommitmentScheme<
    const D: usize,
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128>,
> {
    _cfg: PhantomData<Cfg>,
}

#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct JoltAkitaProof {
    proofs: Vec<ArkBridge<AkitaProof<Fp128>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JoltAkitaOpeningHint<const D: usize> {
    commitment: ArkBridge<RingCommitment<Fp128, D>>,
    akita_hint: ArkBridge<akita_types::AkitaCommitmentHint<Fp128, D>>,
    ring_coeffs: Vec<CyclotomicRing<Fp128, D>>,
}

impl<const D: usize> Valid for JoltAkitaOpeningHint<D> {
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

impl<const D: usize> CanonicalSerialize for JoltAkitaOpeningHint<D> {
    fn serialize_with_mode<W: std::io::Write>(
        &self,
        _writer: W,
        _compress: Compress,
    ) -> Result<(), SerializationError> {
        Ok(())
    }

    fn serialized_size(&self, _compress: Compress) -> usize {
        0
    }
}

impl<const D: usize> CanonicalDeserialize for JoltAkitaOpeningHint<D> {
    fn deserialize_with_mode<R: std::io::Read>(
        _reader: R,
        _compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        Err(SerializationError::IoError(std::io::Error::other(
            "Akita opening hints are prover-only",
        )))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AkitaChunkState<const D: usize> {
    Dense(Vec<Fp128>),
    OneHot {
        onehot_k: usize,
        indices: Vec<Option<u8>>,
    },
}

fn prepare_cpu<const D: usize>(
    setup: &AkitaProverSetup<Fp128, D>,
) -> <CpuBackend as ComputeBackendSetup<Fp128>>::PreparedSetup<D> {
    CpuBackend
        .prepare_setup(setup)
        .expect("Akita CPU backend setup preparation failed")
}

fn akita_prove_one<
    const D: usize,
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128>,
    P: AkitaPolyOps<Fp128, D>,
    ProofTranscript: akita_transcript::Transcript<Fp128>,
>(
    setup: &AkitaProverSetup<Fp128, D>,
    poly: &P,
    opening_point: &[Fp128],
    hint: akita_types::AkitaCommitmentHint<Fp128, D>,
    transcript: &mut ProofTranscript,
    commitment: &RingCommitment<Fp128, D>,
) -> AkitaProof<Fp128> {
    let prepared = prepare_cpu(setup);
    <AkitaCommitmentScheme<D, Cfg> as AkitaCommitmentProver<Fp128, D>>::batched_prove(
        setup,
        &CpuBackend,
        &prepared,
        (
            opening_point,
            vec![CommittedPolynomials {
                polynomials: from_ref(poly),
                commitment,
                hint,
            }],
        ),
        transcript,
        akita_types::BasisMode::Lagrange,
        SetupContributionMode::Direct,
    )
    .expect("Akita batched prove failed")
}

fn akita_verify_one<
    const D: usize,
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128>,
    ProofTranscript: akita_transcript::Transcript<Fp128>,
>(
    proof: &AkitaProof<Fp128>,
    setup: &AkitaVerifierSetup<Fp128>,
    transcript: &mut ProofTranscript,
    opening_point: &[Fp128],
    opening: &Fp128,
    commitment: &RingCommitment<Fp128, D>,
) -> Result<(), ProofVerifyError> {
    let openings = [*opening];
    <AkitaCommitmentScheme<D, Cfg> as AkitaCommitmentVerifier<Fp128, D>>::batched_verify(
        proof,
        setup,
        transcript,
        (
            opening_point,
            vec![CommittedOpenings {
                openings: &openings,
                commitment,
            }],
        ),
        akita_types::BasisMode::Lagrange,
        SetupContributionMode::Direct,
    )
    .map_err(|_| ProofVerifyError::InvalidOpeningProof)
}

fn to_akita_opening_point(point: &[JoltFp128]) -> Vec<Fp128> {
    point.iter().rev().map(jolt_to_akita).collect()
}

impl<const D: usize> JoltAkitaOpeningHint<D> {
    fn prove<ProofTranscript, Cfg>(
        self,
        setup: &AkitaProverSetup<Fp128, D>,
        opening_point: &[JoltFp128],
        transcript: &mut ProofTranscript,
    ) -> AkitaProof<Fp128>
    where
        ProofTranscript: akita_transcript::Transcript<Fp128>,
        Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128>,
    {
        let akita_point = to_akita_opening_point(opening_point);
        let poly = DensePoly::from_ring_coeffs(self.ring_coeffs);
        akita_prove_one::<D, Cfg, _, _>(
            setup,
            &poly,
            &akita_point,
            self.akita_hint.0,
            transcript,
            &self.commitment.0,
        )
    }
}

impl<const D: usize, Cfg> CommitmentScheme for JoltAkitaCommitmentScheme<D, Cfg>
where
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128> + Default,
{
    type Field = JoltFp128;
    type Config = ();
    type ProverSetup = ArkBridge<AkitaProverSetup<Fp128, D>>;
    type VerifierSetup = ArkBridge<AkitaVerifierSetup<Fp128>>;
    type Commitment = ArkBridge<RingCommitment<Fp128, D>>;
    type Proof = JoltAkitaProof;
    type BatchedProof = JoltAkitaProof;
    type OpeningProofHint = JoltAkitaOpeningHint<D>;

    fn setup_prover(max_num_vars: usize) -> Self::ProverSetup {
        ArkBridge(
            <AkitaCommitmentScheme<D, Cfg> as AkitaCommitmentProver<Fp128, D>>::setup_prover(
                max_num_vars,
                1,
            )
            .expect("Akita setup_prover failed"),
        )
    }

    fn setup_verifier(setup: &Self::ProverSetup) -> Self::VerifierSetup {
        ArkBridge(<AkitaCommitmentScheme<D, Cfg> as AkitaCommitmentProver<
            Fp128,
            D,
        >>::setup_verifier(&setup.0))
    }

    fn append_pcs_config_to_transcript<ProofTranscript: Transcript>(
        _config: &Self::Config,
        transcript: &mut ProofTranscript,
    ) {
        transcript.append_u64(b"akita_layout", 0);
    }

    fn commit(
        poly: &MultilinearPolynomial<JoltFp128>,
        setup: &Self::ProverSetup,
    ) -> (Self::Commitment, Self::OpeningProofHint) {
        let ring_coeffs = poly_to_ring_coeffs::<D>(poly);
        let poly = DensePoly::from_ring_coeffs(ring_coeffs.clone());
        let (commitment, akita_hint) = commit_akita_poly::<D, Cfg, _>(&setup.0, &poly);

        let commitment = ArkBridge(commitment);
        let hint = JoltAkitaOpeningHint {
            commitment: commitment.clone(),
            akita_hint: ArkBridge(akita_hint),
            ring_coeffs,
        };
        (commitment, hint)
    }

    fn batch_commit<U>(
        polys: &[U],
        setup: &Self::ProverSetup,
    ) -> Vec<(Self::Commitment, Self::OpeningProofHint)>
    where
        U: Borrow<MultilinearPolynomial<Self::Field>> + Sync,
    {
        polys
            .par_iter()
            .map(|poly| Self::commit(poly.borrow(), setup))
            .collect()
    }

    fn combine_commitments<C: Borrow<Self::Commitment>>(
        commitments: &[C],
        _coeffs: &[Self::Field],
    ) -> Self::Commitment {
        commitments
            .first()
            .map(|commitment| commitment.borrow().clone())
            .unwrap_or_default()
    }

    #[expect(
        clippy::expect_used,
        reason = "empty Akita hint combinations are invalid"
    )]
    fn combine_hints(
        hints: Vec<Self::OpeningProofHint>,
        _coeffs: &[Self::Field],
    ) -> Self::OpeningProofHint {
        hints
            .into_iter()
            .next()
            .expect("missing Akita opening hint")
    }

    fn prove<ProofTranscript: Transcript>(
        setup: &Self::ProverSetup,
        _poly: &MultilinearPolynomial<JoltFp128>,
        opening_point: &[JoltFp128],
        hint: Option<Self::OpeningProofHint>,
        transcript: &mut ProofTranscript,
    ) -> (Self::Proof, Option<Self::Field>) {
        let hint = hint.expect("Akita prove requires an opening hint");
        let mut adapter = JoltToAkitaTranscript::new(transcript);
        let proof = hint.prove::<_, Cfg>(&setup.0, opening_point, &mut adapter);
        (
            JoltAkitaProof {
                proofs: vec![ArkBridge(proof)],
            },
            None,
        )
    }

    fn prove_batch_opening<ProofTranscript: Transcript>(
        state: &BatchOpeningState<Self::Field>,
        mut context: BatchOpeningProverContext<'_, Self::Field, Self>,
        transcript: &mut ProofTranscript,
    ) -> (Self::Proof, Option<Self::Field>) {
        let proofs = state
            .individual_openings
            .iter()
            .enumerate()
            .map(|(index, opening)| {
                let hint = context
                    .opening_hints
                    .remove(&opening.polynomial)
                    .expect("missing Akita opening hint");
                transcript.append_bytes(b"akita_individual_item", &(index as u64).to_le_bytes());
                let mut adapter = JoltToAkitaTranscript::new(transcript);
                ArkBridge(hint.prove::<_, Cfg>(
                    &context.setup.0,
                    &opening.opening_point.r,
                    &mut adapter,
                ))
            })
            .collect();
        (JoltAkitaProof { proofs }, None)
    }

    fn verify<ProofTranscript: Transcript>(
        proof: &Self::Proof,
        setup: &Self::VerifierSetup,
        transcript: &mut ProofTranscript,
        opening_point: &[JoltFp128],
        opening: &JoltFp128,
        commitment: &Self::Commitment,
    ) -> Result<(), ProofVerifyError> {
        if proof.proofs.len() != 1 {
            return Err(ProofVerifyError::InvalidInputLength(1, proof.proofs.len()));
        }
        let akita_point = to_akita_opening_point(opening_point);
        let akita_opening = jolt_to_akita(opening);
        let mut adapter = JoltToAkitaTranscript::new(transcript);
        akita_verify_one::<D, Cfg, _>(
            &proof.proofs[0].0,
            &setup.0,
            &mut adapter,
            &akita_point,
            &akita_opening,
            &commitment.0,
        )
    }

    fn verify_batch_opening<ProofTranscript: Transcript>(
        proof: &Self::Proof,
        setup: &Self::VerifierSetup,
        transcript: &mut ProofTranscript,
        state: &BatchOpeningState<Self::Field>,
        commitment_map: &mut HashMap<CommittedPolynomial, Self::Commitment>,
        _joint_claim: &Self::Field,
    ) -> Result<(), ProofVerifyError> {
        if proof.proofs.len() != state.individual_openings.len() {
            return Err(ProofVerifyError::InvalidInputLength(
                state.individual_openings.len(),
                proof.proofs.len(),
            ));
        }

        for (index, (opening, proof)) in state
            .individual_openings
            .iter()
            .zip(&proof.proofs)
            .enumerate()
        {
            let commitment = commitment_map
                .remove(&opening.polynomial)
                .ok_or(ProofVerifyError::InvalidOpeningProof)?;
            let akita_point = to_akita_opening_point(&opening.opening_point.r);
            let akita_claim = jolt_to_akita(&opening.claim);
            transcript.append_bytes(b"akita_individual_item", &(index as u64).to_le_bytes());
            let mut adapter = JoltToAkitaTranscript::new(transcript);
            akita_verify_one::<D, Cfg, _>(
                &proof.0,
                &setup.0,
                &mut adapter,
                &akita_point,
                &akita_claim,
                &commitment.0,
            )?;
        }

        Ok(())
    }

    fn protocol_name() -> &'static [u8] {
        b"Akita"
    }
}

impl<const D: usize, Cfg> StreamingCommitmentScheme for JoltAkitaCommitmentScheme<D, Cfg>
where
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128> + Default,
{
    type ChunkState = AkitaChunkState<D>;

    fn process_chunk<T: SmallScalar>(_setup: &Self::ProverSetup, chunk: &[T]) -> Self::ChunkState {
        let coeffs = chunk
            .iter()
            .map(|value| {
                let field = value.to_field::<JoltFp128>();
                jolt_to_akita(&field)
            })
            .collect::<Vec<_>>();
        AkitaChunkState::Dense(coeffs)
    }

    fn process_chunk_onehot(
        _setup: &Self::ProverSetup,
        onehot_k: usize,
        chunk: &[Option<usize>],
    ) -> Self::ChunkState {
        AkitaChunkState::OneHot {
            onehot_k,
            indices: chunk.iter().map(|index| index.map(|i| i as u8)).collect(),
        }
    }

    fn aggregate_chunks(
        setup: &Self::ProverSetup,
        onehot_k: Option<usize>,
        tier1_commitments: &[Self::ChunkState],
    ) -> (Self::Commitment, Self::OpeningProofHint) {
        let ring_coeffs = match onehot_k {
            Some(onehot_k) => {
                let indices = tier1_commitments
                    .iter()
                    .flat_map(|chunk| match chunk {
                        AkitaChunkState::OneHot { indices, .. } => indices.clone(),
                        AkitaChunkState::Dense(_) => Vec::new(),
                    })
                    .collect::<Vec<_>>();
                pack_field_to_ring::<D>(&materialize_onehot_coeffs(onehot_k, &indices))
            }
            None => {
                let coeffs = tier1_commitments
                    .iter()
                    .flat_map(|chunk| match chunk {
                        AkitaChunkState::Dense(coeffs) => coeffs.clone(),
                        AkitaChunkState::OneHot { .. } => Vec::new(),
                    })
                    .collect::<Vec<_>>();
                pack_field_to_ring::<D>(&coeffs)
            }
        };

        let poly = DensePoly::from_ring_coeffs(ring_coeffs.clone());
        let (commitment, akita_hint) = commit_akita_poly::<D, Cfg, _>(&setup.0, &poly);
        let commitment = ArkBridge(commitment);
        let hint = JoltAkitaOpeningHint {
            commitment: commitment.clone(),
            akita_hint: ArkBridge(akita_hint),
            ring_coeffs,
        };
        (commitment, hint)
    }
}

impl<const D: usize, Cfg, C> ZkEvalCommitment<C> for JoltAkitaCommitmentScheme<D, Cfg>
where
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128> + Default,
    C: JoltCurve,
{
    fn eval_commitment(_proof: &Self::Proof) -> Option<C::G1> {
        None
    }

    fn eval_commitment_gens(_setup: &Self::ProverSetup) -> Option<(C::G1, C::G1)> {
        None
    }

    fn eval_commitment_gens_verifier(_setup: &Self::VerifierSetup) -> Option<(C::G1, C::G1)> {
        None
    }
}

fn commit_akita_poly<
    const D: usize,
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128>,
    P: AkitaPolyOps<Fp128, D>,
>(
    setup: &AkitaProverSetup<Fp128, D>,
    poly: &P,
) -> (
    RingCommitment<Fp128, D>,
    akita_types::AkitaCommitmentHint<Fp128, D>,
) {
    let prepared = prepare_cpu(setup);
    <AkitaCommitmentScheme<D, Cfg> as AkitaCommitmentProver<Fp128, D>>::commit(
        setup,
        &CpuBackend,
        &prepared,
        from_ref(poly),
    )
    .expect("Akita commit failed")
}

fn poly_to_ring_coeffs<const D: usize>(
    poly: &MultilinearPolynomial<JoltFp128>,
) -> Vec<CyclotomicRing<Fp128, D>> {
    match poly {
        MultilinearPolynomial::LargeScalars(p) => {
            let field_coeffs = p.Z.iter().map(jolt_to_akita).collect::<Vec<_>>();
            pack_field_to_ring::<D>(&field_coeffs)
        }
        MultilinearPolynomial::BoolScalars(p) => {
            pack_scalars::<D, _, _>(&p.coeffs, |&b| if b { Fp128::one() } else { Fp128::zero() })
        }
        MultilinearPolynomial::U8Scalars(p) => {
            pack_scalars::<D, _, _>(&p.coeffs, |&v| Fp128::from_u64(v as u64))
        }
        MultilinearPolynomial::U16Scalars(p) => {
            pack_scalars::<D, _, _>(&p.coeffs, |&v| Fp128::from_u64(v as u64))
        }
        MultilinearPolynomial::U32Scalars(p) => {
            pack_scalars::<D, _, _>(&p.coeffs, |&v| Fp128::from_u64(v as u64))
        }
        MultilinearPolynomial::U64Scalars(p) => {
            pack_scalars::<D, _, _>(&p.coeffs, |&v| Fp128::from_u64(v))
        }
        MultilinearPolynomial::U128Scalars(p) => pack_scalars::<D, _, _>(&p.coeffs, |&v| {
            <Fp128 as CanonicalField>::from_canonical_u128_reduced(v)
        }),
        MultilinearPolynomial::I64Scalars(p) => pack_scalars::<D, _, _>(&p.coeffs, |&v| {
            if v >= 0 {
                Fp128::from_u64(v as u64)
            } else {
                -Fp128::from_u64(v.unsigned_abs())
            }
        }),
        MultilinearPolynomial::I128Scalars(p) => {
            pack_scalars::<D, _, _>(&p.coeffs, |&v| jolt_to_akita(&JoltFp128::from_i128(v)))
        }
        MultilinearPolynomial::S128Scalars(p) => pack_scalars::<D, _, _>(&p.coeffs, s128_to_akita),
        MultilinearPolynomial::OneHot(onehot) => pack_field_to_ring::<D>(
            &materialize_onehot_coeffs(onehot.K, onehot.nonzero_indices.as_ref()),
        ),
        MultilinearPolynomial::RLC(_) => Vec::new(),
    }
}

fn materialize_onehot_coeffs(onehot_k: usize, indices: &[Option<u8>]) -> Vec<Fp128> {
    let T = indices.len();
    let mut coeffs = vec![Fp128::zero(); onehot_k * T];
    for (t, k) in indices.iter().enumerate() {
        if let Some(k) = k {
            coeffs[*k as usize * T + t] = Fp128::one();
        }
    }
    coeffs
}

fn pack_scalars<const D: usize, T: Sync, F: Fn(&T) -> Fp128 + Sync + Send>(
    scalars: &[T],
    convert: F,
) -> Vec<CyclotomicRing<Fp128, D>> {
    scalars
        .par_chunks(D * 256)
        .flat_map_iter(|big_chunk| {
            big_chunk.chunks(D).map(|chunk| {
                let mut coeffs = [Fp128::zero(); D];
                for (i, scalar) in chunk.iter().enumerate() {
                    coeffs[i] = convert(scalar);
                }
                CyclotomicRing::from_coefficients(coeffs)
            })
        })
        .collect()
}

fn pack_field_to_ring<const D: usize>(field_coeffs: &[Fp128]) -> Vec<CyclotomicRing<Fp128, D>> {
    field_coeffs
        .par_chunks(D)
        .map(|chunk| {
            let mut coeffs = [Fp128::zero(); D];
            coeffs[..chunk.len()].copy_from_slice(chunk);
            CyclotomicRing::from_coefficients(coeffs)
        })
        .collect()
}

fn s128_to_akita(value: &S128) -> Fp128 {
    if let Some(i) = value.to_i128() {
        jolt_to_akita(&JoltFp128::from_i128(i))
    } else {
        let magnitude = value.magnitude_as_u128();
        let field = <Fp128 as CanonicalField>::from_canonical_u128_reduced(magnitude);
        if value.is_positive {
            field
        } else {
            -field
        }
    }
}
