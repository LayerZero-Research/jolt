//! Grumpkin guest backend without Arkworks.
//!
//! Field elements are represented as 4 little-endian `u64` limbs in Montgomery form.

// This module contains many helpers that are only exercised in certain build modes
// (guest vs host equivalence tests). CI runs with `-D warnings`, so we silence
// `dead_code` here intentionally.
#![allow(dead_code)]

use crate::constants::*;

use jolt_inlines_field_arith as fa;

use serde::{Deserialize, Serialize};

#[cfg(all(
    not(feature = "host"),
    any(target_arch = "riscv32", target_arch = "riscv64")
))]
#[inline(always)]
fn is_ge_modulus(x: &fa::U256, m: &fa::U256) -> bool {
    fa::cmp_u256(x, m) != core::cmp::Ordering::Less
}

#[cfg(all(
    not(feature = "host"),
    any(target_arch = "riscv32", target_arch = "riscv64")
))]
#[inline(always)]
fn is_not_equal(x: &fa::U256, y: &fa::U256) -> bool {
    x[0] != y[0] || x[1] != y[1] || x[2] != y[2] || x[3] != y[3]
}

/// panic instruction: spoils the proof (used for inline checks).
#[cfg(all(
    not(feature = "host"),
    any(target_arch = "riscv32", target_arch = "riscv64")
))]
#[inline(always)]
pub fn hcf() {
    unsafe {
        let u = 0u64;
        let v = 1u64;
        core::arch::asm!(
            ".insn b {opcode}, {funct3}, {rs1}, {rs2}, 0",
            opcode = const 0x5B,   // virtual instruction opcode
            funct3 = const 0b001,  // VirtualAssertEQ funct3
            rs1 = in(reg) u,
            rs2 = in(reg) v,
            options(nostack)
        );
    }
}

#[cfg(all(
    not(feature = "host"),
    not(any(target_arch = "riscv32", target_arch = "riscv64"))
))]
pub fn hcf() {
    panic!("hcf called on non-RISC-V target without host feature");
}

#[cfg(feature = "host")]
pub fn hcf() {
    panic!("explicit host code panic function called");
}

/// A trait for unwrapping Results in a way that spoils the proof on error.
pub trait UnwrapOrSpoilProof<T> {
    fn unwrap_or_spoil_proof(self) -> T;
}

impl<T> UnwrapOrSpoilProof<T> for Result<T, GrumpkinError> {
    #[inline(always)]
    fn unwrap_or_spoil_proof(self) -> T {
        match self {
            Ok(v) => v,
            Err(_) => {
                hcf();
                unreachable!()
            }
        }
    }
}

/// Error types for grumpkin operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GrumpkinError {
    InvalidFqElement,
    InvalidFrElement,
    NotOnCurve,
}

/// Grumpkin base field element (Fq) in Montgomery form.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GrumpkinFq {
    limbs: fa::U256,
}

