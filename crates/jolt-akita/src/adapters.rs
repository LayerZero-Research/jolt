use std::{collections::BTreeSet, fmt, io::Cursor, sync::Arc, sync::OnceLock};

use akita_config::CommitmentConfig;
use akita_pcs::{AkitaCommitmentScheme, AkitaDeserialize, AkitaSerialize, AkitaTranscript};
use akita_prover::{CpuBackend, CpuPreparedSetup, DensePoly, OneHotPoly, SparseRingPoly};
use akita_types::{
    AkitaBatchedProof as AkitaBackendBatchProof, AkitaBatchedProofShape,
    AkitaCommitmentHint as AkitaBackendCommitmentHint,
    AkitaVerifierSetup as AkitaBackendVerifierSetup, Commitment as AkitaBackendRingCommitment,
};
use jolt_field::{CanonicalBytes, FixedByteSize};
use jolt_openings::{OpeningsError, VerifierOpeningClaim};
use jolt_poly::{MultilinearPoly, OneHotIndexOrder, Polynomial};
use jolt_transcript::{AppendToTranscript, Label, LabelWithCount, Transcript, U64Word};
use serde::{
    de::{Error as DeError, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use tracing::info_span;

pub type AkitaField = akita_config::proof_optimized::fp128::Field;
pub(crate) type AkitaConfig = akita_config::proof_optimized::fp128::D64Dense;
pub(crate) type AkitaOneHotK16Config = crate::configs::JoltD64OneHotK16;
pub(crate) type AkitaOneHotK256Config = crate::configs::JoltD64OneHotK256;
pub(crate) const AKITA_D: usize = AkitaConfig::D;
pub const AKITA_ONE_HOT_K16: usize = 16;
pub const AKITA_ONE_HOT_K256: usize = 256;

/// Akita's transcript sponge panics on session labels longer than this, and
/// [`bridge_jolt_statement_challenge`] appends [`BRIDGE_BYTES`] to every label.
pub(crate) const MAX_AKITA_SESSION_LABEL_BYTES: usize = 64;
pub(crate) const BRIDGE_BYTES: usize = <AkitaField as FixedByteSize>::NUM_BYTES;

/// Serialized proof-shape blob cap. Honest shapes are a few hundred bytes (a
/// handful of fold levels, each a few dozen words); this leaves two orders of
/// magnitude of margin while keeping worst-case shape-blob deserialization
/// allocations trivial.
pub(crate) const MAX_PROOF_SHAPE_BYTES: usize = 16 * 1024;
/// The bridge is one serialized field element ([`BRIDGE_BYTES`]); the cap is
/// two orders of magnitude above that.
const MAX_STATEMENT_BRIDGE_BYTES: usize = 1024;
/// Covers both the serialized backend commitment and the batched proof body.
/// Trace-scale one-hot proofs run to a few MiB, so this is roughly an order of
/// magnitude of headroom while still bounding a forged length.
const MAX_BACKEND_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;
/// Holds one serialized evaluation ([`BRIDGE_BYTES`] again); same two orders of
/// magnitude of margin as the statement bridge.
const MAX_HIDING_COMMITMENT_BYTES: usize = 1024;

pub(crate) type AkitaBackendExtField = <AkitaConfig as CommitmentConfig>::ExtField;

pub(crate) type AkitaBackendScheme = AkitaCommitmentScheme<AkitaConfig>;
pub(crate) type AkitaOneHotK16BackendScheme = AkitaCommitmentScheme<AkitaOneHotK16Config>;
pub(crate) type AkitaOneHotK256BackendScheme = AkitaCommitmentScheme<AkitaOneHotK256Config>;
pub(crate) type AkitaBackendCommitment = AkitaBackendRingCommitment<AkitaField>;
pub(crate) type AkitaBackendHint = AkitaBackendCommitmentHint<AkitaField>;
pub(crate) type AkitaBackendProof = AkitaBackendBatchProof<AkitaField, AkitaBackendExtField>;
pub(crate) type AkitaBackendProofShape = AkitaBatchedProofShape;
pub(crate) type AkitaBackendVerifier = AkitaBackendVerifierSetup<AkitaField>;
pub(crate) type AkitaBackendDensePoly = DensePoly<AkitaField>;
pub(crate) type AkitaBackendOneHotPoly = OneHotPoly<AkitaField, u8>;
pub(crate) type AkitaBackendSparsePoly = SparseRingPoly<AkitaField>;
pub(crate) type AkitaBackendPreparedSetup = CpuPreparedSetup<AkitaField>;
pub(crate) type AkitaBackendProverSetup = akita_prover::AkitaProverSetup<AkitaField>;
pub(crate) type BackendStack<'a> = akita_prover::UniformProverStack<'a, AkitaField, CpuBackend>;

pub(crate) type AkitaLayoutDigest = [u8; 32];

/// Worker stack size for [`with_backend_pool`]. Stacks are lazily committed,
/// so oversizing costs virtual address space only.
const BACKEND_WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

/// Pre-allocation ceiling for length-prefixed byte fields, mirroring serde's own
/// `size_hint::cautious` policy: a decode that will fail must not first reserve
/// the protocol cap.
const PREALLOC_FLOOR_BYTES: usize = 4096;

fn deserialize_bounded_bytes<'de, D, const MAX: usize>(
    deserializer: D,
    field: &'static str,
) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoundedBytesVisitor<const MAX: usize> {
        field: &'static str,
    }

    impl<'de, const MAX: usize> Visitor<'de> for BoundedBytesVisitor<MAX> {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{} containing at most {MAX} bytes", self.field)
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let declared_len = sequence.size_hint();
            if declared_len.is_some_and(|len| len > MAX) {
                return Err(A::Error::custom(format!(
                    "{} declares {} bytes but the protocol cap is {MAX}",
                    self.field,
                    declared_len.unwrap_or_default()
                )));
            }
            // WARNING: `size_hint` is the attacker's declared length. Reserving it
            // outright would let a 5-byte input reserve the whole cap, so grow from
            // a fixed floor instead and let the push loop enforce `MAX`.
            let mut bytes =
                Vec::with_capacity(declared_len.unwrap_or_default().min(PREALLOC_FLOOR_BYTES));
            while let Some(byte) = sequence.next_element()? {
                if bytes.len() == MAX {
                    return Err(A::Error::custom(format!(
                        "{} exceeds the protocol cap of {MAX} bytes",
                        self.field
                    )));
                }
                bytes.push(byte);
            }
            Ok(bytes)
        }
    }

    deserializer.deserialize_seq(BoundedBytesVisitor::<MAX> { field })
}

