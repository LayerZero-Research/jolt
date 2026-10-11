//! Production preprocessing shapes and the Akita setup request each induces.
//!
//! A [`Shape`] is what a Jolt deployment actually chooses: the padded trace
//! length, the one-hot chunk width, the witness chunk profile, the bytecode
//! and RAM domains, the advice capacities baked into the guest's memory
//! layout, and whether the program is committed (and in how many bytecode
//! chunks). Every derived quantity comes from Jolt's own functions:
//! `ProverConfig::for_padded_shape`, `MemoryLayout::try_new`,
//! `one_hot_trace_setup_shape`, `advice_packing_plan`,
//! `committed_program_packing_plan`, and the setup parameters
//! `grouped_setup_params_for_shape` builds for `AkitaScheme::setup`.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::Arc;

use common::constants::RAM_START_ADDRESS;
use common::jolt_device::{MemoryConfig, MemoryLayout};
use jolt_akita::schedules::emit::K16_TRACE_LOG_T;
use jolt_akita::{AkitaChunkProfile, AkitaScheduleArtifacts, AkitaSetupParams, AKITA_ONE_HOT_K256};
use jolt_claims::protocols::jolt::geometry::claim_reductions::bytecode::{
    committed_lane_vars, MAX_COMMITTED_BYTECODE_CHUNK_COUNT,
};
use jolt_claims::protocols::jolt::lattice::strategy::MAX_ONE_HOT_TRACE_COLUMNS;
use jolt_claims::protocols::jolt::lattice::{
    advice_packing_plan, committed_program_packing_plan, one_hot_trace_columns,
    OneHotTraceSetupShape, OneHotTraceShape, PrefixPackedObjectPlan, ADVICE_MAX_PHYSICAL_VARS,
    DIRECT_PROGRAM_MAX_PHYSICAL_VARS,
};
use jolt_claims::protocols::jolt::{
    JoltAdviceKind, JoltOneHotConfig, JoltRelationId, TracePolynomialOrder,
};
use jolt_prover::akita::one_hot_trace_setup_shape;
use jolt_prover::akita::preprocessing::{advice_word_vars, grouped_setup_params_for_shape};
use jolt_prover::{remap_address, ProverConfig, MIN_PADDED_TRACE_LENGTH};
use jolt_verifier::stages::formula_dimensions_from_parts;

use crate::artifacts;
use crate::input::Reader;

pub const MIN_LOG_T: usize = MIN_PADDED_TRACE_LENGTH.ilog2() as usize;
/// Largest trace the production K=16 catalogs cover.
pub const MAX_LOG_T: usize = K16_TRACE_LOG_T.1;
pub const MAX_LOG_CHUNKS: usize = MAX_COMMITTED_BYTECODE_CHUNK_COUNT.ilog2() as usize;

pub const PROFILES: [AkitaChunkProfile; 4] = [
    AkitaChunkProfile::Single,
    AkitaChunkProfile::Two,
    AkitaChunkProfile::Four,
    AkitaChunkProfile::Eight,
];

/// The K=256 override `akita_e2e` forces: 8-bit committed chunks with 32-bit
/// virtual lookup chunks.
const FORCED_K256: JoltOneHotConfig = JoltOneHotConfig {
    log_k_chunk: 8,
    lookups_ra_virtual_log_k_chunk: 32,
};

/// Which one-hot chunk width the trace commits with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chunking {
    /// `ProverConfig::derive`'s rule under Akita: K=16 at every trace length.
    Production,
    /// The K=256 override (`akita_e2e` forces it), supported only at the
    /// trace shapes its catalogs list.
    ForcedK256,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommittedProgram {
    pub log_chunks: usize,
    /// Program-image words before padding (`bytecode_words.len()`).
    pub image_words: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub log_t: usize,
    pub chunking: Chunking,
    pub profile: AkitaChunkProfile,
    pub log_bytecode_len: usize,
    pub log_ram_k: usize,
    /// Advice capacities in bytes (`MemoryLayout::max_*_advice_size`, a
    /// power of two); `None` when the program takes no such advice.
    pub untrusted_advice_bytes: Option<u64>,
    pub trusted_advice_bytes: Option<u64>,
    pub program: Option<CommittedProgram>,
}

