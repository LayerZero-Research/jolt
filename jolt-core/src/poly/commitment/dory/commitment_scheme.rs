//! Dory polynomial commitment scheme implementation

use super::dory_globals::{DoryGlobals, DoryLayout};
use super::jolt_dory_routines::{JoltG1Routines, JoltG2Routines};
use super::layout::DoryCommitmentLayout;
use super::wrappers::{
    ark_to_jolt, jolt_to_ark, ArkDoryProof, ArkFr, ArkG1, ArkGT, ArkworksProverSetup,
    ArkworksVerifierSetup, DoryLayoutBoundPolynomial, JoltToDoryTranscript, BN254,
};
use crate::{
    curve::JoltCurve,
    field::JoltField,
    poly::coefficient_layout::CoefficientLayout,
    poly::commitment::commitment_scheme::{
        CommitmentContext, CommitmentScheme, PolynomialBatchSource, StreamingCommitmentScheme,
        ZkEvalCommitment,
    },
    poly::commitment::opening_point::FinalOpeningPointParts,
    poly::multilinear_polynomial::MultilinearPolynomial,
    poly::opening_proof::{BatchPolynomialSource, OpeningPoint, BIG_ENDIAN},
    transcripts::Transcript,
    utils::{errors::ProofVerifyError, math::Math, small_scalar::SmallScalar},
};
use ark_bn254::{G1Affine, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::Zero;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use dory::primitives::{
    arithmetic::{Field as DoryField, Group, PairingCurve},
    poly::Polynomial,
};
use rayon::prelude::*;
use tracing::trace_span;

#[derive(Clone, Default)]
pub struct DoryCommitmentScheme {
    pub layout: DoryLayout,
}

#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct DoryBatchedProof {
    pub proof: ArkDoryProof,
    pub layout: DoryLayout,
    #[cfg(feature = "zk")]
    pub y_blinding: Option<ark_bn254::Fr>,
}

/// Split `total_vars` into balanced `(sigma, nu)` where sigma = ceil(total_vars / 2)
/// and nu = total_vars - sigma. sigma is the number of column variables,
/// nu is the number of row variables.
#[inline]
pub fn balanced_sigma_nu(total_vars: usize) -> (usize, usize) {
    let sigma = total_vars.div_ceil(2);
    let nu = total_vars - sigma;
    (sigma, nu)
}

#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct DoryOpeningProofHint {
    row_commitments: Vec<ArkG1>,
    commit_blind: ArkFr,
}

impl DoryOpeningProofHint {
    pub fn new(row_commitments: Vec<ArkG1>, commit_blind: ArkFr) -> Self {
        Self {
            row_commitments,
            commit_blind,
        }
    }

    pub fn row_commitments(&self) -> &[ArkG1] {
        &self.row_commitments
    }

    pub fn commit_blind(&self) -> &ArkFr {
        &self.commit_blind
    }

    fn into_parts(self) -> (Vec<ArkG1>, ArkFr) {
        (self.row_commitments, self.commit_blind)
    }
}

fn maybe_blind_commitment(setup: &ArkworksProverSetup, commitment: ArkGT) -> (ArkGT, ArkFr) {
    #[cfg(feature = "zk")]
    {
        let commit_blind = <dory::ZK as dory::Mode>::sample::<ArkFr>();
        let commitment = <dory::ZK as dory::Mode>::mask(commitment, &setup.ht, &commit_blind);
        (commitment, commit_blind)
    }
    #[cfg(not(feature = "zk"))]
    {
        let _ = setup;
        (commitment, <ArkFr as DoryField>::zero())
    }
}

fn compute_final_opening_point<F: JoltField>(
    layout: DoryLayout,
    parts: FinalOpeningPointParts<F>,
) -> Result<OpeningPoint<BIG_ENDIAN, F>, ProofVerifyError> {
    match (layout, parts) {
        (
            DoryLayout::AddressMajor,
            FinalOpeningPointParts::Native {
                r_address_stage7,
                r_cycle_stage6,
                ..
            },
        ) => Ok(OpeningPoint::<BIG_ENDIAN, F>::new(
            [r_cycle_stage6.r.as_slice(), r_address_stage7.as_slice()].concat(),
        )),
        (_, parts) => parts.into_canonical(),
    }
}