fn deserialize_statement_bridge<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_bounded_bytes::<D, MAX_STATEMENT_BRIDGE_BYTES>(
        deserializer,
        "Akita statement bridge",
    )
}

fn deserialize_proof_shape<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_bounded_bytes::<D, MAX_PROOF_SHAPE_BYTES>(deserializer, "Akita proof shape")
}

fn deserialize_backend_payload<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_bounded_bytes::<D, MAX_BACKEND_PAYLOAD_BYTES>(deserializer, "Akita backend payload")
}

fn deserialize_hiding_commitment<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_bounded_bytes::<D, MAX_HIDING_COMMITMENT_BYTES>(
        deserializer,
        "Akita hiding commitment",
    )
}

/// Runs `f` with rayon parallelism on a dedicated pool whose workers have
/// large stacks.
///
/// The Akita backend kernels recurse deeply inside rayon parallel iterators
/// (the bridge splitter re-splits whenever a job migrates to a stealing
/// worker, and the fold kernels carry large frames), which overflows rayon's
/// default 2 MiB worker stacks nondeterministically — observed as SIGABRT in
/// the packed prover at trace-scale shapes. Every backend setup/commit/
/// prove/verify entry funnels through this pool. Nested calls reuse it.
#[expect(
    clippy::expect_used,
    reason = "a pool that cannot spawn threads is an unrecoverable environment failure"
)]
pub(crate) fn with_backend_pool<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    static POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();
    POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .thread_name(|index| format!("jolt-akita-{index}"))
            .stack_size(BACKEND_WORKER_STACK_BYTES)
            .build()
            .expect("the Akita backend thread pool must build")
    })
    .install(f)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AkitaSetupParams {
    pub(crate) max_num_vars: usize,
    pub(crate) max_num_polys_per_commitment_group: usize,
    pub(crate) default_layout_digest: AkitaLayoutDigest,
    pub(crate) one_hot_k: usize,
    /// When set, only the one-hot flavor's backend setup is built — the
    /// dense-flavor setup for the same shape is large and slow, and a packed
    /// one-hot commitment object never touches it.
    pub(crate) one_hot_only: bool,
}

impl AkitaSetupParams {
    pub fn new(
        max_num_vars: usize,
        max_num_polys_per_commitment_group: usize,
        default_layout_digest: AkitaLayoutDigest,
    ) -> Self {
        Self {
            max_num_vars,
            max_num_polys_per_commitment_group,
            default_layout_digest,
            one_hot_k: AKITA_ONE_HOT_K256,
            one_hot_only: false,
        }
    }

    /// Setup parameters for a commitment object that only ever commits and
    /// opens through the one-hot flavor (the packed `OneHotTrace` group): skips
    /// building the dense-flavor backend setup of the same shape.
    pub fn one_hot_only(
        max_num_vars: usize,
        max_num_polys_per_commitment_group: usize,
        default_layout_digest: AkitaLayoutDigest,
        one_hot_k: usize,
    ) -> Self {
        Self {
            max_num_vars,
            max_num_polys_per_commitment_group,
            default_layout_digest,
            one_hot_k,
            one_hot_only: true,
        }
    }

    pub fn one_hot_k(&self) -> usize {
        self.one_hot_k
    }
}

#[derive(Clone, Debug)]
pub struct AkitaProverSetup {
    pub(crate) backend_prover_setup: Option<Arc<AkitaBackendProverSetup>>,
    pub(crate) prepared_backend_setup: Option<Arc<AkitaBackendPreparedSetup>>,
    pub(crate) one_hot_backend_prover_setup: Option<Arc<AkitaBackendProverSetup>>,
    pub(crate) prepared_one_hot_backend_setup: Option<Arc<AkitaBackendPreparedSetup>>,
    pub(crate) verifier: AkitaVerifierSetup,
}

impl AkitaProverSetup {
    pub fn max_num_vars(&self) -> usize {
        self.verifier.max_num_vars
    }

    pub fn max_num_polys_per_commitment_group(&self) -> usize {
        self.verifier.max_num_polys_per_commitment_group
    }

    pub fn default_layout_digest(&self) -> [u8; 32] {
        self.verifier.default_layout_digest
    }

    pub fn one_hot_k(&self) -> usize {
        self.verifier.one_hot_k
    }

