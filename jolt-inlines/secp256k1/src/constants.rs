//! Curve + field constants for Secp256k1 in Montgomery form.
//!
//! All limbs are little-endian `u64` words.

// Many of these constants are only used in specific build modes (guest vs host tests).
#![allow(dead_code)]

/// Secp256k1 base field modulus \(p\).
pub const SECP256K1_FQ_MODULUS: [u64; 4] = [
    0xfffffffefffffc2f,
    0xffffffffffffffff,
    0xffffffffffffffff,
    0xffffffffffffffff,
];

/// Montgomery constant: `INV = -p^{-1} mod 2^64`.
pub const SECP256K1_FQ_INV: u64 = 0xd838091dd2253531;

/// `ONE_MONT = R mod p` where `R = 2^256`.
pub const SECP256K1_FQ_ONE_MONT: [u64; 4] = [0x00000001000003d1, 0, 0, 0];

/// `R2 = R^2 mod p` in little-endian limbs.
pub const SECP256K1_FQ_R2: [u64; 4] = [0x000007a2000e90a1, 0x1, 0, 0];

/// Secp256k1 scalar field modulus \(n\).
pub const SECP256K1_FR_MODULUS: [u64; 4] = [
    0xbfd25e8cd0364141,
    0xbaaedce6af48a03b,
    0xfffffffffffffffe,
    0xffffffffffffffff,
];

/// Montgomery constant: `INV = -n^{-1} mod 2^64`.
pub const SECP256K1_FR_INV: u64 = 0x4b0dff665588b13f;

/// `ONE_MONT = R mod n` where `R = 2^256`.
pub const SECP256K1_FR_ONE_MONT: [u64; 4] = [
    0x402da1732fc9bebf,
    0x4551231950b75fc4,
    0x0000000000000001,
    0x0000000000000000,
];

/// `R2 = R^2 mod n` in little-endian limbs.
pub const SECP256K1_FR_R2: [u64; 4] = [
    0x896cf21467d7d140,
    0x741496c20e7cf878,
    0xe697f5e45bcd07c6,
    0x9d671cd581c69bc5,
];

/// Secp256k1 curve generator X coordinate (Fq) in Montgomery form.
pub const SECP256K1_G_GENERATOR_X_MONT: [u64; 4] = [
    0xd7362e5a487e2097,
    0x231e295329bc66db,
    0x979f48c033fd129c,
    0x9981e643e9089f48,
];

/// Secp256k1 curve generator Y coordinate (Fq) in Montgomery form.
pub const SECP256K1_G_GENERATOR_Y_MONT: [u64; 4] = [
    0xb15ea6d2d3dbabe2,
    0x8dfc5d5d1f1dc64d,
    0x70b6b59aac19c136,
    0xcf3f851fd4a582d6,
];

/// Secp256k1 base-field constant `7` in Montgomery form.
pub const SECP256K1_FQ_SEVEN_MONT: [u64; 4] = [30064777911u64, 0, 0, 0];

/// Secp256k1 endomorphism beta constant (Fq) in Montgomery form.
pub const SECP256K1_ENDO_BETA_MONT: [u64; 4] = [
    6387289667796044110u64,
    287633767014301871u64,
    17936018142961481989u64,
    8811915745022393683u64,
];

/// Secp256k1 GLV lambda constant (Fr) in Montgomery form.
pub const SECP256K1_GLV_LAMBDA_MONT: [u64; 4] = [
    17329265591798885534,
    3212165691671483468,
    8334304762764295569,
    5992109773982062137,
];

/// Generator with endomorphism applied, in Montgomery limbs (x||y).
pub const SECP256K1_G_GENERATOR_W_ENDO_MONT: [u64; 8] = [
    16173582788404280516,
    5747022314874861025,
    3849308819804808767,
    12496950317914431610,
    12780836216951778274,
    10231155108014310989,
    8121878653926228278,
    14933801261141951190,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "arkworks")]
    mod arkworks_checks {
        use super::*;
        use ark_ff::{MontConfig, PrimeField};

        #[test]
        fn secp256k1_fq_constants_match_arkworks() {
            assert_eq!(ark_secp256k1::Fq::MODULUS.0, SECP256K1_FQ_MODULUS);
            assert_eq!(
                <ark_secp256k1::FqConfig as MontConfig<4>>::INV,
                SECP256K1_FQ_INV
            );
            assert_eq!(
                <ark_secp256k1::FqConfig as MontConfig<4>>::R2.0,
                SECP256K1_FQ_R2
            );
            assert_eq!(
                ark_secp256k1::G_GENERATOR_X.0 .0,
                SECP256K1_G_GENERATOR_X_MONT
            );
            assert_eq!(
                ark_secp256k1::G_GENERATOR_Y.0 .0,
                SECP256K1_G_GENERATOR_Y_MONT
            );
        }

        #[test]
        fn secp256k1_fr_constants_match_arkworks() {
            assert_eq!(ark_secp256k1::Fr::MODULUS.0, SECP256K1_FR_MODULUS);
            assert_eq!(
                <ark_secp256k1::FrConfig as MontConfig<4>>::INV,
                SECP256K1_FR_INV
            );
            assert_eq!(
                <ark_secp256k1::FrConfig as MontConfig<4>>::R2.0,
                SECP256K1_FR_R2
            );
        }

        #[test]
        fn secp256k1_precomputed_constants_sane() {
            let seven = ark_secp256k1::Fq::from(7u64);
            assert_eq!(seven.0 .0, SECP256K1_FQ_SEVEN_MONT);
        }
    }
}