#[inline]
fn canonical_setup_log_n(max_num_vars: usize) -> usize {
    // Dory's generator count depends on ceil(max_log_n / 2), so odd/even pairs like
    // 23 and 24 share the same generator bucket. Canonicalizing to the even bucket
    // representative keeps those runs on a single URS file.
    if max_num_vars.is_multiple_of(2) {
        max_num_vars
    } else {
        max_num_vars + 1
    }
}

pub fn bind_opening_inputs<F: JoltField, ProofTranscript: Transcript>(
    transcript: &mut ProofTranscript,
    opening_point: &[F::Challenge],
    opening: &F,
) {
    let mut point_scalars = Vec::with_capacity(opening_point.len());
    for point in opening_point {
        let scalar: F = (*point).into();
        point_scalars.push(scalar);
    }
    transcript.append_scalars(b"dory_opening_point", &point_scalars);

    transcript.append_scalar(b"dory_opening_eval", opening);
}

#[cfg(feature = "zk")]
pub fn bind_opening_inputs_zk<F: JoltField, C: JoltCurve<F = F>, ProofTranscript: Transcript>(
    transcript: &mut ProofTranscript,
    opening_point: &[F::Challenge],
    y_com: &C::G1,
) {
    let mut point_scalars = Vec::with_capacity(opening_point.len());
    for point in opening_point {
        let scalar: F = (*point).into();
        point_scalars.push(scalar);
    }
    transcript.append_scalars(b"dory_opening_point", &point_scalars);

    transcript.append_commitment(b"dory_eval_commitment", y_com);
}

impl CommitmentScheme for DoryCommitmentScheme {
    type Field = ark_bn254::Fr;
    type Config = DoryLayout;
    type ProverSetup = ArkworksProverSetup;
    type VerifierSetup = ArkworksVerifierSetup;
    type Commitment = ArkGT;
    type Proof = ArkDoryProof;
    type BatchedProof = DoryBatchedProof;
    type CommitmentLayout = DoryCommitmentLayout;
    type OpeningProofHint = DoryOpeningProofHint;
    type BatchOpeningHint = Vec<Self::OpeningProofHint>;

    fn setup_prover(max_num_vars: usize) -> Self::ProverSetup {
        let _span = trace_span!("DoryCommitmentScheme::setup_prover").entered();
        let canonical_max_num_vars = canonical_setup_log_n(max_num_vars);
        #[cfg(test)]
        DoryGlobals::configure_test_cache_root();
        #[cfg(not(target_arch = "wasm32"))]
        let setup = ArkworksProverSetup::new_from_urs(canonical_max_num_vars);
        #[cfg(target_arch = "wasm32")]
        let setup = ArkworksProverSetup::new(canonical_max_num_vars);

        // The prepared-point cache in dory-pcs is global and can only be initialized once.
        // In unit tests, multiple setups with different sizes are created, so initializing the
        // cache with a small setup can break later tests that need more generators.
        // We therefore disable cache initialization in `cfg(test)` builds.
        #[cfg(not(test))]
        DoryGlobals::init_prepared_cache(&setup.g1_vec, &setup.g2_vec);

        setup
    }

    fn setup_verifier(setup: &Self::ProverSetup) -> Self::VerifierSetup {
        let _span = trace_span!("DoryCommitmentScheme::setup_verifier").entered();
        setup.to_verifier_setup()
    }

    fn from_proof(proof: &DoryBatchedProof) -> Self {
        Self {
            layout: proof.layout,
        }
    }

    fn config(&self) -> &DoryLayout {
        &self.layout
    }

    fn main_trace_commitment_len(
        _config: &Self::Config,
        context: CommitmentContext,
        padded_trace_len: usize,
    ) -> usize {
        match context {
            CommitmentContext::MainTrace {
                k,
                commitment_total_vars,
                ..
            } => 1usize << commitment_total_vars.saturating_sub(k.log_2()),
            _ => padded_trace_len,
        }
    }

    fn coefficient_layout(config: &Self::Config, context: CommitmentContext) -> CoefficientLayout {
        DoryCommitmentLayout::from_context(config, context).coefficient_layout()
    }

    fn commitment_layout(
        config: &Self::Config,
        context: CommitmentContext,
    ) -> Self::CommitmentLayout {
        DoryCommitmentLayout::from_context(config, context)
    }

