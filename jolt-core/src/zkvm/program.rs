//! Unified program preprocessing module.
//!
//! This module contains all static program data derived from the ELF:
//! - **Instructions** (`instructions`, `pc_map`): Decoded RISC-V instructions for bytecode lookup tables
//! - **Program image** (`min_bytecode_address`, `program_image_words`): Initial RAM state
//!
//! Both come from the same ELF file and are conceptually "the program".

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;

use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use common::constants::BYTES_PER_INSTRUCTION;
use rayon::prelude::*;
use tracer::instruction::{Cycle, Instruction};

use crate::poly::commitment::commitment_scheme::CommitmentScheme;
use crate::poly::commitment::dory::{
    ArkG1, ArkGT, ArkworksProverSetup, DoryContext, DoryGlobals, DoryLayout, BN254,
};
use crate::poly::multilinear_polynomial::MultilinearPolynomial;
use crate::utils::errors::ProofVerifyError;
use crate::utils::math::Math;
use crate::zkvm::bytecode::chunks::{
    committed_bytecode_chunk_cycle_len, committed_lanes, for_each_active_lane_value,
    validate_committed_bytecode_chunking_for_len, ActiveLaneValue,
};
pub use crate::zkvm::bytecode::BytecodePCMapper;
use ark_bn254::{Fr, G1Projective};
use ark_ff::{One, Zero};
use dory::primitives::arithmetic::PairingCurve;

// ─────────────────────────────────────────────────────────────────────────────
// ProgramPreprocessing - Full program data (prover + full-mode verifier)
// ─────────────────────────────────────────────────────────────────────────────

/// Full program preprocessing - includes both bytecode instructions and RAM image.
///
/// Both come from the same ELF file:
/// - `instructions` + `pc_map`: for bytecode lookup tables
/// - `program_image_words`: for initial RAM state
///
/// # Usage
/// - Prover always has full access to this data
/// - In Full mode, verifier also has full access
/// - In Committed mode, verifier only has `TrustedProgramCommitments`
#[derive(Debug, Clone, CanonicalSerialize, CanonicalDeserialize)]
pub struct ProgramPreprocessing {
    // ─── Bytecode (instructions) ───
    /// Decoded RISC-V instructions (padded to power-of-2).
    pub instructions: Vec<Instruction>,
    /// PC mapping for instruction lookup.
    pub pc_map: BytecodePCMapper,

    // ─── Program image (RAM init) ───
    /// Minimum bytecode address (word-aligned).
    pub min_bytecode_address: u64,
    /// Program-image words (little-endian packed u64 values).
    pub program_image_words: Vec<u64>,
}

impl Default for ProgramPreprocessing {
    fn default() -> Self {
        Self {
            instructions: vec![Instruction::NoOp, Instruction::NoOp],
            pc_map: BytecodePCMapper::default(),
            min_bytecode_address: 0,
            program_image_words: Vec::new(),
        }
    }
}

