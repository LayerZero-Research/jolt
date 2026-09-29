//! Whole guest programs through Jolt's Akita preprocess, prove, and verify
//! (liveness first).
//!
//! Input: a guest from [`crate::programs::GUESTS`], its postcard arguments and
//! advice, and the prover's free choices: full or committed program (and the
//! chunk count), the padded-trace bound baked into preprocessing, a forced
//! K=256 one-hot width, address-first read/write binding, and the optimized
//! or reference backend. Every execution the tracer completes, including a
//! guest panic, is a statement Jolt must prove within these documented
//! limits; a failure anywhere after tracing is a finding.
//!
//! After an accepted proof the target also checks, as secondary soundness
//! and transport properties, that the proof and verifier preprocessing survive
//! bincode transport and still verify, and that a changed public output or
//! panic flag is rejected.

use std::sync::{Arc, OnceLock};

use common::jolt_device::{JoltDevice, MemoryConfig, MemoryLayout};
use jolt_akita::{AkitaCommitment, AkitaField, AkitaScheme};
use jolt_claims::protocols::jolt::JoltOneHotConfig;
use jolt_host::JoltProgramSource;
use jolt_program::execution::{OwnedTrace, TraceInputs};
use jolt_program::preprocess::JoltProgramPreprocessing;
use jolt_prover::akita::preprocessing::{
    commit_trusted_advice, preprocess_committed_with_advice, preprocess_full_with_advice,
    AkitaProverPreprocessing, AkitaTranscript, AkitaVc,
};
use jolt_prover::akita::{self, JoltAkitaBackend};
use jolt_prover::ProverConfig;
use jolt_verifier::proof::JoltProof;
use jolt_witness::{JoltVmWitnessConfig, JoltVmWitnessInputs, TraceBackend};
use tracer::execution_backend::TracerBackend;

use crate::input::Reader;
use crate::programs::{self, Args, Guest, GUESTS};
use crate::{artifacts, env, stats, transport};

pub const GUEST_ENV: &str = "JOLT_FUZZ_GUEST_SET";
/// Traces above this are not proved in one iteration.
pub const MAX_LOG_T_ENV: &str = "JOLT_FUZZ_MAX_LOG_T";

type Proof = JoltProof<AkitaScheme, AkitaVc>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Optimized,
    Reference,
}

/// The prover's free choices for one execution.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// `Some(log2 chunk count)` proves a committed program.
    pub committed: Option<u8>,
    /// Padded-trace bound = padded length << slack (capped at `2^24`).
    pub trace_slack: u8,
    pub force_k256: bool,
    pub ram_address_first: bool,
    pub registers_address_first: bool,
    pub backend: Backend,
}

impl Options {
    pub fn decode(reader: &mut Reader<'_>) -> Self {
        let flags = reader.u8();
        let committed = reader.u8();
        Self {
            committed: (flags & 1 != 0).then_some(committed % 4),
            trace_slack: (flags >> 1) % 3,
            force_k256: flags & 0x08 != 0 && reader.u8() % 4 == 0,
            ram_address_first: flags & 0x10 != 0,
            registers_address_first: flags & 0x20 != 0,
            backend: if flags & 0xc0 == 0xc0 {
                Backend::Reference
            } else {
                Backend::Optimized
            },
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let flags = u8::from(self.committed.is_some())
            | (self.trace_slack % 3) << 1
            | u8::from(self.force_k256) << 3
            | u8::from(self.ram_address_first) << 4
            | u8::from(self.registers_address_first) << 5
            | if self.backend == Backend::Reference {
                0xc0
            } else {
                0
            };
        let mut out = vec![flags, self.committed.unwrap_or(0)];
        if self.force_k256 {
            out.push(0);
        }
        out
    }
}

fn guest_set() -> &'static [&'static Guest] {
    static SET: OnceLock<Vec<&'static Guest>> = OnceLock::new();
    SET.get_or_init(|| match std::env::var(GUEST_ENV).ok().as_deref() {
        None | Some("all") => GUESTS.iter().collect(),
        Some("interp") => GUESTS
            .iter()
            .filter(|g| g.key.starts_with("interp"))
            .collect(),
        Some("examples") => GUESTS
            .iter()
            .filter(|g| !g.key.starts_with("interp"))
            .collect(),
        Some(keys) => keys
            .split(',')
            .map(|key| programs::by_key(key).unwrap_or_else(|| panic!("unknown guest {key}")))
            .collect(),
    })
}

fn max_log_t() -> usize {
    std::env::var(MAX_LOG_T_ENV)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(18)
}

pub fn decode(data: &[u8]) -> (&'static Guest, Options, Args) {
    let mut reader = Reader::new(data);
    // The byte names a guest of the full table, so a seed means the same guest
    // in every lane that includes it; other guests map into the lane's set.
    let selector = usize::from(reader.u8());
    let set = guest_set();
    let named = &GUESTS[selector % GUESTS.len()];
    let guest = if set.iter().any(|guest| std::ptr::eq(*guest, named)) {
        named
    } else {
        set[selector % set.len()]
    };
    let options = Options::decode(&mut reader);
    let args = (guest.args)(&mut reader);
    (guest, options, args)
}