    fn final_opening_point(
        config: &Self::Config,
        parts: FinalOpeningPointParts<Self::Field>,
    ) -> Result<OpeningPoint<BIG_ENDIAN, Self::Field>, ProofVerifyError> {
        compute_final_opening_point(*config, parts)
    }

    fn commit(
        &self,
        layout: &Self::CommitmentLayout,
        poly: &MultilinearPolynomial<ark_bn254::Fr>,
        setup: &Self::ProverSetup,
    ) -> (Self::Commitment, Self::OpeningProofHint) {
        let _span = trace_span!("DoryCommitmentScheme::commit").entered();

        let (sigma, nu) = layout.sigma_nu();
        let bound_poly = DoryLayoutBoundPolynomial::new(poly, *layout);

        #[cfg(feature = "zk")]
        type DoryMode = dory::ZK;
        #[cfg(not(feature = "zk"))]
        type DoryMode = dory::Transparent;

        let (tier_2, row_commitments, commit_blind) =
            <DoryLayoutBoundPolynomial<'_> as Polynomial<ArkFr>>::commit::<
                BN254,
                DoryMode,
                JoltG1Routines,
            >(&bound_poly, nu, sigma, setup)
            .expect("commitment should succeed");

        (
            tier_2,
            DoryOpeningProofHint::new(row_commitments, commit_blind),
        )
    }

    fn batch_commit<S: PolynomialBatchSource<ark_bn254::Fr>>(
        &self,
        layout: &Self::CommitmentLayout,
        source: &S,
        gens: &Self::ProverSetup,
    ) -> (Vec<Self::Commitment>, Self::BatchOpeningHint) {
        let _span = trace_span!("DoryCommitmentScheme::batch_commit").entered();

        let results: Vec<(Self::Commitment, Self::OpeningProofHint)> = (0..source.num_polys())
            .into_par_iter()
            .map(|i| self.commit(layout, source.get_poly(i).unwrap(), gens))
            .collect();
        let (commitments, hints): (Vec<_>, Vec<_>) = results.into_iter().unzip();
        (commitments, hints)
    }

    fn prove<ProofTranscript: Transcript>(
        &self,
        layout: &Self::CommitmentLayout,
        setup: &Self::ProverSetup,
        poly: &MultilinearPolynomial<ark_bn254::Fr>,
        opening_point: &[<ark_bn254::Fr as JoltField>::Challenge],
        hint: Option<Self::OpeningProofHint>,
        transcript: &mut ProofTranscript,
        _commitment: &Self::Commitment,
    ) -> (Self::Proof, Option<Self::Field>) {
        let _span = trace_span!("DoryCommitmentScheme::prove").entered();

        let (row_commitments, commit_blind) = hint
            .map(DoryOpeningProofHint::into_parts)
            .unwrap_or_else(|| {
                let (_commitment, hint) = self.commit(layout, poly, setup);
                hint.into_parts()
            });

        let (sigma, nu) = layout.sigma_nu();
        let ark_point: Vec<ArkFr> = opening_point
            .iter()
            .rev()
            .map(|p| {
                let f_val: ark_bn254::Fr = (*p).into();
                jolt_to_ark(&f_val)
            })
            .collect();

        let mut dory_transcript = JoltToDoryTranscript::<ProofTranscript>::new(transcript);

        #[cfg(feature = "zk")]
        type DoryMode = dory::ZK;
        #[cfg(not(feature = "zk"))]
        type DoryMode = dory::Transparent;

        let bound_poly = DoryLayoutBoundPolynomial::new(poly, *layout);
        let (proof, y_blinding) =
            dory::prove::<ArkFr, BN254, JoltG1Routines, JoltG2Routines, _, _, DoryMode>(
                &bound_poly,
                &ark_point,
                row_commitments,
                commit_blind,
                nu,
                sigma,
                setup,
                &mut dory_transcript,
            )
            .expect("proof generation should succeed");

        (proof, y_blinding.map(|b| ark_to_jolt(&b)))
    }

