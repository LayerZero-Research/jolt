#![cfg_attr(feature = "guest", no_std)]

use core::hint::black_box;

use jolt::{end_cycle_tracking, start_cycle_tracking};

use jolt_inlines_blake2 as blake2_inline;
use jolt_inlines_blake3 as blake3_inline;
use jolt_inlines_keccak256 as keccak_inline;
use jolt_inlines_sha2 as sha2_inline;

#[jolt::provable(
    max_output_size = 4096,
    memory_size = 33554432,
    stack_size = 10485760,
    max_trace_length = 20553600
)]
fn hashbench() -> [u8; 32] {
    // SHA-256
    benchmark_sha256_32();
    benchmark_sha256_64();
    benchmark_sha256_128();

    // Blake2b
    benchmark_blake2_64();
    benchmark_blake2_128();

    // Blake3
    benchmark_blake3_32();
    benchmark_blake3_64();

    // Keccak256
    benchmark_keccak_32();
    benchmark_keccak_64();

    return [0; 32];
}

fn assign_random_looking_values(array: &mut [u8], seed: u32) {
    const A: u32 = 1664525;
    const C: u32 = 1013904223;
    let mut state = seed;
    for item in array {
        state = state.wrapping_mul(A).wrapping_add(C);
        let value = (state ^ (state >> 16)) as u8;
        *item = value;
    }
}

// ==================== SHA-256 ====================

fn benchmark_sha256_32() {
    let mut input = [5u8; 32];
    assign_random_looking_values(&mut input, 10);

    start_cycle_tracking("sha2_32");
    let r1 = black_box(sha2_inline::Sha256::digest(black_box(&input[..])));
    end_cycle_tracking("sha2_32");

    let mut buffer = [5u8; 33];
    assign_random_looking_values(&mut buffer[1..], 10);
    let unaligned = &buffer[1..];

    start_cycle_tracking("sha2_32_un");
    let r2 = black_box(sha2_inline::Sha256::digest(black_box(unaligned)));
    end_cycle_tracking("sha2_32_un");

    assert_eq!(r1, r2, "SHA256 32B: aligned != unaligned");
}

fn benchmark_sha256_64() {
    let mut input = [5u8; 64];
    assign_random_looking_values(&mut input, 11);

    start_cycle_tracking("sha2_64");
    let r1 = black_box(sha2_inline::Sha256::digest(black_box(&input[..])));
    end_cycle_tracking("sha2_64");

    let mut buffer = [5u8; 65];
    assign_random_looking_values(&mut buffer[1..], 11);
    let unaligned = &buffer[1..];

    start_cycle_tracking("sha2_64_un");
    let r2 = black_box(sha2_inline::Sha256::digest(black_box(unaligned)));
    end_cycle_tracking("sha2_64_un");

    assert_eq!(r1, r2, "SHA256 64B: aligned != unaligned");
}

fn benchmark_sha256_128() {
    let mut input = [5u8; 128];
    assign_random_looking_values(&mut input, 12);

    start_cycle_tracking("sha2_128");
    let r1 = black_box(sha2_inline::Sha256::digest(black_box(&input[..])));
    end_cycle_tracking("sha2_128");

    let mut buffer = [5u8; 129];
    assign_random_looking_values(&mut buffer[1..], 12);
    let unaligned = &buffer[1..];

    start_cycle_tracking("sha2_128_un");
    let r2 = black_box(sha2_inline::Sha256::digest(black_box(unaligned)));
    end_cycle_tracking("sha2_128_un");

    assert_eq!(r1, r2, "SHA256 128B: aligned != unaligned");
}

// ==================== Blake2b ====================

fn benchmark_blake2_64() {
    let mut input = [5u8; 64];
    assign_random_looking_values(&mut input, 20);

    start_cycle_tracking("b2_64");
    let r1 = black_box(blake2_inline::Blake2b::digest(black_box(&input[..])));
    end_cycle_tracking("b2_64");

    let mut buffer = [5u8; 65];
    assign_random_looking_values(&mut buffer[1..], 20);
    let unaligned = &buffer[1..];

    start_cycle_tracking("b2_64_un");
    let r2 = black_box(blake2_inline::Blake2b::digest(black_box(unaligned)));
    end_cycle_tracking("b2_64_un");

    assert_eq!(r1, r2, "Blake2b 64B: aligned != unaligned");
}

