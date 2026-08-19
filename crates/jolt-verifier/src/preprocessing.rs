//! Verifier preprocessing inputs.

use common::jolt_device::MemoryLayout;
use jolt_claims::protocols::jolt::JoltRelationId;
use jolt_crypto::VectorCommitment;
use jolt_openings::CommitmentScheme;
use jolt_program::preprocess::{JoltProgramPreprocessing, ProgramMetadata};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::VerifierError;

/// Committed-program verifier inputs: trusted bytecode-chunk and program-image
/// commitments plus the program metadata they bind to. Mirrors `jolt-prover-legacy`'s
/// `CommittedProgramPreprocessing`; the chunk count is implied by
/// `bytecode_chunk_commitments.len()`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "PCS::Output: Serialize",
    deserialize = "PCS::Output: serde::de::DeserializeOwned"
))]
pub struct CommittedProgramPreprocessing<PCS: CommitmentScheme> {
    pub meta: ProgramMetadata,
    pub memory_layout: MemoryLayout,
    pub max_padded_trace_length: usize,
    #[cfg(not(feature = "akita"))]
    pub bytecode_chunk_commitments: Vec<PCS::Output>,
    #[cfg(not(feature = "akita"))]
    pub program_image_commitment: PCS::Output,
    /// Fixed-prefix program objects in canonical order: bytecode, then the
    /// independently pointed program-image bytes.
    #[cfg(feature = "akita")]
    pub program_one_hot_commitments: Vec<PCS::Output>,
    #[cfg(feature = "akita")]
    pub bytecode_chunk_count: usize,
}

impl<PCS: CommitmentScheme> CommittedProgramPreprocessing<PCS> {
    pub fn bytecode_chunk_count(&self) -> usize {
        #[cfg(not(feature = "akita"))]
        {
            self.bytecode_chunk_commitments.len()
        }
        #[cfg(feature = "akita")]
        {
            self.bytecode_chunk_count
        }
    }
}

/// Program preprocessing in one of two modes, detected at runtime from the
/// deserialized preprocessing exactly like `jolt-prover-legacy`'s
/// `ProgramPreprocessing`: `Full` carries the bytecode table and initial RAM
/// image, `Committed` replaces them with trusted commitments plus metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "PCS::Output: Serialize",
    deserialize = "PCS::Output: serde::de::DeserializeOwned"
))]
#[expect(
    clippy::large_enum_variant,
    reason = "constructed once per preprocessing; boxing Committed buys nothing"
)]
pub enum ProgramPreprocessing<PCS: CommitmentScheme> {
    /// `Arc` so witness backends take an owning handle without deep-cloning
    /// the program-sized tables (serde `rc`: serializes as the contents).
    Full(Arc<JoltProgramPreprocessing>),
    Committed(CommittedProgramPreprocessing<PCS>),
}

impl<PCS: CommitmentScheme> ProgramPreprocessing<PCS> {
    pub fn as_full(&self) -> Option<&JoltProgramPreprocessing> {
        match self {
            Self::Full(full) => Some(full),
            Self::Committed(_) => None,
        }
    }

    /// The owning counterpart of [`as_full`](Self::as_full) — a refcount
    /// bump, never a copy.
    pub fn as_full_arc(&self) -> Option<Arc<JoltProgramPreprocessing>> {
        match self {
            Self::Full(full) => Some(Arc::clone(full)),
            Self::Committed(_) => None,
        }
    }

    pub fn committed(&self) -> Option<&CommittedProgramPreprocessing<PCS>> {
        match self {
            Self::Full(_) => None,
            Self::Committed(committed) => Some(committed),
        }
    }

    pub fn memory_layout(&self) -> &MemoryLayout {
        match self {
            Self::Full(full) => &full.memory_layout,
            Self::Committed(committed) => &committed.memory_layout,
        }
    }

    pub fn max_padded_trace_length(&self) -> usize {
        match self {
            Self::Full(full) => full.max_padded_trace_length,
            Self::Committed(committed) => committed.max_padded_trace_length,
        }
    }

    pub fn entry_address(&self) -> u64 {
        match self {
            Self::Full(full) => full.bytecode.entry_address,
            Self::Committed(committed) => committed.meta.entry_address,
        }
    }

    pub fn entry_bytecode_index(&self) -> Option<usize> {
        match self {
            Self::Full(full) => full.bytecode.entry_bytecode_index(),
            Self::Committed(committed) => Some(committed.meta.entry_bytecode_index),
        }
    }

    /// [`entry_bytecode_index`](Self::entry_bytecode_index), attributing an
    /// entry address absent from the bytecode to the consuming `stage`.
    pub fn entry_bytecode_index_checked(
        &self,
        stage: JoltRelationId,
    ) -> Result<usize, VerifierError> {
        self.entry_bytecode_index()
            .ok_or_else(|| VerifierError::StageClaimPublicInputFailed {
                stage,
                reason: "entry address was not found in bytecode preprocessing".to_string(),
            })
    }

