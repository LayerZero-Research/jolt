use std::borrow::Borrow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::slice::from_ref;

use super::packed_layout::{choose_packed_bit_layout, PackedBitLayout};
use super::packed_poly::build_packed_poly;
use akita_algebra::CyclotomicRing;
use akita_config::proof_optimized::fp128::{D32Full, D32OneHot};
use akita_config::CommitmentConfig;
use akita_field::CanonicalField;
use akita_pcs::AkitaCommitmentScheme;
use akita_prover::{
    AkitaPolyOps, CommitmentProver as AkitaCommitmentProver, CommittedPolynomials,
    ComputeBackendSetup, CpuBackend, DensePoly, OneHotPoly,
};
use akita_types::{
    CommitmentVerifier as AkitaCommitmentVerifier, CommittedOpenings, RingCommitment,
    SetupContributionMode, ShapedCommittedOpenings,
};
use ark_ff::biginteger::S128;
use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use ark_std::Zero;
use rayon::prelude::*;

use super::wrappers::{
    akita_to_jolt, jolt_to_akita, AkitaProof, AkitaProverSetup, AkitaVerifierSetup, ArkBridge,
    Fp128, JoltToAkitaTranscript,
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
use crate::poly::unipoly::{CompressedUniPoly, UniPoly};
use crate::transcripts::Transcript;
use crate::utils::errors::ProofVerifyError;
use crate::utils::math::Math;
use crate::utils::small_scalar::SmallScalar;
use crate::zkvm::witness::{all_committed_polynomials, CommittedPolynomial};

pub type Fp128OneHot32Config = D32OneHot;

type Fp128DenseConfig = D32Full;
const AKITA_DENSE_MAX_BATCH_POLYS: usize = 64;

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
    dense_batch_proof: Option<ArkBridge<AkitaProof<Fp128>>>,
    dense_batch_item_count: u32,
    dense_batch_openings: Vec<JoltFp128>,
    dense_batch_sumcheck: Vec<CompressedUniPoly<JoltFp128>>,
}

#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct JoltAkitaProverSetup<const D: usize> {
    main: ArkBridge<AkitaProverSetup<Fp128, D>>,
    dense: ArkBridge<AkitaProverSetup<Fp128, D>>,
}

#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct JoltAkitaVerifierSetup<const D: usize> {
    main: ArkBridge<AkitaVerifierSetup<Fp128>>,
    dense: ArkBridge<AkitaVerifierSetup<Fp128>>,
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
    ProofTranscript: akita_transcript::Transcript<Fp128> + akita_prover::ProverTranscriptGrind<Fp128>,
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
    .map_err(|err| ProofVerifyError::AkitaError(format!("single opening verify failed: {err:?}")))
}

fn akita_prove_dense_batch<
    const D: usize,
    ProofTranscript: akita_transcript::Transcript<Fp128> + akita_prover::ProverTranscriptGrind<Fp128>,
>(
    setup: &AkitaProverSetup<Fp128, D>,
    items: Vec<DenseBatchProverItem<D>>,
    opening_point: &[Fp128],
    transcript: &mut ProofTranscript,
) -> AkitaProof<Fp128> {
    let prepared = prepare_cpu(setup);
    let claims = items
        .iter()
        .map(|item| CommittedPolynomials {
            polynomials: from_ref(&item.poly),
            commitment: &item.commitment,
            hint: item.hint.clone(),
        })
        .collect::<Vec<_>>();
    akita_prover::batched_prove_root_direct::<
        Fp128DenseConfig,
        ProofTranscript,
        DensePoly<Fp128, D>,
        CpuBackend,
        D,
    >(
        &setup.expanded,
        &CpuBackend,
        &prepared,
        (opening_point, claims),
        transcript,
        akita_types::BasisMode::Lagrange,
    )
    .expect("Akita dense batched prove failed")
}

fn akita_verify_dense_batch<
    const D: usize,
    ProofTranscript: akita_transcript::Transcript<Fp128>,