    pub(crate) fn dense_backend(
        &self,
    ) -> Result<(&AkitaBackendProverSetup, &AkitaBackendPreparedSetup), OpeningsError> {
        self.backend_prover_setup
            .as_deref()
            .zip(self.prepared_backend_setup.as_deref())
            .ok_or_else(|| {
                OpeningsError::InvalidSetup(
                    "this Akita setup was built without the dense-flavor backend".to_string(),
                )
            })
    }

    pub(crate) fn one_hot_backend(
        &self,
    ) -> Result<(&AkitaBackendProverSetup, &AkitaBackendPreparedSetup), OpeningsError> {
        let backend = self
            .one_hot_backend_prover_setup
            .as_deref()
            .ok_or_else(|| invalid_batch("Akita setup has no one-hot backend"))?;
        let prepared = self
            .prepared_one_hot_backend_setup
            .as_deref()
            .ok_or_else(|| invalid_batch("Akita setup has no prepared one-hot backend"))?;
        Ok((backend, prepared))
    }
}

/// The verifier setup is pure shape: the backend keys are a deterministic
/// function of `(max_num_vars, max_num_polys_per_commitment_group, one_hot_k)`
/// over a fixed internal seed, so they are never serialized or
/// transcript-absorbed — [`append_verifier_setup`] binds these parameters and
/// both sides derive the same keys from them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AkitaVerifierSetup {
    pub(crate) max_num_vars: usize,
    pub(crate) max_num_polys_per_commitment_group: usize,
    pub(crate) default_layout_digest: AkitaLayoutDigest,
    pub(crate) one_hot_k: usize,
    #[serde(skip)]
    pub(crate) backend_cache: BackendVerifierCache,
}

impl AkitaVerifierSetup {
    pub fn max_num_vars(&self) -> usize {
        self.max_num_vars
    }

    pub fn max_num_polys_per_commitment_group(&self) -> usize {
        self.max_num_polys_per_commitment_group
    }

    pub fn default_layout_digest(&self) -> [u8; 32] {
        self.default_layout_digest
    }

    pub fn one_hot_k(&self) -> usize {
        self.one_hot_k
    }

    /// Primes the lazy key cache with freshly built backend keys, so
    /// in-process setups never pay the shape→key re-derivation.
    pub(crate) fn prime_backend_cache(
        &self,
        dense: Option<AkitaBackendVerifier>,
        one_hot: Option<AkitaBackendVerifier>,
    ) {
        if let Some(dense) = dense {
            let _ = self.backend_cache.dense.get_or_init(|| dense);
        }
        if let Some(one_hot) = one_hot {
            let _ = self.backend_cache.one_hot.get_or_init(|| one_hot);
        }
    }

    /// Backend verifier key for `flavor`, cached after the first use.
    /// [`AkitaScheme::setup`](crate::AkitaScheme) primes the cache with the
    /// freshly built keys; a serde-transported setup re-derives them from the
    /// shape on first use (one-time, setup-class cost).
    pub(crate) fn backend_verifier(
        &self,
        flavor: AkitaBackendFlavor,
    ) -> Result<&AkitaBackendVerifier, OpeningsError> {
        let cache = match flavor {
            AkitaBackendFlavor::Dense => &self.backend_cache.dense,
            AkitaBackendFlavor::OneHot => &self.backend_cache.one_hot,
        };
        if let Some(verifier) = cache.get() {
            return Ok(verifier);
        }
        let verifier = self.build_backend_verifier(flavor)?;
        Ok(cache.get_or_init(|| verifier))
    }

    fn build_backend_verifier(
        &self,
        flavor: AkitaBackendFlavor,
    ) -> Result<AkitaBackendVerifier, OpeningsError> {
        let invalid_setup =
            |err: &dyn std::fmt::Display| OpeningsError::InvalidSetup(err.to_string());
        match flavor {
            AkitaBackendFlavor::Dense => {
                let prover_setup = with_backend_pool(|| {
                    AkitaBackendScheme::setup_prover(
                        self.max_num_vars,
                        self.max_num_polys_per_commitment_group,
                    )
                })
                .map_err(|err| invalid_setup(&err))?;
                with_backend_pool(|| AkitaBackendScheme::setup_verifier(&prover_setup))
                    .map_err(|err| invalid_setup(&err))
            }
            AkitaBackendFlavor::OneHot => {
                let log_k = validate_one_hot_k(self.one_hot_k)?;
                if self.max_num_vars < log_k {
                    return Err(invalid_batch("Akita verifier setup has no one-hot backend"));
                }
                let prover_setup = one_hot_setup_prover(
                    self.one_hot_k,
                    self.max_num_vars,
                    self.max_num_polys_per_commitment_group,
                )
                .map_err(|err| invalid_setup(&err))?;
                one_hot_setup_verifier(self.one_hot_k, &prover_setup)
            }
        }
    }
}

/// Lazily deserialized backend verifier keys. Derived state: ignored by
/// equality and skipped by serde; clones share the cache.
#[derive(Clone, Default)]
pub(crate) struct BackendVerifierCache {
    dense: Arc<OnceLock<AkitaBackendVerifier>>,
    one_hot: Arc<OnceLock<AkitaBackendVerifier>>,
}

impl fmt::Debug for BackendVerifierCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BackendVerifierCache")
    }
}

impl PartialEq for BackendVerifierCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for BackendVerifierCache {}