    pub fn bytecode_len(&self) -> usize {
        match self {
            Self::Full(full) => full.bytecode.code_size,
            Self::Committed(committed) => committed.meta.bytecode_len,
        }
    }

    pub fn min_bytecode_address(&self) -> u64 {
        match self {
            Self::Full(full) => full.ram.min_bytecode_address,
            Self::Committed(committed) => committed.meta.min_bytecode_address,
        }
    }

    pub fn program_image_len_words(&self) -> usize {
        match self {
            Self::Full(full) => full.ram.bytecode_words.len(),
            Self::Committed(committed) => committed.meta.program_image_len_words,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "ProgramPreprocessing<PCS>: Serialize, PCS::VerifierSetup: Serialize, VC::Setup: Serialize",
    deserialize = "ProgramPreprocessing<PCS>: serde::de::DeserializeOwned, PCS::VerifierSetup: serde::de::DeserializeOwned, VC::Setup: serde::de::DeserializeOwned"
))]
pub struct JoltVerifierPreprocessing<PCS, VC>
where
    PCS: CommitmentScheme,
    VC: VectorCommitment<Field = PCS::Field>,
{
    pub program: ProgramPreprocessing<PCS>,
    pub preprocessing_digest: [u8; 32],
    /// The main PCS setup: every per-polynomial opening on the homomorphic
    /// build, the `OneHotTrace` object on the `akita` build (whose remaining
    /// objects carry their own shape-exact setups below).
    pub pcs_setup: PCS::VerifierSetup,
    pub vc_setup: Option<VC::Setup>,
    #[cfg(feature = "akita")]
    pub untrusted_advice_setup: Option<PCS::VerifierSetup>,
    #[cfg(feature = "akita")]
    pub trusted_advice_setup: Option<PCS::VerifierSetup>,
    /// Committed-program mode: setups matching `program_one_hot_commitments`.
    #[cfg(feature = "akita")]
    pub program_one_hot_setups: Vec<PCS::VerifierSetup>,
    /// The grouped schedule rows batching this program's advice objects
    /// with the packed trace: one per reachable presence combination per
    /// reachable trace arity.
    ///
    /// A grouped row is keyed on the frozen advice prefix profiles, which follow
    /// `memory_layout.max_{un,}trusted_advice_size`, so these rows cannot be
    /// emitted offline and are planned here at preprocessing instead. They are
    /// a pure function of those public capacities, so they are not serialized:
    /// [`Self::provision_akita_schedules`] rebuilds them on the deserialized
    /// side.
    #[cfg(feature = "akita")]
    #[serde(skip)]
    pub advice_schedules: jolt_akita::schedule_registry::RegisteredRows,
}

impl<PCS, VC> JoltVerifierPreprocessing<PCS, VC>
where
    PCS: CommitmentScheme,
    VC: VectorCommitment<Field = PCS::Field>,
{
    pub fn new(
        program: ProgramPreprocessing<PCS>,
        preprocessing_digest: [u8; 32],
        pcs_setup: PCS::VerifierSetup,
        vc_setup: Option<VC::Setup>,
    ) -> Self {
        Self {
            program,
            preprocessing_digest,
            pcs_setup,
            vc_setup,
            #[cfg(feature = "akita")]
            untrusted_advice_setup: None,
            #[cfg(feature = "akita")]
            trusted_advice_setup: None,
            #[cfg(feature = "akita")]
            program_one_hot_setups: Vec::new(),
            #[cfg(feature = "akita")]
            advice_schedules: Default::default(),
        }
    }

    /// Plan and take ownership of this program's grouped advice
    /// schedule rows, and publish them where the commitment config's resolution
    /// hooks can see them.
    ///
    /// Idempotent: the rows follow the public advice capacity, so re-running it
    /// — after a serde roundtrip, or on a verifier that never saw the prover —
    /// reproduces the same set. Returns the row-set digest, which prover and
    /// verifier can compare to confirm they provisioned identically.
    ///
    /// Must run before the packed setup is sized: setup capacity folds these
    /// rows into the matrix footprint.
    #[cfg(feature = "akita")]
    pub fn provision_akita_schedules(
        &mut self,
        untrusted_advice_physical_vars: Option<usize>,
        trusted_advice_physical_vars: Option<usize>,
        one_hot_k: usize,
        max_final_num_vars: usize,
    ) -> Result<[u8; 32], jolt_akita::AkitaError> {
        if untrusted_advice_physical_vars.is_none() && trusted_advice_physical_vars.is_none() {
            self.advice_schedules = Default::default();
            return Ok(self.advice_schedules.set_digest());
        }
        self.advice_schedules = jolt_akita::schedule_registry::provision_advice_for_k(
            untrusted_advice_physical_vars,
            trusted_advice_physical_vars,
            one_hot_k,
            max_final_num_vars,
        )?;
        Ok(self.advice_schedules.set_digest())
    }
}
