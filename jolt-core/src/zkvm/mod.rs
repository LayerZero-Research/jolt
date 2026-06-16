use std::fs::File;

use crate::zkvm::config::{OneHotConfig, OneHotParams, ProgramMode, ReadWriteConfig};
use crate::zkvm::witness::CommittedPolynomial;
use crate::{
    curve::Bn254Curve,
    field::JoltField,
    poly::commitment::commitment_scheme::CommitmentScheme,
    poly::commitment::dory::DoryCommitmentScheme,
    poly::commitment::opening_point::FinalOpeningPointParts,
    poly::opening_proof::{
        OpeningAccumulator, OpeningId, OpeningPoint, ProverOpeningAccumulator, SumcheckId,
        BIG_ENDIAN,
    },
    transcripts::Blake2bTranscript,
    transcripts::Transcript,
    utils::errors::ProofVerifyError,
    zkvm::claim_reductions::AdviceKind,
};
use ark_bn254::Fr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use eyre::Result;
use proof_serialization::JoltProof;
#[cfg(feature = "prover")]
use prover::JoltCpuProver;
use std::io::Cursor;
use std::path::PathBuf;
use tracer::JoltDevice;
use verifier::JoltVerifier;

pub mod bytecode;
pub mod claim_reductions;
pub mod config;
pub mod instruction;
pub mod instruction_lookups;
pub mod lookup_table;
pub mod program;
pub mod proof_serialization;
#[cfg(feature = "prover")]
pub mod prover;
pub mod r1cs;
pub mod ram;
pub mod registers;
pub mod spartan;
#[cfg(all(test, feature = "host"))]
mod trace_row_parity;
pub mod transpilable_verifier;
pub mod verifier;
pub mod witness;

pub(crate) fn stage8_opening_ids(
    one_hot_params: &OneHotParams,
    include_trusted_advice: bool,
    include_untrusted_advice: bool,
    program_mode: ProgramMode,
    bytecode_chunk_count: usize,
) -> Vec<OpeningId> {
    let mut opening_ids = Vec::new();

    opening_ids.push(OpeningId::committed(
        CommittedPolynomial::RamInc,
        SumcheckId::IncClaimReduction,
    ));
    opening_ids.push(OpeningId::committed(
        CommittedPolynomial::RdInc,
        SumcheckId::IncClaimReduction,
    ));

    for i in 0..one_hot_params.instruction_d {
        opening_ids.push(OpeningId::committed(
            CommittedPolynomial::InstructionRa(i),
            SumcheckId::HammingWeightClaimReduction,
        ));
    }
    for i in 0..one_hot_params.bytecode_d {
        opening_ids.push(OpeningId::committed(
            CommittedPolynomial::BytecodeRa(i),
            SumcheckId::HammingWeightClaimReduction,
        ));
    }
    for i in 0..one_hot_params.ram_d {
        opening_ids.push(OpeningId::committed(
            CommittedPolynomial::RamRa(i),
            SumcheckId::HammingWeightClaimReduction,
        ));
    }

    if include_trusted_advice {
        opening_ids.push(OpeningId::TrustedAdvice(SumcheckId::AdviceClaimReduction));
    }
    if include_untrusted_advice {
        opening_ids.push(OpeningId::UntrustedAdvice(SumcheckId::AdviceClaimReduction));
    }
    if program_mode == ProgramMode::Committed {
        for i in 0..bytecode_chunk_count {
            opening_ids.push(OpeningId::committed(
                CommittedPolynomial::BytecodeChunk(i),
                SumcheckId::BytecodeClaimReduction,
            ));
        }
    }
    if program_mode == ProgramMode::Committed {
        opening_ids.push(OpeningId::committed(
            CommittedPolynomial::ProgramImageInit,
            SumcheckId::ProgramImageClaimReduction,
        ));
    }

    opening_ids
}