    fn verify<ProofTranscript: Transcript>(
        &self,
        proof: &Self::Proof,
        setup: &Self::VerifierSetup,
        transcript: &mut ProofTranscript,
        opening_point: &[<ark_bn254::Fr as JoltField>::Challenge],
        opening: &ark_bn254::Fr,
        commitment: &Self::Commitment,
    ) -> Result<(), ProofVerifyError> {
        let _span = trace_span!("DoryCommitmentScheme::verify").entered();

        // Dory uses the opposite endian-ness as Jolt
        let ark_point: Vec<ArkFr> = opening_point
            .iter()
            .rev()
            .map(|p| {
                let f_val: ark_bn254::Fr = (*p).into();
                jolt_to_ark(&f_val)
            })
            .collect();
        let ark_eval: ArkFr = jolt_to_ark(opening);

        let mut dory_transcript = JoltToDoryTranscript::<ProofTranscript>::new(transcript);

        #[cfg(not(feature = "zk"))]
        if proof.e2.is_some()
            || proof.y_com.is_some()
            || proof.sigma1_proof.is_some()
            || proof.sigma2_proof.is_some()
            || proof.scalar_product_proof.is_some()
        {
            return Err(ProofVerifyError::InvalidOpeningProof);
        }

        dory::verify::<ArkFr, BN254, JoltG1Routines, JoltG2Routines, _>(
            *commitment,
            ark_eval,
            &ark_point,
            proof,
            setup.clone().into_inner(),
            &mut dory_transcript,
        )
        .map_err(|err| ProofVerifyError::DoryError(format!("dory::verify failed: {err:?}")))?;

        Ok(())
    }

    fn batch_prove<ProofTranscript: Transcript, S: BatchPolynomialSource<Self::Field>>(
        &self,
        layout: &Self::CommitmentLayout,
        setup: &Self::ProverSetup,
        poly_source: &S,
        _batch_hint: Self::BatchOpeningHint,
        individual_hints: Vec<Self::OpeningProofHint>,
        commitments: &[&Self::Commitment],
        opening_point: &[<Self::Field as JoltField>::Challenge],
        _claims: &[Self::Field],
        coeffs: &[Self::Field],
        transcript: &mut ProofTranscript,
    ) -> Self::BatchedProof {
        let joint_poly = poly_source.build_joint_polynomial(coeffs);
        let combined_hint =
            Self::combine_hints_for_rows(layout.num_rows(), individual_hints, coeffs);
        let joint_commitment = Self::combine_commitments_internal(commitments, coeffs);
        let (proof, y_blinding) = self.prove(
            layout,
            setup,
            &joint_poly,
            opening_point,
            Some(combined_hint),
            transcript,
            &joint_commitment,
        );
        let _ = &y_blinding;
        DoryBatchedProof {
            proof,
            layout: layout.orientation(),
            #[cfg(feature = "zk")]
            y_blinding,
        }
    }

    fn batch_verify<ProofTranscript: Transcript>(
        &self,
        proof: &Self::BatchedProof,
        setup: &Self::VerifierSetup,
        transcript: &mut ProofTranscript,
        opening_point: &[<Self::Field as JoltField>::Challenge],
        commitments: &[&Self::Commitment],
        claims: &[Self::Field],
        coeffs: &[Self::Field],
    ) -> Result<(), ProofVerifyError> {
        let joint_commitment = Self::combine_commitments_internal(commitments, coeffs);
        let joint_claim: ark_bn254::Fr = coeffs.iter().zip(claims).map(|(c, v)| *c * *v).sum();
        self.verify(
            &proof.proof,
            setup,
            transcript,
            opening_point,
            &joint_claim,
            &joint_commitment,
        )
    }

    fn split_batch_hint(batch_hint: &Self::BatchOpeningHint) -> Vec<Self::OpeningProofHint> {
        batch_hint.clone()
    }

    #[tracing::instrument(skip_all, name = "DoryCommitmentScheme::combine_hints")]
    fn combine_hints(
        hints: Vec<Self::OpeningProofHint>,
        coeffs: &[Self::Field],
    ) -> Self::OpeningProofHint {
        let num_rows = hints
            .iter()
            .map(|hint| hint.row_commitments().len())
            .max()
            .unwrap_or(0);
        Self::combine_hints_for_rows(num_rows, hints, coeffs)
    }

    fn protocol_name() -> &'static [u8] {
        b"Dory"
    }
}

