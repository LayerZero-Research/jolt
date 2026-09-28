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
//! jolt-fuzz-dev grid-sweep [MIN] [MAX] [FAMILY]
//!                                          every catalog row with MIN..=MAX
//!                                          variables (k16, k256, dense)
//! ```

mod grid_sweep;
mod sweep;

use jolt_akita_fuzz::input::SplitMix64;
use jolt_akita_fuzz::targets;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn list() {
    for (name, _) in targets::ALL {
        println!("{name}");
    }
}

fn random_bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut rng = SplitMix64::new(seed);
    (0..len).map(|_| rng.next_u64() as u8).collect()
}

fn smoke(name: &str, count: usize, seed: u64) -> Result<(), String> {
    let run = targets::by_name(name).ok_or_else(|| format!("unknown target {name}"))?;
    let mut rng = SplitMix64::new(seed);
    let started = Instant::now();
    for index in 0..count {
        let len = (rng.next_u64() % 4096) as usize;
        let data = random_bytes(rng.next_u64(), len);
        let one = Instant::now();
        run(&data);
        if index < 3 || one.elapsed().as_secs_f64() > 5.0 {
            eprintln!("input {index}: {:.3}s", one.elapsed().as_secs_f64());
        }
    }
    eprintln!(
        "{name}: {count} inputs in {:.2}s",
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
    for (name, _) in targets::ALL {
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
        Some("grid-sweep") => grid_sweep::run(
            arg(1).and_then(|v| v.parse().ok()).unwrap_or(0),
            arg(2).and_then(|v| v.parse().ok()).unwrap_or(22),
            arg(3),
        ),
        Some("plan-sweep") => sweep::run(arg(1).unwrap_or("all"), arg(2).map(PathBuf::from)),
        _ => Err(
            "usage: jolt-fuzz-dev list|seeds|smoke|replay|plan-sweep|grid-sweep ...".to_string(),
        ),
    };
    if let Err(message) = result {
        eprintln!("jolt-fuzz-dev: {message}");
        std::process::exit(2);
    }
}