>(
    proof: &AkitaProof<Fp128>,
    setup: &AkitaVerifierSetup<Fp128>,
    transcript: &mut ProofTranscript,
    opening_point: &[Fp128],
    openings: &[Fp128],
    natural_num_vars: &[usize],
    commitments: &[RingCommitment<Fp128, D>],
) -> Result<(), ProofVerifyError> {
    let opening_slices = openings
        .iter()
        .copied()
        .map(|opening| [opening])
        .collect::<Vec<_>>();
    let natural_num_vars_slices = natural_num_vars
        .iter()
        .copied()
        .map(|num_vars| [num_vars])
        .collect::<Vec<_>>();
    let claims = commitments
        .iter()
        .enumerate()
        .map(|(index, commitment)| ShapedCommittedOpenings {
            openings: &opening_slices[index],
            natural_num_vars: &natural_num_vars_slices[index],
            commitment,
        })
        .collect::<Vec<_>>();
    akita_verifier::batched_verify_shaped_root_direct::<Fp128DenseConfig, ProofTranscript, D>(
        proof,
        setup,
        transcript,
        (opening_point, claims),
        akita_types::BasisMode::Lagrange,
        SetupContributionMode::Direct,
    )
    .map_err(|err| ProofVerifyError::AkitaError(format!("dense batch verify failed: {err:?}")))
}

fn to_akita_opening_point(point: &[JoltFp128]) -> Vec<Fp128> {
    point.iter().rev().map(jolt_to_akita).collect()
}

fn to_akita_dense_opening_point<const D: usize>(point: &[JoltFp128]) -> Vec<Fp128> {
    let mut akita_point = to_akita_opening_point(point);
    akita_point.resize(akita_point.len().max(D.log_2()), Fp128::zero());
    akita_point
}

fn dense_poly_from_ring_coeffs<const D: usize>(
    ring_coeffs: Vec<CyclotomicRing<Fp128, D>>,
) -> DensePoly<Fp128, D> {
    let total = ring_coeffs
        .len()
        .checked_mul(D)
        .expect("ring elems * D overflow");
    let num_vars = total.trailing_zeros() as usize;
    let mut field_evals = Vec::with_capacity(total);
    for ring in ring_coeffs {
        field_evals.extend_from_slice(ring.coefficients());
    }
    DensePoly::from_field_evals(num_vars, &field_evals)
        .expect("Akita DensePoly construction failed")
}

fn field_evals_from_ring_coeffs<const D: usize>(
    ring_coeffs: &[CyclotomicRing<Fp128, D>],
) -> Vec<JoltFp128> {
    ring_coeffs
        .iter()
        .flat_map(|ring| ring.coefficients().iter().map(akita_to_jolt))
        .collect()
}

fn to_jolt_dense_opening_point(opening_point: &[JoltFp128], num_vars: usize) -> Vec<JoltFp128> {
    assert!(
        opening_point.len() <= num_vars,
        "dense opening point has more variables than its polynomial"
    );
    let mut point = vec![JoltFp128::zero(); num_vars - opening_point.len()];
    point.extend_from_slice(opening_point);
    point
}

fn evaluate_dense_evals_at_point(evals: &[JoltFp128], point: &[JoltFp128]) -> JoltFp128 {
    debug_assert_eq!(evals.len(), 1usize << point.len());
    let eq = EqPolynomial::<JoltFp128>::evals(point);
    evals
        .par_iter()
        .zip(eq.par_iter())
        .map(|(&eval, &weight)| eval * weight)
        .sum()
}

fn ring_padded_dense_num_vars<const D: usize>(len: usize) -> usize {
    let ring_count = len.div_ceil(D).next_power_of_two().max(1);
    (ring_count * D).log_2()
}

fn dense_poly_from_jolt_evals_padded<const D: usize>(
    evals: &[JoltFp128],
    num_vars: usize,
) -> DensePoly<Fp128, D> {
    let padded_len = 1usize << num_vars;
    debug_assert!(evals.len() <= padded_len);
    let mut padded_evals = vec![Fp128::zero(); padded_len];
    for (dst, src) in padded_evals.iter_mut().zip(evals.iter()) {
        *dst = jolt_to_akita(src);
    }
    DensePoly::from_field_evals(num_vars, &padded_evals)
        .expect("Akita padded DensePoly construction failed")
}

struct DenseBatchProverItem<const D: usize> {
    poly: DensePoly<Fp128, D>,
    field_evals: Vec<JoltFp128>,
    opening_point: Vec<JoltFp128>,
    claim: JoltFp128,
    commitment: RingCommitment<Fp128, D>,
    hint: akita_types::AkitaCommitmentHint<Fp128, D>,
}

