//! Safe, word-aligned helpers callers can opt into directly.
//!
//! These avoid `unsafe` entirely at the public API and are useful when you
//! already have a `&mut [u64]` or `&[u64]` (for example, after viewing a
//! `[U256]` stack slot as a `[[u64; 4]]`). Plain byte-slice operations like
//! `<[u8]>::copy_from_slice` already route through the `memcpy` symbol, so
//! the `#[no_mangle]` overrides in `riscv_overrides` cover those cases with
//! no additional helper.

/// Copy `src` into `dst` using `u64`-sized memory operations.
#[inline]
pub fn copy_words(dst: &mut [u64], src: &[u64]) {
    dst.copy_from_slice(src);
}

/// Zero out `dst` using `u64`-sized memory operations.
#[inline]
pub fn zero_words(dst: &mut [u64]) {
    dst.fill(0);
}

/// Swap `a` and `b` limb-wise. Both slices must have the same length.
#[inline]
pub fn swap_words(a: &mut [u64], b: &mut [u64]) {
    a.swap_with_slice(b);
}

/// Compare two `u64` slices byte-wise (big-endian numeric order on each
/// limb), returning the standard `Ordering`.
#[inline]
pub fn cmp_words(a: &[u64], b: &[u64]) -> core::cmp::Ordering {
    let len = a.len().min(b.len());
    for i in 0..len {
        let av = u64::from_be(a[i]);
        let bv = u64::from_be(b[i]);
        match av.cmp(&bv) {
            core::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    a.len().cmp(&b.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_words_roundtrip() {
        let src = [1u64, 2, 3, 4];
        let mut dst = [0u64; 4];
        copy_words(&mut dst, &src);
        assert_eq!(dst, src);
    }

    #[test]
    fn zero_words_clears() {
        let mut dst = [u64::MAX; 3];
        zero_words(&mut dst);
        assert_eq!(dst, [0u64; 3]);
    }

    #[test]
    fn swap_words_exchanges() {
        let mut a = [1u64, 2, 3, 4];
        let mut b = [5u64, 6, 7, 8];
        swap_words(&mut a, &mut b);
        assert_eq!(a, [5, 6, 7, 8]);
        assert_eq!(b, [1, 2, 3, 4]);
    }

    #[test]
    fn cmp_words_byte_order() {
        let a = [u64::from_be(0x01_00_00_00_00_00_00_00), 0];
        let b = [u64::from_be(0x00_ff_ff_ff_ff_ff_ff_ff), 0];
        assert_eq!(cmp_words(&a, &b), core::cmp::Ordering::Greater);
    }
}
