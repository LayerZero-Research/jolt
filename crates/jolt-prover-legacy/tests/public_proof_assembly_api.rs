//! Compile-only coverage for the public external prover-to-verifier assembly seam.

#![cfg(feature = "prover")]
#![allow(unused_imports)]

use jolt_prover_legacy::zkvm::proof::{convert_sumcheck, convert_uniskip};
use jolt_prover_legacy::zkvm::proof_parts::JoltProofParts;

#[cfg(not(feature = "zk"))]
use jolt_prover_legacy::zkvm::{proof::convert_opening_id, proof_parts::ProverOpeningClaims};

#[cfg(all(not(feature = "akita"), not(feature = "zk")))]
use jolt_prover_legacy::zkvm::{
    clear_claims::build_clear_claims, proof::proof_parts_into_verifier,
};

#[cfg(feature = "zk")]
use jolt_prover_legacy::zkvm::proof::proof_parts_into_verifier;

#[cfg(all(feature = "akita", not(feature = "zk")))]
use jolt_prover_legacy::zkvm::clear_claims::build_packed_clear_claims;

#[test]
fn public_proof_assembly_paths_are_importable() {}
