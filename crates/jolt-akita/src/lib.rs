//! Akita PCS adapter for Jolt.
//!
//! Wraps the upstream Akita PCS over its fp128 field using Jolt's
//! `CommitmentScheme` trait plus an adapter delegating same-point batches to
//! Akita's native batched opening protocol, and a fused multi-group root fold
//! ([`multi_group_prove_one_hot`]) that discharges every committed group of
//! Jolt's stage-8 opening — the trace plus its precommitted objects — in one
//! native `batched_prove` at slices of a single shared point.

mod adapters;
pub mod configs;
mod multi_group;
mod native_batching;
pub mod schedules;
mod scheme;
mod shape_guard;

pub use akita_types::PolynomialGroupLayout;
pub use jolt_openings::{fused_stage8_open_eligible, AKITA_FUSED_LOG_K_CHUNK};
pub use multi_group::{
    commit_final_one_hot_group, commit_precommitted_one_hot_group, multi_group_prove_one_hot,
    MultiGroupProverGroup, MultiGroupVerifierGroup,
};

pub use adapters::{
    AkitaBackendFlavor, AkitaBatchProof, AkitaCommitment, AkitaField, AkitaHidingCommitment,
    AkitaProverHint, AkitaProverSetup, AkitaSetupParams, AkitaVerifierSetup, AKITA_ONE_HOT_K16,
    AKITA_ONE_HOT_K256,
};
pub use native_batching::{
    AkitaNativeBatchPolynomials, AkitaNativeBatchStatement, AkitaNativeBatching,
};
pub use scheme::AkitaScheme;

/// Jolt↔Akita basis-order bridging, exposed so benchmarks measuring the raw
/// backend use the exact transform the adapter uses.
#[doc(hidden)]
pub use adapters::{jolt_to_akita_evals, reverse_point};