impl Display for Shape {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "log_T={} K=2^{} profile={:?} bytecode=2^{} ram_K=2^{}",
            self.log_t,
            self.log_k_chunk(),
            self.profile,
            self.log_bytecode_len,
            self.log_ram_k
        )?;
        if let Some(bytes) = self.untrusted_advice_bytes {
            write!(f, " untrusted={bytes}B")?;
        }
        if let Some(bytes) = self.trusted_advice_bytes {
            write!(f, " trusted={bytes}B")?;
        }
        if let Some(program) = self.program {
            write!(
                f,
                " committed(chunks=2^{}, image_words={})",
                program.log_chunks, program.image_words
            )?;
        }
        Ok(())
    }
}

/// The grouped setup a shape induces, with the auxiliary objects' plans.
#[derive(Clone, Debug)]
pub struct SetupRequest {
    pub setup_shape: OneHotTraceSetupShape,
    pub layout_digest: [u8; 32],
    pub one_hot_k: usize,
    pub profile: AkitaChunkProfile,
    pub untrusted: Option<PrefixPackedObjectPlan>,
    pub trusted: Option<PrefixPackedObjectPlan>,
    pub program: Vec<PrefixPackedObjectPlan>,
    pub params: AkitaSetupParams,
}

/// An auxiliary object's packed physical arity.
pub fn arity(plan: &PrefixPackedObjectPlan) -> usize {
    plan.packing().packed_num_vars()
}

impl SetupRequest {
    /// A trace group with no auxiliary objects.
    pub fn trace_only(
        artifacts: &Arc<AkitaScheduleArtifacts>,
        setup_shape: OneHotTraceSetupShape,
        layout_digest: [u8; 32],
        one_hot_k: usize,
        profile: AkitaChunkProfile,
    ) -> Self {
        let params = AkitaSetupParams::one_hot_only_grouped(
            setup_shape.num_vars,
            setup_shape.num_polys,
            setup_shape.num_polys,
            layout_digest,
            one_hot_k,
            None,
            Arc::clone(artifacts),
        )
        .with_akita_chunk_profile(profile);
        Self {
            setup_shape,
            layout_digest,
            one_hot_k,
            profile,
            untrusted: None,
            trusted: None,
            program: Vec::new(),
            params,
        }
    }

    /// Auxiliary objects in canonical opening order (advice, then program).
    pub fn auxiliary(&self) -> impl Iterator<Item = &PrefixPackedObjectPlan> {
        self.untrusted
            .iter()
            .chain(self.trusted.iter())
            .chain(self.program.iter())
    }

    pub fn auxiliary_count(&self) -> usize {
        self.auxiliary().count()
    }

    /// Total committed coefficients across every group of the opening.
    pub fn total_coefficients(&self) -> u128 {
        let trace = (1u128 << self.setup_shape.num_vars) * self.setup_shape.num_polys as u128;
        self.auxiliary()
            .map(|plan| 1u128 << arity(plan))
            .sum::<u128>()
            + trace
    }
}

/// Where a shape stopped on its way to an Akita setup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Jolt's trace geometry (`one_hot_trace_setup_shape`).
    TraceGeometry,
    /// Advice packing (`advice_packing_plan`).
    AdvicePacking,
    /// Committed-program packing (`committed_program_packing_plan`).
    ProgramPacking,
    /// Grouped-row planning (`GroupedScheduleParams::extend_catalog`).
    Planning,
    /// `AkitaScheme::setup` or a later commit/prove/verify step.
    Setup,
    Commit,
    Prove,
    Verify,
}

#[derive(Clone, Debug)]
pub struct Failure {
    pub stage: Stage,
    pub message: String,
}

impl Display for Failure {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}: {}", self.stage, self.message)
    }
}

/// Wrap an error from `stage` as a [`Failure`].
pub fn fail_with<E: Display>(stage: Stage) -> impl Fn(E) -> Failure {
    move |error| Failure {
        stage: stage.clone(),
        message: error.to_string(),
    }
}

impl Shape {
    pub fn log_k_chunk(&self) -> u8 {
        match self.chunking {
            Chunking::Production => 4,
            Chunking::ForcedK256 => FORCED_K256.log_k_chunk,
        }
    }

