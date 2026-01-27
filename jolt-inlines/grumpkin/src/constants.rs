//! Curve + field constants for Grumpkin in Montgomery form.
//!
//! All limbs are little-endian `u64` words.

// Many of these constants are only used in specific build modes (guest vs host tests).
#![allow(dead_code)]

/// Grumpkin base field modulus \(p\) (a.k.a. BN254 scalar field modulus).
pub const GRUMPKIN_FQ_MODULUS: [u64; 4] = [
    0x43e1f593f0000001,
    0x2833e84879b97091,
    0xb85045b68181585d,
    0x30644e72e131a029,
];

/// Montgomery constant: `INV = -p^{-1} mod 2^64`.
pub const GRUMPKIN_FQ_INV: u64 = 0xc2e1f593efffffff;

/// `ONE_MONT = R mod p` where `R = 2^256`.
pub const GRUMPKIN_FQ_ONE_MONT: [u64; 4] = [
    0xac96341c4ffffffb,
    0x36fc76959f60cd29,
    0x666ea36f7879462e,
    0x0e0a77c19a07df2f,
];

/// `R2 = R^2 mod p` in little-endian limbs.
pub const GRUMPKIN_FQ_R2: [u64; 4] = [
    0x1bb8e645ae216da7,
    0x53fe3ab1e35c59e3,
    0x8c49833d53bb8085,
    0x0216d0b17f4e44a5,
];

/// Grumpkin scalar field modulus \(r\) (a.k.a. BN254 base field modulus).
pub const GRUMPKIN_FR_MODULUS: [u64; 4] = [
    0x3c208c16d87cfd47,
    0x97816a916871ca8d,
    0xb85045b68181585d,
    0x30644e72e131a029,
];

/// Montgomery constant: `INV = -r^{-1} mod 2^64`.
pub const GRUMPKIN_FR_INV: u64 = 0x87d20782e4866389;

/// `ONE_MONT = R mod r` where `R = 2^256`.
pub const GRUMPKIN_FR_ONE_MONT: [u64; 4] = [
    0xd35d438dc58f0d9d,
    0x0a78eb28f5c70b3d,
    0x666ea36f7879462c,
    0x0e0a77c19a07df2f,
];

/// `R2 = R^2 mod r` in little-endian limbs.
pub const GRUMPKIN_FR_R2: [u64; 4] = [
    0xf32cfc5b538afa89,
    0xb5e71911d44501fb,
    0x47ab1eff0a417ff6,
    0x06d89f71cab8351f,
];

/// Grumpkin curve generator X coordinate (Fq) in Montgomery form.
pub const GRUMPKIN_G_GENERATOR_X_MONT: [u64; 4] = GRUMPKIN_FQ_ONE_MONT;

/// Grumpkin curve generator Y coordinate (Fq) in Montgomery form.
pub const GRUMPKIN_G_GENERATOR_Y_MONT: [u64; 4] = [
    0x11b2dff1448c41d8,
    0x23d3446f21c77dc3,
    0xaa7b8cf435dfafbb,
    0x14b34cf69dc25d68,
];

/// Grumpkin GLV endomorphism beta constant (Fq) in Montgomery form.
pub const GRUMPKIN_ENDO_BETA_MONT: [u64; 4] = [
    244305545194690131,
    8351807910065594880,
    14266533074055306532,
    404339206190769364,
];

/// Grumpkin GLV lambda constant (Fr) in Montgomery form.
pub const GRUMPKIN_GLV_LAMBDA_MONT: [u64; 4] = [
    3697675806616062876,
    9065277094688085689,
    6918009208039626314,
    2775033306905974752,
];

/// `-17` in Grumpkin Fq, in Montgomery form. Used in curve arithmetic.
pub const GRUMPKIN_FQ_NEGATIVE_SEVENTEEN_MONT: [u64; 4] = [
    0xdd7056026000005a,
    0x223fa97acb319311,
    0xcc388229877910c0,
    0x034394632b724eaa,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "arkworks")]
    mod arkworks_checks {
        use super::*;
        use ark_ff::{MontConfig, PrimeField};

        #[test]
        fn grumpkin_fq_constants_match_arkworks() {
            assert_eq!(ark_grumpkin::Fq::MODULUS.0, GRUMPKIN_FQ_MODULUS);
            assert_eq!(
                <ark_grumpkin::FqConfig as MontConfig<4>>::INV,
                GRUMPKIN_FQ_INV
            );
            assert_eq!(
                <ark_grumpkin::FqConfig as MontConfig<4>>::R2.0,
                GRUMPKIN_FQ_R2
            );
            assert_eq!(
                ark_grumpkin::G_GENERATOR_X.0 .0,
                GRUMPKIN_G_GENERATOR_X_MONT
            );
            assert_eq!(
                ark_grumpkin::G_GENERATOR_Y.0 .0,
                GRUMPKIN_G_GENERATOR_Y_MONT
            );
        }

        #[test]
        fn grumpkin_fr_constants_match_arkworks() {
            assert_eq!(ark_grumpkin::Fr::MODULUS.0, GRUMPKIN_FR_MODULUS);
            assert_eq!(
                <ark_grumpkin::FrConfig as MontConfig<4>>::INV,
                GRUMPKIN_FR_INV
            );
            assert_eq!(
                <ark_grumpkin::FrConfig as MontConfig<4>>::R2.0,
                GRUMPKIN_FR_R2
            );
        }

        #[test]
        fn grumpkin_precomputed_constants_sane() {
            let neg17 = -ark_grumpkin::Fq::from(17u64);
            assert_eq!(neg17.0 .0, GRUMPKIN_FQ_NEGATIVE_SEVENTEEN_MONT);
        }
    }
}

