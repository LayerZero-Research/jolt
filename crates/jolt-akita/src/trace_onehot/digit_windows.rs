//! Output-stationary commit accumulation for K<D trace one-hot rings.
//!
//! A K<D ring packs `D/K` trace rows, so each column adds several shifts of
//! the same `A` entry into one destination. Loading the entry once as
//! negacyclic windows lets each destination tile sum all of its shifts in
//! registers and touch memory once, instead of once per shift.

use akita_algebra::CyclotomicRing;
use jolt_field::{Fp128x8i32, Unreduced};

use crate::{AkitaField, AKITA_ONE_HOT_K16};

/// Eight coefficients expose the larger NEON register file to the shift sum.
/// Keep four on other targets to avoid spilling on baseline SSE2.
const TILE: usize = if cfg!(target_arch = "aarch64") { 8 } else { 4 };

const PAIR_MAX_DELTA: usize = 2 * AKITA_ONE_HOT_K16 - 1;
const PAIR_MAX_SHIFTS: usize = 512 / AKITA_ONE_HOT_K16;

/// One destination ring element as unreduced [`Fp128x8i32`] lanes.
pub(super) type DigitAccumulator<const D: usize> = [Fp128x8i32; D];

/// Every negacyclic shift of one `A` entry as canonical digits in accumulator-width lanes.
///
/// Holds the digits of `[-a_0, …, -a_{D-1}, a_0, …, a_{D-1}]`, so coefficient
/// `j` of `a · X^k` is entry `D + j - k` for every `k < D`. The digits are the
/// non-negative [`Fp128x8i32`] lanes of each canonical value, so a shift reads
/// accumulator-width lanes directly and adds a value below `2^16` to
/// each destination lane. At most `MAX_WIDE_ACCUMULATIONS` shifts per
/// destination between flushes keep every lane inside `reduce_wide`'s `i32`
/// range.
pub(super) struct DigitWindows<const D: usize> {
    digits: Vec<[i32; 8]>,
    pairs: Vec<[u16; 8]>,
}

impl<const D: usize> DigitWindows<D> {
    pub(super) fn new() -> Self {
        const { assert!(D.is_multiple_of(TILE)) };
        Self {
            digits: vec![[0; 8]; 2 * D],
            pairs: Vec::new(),
        }
    }

    pub(super) fn supports_pairs(one_hot_k: usize) -> bool {
        one_hot_k == AKITA_ONE_HOT_K16 && D > one_hot_k && D / one_hot_k <= PAIR_MAX_SHIFTS
    }

    /// Replaces the held entry with `src`.
    pub(super) fn load(&mut self, src: &CyclotomicRing<AkitaField, D>) {
        let (negative, positive) = self.digits.split_at_mut(D);
        for ((negative, positive), &value) in negative.iter_mut().zip(positive).zip(&src.coeffs) {
            *negative = Fp128x8i32::from(-value).0;
            *positive = Fp128x8i32::from(value).0;
        }
    }

    /// Prepares A and A + X^delta A for distances 1..=31 as canonical
    /// 16-bit digits. Each paired rotation then adds one canonical value,
    /// halving both table traffic and wide-lane additions for dense rows.
    pub(super) fn load_paired(&mut self, src: &CyclotomicRing<AkitaField, D>) {
        self.pairs.resize((PAIR_MAX_DELTA + 1) * 2 * D, [0; 8]);
        for (delta, pair) in self.pairs.chunks_exact_mut(2 * D).enumerate() {
            let (negative, positive) = pair.split_at_mut(D);
            for index in 0..D {
                let value = if delta == 0 {
                    src.coeffs[index]
                } else if index >= delta {
                    src.coeffs[index] + src.coeffs[index - delta]
                } else {
                    src.coeffs[index] - src.coeffs[D + index - delta]
                };
                negative[index] = Fp128x8i32::from(-value).0.map(|lane| lane as u16);
                positive[index] = Fp128x8i32::from(value).0.map(|lane| lane as u16);
            }
        }
    }

    /// Adds sorted K=16 shifts, pairing neighbors at distance at most 31.
    /// Returns the number of canonical values added to each coefficient.
    pub(super) fn accumulate_paired(
        &self,
        dst: &mut DigitAccumulator<D>,
        shifts: &[usize],
    ) -> usize {
        if shifts.is_empty() {
            return 0;
        }
        debug_assert!(shifts.len() <= PAIR_MAX_SHIFTS);
        debug_assert!(shifts.iter().all(|&shift| shift < D));
        debug_assert!(shifts.windows(2).all(|pair| pair[0] < pair[1]));
        let mut sources: [&[[u16; 8]]; PAIR_MAX_SHIFTS] = [&[]; PAIR_MAX_SHIFTS];
        let mut count = 0;
        let mut index = 0;
        while index < shifts.len() {
            let shift = shifts[index];
            let start = D - shift;
            if index + 1 < shifts.len() && shifts[index + 1] - shift <= PAIR_MAX_DELTA {
                let delta = shifts[index + 1] - shift;
                let pair = delta * 2 * D;
                sources[count] = &self.pairs[pair + start..pair + start + D];
                index += 2;
            } else {
                sources[count] = &self.pairs[start..start + D];
                index += 1;
            }
            count += 1;
        }
        for (tile, out) in dst.chunks_exact_mut(TILE).enumerate() {
            let base = tile * TILE;
            let mut sums: [[i32; 8]; TILE] = std::array::from_fn(|index| out[index].0);
            for source in &sources[..count] {
                let digits = &source[base..][..TILE];
                sums = std::array::from_fn(|index| {
                    std::array::from_fn(|lane| sums[index][lane] + i32::from(digits[index][lane]))
                });
            }
            for (out, sum) in out.iter_mut().zip(sums) {
                out.0 = sum;
            }
        }
        count
    }

    /// `dst += a · Σ_k X^k` over `shifts`, each `< D`.
    pub(super) fn accumulate(&self, dst: &mut DigitAccumulator<D>, shifts: &[usize]) {
        debug_assert!(shifts.iter().all(|&shift| shift < D));
        if shifts.is_empty() {
            return;
        }
        for (tile, out) in dst.chunks_exact_mut(TILE).enumerate() {
            let base = D + tile * TILE;
            let mut sums: [[i32; 8]; TILE] = std::array::from_fn(|index| out[index].0);
            for &shift in shifts {
                for (sum, digits) in sums.iter_mut().zip(&self.digits[base - shift..][..TILE]) {
                    for (sum, &digit) in sum.iter_mut().zip(digits) {
                        *sum += digit;
                    }
                }
            }
            for (out, sum) in out.iter_mut().zip(sums) {
                out.0 = sum;
            }
        }
    }
}

/// Adds every accumulator into its reduced ring element and clears it.
pub(super) fn flush_digit_accumulators<const D: usize>(
    accumulators: &mut [DigitAccumulator<D>],
    reduced: &mut [CyclotomicRing<AkitaField, D>],
) {
    for (accumulator, reduced) in accumulators.iter_mut().zip(reduced) {
        for (lanes, coefficient) in accumulator.iter_mut().zip(&mut reduced.coeffs) {
            *coefficient += AkitaField::reduce_wide(std::mem::replace(lanes, Fp128x8i32([0; 8])));
        }
    }
}
