//! Near-exhaustive preprocessing-planning sweep.
//!
//! Each stage of `Shape -> AkitaScheme::setup` depends on few parameters, so
//! the sweep enumerates each stage's own inputs completely instead of their
//! Cartesian product:
//!
//! - `geometry`: every `(chunking, log_T, bytecode length, ram_K)` through
//!   Jolt's trace geometry (`one_hot_trace_setup_shape`); the chunk profile
//!   does not enter it;
//! - `advice`: every `(chunking, profile, log_T)` with every untrusted and
//!   trusted advice capacity (absent or `2^3..=2^38` bytes) through grouped
//!   planning;
//! - `program`: every `(chunking, profile, log_T)` with every
//!   committed-program chunk count and chunk/image arity edge, alone and with
//!   both advice kinds.
//!
//! Planning only; proving is the `planning` target's job. Every result is
//! written to the CSV, and failures are summarized by stage and message
//! (numbers masked) with in-contract failures, the liveness findings,
//! listed first.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use jolt_akita::AkitaChunkProfile;
use jolt_akita_fuzz::shape::{self, Chunking, CommittedProgram, Shape, PROFILES};
use rayon::prelude::*;

const CHUNKINGS: [Chunking; 2] = [Chunking::Production, Chunking::ForcedK256];

/// Every `(chunking, profile)` trace family.
fn families() -> impl Iterator<Item = (Chunking, AkitaChunkProfile)> {
    CHUNKINGS
        .into_iter()
        .flat_map(|chunking| PROFILES.into_iter().map(move |profile| (chunking, profile)))
}

/// One past the documented end, so the sweep also shows the rejection.
fn log_ts() -> std::ops::RangeInclusive<usize> {
    shape::MIN_LOG_T..=shape::MAX_LOG_T + 1
}

fn base(chunking: Chunking, profile: AkitaChunkProfile, log_t: usize) -> Shape {
    Shape {
        log_t,
        chunking,
        profile,
        log_bytecode_len: 16,
        log_ram_k: 22,
        untrusted_advice_bytes: None,
        trusted_advice_bytes: None,
        program: None,
    }
}

fn geometry_shapes() -> Vec<Shape> {
    let mut shapes = Vec::new();
    for chunking in CHUNKINGS {
        for log_t in log_ts() {
            for log_bytecode_len in 0..=32 {
                for log_ram_k in 1..=61 {
                    shapes.push(Shape {
                        log_bytecode_len,
                        log_ram_k,
                        ..base(chunking, AkitaChunkProfile::Single, log_t)
                    });
                }
            }
        }
    }
    shapes
}

/// Advice capacities: absent, then every power of two from 8 bytes (arity
/// 14 after padding) to `2^38` bytes (arity 35, one past the limit).
fn advice_capacities() -> Vec<Option<u64>> {
    std::iter::once(None)
        .chain((3..=38).map(|log_bytes| Some(1u64 << log_bytes)))
        .collect()
}

fn advice_shapes() -> Vec<Shape> {
    let capacities = advice_capacities();
    let mut shapes = Vec::new();
    for (chunking, profile) in families() {
        for log_t in log_ts() {
            for &untrusted in &capacities {
                for &trusted in &capacities {
                    if untrusted.is_none() && trusted.is_none() {
                        continue;
                    }
                    shapes.push(Shape {
                        untrusted_advice_bytes: untrusted,
                        trusted_advice_bytes: trusted,
                        ..base(chunking, profile, log_t)
                    });
                }
            }
        }
    }
    shapes
}

fn program_shapes() -> Vec<Shape> {
    // Default guest advice (4096 bytes each) and none.
    let advice = [(None, None), (Some(4096), Some(4096))];
    // Bytecode lengths put the chunk arity at its edges for every chunk count.
    let log_bytecode_lens = [4usize, 8, 12, 16, 20, 24, 26];
    let image_words = [1usize, 1 << 13, (1 << 13) + 1, 1 << 20, 1 << 26, 1 << 34];
    let mut shapes = Vec::new();
    for (chunking, profile) in families() {
        for log_t in log_ts() {
            for log_bytecode_len in log_bytecode_lens {
                for log_chunks in 0..=shape::MAX_LOG_CHUNKS.min(log_bytecode_len) {
                    for &image_words in &image_words {
                        for &(untrusted, trusted) in &advice {
                            shapes.push(Shape {
                                log_bytecode_len,
                                untrusted_advice_bytes: untrusted,
                                trusted_advice_bytes: trusted,
                                program: Some(CommittedProgram {
                                    log_chunks,
                                    image_words,
                                }),
                                ..base(chunking, profile, log_t)
                            });
                        }
                    }
                }
            }
        }
    }
    shapes
}

