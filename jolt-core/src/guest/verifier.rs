#![cfg_attr(not(feature = "std"), no_std)]

use crate::jolt::vm::{
    rv32i_vm::{JoltHyperKZGProof, ProofTranscript, RV32IJoltVM, PCS},
    Jolt, JoltVerifierPreprocessing,
};
use anyhow::{ensure, Result};
use ark_bn254::Fr as F;
use common::jolt_device::{JoltDevice, MemoryConfig, MemoryLayout};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracer::instruction::RV32IMInstruction;

/// Generate preprocessing data for verification
pub fn preprocess(
    instructions: &[RV32IMInstruction],
    memory_init: &[(u64, u8)],
    memory_config: &MemoryConfig,
) -> JoltVerifierPreprocessing<F, PCS, ProofTranscript> {
    let memory_layout = MemoryLayout::new(memory_config);
    
    RV32IJoltVM::verifier_preprocess(
        instructions.to_vec(),
        memory_layout,
        memory_init.to_vec(),
        1 << 20,
        1 << 20,
        1 << 24,
    )
}

/// Verify a proof for the given input and output bytes using provided preprocessing data
pub fn verify(
    input_bytes: &[u8],
    output_bytes: &[u8],
    proof: JoltHyperKZGProof,
    preprocessing: JoltVerifierPreprocessing<F, PCS, ProofTranscript>,
) -> Result<bool> {
    // Verify input and output buffer sizes
    ensure!(
        input_bytes.len() <= preprocessing.memory_layout.max_input_size as usize,
        "Input too large: got {} bytes but max is {} bytes",
        input_bytes.len(),
        preprocessing.memory_layout.max_input_size
    );
    ensure!(
        output_bytes.len() <= preprocessing.memory_layout.max_output_size as usize,
        "Output too large: got {} bytes but max is {} bytes",
        output_bytes.len(),
        preprocessing.memory_layout.max_output_size
    );

    let memory_config = MemoryConfig {
        max_input_size: preprocessing.memory_layout.max_input_size,
        max_output_size: preprocessing.memory_layout.max_output_size,
        ..Default::default()
    };

    let mut io_device = JoltDevice::new(&memory_config);
    io_device.inputs.extend_from_slice(input_bytes);
    io_device.outputs.extend_from_slice(output_bytes);

    Ok(RV32IJoltVM::verify(preprocessing, proof.proof, io_device, None).is_ok())
}

/// Build a verifier function that captures preprocessing data and handles serialization
pub fn build_verifier<T, U>(
    preprocessing: JoltVerifierPreprocessing<F, PCS, ProofTranscript>,
) -> impl Fn(T, U, JoltHyperKZGProof) -> Result<bool> + Send + Sync
where
    T: Serialize + DeserializeOwned + Send + Sync,
    U: Serialize + DeserializeOwned + Send + Sync,
{
    let preprocessing = Arc::new(preprocessing);

    move |input, output, proof| {
        let input_bytes = postcard::to_stdvec(&input)?;
        ensure!(
            input_bytes.len() <= preprocessing.memory_layout.max_input_size as usize,
            "Input size exceeds maximum allowed size"
        );

        let output_bytes = postcard::to_stdvec(&output)?;
        ensure!(
            output_bytes.len() <= preprocessing.memory_layout.max_output_size as usize,
            "Output size exceeds maximum allowed size"
        );

        verify(&input_bytes, &output_bytes, proof, (*preprocessing).clone())
    }
}