impl DoryCommitmentScheme {
    fn combine_hints_for_rows(
        num_rows: usize,
        hints: Vec<DoryOpeningProofHint>,
        coeffs: &[ark_bn254::Fr],
    ) -> DoryOpeningProofHint {
        let mut rlc_hint = vec![ArkG1(G1Projective::zero()); num_rows];
        let mut rlc_commit_blind = <ArkFr as DoryField>::zero();
        for (coeff, hint) in coeffs.iter().zip(hints) {
            let DoryOpeningProofHint {
                mut row_commitments,
                commit_blind,
            } = hint;
            row_commitments.resize(num_rows, ArkG1(G1Projective::zero()));
            let ark_coeff = jolt_to_ark(coeff);
            rlc_commit_blind = rlc_commit_blind + ark_coeff * commit_blind;

            // SAFETY: ArkG1 is repr(transparent) over G1Projective
            let row_commitment_projects: &mut [G1Projective] = unsafe {
                std::slice::from_raw_parts_mut(
                    row_commitments.as_mut_ptr() as *mut G1Projective,
                    row_commitments.len(),
                )
            };

            let rlc_row_commitments: &[G1Projective] = unsafe {
                std::slice::from_raw_parts(rlc_hint.as_ptr() as *const G1Projective, rlc_hint.len())
            };

            let _span = trace_span!("vector_scalar_mul_add_gamma_g1_online");
            let _enter = _span.enter();

            jolt_optimizations::vector_scalar_mul_add_gamma_g1_online(
                row_commitment_projects,
                *coeff,
                rlc_row_commitments,
            );

            let _ = std::mem::replace(&mut rlc_hint, row_commitments);
        }

        DoryOpeningProofHint::new(rlc_hint, rlc_commit_blind)
    }
    #[tracing::instrument(skip_all, name = "DoryCommitmentScheme::combine_commitments_internal")]
    pub(crate) fn combine_commitments_internal(
        commitments: &[&ArkGT],
        coeffs: &[ark_bn254::Fr],
    ) -> ArkGT {
        coeffs
            .par_iter()
            .zip(commitments.par_iter())
            .map(|(coeff, commitment)| {
                let ark_coeff = jolt_to_ark(coeff);
                ark_coeff * **commitment
            })
            .reduce(ArkGT::identity, |a, b| a + b)
    }
}

impl StreamingCommitmentScheme for DoryCommitmentScheme {
    type ChunkState = Vec<ArkG1>;

    #[allow(non_snake_case)]
    fn streaming_chunk_size(
        &self,
        layout: &Self::CommitmentLayout,
        _K: usize,
        _T: usize,
    ) -> Option<usize> {
        if layout.orientation() == DoryLayout::AddressMajor {
            None
        } else {
            Some(layout.num_columns())
        }
    }

    #[tracing::instrument(skip_all, name = "DoryCommitmentScheme::compute_tier1_commitment")]
    fn process_chunk<T: SmallScalar>(
        &self,
        setup: &Self::ProverSetup,
        chunk: &[T],
    ) -> Self::ChunkState {
        let row_len = chunk.len();
        let g1_slice =
            unsafe { std::slice::from_raw_parts(setup.g1_vec.as_ptr(), setup.g1_vec.len()) };

        let g1_bases: Vec<G1Affine> = g1_slice[..row_len]
            .iter()
            .map(|g| g.0.into_affine())
            .collect();

        let row_commitment =
            ArkG1(T::msm(&g1_bases[..chunk.len()], chunk).expect("MSM calculation failed."));
        vec![row_commitment]
    }

