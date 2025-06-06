#![cfg_attr(not(feature = "host"), no_std)]

extern crate jolt_sdk_macros;

pub use jolt_sdk_macros::provable;
pub use postcard;

pub use ark_bn254::{Fr as F, G1Projective as G};
pub use ark_ec::CurveGroup;
pub use jolt_core::{field::JoltField, poly::commitment::hyperkzg::HyperKZG};

pub use common::jolt_device::{MemoryConfig, MemoryLayout};
pub use jolt_core::jolt::lookup_table;
pub use jolt_core::jolt::vm::{
    rv32i_vm::{
        JoltHyperKZGProof, ProofTranscript, RV32IJoltProof, RV32IJoltVM, Serializable, PCS,
    },
    Jolt, JoltProof, JoltProverPreprocessing, JoltVerifierPreprocessing,
};
pub use tracer;

#[cfg(feature = "host")]
pub mod host_utils;
#[cfg(feature = "host")]
pub use host_utils::*;

pub mod cycle_tracking;
pub use cycle_tracking::*;

pub mod alloc;
pub use alloc::*;

// This is a dummy _HEAP_PTR to keep the compiler happy.
// It should never be used when compiled as a guest or with
// our custom allocator
#[no_mangle]
#[cfg(feature = "host")]
pub static mut _HEAP_PTR: u8 = 0;