struct DenseBatchVerifierItem<const D: usize> {
    field_evals_len: usize,
    opening_point: Vec<JoltFp128>,
    claim: JoltFp128,
    commitment: RingCommitment<Fp128, D>,
}

fn bind_dense_round(values: &mut Vec<JoltFp128>, r: JoltFp128) {
    let half = values.len() / 2;
    for index in 0..half {
        let low = values[index];
        let high = values[index + half];
        values[index] = low + r * (high - low);
    }
    values.truncate(half);
}

fn dense_multipoint_initial_claim<const D: usize>(
    items: &[DenseBatchProverItem<D>],
    coeffs: &[JoltFp128],
) -> JoltFp128 {
    items
        .iter()
        .zip(coeffs.iter())
        .map(|(item, &coeff)| coeff * item.claim)
        .sum()
}

fn prove_dense_multipoint_sumcheck<const D: usize, ProofTranscript: Transcript>(
    items: &[DenseBatchProverItem<D>],
    coeffs: &[JoltFp128],
    max_vars: usize,
    transcript: &mut ProofTranscript,
) -> (Vec<CompressedUniPoly<JoltFp128>>, Vec<JoltFp128>, JoltFp128) {
    let mut polys = Vec::with_capacity(items.len());
    let mut eqs = Vec::with_capacity(items.len());
    let padded_len = 1usize << max_vars;
    for item in items {
        let item_vars = item.field_evals.len().log_2();
        let head_vars = max_vars - item_vars;
        let mut poly = vec![JoltFp128::zero(); padded_len];
        poly[..item.field_evals.len()].copy_from_slice(&item.field_evals);
        let mut padded_point = vec![JoltFp128::zero(); head_vars];
        padded_point.extend_from_slice(&item.opening_point);
        polys.push(poly);
        eqs.push(EqPolynomial::<JoltFp128>::evals(&padded_point));
    }

    let mut claim = dense_multipoint_initial_claim(items, coeffs);
    let mut proof = Vec::with_capacity(max_vars);
    let mut challenges = Vec::with_capacity(max_vars);
    for _round in 0..max_vars {
        let half = polys[0].len() / 2;
        let evals = (0..half)
            .into_par_iter()
            .fold(
                || [JoltFp128::zero(); 3],
                |mut acc, index| {
                    for ((poly, eq), &coeff) in polys.iter().zip(eqs.iter()).zip(coeffs.iter()) {
                        let p0 = poly[index];
                        let p1 = poly[index + half];
                        let e0 = eq[index];
                        let e1 = eq[index + half];
                        let p_step = p1 - p0;
                        let e_step = e1 - e0;
                        acc[0] += coeff * p0 * e0;
                        acc[1] += coeff * p1 * e1;
                        acc[2] += coeff
                            * (p0 + JoltFp128::from_u64(2) * p_step)
                            * (e0 + JoltFp128::from_u64(2) * e_step);
                    }
                    acc
                },
            )
            .reduce(
                || [JoltFp128::zero(); 3],
                |mut lhs, rhs| {
                    lhs[0] += rhs[0];
                    lhs[1] += rhs[1];
                    lhs[2] += rhs[2];
                    lhs
                },
            );
        debug_assert_eq!(evals[0] + evals[1], claim);
        let compressed = UniPoly::from_evals(&evals).compress();
        transcript.append_scalars(b"sumcheck_poly", &compressed.coeffs_except_linear_term);
        let r = transcript.challenge_scalar_optimized::<JoltFp128>();
        claim = compressed.eval_from_hint(&claim, &r);
        proof.push(compressed);
        challenges.push(r);
        for values in polys.iter_mut().chain(eqs.iter_mut()) {
            bind_dense_round(values, r);
        }
    }

    (proof, challenges, claim)
}