    #[tracing::instrument(
        skip_all,
        name = "DoryCommitmentScheme::compute_tier1_commitment_onehot"
    )]
    fn process_chunk_onehot(
        &self,
        setup: &Self::ProverSetup,
        onehot_k: usize,
        chunk: &[Option<usize>],
    ) -> Self::ChunkState {
        let K = onehot_k;
        let row_len = chunk.len();

        let g1_slice =
            unsafe { std::slice::from_raw_parts(setup.g1_vec.as_ptr(), setup.g1_vec.len()) };

        let g1_bases: Vec<G1Affine> = g1_slice[..row_len]
            .iter()
            .map(|g| g.0.into_affine())
            .collect();

        let mut indices_per_k: Vec<Vec<usize>> = vec![Vec::new(); K];
        for (col_index, k) in chunk.iter().enumerate() {
            if let Some(k) = k {
                indices_per_k[*k].push(col_index);
            }
        }

        let results = jolt_optimizations::batch_g1_additions_multi(&g1_bases, &indices_per_k);

        let mut row_commitments = vec![ArkG1(G1Projective::zero()); K];
        for (k, result) in results.into_iter().enumerate() {
            if !indices_per_k[k].is_empty() {
                row_commitments[k] = ArkG1(G1Projective::from(result));
            }
        }
        row_commitments
    }

    #[tracing::instrument(skip_all, name = "DoryCommitmentScheme::compute_tier2_commitment")]
    fn aggregate_chunks(
        &self,
        setup: &Self::ProverSetup,
        onehot_k: Option<usize>,
        chunks: &[Self::ChunkState],
    ) -> (Self::Commitment, Self::OpeningProofHint) {
        if let Some(K) = onehot_k {
            let rows_per_k = chunks.len();
            let num_rows = K * rows_per_k;

            let mut row_commitments = vec![ArkG1(G1Projective::zero()); num_rows];
            for (chunk_index, commitments) in chunks.iter().enumerate() {
                row_commitments
                    .par_iter_mut()
                    .skip(chunk_index)
                    .step_by(rows_per_k)
                    .zip(commitments.par_iter())
                    .for_each(|(dest, src)| *dest = *src);
            }

            let g2_bases = &setup.g2_vec[..num_rows];
            let tier_2 = <BN254 as PairingCurve>::multi_pair_g2_setup(&row_commitments, g2_bases);
            let (tier_2, commit_blind) = maybe_blind_commitment(setup, tier_2);

            (
                tier_2,
                DoryOpeningProofHint::new(row_commitments, commit_blind),
            )
        } else {
            let row_commitments: Vec<ArkG1> =
                chunks.iter().flat_map(|chunk| chunk.clone()).collect();

            let g2_bases = &setup.g2_vec[..row_commitments.len()];
            let tier_2 = <BN254 as PairingCurve>::multi_pair_g2_setup(&row_commitments, g2_bases);
            let (tier_2, commit_blind) = maybe_blind_commitment(setup, tier_2);

            (
                tier_2,
                DoryOpeningProofHint::new(row_commitments, commit_blind),
            )
        }
    }

    fn streaming_batch_hint(hints: Vec<Self::OpeningProofHint>) -> Self::BatchOpeningHint {
        hints
    }
}

impl<C: JoltCurve> ZkEvalCommitment<C> for DoryCommitmentScheme
where
    C::G1: From<ArkG1>,
{
    fn eval_commitment(proof: &Self::BatchedProof) -> Option<C::G1> {
        #[cfg(feature = "zk")]
        {
            proof.proof.y_com.as_ref().copied().map(C::G1::from)
        }
        #[cfg(not(feature = "zk"))]
        {
            let _ = proof;
            None
        }
    }

    #[cfg(feature = "zk")]
    fn eval_blinding(proof: &Self::BatchedProof) -> Option<Self::Field> {
        proof.y_blinding
    }

    fn eval_commitment_gens(setup: &Self::ProverSetup) -> Option<(C::G1, C::G1)> {
        let g1_0 = setup.0.g1_vec.first().copied().map(C::G1::from)?;
        let h1 = C::G1::from(setup.0.h1);
        Some((g1_0, h1))
    }

    fn eval_commitment_gens_verifier(setup: &Self::VerifierSetup) -> Option<(C::G1, C::G1)> {
        let g1_0 = C::G1::from(setup.0.g1_0);
        let h1 = C::G1::from(setup.0.h1);
        Some((g1_0, h1))
    }

    #[cfg(feature = "zk")]
    fn zk_generators(setup: &Self::ProverSetup, count: usize) -> Option<(Vec<C::G1>, C::G1)> {
        let count = std::cmp::min(count, setup.0.g1_vec.len());
        let g1s = setup.0.g1_vec[..count]
            .iter()
            .map(|g| C::G1::from(*g))
            .collect();
        let h1 = C::G1::from(setup.0.h1);
        Some((g1s, h1))
    }
}
