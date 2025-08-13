use super::program::Program;
use crate::field::JoltField;
use crate::poly::commitment::commitment_scheme::CommitmentScheme;
use crate::poly::commitment::dory::DoryCommitmentScheme;
use crate::utils::transcript::Transcript;
use crate::zkvm::dag::proof_serialization::JoltProof;
use crate::zkvm::{Jolt, JoltProverPreprocessing, JoltRV32IM, ProverDebugInfo};
use common::jolt_device::MemoryLayout;
use tracer::JoltDevice;

pub fn preprocess(
    guest: &Program,
    max_trace_length: usize,
) -> JoltProverPreprocessing<ark_bn254::Fr, DoryCommitmentScheme> {
    let mut memory_config = guest.memory_config;
    let memory_layout = MemoryLayout::new(&memory_config);
    let (bytecode, memory_init, program_size) = guest.decode();
    memory_config.program_size = Some(program_size);

    JoltRV32IM::prover_preprocess(bytecode, memory_layout, memory_init, max_trace_length)
}

pub fn prove<F, PCS, FS>(
    guest: &Program,
    inputs_bytes: &[u8],
    output_bytes: &mut [u8],
    preprocessing: &JoltProverPreprocessing<F, PCS>,
) -> (
    JoltProof<F, PCS, FS>,
    JoltDevice,
    Option<ProverDebugInfo<F, FS, PCS>>,
)
where
    F: JoltField,
    PCS: CommitmentScheme<Field = F>,
    FS: Transcript,
    JoltRV32IM: Jolt<F, PCS, FS>,
{
    let (proof, io_device, debug_info) =
        JoltRV32IM::prove(preprocessing, &guest.elf_contents, inputs_bytes);
    output_bytes[..io_device.outputs.len()].copy_from_slice(&io_device.outputs);
    (proof, io_device, debug_info)
}
