//! Word-aligned `memset` / shared-alignment `memcmp` overrides, plus a
//! small set of safe helpers for word-aligned slices.
//!
//! # Why this exists
//!
//! Jolt's RAM model is 8-byte addressed: every sub-word load or store
//! (`LBU`, `SB`, `LW`, `SW`, etc.) expands into 7-14 virtual instructions
//! because the emulator has to read the containing 8-byte word, mask,
//! shift, and write back. `compiler_builtins`'s `memset` uses a mixed
//! `SW`/`SB` loop that still pays sub-word expansion on every store, and
//! its `memcmp` is still a pure byte loop. A `u64`-wide fill-loop is a
//! straight win for `memset`, and a shared-alignment word-compare is a
//! clear win for long `memcmp`s.
//!
//! `memcpy` / `memmove` are intentionally NOT overridden here. See
//! `riscv_overrides` and `PERF_NOTES.md` section 10 for the measurements:
//! `compiler_builtins`'s `memcpy` runs a shifted-write algorithm that
//! out-performs any naive word-aligned replacement on mismatched-
//! alignment input, which is common in MPT/RLP/EVM memory paths.
//!
//! The overrides here:
//!
//! 1. Replace `memset` with a byte-prefix + `u64`-stride fill + byte-tail
//!    sequence.
//! 2. Replace `memcmp` with a shared-alignment fast path: compare a short
//!    byte prefix until aligned, then compare full words.
//! 3. Only define the C-ABI symbols when `target_arch = "riscv64"` so
//!    host builds still link against the system libc.
//!
//! Callers that know their pointers are word-aligned (e.g. after viewing
//! a `[U256]` stack slot as `[[u64; 4]]`) can bypass `memcpy` entirely
//! by using the safe helpers in the `safe` module.

#![no_std]

#[cfg(target_arch = "riscv64")]
mod riscv_overrides;

pub mod safe;

/// Size of a Jolt-native word in bytes.
pub const WORD_BYTES: usize = 8;

/// Pull the C-ABI overrides into the final binary. On non-`riscv64` targets
/// this is a no-op so the helper can be called unconditionally from the
/// guest entry point.
#[inline(always)]
pub fn link_overrides() {
    #[cfg(target_arch = "riscv64")]
    riscv_overrides::link_overrides();
}
