//! Microbenchmark: stack VM vs. KernelIR interpreter.
//!
//! Direct head-to-head on the `bytecode_read_raf` address-phase kernel
//! (12 inputs, 6 challenges, degree 2). Mirrors the per-pair call shape
//! used inside `CpuBackend::pairwise_reduce` — the only thing varied
//! between groups is the kernel evaluator itself.
//!
//! Companion to `Notes/jolt-compute-backend-walk-the-walk-2026-04-21.md`
//! Appendix B.8.
//!
//! Run with:
//!
//! ```bash
//! cargo bench -p jolt-cpu --bench kernel_ir_vs_stack_vm
//! ```
//!
//! For asm/object inspection, the wrappers `address_kernel_ir_loop` and
//! `address_kernel_stack_vm_loop` are `#[inline(never)]` so they show up
//! as standalone symbols.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    unused_must_use,
    unused_results
)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use jolt_cpu::{compile_with_challenges, evaluate_ir_pair, CpuKernel};
use jolt_field::{Field, Fr};
use jolt_ir::{
    BindingOrder, ConstVal, ExprBuilder, KernelDescriptor, KernelIR, KernelIteration, KernelOp,
    KernelShape,
};
use num_traits::Zero;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

const N_STAGES: usize = 5;

/// Mirrors `tests/kernel_ir_parity.rs::build_address_descriptor`.
fn build_address_descriptor() -> KernelDescriptor {
    let num_inputs = 2 * N_STAGES + 2;
    let b = ExprBuilder::new();

    let mut sum = b.challenge(0) * b.opening(0) * b.opening(N_STAGES as u32);
    for s in 1..N_STAGES {
        sum = sum + b.challenge(s as u32) * b.opening(s as u32) * b.opening((N_STAGES + s) as u32);
    }
    let trace_idx = (2 * N_STAGES) as u32;
    let expected_idx = (2 * N_STAGES + 1) as u32;
    sum = sum + b.challenge(N_STAGES as u32) * b.opening(trace_idx) * b.opening(expected_idx);

    KernelDescriptor {
        shape: KernelShape::Custom {
            expr: b.build(sum),
            num_inputs,
        },
        degree: 2,
        tensor_split: None,
    }
}

/// Mirrors `tests/kernel_ir_parity.rs::lower_address_kernel`. 69 ops.
fn lower_address_kernel() -> KernelIR {
    let mut ops = Vec::with_capacity(69);

    for i in 0..12u16 {
        ops.push(KernelOp::LoadPair {
            buf: i,
            dst_lo: i,
            dst_hi: 12 + i,
        });
    }
    for c in 0..6u8 {
        ops.push(KernelOp::LoadChallenge {
            idx: c,
            dst: 24 + u16::from(c),
        });
    }

    // Slot 0 (t=0).
    ops.push(KernelOp::Mul {
        lhs: 24,
        rhs: 0,
        dst: 55,
    });
    ops.push(KernelOp::Mul {
        lhs: 55,
        rhs: 5,
        dst: 56,
    });
    for s in 1..N_STAGES as u16 {
        ops.push(KernelOp::Mul {
            lhs: 24 + s,
            rhs: s,
            dst: 55,
        });
        ops.push(KernelOp::Fma {
            a: 55,
            b: N_STAGES as u16 + s,
            c: 56,
            dst: 56,
        });
    }
    ops.push(KernelOp::Mul {
        lhs: 29,
        rhs: 10,
        dst: 55,
    });
    ops.push(KernelOp::Fma {
        a: 55,
        b: 11,
        c: 56,
        dst: 56,
    });
    ops.push(KernelOp::StoreSlot { slot: 0, src: 56 });

    // Slot 1 prep.
    for i in 0..12u16 {
        ops.push(KernelOp::Sub {
            lhs: 12 + i,
            rhs: i,
            dst: 30 + i,
        });
    }
    ops.push(KernelOp::Const {
        value: ConstVal::I128(2),
        dst: 42,
    });
    for i in 0..12u16 {
        ops.push(KernelOp::Fma {
            a: 30 + i,
            b: 42,
            c: i,
            dst: 43 + i,
        });
    }

    // Slot 1 (t=2).
    ops.push(KernelOp::Mul {
        lhs: 24,
        rhs: 43,
        dst: 55,
    });
    ops.push(KernelOp::Mul {
        lhs: 55,
        rhs: 48,
        dst: 56,
    });
    for s in 1..N_STAGES as u16 {
        ops.push(KernelOp::Mul {
            lhs: 24 + s,
            rhs: 43 + s,
            dst: 55,
        });
        ops.push(KernelOp::Fma {
            a: 55,
            b: 43 + N_STAGES as u16 + s,
            c: 56,
            dst: 56,
        });
    }
    ops.push(KernelOp::Mul {
        lhs: 29,
        rhs: 53,
        dst: 55,
    });
    ops.push(KernelOp::Fma {
        a: 55,
        b: 54,
        c: 56,
        dst: 56,
    });
    ops.push(KernelOp::StoreSlot { slot: 1, src: 56 });

    KernelIR {
        num_inputs: 12,
        num_registers: 57,
        num_evals: 2,
        iteration: KernelIteration::PerPair {
            binding_order: BindingOrder::LowToHigh,
        },
        ops,
    }
}

