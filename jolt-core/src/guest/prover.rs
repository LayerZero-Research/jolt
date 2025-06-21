use crate::jolt::vm::{
    rv32i_vm::{JoltHyperKZGProof, ProofTranscript, RV32IJoltVM, PCS},
    Jolt, JoltProverPreprocessing,
};
use anyhow::{ensure, Result};
use ark_bn254::Fr as F;
use common::jolt_device::{MemoryConfig, MemoryLayout};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracer;
use tracer::instruction::RV32IMInstruction;

use super::{Program, RuntimeConfig};

/// Generate preprocessing data for proving
pub fn preprocess_prove(
    instructions: &[RV32IMInstruction],
    memory_init: &[(u64, u8)],
    memory_config: &MemoryConfig,
) -> JoltProverPreprocessing<F, PCS, ProofTranscript> {
    let memory_layout = MemoryLayout::new(memory_config);
    RV32IJoltVM::prover_preprocess(
        instructions.to_vec(),
        memory_layout,
        memory_init.to_vec(),
        1 << 20,
        1 << 20,
        1 << 24,
    )
}

/// Generate a proof for the given input bytes and write output to the provided buffer
pub fn prove(
    program: &Program,
    input_bytes: &[u8],
    output_bytes: &mut [u8],
    preprocessing: JoltProverPreprocessing<F, PCS, ProofTranscript>,
) -> Result<JoltHyperKZGProof> {
    let runtime_config = RuntimeConfig {
        max_input_size: preprocessing.shared.memory_layout.max_input_size,
        max_output_size: preprocessing.shared.memory_layout.max_output_size,
    };

    // Verify output buffer size
    ensure!(
        output_bytes.len() >= runtime_config.max_output_size as usize,
        "Output buffer too small: need {} bytes but got {} bytes",
        runtime_config.max_output_size,
        output_bytes.len()
    );

    let (io_device, trace) = program.trace(input_bytes, &runtime_config);
    let (jolt_proof, output_io_device, _) = RV32IJoltVM::prove(io_device, trace, preprocessing);

    output_bytes[..output_io_device.outputs.len()].copy_from_slice(&output_io_device.outputs);

    Ok(JoltHyperKZGProof { proof: jolt_proof })
}

/// Build a prover function that captures preprocessing data and handles serialization
pub fn build_prover<T, U>(
    program: Program,
    preprocessing: JoltProverPreprocessing<F, PCS, ProofTranscript>,
) -> impl Fn(T) -> Result<(U, JoltHyperKZGProof)> + Send + Sync
where
    T: Serialize + DeserializeOwned + Send + Sync,
    U: Serialize + DeserializeOwned + Send + Sync,
{
    let program = Arc::new(program);
    let preprocessing = Arc::new(preprocessing);

    move |input| {
        let program = program.clone();
        let input_bytes = postcard::to_stdvec(&input)?;
        let mut output_bytes =
            vec![0u8; preprocessing.shared.memory_layout.max_output_size as usize];
        let proof = prove(
            &program,
            &input_bytes,
            &mut output_bytes,
            (*preprocessing).clone(),
        )?;

        let output = postcard::from_bytes::<U>(&output_bytes)?;
        Ok((output, proof))
    }
}