/// Binds the setup key for one backend flavor into the transcript, by shape
/// only: the backend key is a deterministic function of the absorbed
/// dimensions over a fixed internal seed, so hashing the (large) serialized
/// key adds no binding.
pub(crate) fn append_verifier_setup<T: Transcript>(
    transcript: &mut T,
    setup: &AkitaVerifierSetup,
    flavor: AkitaBackendFlavor,
) {
    transcript.append(&Label(b"akita_setup_key"));
    transcript.append_bytes(b"akita/fp128/d64");
    transcript.append_bytes(flavor.transcript_label());
    transcript.append(&U64Word(AKITA_D as u64));
    transcript.append(&U64Word(setup.max_num_vars as u64));
    transcript.append(&U64Word(setup.max_num_polys_per_commitment_group as u64));
    transcript.append(&U64Word(setup.one_hot_k as u64));
    transcript.append_bytes(&setup.default_layout_digest);
}

/// Binds the batch statement (commitment group, point, per-claim data) into
/// the transcript.
pub(crate) fn append_batch_statement<T: Transcript>(
    transcript: &mut T,
    statement: &[VerifierOpeningClaim<AkitaField, AkitaCommitment>],
    commitment: &AkitaCommitment,
    point: &[AkitaField],
) {
    append_batch_statement_header(transcript, commitment, point, statement.len());
    for claim in statement {
        claim.commitment.append_to_transcript(transcript);
        claim.evaluation.value.append_to_transcript(transcript);
    }
}

/// [`append_batch_statement`] for a group whose claims all carry the same
/// commitment and point: emits exactly the same bytes without materializing
/// the per-claim [`VerifierOpeningClaim`]s (each of which would clone
/// `commitment`, byte payload included, once per evaluation).
pub(crate) fn append_batch_statement_values<T: Transcript>(
    transcript: &mut T,
    commitment: &AkitaCommitment,
    point: &[AkitaField],
    evaluations: &[AkitaField],
) {
    append_batch_statement_header(transcript, commitment, point, evaluations.len());
    for value in evaluations {
        commitment.append_to_transcript(transcript);
        value.append_to_transcript(transcript);
    }
}

