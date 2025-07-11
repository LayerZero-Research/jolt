#![cfg_attr(feature = "guest", no_std)]

use jolt::keccak256;
use sha3::{Digest, Keccak256};

#[jolt::provable]
fn sha3_chain_default(input: [u8; 32], num_iters: u32) -> [u8; 32] {
    let mut hash = input;
    for _ in 0..num_iters {
        hash = Keccak256::digest(&hash).into();
    }
    hash
}
