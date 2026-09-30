//! Production preprocessing shapes and the Akita setup request each induces.
//!
//! A [`Shape`] is what a Jolt deployment actually chooses: the padded trace
//! length, the one-hot chunk width, the bytecode and RAM domains, the advice
//! capacities baked into the guest's memory layout, and whether the program is
//! committed (and in how many bytecode chunks). Every derived quantity comes
//! from Jolt's own geometry functions (`one_hot_trace_setup_shape`,
//! `advice_packing_plan`, `committed_program_packing_plan`), so the harness
//! restates no sizing law; the only mirrored code is the ten-line assembly of
//! `PrecommittedScheduleParams` in `jolt_prover::akita::preprocessing::
//! grouped_setup`, which is private. The `program` target exercises that
//! private path itself on real guests.

use std::fmt;

use common::constants::ONEHOT_CHUNK_THRESHOLD_LOG_T;
use jolt_akita::schedule_registry::provision_precommitted_for_k;
use jolt_akita::{AkitaScheduleArtifacts, PrecommittedScheduleParams};
use jolt_claims::protocols::jolt::geometry::claim_reductions::bytecode::committed_lane_vars;
use jolt_claims::protocols::jolt::lattice::{
    advice_packing_plan, committed_program_packing_plan, OneHotTraceSetupShape,
    PrefixPackedObjectPlan, ADVICE_MAX_PHYSICAL_VARS, DIRECT_PROGRAM_MAX_PHYSICAL_VARS,
};
use jolt_claims::protocols::jolt::{
    JoltAdviceKind, JoltOneHotConfig, JoltReadWriteConfig, TracePolynomialOrder,
};
use jolt_openings::PrecommittedRole;
use jolt_prover::akita::one_hot_trace_setup_shape;
use jolt_prover::ProverConfig;

use crate::input::Reader;

/// Smallest padded trace under Akita (`MIN_PADDED_TRACE_LENGTH`, 2^12).
pub const MIN_LOG_T: usize = 12;
/// Largest trace a one-hot catalog covers: K=16 rows reach `log_T + 10 = 34`,
/// K=256 rows reach `log_T + 13 = 43` (`schedules::emit::K*_NUM_VARS`).
pub const MAX_LOG_T_K16: usize = 24;
pub const MAX_LOG_T_K256: usize = 30;
/// `MAX_COMMITTED_BYTECODE_CHUNK_COUNT`.
pub const MAX_LOG_CHUNKS: usize = 8;

/// Which one-hot chunk width the trace commits with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chunking {
    /// `ProverConfig::derive`'s rule: K=16 below `log_T = 25`, K=256 above.
    Production,
    /// A caller-overridden width (supported: `akita_e2e` forces K=256).
    Forced { log_k_chunk: u8 },
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
    pub log_bytecode_len: usize,
    pub log_ram_k: usize,
    /// Advice capacities in bytes (`MemoryLayout::max_*_advice_size`, a
    /// power of two); `None` when the program takes no such advice.
    pub untrusted_advice_bytes: Option<u64>,
    pub trusted_advice_bytes: Option<u64>,
    pub program: Option<CommittedProgram>,
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "log_T={} K=2^{} bytecode=2^{} ram_K=2^{}",
            self.log_t,
            self.log_k_chunk(),
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

/// The request `grouped_setup` hands to `AkitaScheme::setup`.
#[derive(Clone, Debug)]
pub struct SetupRequest {
    pub setup_shape: OneHotTraceSetupShape,
    pub layout_digest: [u8; 32],
    pub one_hot_k: usize,
    pub untrusted: Option<PrefixPackedObjectPlan>,
    pub trusted: Option<PrefixPackedObjectPlan>,
    pub program: Vec<PrefixPackedObjectPlan>,
}

impl SetupRequest {
    /// Precommitted objects in canonical opening order (advice, then program).
    pub fn precommitted(&self) -> impl Iterator<Item = &PrefixPackedObjectPlan> {
        self.untrusted
            .iter()
            .chain(self.trusted.iter())
            .chain(self.program.iter())
    }

    pub fn precommitted_count(&self) -> usize {
        self.precommitted().count()
    }

    pub fn schedule_params(&self) -> Option<PrecommittedScheduleParams> {
        let arity = |plan: &PrefixPackedObjectPlan| plan.packing().packed_num_vars();
        (self.precommitted_count() > 0).then(|| {
            PrecommittedScheduleParams::new(
                self.untrusted.as_ref().map(arity),
                self.trusted.as_ref().map(arity),
                self.setup_shape.num_vars,
            )
            .with_direct_program_physical_arities(self.program.iter().map(arity).collect())
        })
    }

    pub fn roles(&self) -> Vec<PrecommittedRole> {
        self.precommitted()
            .map(PrefixPackedObjectPlan::precommitted_role)
            .collect()
    }

