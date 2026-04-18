//! `#[no_mangle]` C-ABI overrides for `memset` and `memcmp`, compiled
//! only for `target_arch = "riscv64"`.
//!
//! Why these two are worth overriding:
//!
//! - `memset` has only one pointer, so after a short byte prefix we can
//!   switch to aligned `SD` stores and avoid sub-word write helpers.
//! - `compiler_builtins`'s `memcmp` is still a pure byte loop. On Jolt
//!   that means every compared byte pays the `LBU` helper cost. We can do
//!   better when the two pointers share the same alignment: compare a
//!   short byte prefix until aligned, then compare full words. When the
//!   alignments differ we keep the baseline byte loop, because the full
//!   reassembly machinery was not worth its setup cost on smaller fixtures.
//!
//! `memcpy` / `memmove` are intentionally not overridden. Their baseline
//! implementation already has a sophisticated misaligned fast path, and
//! naive replacements regressed the mainnet trace.

use crate::WORD_BYTES;

const WORD: usize = WORD_BYTES;
const WORD_MASK: usize = WORD - 1;
const WORD_COMPARE_THRESHOLD: usize = 16;

type MemsetFn = unsafe extern "C" fn(*mut u8, i32, usize) -> *mut u8;
type MemcmpFn = unsafe extern "C" fn(*const u8, *const u8, usize) -> i32;

// Force the linker to keep the `#[no_mangle]` symbols below even under
// `--gc-sections` and cross-rlib LTO.
#[used]
static KEEP_MEMSET: MemsetFn = memset;
#[used]
static KEEP_MEMCMP: MemcmpFn = memcmp;

/// Call once from the guest crate to guarantee this compilation unit is
/// pulled in by the linker. The body is intentionally trivial.
#[inline(never)]
pub fn link_overrides() {
    core::hint::black_box((&KEEP_MEMSET, &KEEP_MEMCMP));
}

#[inline(always)]
unsafe fn cmp_bytes(mut lhs: *const u8, mut rhs: *const u8, mut n: usize) -> i32 {
    while n > 0 {
        let lhs_byte = *lhs;
        let rhs_byte = *rhs;
        if lhs_byte != rhs_byte {
            return if lhs_byte < rhs_byte { -1 } else { 1 };
        }
        lhs = lhs.add(1);
        rhs = rhs.add(1);
        n -= 1;
    }
    0
}

#[inline(always)]
fn cmp_word(lhs: usize, rhs: usize) -> i32 {
    match usize::from_be(lhs).cmp(&usize::from_be(rhs)) {
        core::cmp::Ordering::Less => -1,
        core::cmp::Ordering::Equal => 0,
        core::cmp::Ordering::Greater => 1,
    }
}

#[inline(always)]
unsafe fn cmp_aligned_words(mut lhs: *const usize, mut rhs: *const usize, n: usize) -> i32 {
    let end = lhs.wrapping_byte_add(n);
    while lhs < end {
        let lhs_word = *lhs;
        let rhs_word = *rhs;
        if lhs_word != rhs_word {
            return cmp_word(lhs_word, rhs_word);
        }
        lhs = lhs.add(1);
        rhs = rhs.add(1);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, val: i32, n: usize) -> *mut u8 {
    let byte = val as u8;
    let mut n = n;
    let mut p = dst;

    if n < WORD {
        while n > 0 {
            *p = byte;
            p = p.add(1);
            n -= 1;
        }
        return dst;
    }

    let off = p as usize & (WORD - 1);
    if off != 0 {
        let head = WORD - off;
        for _ in 0..head {
            *p = byte;
            p = p.add(1);
        }
        n -= head;
    }

    let word = u64::from_ne_bytes([byte; 8]);
    while n >= WORD {
        // SAFETY: `p` is now 8-byte aligned by construction.
        *(p as *mut u64) = word;
        p = p.add(WORD);
        n -= WORD;
    }

    while n > 0 {
        *p = byte;
        p = p.add(1);
        n -= 1;
    }

    dst
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(mut lhs: *const u8, mut rhs: *const u8, mut n: usize) -> i32 {
    let align_match = (((lhs as usize) ^ (rhs as usize)) & WORD_MASK) == 0;
    if align_match && n >= WORD_COMPARE_THRESHOLD {
        let prefix = (lhs as usize).wrapping_neg() & WORD_MASK;
        let prefix_cmp = cmp_bytes(lhs, rhs, prefix);
        if prefix_cmp != 0 {
            return prefix_cmp;
        }

        lhs = lhs.add(prefix);
        rhs = rhs.add(prefix);
        n -= prefix;

        let n_words = n & !WORD_MASK;
        if n_words != 0 {
            let word_cmp = cmp_aligned_words(lhs.cast::<usize>(), rhs.cast::<usize>(), n_words);
            if word_cmp != 0 {
                return word_cmp;
            }

            lhs = lhs.add(n_words);
            rhs = rhs.add(n_words);
            n -= n_words;
        }
    }

    cmp_bytes(lhs, rhs, n)
}