    /// The configuration `ProverConfig::derive` selects for this padded
    /// shape, with the K=256 override and the chunk profile applied.
    pub fn prover_config(&self) -> ProverConfig {
        let mut config = ProverConfig::for_padded_shape(1 << self.log_t, 1 << self.log_ram_k);
        if self.chunking == Chunking::ForcedK256 {
            config.one_hot_config = FORCED_K256;
        }
        config.akita_chunk_profile = self.profile;
        config
    }

    fn image_words(&self) -> usize {
        self.program.map_or(1, |program| program.image_words)
    }

    /// The guest memory layout this shape's advice capacities require.
    pub fn memory_layout(&self) -> Option<MemoryLayout> {
        MemoryLayout::try_new(&MemoryConfig {
            max_untrusted_advice_size: self.untrusted_advice_bytes.unwrap_or(0),
            max_trusted_advice_size: self.trusted_advice_bytes.unwrap_or(0),
            program_size: Some(8 * self.image_words() as u64),
            ..MemoryConfig::default()
        })
        .ok()
    }

    /// The trace's native column count, before the 64-column limit is
    /// applied; `None` when the geometry itself rejects the shape.
    fn trace_columns(&self) -> Option<usize> {
        let config = self.prover_config();
        let dimensions = formula_dimensions_from_parts(
            config.one_hot_config,
            self.log_t,
            1 << self.log_bytecode_len,
            config.ram_K,
            JoltRelationId::HammingWeightClaimReduction,
        )
        .ok()?;
        one_hot_trace_columns(&OneHotTraceShape {
            ra_layout: dimensions.ra_layout,
            log_t: self.log_t,
            log_k_chunk: usize::from(self.log_k_chunk()),
        })
        .ok()
        .map(|columns| columns.len())
    }

    /// Whether every documented Jolt limit admits this shape. A shape inside
    /// the contract must preprocess, commit, prove, and verify; anything that
    /// stops it is a liveness finding. Limits (each with its source):
    ///
    /// - `log_T` from `MIN_PADDED_TRACE_LENGTH` to the K=16 catalogs'
    ///   `K16_TRACE_LOG_T`, in every chunk profile;
    /// - at most `MAX_ONE_HOT_TRACE_COLUMNS` native trace columns;
    /// - under the K=256 override, a trace group the selected profile's K=256
    ///   catalog lists (it has no general trace range);
    /// - advice capacities `MemoryLayout::try_new` admits together, each of
    ///   physical arity at most `ADVICE_MAX_PHYSICAL_VARS`;
    /// - a RAM domain covering the I/O region and program image, as
    ///   `ProverConfig::derive` sizes it;
    /// - a committed program with a power-of-two chunk count at most
    ///   `MAX_COMMITTED_BYTECODE_CHUNK_COUNT` dividing the bytecode length,
    ///   and chunk and image arities at most `DIRECT_PROGRAM_MAX_PHYSICAL_VARS`.
    ///
    /// A shape whose geometry Jolt rejects for another reason stays in the
    /// contract, so the rejection surfaces as a finding.
    pub fn in_contract(&self) -> bool {
        let columns = self.trace_columns();
        let trace_ok = (MIN_LOG_T..=MAX_LOG_T).contains(&self.log_t)
            && columns.is_none_or(|columns| columns <= MAX_ONE_HOT_TRACE_COLUMNS)
            && match self.chunking {
                Chunking::Production => true,
                Chunking::ForcedK256 => {
                    one_hot_trace_setup_shape(&self.prover_config(), 1 << self.log_bytecode_len)
                        .is_ok_and(|(shape, _, _)| {
                            artifacts::catalogs(AKITA_ONE_HOT_K256, self.profile)
                                .one_hot_rows
                                .contains(&(shape.num_vars, shape.num_polys))
                        })
                }
            };
        let Some(layout) = self.memory_layout() else {
            return false;
        };
        let ram_ok = remap_address(RAM_START_ADDRESS, &layout).is_some_and(|start| {
            (start + self.image_words() as u64 + 1)
                .checked_next_power_of_two()
                .is_some_and(|floor| floor <= 1u64 << self.log_ram_k)
        });
        let advice_ok = |bytes: Option<u64>| {
            bytes.is_none_or(|bytes| advice_word_vars(bytes as usize) <= ADVICE_MAX_PHYSICAL_VARS)
        };
        let program_ok = self.program.is_none_or(|program| {
            program.log_chunks <= MAX_LOG_CHUNKS
                && program.log_chunks <= self.log_bytecode_len
                && committed_lane_vars() + self.log_bytecode_len - program.log_chunks
                    <= DIRECT_PROGRAM_MAX_PHYSICAL_VARS
                && program.image_words >= 1
                && program
                    .image_words
                    .checked_next_power_of_two()
                    .is_some_and(|words| words.ilog2() as usize <= DIRECT_PROGRAM_MAX_PHYSICAL_VARS)
        });
        trace_ok
            && ram_ok
            && advice_ok(self.untrusted_advice_bytes)
            && advice_ok(self.trusted_advice_bytes)
            && program_ok
    }