pub(crate) fn final_opening_point_parts<F: JoltField>(
    opening_accumulator: &impl OpeningAccumulator<F>,
    native_main_vars: usize,
    log_k_chunk: usize,
    program_mode: ProgramMode,
    bytecode_chunk_count: usize,
) -> Result<FinalOpeningPointParts<F>, ProofVerifyError> {
    let mut opening_candidates: Vec<(String, OpeningPoint<BIG_ENDIAN, F>)> = Vec::new();
    if let Some((point, _)) = opening_accumulator
        .get_advice_opening(AdviceKind::Trusted, SumcheckId::AdviceClaimReduction)
    {
        opening_candidates.push(("trusted_advice".to_string(), point));
    }
    if let Some((point, _)) = opening_accumulator
        .get_advice_opening(AdviceKind::Untrusted, SumcheckId::AdviceClaimReduction)
    {
        opening_candidates.push(("untrusted_advice".to_string(), point));
    }
    if program_mode == ProgramMode::Committed {
        for chunk_idx in 0..bytecode_chunk_count {
            let (point, _) = opening_accumulator.get_committed_polynomial_opening(
                CommittedPolynomial::BytecodeChunk(chunk_idx),
                SumcheckId::BytecodeClaimReduction,
            );
            opening_candidates.push((format!("bytecode_chunk[{chunk_idx}]"), point));
        }
    }
    if program_mode == ProgramMode::Committed {
        let (program_image_point, _) = opening_accumulator.get_committed_polynomial_opening(
            CommittedPolynomial::ProgramImageInit,
            SumcheckId::ProgramImageClaimReduction,
        );
        opening_candidates.push(("program_image".to_string(), program_image_point));
    }

    let (hamming_point, _) = opening_accumulator.get_committed_polynomial_opening(
        CommittedPolynomial::InstructionRa(0),
        SumcheckId::HammingWeightClaimReduction,
    );
    let (r_cycle_stage6, _) = opening_accumulator.get_committed_polynomial_opening(
        CommittedPolynomial::RamInc,
        SumcheckId::IncClaimReduction,
    );

    let max_len = opening_candidates
        .iter()
        .map(|(_, point)| point.r.len())
        .max()
        .unwrap_or(0);
    if max_len > native_main_vars {
        let dominant = opening_candidates
            .iter()
            .find(|(_, point)| point.r.len() == max_len)
            .expect("at least one dominant precommitted candidate expected");
        for (name, point) in opening_candidates
            .iter()
            .filter(|(_, point)| point.r.len() == max_len)
        {
            if point.r != dominant.1.r {
                return Err(ProofVerifyError::DoryError(format!(
                    "incompatible dominant precommitted anchors: {} and {} have equal dimensionality {} but different opening points",
                    dominant.0, name, max_len
                )));
            }
        }
        Ok(FinalOpeningPointParts::DominantPrecommittedAnchor {
            point: OpeningPoint::<BIG_ENDIAN, F>::new(dominant.1.r.clone()),
        })
    } else {
        let r_address_stage7 = hamming_point.r[..log_k_chunk].to_vec();

        Ok(FinalOpeningPointParts::Native {
            r_address_stage7,
            r_cycle_stage6,
            hamming_point,
            log_k_chunk,
        })
    }
}

// Scoped CPU profiler for performance analysis. Feature-gated by "pprof".
// Usage: let _guard = pprof_scope!("label");
//
// Writes pprof/label.pb on scope exit
// View with: go tool pprof -http=:8080 pprof/label.pb

// Public type for the profiling guard
#[cfg(feature = "pprof")]
pub struct PprofGuard {
    guard: pprof::ProfilerGuard<'static>,
    label: &'static str,
}

#[cfg(not(feature = "pprof"))]
pub struct PprofGuard;

#[cfg(feature = "pprof")]
impl Drop for PprofGuard {
    fn drop(&mut self) {
        if let Ok(report) = self.guard.report().build() {
            let prefix = std::env::var("PPROF_PREFIX")
                .unwrap_or_else(|_| String::from("benchmark-runs/pprof/"));
            let filename = format!("{}{}.pb", prefix, self.label);
            // Extract directory from prefix for creation
            if let Some(dir) = std::path::Path::new(&filename).parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(mut f) = std::fs::File::create(&filename) {
                use pprof::protos::Message;
                if let Ok(p) = report.pprof() {
                    let mut buf = Vec::new();
                    if p.encode(&mut buf).is_ok() {
                        let _ = std::io::Write::write_all(&mut f, &buf);
                        tracing::info!("Wrote pprof profile to {}", filename);
                    }
                }
            }
        }
    }
}

#[macro_export]
macro_rules! pprof_scope {
    ($label:expr) => {{
        #[cfg(feature = "pprof")]
        {
            Some($crate::zkvm::PprofGuard {
                guard: pprof::ProfilerGuardBuilder::default()
                    .frequency(
                        std::env::var("PPROF_FREQ")
                            .unwrap_or("100".to_string())
                            .parse::<i32>()
                            .unwrap(),
                    )
                    .blocklist(&["libc", "libgcc", "pthread", "vdso"])
                    .build()
                    .expect("Failed to initialize profiler"),
                label: $label,
            })
        }
        #[cfg(not(feature = "pprof"))]
        None::<$crate::zkvm::PprofGuard>
    }};
    () => {
        pprof_scope!("default");
    };
}

#[allow(dead_code)]
pub struct ProverDebugInfo<F, ProofTranscript, PCS>
where
    F: JoltField,
    ProofTranscript: Transcript,
    PCS: CommitmentScheme<Field = F>,
{
    pub(crate) transcript: ProofTranscript,
    pub(crate) opening_accumulator: ProverOpeningAccumulator<F>,
    pub(crate) prover_setup: PCS::ProverSetup,
}