fn benchmark_blake2_128() {
    let mut input = [5u8; 128];
    assign_random_looking_values(&mut input, 21);

    start_cycle_tracking("b2_128");
    let r1 = black_box(blake2_inline::Blake2b::digest(black_box(&input[..])));
    end_cycle_tracking("b2_128");

    let mut buffer = [5u8; 129];
    assign_random_looking_values(&mut buffer[1..], 21);
    let unaligned = &buffer[1..];

    start_cycle_tracking("b2_128_un");
    let r2 = black_box(blake2_inline::Blake2b::digest(black_box(unaligned)));
    end_cycle_tracking("b2_128_un");

    assert_eq!(r1, r2, "Blake2b 128B: aligned != unaligned");
}

// ==================== Blake3 ====================

fn benchmark_blake3_32() {
    let mut input = [5u8; 32];
    assign_random_looking_values(&mut input, 30);

    start_cycle_tracking("b3_32");
    let r1 = black_box(blake3_inline::Blake3::digest(black_box(&input[..])));
    end_cycle_tracking("b3_32");

    let mut buffer = [5u8; 33];
    assign_random_looking_values(&mut buffer[1..], 30);
    let unaligned = &buffer[1..];

    start_cycle_tracking("b3_32_un");
    let r2 = black_box(blake3_inline::Blake3::digest(black_box(unaligned)));
    end_cycle_tracking("b3_32_un");

    assert_eq!(r1, r2, "Blake3 32B: aligned != unaligned");
}

fn benchmark_blake3_64() {
    let mut input = [5u8; 64];
    assign_random_looking_values(&mut input, 31);

    start_cycle_tracking("b3_64");
    let r1 = black_box(blake3_inline::Blake3::digest(black_box(&input[..])));
    end_cycle_tracking("b3_64");

    let mut buffer = [5u8; 65];
    assign_random_looking_values(&mut buffer[1..], 31);
    let unaligned = &buffer[1..];

    start_cycle_tracking("b3_64_un");
    let r2 = black_box(blake3_inline::Blake3::digest(black_box(unaligned)));
    end_cycle_tracking("b3_64_un");

    assert_eq!(r1, r2, "Blake3 64B: aligned != unaligned");
}

// ==================== Keccak256 ====================

fn benchmark_keccak_32() {
    let mut input = [5u8; 32];
    assign_random_looking_values(&mut input, 40);

    start_cycle_tracking("kec_32");
    let r1 = black_box(keccak_inline::Keccak256::digest(black_box(&input[..])));
    end_cycle_tracking("kec_32");

    let mut buffer = [5u8; 33];
    assign_random_looking_values(&mut buffer[1..], 40);
    let unaligned = &buffer[1..];

    start_cycle_tracking("kec_32_un");
    let r2 = black_box(keccak_inline::Keccak256::digest(black_box(unaligned)));
    end_cycle_tracking("kec_32_un");

    assert_eq!(r1, r2, "Keccak256 32B: aligned != unaligned");
}

fn benchmark_keccak_64() {
    let mut input = [5u8; 64];
    assign_random_looking_values(&mut input, 41);

    start_cycle_tracking("kec_64");
    let r1 = black_box(keccak_inline::Keccak256::digest(black_box(&input[..])));
    end_cycle_tracking("kec_64");

    let mut buffer = [5u8; 65];
    assign_random_looking_values(&mut buffer[1..], 41);
    let unaligned = &buffer[1..];

    start_cycle_tracking("kec_64_un");
    let r2 = black_box(keccak_inline::Keccak256::digest(black_box(unaligned)));
    end_cycle_tracking("kec_64_un");

    assert_eq!(r1, r2, "Keccak256 64B: aligned != unaligned");
}