impl ProgramPreprocessing {
    /// Preprocess program from decoded ELF outputs.
    ///
    /// # Arguments
    /// - `instructions`: Decoded RISC-V instructions from ELF
    /// - `memory_init`: Raw bytes from ELF that form initial RAM
    #[tracing::instrument(skip_all, name = "ProgramPreprocessing::preprocess")]
    pub fn preprocess(instructions: Vec<Instruction>, memory_init: Vec<(u64, u8)>) -> Self {
        // ─── Process instructions (from BytecodePreprocessing::preprocess) ───
        let mut bytecode = instructions;
        // Prepend a single no-op instruction
        bytecode.insert(0, Instruction::NoOp);
        let pc_map = BytecodePCMapper::new(&bytecode);

        let bytecode_size = bytecode.len().next_power_of_two().max(2);
        // Pad to nearest power of 2
        bytecode.resize(bytecode_size, Instruction::NoOp);

        // ─── Process program image (from ProgramImagePreprocessing::preprocess) ───
        let min_bytecode_address = memory_init
            .iter()
            .map(|(address, _)| *address)
            .min()
            .unwrap_or(0);

        let max_bytecode_address = memory_init
            .iter()
            .map(|(address, _)| *address)
            .max()
            .unwrap_or(0)
            + (BYTES_PER_INSTRUCTION as u64 - 1);

        let num_words = max_bytecode_address.next_multiple_of(8) / 8 - min_bytecode_address / 8 + 1;
        let mut program_image_words = vec![0u64; num_words as usize];
        tracing::info!("num_words: {}", num_words);
        tracing::info!("max_bytecode_address: {}", max_bytecode_address);
        tracing::info!("min_bytecode_address: {}", min_bytecode_address);
        // Convert bytes into words and populate `program_image_words`
        for chunk in
            memory_init.chunk_by(|(address_a, _), (address_b, _)| address_a / 8 == address_b / 8)
        {
            let mut word = [0u8; 8];
            for (address, byte) in chunk {
                word[(address % 8) as usize] = *byte;
            }
            let word = u64::from_le_bytes(word);
            let remapped_index = (chunk[0].0 / 8 - min_bytecode_address / 8) as usize;
            program_image_words[remapped_index] = word;
        }

        Self {
            instructions: bytecode,
            pc_map,
            min_bytecode_address,
            program_image_words,
        }
    }

    /// Bytecode length (power-of-2 padded).
    pub fn bytecode_len(&self) -> usize {
        self.instructions.len()
    }

    /// Program image word count (unpadded).
    pub fn program_image_len_words(&self) -> usize {
        self.program_image_words.len()
    }

    /// Program image word count (power-of-2 padded).
    pub fn program_image_len_words_padded(&self) -> usize {
        self.program_image_words.len().next_power_of_two().max(2)
    }

    /// Extract metadata-only for shared preprocessing.
    pub fn meta(&self) -> ProgramMetadata {
        ProgramMetadata {
            min_bytecode_address: self.min_bytecode_address,
            program_image_len_words: self.program_image_words.len(),
            bytecode_len: self.instructions.len(),
        }
    }

    /// Get PC for a given cycle (instruction lookup).
    #[inline(always)]
    pub fn get_pc(&self, cycle: &Cycle) -> usize {
        if matches!(cycle, Cycle::NoOp) {
            return 0;
        }
        let instr = cycle.instruction().normalize();
        self.pc_map
            .get_pc(instr.address, instr.virtual_sequence_remaining.unwrap_or(0))
    }

