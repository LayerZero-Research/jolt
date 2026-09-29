//! The guests the `program` target proves, and how inputs become their
//! arguments.
//!
//! Every guest is built once by `jolt-fuzz-dev build-guests` (the `jolt` CLI,
//! exactly as `jolt_host::Program::build` does it) and shipped as an ELF, so
//! fuzzing never compiles guests. Each entry repeats its `#[jolt::provable]`
//! memory attributes, because the host-side `MemoryConfig` must match the
//! layout compiled into the ELF; `build-guests` builds with these same values.
//! Inputs are postcard-encoded, as the SDK encodes them.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::input::Reader;

/// Which cargo workspace owns the guest package (`jolt build -p` runs there).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Workspace {
    /// The repository root (`examples/*/guest`).
    Root,
    /// `fuzz/guests`.
    Fuzz,
}

/// Postcard-encoded inputs and advice for one execution.
#[derive(Clone, Debug, Default)]
pub struct Args {
    pub inputs: Vec<u8>,
    pub trusted_advice: Vec<u8>,
    pub untrusted_advice: Vec<u8>,
}

pub struct Guest {
    /// Stable name: the ELF file stem and the report label.
    pub key: &'static str,
    pub package: &'static str,
    pub func: Option<&'static str>,
    pub std: bool,
    pub workspace: Workspace,
    pub heap_size: u64,
    pub stack_size: u64,
    pub max_input_size: u64,
    pub max_trusted_advice_size: u64,
    pub max_untrusted_advice_size: u64,
    pub args: fn(&mut Reader<'_>) -> Args,
}

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    postcard::to_stdvec(value).expect("postcard encodes guest arguments")
}

/// Program for the interpreter guest: header (heap words, step budget) then
/// code, with the budget capped so traces stay within one iteration's reach.
pub fn interp_program(reader: &mut Reader<'_>, max_steps: u32) -> Vec<u8> {
    let heap_words = reader.u32();
    let budget = reader.u32() % (max_steps + 1);
    let code_len = usize::from(reader.u16()) % 2048;
    let mut program = heap_words.to_le_bytes().to_vec();
    program.extend(budget.to_le_bytes());
    program.extend(reader.take(code_len));
    program
}

fn bytes_up_to(reader: &mut Reader<'_>, max: usize) -> Vec<u8> {
    let len = usize::from(reader.u16()) % (max + 1);
    reader.take(len).to_vec()
}

/// Steps the interpreter may run: about `2^14`-`2^17` cycles.
pub const INTERP_MAX_STEPS: u32 = 4096;

fn interp_args(reader: &mut Reader<'_>) -> Args {
    let program = interp_program(reader, INTERP_MAX_STEPS);
    let trusted = bytes_up_to(reader, 4000);
    let untrusted = bytes_up_to(reader, 4000);
    Args {
        inputs: encode(&program),
        trusted_advice: encode(&trusted),
        untrusted_advice: encode(&untrusted),
    }
}

fn interp_plain_args(reader: &mut Reader<'_>) -> Args {
    Args {
        inputs: encode(&interp_program(reader, INTERP_MAX_STEPS)),
        ..Args::default()
    }
}

fn interp_advice_large_args(reader: &mut Reader<'_>) -> Args {
    let program = interp_program(reader, INTERP_MAX_STEPS);
    // Advice lengths near the capacities, filled by a repeated pattern.
    let pattern = reader.bytes::<8>();
    let trusted_len = [0, 1, 4096, (1 << 20) - 16][usize::from(reader.u8() % 4)];
    let untrusted_len = [0, 1, 4096, (1 << 17) - 16][usize::from(reader.u8() % 4)];
    let fill = |len: usize| {
        (0..len)
            .map(|index| pattern[index % 8])
            .collect::<Vec<u8>>()
    };
    Args {
        inputs: encode(&program),
        trusted_advice: encode(&fill(trusted_len)),
        untrusted_advice: encode(&fill(untrusted_len)),
    }
}

fn muldiv_args(reader: &mut Reader<'_>) -> Args {
    Args {
        inputs: encode(&(reader.u32(), reader.u32(), reader.u32())),
        ..Args::default()
    }
}

