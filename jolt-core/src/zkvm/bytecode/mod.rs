use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use tracer::instruction::Cycle;

use crate::poly::commitment::commitment_scheme::CommitmentScheme;
use crate::poly::commitment::dory::{DoryContext, DoryGlobals};
use crate::utils::math::Math;
use crate::zkvm::bytecode::chunks::{
    build_committed_bytecode_chunk_polynomials, committed_bytecode_chunk_cycle_len, committed_lanes,
};

pub mod chunks;
pub mod read_raf_checking;

pub use jolt_program::preprocess::{BytecodePCMapper, BytecodePreprocessing, PreprocessingError};

#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct TrustedBytecodeCommitments<PCS: CommitmentScheme> {
    pub commitments: Vec<PCS::Commitment>,
    pub num_columns: usize,
    pub log_k_chunk: u8,
    pub bytecode_chunk_count: usize,
    pub bytecode_len: usize,
    pub bytecode_T: usize,
}

#[derive(Clone, Debug)]
pub struct TrustedBytecodeHints<PCS: CommitmentScheme> {
    pub hints: Vec<PCS::OpeningProofHint>,
}

// NOTE: `OpeningProofHint` is not bounded by `CanonicalSerialize` on the
// `CommitmentScheme` trait (Akita's hint wraps non-serializable external ring
// types), so these impls are written manually with a conditional bound. They are
// only instantiated for committed program mode, which is Dory-only.
impl<PCS: CommitmentScheme> CanonicalSerialize for TrustedBytecodeHints<PCS>
where
    PCS::OpeningProofHint: CanonicalSerialize,
{
    fn serialize_with_mode<W: std::io::Write>(
        &self,
        writer: W,
        compress: ark_serialize::Compress,
    ) -> Result<(), ark_serialize::SerializationError> {
        self.hints.serialize_with_mode(writer, compress)
    }

    fn serialized_size(&self, compress: ark_serialize::Compress) -> usize {
        self.hints.serialized_size(compress)
    }
}

impl<PCS: CommitmentScheme> ark_serialize::Valid for TrustedBytecodeHints<PCS>
where
    PCS::OpeningProofHint: ark_serialize::Valid,
{
    fn check(&self) -> Result<(), ark_serialize::SerializationError> {
        self.hints.check()
    }
}

impl<PCS: CommitmentScheme> CanonicalDeserialize for TrustedBytecodeHints<PCS>
where
    PCS::OpeningProofHint: CanonicalDeserialize,
{
    fn deserialize_with_mode<R: std::io::Read>(
        reader: R,
        compress: ark_serialize::Compress,
        validate: ark_serialize::Validate,
    ) -> Result<Self, ark_serialize::SerializationError> {
        Ok(Self {
            hints: Vec::<PCS::OpeningProofHint>::deserialize_with_mode(reader, compress, validate)?,
        })
    }
}

impl<PCS: CommitmentScheme> TrustedBytecodeCommitments<PCS> {
    #[tracing::instrument(skip_all, name = "TrustedBytecodeCommitments::derive")]
    pub fn derive(
        bytecode: &BytecodePreprocessing,
        generators: &PCS::ProverSetup,
        log_k_chunk: usize,
        bytecode_chunk_count: usize,
    ) -> (Self, TrustedBytecodeHints<PCS>) {
        let bytecode_len = bytecode.code_size;
        let bytecode_T = committed_bytecode_chunk_cycle_len(bytecode_len, bytecode_chunk_count);

        let total_vars = bytecode_T.log_2() + committed_lanes().log_2();
        let (bytecode_sigma, _) = DoryGlobals::balanced_sigma_nu(total_vars);
        let num_columns = 1usize << bytecode_sigma;

        let bytecode_chunk_polys = build_committed_bytecode_chunk_polynomials::<PCS::Field>(
            &bytecode.bytecode,
            bytecode_chunk_count,
        );
        let _bytecode_guard = DoryGlobals::initialize_context(
            committed_lanes(),
            bytecode_T,
            DoryContext::UntrustedAdvice,
            None,
        );
        let _ctx = DoryGlobals::with_context(DoryContext::UntrustedAdvice);
        let (commitments, hints): (Vec<_>, Vec<_>) = bytecode_chunk_polys
            .iter()
            .map(|poly| PCS::default().commit(poly, generators))
            .unzip();

        (
            Self {
                commitments,
                num_columns,
                log_k_chunk: log_k_chunk as u8,
                bytecode_chunk_count,
                bytecode_len,
                bytecode_T,
            },
            TrustedBytecodeHints { hints },
        )
    }
}

pub fn get_pc_for_cycle(bytecode: &BytecodePreprocessing, cycle: &Cycle) -> usize {
    if matches!(cycle, Cycle::NoOp) {
        return 0;
    }
    let instruction = cycle
        .instruction()
        .try_jolt_instruction_row()
        .expect("trace cycle must be a final Jolt instruction row");
    match bytecode.get_pc(&instruction) {
        Some(pc) => pc,
        None => panic!(
            "bytecode preprocessing is missing PC mapping for instruction at address {:#x} with virtual_sequence_remaining {:?}",
            instruction.address, instruction.virtual_sequence_remaining
        ),
    }
}

pub fn entry_bytecode_index(bytecode: &BytecodePreprocessing) -> usize {
    match bytecode.entry_bytecode_index() {
        Some(pc) => pc,
        None => panic!(
            "bytecode preprocessing is missing entry mapping for address {:#x}",
            bytecode.entry_address
        ),
    }
}