    /// Get a BytecodePreprocessing-compatible view.
    ///
    /// This is for backward compatibility with code that expects BytecodePreprocessing.
    pub fn as_bytecode(&self) -> crate::zkvm::bytecode::BytecodePreprocessing {
        crate::zkvm::bytecode::BytecodePreprocessing {
            bytecode: self.instructions.clone(),
            pc_map: self.pc_map.clone(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ProgramMetadata - O(1) metadata (shared between prover and verifier)
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata-only program info (shared between prover and verifier).
///
/// O(1) data, safe for committed mode verifier. Does NOT contain
/// the actual instructions or program image words.
#[derive(Debug, Clone, CanonicalSerialize, CanonicalDeserialize)]
pub struct ProgramMetadata {
    /// Minimum bytecode address (word-aligned).
    pub min_bytecode_address: u64,
    /// Number of program-image words (unpadded).
    pub program_image_len_words: usize,
    /// Bytecode length (power-of-2 padded).
    pub bytecode_len: usize,
}

impl ProgramMetadata {
    /// Create metadata from full preprocessing.
    pub fn from_program(program: &ProgramPreprocessing) -> Self {
        program.meta()
    }

    /// Program image word count (power-of-2 padded).
    pub fn program_image_len_words_padded(&self) -> usize {
        self.program_image_len_words.next_power_of_two().max(2)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TrustedProgramCommitments - Unified commitments for committed mode
// ─────────────────────────────────────────────────────────────────────────────

/// Trusted commitments for the entire program (bytecode + program image).
///
/// Derived from full `ProgramPreprocessing` during offline preprocessing.
/// This is what the verifier receives in Committed mode.
///
/// # Trust Model
/// - Create via `derive()` from full program (offline preprocessing)
/// - Or deserialize from a trusted source (assumes honest origin)
/// - Pass to verifier preprocessing for succinct (online) verification
///
/// # Security Warning
/// If you construct this type with arbitrary commitments (bypassing `derive()`),
/// verification will be unsound. Only use `derive()` or trusted deserialization.
#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct TrustedProgramCommitments<PCS: CommitmentScheme> {
    // ─── Bytecode commitment ───
    /// Commitments to individual chunked bytecode chunk polynomials.
    ///
    /// For chunk count `c`, this contains exactly `c` commitments:
    /// `BytecodeChunk(0)..BytecodeChunk(c-1)`.
    pub bytecode_chunk_commitments: Vec<PCS::Commitment>,
    /// Bytecode-context column count used when deriving bytecode commitments.
    ///
    /// This is derived from bytecode dimensions only (`K_bytecode * bytecode_len`) and is
    /// independent from Main-context matrix dimensions.
    pub bytecode_num_columns: usize,
    /// log2(k_chunk) from one-hot configuration (used for main-matrix sizing).
    pub log_k_chunk: u8,
    /// Bytecode length (power-of-two padded).
    pub bytecode_len: usize,
    /// Chunk count used to build committed bytecode chunk polynomials.
    ///
    /// - `1` means a single chunk
    /// - `c > 1` splits cycles into `c` chunks stacked on one cycle domain
    pub bytecode_chunk_count: usize,
    /// The T value used for bytecode coefficient indexing.
    /// In committed mode this is fixed to `bytecode_len` so bytecode always embeds
    /// into the top-left block of the Main matrix.
    /// Used in Stage 8 VMP to ensure correct index mapping.
    pub bytecode_T: usize,

    // ─── Program image commitment ───
    /// Commitment to the program-image polynomial.
    pub program_image_commitment: PCS::Commitment,
    /// Number of columns used when committing program image.
    pub program_image_num_columns: usize,
    /// Number of program-image words (power-of-two padded).
    pub program_image_num_words: usize,
}

/// Opening hints for `TrustedProgramCommitments`.
///
/// These are the Dory tier-1 data needed to build opening proofs.
#[derive(Clone, CanonicalSerialize, CanonicalDeserialize)]
pub struct TrustedProgramHints<PCS: CommitmentScheme> {
    /// Hints for each chunked bytecode chunk polynomial commitment.
    pub bytecode_chunk_hints: Vec<PCS::OpeningProofHint>,
    /// Hint for program image commitment.
    pub program_image_hint: PCS::OpeningProofHint,
}

impl<PCS: CommitmentScheme> TrustedProgramCommitments<PCS> {
    /// Derive all program commitments from full preprocessing.
    ///
    /// This is the "offline preprocessing" step that must be done honestly.
    /// Returns trusted commitments + hints for opening proofs.
    #[tracing::instrument(skip_all, name = "TrustedProgramCommitments::derive")]
    pub fn derive(
        program: &ProgramPreprocessing,
        generators: &PCS::ProverSetup,
        log_k_chunk: usize,
        bytecode_chunk_count: usize,
    ) -> (Self, TrustedProgramHints<PCS>) {
        // ─── Derive bytecode commitments ───
        let bytecode_len = program.bytecode_len();
        validate_committed_bytecode_chunking_for_len(bytecode_len, bytecode_chunk_count);

        // Get layout before context initialization. Layout affects coefficient indexing.
        let layout = DoryGlobals::get_layout();

        // Bytecode commitments use the sparse Dory path.
        let dory_setup: &ArkworksProverSetup =
            unsafe { &*(generators as *const PCS::ProverSetup as *const ArkworksProverSetup) };
        let (commitments, hints, bytecode_num_columns, bytecode_T) =
            derive_bytecode_chunk_commitments_sparse_dory(
                program,
                dory_setup,
                layout,
                bytecode_chunk_count,
            );
        let bytecode_chunk_commitments: Vec<PCS::Commitment> =
            unsafe { std::mem::transmute(commitments) };
        let bytecode_chunk_hints: Vec<PCS::OpeningProofHint> =
            unsafe { std::mem::transmute(hints) };

        // ─── Derive program image commitment ───
        // Pad to a power-of-two length (minimum 1).
        let program_image_num_words = program.program_image_len_words().next_power_of_two().max(1);

        // Commit ProgramImage in its own balanced context (advice-style top-left embedding).
        let _guard_prog = DoryGlobals::initialize_context(
            1,
            program_image_num_words,
            DoryContext::ProgramImage,
            None,
        );
        let _ctx2 = DoryGlobals::with_context(DoryContext::ProgramImage);
        let program_image_num_columns = DoryGlobals::get_num_columns();

        // Build program image polynomial with padded size
        let program_image_mle: MultilinearPolynomial<PCS::Field> =
            build_program_image_polynomial_padded(program, program_image_num_words);
        let (program_image_commitment, program_image_hint) =
            PCS::commit(&program_image_mle, generators);

        (
            Self {
                bytecode_chunk_commitments,
                bytecode_num_columns,
                log_k_chunk: log_k_chunk as u8,
                bytecode_len,
                bytecode_chunk_count,
                bytecode_T,
                program_image_commitment,
                program_image_num_columns,
                program_image_num_words,
            },
            TrustedProgramHints {
                bytecode_chunk_hints,
                program_image_hint,
            },
        )
    }
}

/// Build program-image polynomial from ProgramPreprocessing with explicit padded size.
///
/// Implementation note: we store program-image coefficients as `u64` small scalars (U64Scalars)
/// to avoid eagerly converting the entire image to field elements.
pub(crate) fn build_program_image_polynomial_padded<F: crate::field::JoltField>(
    program: &ProgramPreprocessing,
    padded_len: usize,
) -> MultilinearPolynomial<F> {
    debug_assert!(padded_len.is_power_of_two());
    debug_assert!(padded_len >= program.program_image_words.len());
    let mut coeffs = vec![0u64; padded_len];
    for (i, &word) in program.program_image_words.iter().enumerate() {
        coeffs[i] = word;
    }
    MultilinearPolynomial::from(coeffs)
}

/// Committed-mode bytecode matrix dimensions.
///
/// Flow is explicit:
/// 1) use chunked bytecode cycle domain (`T = bytecode_len / chunk_count`)
/// 2) derive bytecode width from bytecode total variables only (`K_bytecode * T`)
#[inline]
fn committed_bytecode_dimensions(
    bytecode_len: usize,
    bytecode_chunk_count: usize,
) -> (usize, usize) {
    debug_assert!(bytecode_len.is_power_of_two());
    let chunk_cycle_len = committed_bytecode_chunk_cycle_len(bytecode_len, bytecode_chunk_count);
    let total_vars = committed_lanes().log_2() + chunk_cycle_len.log_2();
    let (sigma_bytecode, _) = DoryGlobals::balanced_sigma_nu(total_vars);
    let bytecode_num_columns = 1usize << sigma_bytecode;
    (bytecode_num_columns, chunk_cycle_len)
}

/// Streaming/sparse committed-bytecode chunk commitments for Dory.
///
/// Computes tier-1 row commitments directly from the instruction stream by only touching
/// nonzero lane values (via `for_each_active_lane_value`). This avoids materializing the
/// dense coefficient vectors for committed bytecode chunk polynomials.
///
/// Returns:
/// - commitments: committed bytecode chunk polynomial commitments (one per chunk)
/// - hints: tier-1 row commitments (Dory opening proof hint) per chunk
/// - num_columns: bytecode context matrix width
/// - bytecode_T: the T used for coefficient indexing (needed later in Stage 8 VMP)
fn derive_bytecode_chunk_commitments_sparse_dory(
    program: &ProgramPreprocessing,
    setup: &ArkworksProverSetup,
    layout: DoryLayout,
    bytecode_chunk_count: usize,
) -> (Vec<ArkGT>, Vec<Vec<ArkG1>>, usize, usize) {
    let k_bytecode = committed_lanes();
    let bytecode_len = program.bytecode_len();

    let (balanced_num_columns, bytecode_T) =
        committed_bytecode_dimensions(bytecode_len, bytecode_chunk_count);
    let total_vars = k_bytecode.log_2() + bytecode_T.log_2();
    let sigma_balanced = balanced_num_columns.log_2();

    // Validate that the balanced bytecode sigma fits the Dory setup capacity:
    // - columns need sigma <= log2(|G1|)
    // - rows need nu <= log2(|G2|) => sigma >= total_vars - log2(|G2|)
    //
    // In the canonical committed flow, `setup_generators_committed` provisions enough generators
    // for bytecode vars, so this should hold without any dimension adjustment.
    let max_sigma = setup.g1_vec.len().log_2();
    let max_nu = setup.g2_vec.len().log_2();
    let min_sigma = total_vars.saturating_sub(max_nu);
    assert!(
        min_sigma <= max_sigma,
        "bytecode commitment exceeds setup capacity: total_vars={total_vars}, setup(log_g1={max_sigma}, log_g2={max_nu})"
    );
    assert!(
        sigma_balanced >= min_sigma && sigma_balanced <= max_sigma,
        "balanced bytecode sigma out of setup range: sigma={sigma_balanced}, allowed=[{min_sigma},{max_sigma}] (total_vars={total_vars})"
    );
    let sigma_bytecode = sigma_balanced;
    let num_columns = 1usize << sigma_bytecode;
    let _guard = DoryGlobals::initialize_bytecode_context_with_dimensions(
        k_bytecode,
        bytecode_T,
        num_columns,
    );
    let _ctx = DoryGlobals::with_context(DoryContext::Bytecode);

    let total_size = k_bytecode * bytecode_T;
    let num_rows = if total_size >= num_columns {
        debug_assert!(
            total_size % num_columns == 0,
            "expected (k_bytecode*bytecode_T) divisible by num_columns"
        );
        total_size / num_columns
    } else {
        1
    };

    let chunk_results: Vec<(ArkGT, Vec<ArkG1>)> = (0..bytecode_chunk_count)
        .into_par_iter()
        .map(|chunk_idx| {
            // Build tier-1 row commitments for this chunk over its instruction slice only.
            let chunk_start = chunk_idx * bytecode_T;
            let chunk_end = chunk_start + bytecode_T;
            let sparse_rows: HashMap<usize, G1Projective> = program.instructions
                [chunk_start..chunk_end]
                .par_iter()
                .enumerate()
                .fold(
                    HashMap::<usize, G1Projective>::new,
                    |mut acc, (chunk_cycle, instr)| {
                        for_each_active_lane_value::<Fr>(instr, |global_lane, lane_val| {
                            let global_index = layout.address_cycle_to_index(
                                global_lane,
                                chunk_cycle,
                                k_bytecode,
                                bytecode_T,
                            );
                            let row_idx = global_index / num_columns;
                            let col_idx = global_index % num_columns;
                            debug_assert!(row_idx < num_rows);

                            let scalar = match lane_val {
                                ActiveLaneValue::One => Fr::one(),
                                ActiveLaneValue::Scalar(v) => v,
                            };
                            if scalar.is_zero() {
                                return;
                            }

                            let base = setup.g1_vec[col_idx].0;
                            let entry = acc.entry(row_idx).or_insert_with(G1Projective::zero);
                            if scalar.is_one() {
                                *entry += base;
                            } else {
                                *entry += base * scalar;
                            }
                        });
                        acc
                    },
                )
                .reduce(HashMap::<usize, G1Projective>::new, |mut a, b| {
                    for (row_idx, row_commitment) in b.into_iter() {
                        let entry = a.entry(row_idx).or_insert_with(G1Projective::zero);
                        *entry += row_commitment;
                    }
                    a
                });

            // Materialize full row-commitment vector (hint) and compute tier-2 commitment.
            let mut row_commitments: Vec<ArkG1> = vec![ArkG1(G1Projective::zero()); num_rows];
            let mut nonzero_rows: Vec<ArkG1> = Vec::with_capacity(sparse_rows.len());
            let mut nonzero_g2: Vec<_> = Vec::with_capacity(sparse_rows.len());
            let mut nonzero_indices: Vec<usize> = Vec::with_capacity(sparse_rows.len());

            for (row_idx, row_commitment) in sparse_rows.into_iter() {
                let rc = ArkG1(row_commitment);
                row_commitments[row_idx] = rc;
                nonzero_rows.push(rc);
                nonzero_g2.push(setup.g2_vec[row_idx].clone());
                nonzero_indices.push(row_idx);
            }

            let tier2 = <BN254 as PairingCurve>::multi_pair_g2_indexed(
                &nonzero_rows,
                &nonzero_g2,
                &nonzero_indices,
            );
            (tier2, row_commitments)
        })
        .collect();
    let (chunk_commitments, chunk_hints): (Vec<_>, Vec<_>) = chunk_results.into_iter().unzip();

    (chunk_commitments, chunk_hints, num_columns, bytecode_T)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_major_bytecode_dimensions_are_main_independent() {
        // Tiny program fixture: default preprocessing already has minimal padded bytecode length.
        let program = ProgramPreprocessing::default();
        let bytecode_len = program.bytecode_len();
        assert_eq!(bytecode_len, 2);

        // Example Main parameters (intentionally large so Main width differs).
        let log_k_chunk = 3usize;
        let k_bytecode = committed_lanes();
        let max_trace_len = 1usize << 20;
        let log_t = max_trace_len.log_2();
        let main_num_columns = DoryGlobals::main_num_columns(log_k_chunk, log_t);
        assert_eq!(main_num_columns, 4096);

        let total_size = k_bytecode * bytecode_len;
        assert_eq!(total_size, 1024);
        let total_vars = total_size.log_2();
        let (sigma_bytecode, _) = DoryGlobals::balanced_sigma_nu(total_vars);
        let expected_bytecode_cols = 1usize << sigma_bytecode;

        // Bytecode dimensions are derived from bytecode only.
        let (num_columns, bytecode_t) = committed_bytecode_dimensions(bytecode_len, 1);
        assert_eq!(bytecode_t, bytecode_len);
        assert_eq!(num_columns, expected_bytecode_cols);
        assert_ne!(num_columns, main_num_columns);

        let _guard = DoryGlobals::initialize_bytecode_context_with_dimensions(
            k_bytecode,
            bytecode_t,
            num_columns,
        );
        let _ctx = DoryGlobals::with_context(DoryContext::Bytecode);
        let (rows, cols) = DoryGlobals::matrix_shape();
        assert_eq!(cols, expected_bytecode_cols);
        assert_eq!(rows, total_size / expected_bytecode_cols);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VerifierProgram - Verifier's view of program data
// ─────────────────────────────────────────────────────────────────────────────

/// Verifier's view of program data.
///
/// - `Full`: Verifier has full access to the program data (O(program_size) data).
/// - `Committed`: Verifier only has trusted commitments (O(1) data).
#[derive(Debug, Clone)]
pub enum VerifierProgram<PCS: CommitmentScheme> {
    /// Full program data available (Full mode).
    Full(Arc<ProgramPreprocessing>),
    /// Only trusted commitments available (Committed mode).
    Committed(TrustedProgramCommitments<PCS>),
}

impl<PCS: CommitmentScheme> VerifierProgram<PCS> {
    /// Returns the full program preprocessing, or an error if in Committed mode.
    pub fn as_full(&self) -> Result<&Arc<ProgramPreprocessing>, ProofVerifyError> {
        match self {
            VerifierProgram::Full(p) => Ok(p),
            VerifierProgram::Committed(_) => Err(ProofVerifyError::BytecodeTypeMismatch(
                "expected Full, got Committed".to_string(),
            )),
        }
    }

    /// Returns true if this is Full mode.
    pub fn is_full(&self) -> bool {
        matches!(self, VerifierProgram::Full(_))
    }

    /// Returns true if this is Committed mode.
    pub fn is_committed(&self) -> bool {
        matches!(self, VerifierProgram::Committed(_))
    }

    /// Returns the trusted commitments, or an error if in Full mode.
    pub fn as_committed(&self) -> Result<&TrustedProgramCommitments<PCS>, ProofVerifyError> {
        match self {
            VerifierProgram::Committed(trusted) => Ok(trusted),
            VerifierProgram::Full(_) => Err(ProofVerifyError::BytecodeTypeMismatch(
                "expected Committed, got Full".to_string(),
            )),
        }
    }

    /// Get the program-image words (only in Full mode).
    pub fn program_image_words(&self) -> Option<&[u64]> {
        match self {
            VerifierProgram::Full(p) => Some(&p.program_image_words),
            VerifierProgram::Committed(_) => None,
        }
    }

    /// Get the instructions (only in Full mode).
    pub fn instructions(&self) -> Option<&[Instruction]> {
        match self {
            VerifierProgram::Full(p) => Some(&p.instructions),
            VerifierProgram::Committed(_) => None,
        }
    }

    /// Get the full program preprocessing (only in Full mode).
    pub fn full(&self) -> Option<&Arc<ProgramPreprocessing>> {
        match self {
            VerifierProgram::Full(p) => Some(p),
            VerifierProgram::Committed(_) => None,
        }
    }

    /// Get a BytecodePreprocessing-compatible view (only in Full mode).
    ///
    /// Returns a new BytecodePreprocessing struct for backward compatibility.
    pub fn as_bytecode(&self) -> Option<crate::zkvm::bytecode::BytecodePreprocessing> {
        match self {
            VerifierProgram::Full(p) => Some(p.as_bytecode()),
            VerifierProgram::Committed(_) => None,
        }
    }
}

// Manual serialization for VerifierProgram
impl<PCS: CommitmentScheme> CanonicalSerialize for VerifierProgram<PCS> {
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        match self {
            VerifierProgram::Full(p) => {
                0u8.serialize_with_mode(&mut writer, compress)?;
                p.as_ref().serialize_with_mode(&mut writer, compress)?;
            }
            VerifierProgram::Committed(trusted) => {
                1u8.serialize_with_mode(&mut writer, compress)?;
                trusted.serialize_with_mode(&mut writer, compress)?;
            }
        }
        Ok(())
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        1 + match self {
            VerifierProgram::Full(p) => p.serialized_size(compress),
            VerifierProgram::Committed(trusted) => trusted.serialized_size(compress),
        }
    }
}

impl<PCS: CommitmentScheme> Valid for VerifierProgram<PCS> {
    fn check(&self) -> Result<(), SerializationError> {
        match self {
            VerifierProgram::Full(p) => p.check(),
            VerifierProgram::Committed(trusted) => trusted.check(),
        }
    }
}

impl<PCS: CommitmentScheme> CanonicalDeserialize for VerifierProgram<PCS> {
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let tag = u8::deserialize_with_mode(&mut reader, compress, validate)?;
        match tag {
            0 => {
                let p =
                    ProgramPreprocessing::deserialize_with_mode(&mut reader, compress, validate)?;
                Ok(VerifierProgram::Full(Arc::new(p)))
            }
            1 => {
                let trusted = TrustedProgramCommitments::<PCS>::deserialize_with_mode(
                    &mut reader,
                    compress,
                    validate,
                )?;
                Ok(VerifierProgram::Committed(trusted))
            }
            _ => Err(SerializationError::InvalidData),
        }
    }
}
