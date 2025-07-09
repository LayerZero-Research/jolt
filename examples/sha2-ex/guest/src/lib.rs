#![cfg_attr(feature = "guest", no_std)]

use sha2::Digest;

#[jolt::provable]
fn sha2(input: &[u8]) -> [u8; 32] {
    // Use Jolt's optimized SHA256 implementation
    sha2::Sha256::digest(input).into()
}