fn fib_args(reader: &mut Reader<'_>) -> Args {
    Args {
        inputs: encode(&(reader.u32() % 2000)),
        ..Args::default()
    }
}

fn collatz_args(reader: &mut Reader<'_>) -> Args {
    Args {
        inputs: encode(&(1 + u128::from(reader.u16()))),
        ..Args::default()
    }
}

fn bytes_args(reader: &mut Reader<'_>) -> Args {
    Args {
        inputs: encode(&bytes_up_to(reader, 2048)),
        ..Args::default()
    }
}

fn sha3_aligned_args(reader: &mut Reader<'_>) -> Args {
    let mut blocks = [[0u64; 17]; 2];
    for lane in blocks.iter_mut().flatten() {
        *lane = reader.u64();
    }
    Args {
        inputs: encode(&blocks),
        ..Args::default()
    }
}

fn chain_args(reader: &mut Reader<'_>) -> Args {
    Args {
        inputs: encode(&(reader.bytes::<32>(), reader.u32() % 8)),
        ..Args::default()
    }
}

fn btreemap_args(reader: &mut Reader<'_>) -> Args {
    Args {
        inputs: encode(&(reader.u32() % 64)),
        ..Args::default()
    }
}

fn advice_consumer_args(reader: &mut Reader<'_>) -> Args {
    // The guest asserts `trusted + untrusted == public_sum`; a mismatch is a
    // guest panic, which is still a statement Jolt must prove.
    let (trusted, untrusted) = (reader.u64() >> 2, reader.u64() >> 2);
    let sum = if reader.u8().is_multiple_of(8) {
        reader.u64()
    } else {
        trusted + untrusted
    };
    Args {
        inputs: encode(&sum),
        trusted_advice: encode(&trusted),
        untrusted_advice: encode(&untrusted),
    }
}

fn two_u32_args(reader: &mut Reader<'_>) -> Args {
    Args {
        inputs: encode(&(reader.u32(), reader.u32())),
        ..Args::default()
    }
}

fn u32_small_args(reader: &mut Reader<'_>) -> Args {
    Args {
        inputs: encode(&(reader.u32() % 10_000)),
        ..Args::default()
    }
}

fn no_args(_: &mut Reader<'_>) -> Args {
    Args::default()
}

const KIB: u64 = 1024;
const MIB: u64 = 1024 * KIB;

