//! Process environment shared by every harness.

use rayon::ThreadPoolBuilder;
use std::path::PathBuf;
use std::sync::Once;
use std::thread::Builder;

pub const THREADS_ENV: &str = "JOLT_FUZZ_THREADS";
pub const ARTIFACTS_ENV: &str = "JOLT_FUZZ_ARTIFACTS";

/// Stack for Rayon workers and harness threads; proving recursion is deep
/// under sanitizers.
pub const STACK_SIZE: usize = 64 * 1024 * 1024;

static INIT: Once = Once::new();

/// Size the global Rayon pool (Jolt and Akita share it) once per process.
///
/// The runner sets `JOLT_FUZZ_THREADS` (default 1) so worker count times
/// internal threads never exceeds the campaign's CPU budget.
pub fn init() {
    INIT.call_once(|| {
        let threads = internal_threads();
        ThreadPoolBuilder::new()
            .num_threads(threads)
            .stack_size(STACK_SIZE)
            .thread_name(|index| format!("jolt-fuzz-rayon-{index}"))
            .build_global()
            .expect("the harness owns the global Rayon pool");
    });
}

pub fn internal_threads() -> usize {
    std::env::var(THREADS_ENV)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&threads| threads > 0)
        .unwrap_or(1)
}

/// Directory holding `<family>.aks` schedule artifacts.
pub fn artifacts_dir() -> PathBuf {
    std::env::var_os(ARTIFACTS_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../crates/jolt-akita/schedules")
        })
}

/// Run `f` on a thread with a large stack and propagate its panic.
///
/// libFuzzer's panic hook aborts the process on a panic in any thread, so
/// findings keep their original backtrace.
pub fn on_large_stack<T: Send>(f: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|scope| {
        Builder::new()
            .stack_size(STACK_SIZE)
            .spawn_scoped(scope, f)
            .expect("spawn harness thread")
            .join()
            .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
    })
}

pub const MAX_CASE_COEFFS_ENV: &str = "JOLT_FUZZ_MAX_CASE_COEFFS";

/// Openings above this many committed coefficients (all groups) are planned
/// but not proved in one iteration. Default `2^23`: the smallest production
/// trace (K=16, `log_T = 12`, final arity 22) plus advice fits.
pub fn max_case_coefficients() -> u128 {
    std::env::var(MAX_CASE_COEFFS_ENV)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1 << 23)
}