pub fn run(data: &[u8]) {
    env::init();
    let (guest, options, args) = decode(data);
    env::on_large_stack(|| check(guest, &args, &options));
}

fn memory_config(layout: &MemoryLayout) -> MemoryConfig {
    MemoryConfig {
        max_untrusted_advice_size: layout.max_untrusted_advice_size,
        max_trusted_advice_size: layout.max_trusted_advice_size,
        max_input_size: layout.max_input_size,
        max_output_size: layout.max_output_size,
        stack_size: layout.stack_size,
        heap_size: layout.heap_size,
        program_size: Some(layout.program_size),
    }
}

/// A proved execution, as the verifier receives it.
pub struct Proved {
    pub preprocessing: AkitaProverPreprocessing,
    pub public_io: JoltDevice,
    pub proof: Proof,
    pub trusted_advice_commitment: Option<AkitaCommitment>,
}

impl Proved {
    pub fn verify(&self, public_io: &JoltDevice, proof: &Proof) -> Result<(), String> {
        jolt_verifier::verify::<AkitaField, AkitaScheme, AkitaVc, AkitaTranscript>(
            &self.preprocessing.verifier,
            public_io,
            proof,
            self.trusted_advice_commitment.as_ref(),
        )
        .map_err(|error| format!("{error:?}"))
    }
}

/// Trace, preprocess, prove, and verify one execution. `Ok(None)` means the
/// execution lies outside the lane's size budget (not a finding); `Err` is a
/// failure of a provable execution.
pub fn prove(guest: &Guest, args: &Args, options: &Options) -> Result<Option<Proved>, String> {
    let elf = guest.elf_path(&programs::guests_dir());
    assert!(
        elf.is_file(),
        "guest ELF {} is missing: run `jolt-fuzz-dev build-guests`",
        elf.display()
    );
    let mut source = guest.program(Some(elf));
    let (_, sizing_trace, _, device) = stats::time("trace", || {
        source.trace(&args.inputs, &args.untrusted_advice, &args.trusted_advice)
    });
    if device.panic {
        stats::count("guest_panicked");
    }
    let padded = (sizing_trace.len() + 1).next_power_of_two().max(1 << 12);
    if padded.ilog2() as usize > max_log_t() {
        stats::count("trace_over_budget");
        return Ok(None);
    }
    let max_padded_trace_length = (padded << options.trace_slack).min(1 << 24).max(padded);
    let layout = device.memory_layout.clone();
    let program = Arc::new(
        source
            .build_jolt_program()
            .map_err(|error| format!("build_jolt_program: {error:?}"))?,
    );
    let program_preprocessing = JoltProgramPreprocessing::new(
        program.expanded_bytecode.clone(),
        program.memory_init.clone(),
        layout.clone(),
        program.entry_address,
        max_padded_trace_length,
        source.instruction_profile(),
    )
    .map_err(|error| format!("program preprocessing: {error:?}"))?;
    let trace = TracerBackend::new()
        .trace_compact(
            &program,
            TraceInputs::new(
                args.inputs.clone(),
                args.untrusted_advice.clone(),
                args.trusted_advice.clone(),
                memory_config(&layout),
            ),
            &program_preprocessing.bytecode,
        )
        .map_err(|error| format!("modular trace: {error:?}"))?;
    let mut config = ProverConfig::derive_compact::<AkitaField>(
        trace.trace.as_slice(),
        &program_preprocessing.memory_layout,
        program_preprocessing.ram.min_bytecode_address,
        program_preprocessing.ram.bytecode_words.len(),
        program_preprocessing.max_padded_trace_length,
    )
    .map_err(|error| format!("derive config: {error:?}"))?;
    if options.force_k256 {
        config.one_hot_config = JoltOneHotConfig {
            log_k_chunk: 8,
            lookups_ra_virtual_log_k_chunk: 32,
        };
    }
    if options.ram_address_first {
        config.rw_config.ram_rw_phase1_num_rounds = 0;
    }
    if options.registers_address_first {
        config.rw_config.registers_rw_phase1_num_rounds = 0;
    }
    let has_untrusted = !args.untrusted_advice.is_empty();
    let has_trusted = !args.trusted_advice.is_empty();
    let schedule_artifacts = artifacts::shared();
    let bytecode_len = program_preprocessing.bytecode.bytecode.len();
    let preprocessing = stats::time("preprocess", || match options.committed {
        None => preprocess_full_with_advice(
            &schedule_artifacts,
            program_preprocessing,
            &config,
            has_untrusted,
            has_trusted,
        ),
        Some(log_chunks) => {
            // Chunk counts must divide the (power-of-two) bytecode length.
            let chunks =
                1usize << usize::from(log_chunks).min(bytecode_len.trailing_zeros() as usize);
            preprocess_committed_with_advice(
                &schedule_artifacts,
                program_preprocessing,
                &config,
                chunks,
                has_untrusted,
                has_trusted,
            )
        }
    })
    .map_err(|error| format!("preprocess: {error:?}"))?;
    let trusted = has_trusted
        .then(|| commit_trusted_advice(&preprocessing, &args.trusted_advice))
        .transpose()
        .map_err(|error| format!("commit trusted advice: {error:?}"))?;
    let full_program = preprocessing
        .program_arc()
        .ok_or("preprocessing retained no full program")?;
    let public_io = trace.device.clone();
    let witness = TraceBackend::<OwnedTrace>::from_compact(
        JoltVmWitnessConfig::new(
            config.trace_length.ilog2() as usize,
            config.ram_K,
            config.one_hot_config,
        )
        .include_untrusted_advice(has_untrusted)
        .include_trusted_advice(has_trusted),
        JoltVmWitnessInputs::new(&program, &full_program, trace),
    );
    let backend = match options.backend {
        Backend::Optimized => JoltAkitaBackend::optimized(),
        // The reference tier materializes a dense address-by-cycle grid and
        // refuses shapes above 32 GiB ("a test oracle sized for small
        // traces"); larger shapes use the optimized tier.
        Backend::Reference if config.trace_length.ilog2() + config.ram_K.ilog2() <= 30 => {
            JoltAkitaBackend::reference()
        }
        Backend::Reference => {
            stats::count("reference_backend_too_large");
            JoltAkitaBackend::optimized()
        }
    };
    let context = format!("{} {options:?}", guest.key);
    let proof = stats::time("prove", || {
        crate::liveness::observe(&context, || {
            akita::prove::<AkitaField, AkitaScheme, AkitaVc, AkitaTranscript, _>(
                &backend,
                &preprocessing,
                &config,
                trusted.as_ref(),
                &witness,
                &public_io,
            )
        })
    })
    .map_err(|error| format!("prove: {error:?}"))?;
    stats::count("honest_proofs");
    let proved = Proved {
        preprocessing,
        public_io,
        proof,
        trusted_advice_commitment: trusted.map(|object| object.commitment),
    };
    stats::time("verify", || proved.verify(&proved.public_io, &proved.proof))
        .map_err(|error| format!("verify: {error}"))?;
    stats::count("honest_verified");
    Ok(Some(proved))
}

