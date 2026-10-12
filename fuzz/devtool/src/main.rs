//! `jolt-fuzz-dev`: uninstrumented developer commands over the harness
//! (`jolt-fuzz-dev --help` lists them).

mod grid_sweep;
mod sweep;

use clap::{Args, Parser, Subcommand, ValueEnum};
use jolt_akita::AkitaChunkProfile;
use jolt_akita_fuzz::env::MAX_CASE_COEFFS_ENV;
use jolt_akita_fuzz::input::SplitMix64;
use jolt_akita_fuzz::targets::{self, ALL};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn list() {
    for (name, _) in ALL {
        println!("{name}");
    }
}

fn random_bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut rng = SplitMix64::new(seed);
    (0..len).map(|_| rng.next_u64() as u8).collect()
}

/// Run `count` pseudo-random inputs, saving each panicking one to
/// `smoke-failures/<target>-<index>.bin` and continuing.
fn smoke(name: &str, count: usize, seed: u64) -> Result<(), String> {
    let run = targets::by_name(name).ok_or_else(|| format!("unknown target {name}"))?;
    let mut rng = SplitMix64::new(seed);
    let started = Instant::now();
    let mut failures = 0usize;
    for index in 0..count {
        let len = (rng.next_u64() % 4096) as usize;
        let data = random_bytes(rng.next_u64(), len);
        let one = Instant::now();
        if std::panic::catch_unwind(|| run(&data)).is_err() {
            failures += 1;
            let dir = Path::new("smoke-failures");
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            let path = dir.join(format!("{name}-{seed}-{index}.bin"));
            std::fs::write(&path, &data).map_err(|e| e.to_string())?;
            eprintln!("input {index}: FAILED, saved {}", path.display());
        } else if index < 3 || one.elapsed().as_secs_f64() > 5.0 {
            eprintln!("input {index}: {:.3}s", one.elapsed().as_secs_f64());
        }
    }
    eprintln!(
        "{name}: {count} inputs in {:.2}s, {failures} failed",
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn replay(name: &str, inputs: &[PathBuf]) -> Result<(), String> {
    let run = targets::by_name(name).ok_or_else(|| format!("unknown target {name}"))?;
    for path in inputs {
        let data = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let started = Instant::now();
        run(&data);
        eprintln!(
            "{}: ok in {:.3}s",
            path.display(),
            started.elapsed().as_secs_f64()
        );
    }
    Ok(())
}

/// Per target: an all-zero input, an all-`0xff` input, 32 pseudo-random ones,
/// and whatever structured seeds the target itself provides.
fn seeds(out: &Path) -> Result<(), String> {
    for (name, _) in ALL {
        let dir = out.join(name);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let write = |file: &str, bytes: &[u8]| {
            std::fs::write(dir.join(file), bytes).map_err(|e| e.to_string())
        };
        write("zeros", &[0u8; 512])?;
        write("ones", &[0xffu8; 512])?;
        for index in 0..32u64 {
            let len = [64, 256, 1024, 4096][index as usize % 4];
            write(
                &format!("random-{index:02}"),
                &random_bytes(0x5eed_0000 + index, len),
            )?;
        }
        for (file, bytes) in targets::seeds(name) {
            write(&file, &bytes)?;
        }
    }
    Ok(())
}

/// The production shape a `planning` input decodes to, and the grouped
/// request it induces, timing grouped-row provisioning alone.
fn describe_planning(paths: &[PathBuf]) -> Result<(), String> {
    use jolt_akita_fuzz::input::Reader;
    use jolt_akita_fuzz::shape::{self, Shape};
    jolt_akita_fuzz::env::init();
    for path in paths {
        let data = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let shape = Shape::decode(&mut Reader::new(&data));
        println!(
            "== {}\n{shape} (in contract: {})",
            path.display(),
            shape.in_contract()
        );
        match shape.setup_request() {
            Err(failure) => println!("  setup request: {failure}"),
            Ok(request) => {
                println!(
                    "  final group: {} vars x {} polys, K={}, profile {:?}",
                    request.setup_shape.num_vars,
                    request.setup_shape.num_polys,
                    request.one_hot_k,
                    request.profile
                );
                println!(
                    "  untrusted {:?}, trusted {:?}, program {:?}",
                    request.untrusted.as_ref().map(shape::arity),
                    request.trusted.as_ref().map(shape::arity),
                    request.program.iter().map(shape::arity).collect::<Vec<_>>()
                );
                let started = Instant::now();
                let planned = shape::plan(&request);
                println!(
                    "  planning: {} in {:.1}s",
                    match planned {
                        Ok(rows) => format!("ok, {rows} rows provisioned"),
                        Err(failure) => failure.to_string(),
                    },
                    started.elapsed().as_secs_f64()
                );
            }
        }
    }
    Ok(())
}

fn explain_verifier(paths: &[PathBuf]) -> Result<(), String> {
    for path in paths {
        let data = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        println!(
            "== {}\n{}",
            path.display(),
            targets::verifier::explain(&data)
        );
    }
    Ok(())
}

/// `planning-case` K: the one-hot chunking.
#[derive(Clone, Copy, ValueEnum)]
enum ChunkingArg {
    /// `ProverConfig::derive`'s rule under Akita (K=16).
    #[value(name = "16", alias = "p")]
    Production,
    /// The K=256 override.
    #[value(name = "256")]
    ForcedK256,
}

/// `planning-case` PROFILE: the Akita chunk profile.
#[derive(Clone, Copy, ValueEnum)]
enum ProfileArg {
    Single,
    Two,
    Four,
    Eight,
}

/// An advice capacity in bytes; `None` for no advice.
#[derive(Clone, Copy)]
struct Advice(Option<u64>);

fn advice(value: &str) -> Result<Advice, String> {
    match value {
        "-" => Ok(Advice(None)),
        value => value
            .parse::<u32>()
            .ok()
            .and_then(|log| 1u64.checked_shl(log))
            .map(|bytes| Advice(Some(bytes)))
            .ok_or_else(|| format!("expected log2 bytes (0..=63) or `-`, got {value:?}")),
    }
}

#[derive(Args)]
struct PlanningCase {
    log_t: usize,
    #[arg(value_enum)]
    k: ChunkingArg,
    #[arg(value_enum)]
    profile: ProfileArg,
    log_bytecode: usize,
    log_ram_k: usize,
    /// Untrusted advice: log2 bytes, or `-` for none.
    #[arg(value_parser = advice)]
    u: Advice,
    /// Trusted advice: log2 bytes, or `-` for none.
    #[arg(value_parser = advice)]
    t: Advice,
    /// Committed-program chunks (log2); requires IMAGE.
    #[arg(requires = "image")]
    chunks: Option<usize>,
    /// Committed-program image words.
    image: Option<usize>,
}

fn planning_case(case: PlanningCase) -> Result<(), String> {
    use jolt_akita_fuzz::opening::{Fill, Witness};
    use jolt_akita_fuzz::shape::{Chunking, CommittedProgram, Shape};
    let shape = Shape {
        log_t: case.log_t,
        chunking: match case.k {
            ChunkingArg::Production => Chunking::Production,
            ChunkingArg::ForcedK256 => Chunking::ForcedK256,
        },
        profile: match case.profile {
            ProfileArg::Single => AkitaChunkProfile::Single,
            ProfileArg::Two => AkitaChunkProfile::Two,
            ProfileArg::Four => AkitaChunkProfile::Four,
            ProfileArg::Eight => AkitaChunkProfile::Eight,
        },
        log_bytecode_len: case.log_bytecode,
        log_ram_k: case.log_ram_k,
        untrusted_advice_bytes: case.u.0,
        trusted_advice_bytes: case.t.0,
        program: case
            .chunks
            .zip(case.image)
            .map(|(log_chunks, image_words)| CommittedProgram {
                log_chunks,
                image_words,
            }),
    };
    std::env::set_var(MAX_CASE_COEFFS_ENV, u128::MAX.to_string());
    jolt_akita_fuzz::env::init();
    eprintln!("{shape} (in contract: {})", shape.in_contract());
    let started = Instant::now();
    let witness = Witness {
        seed: 1,
        dense: Fill::Random,
        trace: Fill::Random,
        zero_committed_columns: 1,
        point: Fill::Random,
    };
    targets::planning::check(&shape, &witness);
    eprintln!("ok in {:.1}s", started.elapsed().as_secs_f64());
    Ok(())
}

#[derive(Parser)]
#[command(
    name = "jolt-fuzz-dev",
    about = "Uninstrumented developer commands over the fuzz harness"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Library target names.
    List,
    /// Deterministic seed corpora, one directory per target.
    Seeds { out_dir: PathBuf },
    /// Pseudo-random inputs without libFuzzer; failing inputs are saved to
    /// `smoke-failures/`.
    Smoke {
        target: String,
        #[arg(default_value_t = 100)]
        n: usize,
        #[arg(default_value_t = 1)]
        seed: u64,
    },
    /// Run inputs once.
    Replay {
        target: String,
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Exhaustive preprocessing-planning sweep.
    PlanSweep {
        #[arg(default_value = "all", value_parser = ["geometry", "advice", "program", "all"])]
        phase: String,
        csv: Option<PathBuf>,
    },
    /// Build every guest ELF (jolt CLI).
    BuildGuests { out_dir: PathBuf },
    /// Honest verifier bundles (needs built guests).
    Bundles { out_dir: PathBuf },
    /// Plan, commit, prove, and verify one shape with no size cap.
    PlanningCase(PlanningCase),
    /// Decode planning inputs to their production shape and time grouped
    /// provisioning alone.
    DescribePlanning {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Which decoded fields a verifier input changes.
    ExplainVerifier {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Every catalog row with MIN..=MAX variables through the grid target.
    GridSweep {
        #[arg(default_value_t = 0)]
        min: usize,
        #[arg(default_value_t = 22)]
        max: usize,
        #[arg(value_parser = ["k16", "k256", "dense"])]
        family: Option<String>,
        /// Only rows of at least 2^N committed coefficients (all polynomials).
        #[arg(long, value_name = "N")]
        min_log_coeffs: Option<u32>,
        /// Only rows of at most 2^N committed coefficients (all polynomials).
        #[arg(long, value_name = "N")]
        max_log_coeffs: Option<u32>,
    },
}

fn main() {
    let result = match Cli::parse().command {
        Cmd::List => {
            list();
            Ok(())
        }
        Cmd::Seeds { out_dir } => seeds(&out_dir),
        Cmd::Smoke { target, n, seed } => smoke(&target, n, seed),
        Cmd::Replay { target, files } => replay(&target, &files),
        Cmd::BuildGuests { out_dir } => jolt_akita_fuzz::programs::build_all(&out_dir),
        Cmd::Bundles { out_dir } => targets::verifier::write_bundles(&out_dir),
        Cmd::ExplainVerifier { files } => explain_verifier(&files),
        Cmd::DescribePlanning { files } => describe_planning(&files),
        Cmd::PlanningCase(case) => planning_case(case),
        Cmd::GridSweep {
            min,
            max,
            family,
            min_log_coeffs,
            max_log_coeffs,
        } => {
            let coeffs = min_log_coeffs.map_or(0, |n| 1u128 << n)
                ..=max_log_coeffs.map_or(u128::MAX, |n| 1u128 << n);
            grid_sweep::run(min..=max, coeffs, family.as_deref())
        }
        Cmd::PlanSweep { phase, csv } => sweep::run(&phase, csv),
    };
    if let Err(message) = result {
        eprintln!("jolt-fuzz-dev: {message}");
        std::process::exit(2);
    }
}
