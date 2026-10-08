//! `jolt-fuzz-dev`: uninstrumented developer commands over the harness.
//!
//! ```text
//! jolt-fuzz-dev list                       library target names
//! jolt-fuzz-dev seeds OUT_DIR              deterministic seed corpora
//! jolt-fuzz-dev smoke TARGET [N] [SEED]    pseudo-random inputs, no libFuzzer
//! jolt-fuzz-dev replay TARGET FILE...      run inputs once
//! jolt-fuzz-dev plan-sweep [PHASE] [CSV]   exhaustive preprocessing-planning
//!                                          sweep (phases: geometry, advice,
//!                                          program, all)
//! jolt-fuzz-dev build-guests OUT_DIR      build every guest ELF (jolt CLI)
//! jolt-fuzz-dev bundles OUT_DIR           honest verifier bundles (needs guests)
//! jolt-fuzz-dev planning-case LOG_T K LOG_BYTECODE LOG_RAM_K U T [CHUNKS IMAGE]
//!                                          plan, commit, prove, and verify one
//!                                          shape with no size cap (U/T: log2
//!                                          advice bytes or `-`; K: 16, 256, or
//!                                          `p` for production)
//! jolt-fuzz-dev explain-verifier FILE...  which decoded fields a verifier input changes
//! jolt-fuzz-dev grid-sweep [MIN] [MAX] [FAMILY]
//!                                          every catalog row with MIN..=MAX
//!                                          variables (k16, k256, dense)
//! ```

mod grid_sweep;
mod sweep;

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
fn describe_planning(paths: &[String]) -> Result<(), String> {
    use jolt_akita_fuzz::input::Reader;
    use jolt_akita_fuzz::shape::{self, Shape};
    jolt_akita_fuzz::env::init();
    for path in paths {
        let data = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
        let shape = Shape::decode(&mut Reader::new(&data));
        println!("== {path}\n{shape} (in contract: {})", shape.in_contract());
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

fn explain_verifier(paths: &[String]) -> Result<(), String> {
    for path in paths {
        let data = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
        println!("== {path}\n{}", targets::verifier::explain(&data));
    }
    Ok(())
}

fn planning_case(args: &[String]) -> Result<(), String> {
    use jolt_akita_fuzz::opening::{Fill, Witness};
    use jolt_akita_fuzz::shape::{Chunking, CommittedProgram, Shape, PROFILES};
    const USAGE: &str = "planning-case LOG_T K PROFILE LOG_BYTECODE LOG_RAM_K U T [CHUNKS IMAGE]";
    let number = |index: usize| -> Result<usize, String> {
        args.get(index)
            .ok_or(USAGE)?
            .parse()
            .map_err(|e| format!("argument {index}: {e}"))
    };
    let advice = |index: usize| -> Result<Option<u64>, String> {
        match args.get(index).map(String::as_str) {
            None | Some("-") => Ok(None),
            Some(value) => value
                .parse::<u32>()
                .map(|log| Some(1u64 << log))
                .map_err(|e| e.to_string()),
        }
    };
    let chunking = match args.get(1).map(String::as_str) {
        Some("256") => Chunking::ForcedK256,
        _ => Chunking::Production,
    };
    let profile_name = args.get(2).ok_or(USAGE)?;
    let profile = PROFILES
        .into_iter()
        .find(|profile| format!("{profile:?}").eq_ignore_ascii_case(profile_name))
        .ok_or_else(|| format!("PROFILE {profile_name}: expected single|two|four|eight"))?;
    let shape = Shape {
        log_t: number(0)?,
        chunking,
        profile,
        log_bytecode_len: number(3)?,
        log_ram_k: number(4)?,
        untrusted_advice_bytes: advice(5)?,
        trusted_advice_bytes: advice(6)?,
        program: match (args.get(7), args.get(8)) {
            (Some(chunks), Some(image)) => Some(CommittedProgram {
                log_chunks: chunks.parse().map_err(|e| format!("CHUNKS: {e}"))?,
                image_words: image.parse().map_err(|e| format!("IMAGE: {e}"))?,
            }),
            _ => None,
        },
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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg = |index: usize| args.get(index).map(String::as_str);
    let result = match arg(0) {
        Some("list") => {
            list();
            Ok(())
        }
        Some("seeds") => match arg(1) {
            Some(dir) => seeds(Path::new(dir)),
            None => Err("seeds OUT_DIR".to_string()),
        },
        Some("smoke") => match arg(1) {
            Some(target) => smoke(
                target,
                arg(2).and_then(|v| v.parse().ok()).unwrap_or(100),
                arg(3).and_then(|v| v.parse().ok()).unwrap_or(1),
            ),
            None => Err("smoke TARGET [N] [SEED]".to_string()),
        },
        Some("replay") => match arg(1) {
            Some(target) => replay(
                target,
                &args[2..].iter().map(PathBuf::from).collect::<Vec<_>>(),
            ),
            None => Err("replay TARGET FILE...".to_string()),
        },
        Some("build-guests") => match arg(1) {
            Some(dir) => jolt_akita_fuzz::programs::build_all(Path::new(dir)),
            None => Err("build-guests OUT_DIR".to_string()),
        },
        Some("bundles") => match arg(1) {
            Some(dir) => targets::verifier::write_bundles(Path::new(dir)),
            None => Err("bundles OUT_DIR".to_string()),
        },
        Some("explain-verifier") => explain_verifier(&args[1..]),
        Some("describe-planning") => describe_planning(&args[1..]),
        Some("planning-case") => planning_case(&args[1..]),
        Some("grid-sweep") => grid_sweep::run(
            arg(1).and_then(|v| v.parse().ok()).unwrap_or(0),
            arg(2).and_then(|v| v.parse().ok()).unwrap_or(22),
            arg(3),
        ),
        Some("plan-sweep") => sweep::run(arg(1).unwrap_or("all"), arg(2).map(PathBuf::from)),
        _ => Err(
            "usage: jolt-fuzz-dev list|seeds|smoke|replay|build-guests|planning-case|describe-planning|explain-verifier|bundles|plan-sweep|grid-sweep ..."
                .to_string(),
        ),
    };
    if let Err(message) = result {
        eprintln!("jolt-fuzz-dev: {message}");
        std::process::exit(2);
    }
}