fn random_pairs(n_pairs: usize, num_inputs: usize, seed: u64) -> (Vec<Vec<Fr>>, Vec<Vec<Fr>>) {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let mut los = Vec::with_capacity(n_pairs);
    let mut his = Vec::with_capacity(n_pairs);
    for _ in 0..n_pairs {
        let lo: Vec<Fr> = (0..num_inputs).map(|_| Fr::random(&mut rng)).collect();
        let hi: Vec<Fr> = (0..num_inputs).map(|_| Fr::random(&mut rng)).collect();
        los.push(lo);
        his.push(hi);
    }
    (los, his)
}

/// Drive the IR interpreter over `n_pairs` random (lo, hi) pairs in tight loop.
/// Marked `#[inline(never)]` so cargo-asm can isolate the inner loop.
#[inline(never)]
fn address_kernel_ir_loop(
    ir: &KernelIR,
    los: &[Vec<Fr>],
    his: &[Vec<Fr>],
    challenges: &[Fr],
    out: &mut [Fr; 2],
) -> [Fr; 2] {
    for (lo, hi) in los.iter().zip(his.iter()) {
        evaluate_ir_pair(ir, lo, hi, challenges, out);
        // Force the result through a black_box-like dependency so LLVM cannot
        // hoist the entire call out of the loop.
        out[0] += out[1];
    }
    *out
}

/// Drive the stack VM over `n_pairs` random (lo, hi) pairs in tight loop.
/// Marked `#[inline(never)]` so cargo-asm can isolate the inner loop.
#[inline(never)]
fn address_kernel_stack_vm_loop(
    kernel: &CpuKernel<Fr>,
    los: &[Vec<Fr>],
    his: &[Vec<Fr>],
    out: &mut [Fr; 2],
) -> [Fr; 2] {
    for (lo, hi) in los.iter().zip(his.iter()) {
        kernel.evaluate(lo, hi, out);
        out[0] += out[1];
    }
    *out
}

fn bench_address_kernel(c: &mut Criterion) {
    let desc = build_address_descriptor();
    let ir = lower_address_kernel();
    assert!(ir.is_valid(), "lowered IR must be valid");

    // Use one fixed challenge vector. The stack VM bakes these at compile
    // time; the IR interpreter loads them per-pair as `LoadChallenge`. The
    // overhead of those 6 loads is part of what the IR pays vs. the stack VM.
    let mut rng = ChaCha20Rng::seed_from_u64(0xDEAD_BEEF);
    let challenges: Vec<Fr> = (0..6).map(|_| Fr::random(&mut rng)).collect();
    let kernel: CpuKernel<Fr> = compile_with_challenges(&desc, &challenges);

    let mut group = c.benchmark_group("address_kernel_per_pair");

    for n_pairs in [1024usize, 4096, 16384] {
        let (los, his) = random_pairs(n_pairs, 12, n_pairs as u64);

        // Throughput = pairs processed per call = n_pairs.
        group.throughput(Throughput::Elements(n_pairs as u64));

        group.bench_with_input(BenchmarkId::new("stack_vm", n_pairs), &n_pairs, |b, _| {
            let mut out = [Fr::zero(); 2];
            b.iter(|| {
                let r = address_kernel_stack_vm_loop(
                    black_box(&kernel),
                    black_box(&los),
                    black_box(&his),
                    &mut out,
                );
                black_box(r);
            });
        });

        group.bench_with_input(BenchmarkId::new("kernel_ir", n_pairs), &n_pairs, |b, _| {
            let mut out = [Fr::zero(); 2];
            b.iter(|| {
                let r = address_kernel_ir_loop(
                    black_box(&ir),
                    black_box(&los),
                    black_box(&his),
                    black_box(&challenges),
                    &mut out,
                );
                black_box(r);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_address_kernel);
criterion_main!(benches);