const fn example(
    key: &'static str,
    package: &'static str,
    func: Option<&'static str>,
    heap_size: u64,
    args: fn(&mut Reader<'_>) -> Args,
) -> Guest {
    Guest {
        key,
        package,
        func,
        std: false,
        workspace: Workspace::Root,
        heap_size,
        stack_size: 4096,
        max_input_size: 4096,
        max_trusted_advice_size: 4096,
        max_untrusted_advice_size: 4096,
        args,
    }
}

const fn interp(
    key: &'static str,
    func: &'static str,
    heap_size: u64,
    max_trusted_advice_size: u64,
    max_untrusted_advice_size: u64,
    args: fn(&mut Reader<'_>) -> Args,
) -> Guest {
    Guest {
        key,
        package: "jolt-fuzz-interp-guest",
        func: Some(func),
        std: false,
        workspace: Workspace::Fuzz,
        heap_size,
        stack_size: 64 * KIB,
        max_input_size: 4096,
        max_trusted_advice_size,
        max_untrusted_advice_size,
        args,
    }
}

pub const GUESTS: &[Guest] = &[
    interp("interp", "interp", MIB, 4096, 4096, interp_args),
    interp(
        "interp-plain",
        "interp_plain",
        MIB,
        4096,
        4096,
        interp_plain_args,
    ),
    interp(
        "interp-advice-large",
        "interp_advice_large",
        MIB,
        64 * MIB,
        MIB,
        interp_advice_large_args,
    ),
    interp(
        "interp-big-heap",
        "interp_big_heap",
        256 * MIB,
        4096,
        4096,
        interp_plain_args,
    ),
    example("muldiv", "muldiv-guest", None, 32 * KIB, muldiv_args),
    example("fibonacci", "fibonacci-guest", None, 32 * KIB, fib_args),
    example(
        "collatz",
        "collatz-guest",
        Some("collatz_convergence"),
        32 * KIB,
        collatz_args,
    ),
    example("sha2", "sha2-guest", None, 32 * KIB, bytes_args),
    example("sha3", "sha3-guest", Some("sha3"), 32 * KIB, bytes_args),
    example(
        "sha3-aligned",
        "sha3-guest",
        Some("sha3_aligned"),
        32 * KIB,
        sha3_aligned_args,
    ),
    example("sha2-chain", "sha2-chain-guest", None, 32 * KIB, chain_args),
    example("sha3-chain", "sha3-chain-guest", None, 32 * KIB, chain_args),
    Guest {
        stack_size: 10000,
        ..example(
            "btreemap",
            "btreemap-guest",
            None,
            10_000_000,
            btreemap_args,
        )
    },
    example("memory-ops", "memory-ops-guest", None, 64 * KIB, no_args),
    example(
        "advice-consumer",
        "advice-consumer-guest",
        None,
        32 * KIB,
        advice_consumer_args,
    ),
    example("alloc", "alloc-guest", None, 32 * KIB, u32_small_args),
    example("random", "random-guest", None, 64 * KIB, two_u32_args),
];

pub fn by_key(key: &str) -> Option<&'static Guest> {
    GUESTS.iter().find(|guest| guest.key == key)
}

pub const GUESTS_ENV: &str = "JOLT_FUZZ_GUESTS";

/// Directory of built guest ELFs (`<key>.elf`).
pub fn guests_dir() -> PathBuf {
    std::env::var_os(GUESTS_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/guests"))
}

impl Guest {
    pub fn elf_path(&self, dir: &Path) -> PathBuf {
        dir.join(format!("{}.elf", self.key))
    }

    /// The host-side program description, bound to its prebuilt ELF when one
    /// is given (otherwise `build` compiles it with the `jolt` CLI).
    pub fn program(&self, elf: Option<PathBuf>) -> jolt_host::Program {
        let mut program = jolt_host::Program::new(self.package);
        if let Some(func) = self.func {
            program.set_func(func);
        }
        program.set_std(self.std);
        program.set_heap_size(self.heap_size);
        program.set_stack_size(self.stack_size);
        program.set_max_input_size(self.max_input_size);
        program.set_max_trusted_advice_size(self.max_trusted_advice_size);
        program.set_max_untrusted_advice_size(self.max_untrusted_advice_size);
        program.elf = elf;
        program
    }
}

/// The repository root this harness was built from.
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

impl Workspace {
    pub fn dir(self) -> PathBuf {
        match self {
            Workspace::Root => repo_root(),
            Workspace::Fuzz => repo_root().join("fuzz/guests"),
        }
    }
}

/// Build every guest with the `jolt` CLI and copy its ELF to `out/<key>.elf`.
/// Changes the process directory (the CLI resolves `-p` in the current
/// workspace), so it is for single-threaded tools only.
pub fn build_all(out: &Path) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| format!("create {}: {e}", out.display()))?;
    let target_dir = repo_root().join("fuzz/target/guest-build");
    let target_dir = target_dir.to_str().ok_or("non-UTF-8 target directory")?;
    for guest in GUESTS {
        std::env::set_current_dir(guest.workspace.dir())
            .map_err(|e| format!("enter {:?} workspace: {e}", guest.workspace))?;
        let mut program = guest.program(None);
        eprintln!(
            "building {} ({}{})",
            guest.key,
            guest.package,
            guest.func.map(|f| format!("::{f}")).unwrap_or_default()
        );
        program.build(target_dir);
        let elf = program
            .elf
            .clone()
            .ok_or_else(|| format!("{}: build produced no ELF", guest.key))?;
        std::fs::copy(&elf, guest.elf_path(out))
            .map_err(|e| format!("copy {}: {e}", elf.display()))?;
    }
    Ok(())
}