pub fn check(guest: &Guest, args: &Args, options: &Options) {
    let proved = match prove(guest, args, options) {
        Ok(Some(proved)) => proved,
        Ok(None) => return,
        Err(failure) => panic!(
            "liveness: guest {} ({options:?}, inputs {} B, trusted {} B, untrusted {} B) failed: {failure}",
            guest.key,
            args.inputs.len(),
            args.trusted_advice.len(),
            args.untrusted_advice.len()
        ),
    };
    check_transport_and_binding(guest, &proved);
}

fn check_transport_and_binding(guest: &Guest, proved: &Proved) {
    // Transport: the proof and the verifier preprocessing as bytes.
    let proof: Proof = transport::roundtrip(&proved.proof);
    let verifier = transport::roundtrip(&proved.preprocessing.verifier);
    jolt_verifier::verify::<AkitaField, AkitaScheme, AkitaVc, AkitaTranscript>(
        &verifier,
        &proved.public_io,
        &proof,
        proved.trusted_advice_commitment.as_ref(),
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: transported proof and preprocessing failed to verify: {error:?}",
            guest.key
        )
    });
    stats::count("transported_verified");

    // Binding: another public output or panic flag must be rejected.
    let mut changed = proved.public_io.clone();
    match changed.outputs.first_mut() {
        Some(byte) => *byte ^= 1,
        None => changed.outputs.push(1),
    }
    assert!(
        proved.verify(&changed, &proved.proof).is_err(),
        "{}: proof verified against a different public output",
        guest.key
    );
    let mut flipped = proved.public_io.clone();
    flipped.panic = !flipped.panic;
    assert!(
        proved.verify(&flipped, &proved.proof).is_err(),
        "{}: proof verified against the opposite panic flag",
        guest.key
    );
    stats::count("binding_rejected");
}

/// One seed per guest with default options, one committed, one forced K=256.
pub fn seeds() -> Vec<(String, Vec<u8>)> {
    let base = Options {
        committed: None,
        trace_slack: 0,
        force_k256: false,
        ram_address_first: false,
        registers_address_first: false,
        backend: Backend::Optimized,
    };
    let variants = [
        ("default", base),
        (
            "committed2",
            Options {
                committed: Some(1),
                ..base
            },
        ),
        (
            "k256-address-first",
            Options {
                force_k256: true,
                ram_address_first: true,
                registers_address_first: true,
                ..base
            },
        ),
    ];
    let mut seeds = Vec::new();
    for (index, guest) in GUESTS.iter().enumerate() {
        for (name, options) in variants {
            // A known J-1 reproduction (committed program with large advice
            // capacities); as a seed it would stop the lane at startup.
            if guest.key == "interp-advice-large" && options.committed.is_some() {
                continue;
            }
            let mut bytes = vec![index as u8];
            bytes.extend(options.encode());
            bytes.extend(std::iter::repeat_n(0x11u8, 64));
            seeds.push((format!("{}-{name}", guest.key), bytes));
        }
    }
    seeds
}