fn dense_sumcheck_final_claim(
    output_claims: &[JoltFp128],
    original_points: &[Vec<JoltFp128>],
    coeffs: &[JoltFp128],
    common_point: &[JoltFp128],
) -> JoltFp128 {
    output_claims
        .iter()
        .zip(original_points.iter())
        .zip(coeffs.iter())
        .map(|((&opening, original_point), &coeff)| {
            let head_vars = common_point.len() - original_point.len();
            let head_selector =
                EqPolynomial::<JoltFp128>::zero_selector(&common_point[..head_vars]);
            let tail_eq =
                EqPolynomial::<JoltFp128>::mle(original_point, &common_point[head_vars..]);
            coeff * head_selector * tail_eq * opening
        })
        .sum()
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
        setup: &JoltAkitaProverSetup<D>,
        opening_point: &[JoltFp128],
        transcript: &mut ProofTranscript,
    ) -> AkitaProof<Fp128>
    where
        ProofTranscript:
            akita_transcript::Transcript<Fp128> + akita_prover::ProverTranscriptGrind<Fp128>,
        Cfg: CommitmentConfig<Field = Fp128, ExtField = Fp128>,
    {
        if let Some(onehot_k) = self.onehot_k {
            debug_assert!(onehot_k.is_power_of_two());
            let akita_point =
                to_akita_onehot_opening_point(opening_point, onehot_k.trailing_zeros() as usize);
            let poly = OneHotPoly::<Fp128, D, u8>::new(onehot_k, self.onehot_indices)
                .expect("OneHotPoly construction failed");
            akita_prove_one::<D, Cfg, _, _>(
                &setup.main.0,
                &poly,
                &akita_point,
                self.akita_hint.0,
                transcript,
                &self.commitment.0,
            )
        } else {
            let akita_point = to_akita_dense_opening_point::<D>(opening_point);
            let poly = dense_poly_from_ring_coeffs(self.ring_coeffs);
            akita_prove_one::<D, Fp128DenseConfig, _, _>(
                &setup.dense.0,
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
    type ProverSetup = JoltAkitaProverSetup<D>;
    type VerifierSetup = JoltAkitaVerifierSetup<D>;
    type Commitment = ArkBridge<RingCommitment<Fp128, D>>;
    type Proof = JoltAkitaProof;
    type BatchedProof = JoltAkitaProof;
    type OpeningProofHint = JoltAkitaOpeningHint<D>;
    type BatchOpeningHint = JoltAkitaBatchHint<D>;

    fn setup_prover(max_num_vars: usize) -> Self::ProverSetup {
        let main =
            <AkitaCommitmentScheme<D, Cfg> as AkitaCommitmentProver<Fp128, D>>::setup_prover(
                max_num_vars,
                1,
            )
            .expect("Akita setup_prover failed");
        let dense = <AkitaCommitmentScheme<D, Fp128DenseConfig> as AkitaCommitmentProver<
            Fp128,
            D,
        >>::setup_prover(max_num_vars, AKITA_DENSE_MAX_BATCH_POLYS)
        .expect("Akita dense setup_prover failed");
        JoltAkitaProverSetup {
            main: ArkBridge(main),
            dense: ArkBridge(dense),
        }
    }

    fn setup_prover_from_shape(
        max_log_t: usize,
        max_log_k: usize,
        log_packed: Option<usize>,
    ) -> Self::ProverSetup {
        Self::setup_prover_from_shape_with_extra(max_log_t, max_log_k, log_packed, &[])
    }

    fn setup_prover_from_shape_with_extra(
        max_log_t: usize,
        max_log_k: usize,
        log_packed: Option<usize>,
        extra_num_vars: &[usize],
    ) -> Self::ProverSetup {
        let max_num_vars = if let Some(log_packed) = log_packed {
            choose_packed_layout_for_shape::<D, Cfg>(max_log_k, max_log_t, log_packed)
                .total_num_vars()
        } else {
            max_log_t + max_log_k
        };
        let max_num_vars = extra_num_vars
            .iter()
            .copied()
            .fold(max_num_vars, usize::max);
        Self::setup_prover(max_num_vars)
    }

    fn setup_verifier(setup: &Self::ProverSetup) -> Self::VerifierSetup {
        JoltAkitaVerifierSetup {
            main: ArkBridge(<AkitaCommitmentScheme<D, Cfg> as AkitaCommitmentProver<
                Fp128,
                D,
            >>::setup_verifier(&setup.main.0)),
            dense:
                ArkBridge(
                    <AkitaCommitmentScheme<D, Fp128DenseConfig> as AkitaCommitmentProver<
                        Fp128,
                        D,
                    >>::setup_verifier(&setup.dense.0),
                ),
        }
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
                let (commitment, hint) = commit_akita_poly::<D, Cfg, _>(&setup.main.0, &poly);
                (commitment, hint, Vec::new(), Some(onehot.K), indices)
            } else {
                let ring_coeffs = poly_to_ring_coeffs::<D>(poly);
                let poly = dense_poly_from_ring_coeffs(ring_coeffs.clone());
                let (commitment, hint) =
                    commit_akita_poly::<D, Fp128DenseConfig, _>(&setup.dense.0, &poly);
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

    fn commit_dense_batch(
        polys: &[MultilinearPolynomial<JoltFp128>],
        setup: &Self::ProverSetup,
    ) -> Option<Vec<(Self::Commitment, Self::OpeningProofHint)>> {
        if polys.is_empty()
            || polys
                .iter()
                .any(|poly| matches!(poly, MultilinearPolynomial::OneHot(_)))
        {
            return None;
        }

        let mut ring_coeffs = polys
            .iter()
            .map(poly_to_ring_coeffs::<D>)
            .collect::<Vec<_>>();
        let max_ring_count = ring_coeffs
            .iter()
            .map(Vec::len)
            .max()
            .expect("non-empty dense batch");
        for coeffs in ring_coeffs.iter_mut() {
            coeffs.resize_with(max_ring_count, CyclotomicRing::<Fp128, D>::zero);
        }
        let dense_polys = ring_coeffs
            .iter()
            .cloned()
            .map(dense_poly_from_ring_coeffs::<D>)
            .collect::<Vec<_>>();
        let groups = dense_polys
            .iter()
            .map(from_ref)
            .collect::<Vec<&[DensePoly<Fp128, D>]>>();
        let prepared = prepare_cpu(&setup.dense.0);
        let committed = <AkitaCommitmentScheme<D, Fp128DenseConfig> as AkitaCommitmentProver<
            Fp128,
            D,
        >>::batched_commit(&setup.dense.0, &CpuBackend, &prepared, &groups)
        .expect("Akita dense batched commit failed");

        Some(
            committed
                .into_iter()
                .zip(ring_coeffs)
                .map(|((commitment, akita_hint), ring_coeffs)| {
                    let commitment = ArkBridge(commitment);
                    let hint = JoltAkitaOpeningHint {
                        commitment: commitment.clone(),
                        akita_hint: ArkBridge(akita_hint),
                        ring_coeffs,
                        onehot_k: None,
                        onehot_indices: Vec::new(),
                    };
                    (commitment, hint)
                })
                .collect(),
        )
    }

    fn dense_num_vars(len: usize) -> usize {
        ring_padded_dense_num_vars::<D>(len)
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
        let (commitment, akita_hint) = commit_akita_poly::<D, Cfg, _>(&setup.main.0, &packed_poly);
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
        let proof = hint.prove::<_, Cfg>(setup, opening_point, &mut adapter);
        (
            JoltAkitaProof {
                packed_poly_proof: ArkBridge(proof),
                num_packed_polys: 1,
                log_k: opening_point.len() as u32,
                dense_batch_proof: None,
                dense_batch_item_count: 0,
                dense_batch_openings: Vec::new(),
                dense_batch_sumcheck: Vec::new(),
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

        let individual_openings = state.individual_openings.clone();

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
            &context.setup.main.0,
            &packed_poly,
            &packed_point,
            context.batch_hint.akita_hint.0,
            &mut adapter,
            &context.batch_hint.commitment.0,
        );

        let mut dense_items = Vec::with_capacity(individual_openings.len());
        for opening in individual_openings.iter() {
            let hint = context
                .opening_hints
                .remove(&opening.polynomial)
                .expect("missing Akita dense opening hint");
            assert!(
                hint.onehot_k.is_none(),
                "Akita individual Stage 8 openings must be dense"
            );
            let field_evals = field_evals_from_ring_coeffs(&hint.ring_coeffs);
            let num_vars = field_evals.len().log_2();
            let opening_point = to_jolt_dense_opening_point(&opening.opening_point.r, num_vars);
            let poly = dense_poly_from_ring_coeffs(hint.ring_coeffs);
            dense_items.push(DenseBatchProverItem {
                poly,
                field_evals,
                opening_point,
                claim: opening.claim,
                commitment: hint.commitment.0,
                hint: hint.akita_hint.0,
            });
        }

        let (dense_batch_proof, dense_batch_openings, dense_batch_sumcheck) =
            if dense_items.is_empty() {
                (None, Vec::new(), Vec::new())
            } else {
                let dense_count = dense_items.len();
                transcript.append_bytes(
                    b"akita_dense_batch_num",
                    &(dense_count as u64).to_le_bytes(),
                );
                let coeffs = transcript.challenge_scalar_powers(dense_count);
                let max_vars = dense_items
                    .iter()
                    .map(|item| item.field_evals.len().log_2())
                    .max()
                    .expect("nonempty dense batch");
                let (sumcheck_proof, common_point, _sumcheck_claim) =
                    prove_dense_multipoint_sumcheck(&dense_items, &coeffs, max_vars, transcript);
                let dense_openings = dense_items
                    .iter()
                    .map(|item| {
                        let item_vars = item.field_evals.len().log_2();
                        let head_vars = max_vars - item_vars;
                        let head_selector =
                            EqPolynomial::<JoltFp128>::zero_selector(&common_point[..head_vars]);
                        let item_point = &common_point[max_vars - item_vars..];
                        head_selector * evaluate_dense_evals_at_point(&item.field_evals, item_point)
                    })
                    .collect::<Vec<_>>();
                transcript.append_scalars(b"akita_dense_openings", &dense_openings);
                let common_akita_point = to_akita_dense_opening_point::<D>(&common_point);
                for item in dense_items.iter_mut() {
                    item.poly = dense_poly_from_jolt_evals_padded(&item.field_evals, max_vars);
                }
                let mut adapter = JoltToAkitaTranscript::new(transcript);
                let proof = akita_prove_dense_batch::<D, _>(
                    &context.setup.dense.0,
                    dense_items,
                    &common_akita_point,
                    &mut adapter,
                );
                (Some(ArkBridge(proof)), dense_openings, sumcheck_proof)
            };

        let proof = JoltAkitaProof {
            packed_poly_proof: ArkBridge(packed_proof),
            num_packed_polys: num_packed as u32,
            log_k: context.batch_hint.log_k as u32,
            dense_batch_proof,
            dense_batch_item_count: individual_openings.len() as u32,
            dense_batch_openings,
            dense_batch_sumcheck,
        };

        (proof, None)
    }

    fn verify<ProofTranscript: Transcript>(
        proof: &Self::Proof,
        setup: &Self::VerifierSetup,
        transcript: &mut ProofTranscript,
        opening_point: &[JoltFp128],
        opening: &JoltFp128,
        commitment: &Self::Commitment,
    ) -> Result<(), ProofVerifyError> {
        let akita_point = to_akita_dense_opening_point::<D>(opening_point);
        let akita_opening = jolt_to_akita(opening);
        let mut adapter = JoltToAkitaTranscript::new(transcript);
        akita_verify_one::<D, Fp128DenseConfig, _>(
            &proof.packed_poly_proof.0,
            &setup.dense.0,
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
        let num_packed = state
            .polynomial_claims
            .len()
            .checked_sub(state.individual_openings.len())
            .ok_or(ProofVerifyError::InvalidOpeningProof)?;
        if num_packed == 0 {
            return Err(ProofVerifyError::InvalidInputLength(1, 0));
        }
        let proof_num_packed = proof.num_packed_polys as usize;
        if proof_num_packed != num_packed {
            return Err(ProofVerifyError::InvalidInputLength(
                num_packed,
                proof_num_packed,
            ));
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
        let log_k = state
            .packed_log_k
            .ok_or(ProofVerifyError::InvalidOpeningProof)?;
        let proof_log_k = proof.log_k as usize;
        if proof_log_k != log_k {
            return Err(ProofVerifyError::InvalidInputLength(log_k, proof_log_k));
        }
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
        let individual_openings = state.individual_openings.iter().collect::<Vec<_>>();
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
            &setup.main.0,
            &mut adapter,
            &packed_point,
            &akita_combined,
            &packed_commitment.0,
        )?;

        let proof_dense_count = proof.dense_batch_item_count as usize;
        if proof_dense_count != individual_openings.len() {
            return Err(ProofVerifyError::InvalidInputLength(
                individual_openings.len(),
                proof_dense_count,
            ));
        }
        let mut dense_items = Vec::with_capacity(individual_openings.len());
        for opening in individual_openings {
            let commitment = commitment_map
                .remove(&opening.polynomial)
                .ok_or(ProofVerifyError::InvalidOpeningProof)?;
            let natural_num_vars = opening.num_vars.unwrap_or(opening.opening_point.r.len());
            let jolt_point =
                to_jolt_dense_opening_point(&opening.opening_point.r, natural_num_vars);
            dense_items.push(DenseBatchVerifierItem {
                field_evals_len: 1usize << natural_num_vars,
                opening_point: jolt_point,
                claim: opening.claim,
                commitment: commitment.0,
            });
        }
        if dense_items.is_empty() {
            if proof.dense_batch_proof.is_some()
                || !proof.dense_batch_sumcheck.is_empty()
                || !proof.dense_batch_openings.is_empty()
            {
                return Err(ProofVerifyError::InvalidOpeningProof);
            }
        } else {
            let Some(dense_batch_proof) = &proof.dense_batch_proof else {
                return Err(ProofVerifyError::InvalidOpeningProof);
            };
            let dense_count = dense_items.len();
            if proof.dense_batch_openings.len() != dense_count {
                return Err(ProofVerifyError::InvalidInputLength(
                    dense_count,
                    proof.dense_batch_openings.len(),
                ));
            }
            transcript.append_bytes(
                b"akita_dense_batch_num",
                &(dense_count as u64).to_le_bytes(),
            );
            let coeffs = transcript.challenge_scalar_powers(dense_count);
            let initial_claim = dense_items
                .iter()
                .zip(coeffs.iter())
                .map(|(item, &coeff)| coeff * item.claim)
                .sum::<JoltFp128>();
            let max_vars = dense_items
                .iter()
                .map(|item| item.field_evals_len.log_2())
                .max()
                .ok_or(ProofVerifyError::InvalidOpeningProof)?;
            if proof.dense_batch_sumcheck.len() != max_vars {
                return Err(ProofVerifyError::InvalidInputLength(
                    max_vars,
                    proof.dense_batch_sumcheck.len(),
                ));
            }
            let mut sumcheck_claim = initial_claim;
            let mut common_point = Vec::with_capacity(max_vars);
            for compressed in &proof.dense_batch_sumcheck {
                let degree = compressed.degree();
                if degree == 0 || degree > 2 {
                    return Err(ProofVerifyError::InvalidInputLength(2, degree));
                }
                transcript.append_scalars(b"sumcheck_poly", &compressed.coeffs_except_linear_term);
                let r = transcript.challenge_scalar_optimized::<JoltFp128>();
                sumcheck_claim = compressed.eval_from_hint(&sumcheck_claim, &r);
                common_point.push(r);
            }
            transcript.append_scalars(b"akita_dense_openings", &proof.dense_batch_openings);
            let original_points = dense_items
                .iter()
                .map(|item| item.opening_point.clone())
                .collect::<Vec<_>>();
            let expected_sumcheck_claim = dense_sumcheck_final_claim(
                &proof.dense_batch_openings,
                &original_points,
                &coeffs,
                &common_point,
            );
            if sumcheck_claim != expected_sumcheck_claim {
                return Err(ProofVerifyError::AkitaError(
                    format!(
                        "dense sumcheck final claim mismatch: max_vars={}, item_vars={:?}, point_vars={:?}",
                        max_vars,
                        dense_items
                            .iter()
                            .map(|item| item.field_evals_len.log_2())
                            .collect::<Vec<_>>(),
                        original_points
                            .iter()
                            .map(Vec::len)
                            .collect::<Vec<_>>()
                    ),
                ));
            }
            let akita_opening_point = to_akita_dense_opening_point::<D>(&common_point);
            let akita_openings = proof
                .dense_batch_openings
                .iter()
                .map(jolt_to_akita)
                .collect::<Vec<_>>();
            let natural_num_vars = dense_items.iter().map(|_| max_vars).collect::<Vec<_>>();
            let commitments = dense_items
                .into_iter()
                .map(|item| item.commitment)
                .collect::<Vec<_>>();
            let mut adapter = JoltToAkitaTranscript::new(transcript);
            akita_verify_dense_batch::<D, _>(
                &dense_batch_proof.0,
                &setup.dense.0,
                &mut adapter,
                &akita_opening_point,
                &akita_openings,
                &natural_num_vars,
                &commitments,
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
                let (commitment, akita_hint) = commit_akita_poly::<D, Cfg, _>(&setup.main.0, &poly);
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
                let poly = dense_poly_from_ring_coeffs(ring_coeffs.clone());
                let (commitment, akita_hint) =
                    commit_akita_poly::<D, Fp128DenseConfig, _>(&setup.dense.0, &poly);
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