    /// Total committed coefficients across every group of the opening.
    pub fn total_coefficients(&self) -> u128 {
        let final_group = 1u128 << self.setup_shape.num_vars;
        self.precommitted()
            .map(|plan| 1u128 << plan.packing().packed_num_vars())
            .sum::<u128>()
            + final_group
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
    /// Grouped-row planning (`provision_precommitted_for_k`).
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

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.stage, self.message)
    }
}

fn fail(stage: Stage) -> impl FnOnce(String) -> Failure {
    move |message| Failure { stage, message }
}

impl Shape {
    pub fn log_k_chunk(&self) -> u8 {
        match self.chunking {
            Chunking::Production if self.log_t < ONEHOT_CHUNK_THRESHOLD_LOG_T => 4,
            Chunking::Production => 8,
            Chunking::Forced { log_k_chunk } => log_k_chunk,
        }
    }

    /// The configuration `ProverConfig::derive` would produce, with the
    /// one-hot width replaced when forced.
    pub fn prover_config(&self) -> ProverConfig {
        let log_k_chunk = self.log_k_chunk();
        ProverConfig {
            trace_length: 1 << self.log_t,
            ram_K: 1 << self.log_ram_k,
            rw_config: JoltReadWriteConfig {
                ram_rw_phase1_num_rounds: self.log_t as u8,
                ram_rw_phase2_num_rounds: self.log_ram_k as u8,
                registers_rw_phase1_num_rounds: self.log_t as u8,
                registers_rw_phase2_num_rounds: 7,
            },
            one_hot_config: JoltOneHotConfig {
                log_k_chunk,
                // 128-bit lookup keys: 16-bit virtual chunks with K=16, 32-bit with K=256.
                lookups_ra_virtual_log_k_chunk: if log_k_chunk == 4 { 16 } else { 32 },
            },
            trace_polynomial_order: TracePolynomialOrder::CycleMajor,
        }
    }

    /// Whether every documented Jolt limit admits this shape. A shape inside
    /// the contract must preprocess, commit, prove, and verify; anything that
    /// stops it is a liveness finding. Limits (each with its source):
    ///
    /// - `log_T` in `12..=24` for K=16 and `12..=30` for K=256: the Akita
    ///   minimum padded trace and the one-hot catalog grids;
    /// - advice capacities a power of two (`MemoryLayout::new` asserts it)
    ///   whose physical arity is at most 34 (`ADVICE_MAX_PHYSICAL_VARS`);
    /// - a committed program with a power-of-two chunk count at most 256
    ///   dividing the bytecode length, and chunk and image arities at most 34
    ///   (`DIRECT_PROGRAM_MAX_PHYSICAL_VARS`, `precommitted_packing_plan`).
    pub fn in_contract(&self) -> bool {
        let max_log_t = match self.log_k_chunk() {
            4 => MAX_LOG_T_K16,
            8 => MAX_LOG_T_K256,
            _ => return false,
        };
        let advice_ok = |bytes: Option<u64>| {
            bytes.is_none_or(|bytes| {
                bytes.is_power_of_two() && advice_word_vars(bytes) <= ADVICE_MAX_PHYSICAL_VARS
            })
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
                    .is_some_and(|words| words.ilog2() as usize <= 34)
        });
        (MIN_LOG_T..=max_log_t).contains(&self.log_t)
            && advice_ok(self.untrusted_advice_bytes)
            && advice_ok(self.trusted_advice_bytes)
            && program_ok
    }

    /// Everything `grouped_setup` derives before calling `AkitaScheme::setup`.
    pub fn setup_request(&self) -> Result<SetupRequest, Failure> {
        let bytecode_len = 1usize << self.log_bytecode_len;
        let (setup_shape, layout_digest, one_hot_k) =
            one_hot_trace_setup_shape(&self.prover_config(), bytecode_len)
                .map_err(|error| fail(Stage::TraceGeometry)(error.to_string()))?;
        let advice = |kind: JoltAdviceKind, bytes: Option<u64>| {
            bytes
                .map(|bytes| {
                    advice_packing_plan(kind, advice_word_vars(bytes))
                        .map_err(|error| fail(Stage::AdvicePacking)(error.to_string()))
                })
                .transpose()
        };
        let untrusted = advice(JoltAdviceKind::Untrusted, self.untrusted_advice_bytes)?;
        let trusted = advice(JoltAdviceKind::Trusted, self.trusted_advice_bytes)?;
        let program = match self.program {
            None => Vec::new(),
            Some(program) => committed_program_packing_plan(
                bytecode_len,
                1 << program.log_chunks,
                program.image_words,
                TracePolynomialOrder::CycleMajor,
            )
            .map_err(|error| fail(Stage::ProgramPacking)(error.to_string()))?
            .objects()
            .cloned()
            .collect(),
        };
        Ok(SetupRequest {
            setup_shape,
            layout_digest,
            one_hot_k,
            untrusted,
            trusted,
            program,
        })
    }

    /// Decode a shape biased toward the documented edges.
    pub fn decode(reader: &mut Reader<'_>) -> Self {
        let chunking = match reader.u8() % 8 {
            0 => Chunking::Forced { log_k_chunk: 8 },
            1 => Chunking::Forced { log_k_chunk: 4 },
            _ => Chunking::Production,
        };
        let log_t = edge_biased(reader, MIN_LOG_T, MAX_LOG_T_K256, &[12, 20, 21, 24, 25, 30]);
        let log_bytecode_len = edge_biased(reader, 0, 26, &[4, 8, 16, 20, 24]);
        let log_ram_k = edge_biased(reader, 1, 40, &[16, 20, 22, 24, 28, 32]);
        let advice = |reader: &mut Reader<'_>| {
            let tag = reader.u8();
            (!tag.is_multiple_of(3)).then(|| {
                let log_bytes = edge_biased(reader, 3, 40, &[3, 12, 17, 20, 26, 37]);
                1u64 << log_bytes
            })
        };
        let untrusted_advice_bytes = advice(reader);
        let trusted_advice_bytes = advice(reader);
        let program = reader.u8().is_multiple_of(2).then(|| CommittedProgram {
            log_chunks: edge_biased(reader, 0, MAX_LOG_CHUNKS, &[0, 1, 7, 8]),
            image_words: match reader.u8() % 4 {
                0 => 1 << edge_biased(reader, 0, 34, &[0, 13, 14, 20]),
                1 => (1 << edge_biased(reader, 0, 33, &[13, 14])) + 1,
                _ => 1 + (reader.u32() as usize % (1 << 20)),
            },
        });
        Self {
            log_t,
            chunking,
            log_bytecode_len,
            log_ram_k,
            untrusted_advice_bytes,
            trusted_advice_bytes,
            program,
        }
    }
}

