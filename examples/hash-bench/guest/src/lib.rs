#![cfg_attr(feature = "guest", no_std)]

use core::hint::black_box;

use jolt::{end_cycle_tracking, start_cycle_tracking};
use sha2::{self as sha2_reference, Digest};
use jolt_inlines_sha2 as sha2_inline;

use sha3 as keccak_reference;
use jolt_inlines_keccak256 as keccak_inline;

use blake2 as blake2_reference;
use jolt_inlines_blake2 as blake2_inline;
use blake3 as blake3_reference;
use jolt_inlines_blake3 as blake3_inline;


#[jolt::provable(max_input_size= 4096, max_output_size= 4096, memory_size= 33554432, stack_size=10485760, max_trace_length = 655360)]
fn hashbench(_input: &[u8]) -> [u8; 32] {

    // let sha2_input = [5u8; 2048];
    // start_cycle_tracking("sha2_reference");
    // let hash = black_box(sha2_reference::Sha256::digest(black_box(&sha2_input)));
    // black_box(hash);
    // end_cycle_tracking("sha2_reference");

    // start_cycle_tracking("sha2_inline");
    // let hash = black_box(sha2_inline::Sha256::digest(black_box(&sha2_input)));
    // black_box(hash);
    // end_cycle_tracking("sha2_inline");

    // let keccak_input = [5u8; 1024];
    // start_cycle_tracking("keccak_reference");
    // let hash_k = black_box(keccak_reference::Keccak256::digest(black_box(&keccak_input)));
    // black_box(hash_k);
    // end_cycle_tracking("keccak_reference");

    // start_cycle_tracking("keccak_inline");
    // let hash_k = black_box(keccak_inline::Keccak256::digest(black_box(&keccak_input)));
    // black_box(hash_k);
    // end_cycle_tracking("keccak_inline");


    // let blake2_input = [5u8; 1024];
    // start_cycle_tracking("blake2_reference");
    // let hash_b = black_box(blake2_reference::Blake2b512::digest(black_box(&blake2_input)));
    // black_box(hash_b);
    // end_cycle_tracking("blake2_reference");

    // start_cycle_tracking("blake2_inline");
    // let hash_b = black_box(blake2_inline::Blake2b::digest(black_box(&blake2_input)));
    // black_box(hash_b);
    // end_cycle_tracking("blake2_inline");

    let blake3_input = [5u8; 64];

    start_cycle_tracking("blake3_reference");
    let hash_b3 = black_box(blake3_reference::hash(black_box(&blake3_input)));
    black_box(hash_b3);
    end_cycle_tracking("blake3_reference");

    start_cycle_tracking("blake3_inline");
    let hash_b3 = black_box(blake3_inline::Blake3::digest(black_box(&blake3_input)));
    black_box(hash_b3);
    end_cycle_tracking("blake3_inline");

    return [0; 32];
}