fn append_batch_statement_header<T: Transcript>(
    transcript: &mut T,
    commitment: &AkitaCommitment,
    point: &[AkitaField],
    claim_count: usize,
) {
    transcript.append(&Label(b"akita_batch_statement"));
    commitment.append_to_transcript(transcript);
    transcript.append_values(b"akita_pcs_point", point);
    transcript.append(&LabelWithCount(b"akita_claims", claim_count as u64));
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AkitaBackendFlavor {
    #[default]
    Dense,
    OneHot,
}

impl AkitaBackendFlavor {
    pub(crate) const fn transcript_label(self) -> &'static [u8] {
        match self {
            Self::Dense => b"dense",
            Self::OneHot => b"one_hot",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AkitaCommitment {
    pub(crate) backend_flavor: AkitaBackendFlavor,
    pub(crate) layout_digest: AkitaLayoutDigest,
    pub(crate) num_vars: usize,
    pub(crate) poly_count: usize,
    pub(crate) one_hot_k: usize,
    /// Field-coefficient count of the serialized backend commitment — the
    /// deserialization context [`akita_types::Commitment`] requires.
    pub(crate) backend_coeff_len: usize,
    #[serde(deserialize_with = "deserialize_backend_payload")]
    pub(crate) serialized_backend_bytes: Vec<u8>,
}

impl jolt_openings::GroupCommitmentMetadata for AkitaCommitment {
    fn is_one_hot_backend(&self) -> bool {
        self.backend_flavor() == AkitaBackendFlavor::OneHot
    }

    fn layout_digest(&self) -> [u8; 32] {
        self.layout_digest()
    }

    fn num_vars(&self) -> usize {
        self.num_vars()
    }

    fn poly_count(&self) -> usize {
        self.poly_count()
    }

    fn one_hot_k(&self) -> usize {
        self.one_hot_k()
    }
}

impl jolt_openings::GroupSetupMetadata for AkitaVerifierSetup {
    fn max_num_vars(&self) -> usize {
        self.max_num_vars()
    }

    fn max_num_polys_per_commitment_group(&self) -> usize {
        self.max_num_polys_per_commitment_group()
    }

    fn default_layout_digest(&self) -> [u8; 32] {
        self.default_layout_digest()
    }

    fn one_hot_k(&self) -> usize {
        self.one_hot_k()
    }
}

impl AkitaCommitment {
    pub fn backend_flavor(&self) -> AkitaBackendFlavor {
        self.backend_flavor
    }

    pub fn layout_digest(&self) -> [u8; 32] {
        self.layout_digest
    }

    pub fn num_vars(&self) -> usize {
        self.num_vars
    }

    pub fn poly_count(&self) -> usize {
        self.poly_count
    }

    pub fn one_hot_k(&self) -> usize {
        self.one_hot_k
    }
}

impl AppendToTranscript for AkitaCommitment {
    fn append_to_transcript<T: Transcript>(&self, transcript: &mut T) {
        transcript.append(&Label(b"akita_commitment"));
        transcript.append_bytes(self.backend_flavor.transcript_label());
        transcript.append_bytes(&self.layout_digest);
        transcript.append(&U64Word(self.num_vars as u64));
        transcript.append(&U64Word(self.poly_count as u64));
        transcript.append(&U64Word(self.one_hot_k as u64));
        transcript.append(&U64Word(self.backend_coeff_len as u64));
        transcript.append(&LabelWithCount(
            b"akita_commitment_bytes",
            self.serialized_backend_bytes.len() as u64,
        ));
        transcript.append_bytes(&self.serialized_backend_bytes);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AkitaBatchProof {
    #[serde(deserialize_with = "deserialize_statement_bridge")]
    pub(crate) statement_bridge: Vec<u8>,
    #[serde(deserialize_with = "deserialize_proof_shape")]
    pub(crate) serialized_akita_proof_shape: Vec<u8>,
    #[serde(deserialize_with = "deserialize_backend_payload")]
    pub(crate) serialized_akita_proof: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AkitaHidingCommitment {
    #[serde(deserialize_with = "deserialize_hiding_commitment")]
    pub(crate) eval: Vec<u8>,
}

impl AkitaHidingCommitment {
    pub(crate) fn new(eval: Vec<u8>) -> Self {
        Self { eval }
    }
}

impl AppendToTranscript for AkitaHidingCommitment {
    fn append_to_transcript<T: Transcript>(&self, transcript: &mut T) {
        transcript.append(&Label(b"akita_hiding_commitment"));
        transcript.append(&LabelWithCount(
            b"akita_hiding_eval",
            self.eval.len() as u64,
        ));
        transcript.append_bytes(&self.eval);
    }
}

#[derive(Clone, Debug, Default)]
pub struct AkitaProverHint {
    pub(crate) commitment: AkitaCommitment,
    pub(crate) backend: Option<(AkitaBackendCommitment, AkitaBackendHint)>,
    pub(crate) polynomials: AkitaHintPolynomials,
}

/// Backend representation of the committed polynomials, produced at commit
/// time and reused when opening. The variant doubles as the source-kind
/// discriminator, so a hint can never pair one kind's metadata with another
/// kind's polynomials.
#[derive(Clone, Debug)]
pub(crate) enum AkitaHintPolynomials {
    Dense(Arc<[AkitaBackendDensePoly]>),
    OneHot(Arc<[AkitaBackendOneHotPoly]>),
    SparseUnit(Arc<[AkitaBackendSparsePoly]>),
}

impl Default for AkitaHintPolynomials {
    fn default() -> Self {
        Self::Dense(Vec::new().into())
    }
}

impl AkitaHintPolynomials {
    pub(crate) const fn backend_flavor(&self) -> AkitaBackendFlavor {
        match self {
            Self::Dense(_) | Self::SparseUnit(_) => AkitaBackendFlavor::Dense,
            Self::OneHot(_) => AkitaBackendFlavor::OneHot,
        }
    }

    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::Dense(_) => "dense",
            Self::OneHot(_) => "one_hot",
            Self::SparseUnit(_) => "sparse_unit",
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Dense(polys) => polys.len(),
            Self::OneHot(polys) => polys.len(),
            Self::SparseUnit(polys) => polys.len(),
        }
    }

    pub(crate) fn one_hot_k(&self) -> Option<usize> {
        match self {
            Self::OneHot(polys) => polys
                .first()
                .and_then(akita_prover::RootPolyMeta::onehot_chunk_size),
            Self::Dense(_) | Self::SparseUnit(_) => None,
        }
    }
}

/// `2^num_vars`, or `None` when it does not fit in `usize`.
pub(crate) fn domain_size(num_vars: usize) -> Option<usize> {
    u32::try_from(num_vars)
        .ok()
        .and_then(|shift| 1usize.checked_shl(shift))
}

#[doc(hidden)]
pub fn reverse_point(point: &[AkitaField]) -> Vec<AkitaField> {
    point.iter().rev().copied().collect()
}

pub(crate) fn backend_stack<'a>(
    backend_prover_setup: &'a AkitaBackendProverSetup,
    prepared_backend_setup: &'a AkitaBackendPreparedSetup,
) -> Result<BackendStack<'a>, OpeningsError> {
    let _span = info_span!("jolt_akita::make_backend_stack").entered();
    akita_prover::UniformProverStack::uniform(
        &CpuBackend,
        prepared_backend_setup,
        backend_prover_setup.expanded.as_ref(),
    )
    .map_err(|err| OpeningsError::InvalidSetup(err.to_string()))
}

pub(crate) fn one_hot_polynomial<P>(
    polynomial: &P,
    one_hot_k: usize,
) -> Result<Option<AkitaBackendOneHotPoly>, OpeningsError>
where
    P: MultilinearPoly<AkitaField> + ?Sized,
{
    if !polynomial.is_one_hot()
        || polynomial.one_hot_k() != Some(one_hot_k)
        || polynomial.one_hot_index_order() != Some(OneHotIndexOrder::RowMajor)
    {
        return Ok(None);
    }

    let indices = polynomial
        .one_hot_indices()
        .ok_or_else(|| invalid_batch("Jolt one-hot polynomial did not expose its indices"))?;
    let _ = validate_one_hot_k(one_hot_k)?;
    AkitaBackendOneHotPoly::new(one_hot_k, AKITA_D, indices.to_vec())
        .map(Some)
        .map_err(akita_error)
}

pub(crate) fn validate_one_hot_k(one_hot_k: usize) -> Result<usize, OpeningsError> {
    match one_hot_k {
        AKITA_ONE_HOT_K16 => Ok(4),
        AKITA_ONE_HOT_K256 => Ok(8),
        _ => Err(invalid_batch(format!(
            "Akita one-hot chunk size must be 16 or 256, got {one_hot_k}"
        ))),
    }
}

pub(crate) fn one_hot_setup_prover(
    one_hot_k: usize,
    max_num_vars: usize,
    max_num_polys: usize,
) -> Result<AkitaBackendProverSetup, akita_pcs::AkitaError> {
    with_backend_pool(|| match one_hot_k {
        AKITA_ONE_HOT_K16 => AkitaOneHotK16BackendScheme::setup_prover(max_num_vars, max_num_polys),
        AKITA_ONE_HOT_K256 => {
            AkitaOneHotK256BackendScheme::setup_prover(max_num_vars, max_num_polys)
        }
        _ => Err(akita_pcs::AkitaError::InvalidSetup(format!(
            "Akita one-hot chunk size must be 16 or 256, got {one_hot_k}"
        ))),
    })
}

pub(crate) fn one_hot_setup_verifier(
    one_hot_k: usize,
    prover_setup: &AkitaBackendProverSetup,
) -> Result<AkitaBackendVerifier, OpeningsError> {
    let invalid_setup = |err: &dyn std::fmt::Display| OpeningsError::InvalidSetup(err.to_string());
    match one_hot_k {
        AKITA_ONE_HOT_K16 => {
            with_backend_pool(|| AkitaOneHotK16BackendScheme::setup_verifier(prover_setup))
                .map_err(|err| invalid_setup(&err))
        }
        AKITA_ONE_HOT_K256 => {
            with_backend_pool(|| AkitaOneHotK256BackendScheme::setup_verifier(prover_setup))
                .map_err(|err| invalid_setup(&err))
        }
        _ => Err(invalid_batch(format!(
            "Akita one-hot chunk size must be 16 or 256, got {one_hot_k}"
        ))),
    }
}

pub(crate) fn sparse_unit_polynomial(
    num_vars: usize,
    indices: impl IntoIterator<Item = usize>,
) -> Result<AkitaBackendSparsePoly, OpeningsError> {
    let domain_size = domain_size(num_vars).ok_or_else(|| {
        invalid_batch(format!(
            "Akita sparse polynomial dimension {num_vars} exceeds usize bit width"
        ))
    })?;
    if domain_size < AKITA_D {
        return Err(invalid_batch(format!(
            "Akita sparse polynomial domain {domain_size} is smaller than ring dimension {AKITA_D}"
        )));
    }

    let mut seen = BTreeSet::new();
    let mut coeffs = Vec::new();
    for index in indices {
        if index >= domain_size {
            return Err(invalid_batch(format!(
                "Akita sparse polynomial index {index} outside domain size {domain_size}"
            )));
        }
        if !seen.insert(index) {
            return Err(invalid_batch(format!(
                "Akita sparse polynomial index {index} appears more than once"
            )));
        }
        let akita_index = jolt_to_akita_index(num_vars, index);
        coeffs.push((akita_index / AKITA_D, akita_index % AKITA_D, 1i8));
    }

    AkitaBackendSparsePoly::from_signed_coeffs(num_vars, AKITA_D, domain_size / AKITA_D, coeffs)
        .map_err(|error| {
            invalid_batch(format!(
                "Akita sparse polynomial construction failed: {error}"
            ))
        })
}

pub(crate) fn jolt_to_akita_index(num_vars: usize, index: usize) -> usize {
    if num_vars == 0 {
        return index;
    }
    index.reverse_bits() >> (usize::BITS as usize - num_vars)
}

pub(crate) fn dense_polynomials(
    polynomials: &[Polynomial<AkitaField>],
) -> Result<Vec<AkitaBackendDensePoly>, OpeningsError> {
    polynomials
        .iter()
        .map(|poly| {
            let evals = jolt_to_akita_evals(poly.num_vars(), poly.evals())?;
            AkitaBackendDensePoly::from_field_evals(poly.num_vars(), AKITA_D, &evals)
                .map_err(akita_error)
        })
        .collect()
}

#[doc(hidden)]
pub fn jolt_to_akita_evals(
    num_vars: usize,
    jolt_evals: &[AkitaField],
) -> Result<Vec<AkitaField>, OpeningsError> {
    let Some(expected) = domain_size(num_vars) else {
        return Err(invalid_batch(format!(
            "Akita polynomial dimension {num_vars} exceeds usize bit width"
        )));
    };
    if jolt_evals.len() != expected {
        return Err(invalid_batch(format!(
            "Akita polynomial has {} evaluations but dimension {num_vars} requires {expected}",
            jolt_evals.len()
        )));
    }
    if num_vars == 0 {
        return Ok(jolt_evals.to_vec());
    }
    let mut akita_evals = vec![AkitaField::zero(); jolt_evals.len()];
    for (jolt_index, &eval) in jolt_evals.iter().enumerate() {
        let akita_index = jolt_to_akita_index(num_vars, jolt_index);
        akita_evals[akita_index] = eval;
    }
    Ok(akita_evals)
}

/// Materializes a polynomial's evaluations directly in Akita's (bit-reversed)
/// index order, avoiding a second full-size buffer for the reorder pass.
pub(crate) fn akita_ordered_evaluations<P>(polynomial: &P) -> Result<Vec<AkitaField>, OpeningsError>
where
    P: MultilinearPoly<AkitaField> + ?Sized,
{
    let num_vars = polynomial.num_vars();
    let Some(len) = domain_size(num_vars) else {
        return Err(invalid_batch(format!(
            "Akita polynomial dimension {num_vars} exceeds usize bit width"
        )));
    };
    let mut evals = vec![AkitaField::zero(); len];
    let mut jolt_index = 0usize;
    polynomial.for_each_row(num_vars, &mut |_, row| {
        for &eval in row {
            evals[jolt_to_akita_index(num_vars, jolt_index)] = eval;
            jolt_index += 1;
        }
    });
    Ok(evals)
}

pub(crate) fn serialize_akita<T>(value: &T) -> Result<Vec<u8>, OpeningsError>
where
    T: AkitaSerialize,
{
    let mut bytes = Vec::with_capacity(value.compressed_size());
    value
        .serialize_compressed(&mut bytes)
        .map_err(akita_error)?;
    Ok(bytes)
}

pub(crate) fn deserialize_akita<T>(bytes: &[u8], ctx: &T::Context) -> Result<T, OpeningsError>
where
    T: AkitaDeserialize,
{
    let mut cursor = Cursor::new(bytes);
    let value = T::deserialize_compressed(&mut cursor, ctx).map_err(akita_error)?;
    if cursor.position() != bytes.len() as u64 {
        return Err(invalid_batch(
            "Akita payload has trailing bytes after deserialization",
        ));
    }
    Ok(value)
}

pub(crate) fn invalid_batch(message: impl Into<String>) -> OpeningsError {
    OpeningsError::InvalidBatch(message.into())
}

pub(crate) fn akita_error(error: impl ToString) -> OpeningsError {
    OpeningsError::InvalidBatch(error.to_string())
}

pub(crate) fn commit_failed(error: impl ToString) -> OpeningsError {
    OpeningsError::CommitFailed(error.to_string())
}

pub(crate) fn prove_failed(error: impl ToString) -> OpeningsError {
    OpeningsError::ProveFailed(error.to_string())
}

pub(crate) fn transparent_zk_error() -> OpeningsError {
    OpeningsError::InvalidBatch(
        "Akita backend adapter is transparent-only and does not support ZK openings yet".to_owned(),
    )
}

impl AppendToTranscript for AkitaBatchProof {
    fn append_to_transcript<T: Transcript>(&self, transcript: &mut T) {
        transcript.append(&LabelWithCount(
            b"akita_stmt_bridge",
            self.statement_bridge.len() as u64,
        ));
        transcript.append_bytes(&self.statement_bridge);
        transcript.append(&LabelWithCount(
            b"akita_proof_shape",
            self.serialized_akita_proof_shape.len() as u64,
        ));
        transcript.append_bytes(&self.serialized_akita_proof_shape);
        transcript.append(&LabelWithCount(
            b"akita_proof",
            self.serialized_akita_proof.len() as u64,
        ));
        transcript.append_bytes(&self.serialized_akita_proof);
    }
}

/// Opens a fresh Akita transcript bound to everything Jolt has observed, and
/// returns it alongside the bridge bytes carried in the proof.
///
/// WARNING: the bridge has to travel inside the *session label*. Akita's
/// `batched_prove`/`batched_verify` call `bind_instance_bytes` before their
/// first absorb, which rebuilds the sponge from scratch; only the session tag
/// survives that rebind, so a bridge merely absorbed into the transcript would
/// be silently discarded and bind nothing.
pub(crate) fn bridge_jolt_statement_challenge<T>(
    jolt_transcript: &mut T,
    session_label: &[u8],
) -> (AkitaTranscript<AkitaField>, Vec<u8>)
where
    T: Transcript<Challenge = AkitaField>,
{
    let bridge = jolt_transcript.challenge_scalar().to_bytes_le_vec();
    let mut label = Vec::with_capacity(session_label.len() + bridge.len());
    label.extend_from_slice(session_label);
    label.extend_from_slice(&bridge);
    (AkitaTranscript::new(&label), bridge)
}

#[cfg(test)]
mod batch_statement_tests {
    use super::*;
    use jolt_openings::EvaluationClaim;
    use jolt_transcript::Blake2bTranscript;

    /// The fused multi-group path binds a shared-commitment group through
    /// [`append_batch_statement_values`]; the single-group path still walks
    /// materialized claims. The two must be byte-identical or a fused proof and
    /// its verifier would diverge from the single-group Fiat-Shamir domain.
    #[test]
    fn value_and_claim_bindings_are_byte_identical() {
        let commitment = AkitaCommitment {
            backend_flavor: AkitaBackendFlavor::OneHot,
            layout_digest: [3; 32],
            num_vars: 5,
            poly_count: 3,
            one_hot_k: AKITA_ONE_HOT_K256,
            backend_coeff_len: 17,
            serialized_backend_bytes: vec![1, 2, 3, 4, 5],
        };
        let point: Vec<AkitaField> = (0..5).map(AkitaField::from_u64).collect();
        let evaluations: Vec<AkitaField> = (10..13).map(AkitaField::from_u64).collect();
        let statement: Vec<_> = evaluations
            .iter()
            .map(|value| VerifierOpeningClaim {
                commitment: commitment.clone(),
                evaluation: EvaluationClaim::new(point.clone(), *value),
            })
            .collect();

        let mut claims_transcript = Blake2bTranscript::<AkitaField>::new(b"batch-statement");
        append_batch_statement(&mut claims_transcript, &statement, &commitment, &point);

        let mut values_transcript = Blake2bTranscript::<AkitaField>::new(b"batch-statement");
        append_batch_statement_values(&mut values_transcript, &commitment, &point, &evaluations);

        assert_eq!(claims_transcript.state(), values_transcript.state());
    }
}

#[cfg(test)]
mod statement_bridge_tests {
    use super::*;
    use akita_transcript::Transcript as AkitaBackendTranscript;
    use jolt_transcript::Blake2bTranscript;

    /// Squeezes a challenge the way `batched_prove` does: rebind to the instance
    /// descriptor first, then absorb. Anything bound before the rebind is only
    /// observable here if it survived it.
    fn challenge_after_instance_rebind(mut akita: AkitaTranscript<AkitaField>) -> Vec<u8> {
        akita.bind_instance_bytes(b"akita/instance-descriptor");
        akita.challenge_bytes(b"probe", 32)
    }

    #[test]
    fn bridge_survives_the_instance_rebind() {
        let mut left = Blake2bTranscript::new(b"bridge-test");
        left.append(&U64Word(1));
        let (left_akita, left_bridge) = bridge_jolt_statement_challenge(&mut left, b"label");

        let mut right = Blake2bTranscript::new(b"bridge-test");
        right.append(&U64Word(2));
        let (right_akita, right_bridge) = bridge_jolt_statement_challenge(&mut right, b"label");

        assert_ne!(left_bridge, right_bridge, "distinct Jolt states must bridge");
        assert_ne!(
            challenge_after_instance_rebind(left_akita),
            challenge_after_instance_rebind(right_akita),
            "the bridge must still steer Akita's challenges after bind_instance_bytes; \
             absorbing it into the transcript instead of the session label loses it"
        );
    }

    #[test]
    fn equal_jolt_states_bridge_identically() {
        let mut left = Blake2bTranscript::new(b"bridge-test");
        left.append(&U64Word(7));
        let (left_akita, left_bridge) = bridge_jolt_statement_challenge(&mut left, b"label");

        let mut right = Blake2bTranscript::new(b"bridge-test");
        right.append(&U64Word(7));
        let (right_akita, right_bridge) = bridge_jolt_statement_challenge(&mut right, b"label");

        assert_eq!(left_bridge, right_bridge);
        assert_eq!(
            challenge_after_instance_rebind(left_akita),
            challenge_after_instance_rebind(right_akita),
        );
    }

    #[test]
    fn session_labels_domain_separate() {
        let bridged = |label: &[u8]| {
            let mut jolt = Blake2bTranscript::new(b"bridge-test");
            jolt.append(&U64Word(1));
            challenge_after_instance_rebind(bridge_jolt_statement_challenge(&mut jolt, label).0)
        };
        assert_ne!(
            bridged(b"jolt-akita/batch"),
            bridged(b"jolt-akita/multi-group-batch"),
        );
    }
}

#[cfg(test)]
mod bounded_deserialization_tests {
    #![expect(
        clippy::expect_used,
        reason = "tests assert successful fixture serialization"
    )]

    use super::*;

    fn encode_length(length: usize) -> Vec<u8> {
        bincode::encode_to_vec(length, bincode::config::standard()).expect("encode length")
    }

    #[test]
    fn proof_shape_length_rejects_before_reading_payload() {
        let mut encoded = vec![0];
        encoded.extend(encode_length(MAX_PROOF_SHAPE_BYTES + 1));
        let error = bincode::serde::decode_from_slice::<AkitaBatchProof, _>(
            &encoded,
            bincode::config::standard(),
        )
        .expect_err("oversized declared proof shape must reject");
        assert!(error.to_string().contains("protocol cap"));
    }

    #[test]
    fn backend_proof_length_rejects_before_reading_payload() {
        let mut encoded = vec![0, 0];
        encoded.extend(encode_length(MAX_BACKEND_PAYLOAD_BYTES + 1));
        let error = bincode::serde::decode_from_slice::<AkitaBatchProof, _>(
            &encoded,
            bincode::config::standard(),
        )
        .expect_err("oversized declared backend proof must reject");
        assert!(error.to_string().contains("protocol cap"));
    }

    #[test]
    fn backend_commitment_length_rejects_before_reading_payload() {
        let commitment = AkitaCommitment::default();
        let mut encoded = bincode::serde::encode_to_vec(&commitment, bincode::config::standard())
            .expect("encode empty commitment");
        assert_eq!(encoded.pop(), Some(0), "empty payload has a zero length");
        encoded.extend(encode_length(MAX_BACKEND_PAYLOAD_BYTES + 1));
        let error = bincode::serde::decode_from_slice::<AkitaCommitment, _>(
            &encoded,
            bincode::config::standard(),
        )
        .expect_err("oversized declared backend commitment must reject");
        assert!(error.to_string().contains("protocol cap"));
    }

    #[test]
    fn proof_shape_cap_is_inclusive() {
        let proof = AkitaBatchProof {
            statement_bridge: Vec::new(),
            serialized_akita_proof_shape: vec![0; MAX_PROOF_SHAPE_BYTES],
            serialized_akita_proof: Vec::new(),
        };
        let encoded = bincode::serde::encode_to_vec(&proof, bincode::config::standard())
            .expect("encode boundary proof");
        let (decoded, consumed) = bincode::serde::decode_from_slice::<AkitaBatchProof, _>(
            &encoded,
            bincode::config::standard(),
        )
        .expect("shape exactly at the cap must deserialize");
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded, proof);
    }
}