impl GrumpkinFq {
    #[inline(always)]
    pub fn zero() -> Self {
        Self { limbs: [0u64; 4] }
    }

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        fa::is_zero_u256(&self.limbs)
    }

    /// Create from canonical (non-Montgomery) limbs, validating `< p`.
    #[inline(always)]
    pub fn from_u64_arr(arr: &[u64; 4]) -> Result<Self, GrumpkinError> {
        if fa::cmp_u256(arr, &GRUMPKIN_FQ_MODULUS) != core::cmp::Ordering::Less {
            return Err(GrumpkinError::InvalidFqElement);
        }
        Ok(Self {
            limbs: fa::to_mont(arr, &GRUMPKIN_FQ_MODULUS, GRUMPKIN_FQ_INV, &GRUMPKIN_FQ_R2),
        })
    }

    /// Create from Montgomery limbs (unchecked).
    #[inline(always)]
    pub fn from_u64_arr_unchecked(arr: &[u64; 4]) -> Self {
        Self { limbs: *arr }
    }

    #[inline(always)]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            limbs: fa::add_mod(&self.limbs, &other.limbs, &GRUMPKIN_FQ_MODULUS),
        }
    }

    #[inline(always)]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            limbs: fa::sub_mod(&self.limbs, &other.limbs, &GRUMPKIN_FQ_MODULUS),
        }
    }

    #[inline(always)]
    pub fn neg(&self) -> Self {
        if self.is_zero() {
            *self
        } else {
            Self {
                limbs: fa::sub_mod(&[0u64; 4], &self.limbs, &GRUMPKIN_FQ_MODULUS),
            }
        }
    }

    #[inline(always)]
    pub fn dbl(&self) -> Self {
        self.add(self)
    }

    #[inline(always)]
    pub fn tpl(&self) -> Self {
        self.dbl().add(self)
    }

    #[inline(always)]
    pub fn mul(&self, other: &Self) -> Self {
        Self {
            limbs: fa::mont_mul(&self.limbs, &other.limbs, &GRUMPKIN_FQ_MODULUS, GRUMPKIN_FQ_INV),
        }
    }

    #[inline(always)]
    pub fn square(&self) -> Self {
        Self {
            limbs: fa::mont_square(&self.limbs, &GRUMPKIN_FQ_MODULUS, GRUMPKIN_FQ_INV),
        }
    }

    #[inline(always)]
    pub fn to_u64_arr_mont(&self) -> [u64; 4] {
        self.limbs
    }

    #[inline(always)]
    pub fn to_u64_arr_canonical(&self) -> [u64; 4] {
        fa::from_mont(&self.limbs, &GRUMPKIN_FQ_MODULUS, GRUMPKIN_FQ_INV)
    }

    /// returns self / other via inline advice + verification (guest only).
    #[cfg(all(
        not(feature = "host"),
        any(target_arch = "riscv32", target_arch = "riscv64")
    ))]
    #[inline(always)]
    fn div_impl(&self, other: &Self) -> Self {
        let mut c = Self::zero();
        unsafe {
            use crate::{GRUMPKIN_DIVQ_ADV_FUNCT3, GRUMPKIN_FUNCT7, INLINE_OPCODE};
            core::arch::asm!(
                ".insn r {opcode}, {funct3}, {funct7}, {rd}, {rs1}, {rs2}",
                opcode = const INLINE_OPCODE,
                funct3 = const GRUMPKIN_DIVQ_ADV_FUNCT3,
                funct7 = const GRUMPKIN_FUNCT7,
                rd = in(reg) c.limbs.as_mut_ptr(),
                rs1 = in(reg) self.limbs.as_ptr(),
                rs2 = in(reg) other.limbs.as_ptr(),
                options(nostack)
            );
        }
        let tmp = other.mul(&c);
        if is_ge_modulus(&c.limbs, &GRUMPKIN_FQ_MODULUS) || is_not_equal(&tmp.limbs, &self.limbs) {
            hcf();
        }
        c
    }

    #[cfg(all(
        not(feature = "host"),
        any(target_arch = "riscv32", target_arch = "riscv64")
    ))]
    #[inline(always)]
    pub fn div(&self, other: &Self) -> Self {
        if other.is_zero() {
            hcf();
        }
        self.div_impl(other)
    }

    #[cfg(all(
        not(feature = "host"),
        any(target_arch = "riscv32", target_arch = "riscv64")
    ))]
    #[inline(always)]
    pub fn div_unchecked(&self, other: &Self) -> Self {
        self.div_impl(other)
    }

    #[cfg(all(
        not(feature = "host"),
        not(any(target_arch = "riscv32", target_arch = "riscv64"))
    ))]
    pub fn div(&self, _other: &Self) -> Self {
        panic!("GrumpkinFq::div called on non-RISC-V target without host feature");
    }

    #[cfg(all(
        not(feature = "host"),
        not(any(target_arch = "riscv32", target_arch = "riscv64"))
    ))]
    pub fn div_unchecked(&self, _other: &Self) -> Self {
        panic!("GrumpkinFq::div_unchecked called on non-RISC-V target without host feature");
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    fn pow(&self, exp: &fa::U256) -> Self {
        // Square-and-multiply, MSB-first.
        let mut acc = Self::from_u64_arr_unchecked(&GRUMPKIN_FQ_ONE_MONT);
        let mut limb_idx = 4usize;
        while limb_idx > 0 {
            limb_idx -= 1;
            let limb = exp[limb_idx];
            let mut bit = 64usize;
            while bit > 0 {
                bit -= 1;
                acc = acc.square();
                if ((limb >> bit) & 1) == 1 {
                    acc = acc.mul(self);
                }
            }
        }
        acc
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    fn inv(&self) -> Self {
        // Fermat inverse: a^(p-2)
        let exp = fa::sub_u256(&GRUMPKIN_FQ_MODULUS, &[2u64, 0, 0, 0]).0;
        self.pow(&exp)
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    pub fn div(&self, other: &Self) -> Self {
        if other.is_zero() {
            hcf();
        }
        self.mul(&other.inv())
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    pub fn div_unchecked(&self, other: &Self) -> Self {
        self.mul(&other.inv())
    }

    #[inline(always)]
    pub fn negative_seventeen() -> Self {
        Self::from_u64_arr_unchecked(&GRUMPKIN_FQ_NEGATIVE_SEVENTEEN_MONT)
    }
}

/// Grumpkin scalar field element (Fr) in Montgomery form.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GrumpkinFr {
    limbs: fa::U256,
}

impl GrumpkinFr {
    #[inline(always)]
    pub fn zero() -> Self {
        Self { limbs: [0u64; 4] }
    }

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        fa::is_zero_u256(&self.limbs)
    }

    /// Create from canonical (non-Montgomery) limbs, validating `< r`.
    #[inline(always)]
    pub fn from_u64_arr(arr: &[u64; 4]) -> Result<Self, GrumpkinError> {
        if fa::cmp_u256(arr, &GRUMPKIN_FR_MODULUS) != core::cmp::Ordering::Less {
            return Err(GrumpkinError::InvalidFrElement);
        }
        Ok(Self {
            limbs: fa::to_mont(arr, &GRUMPKIN_FR_MODULUS, GRUMPKIN_FR_INV, &GRUMPKIN_FR_R2),
        })
    }

    #[inline(always)]
    pub fn from_u64_arr_unchecked(arr: &[u64; 4]) -> Self {
        Self { limbs: *arr }
    }

    #[inline(always)]
    fn from_u128_canonical(x: u128) -> Self {
        let arr = [x as u64, (x >> 64) as u64, 0u64, 0u64];
        // x < 2^128 << r, so canonical.
        Self {
            limbs: fa::to_mont(&arr, &GRUMPKIN_FR_MODULUS, GRUMPKIN_FR_INV, &GRUMPKIN_FR_R2),
        }
    }

    #[inline(always)]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            limbs: fa::add_mod(&self.limbs, &other.limbs, &GRUMPKIN_FR_MODULUS),
        }
    }

    #[inline(always)]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            limbs: fa::sub_mod(&self.limbs, &other.limbs, &GRUMPKIN_FR_MODULUS),
        }
    }

    #[inline(always)]
    pub fn neg(&self) -> Self {
        if self.is_zero() {
            *self
        } else {
            Self {
                limbs: fa::sub_mod(&[0u64; 4], &self.limbs, &GRUMPKIN_FR_MODULUS),
            }
        }
    }

    #[inline(always)]
    pub fn mul(&self, other: &Self) -> Self {
        Self {
            limbs: fa::mont_mul(&self.limbs, &other.limbs, &GRUMPKIN_FR_MODULUS, GRUMPKIN_FR_INV),
        }
    }

    #[inline(always)]
    pub fn square(&self) -> Self {
        Self {
            limbs: fa::mont_square(&self.limbs, &GRUMPKIN_FR_MODULUS, GRUMPKIN_FR_INV),
        }
    }

    #[inline(always)]
    pub fn to_u64_arr_mont(&self) -> [u64; 4] {
        self.limbs
    }

    #[inline(always)]
    pub fn to_u64_arr_canonical(&self) -> [u64; 4] {
        fa::from_mont(&self.limbs, &GRUMPKIN_FR_MODULUS, GRUMPKIN_FR_INV)
    }

    /// returns self / other via inline advice + verification (guest only).
    #[cfg(all(
        not(feature = "host"),
        any(target_arch = "riscv32", target_arch = "riscv64")
    ))]
    #[inline(always)]
    fn div_impl(&self, other: &Self) -> Self {
        let mut c = Self::zero();
        unsafe {
            use crate::{GRUMPKIN_DIVR_ADV_FUNCT3, GRUMPKIN_FUNCT7, INLINE_OPCODE};
            core::arch::asm!(
                ".insn r {opcode}, {funct3}, {funct7}, {rd}, {rs1}, {rs2}",
                opcode = const INLINE_OPCODE,
                funct3 = const GRUMPKIN_DIVR_ADV_FUNCT3,
                funct7 = const GRUMPKIN_FUNCT7,
                rd = in(reg) c.limbs.as_mut_ptr(),
                rs1 = in(reg) self.limbs.as_ptr(),
                rs2 = in(reg) other.limbs.as_ptr(),
                options(nostack)
            );
        }
        let tmp = other.mul(&c);
        if is_ge_modulus(&c.limbs, &GRUMPKIN_FR_MODULUS) || is_not_equal(&tmp.limbs, &self.limbs) {
            hcf();
        }
        c
    }

    #[cfg(all(
        not(feature = "host"),
        any(target_arch = "riscv32", target_arch = "riscv64")
    ))]
    #[inline(always)]
    pub fn div(&self, other: &Self) -> Self {
        if other.is_zero() {
            hcf();
        }
        self.div_impl(other)
    }

    #[cfg(all(
        not(feature = "host"),
        any(target_arch = "riscv32", target_arch = "riscv64")
    ))]
    #[inline(always)]
    pub fn div_unchecked(&self, other: &Self) -> Self {
        self.div_impl(other)
    }

    #[cfg(all(
        not(feature = "host"),
        not(any(target_arch = "riscv32", target_arch = "riscv64"))
    ))]
    pub fn div(&self, _other: &Self) -> Self {
        panic!("GrumpkinFr::div called on non-RISC-V target without host feature");
    }

    #[cfg(all(
        not(feature = "host"),
        not(any(target_arch = "riscv32", target_arch = "riscv64"))
    ))]
    pub fn div_unchecked(&self, _other: &Self) -> Self {
        panic!("GrumpkinFr::div_unchecked called on non-RISC-V target without host feature");
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    fn pow(&self, exp: &fa::U256) -> Self {
        let mut acc = Self::from_u64_arr_unchecked(&GRUMPKIN_FR_ONE_MONT);
        let mut limb_idx = 4usize;
        while limb_idx > 0 {
            limb_idx -= 1;
            let limb = exp[limb_idx];
            let mut bit = 64usize;
            while bit > 0 {
                bit -= 1;
                acc = acc.square();
                if ((limb >> bit) & 1) == 1 {
                    acc = acc.mul(self);
                }
            }
        }
        acc
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    fn inv(&self) -> Self {
        let exp = fa::sub_u256(&GRUMPKIN_FR_MODULUS, &[2u64, 0, 0, 0]).0;
        self.pow(&exp)
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    pub fn div(&self, other: &Self) -> Self {
        if other.is_zero() {
            hcf();
        }
        self.mul(&other.inv())
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    pub fn div_unchecked(&self, other: &Self) -> Self {
        self.mul(&other.inv())
    }
}

/// Grumpkin point in affine form (x, y) in Montgomery form.
/// Infinity is represented as (0, 0).
#[derive(Clone, PartialEq, Debug)]
pub struct GrumpkinPoint {
    x: GrumpkinFq,
    y: GrumpkinFq,
}

impl GrumpkinPoint {
    #[inline(always)]
    pub fn new(x: GrumpkinFq, y: GrumpkinFq) -> Result<Self, GrumpkinError> {
        let p = Self { x, y };
        if p.is_on_curve() {
            Ok(p)
        } else {
            Err(GrumpkinError::NotOnCurve)
        }
    }

    #[inline(always)]
    pub fn new_unchecked(x: GrumpkinFq, y: GrumpkinFq) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub fn to_u64_arr(&self) -> [u64; 8] {
        let mut arr = [0u64; 8];
        arr[0..4].copy_from_slice(&self.x.to_u64_arr_mont());
        arr[4..8].copy_from_slice(&self.y.to_u64_arr_mont());
        arr
    }

    #[inline(always)]
    pub fn to_u64_arr_canonical(&self) -> [u64; 8] {
        let mut arr = [0u64; 8];
        arr[0..4].copy_from_slice(&self.x.to_u64_arr_canonical());
        arr[4..8].copy_from_slice(&self.y.to_u64_arr_canonical());
        arr
    }

    #[inline(always)]
    pub fn from_u64_arr(arr: &[u64; 8]) -> Result<Self, GrumpkinError> {
        let x = GrumpkinFq::from_u64_arr(&[arr[0], arr[1], arr[2], arr[3]])?;
        let y = GrumpkinFq::from_u64_arr(&[arr[4], arr[5], arr[6], arr[7]])?;
        Self::new(x, y)
    }

    #[inline(always)]
    pub fn from_u64_arr_unchecked(arr: &[u64; 8]) -> Self {
        let x = GrumpkinFq::from_u64_arr_unchecked(&[arr[0], arr[1], arr[2], arr[3]]);
        let y = GrumpkinFq::from_u64_arr_unchecked(&[arr[4], arr[5], arr[6], arr[7]]);
        Self { x, y }
    }

    #[inline(always)]
    pub fn x(&self) -> GrumpkinFq {
        self.x
    }

    #[inline(always)]
    pub fn y(&self) -> GrumpkinFq {
        self.y
    }

    #[inline(always)]
    pub fn generator() -> Self {
        Self {
            x: GrumpkinFq::from_u64_arr_unchecked(&GRUMPKIN_G_GENERATOR_X_MONT),
            y: GrumpkinFq::from_u64_arr_unchecked(&GRUMPKIN_G_GENERATOR_Y_MONT),
        }
    }

    #[inline(always)]
    pub fn generator_w_endomorphism() -> Self {
        Self::generator().endomorphism()
    }

    #[inline(always)]
    pub fn endomorphism(&self) -> Self {
        if self.is_infinity() {
            Self::infinity()
        } else {
            let beta = GrumpkinFq::from_u64_arr_unchecked(&GRUMPKIN_ENDO_BETA_MONT);
            Self {
                x: self.x.mul(&beta),
                y: self.y,
            }
        }
    }

    #[cfg(all(
        not(feature = "host"),
        any(target_arch = "riscv32", target_arch = "riscv64")
    ))]
    #[inline(always)]
    pub fn decompose_scalar(k: &GrumpkinFr) -> [(bool, u128); 2] {
        let mut out = [0u64; 6];
        unsafe {
            use crate::{GRUMPKIN_FUNCT7, GRUMPKIN_GLVR_ADV_FUNCT3, INLINE_OPCODE};
            core::arch::asm!(
                ".insn r {opcode}, {funct3}, {funct7}, {rd}, {rs1}, x0",
                opcode = const INLINE_OPCODE,
                funct3 = const GRUMPKIN_GLVR_ADV_FUNCT3,
                funct7 = const GRUMPKIN_FUNCT7,
                rd = in(reg) out.as_mut_ptr(),
                rs1 = in(reg) k.limbs.as_ptr(),
                options(nostack)
            );
        }

        // Decode (sign, abs) pairs.
        let sign1 = out[0] == 1u64;
        let k1_u = (out[1] as u128) | ((out[2] as u128) << 64);
        let sign2 = out[3] == 1u64;
        let k2_u = (out[4] as u128) | ((out[5] as u128) << 64);

        // Verify recomposition: k ≡ k1 + k2·lambda (mod r)
        let lambda = GrumpkinFr::from_u64_arr_unchecked(&GRUMPKIN_GLV_LAMBDA_MONT);

        let mut k1 = GrumpkinFr::from_u128_canonical(k1_u);
        if sign1 {
            k1 = k1.neg();
        }
        let mut k2 = GrumpkinFr::from_u128_canonical(k2_u);
        if sign2 {
            k2 = k2.neg();
        }
        let recomposed = k1.add(&k2.mul(&lambda));
        if recomposed.limbs != k.limbs {
            hcf();
        }

        [(sign1, k1_u), (sign2, k2_u)]
    }

    #[cfg(all(
        not(feature = "host"),
        not(any(target_arch = "riscv32", target_arch = "riscv64"))
    ))]
    pub fn decompose_scalar(_k: &GrumpkinFr) -> [(bool, u128); 2] {
        panic!("GrumpkinPoint::decompose_scalar called on non-RISC-V target without host feature");
    }

    #[cfg(feature = "host")]
    pub fn decompose_scalar(_k: &GrumpkinFr) -> [(bool, u128); 2] {
        panic!("GrumpkinPoint::decompose_scalar called on host without arkworks backend");
    }

    #[inline(always)]
    pub fn infinity() -> Self {
        Self {
            x: GrumpkinFq::zero(),
            y: GrumpkinFq::zero(),
        }
    }

    #[inline(always)]
    pub fn is_infinity(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }

    #[inline(always)]
    pub fn is_on_curve(&self) -> bool {
        self.is_infinity()
            || self.y.square()
                == self
                    .x
                    .square()
                    .mul(&self.x)
                    .add(&GrumpkinFq::negative_seventeen())
    }

    #[inline(always)]
    pub fn neg(&self) -> Self {
        if self.is_infinity() {
            Self::infinity()
        } else {
            Self {
                x: self.x,
                y: self.y.neg(),
            }
        }
    }

    #[inline(always)]
    pub fn double(&self) -> Self {
        if self.y.is_zero() {
            Self::infinity()
        } else {
            let s = self.x.square().tpl().div_unchecked(&self.y.dbl());
            let x2 = s.square().sub(&self.x.dbl());
            let y2 = s.mul(&self.x.sub(&x2)).sub(&self.y);
            Self { x: x2, y: y2 }
        }
    }

    #[inline(always)]
    pub fn add(&self, other: &Self) -> Self {
        if self.is_infinity() {
            return other.clone();
        }
        if other.is_infinity() {
            return self.clone();
        }
        if self.x == other.x {
            if self.y == other.y {
                return self.double();
            }
            return Self::infinity();
        }

        let dy = self.y.sub(&other.y);
        let dx = self.x.sub(&other.x);
        let s = dy.div_unchecked(&dx);
        let x2 = s.square().sub(&self.x).sub(&other.x);
        let y2 = s.mul(&self.x.sub(&x2)).sub(&self.y);
        Self { x: x2, y: y2 }
    }

    #[inline(always)]
    pub fn double_and_add(&self, other: &Self) -> Self {
        if self.is_infinity() {
            other.clone()
        } else if other.is_infinity() {
            self.add(self)
        } else if self.x == other.x && self.y == other.y {
            self.add(self).add(other)
        } else if self.x == other.x && self.y != other.y {
            self.clone()
        } else {
            let s = self.y.sub(&other.y).div_unchecked(&self.x.sub(&other.x));
            let x2 = s.square().sub(&self.x).sub(&other.x);
            let denom = self.x.sub(&x2);
            if denom.is_zero() {
                return self.double().add(other);
            }
            let t = self.y.dbl().div(&denom).sub(&s);
            let x3 = t.square().sub(&self.x).sub(&x2);
            let y3 = t.mul(&self.x.sub(&x3)).sub(&self.y);
            Self { x: x3, y: y3 }
        }
    }
}

