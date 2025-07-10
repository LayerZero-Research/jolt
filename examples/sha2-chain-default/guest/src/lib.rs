#![cfg_attr(feature = "guest", no_std)]

use sha2::Digest;

#[jolt::provable]
fn sha2_chain_default(input: [u8; 32], num_iters: u32) -> [u8; 32] {
    let mut hash = input;
    for _ in 0..num_iters {
        hash = sha2::Sha256::digest(&hash).into();
    }
    hash
}
