use std::borrow::Borrow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::slice::from_ref;

use super::packed_layout::{choose_packed_bit_layout, PackedBitLayout};
use super::packed_poly::build_packed_poly;
use akita_algebra::CyclotomicRing;
use akita_config::proof_optimized::fp128::D32OneHot;
use akita_config::CommitmentConfig;
use akita_field::CanonicalField;
use akita_pcs::AkitaCommitmentScheme;
use akita_prover::{
    AkitaPolyOps, CommitmentProver as AkitaCommitmentProver, CommittedPolynomials,
    ComputeBackendSetup, CpuBackend, DensePoly, OneHotPoly,
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
    BatchOpeningProverContext, CommitmentScheme, PolynomialBatchSource, StreamingCommitmentScheme,
    ZkEvalCommitment,
};
use crate::poly::eq_poly::EqPolynomial;
use crate::poly::multilinear_polynomial::MultilinearPolynomial;
use crate::poly::opening_proof::BatchOpeningState;
use crate::poly::rlc_polynomial::TraceSource;
use crate::transcripts::Transcript;
use crate::utils::errors::ProofVerifyError;
use crate::utils::small_scalar::SmallScalar;
use crate::zkvm::witness::{all_committed_polynomials, CommittedPolynomial};

pub type Fp128OneHot32Config = D32OneHot;

#[derive(Clone, Default)]
pub struct JoltAkitaCommitmentScheme<
    const D: usize,
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128>,
> {
    _cfg: PhantomData<Cfg>,
}

#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct JoltAkitaProof {
    packed_poly_proof: ArkBridge<AkitaProof<Fp128>>,
    num_packed_polys: u32,
    log_k: u32,
    individual_proofs: Vec<ArkBridge<AkitaProof<Fp128>>>,
}

