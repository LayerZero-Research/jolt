//! Secp256k1 guest backend without Arkworks.
//!
//! Field elements are represented as 4 little-endian `u64` limbs in Montgomery form.

// This module contains many helpers that are only exercised in certain build modes
// (guest vs host equivalence tests). CI runs with `-D warnings`, so we silence
// `dead_code` here intentionally.
#![allow(dead_code)]

use crate::constants::*;

use jolt_inlines_field_arith as fa;

extern crate alloc;
use alloc::vec::Vec;

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

impl<T> UnwrapOrSpoilProof<T> for Result<T, Secp256k1Error> {
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

/// Error types for secp256k1 operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Secp256k1Error {
    InvalidFqElement,
    InvalidFrElement,
    NotOnCurve,
    QAtInfinity,
    ROrSZero,
    RxMismatch,
}

/// Secp256k1 base field element (Fq) in Montgomery form.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Secp256k1Fq {
    limbs: fa::U256,
}

impl Secp256k1Fq {
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
    pub fn from_u64_arr(arr: &[u64; 4]) -> Result<Self, Secp256k1Error> {
        if fa::cmp_u256(arr, &SECP256K1_FQ_MODULUS) != core::cmp::Ordering::Less {
            return Err(Secp256k1Error::InvalidFqElement);
        }
        Ok(Self {
            limbs: fa::to_mont(arr, &SECP256K1_FQ_MODULUS, SECP256K1_FQ_INV, &SECP256K1_FQ_R2),
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
            limbs: fa::add_mod(&self.limbs, &other.limbs, &SECP256K1_FQ_MODULUS),
        }
    }

    #[inline(always)]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            limbs: fa::sub_mod(&self.limbs, &other.limbs, &SECP256K1_FQ_MODULUS),
        }
    }

    #[inline(always)]
    pub fn neg(&self) -> Self {
        if self.is_zero() {
            *self
        } else {
            Self {
                limbs: fa::sub_mod(&[0u64; 4], &self.limbs, &SECP256K1_FQ_MODULUS),
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
            limbs: fa::mont_mul(&self.limbs, &other.limbs, &SECP256K1_FQ_MODULUS, SECP256K1_FQ_INV),
        }
    }

    #[inline(always)]
    pub fn square(&self) -> Self {
        Self {
            limbs: fa::mont_square(&self.limbs, &SECP256K1_FQ_MODULUS, SECP256K1_FQ_INV),
        }
    }

    #[inline(always)]
    pub fn to_u64_arr_mont(&self) -> [u64; 4] {
        self.limbs
    }

    #[inline(always)]
    pub fn to_u64_arr_canonical(&self) -> [u64; 4] {
        fa::from_mont(&self.limbs, &SECP256K1_FQ_MODULUS, SECP256K1_FQ_INV)
    }

    #[inline(always)]
    pub fn seven() -> Self {
        Self::from_u64_arr_unchecked(&SECP256K1_FQ_SEVEN_MONT)
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
            use crate::{INLINE_OPCODE, SECP256K1_DIVQ_ADV_FUNCT3, SECP256K1_FUNCT7};
            core::arch::asm!(
                ".insn r {opcode}, {funct3}, {funct7}, {rd}, {rs1}, {rs2}",
                opcode = const INLINE_OPCODE,
                funct3 = const SECP256K1_DIVQ_ADV_FUNCT3,
                funct7 = const SECP256K1_FUNCT7,
                rd = in(reg) c.limbs.as_mut_ptr(),
                rs1 = in(reg) self.limbs.as_ptr(),
                rs2 = in(reg) other.limbs.as_ptr(),
                options(nostack)
            );
        }
        let tmp = other.mul(&c);
        if is_ge_modulus(&c.limbs, &SECP256K1_FQ_MODULUS) || is_not_equal(&tmp.limbs, &self.limbs) {
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
        panic!("Secp256k1Fq::div called on non-RISC-V target without host feature");
    }

    #[cfg(all(
        not(feature = "host"),
        not(any(target_arch = "riscv32", target_arch = "riscv64"))
    ))]
    pub fn div_unchecked(&self, _other: &Self) -> Self {
        panic!("Secp256k1Fq::div_unchecked called on non-RISC-V target without host feature");
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    fn pow(&self, exp: &fa::U256) -> Self {
        let mut acc = Self::from_u64_arr_unchecked(&SECP256K1_FQ_ONE_MONT);
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
        let exp = fa::sub_u256(&SECP256K1_FQ_MODULUS, &[2u64, 0, 0, 0]).0;
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

/// Secp256k1 scalar field element (Fr) in Montgomery form.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Secp256k1Fr {
    limbs: fa::U256,
}

impl Secp256k1Fr {
    #[inline(always)]
    pub fn zero() -> Self {
        Self { limbs: [0u64; 4] }
    }

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        fa::is_zero_u256(&self.limbs)
    }

    /// Create from canonical (non-Montgomery) limbs, validating `< n`.
    #[inline(always)]
    pub fn from_u64_arr(arr: &[u64; 4]) -> Result<Self, Secp256k1Error> {
        if fa::cmp_u256(arr, &SECP256K1_FR_MODULUS) != core::cmp::Ordering::Less {
            return Err(Secp256k1Error::InvalidFrElement);
        }
        Ok(Self {
            limbs: fa::to_mont(arr, &SECP256K1_FR_MODULUS, SECP256K1_FR_INV, &SECP256K1_FR_R2),
        })
    }

    #[inline(always)]
    pub fn from_u64_arr_unchecked(arr: &[u64; 4]) -> Self {
        Self { limbs: *arr }
    }

    #[inline(always)]
    fn from_u128_canonical(x: u128) -> Self {
        let arr = [x as u64, (x >> 64) as u64, 0u64, 0u64];
        Self {
            limbs: fa::to_mont(&arr, &SECP256K1_FR_MODULUS, SECP256K1_FR_INV, &SECP256K1_FR_R2),
        }
    }

    #[inline(always)]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            limbs: fa::add_mod(&self.limbs, &other.limbs, &SECP256K1_FR_MODULUS),
        }
    }

    #[inline(always)]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            limbs: fa::sub_mod(&self.limbs, &other.limbs, &SECP256K1_FR_MODULUS),
        }
    }

    #[inline(always)]
    pub fn neg(&self) -> Self {
        if self.is_zero() {
            *self
        } else {
            Self {
                limbs: fa::sub_mod(&[0u64; 4], &self.limbs, &SECP256K1_FR_MODULUS),
            }
        }
    }

    #[inline(always)]
    pub fn mul(&self, other: &Self) -> Self {
        Self {
            limbs: fa::mont_mul(&self.limbs, &other.limbs, &SECP256K1_FR_MODULUS, SECP256K1_FR_INV),
        }
    }

    #[inline(always)]
    pub fn square(&self) -> Self {
        Self {
            limbs: fa::mont_square(&self.limbs, &SECP256K1_FR_MODULUS, SECP256K1_FR_INV),
        }
    }

    #[inline(always)]
    pub fn to_u64_arr_mont(&self) -> [u64; 4] {
        self.limbs
    }

    #[inline(always)]
    pub fn to_u64_arr_canonical(&self) -> [u64; 4] {
        fa::from_mont(&self.limbs, &SECP256K1_FR_MODULUS, SECP256K1_FR_INV)
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
            use crate::{INLINE_OPCODE, SECP256K1_DIVR_ADV_FUNCT3, SECP256K1_FUNCT7};
            core::arch::asm!(
                ".insn r {opcode}, {funct3}, {funct7}, {rd}, {rs1}, {rs2}",
                opcode = const INLINE_OPCODE,
                funct3 = const SECP256K1_DIVR_ADV_FUNCT3,
                funct7 = const SECP256K1_FUNCT7,
                rd = in(reg) c.limbs.as_mut_ptr(),
                rs1 = in(reg) self.limbs.as_ptr(),
                rs2 = in(reg) other.limbs.as_ptr(),
                options(nostack)
            );
        }
        let tmp = other.mul(&c);
        if is_ge_modulus(&c.limbs, &SECP256K1_FR_MODULUS) || is_not_equal(&tmp.limbs, &self.limbs) {
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
        panic!("Secp256k1Fr::div called on non-RISC-V target without host feature");
    }

    #[cfg(all(
        not(feature = "host"),
        not(any(target_arch = "riscv32", target_arch = "riscv64"))
    ))]
    pub fn div_unchecked(&self, _other: &Self) -> Self {
        panic!("Secp256k1Fr::div_unchecked called on non-RISC-V target without host feature");
    }

    #[cfg(feature = "host")]
    #[inline(always)]
    fn pow(&self, exp: &fa::U256) -> Self {
        let mut acc = Self::from_u64_arr_unchecked(&SECP256K1_FR_ONE_MONT);
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
        let exp = fa::sub_u256(&SECP256K1_FR_MODULUS, &[2u64, 0, 0, 0]).0;
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

/// Secp256k1 point in affine form (x, y) in Montgomery form.
/// Infinity is represented as (0, 0).
#[derive(Clone, PartialEq, Debug)]
pub struct Secp256k1Point {
    x: Secp256k1Fq,
    y: Secp256k1Fq,
}

impl Secp256k1Point {
    #[inline(always)]
    pub fn new(x: Secp256k1Fq, y: Secp256k1Fq) -> Result<Self, Secp256k1Error> {
        let p = Self { x, y };
        if p.is_on_curve() {
            Ok(p)
        } else {
            Err(Secp256k1Error::NotOnCurve)
        }
    }

    #[inline(always)]
    pub fn new_unchecked(x: Secp256k1Fq, y: Secp256k1Fq) -> Self {
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
    pub fn from_u64_arr(arr: &[u64; 8]) -> Result<Self, Secp256k1Error> {
        let x = Secp256k1Fq::from_u64_arr(&[arr[0], arr[1], arr[2], arr[3]])?;
        let y = Secp256k1Fq::from_u64_arr(&[arr[4], arr[5], arr[6], arr[7]])?;
        Self::new(x, y)
    }

    #[inline(always)]
    pub fn from_u64_arr_unchecked(arr: &[u64; 8]) -> Self {
        let x = Secp256k1Fq::from_u64_arr_unchecked(&[arr[0], arr[1], arr[2], arr[3]]);
        let y = Secp256k1Fq::from_u64_arr_unchecked(&[arr[4], arr[5], arr[6], arr[7]]);
        Self { x, y }
    }

    #[inline(always)]
    pub fn x(&self) -> Secp256k1Fq {
        self.x
    }

    #[inline(always)]
    pub fn y(&self) -> Secp256k1Fq {
        self.y
    }

    #[inline(always)]
    pub fn generator() -> Self {
        Self {
            x: Secp256k1Fq::from_u64_arr_unchecked(&SECP256K1_G_GENERATOR_X_MONT),
            y: Secp256k1Fq::from_u64_arr_unchecked(&SECP256K1_G_GENERATOR_Y_MONT),
        }
    }

    #[inline(always)]
    pub fn generator_w_endomorphism() -> Self {
        Self::from_u64_arr_unchecked(&SECP256K1_G_GENERATOR_W_ENDO_MONT)
    }

    #[inline(always)]
    pub fn infinity() -> Self {
        Self {
            x: Secp256k1Fq::zero(),
            y: Secp256k1Fq::zero(),
        }
    }

    #[inline(always)]
    pub fn is_infinity(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }

    #[inline(always)]
    pub fn is_on_curve(&self) -> bool {
        self.is_infinity()
            || self.y.square() == self.x.square().mul(&self.x).add(&Secp256k1Fq::seven())
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
            other.clone()
        } else if other.is_infinity() {
            self.clone()
        } else if self.x == other.x && self.y == other.y {
            self.double()
        } else if self.x == other.x && self.y != other.y {
            Self::infinity()
        } else {
            let s = self.y.sub(&other.y).div_unchecked(&self.x.sub(&other.x));
            let x2 = s.square().sub(&self.x).sub(&other.x);
            let y2 = s.mul(&self.x.sub(&x2)).sub(&self.y);
            Self { x: x2, y: y2 }
        }
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
            let t = self.y.dbl().div(&self.x.sub(&x2)).sub(&s);
            let x3 = t.square().sub(&self.x).sub(&x2);
            let y3 = t.mul(&self.x.sub(&x3)).sub(&self.y);
            Self { x: x3, y: y3 }
        }
    }

    #[inline(always)]
    pub fn endomorphism(&self) -> Self {
        if self.is_infinity() {
            Self::infinity()
        } else {
            let beta = Secp256k1Fq::from_u64_arr_unchecked(&SECP256K1_ENDO_BETA_MONT);
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
    pub fn decompose_scalar(k: &Secp256k1Fr) -> [(bool, u128); 2] {
        let mut out = [0u64; 6];
        unsafe {
            use crate::{INLINE_OPCODE, SECP256K1_FUNCT7, SECP256K1_GLVR_ADV_FUNCT3};
            core::arch::asm!(
                ".insn r {opcode}, {funct3}, {funct7}, {rd}, {rs1}, x0",
                opcode = const INLINE_OPCODE,
                funct3 = const SECP256K1_GLVR_ADV_FUNCT3,
                funct7 = const SECP256K1_FUNCT7,
                rd = in(reg) out.as_mut_ptr(),
                rs1 = in(reg) k.limbs.as_ptr(),
                options(nostack)
            );
        }

        let sign1 = out[0] == 1u64;
        let k1_u = (out[1] as u128) | ((out[2] as u128) << 64);
        let sign2 = out[3] == 1u64;
        let k2_u = (out[4] as u128) | ((out[5] as u128) << 64);

        let lambda = Secp256k1Fr::from_u64_arr_unchecked(&SECP256K1_GLV_LAMBDA_MONT);
        let mut k1 = Secp256k1Fr::from_u128_canonical(k1_u);
        if sign1 {
            k1 = k1.neg();
        }
        let mut k2 = Secp256k1Fr::from_u128_canonical(k2_u);
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
    pub fn decompose_scalar(_k: &Secp256k1Fr) -> [(bool, u128); 2] {
        panic!("Secp256k1Point::decompose_scalar called on non-RISC-V target without host feature");
    }

    #[cfg(feature = "host")]
    pub fn decompose_scalar(_k: &Secp256k1Fr) -> [(bool, u128); 2] {
        panic!("Secp256k1Point::decompose_scalar called on host without arkworks backend");
    }
}

// ECDSA signature verification function + helpers

#[inline(always)]
fn secp256k1_4x128_scalar_mul(scalars: [u128; 4], points: [Secp256k1Point; 4]) -> Secp256k1Point {
    let mut lookup = Vec::<Secp256k1Point>::with_capacity(16);
    lookup.push(Secp256k1Point::infinity());
    lookup.push(points[0].clone());
    lookup.push(points[1].clone());
    lookup.push(lookup[1].add(&lookup[2]));
    lookup.push(points[2].clone());
    lookup.push(lookup[1].add(&lookup[4]));
    lookup.push(lookup[2].add(&lookup[4]));
    lookup.push(lookup[1].add(&lookup[6]));
    lookup.push(points[3].clone());
    for i in 1..8 {
        lookup.push(lookup[i].add(&lookup[8]));
    }
    let mut res = Secp256k1Point::infinity();
    for i in (0..128).rev() {
        let mut idx = 0usize;
        for (j, scalar) in scalars.iter().enumerate() {
            if (scalar >> i) & 1 == 1 {
                idx |= 1 << j;
            }
        }
        if idx != 0 {
            res = res.double_and_add(&lookup[idx]);
        } else {
            res = res.double();
        }
    }
    res
}

#[inline(always)]
fn conditional_negate(x: Secp256k1Point, cond: bool) -> Secp256k1Point {
    if cond { x.neg() } else { x }
}

/// Verify an ECDSA signature.
#[inline(always)]
pub fn ecdsa_verify(
    z: Secp256k1Fr,
    r: Secp256k1Fr,
    s: Secp256k1Fr,
    q: Secp256k1Point,
) -> Result<(), Secp256k1Error> {
    if q.is_infinity() {
        return Err(Secp256k1Error::QAtInfinity);
    }
    if r.is_zero() || s.is_zero() {
        return Err(Secp256k1Error::ROrSZero);
    }

    let u1 = z.div_unchecked(&s);
    let u2 = r.div_unchecked(&s);

    let decomp_u = Secp256k1Point::decompose_scalar(&u1);
    let decomp_v = Secp256k1Point::decompose_scalar(&u2);
    let scalars = [decomp_u[0].1, decomp_u[1].1, decomp_v[0].1, decomp_v[1].1];

    let points = [
        conditional_negate(Secp256k1Point::generator(), decomp_u[0].0),
        conditional_negate(Secp256k1Point::generator_w_endomorphism(), decomp_u[1].0),
        conditional_negate(q.clone(), decomp_v[0].0),
        conditional_negate(q.endomorphism(), decomp_v[1].0),
    ];
    let r_claim = secp256k1_4x128_scalar_mul(scalars, points);

    // Check r == x_R mod n, with a single conditional subtraction (since p < 2n).
    let mut x_r = r_claim.x.to_u64_arr_canonical();
    if fa::cmp_u256(&x_r, &SECP256K1_FR_MODULUS) != core::cmp::Ordering::Less {
        x_r = fa::sub_u256(&x_r, &SECP256K1_FR_MODULUS).0;
    }
    let r_canon = r.to_u64_arr_canonical();
    if x_r != r_canon {
        return Err(Secp256k1Error::RxMismatch);
    }
    Ok(())
}