    /// Everything preprocessing derives before calling `AkitaScheme::setup`.
    pub fn setup_request(&self) -> Result<SetupRequest, Failure> {
        let bytecode_len = 1usize << self.log_bytecode_len;
        let config = self.prover_config();
        let (setup_shape, layout_digest, one_hot_k) =
            one_hot_trace_setup_shape(&config, bytecode_len)
                .map_err(fail_with(Stage::TraceGeometry))?;
        let advice = |kind: JoltAdviceKind, bytes: Option<u64>| {
            bytes
                .map(|bytes| {
                    advice_packing_plan(kind, advice_word_vars(bytes as usize))
                        .map_err(fail_with(Stage::AdvicePacking))
                })
                .transpose()
        };
        let untrusted = advice(JoltAdviceKind::Untrusted, self.untrusted_advice_bytes)?;
        let trusted = advice(JoltAdviceKind::Trusted, self.trusted_advice_bytes)?;
        let program: Vec<PrefixPackedObjectPlan> = match self.program {
            None => Vec::new(),
            Some(program) => committed_program_packing_plan(
                bytecode_len,
                1 << program.log_chunks,
                program.image_words,
                TracePolynomialOrder::CycleMajor,
            )
            .map_err(fail_with(Stage::ProgramPacking))?
            .objects()
            .cloned()
            .collect(),
        };
        let program_vars: Vec<usize> = program.iter().map(arity).collect();
        let params = grouped_setup_params_for_shape(
            &artifacts::shared(),
            &config,
            bytecode_len,
            untrusted.as_ref().map(arity),
            trusted.as_ref().map(arity),
            &program_vars,
        )
        .map_err(fail_with(Stage::TraceGeometry))?;
        Ok(SetupRequest {
            setup_shape,
            layout_digest,
            one_hot_k,
            profile: self.profile,
            untrusted,
            trusted,
            program,
            params,
        })
    }

    /// Decode a shape biased toward the documented edges. Every field reads
    /// its bytes whether or not it is present, so toggling one field leaves
    /// the others in place.
    pub fn decode(reader: &mut Reader<'_>) -> Self {
        let chunking = match reader.u8() % 8 {
            0 => Chunking::ForcedK256,
            _ => Chunking::Production,
        };
        let profile = PROFILES[usize::from(reader.u8()) % PROFILES.len()];
        let log_t = edge_biased(reader, MIN_LOG_T, MAX_LOG_T + 1, &[12, 16, 20, 21, 25, 30]);
        let log_bytecode_len = edge_biased(reader, 0, 32, &[4, 8, 16, 20, 24, 32]);
        let log_ram_k = edge_biased(reader, 1, 61, &[16, 20, 24, 32, 40, 60]);
        let advice = |reader: &mut Reader<'_>| {
            let present = !reader.u8().is_multiple_of(3);
            let log_bytes = edge_biased(reader, 3, 40, &[3, 12, 17, 20, 26, 29]);
            present.then(|| 1u64 << log_bytes)
        };
        let untrusted_advice_bytes = advice(reader);
        let trusted_advice_bytes = advice(reader);
        let present = reader.u8().is_multiple_of(2);
        let log_chunks = edge_biased(reader, 0, MAX_LOG_CHUNKS, &[0, 1, 7, 8]);
        let image_kind = reader.u8() % 4;
        let log_image = edge_biased(reader, 0, 34, &[0, 13, 14, 20]);
        let small_image = reader.u32() as usize % (1 << 20);
        let image_words = match image_kind {
            0 => 1 << log_image,
            1 => (1 << log_image.min(33)) + 1,
            _ => 1 + small_image,
        };
        Self {
            log_t,
            chunking,
            profile,
            log_bytecode_len,
            log_ram_k,
            untrusted_advice_bytes,
            trusted_advice_bytes,
            program: present.then_some(CommittedProgram {
                log_chunks,
                image_words,
            }),
        }
    }
}

