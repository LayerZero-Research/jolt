//! Every base catalog row up to `2^MAX_LOG2` coefficients (per group) through
//! the `grid` target in release mode, including the setup-offloaded rows too
//! large for an instrumented iteration.
//!
//! Each row runs with zero entries committed in every row, the identically
//! zero polynomial, an all-maximal, and a random witness, printing its
//! fold-grind peaks. A failing row is
//! reported and the sweep continues; the exit status is nonzero if any failed.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use jolt_akita_fuzz::artifacts;
use jolt_akita_fuzz::liveness;
use jolt_akita_fuzz::opening::{Fill, Witness};
use jolt_akita_fuzz::targets::grid;

pub fn run(min_log2: usize, max_log2: usize, family: Option<&str>) -> Result<(), String> {
    jolt_akita_fuzz::env::init();
    let rows: Vec<_> = grid::rows(&artifacts::shared())
        .into_iter()
        .filter(|row| (min_log2..=max_log2).contains(&row.num_vars))
        .filter(|row| family.is_none_or(|name| row.family.name() == name))
        .collect();
    println!(
        "{:<32} {:>6} {:>8} {:>8} {:>8} {:>6}  status",
        "row", "folds", "l2", "linf", "attempts", "mean"
    );
    let mut failed = 0usize;
    for row in &rows {
        let _ = liveness::take_peak();
        let started = Instant::now();
        let mut failures = Vec::new();
        // (label, fill, zero-row mask): zero rows committed, the identically
        // zero polynomial, all-maximal, and random.
        let cases = [
            ("zero-rows", Fill::Zero, u64::MAX),
            ("zero-poly", Fill::Zero, 0),
            ("max", Fill::Max, 1),
            ("random", Fill::Random, 1),
        ];
        for (index, (label, fill, mask)) in cases.into_iter().enumerate() {
            let witness = Witness {
                seed: 0x5eed + index as u64,
                dense: fill,
                trace: fill,
                zero_committed_columns: mask,
                point: fill,
            };
            let digest = [row.num_vars as u8; 32];
            if catch_unwind(AssertUnwindSafe(|| grid::check(*row, digest, &witness))).is_err() {
                failures.push(label);
            }
        }
        let peak = liveness::take_peak();
        println!(
            "{:<32} {:>6} {:>8.4} {:>8.4} {:>8} {:>6.2}  {} ({:.1}s)",
            row.label(),
            peak.folds,
            peak.max_margin,
            peak.max_linf_margin,
            peak.max_attempts,
            peak.attempts_total as f64 / peak.folds.max(1) as f64,
            if failures.is_empty() {
                "ok".to_string()
            } else {
                format!("FAILED {failures:?}")
            },
            started.elapsed().as_secs_f64()
        );
        failed += usize::from(!failures.is_empty());
    }
    if failed > 0 {
        return Err(format!("{failed} of {} rows failed", rows.len()));
    }
    Ok(())
}