pub struct FiatShamirPreamble<'a, PCS: CommitmentScheme> {
    pub program_io: &'a JoltDevice,
    pub ram_K: usize,
    pub trace_length: usize,
    pub entry_address: u64,
    pub rw_config: &'a ReadWriteConfig,
    pub one_hot_config: &'a OneHotConfig,
    pub pcs_config: &'a PCS::Config,
    pub preprocessing_digest: &'a [u8; 32],
}

/// Absorb public instance data into the transcript for Fiat-Shamir.
pub fn fiat_shamir_preamble<PCS: CommitmentScheme>(
    preamble: FiatShamirPreamble<'_, PCS>,
    transcript: &mut impl Transcript,
) {
    let FiatShamirPreamble {
        program_io,
        ram_K,
        trace_length,
        entry_address,
        rw_config,
        one_hot_config,
        pcs_config,
        preprocessing_digest,
    } = preamble;

    transcript.append_bytes(b"preprocessing_digest", preprocessing_digest);
    transcript.append_u64(b"max_input_size", program_io.memory_layout.max_input_size);
    transcript.append_u64(b"max_output_size", program_io.memory_layout.max_output_size);
    transcript.append_u64(b"heap_size", program_io.memory_layout.heap_size);
    transcript.append_bytes(b"inputs", &program_io.inputs);
    transcript.append_bytes(b"outputs", &program_io.outputs);
    transcript.append_u64(b"panic", program_io.panic as u64);
    transcript.append_u64(b"ram_K", ram_K as u64);
    transcript.append_u64(b"trace_length", trace_length as u64);
    transcript.append_u64(b"entry_address", entry_address);
    transcript.append_u64(
        b"ram_rw_phase1_num_rounds",
        rw_config.ram_rw_phase1_num_rounds as u64,
    );
    transcript.append_u64(
        b"ram_rw_phase2_num_rounds",
        rw_config.ram_rw_phase2_num_rounds as u64,
    );
    transcript.append_u64(
        b"registers_rw_phase1_num_rounds",
        rw_config.registers_rw_phase1_num_rounds as u64,
    );
    transcript.append_u64(
        b"registers_rw_phase2_num_rounds",
        rw_config.registers_rw_phase2_num_rounds as u64,
    );
    transcript.append_u64(b"log_k_chunk", one_hot_config.log_k_chunk as u64);
    transcript.append_u64(
        b"lookups_ra_virtual_log_k_chunk",
        one_hot_config.lookups_ra_virtual_log_k_chunk as u64,
    );
    PCS::append_pcs_config_to_transcript(pcs_config, transcript);
}

#[cfg(feature = "prover")]
pub type RV64IMACProver<'a> =
    JoltCpuProver<'a, Fr, Bn254Curve, DoryCommitmentScheme, Blake2bTranscript>;
pub type RV64IMACVerifier<'a> =
    JoltVerifier<'a, Fr, Bn254Curve, DoryCommitmentScheme, Blake2bTranscript>;
pub type RV64IMACProof = JoltProof<Fr, Bn254Curve, DoryCommitmentScheme, Blake2bTranscript>;

pub type AkitaPcs = crate::poly::commitment::akita::JoltAkitaCommitmentScheme<
    { <crate::poly::commitment::akita::Fp128OneHot32Config as akita_config::CommitmentConfig>::D },
    crate::poly::commitment::akita::Fp128OneHot32Config,
>;
#[cfg(feature = "prover")]
pub type RV64IMACAkitaProver<'a> = JoltCpuProver<
    'a,
    crate::field::fp128::JoltFp128,
    crate::curve::fp128_curve::Fp128Curve,
    AkitaPcs,
    Blake2bTranscript,
>;
pub type RV64IMACAkitaVerifier<'a> = JoltVerifier<
    'a,
    crate::field::fp128::JoltFp128,
    crate::curve::fp128_curve::Fp128Curve,
    AkitaPcs,
    Blake2bTranscript,
>;

pub trait Serializable: CanonicalSerialize + CanonicalDeserialize + Sized {
    /// Gets the byte size of the serialized data
    fn size(&self) -> Result<usize> {
        let mut buffer = Vec::new();
        self.serialize_compressed(&mut buffer)?;
        Ok(buffer.len())
    }

    /// Saves the data to a file
    fn save_to_file<P: Into<PathBuf>>(&self, path: P) -> Result<()> {
        let file = File::create(path.into())?;
        self.serialize_compressed(file)?;
        Ok(())
    }

    /// Reads data from a file
    fn from_file<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let file = File::open(path.into())?;
        Ok(Self::deserialize_compressed(file)?)
    }

    /// Serializes the data to a byte vector
    fn serialize_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.serialize_compressed(&mut buffer)?;
        Ok(buffer)
    }

    /// Deserializes data from a byte vector
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(bytes);
        Ok(Self::deserialize_compressed(cursor)?)
    }
}

impl Serializable for RV64IMACProof {}
impl Serializable for JoltDevice {}