#[derive(Clone, Debug)]
pub struct JoltAkitaBatchHint<const D: usize> {
    commitment: ArkBridge<RingCommitment<Fp128, D>>,
    akita_hint: ArkBridge<akita_types::AkitaCommitmentHint<Fp128, D>>,
    packed_layout: PackedBitLayout,
    num_packed_polys: usize,
    log_k: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JoltAkitaOpeningHint<const D: usize> {
    commitment: ArkBridge<RingCommitment<Fp128, D>>,
    akita_hint: ArkBridge<akita_types::AkitaCommitmentHint<Fp128, D>>,
    ring_coeffs: Vec<CyclotomicRing<Fp128, D>>,
    onehot_k: Option<usize>,
    onehot_indices: Vec<Option<u8>>,
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

fn to_akita_packed_opening_point(
    opening_point: &[JoltFp128],
    rho: &[JoltFp128],
    packed_layout: PackedBitLayout,
) -> Vec<Fp128> {
    let reversed = to_akita_opening_point(opening_point);
    let log_k = packed_layout.log_k();
    assert!(
        log_k <= reversed.len(),
        "packed opening point expects log_k <= num_vars"
    );
    let log_t = reversed.len() - log_k;
    let rho_le = rho.iter().rev().map(jolt_to_akita).collect::<Vec<_>>();
    packed_layout.reorder_packed_point(&reversed[..log_t], &reversed[log_t..], &rho_le)
}

fn choose_packed_layout_for_shape<const D: usize, Cfg>(
    log_k: usize,
    log_t: usize,
    log_packed: usize,
) -> PackedBitLayout
where
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128>,
{
    choose_packed_bit_layout::<D, Cfg>(log_k, log_t, log_packed)
}

fn choose_packed_layout_for_dims<const D: usize, Cfg>(
    num_cycles: usize,
    num_polys: usize,
    onehot_k: usize,
) -> PackedBitLayout
where
    Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128>,
{
    assert!(num_cycles.is_power_of_two());
    assert!(onehot_k.is_power_of_two());
    let log_k = onehot_k.trailing_zeros() as usize;
    let log_t = num_cycles.trailing_zeros() as usize;
    let log_packed = num_polys.next_power_of_two().trailing_zeros() as usize;
    choose_packed_layout_for_shape::<D, Cfg>(log_k, log_t, log_packed)
}

// Akita stores one-hot chunks as cycle-major `cycle * K + address`, while Jolt
// sumcheck openings are ordered as `[address, cycle]`.
fn to_akita_onehot_opening_point(point: &[JoltFp128], log_k: usize) -> Vec<Fp128> {
    let (r_address, r_cycle) = point.split_at(log_k);
    r_address
        .iter()
        .rev()
        .chain(r_cycle.iter().rev())
        .map(jolt_to_akita)
        .collect()
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
        if let Some(onehot_k) = self.onehot_k {
            debug_assert!(onehot_k.is_power_of_two());
            let akita_point =
                to_akita_onehot_opening_point(opening_point, onehot_k.trailing_zeros() as usize);
            let poly = OneHotPoly::<Fp128, D, u8>::new(onehot_k, self.onehot_indices)
                .expect("OneHotPoly construction failed");
            akita_prove_one::<D, Cfg, _, _>(
                setup,
                &poly,
                &akita_point,
                self.akita_hint.0,
                transcript,
                &self.commitment.0,
            )
        } else {
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
    type BatchOpeningHint = JoltAkitaBatchHint<D>;

    fn setup_prover(max_num_vars: usize) -> Self::ProverSetup {
        ArkBridge(
            <AkitaCommitmentScheme<D, Cfg> as AkitaCommitmentProver<Fp128, D>>::setup_prover(
                max_num_vars,
                1,
            )
            .expect("Akita setup_prover failed"),
        )
    }

    fn setup_prover_from_shape(
        max_log_t: usize,
        max_log_k: usize,
        log_packed: Option<usize>,
    ) -> Self::ProverSetup {
        let max_num_vars = if let Some(log_packed) = log_packed {
            choose_packed_layout_for_shape::<D, Cfg>(max_log_k, max_log_t, log_packed)
                .total_num_vars()
        } else {
            max_log_t + max_log_k
        };
        Self::setup_prover(max_num_vars)
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
        let (commitment, akita_hint, ring_coeffs, onehot_k, onehot_indices) =
            if let MultilinearPolynomial::OneHot(onehot) = poly {
                let indices = onehot.nonzero_indices.as_ref().clone();
                let poly = OneHotPoly::<Fp128, D, u8>::new(onehot.K, indices.clone())
                    .expect("OneHotPoly construction failed");
                let (commitment, hint) = commit_akita_poly::<D, Cfg, _>(&setup.0, &poly);
                (commitment, hint, Vec::new(), Some(onehot.K), indices)
            } else {
                let ring_coeffs = poly_to_ring_coeffs::<D>(poly);
                let poly = DensePoly::from_ring_coeffs(ring_coeffs.clone());
                let (commitment, hint) = commit_akita_poly::<D, Cfg, _>(&setup.0, &poly);
                (commitment, hint, ring_coeffs, None, Vec::new())
            };

        let commitment = ArkBridge(commitment);
        let hint = JoltAkitaOpeningHint {
            commitment: commitment.clone(),
            akita_hint: ArkBridge(akita_hint),
            ring_coeffs,
            onehot_k,
            onehot_indices,
        };
        (commitment, hint)
    }

    fn batch_commit<S>(
        source: &S,
        setup: &Self::ProverSetup,
    ) -> (Vec<Self::Commitment>, Self::BatchOpeningHint)
    where
        S: PolynomialBatchSource<Self::Field>,
    {
        let num_cycles = source
            .num_cycles()
            .expect("Akita batch_commit requires lazy source");
        let onehot_k = source
            .onehot_k()
            .expect("Akita batch_commit requires one-hot source");
        let num_polys = source.num_polys();
        let packed_layout =
            choose_packed_layout_for_dims::<D, Cfg>(num_cycles, num_polys, onehot_k);
        let index_fn = |cycle_idx: usize, poly_idx: usize| source.onehot_index(cycle_idx, poly_idx);
        let batch_fn = |cycle_idx: usize, poly_start: usize, buf: &mut [Option<u8>]| {
            source.batch_onehot_indices(cycle_idx, poly_start, buf);
        };
        let packed_poly =
            build_packed_poly(index_fn, batch_fn, num_cycles, num_polys, packed_layout);
        let (commitment, akita_hint) = commit_akita_poly::<D, Cfg, _>(&setup.0, &packed_poly);
        let commitment = ArkBridge(commitment);
        let hint = JoltAkitaBatchHint {
            commitment: commitment.clone(),
            akita_hint: ArkBridge(akita_hint),
            packed_layout,
            num_packed_polys: num_polys,
            log_k: onehot_k.trailing_zeros() as usize,
        };
        (vec![commitment], hint)
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
                packed_poly_proof: ArkBridge(proof),
                num_packed_polys: 1,
                log_k: opening_point.len() as u32,
                individual_proofs: Vec::new(),
            },
            None,
        )
    }

    fn prove_batch_opening<ProofTranscript: Transcript>(
        state: &BatchOpeningState<Self::Field>,
        mut context: BatchOpeningProverContext<'_, Self::Field, Self>,
        transcript: &mut ProofTranscript,
    ) -> (Self::Proof, Option<Self::Field>) {
        let main_polys = all_committed_polynomials(&context.one_hot_params, true);
        let mut claim_map = state
            .polynomial_claims
            .iter()
            .copied()
            .collect::<HashMap<_, _>>();
        let packed_claims = main_polys
            .iter()
            .map(|poly| {
                claim_map
                    .remove(poly)
                    .expect("missing packed Akita main claim")
            })
            .collect::<Vec<_>>();

        let individual_openings = state
            .individual_openings
            .iter()
            .filter(|opening| {
                matches!(
                    opening.polynomial,
                    CommittedPolynomial::TrustedAdvice | CommittedPolynomial::UntrustedAdvice
                )
            })
            .cloned()
            .collect::<Vec<_>>();

        let num_packed = context.batch_hint.num_packed_polys;
        assert_eq!(packed_claims.len(), num_packed);
        let selector_vars = num_packed.next_power_of_two().trailing_zeros() as usize;
        transcript.append_bytes(b"akita_packed_num", &(num_packed as u64).to_le_bytes());
        let rho = transcript.challenge_vector(selector_vars);
        let eq_table = EqPolynomial::<JoltFp128>::evals(&rho);
        let combined_claim = packed_claims
            .iter()
            .zip(eq_table.iter())
            .map(|(&claim, &eq)| claim * eq)
            .sum::<JoltFp128>();
        let packed_point = to_akita_packed_opening_point(
            &state.opening_point,
            &rho,
            context.batch_hint.packed_layout,
        );
        let akita_combined = jolt_to_akita(&combined_claim);
        transcript.append_bytes(
            b"akita_packed_claim",
            &akita_combined.to_canonical_u128().to_le_bytes(),
        );

        let trace = match &context.trace_source {
            TraceSource::Materialized(trace) => trace,
            TraceSource::Lazy(_) => panic!("Akita packed opening requires materialized trace"),
        };
        let index_fn = |cycle_idx: usize, poly_idx: usize| {
            main_polys[poly_idx].extract_index(
                &trace[cycle_idx],
                &context.rlc_streaming_data.bytecode,
                &context.rlc_streaming_data.memory_layout,
                &context.one_hot_params,
            )
        };
        let batch_fn = |cycle_idx: usize, poly_start: usize, buf: &mut [Option<u8>]| {
            for (i, slot) in buf.iter_mut().enumerate() {
                *slot = main_polys[poly_start + i].extract_index(
                    &trace[cycle_idx],
                    &context.rlc_streaming_data.bytecode,
                    &context.rlc_streaming_data.memory_layout,
                    &context.one_hot_params,
                );
            }
        };
        let packed_poly = build_packed_poly(
            index_fn,
            batch_fn,
            trace.len(),
            main_polys.len(),
            context.batch_hint.packed_layout,
        );
        let mut adapter = JoltToAkitaTranscript::new(transcript);
        let packed_proof = akita_prove_one::<D, Cfg, _, _>(
            &context.setup.0,
            &packed_poly,
            &packed_point,
            context.batch_hint.akita_hint.0,
            &mut adapter,
            &context.batch_hint.commitment.0,
        );

        let individual_proofs = individual_openings
            .iter()
            .enumerate()
            .map(|(index, opening)| {
                let hint = context
                    .opening_hints
                    .remove(&opening.polynomial)
                    .expect("missing Akita individual opening hint");
                transcript.append_bytes(b"akita_individual_item", &(index as u64).to_le_bytes());
                let mut adapter = JoltToAkitaTranscript::new(transcript);
                ArkBridge(hint.prove::<_, Cfg>(
                    &context.setup.0,
                    &opening.opening_point.r,
                    &mut adapter,
                ))
            })
            .collect();

        (
            JoltAkitaProof {
                packed_poly_proof: ArkBridge(packed_proof),
                num_packed_polys: num_packed as u32,
                log_k: context.batch_hint.log_k as u32,
                individual_proofs,
            },
            None,
        )
    }

    fn verify<ProofTranscript: Transcript>(
        proof: &Self::Proof,
        setup: &Self::VerifierSetup,
        transcript: &mut ProofTranscript,
        opening_point: &[JoltFp128],
        opening: &JoltFp128,
        commitment: &Self::Commitment,
    ) -> Result<(), ProofVerifyError> {
        let akita_point = to_akita_opening_point(opening_point);
        let akita_opening = jolt_to_akita(opening);
        let mut adapter = JoltToAkitaTranscript::new(transcript);
        akita_verify_one::<D, Cfg, _>(
            &proof.packed_poly_proof.0,
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
        let num_packed = proof.num_packed_polys as usize;
        if num_packed == 0 {
            return Err(ProofVerifyError::InvalidInputLength(1, 0));
        }
        let packed_claims = state
            .polynomial_claims
            .iter()
            .take(num_packed)
            .map(|(_, claim)| *claim)
            .collect::<Vec<_>>();
        let selector_vars = num_packed.next_power_of_two().trailing_zeros() as usize;
        transcript.append_bytes(b"akita_packed_num", &(num_packed as u64).to_le_bytes());
        let rho = transcript.challenge_vector(selector_vars);
        let eq_table = EqPolynomial::<JoltFp128>::evals(&rho);
        let combined_claim = packed_claims
            .iter()
            .zip(eq_table.iter())
            .map(|(&claim, &eq)| claim * eq)
            .sum::<JoltFp128>();
        let log_k = proof.log_k as usize;
        let log_t = state.opening_point.len().checked_sub(log_k).ok_or(
            ProofVerifyError::InvalidInputLength(log_k, state.opening_point.len()),
        )?;
        let packed_layout = choose_packed_layout_for_shape::<D, Cfg>(log_k, log_t, selector_vars);
        let packed_point = to_akita_packed_opening_point(&state.opening_point, &rho, packed_layout);
        let akita_combined = jolt_to_akita(&combined_claim);
        transcript.append_bytes(
            b"akita_packed_claim",
            &akita_combined.to_canonical_u128().to_le_bytes(),
        );
        let packed_commitment = commitment_map
            .remove(
                &state
                    .polynomial_claims
                    .first()
                    .ok_or(ProofVerifyError::InvalidOpeningProof)?
                    .0,
            )
            .ok_or(ProofVerifyError::InvalidOpeningProof)?;
        let mut adapter = JoltToAkitaTranscript::new(transcript);
        akita_verify_one::<D, Cfg, _>(
            &proof.packed_poly_proof.0,
            &setup.0,
            &mut adapter,
            &packed_point,
            &akita_combined,
            &packed_commitment.0,
        )?;

        let individual_openings = state
            .individual_openings
            .iter()
            .filter(|opening| {
                matches!(
                    opening.polynomial,
                    CommittedPolynomial::TrustedAdvice | CommittedPolynomial::UntrustedAdvice
                )
            })
            .collect::<Vec<_>>();
        if proof.individual_proofs.len() != individual_openings.len() {
            return Err(ProofVerifyError::InvalidInputLength(
                individual_openings.len(),
                proof.individual_proofs.len(),
            ));
        }
        for (index, (opening, proof)) in individual_openings
            .into_iter()
            .zip(proof.individual_proofs.iter())
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

    fn split_batch_hint(_batch_hint: &Self::BatchOpeningHint) -> Vec<Self::OpeningProofHint> {
        Vec::new()
    }

    fn packed_main_commitment_arity() -> Option<usize> {
        Some(1)
    }

    fn uses_onehot_inc() -> bool {
        true
    }

    fn supported_log_k_chunks(max_log_k: usize) -> Vec<usize> {
        if max_log_k > 4 {
            vec![4, max_log_k]
        } else {
            vec![max_log_k]
        }
    }

    fn validate_batch_proof_shape(
        proof: &Self::BatchedProof,
        one_hot_log_k_chunk: usize,
    ) -> Result<(), ProofVerifyError> {
        let proof_log_k = proof.log_k as usize;
        if proof_log_k != one_hot_log_k_chunk {
            return Err(ProofVerifyError::InvalidInputLength(
                one_hot_log_k_chunk,
                proof_log_k,
            ));
        }
        Ok(())
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
        match onehot_k {
            Some(onehot_k) => {
                let indices = tier1_commitments
                    .iter()
                    .flat_map(|chunk| match chunk {
                        AkitaChunkState::OneHot { indices, .. } => indices.clone(),
                        AkitaChunkState::Dense(_) => Vec::new(),
                    })
                    .collect::<Vec<_>>();
                let poly = OneHotPoly::<Fp128, D, u8>::new(onehot_k, indices.clone())
                    .expect("OneHotPoly construction failed");
                let (commitment, akita_hint) = commit_akita_poly::<D, Cfg, _>(&setup.0, &poly);
                let commitment = ArkBridge(commitment);
                let hint = JoltAkitaOpeningHint {
                    commitment: commitment.clone(),
                    akita_hint: ArkBridge(akita_hint),
                    ring_coeffs: Vec::new(),
                    onehot_k: Some(onehot_k),
                    onehot_indices: indices,
                };
                (commitment, hint)
            }
            None => {
                let coeffs = tier1_commitments
                    .iter()
                    .flat_map(|chunk| match chunk {
                        AkitaChunkState::Dense(coeffs) => coeffs.clone(),
                        AkitaChunkState::OneHot { .. } => Vec::new(),
                    })
                    .collect::<Vec<_>>();
                let ring_coeffs = pack_field_to_ring::<D>(&coeffs);
                let poly = DensePoly::from_ring_coeffs(ring_coeffs.clone());
                let (commitment, akita_hint) = commit_akita_poly::<D, Cfg, _>(&setup.0, &poly);
                let commitment = ArkBridge(commitment);
                let hint = JoltAkitaOpeningHint {
                    commitment: commitment.clone(),
                    akita_hint: ArkBridge(akita_hint),
                    ring_coeffs,
                    onehot_k: None,
                    onehot_indices: Vec::new(),
                };
                (commitment, hint)
            }
        }
    }

    fn streaming_batch_hint(_hints: Vec<Self::OpeningProofHint>) -> Self::BatchOpeningHint {
        panic!("Akita uses packed batch_commit via PolynomialBatchSource")
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
        MultilinearPolynomial::OneHot(_) => Vec::new(),
        MultilinearPolynomial::RLC(_) => Vec::new(),
    }
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