/// `advice_physical_num_vars`'s word count: `log2(next_pow2(bytes / 8))`.
pub fn advice_word_vars(max_bytes: u64) -> usize {
    ((max_bytes / 8) as usize).next_power_of_two().ilog2() as usize
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

/// Grouped-row planning alone: the part of `AkitaScheme::setup` that depends
/// on the shape's precommitted objects. Cheap enough to sweep exhaustively.
pub fn plan(artifacts: &AkitaScheduleArtifacts, request: &SetupRequest) -> Result<usize, Failure> {
    if request.precommitted_count() == 0 {
        return Ok(0);
    }
    let dense = artifacts
        .dense_catalog()
        .map_err(|error| fail(Stage::Planning)(error.to_string()))?;
    let one_hot = artifacts
        .one_hot_catalog(request.one_hot_k)
        .map_err(|error| fail(Stage::Planning)(error.to_string()))?;
    let arity = |plan: &PrefixPackedObjectPlan| plan.packing().packed_num_vars();
    let program: Vec<usize> = request.program.iter().map(arity).collect();
    provision_precommitted_for_k(
        &dense,
        &one_hot,
        request.untrusted.as_ref().map(arity),
        request.trusted.as_ref().map(arity),
        &program,
        request.one_hot_k,
        request.setup_shape.num_vars,
    )
    .map(|rows| rows.rows().len())
    .map_err(|error| fail(Stage::Planning)(error.to_string()))
}

/// Bytes that make `edge_biased(min, ..)` return `value` (uniform branch).
fn push_uniform(out: &mut Vec<u8>, min: usize, value: usize) {
    out.push(1);
    out.push((value - min) as u8);
}

impl Shape {
    /// An input prefix that [`Shape::decode`] maps back to `self`.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![match self.chunking {
            Chunking::Forced { log_k_chunk: 8 } => 0,
            Chunking::Forced { .. } => 1,
            Chunking::Production => 2,
        }];
        push_uniform(&mut out, MIN_LOG_T, self.log_t);
        push_uniform(&mut out, 0, self.log_bytecode_len);
        push_uniform(&mut out, 1, self.log_ram_k);
        for advice in [self.untrusted_advice_bytes, self.trusted_advice_bytes] {
            match advice {
                None => out.push(0),
                Some(bytes) => {
                    out.push(1);
                    push_uniform(&mut out, 3, bytes.ilog2() as usize);
                }
            }
        }
        match self.program {
            None => out.push(1),
            Some(program) => {
                out.push(0);
                push_uniform(&mut out, 0, program.log_chunks);
                let words = program.image_words;
                if words.is_power_of_two() {
                    out.push(0);
                    push_uniform(&mut out, 0, words.ilog2() as usize);
                } else if words <= 1 << 20 {
                    out.push(2);
                    out.extend_from_slice(&((words - 1) as u32).to_le_bytes());
                } else {
                    out.push(1);
                    push_uniform(&mut out, 0, (words - 1).ilog2() as usize);
                }
            }
        }
        out
    }
}
