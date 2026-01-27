//! Minimal `no_std` 256-bit field arithmetic helpers.
//!
//! This crate provides a small set of primitives needed by Jolt inline curve crates
//! in guest builds, without pulling in Arkworks or `std`.
//!
//! Representation:
//! - 256-bit integers are represented as 4 little-endian `u64` limbs.
//! - Field elements are represented in Montgomery form, i.e. `x = a·R mod p`,
//!   where `R = 2^256 mod p`.

#![no_std]

/// 256-bit integer / field limb representation (little-endian).
pub type U256 = [u64; 4];

/// Compare two 256-bit integers in little-endian limb form.
#[inline(always)]
pub fn cmp_u256(a: &U256, b: &U256) -> core::cmp::Ordering {
    // Compare most significant limb first.
    let mut i = 4usize;
    while i > 0 {
        i -= 1;
        if a[i] < b[i] {
            return core::cmp::Ordering::Less;
        }
        if a[i] > b[i] {
            return core::cmp::Ordering::Greater;
        }
    }
    core::cmp::Ordering::Equal
}

#[inline(always)]
pub fn is_zero_u256(a: &U256) -> bool {
    a[0] == 0 && a[1] == 0 && a[2] == 0 && a[3] == 0
}

/// Add two 256-bit integers returning (sum, carry).
#[inline(always)]
pub fn add_u256(a: &U256, b: &U256) -> (U256, u64) {
    let mut out = [0u64; 4];
    let mut carry = 0u128;
    let mut i = 0usize;
    while i < 4 {
        let s = a[i] as u128 + b[i] as u128 + carry;
        out[i] = s as u64;
        carry = s >> 64;
        i += 1;
    }
    (out, carry as u64)
}

/// Subtract two 256-bit integers returning (diff, borrow) where borrow is 0 or 1.
#[inline(always)]
pub fn sub_u256(a: &U256, b: &U256) -> (U256, u64) {
    let mut out = [0u64; 4];
    let mut borrow = 0u128;
    let mut i = 0usize;
    while i < 4 {
        let ai = a[i] as u128;
        let bi = b[i] as u128 + borrow;
        if ai >= bi {
            out[i] = (ai - bi) as u64;
            borrow = 0;
        } else {
            out[i] = ((1u128 << 64) + ai - bi) as u64;
            borrow = 1;
        }
        i += 1;
    }
    (out, borrow as u64)
}

/// Add modulo `modulus` (both operands assumed < modulus).
#[inline(always)]
pub fn add_mod(a: &U256, b: &U256, modulus: &U256) -> U256 {
    let (s, carry) = add_u256(a, b);
    // If overflowed or >= modulus, subtract modulus.
    if carry != 0 || cmp_u256(&s, modulus) != core::cmp::Ordering::Less {
        let (d, _borrow) = sub_u256(&s, modulus);
        d
    } else {
        s
    }
}

/// Sub modulo `modulus` (both operands assumed < modulus).
#[inline(always)]
pub fn sub_mod(a: &U256, b: &U256, modulus: &U256) -> U256 {
    let (d, borrow) = sub_u256(a, b);
    if borrow != 0 {
        let (s, _carry) = add_u256(&d, modulus);
        s
    } else {
        d
    }
}

/// Montgomery multiplication for 256-bit moduli using the coarsely integrated operand scanning (CIOS) method.
///
/// Inputs/outputs are in Montgomery form.
///
/// Parameters:
/// - `modulus`: p (4 limbs, little-endian)
/// - `inv`: `-p^{-1} mod 2^64` (single-limb Montgomery constant)
#[inline(always)]
pub fn mont_mul(a: &U256, b: &U256, modulus: &U256, inv: u64) -> U256 {
    // Compute t = a * b (512-bit), but integrate reduction per-limb (CIOS).
    // We keep one extra limb to safely propagate carries during reduction.
    let mut t = [0u64; 9];

    // Schoolbook multiplication into t (8 limbs, plus a carry limb).
    let mut i = 0usize;
    while i < 4 {
        let mut carry = 0u128;
        let ai = a[i] as u128;
        let mut j = 0usize;
        while j < 4 {
            let idx = i + j;
            let uv = ai * (b[j] as u128) + (t[idx] as u128) + carry;
            t[idx] = uv as u64;
            carry = uv >> 64;
            j += 1;
        }
        // propagate carry into t[i+4..] (never overflows past t[8]).
        let idx = i + 4;
        let uv = (t[idx] as u128) + carry;
        t[idx] = uv as u64;
        carry = uv >> 64;
        let mut k = idx + 1;
        while carry != 0 && k < 9 {
            let uv2 = (t[k] as u128) + carry;
            t[k] = uv2 as u64;
            carry = uv2 >> 64;
            k += 1;
        }
        i += 1;
    }

    // Montgomery reduction: for i=0..3, compute m = t[i]*inv mod 2^64 and add m*modulus to t starting at i.
    let mut k = 0usize;
    while k < 4 {
        let m = t[k].wrapping_mul(inv);

        let mut carry = 0u128;
        let mk = m as u128;
        let mut j = 0usize;
        while j < 4 {
            let idx = k + j;
            let uv = mk * (modulus[j] as u128) + (t[idx] as u128) + carry;
            t[idx] = uv as u64;
            carry = uv >> 64;
            j += 1;
        }

        // add carry into t[k+4..]
        let idx = k + 4;
        let uv = (t[idx] as u128) + carry;
        t[idx] = uv as u64;
        let mut c2 = uv >> 64;
        let mut idx2 = idx + 1;
        while c2 != 0 && idx2 < 9 {
            let uv2 = (t[idx2] as u128) + c2;
            t[idx2] = uv2 as u64;
            c2 = uv2 >> 64;
            idx2 += 1;
        }

        k += 1;
    }

    // The result is in the top 4 limbs: t[4..8). Conditionally subtract modulus.
    let mut out = [t[4], t[5], t[6], t[7]];
    if t[8] != 0 || cmp_u256(&out, modulus) != core::cmp::Ordering::Less {
        let (d, _borrow) = sub_u256(&out, modulus);
        out = d;
    }
    out
}

#[inline(always)]
pub fn mont_square(a: &U256, modulus: &U256, inv: u64) -> U256 {
    mont_mul(a, a, modulus, inv)
}

/// Convert a canonical value into Montgomery form: `a -> a·R mod p`.
#[inline(always)]
pub fn to_mont(a: &U256, modulus: &U256, inv: u64, r2: &U256) -> U256 {
    // a * R^2 * R^{-1} = a * R mod p
    mont_mul(a, r2, modulus, inv)
}

/// Convert a Montgomery value back to canonical form: `a·R -> a`.
#[inline(always)]
pub fn from_mont(a_mont: &U256, modulus: &U256, inv: u64) -> U256 {
    // Multiply by 1 in canonical form; 1_mont = R mod p, but mont_mul expects mont inputs.
    // Using mont_mul(a_mont, 1) where 1 is canonical interpreted as mont is wrong.
    //
    // In Montgomery arithmetic, `mont_mul(x, 1)` yields `x·1·R^{-1} = x·R^{-1}`.
    // Since x is `a·R`, this returns `a`.
    const ONE: U256 = [1u64, 0, 0, 0];
    mont_mul(a_mont, &ONE, modulus, inv)
}