/// Uniform in `min..=max` or one of `edges` (clamped), half the time each.
fn edge_biased(reader: &mut Reader<'_>, min: usize, max: usize, edges: &[usize]) -> usize {
    let tag = reader.u8();
    let value = reader.u8() as usize;
    let span = max - min + 1;
    if tag.is_multiple_of(2) && !edges.is_empty() {
        let edge = edges[value % edges.len()];
        let jitter = usize::from(tag >> 1) % 3;
        (edge + jitter).saturating_sub(1).clamp(min, max)
    } else {
        min + value % span
    }
}

/// Grouped-row provisioning alone: the part of `AkitaScheme::setup` that
/// depends on the shape's auxiliary objects. Cheap enough to sweep
/// exhaustively. Returns the number of rows provisioned beyond the base
/// catalog.
pub fn plan(request: &SetupRequest) -> Result<usize, Failure> {
    let Some(grouped) = request.params.grouped_schedule() else {
        return Ok(0);
    };
    let catalogs = artifacts::catalogs(request.one_hot_k, request.profile);
    grouped
        .extend_catalog(
            &catalogs.dense,
            &catalogs.full_dense,
            &catalogs.one_hot,
            request.one_hot_k,
            request.profile,
        )
        .map(|extended| {
            extended
                .rows()
                .len()
                .saturating_sub(catalogs.one_hot.rows().len())
        })
        .map_err(fail_with(Stage::Planning))
}

/// Bytes that make `edge_biased(min, ..)` return `value` (uniform branch).
fn push_uniform(out: &mut Vec<u8>, min: usize, value: usize) {
    out.push(1);
    out.push((value - min) as u8);
}

impl Shape {
    /// An input prefix that [`Shape::decode`] maps back to `self`.
    ///
    /// Panics on an image size the decoder cannot produce: a power of two up
    /// to `2^34`, one past a power of two up to `2^33 + 1`, or at most `2^20`.
    pub fn encode(&self) -> Vec<u8> {
        let profile = PROFILES
            .iter()
            .position(|profile| *profile == self.profile)
            .unwrap_or_default();
        let mut out = vec![
            match self.chunking {
                Chunking::ForcedK256 => 0,
                Chunking::Production => 1,
            },
            profile as u8,
        ];
        push_uniform(&mut out, MIN_LOG_T, self.log_t);
        push_uniform(&mut out, 0, self.log_bytecode_len);
        push_uniform(&mut out, 1, self.log_ram_k);
        for advice in [self.untrusted_advice_bytes, self.trusted_advice_bytes] {
            out.push(u8::from(advice.is_some()));
            push_uniform(
                &mut out,
                3,
                advice.map_or(3, |bytes| bytes.ilog2() as usize),
            );
        }
        let program = self.program.unwrap_or(CommittedProgram {
            log_chunks: 0,
            image_words: 1,
        });
        out.push(if self.program.is_some() { 0 } else { 1 });
        push_uniform(&mut out, 0, program.log_chunks);
        let words = program.image_words;
        let (kind, log_image, small) = if words.is_power_of_two() {
            (0, words.ilog2() as usize, 0)
        } else if (words - 1).is_power_of_two() && words > 1 << 20 {
            (1, (words - 1).ilog2() as usize, 0)
        } else {
            assert!(words <= 1 << 20, "image of {words} words is not encodable");
            (2, 0, words - 1)
        };
        out.push(kind);
        push_uniform(&mut out, 0, log_image);
        out.extend_from_slice(&(small as u32).to_le_bytes());
        out
    }
}