struct Outcome {
    shape: Shape,
    in_contract: bool,
    result: Result<usize, shape::Failure>,
    seconds: f64,
}

/// Replace digit runs with `N` so messages group across sizes.
fn mask(message: &str) -> String {
    let mut out = String::new();
    let mut in_number = false;
    for ch in message.chars() {
        if ch.is_ascii_digit() {
            if !in_number {
                out.push('N');
            }
            in_number = true;
        } else {
            in_number = false;
            out.push(ch);
        }
    }
    out
}

fn evaluate(shapes: Vec<Shape>, plan: bool) -> Vec<Outcome> {
    let done = Mutex::new(0usize);
    let total = shapes.len();
    shapes
        .into_par_iter()
        .map(|shape| {
            let started = Instant::now();
            let result =
                shape.setup_request().and_then(
                    |request| {
                        if plan {
                            shape::plan(&request)
                        } else {
                            Ok(0)
                        }
                    },
                );
            let seconds = started.elapsed().as_secs_f64();
            if plan {
                let mut done = done.lock().unwrap();
                *done += 1;
                if done.is_multiple_of(500) {
                    eprintln!("  {done}/{total}");
                }
            }
            Outcome {
                in_contract: shape.in_contract(),
                shape,
                result,
                seconds,
            }
        })
        .collect()
}

fn report(phase: &str, outcomes: &[Outcome], csv: &mut String) {
    let mut groups: BTreeMap<(bool, String), Vec<&Outcome>> = BTreeMap::new();
    let (mut ok, mut failed_in, mut failed_out) = (0usize, 0usize, 0usize);
    let mut slowest = 0.0f64;
    for outcome in outcomes {
        slowest = slowest.max(outcome.seconds);
        let (status, stage, message, rows) = match &outcome.result {
            Ok(rows) => {
                ok += 1;
                ("ok", String::new(), String::new(), *rows)
            }
            Err(failure) => {
                if outcome.in_contract {
                    failed_in += 1;
                } else {
                    failed_out += 1;
                }
                groups
                    .entry((
                        !outcome.in_contract,
                        format!("{:?}: {}", failure.stage, mask(&failure.message)),
                    ))
                    .or_default()
                    .push(outcome);
                (
                    "failed",
                    format!("{:?}", failure.stage),
                    failure.message.clone(),
                    0,
                )
            }
        };
        let _ = writeln!(
            csv,
            "{phase},\"{}\",{},{status},{stage},\"{}\",{rows},{:.4}",
            outcome.shape,
            outcome.in_contract,
            message.replace('"', "'"),
            outcome.seconds
        );
    }
    println!(
        "== {phase}: {} shapes, {ok} ok, {failed_in} in-contract failures, {failed_out} out-of-contract rejections; slowest {slowest:.2}s",
        outcomes.len()
    );
    for ((out_of_contract, key), members) in &groups {
        println!(
            "  [{}] x{} {key}",
            if *out_of_contract {
                "out-of-contract"
            } else {
                "IN-CONTRACT"
            },
            members.len()
        );
        for member in members.iter().take(3) {
            println!("      e.g. {}", member.shape);
        }
    }
}

pub fn run(phase: &str, csv_path: Option<PathBuf>) -> Result<(), String> {
    jolt_akita_fuzz::env::init();
    let mut csv = String::from("phase,shape,in_contract,status,stage,message,rows,seconds\n");
    let mut failed = 0usize;
    let mut phases: Vec<(&str, Vec<Shape>, bool)> = Vec::new();
    if matches!(phase, "geometry" | "all") {
        phases.push(("geometry", geometry_shapes(), false));
    }
    if matches!(phase, "advice" | "all") {
        phases.push(("advice", advice_shapes(), true));
    }
    if matches!(phase, "program" | "all") {
        phases.push(("program", program_shapes(), true));
    }
    if phases.is_empty() {
        return Err(format!("unknown phase {phase}"));
    }
    for (name, shapes, plan) in phases {
        let started = Instant::now();
        eprintln!("{name}: {} shapes", shapes.len());
        let outcomes = evaluate(shapes, plan);
        report(name, &outcomes, &mut csv);
        println!("  ({:.1}s)", started.elapsed().as_secs_f64());
        failed += outcomes
            .iter()
            .filter(|outcome| outcome.in_contract && outcome.result.is_err())
            .count();
    }
    if let Some(path) = csv_path {
        std::fs::write(&path, csv).map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    if failed > 0 {
        return Err(format!("{failed} in-contract shapes failed"));
    }
    Ok(())
}
