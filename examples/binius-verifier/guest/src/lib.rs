use binius_core::constraint_system::{ConstraintSystem, Proof, ValuesData};
use binius_utils::serialization::DeserializeBytes;
use binius_verifier::{
    config::{ChallengerWithName, StdChallenger},
    hash::{StdCompression, StdDigest},
    transcript::VerifierTranscript,
    Verifier,
};
use jolt_sdk::{self as jolt};

use jolt::{end_cycle_tracking, start_cycle_tracking};

include!("./provable_macro.rs");

const STATUS_OK: u32 = 1;
const STATUS_LOG_INV_RATE_DESERIALIZE_ERROR: u32 = 10;
const STATUS_CONSTRAINT_SYSTEM_DESERIALIZE_ERROR: u32 = 11;
const STATUS_PUBLIC_VALUES_DESERIALIZE_ERROR: u32 = 12;
const STATUS_PROOF_DESERIALIZE_ERROR: u32 = 13;
const STATUS_TRAILING_INPUT: u32 = 20;
const STATUS_CHALLENGER_MISMATCH: u32 = 21;
const STATUS_SETUP_ERROR: u32 = 30;
const STATUS_VERIFY_ERROR: u32 = 40;

provable_with_config! {
fn verify_binius_proof(bytes: &[u8]) -> u32 {
    let mut read_buf = bytes;

    start_cycle_tracking("deserialize");
    let log_inv_rate = match u32::deserialize(&mut read_buf) {
        Ok(log_inv_rate) => log_inv_rate,
        Err(_) => {
            end_cycle_tracking("deserialize");
            return STATUS_LOG_INV_RATE_DESERIALIZE_ERROR;
        }
    };
    let constraint_system = match ConstraintSystem::deserialize(&mut read_buf) {
        Ok(constraint_system) => constraint_system,
        Err(_) => {
            end_cycle_tracking("deserialize");
            return STATUS_CONSTRAINT_SYSTEM_DESERIALIZE_ERROR;
        }
    };
    let public = match ValuesData::deserialize(&mut read_buf) {
        Ok(public) => public,
        Err(_) => {
            end_cycle_tracking("deserialize");
            return STATUS_PUBLIC_VALUES_DESERIALIZE_ERROR;
        }
    };
    let proof = match Proof::deserialize(&mut read_buf) {
        Ok(proof) => proof,
        Err(_) => {
            end_cycle_tracking("deserialize");
            return STATUS_PROOF_DESERIALIZE_ERROR;
        }
    };
    end_cycle_tracking("deserialize");

    if !read_buf.is_empty() {
        return STATUS_TRAILING_INPUT;
    }

    if proof.challenger_type() != StdChallenger::NAME {
        return STATUS_CHALLENGER_MISMATCH;
    }

    start_cycle_tracking("setup");
    let verifier: Verifier<StdDigest, StdCompression> = match Verifier::setup(
        constraint_system,
        log_inv_rate as usize,
        StdCompression::default(),
    ) {
        Ok(verifier) => verifier,
        Err(_) => {
            end_cycle_tracking("setup");
            return STATUS_SETUP_ERROR;
        }
    };
    end_cycle_tracking("setup");

    let (proof_data, _) = proof.into_owned();

    start_cycle_tracking("verify");
    let mut verifier_transcript = VerifierTranscript::new(StdChallenger::default(), proof_data);
    let is_valid = verifier
        .verify(public.as_slice(), &mut verifier_transcript)
        .is_ok()
        && verifier_transcript.finalize().is_ok();
    end_cycle_tracking("verify");

    if is_valid {
        STATUS_OK
    } else {
        STATUS_VERIFY_ERROR
    }
}
}
