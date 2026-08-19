//! Verifier-selected protocol configuration.
//!
//! Both protocol axes are fixed at compile time — the `zk` feature selects
//! BlindFold, the `akita` feature selects the packed commitment mode — so one
//! compiled verifier runs exactly one protocol. A proof self-describes its
//! axes and [`validate_proof_config`] rejects a mismatch fail-closed.

use serde::{Deserialize, Serialize};

use crate::VerifierError;

#[cfg(all(feature = "zk", feature = "akita"))]
compile_error!(
    "the `zk` and `akita` features are mutually exclusive: no zk protocol exists over the \
     packed commitment axis (a lattice-friendly hiding commitment is a future workstream)"
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZkConfig {
    Transparent,
    BlindFold,
}

/// The commitment axis of the protocol: how committed polynomials are
/// discharged at the final opening.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitmentConfig {
    /// Per-polynomial commitments, RLC batch opening (requires additive
    /// homomorphism).
    Homomorphic,
    /// Legacy packed one-hot advice protocol. Retained as a wire tombstone.
    Packed,
    /// Packed trace/program objects with direct advice-word commitments.
    PackedDenseAdvice,
    /// Packed dense advice with Akita precommitted-group batching for the
    /// trusted-advice and final `OneHotTrace` openings. Retained as a wire
    /// tombstone: superseded by [`Self::PackedAllAdviceBatched`].
    PackedDenseAdviceBatched,
    /// Packed advice with Akita precommitted-group batching over the full
    /// canonical group order `[UntrustedAdvice, TrustedAdvice, OneHotTrace]`, so
    /// both advice objects and the trace share one joint opening proof.
    PackedAllAdviceBatched,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoltProtocolConfig {
    pub zk: ZkConfig,
    pub commitment: CommitmentConfig,
}

impl JoltProtocolConfig {
    pub const fn for_zk(zk: bool) -> Self {
        Self {
            zk: if zk {
                ZkConfig::BlindFold
            } else {
                ZkConfig::Transparent
            },
            commitment: SELECTED_COMMITMENT_CONFIG,
        }
    }
}

#[cfg(feature = "zk")]
pub const SELECTED_ZK_CONFIG: ZkConfig = ZkConfig::BlindFold;

#[cfg(not(feature = "zk"))]
pub const SELECTED_ZK_CONFIG: ZkConfig = ZkConfig::Transparent;

#[cfg(feature = "akita")]
pub const SELECTED_COMMITMENT_CONFIG: CommitmentConfig = CommitmentConfig::PackedAllAdviceBatched;

pub const PACKED_DENSE_ADVICE_TRANSCRIPT_VERSION: u64 = 1;
pub const PACKED_DENSE_ADVICE_ENCODING: u64 = 1;
pub const PACKED_DENSE_ADVICE_BATCHED_TRANSCRIPT_VERSION: u64 = 2;
pub const PACKED_DENSE_ADVICE_BATCHED_ENCODING: u64 = 2;
pub const PACKED_ALL_ADVICE_BATCHED_TRANSCRIPT_VERSION: u64 = 3;
pub const PACKED_ALL_ADVICE_BATCHED_ENCODING: u64 = 3;

#[cfg(not(feature = "akita"))]
pub const SELECTED_COMMITMENT_CONFIG: CommitmentConfig = CommitmentConfig::Homomorphic;

/// The one protocol this build verifies.
pub const JOLT_VERIFIER_CONFIG: JoltProtocolConfig = JoltProtocolConfig {
    zk: SELECTED_ZK_CONFIG,
    commitment: SELECTED_COMMITMENT_CONFIG,
};

pub fn validate_proof_config(
    config: &JoltProtocolConfig,
    protocol: JoltProtocolConfig,
) -> Result<(), VerifierError> {
    if protocol != *config {
        return Err(VerifierError::ProtocolConfigMismatch {
            expected: *config,
            got: protocol,
        });
    }

    Ok(())
}
