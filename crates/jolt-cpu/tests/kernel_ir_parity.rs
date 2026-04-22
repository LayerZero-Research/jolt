//! Parity test: stack-VM kernel vs. KernelIR interpreter.
//!
//! For a hand-lowered IR matching a real production kernel
//! (`bytecode_read_raf` address-phase from `jolt-zkvm`), randomly sample
//! inputs and challenges and assert that:
//!
//!   `compile_with_challenges(expr, ...).evaluate(lo, hi, &mut out_a)`
//!   `evaluate_ir_pair(&ir, lo, hi, challenges, &mut out_b)`
//!   `out_a == out_b`
//!
//! Demonstrates that the proposed [`KernelIR`] is expressive enough to
//! represent a real kernel, and that the CPU interpreter produces byte-
//! identical output to the existing stack VM.
//!
//! Companion to `Notes/jolt-compute-backend-walk-the-walk-2026-04-21.md`
//! Appendix A.

use jolt_compute::ComputeBackend;
use jolt_cpu::{compile_with_challenges, evaluate_ir_pair, CpuBackend, CpuKernel};
use jolt_field::{Field, Fr};
use jolt_ir::{
    lower_custom_expr, BindingOrder, ConstVal, ExprBuilder, KernelDescriptor, KernelIR,
    KernelIteration, KernelOp, KernelShape,
};
use num_traits::Zero;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

/// Mirrors `N_STAGES` in `crates/jolt-zkvm/src/stages/s_bytecode_read_raf.rs`.
const N_STAGES: usize = 5;

/// Re-implements `build_address_descriptor` from
/// `crates/jolt-zkvm/src/stages/s_bytecode_read_raf.rs` (it's `fn`, not `pub`).
///
/// Formula: `Σ_s γ^s · F_s(k) · Val_s(k) + γ_entry · f_trace(k) · f_expected(k)`
///
/// Layout: `[F_0..F_4, Val_0..Val_4, f_trace, f_expected]` (12 inputs)
/// Challenges: `[γ^0..γ^4, γ_entry]` (6 entries)
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

/// Hand-lowered KernelIR for the address-phase kernel.
///
/// Register allocation:
/// ```text
///   r0..r11   : lo[0..12]   (loaded by 12 LoadPair ops)
///   r12..r23  : hi[0..12]   (loaded by 12 LoadPair ops, paired with lo)
///   r24..r29  : challenges (γ^0..γ^4, γ_entry)
///   r30..r41  : diffs[0..12] = hi - lo
///   r42       : Const(2)
///   r43..r54  : p[0..12] = lo + 2*diff = interpolated value at t=2
///   r55       : temp (γ_s · F_s)
///   r56       : acc (running sum across stage terms)
/// ```
///
/// 69 ops total: 12 LoadPair + 6 LoadChallenge + 13 (slot 0) + 12 Sub +
/// 1 Const + 12 Fma + 13 (slot 1).
fn lower_address_kernel() -> KernelIR {
    let mut ops = Vec::with_capacity(69);

    // Preamble: 12 LoadPair (lo[i] -> r_i, hi[i] -> r_{12+i}).
    for i in 0..12u16 {
        ops.push(KernelOp::LoadPair {
            buf: i,
            dst_lo: i,
            dst_hi: 12 + i,
        });
    }
    // Preamble: 6 LoadChallenge.
    for c in 0..6u8 {
        ops.push(KernelOp::LoadChallenge {
            idx: c,
            dst: 24 + u16::from(c),
        });
    }

    // Slot 0 (t=0): use lo values directly.
    //   acc = γ_0 · F_0 · Val_0 = mul(mul(γ_0, lo[0]), lo[5])
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
    //   For s=1..4: tmp = γ_s · F_s; acc = Fma(tmp, Val_s, acc)
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
    //   Entry: tmp = γ_e · f_trace; acc = Fma(tmp, f_expected, acc)
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
    //   StoreSlot(0, acc)
    ops.push(KernelOp::StoreSlot { slot: 0, src: 56 });

    // Slot 1 prep: compute 12 diffs.
    for i in 0..12u16 {
        ops.push(KernelOp::Sub {
            lhs: 12 + i,
            rhs: i,
            dst: 30 + i,
        });
    }
    // Materialize Const(2) into r42.
    ops.push(KernelOp::Const {
        value: ConstVal::I128(2),
        dst: 42,
    });
    // Compute 12 interpolated values: p_i = lo_i + 2 * diff_i = Fma(diff_i, 2, lo_i).
    for i in 0..12u16 {
        ops.push(KernelOp::Fma {
            a: 30 + i,
            b: 42,
            c: i,
            dst: 43 + i,
        });
    }

    // Slot 1 (t=2): same structure as slot 0 but with p[i] instead of lo[i].
    //   p[F_s] = r_{43+s},  p[Val_s] = r_{43 + N_STAGES + s} = r_{48+s}
    //   p[f_trace]  = r_{43+10} = r_53
    //   p[f_expected] = r_{43+11} = r_54
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

/// Sanity check: the lowered IR is internally consistent.
#[test]
fn address_kernel_ir_is_valid() {
    let ir = lower_address_kernel();
    assert!(ir.is_valid(), "lowered IR failed validation");
    assert_eq!(ir.ops.len(), 69, "op count mismatch");
}

/// Run the descriptor through both compilers on a single fixed input and
/// confirm byte-equal outputs.
#[test]
fn address_kernel_parity_fixed_input() {
    let desc = build_address_descriptor();
    let ir = lower_address_kernel();

    // Fixed challenges and inputs.
    let challenges: Vec<Fr> = (1..=6).map(Fr::from_u64).collect();
    let lo: Vec<Fr> = (1..=12).map(|i| Fr::from_u64(i * 7)).collect();
    let hi: Vec<Fr> = (1..=12).map(|i| Fr::from_u64(i * 13 + 1)).collect();

    let kernel: CpuKernel<Fr> = compile_with_challenges(&desc, &challenges);
    let mut out_stack = vec![Fr::zero(); 2];
    kernel.evaluate(&lo, &hi, &mut out_stack);

    let mut out_ir = vec![Fr::zero(); 2];
    evaluate_ir_pair(&ir, &lo, &hi, &challenges, &mut out_ir);

    assert_eq!(
        out_stack, out_ir,
        "stack VM and IR interpreter disagree on fixed input"
    );
}

/// Run 256 randomly sampled (lo, hi, challenges) tuples through both
/// compilers and confirm byte-equal outputs every time.
#[test]
fn address_kernel_parity_random() {
    let desc = build_address_descriptor();
    let ir = lower_address_kernel();

    for seed in 0..256u64 {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);

        let challenges: Vec<Fr> = (0..6).map(|_| Fr::random(&mut rng)).collect();
        let lo: Vec<Fr> = (0..12).map(|_| Fr::random(&mut rng)).collect();
        let hi: Vec<Fr> = (0..12).map(|_| Fr::random(&mut rng)).collect();

        let kernel: CpuKernel<Fr> = compile_with_challenges(&desc, &challenges);
        let mut out_stack = vec![Fr::zero(); 2];
        kernel.evaluate(&lo, &hi, &mut out_stack);

        let mut out_ir = vec![Fr::zero(); 2];
        evaluate_ir_pair(&ir, &lo, &hi, &challenges, &mut out_ir);

        assert_eq!(
            out_stack, out_ir,
            "mismatch at seed {seed}: stack={out_stack:?}, ir={out_ir:?}"
        );
    }
}

/// Test the simpler booleanity kernel `γ · (h² − h)` for cross-validation.
///
/// Hand-lowered IR is in `kernel_ir_interp.rs::tests::booleanity_ir`.
/// Re-derived here against the descriptor compiler.
#[test]
fn booleanity_parity_random() {
    let b = ExprBuilder::new();
    let h = b.opening(0);
    let gamma = b.challenge(0);
    let desc = KernelDescriptor {
        shape: KernelShape::Custom {
            expr: b.build(gamma * (h * h - h)),
            num_inputs: 1,
        },
        degree: 2,
        tensor_split: None,
    };

    let ir = booleanity_ir();
    assert!(ir.is_valid());

    for seed in 0..64u64 {
        let mut rng = ChaCha20Rng::seed_from_u64(seed * 17 + 3);
        let challenges = vec![Fr::random(&mut rng)];
        let lo = vec![Fr::random(&mut rng)];
        let hi = vec![Fr::random(&mut rng)];

        let kernel = compile_with_challenges::<Fr>(&desc, &challenges);
        let mut out_stack = vec![Fr::zero(); 2];
        kernel.evaluate(&lo, &hi, &mut out_stack);

        let mut out_ir = vec![Fr::zero(); 2];
        evaluate_ir_pair(&ir, &lo, &hi, &challenges, &mut out_ir);

        assert_eq!(out_stack, out_ir, "mismatch at seed {seed}");
    }
}

/// Inline copy of the IR for the booleanity expression.
fn booleanity_ir() -> KernelIR {
    KernelIR {
        num_inputs: 1,
        num_registers: 9,
        num_evals: 2,
        iteration: KernelIteration::PerPair {
            binding_order: BindingOrder::LowToHigh,
        },
        ops: vec![
            KernelOp::LoadPair {
                buf: 0,
                dst_lo: 0,
                dst_hi: 1,
            },
            KernelOp::LoadChallenge { idx: 0, dst: 2 },
            KernelOp::Sub {
                lhs: 1,
                rhs: 0,
                dst: 3,
            },
            KernelOp::Mul {
                lhs: 0,
                rhs: 0,
                dst: 5,
            },
            KernelOp::Sub {
                lhs: 5,
                rhs: 0,
                dst: 6,
            },
            KernelOp::Mul {
                lhs: 2,
                rhs: 6,
                dst: 7,
            },
            KernelOp::StoreSlot { slot: 0, src: 7 },
            KernelOp::Const {
                value: ConstVal::I128(2),
                dst: 8,
            },
            KernelOp::Fma {
                a: 3,
                b: 8,
                c: 0,
                dst: 4,
            },
            KernelOp::Mul {
                lhs: 4,
                rhs: 4,
                dst: 5,
            },
            KernelOp::Sub {
                lhs: 5,
                rhs: 4,
                dst: 6,
            },
            KernelOp::Mul {
                lhs: 2,
                rhs: 6,
                dst: 7,
            },
            KernelOp::StoreSlot { slot: 1, src: 7 },
        ],
    }
}

/// Edge case: kernel `γ_0 · h + γ_1 · 0` to exercise i128 zero const baking
/// in the multiplicative position.
///
/// `KernelDescriptor::Custom` requires `num_inputs > 0` (`crates/jolt-ir/src/kernel.rs:239`),
/// so we use one opening `h` with a coefficient that involves Const(0).
#[test]
fn const_zero_lifts_correctly() {
    let b = ExprBuilder::new();
    let zero = b.constant(0);
    let h = b.opening(0);
    let g0 = b.challenge(0);
    let g1 = b.challenge(1);
    let desc = KernelDescriptor {
        shape: KernelShape::Custom {
            expr: b.build(g0 * h + g1 * zero),
            num_inputs: 1,
        },
        degree: 1,
        tensor_split: None,
    };

    // degree=1 → num_evals=1 (slot 0 at t=0).
    let ir = KernelIR {
        num_inputs: 1,
        num_registers: 6,
        num_evals: 1,
        iteration: KernelIteration::PerPair {
            binding_order: BindingOrder::LowToHigh,
        },
        ops: vec![
            KernelOp::LoadPair {
                buf: 0,
                dst_lo: 0,
                dst_hi: 1,
            },
            KernelOp::LoadChallenge { idx: 0, dst: 2 },
            KernelOp::LoadChallenge { idx: 1, dst: 3 },
            KernelOp::Const {
                value: ConstVal::I128(0),
                dst: 4,
            },
            KernelOp::Mul {
                lhs: 3,
                rhs: 4,
                dst: 5,
            },
            KernelOp::Fma {
                a: 2,
                b: 0,
                c: 5,
                dst: 5,
            },
            KernelOp::StoreSlot { slot: 0, src: 5 },
        ],
    };

    let challenges = vec![Fr::from_u64(7), Fr::from_u64(99)];
    let lo = vec![Fr::from_u64(13)];
    let hi = vec![Fr::from_u64(27)];
    let kernel = compile_with_challenges::<Fr>(&desc, &challenges);
    let mut out_stack = vec![Fr::zero(); 1];
    kernel.evaluate(&lo, &hi, &mut out_stack);

    let mut out_ir = vec![Fr::zero(); 1];
    evaluate_ir_pair(&ir, &lo, &hi, &challenges, &mut out_ir);

    assert_eq!(out_stack, out_ir);
    // t=0: 7 · 13 + 99 · 0 = 91.
    assert_eq!(out_stack[0], Fr::from_u64(91));
}

/// Edge case: negative constant `-3 · h`.
#[test]
fn negative_const_lifts_correctly() {
    let b = ExprBuilder::new();
    let h = b.opening(0);
    let neg_three = b.constant(-3);
    let desc = KernelDescriptor {
        shape: KernelShape::Custom {
            expr: b.build(neg_three * h),
            num_inputs: 1,
        },
        degree: 1,
        tensor_split: None,
    };

    let ir = KernelIR {
        num_inputs: 1,
        num_registers: 4,
        num_evals: 1,
        iteration: KernelIteration::PerPair {
            binding_order: BindingOrder::LowToHigh,
        },
        ops: vec![
            KernelOp::LoadPair {
                buf: 0,
                dst_lo: 0,
                dst_hi: 1,
            },
            KernelOp::Const {
                value: ConstVal::I128(-3),
                dst: 2,
            },
            KernelOp::Mul {
                lhs: 2,
                rhs: 0,
                dst: 3,
            },
            KernelOp::StoreSlot { slot: 0, src: 3 },
        ],
    };

    for seed in 0..32u64 {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let lo = vec![Fr::random(&mut rng)];
        let hi = vec![Fr::random(&mut rng)];

        let kernel = compile_with_challenges::<Fr>(&desc, &[]);
        let mut out_stack = vec![Fr::zero(); 1];
        kernel.evaluate(&lo, &hi, &mut out_stack);

        let mut out_ir = vec![Fr::zero(); 1];
        evaluate_ir_pair(&ir, &lo, &hi, &[], &mut out_ir);

        assert_eq!(out_stack, out_ir, "mismatch at seed {seed}");
    }
}

/// End-to-end lowering test: drive the address-phase kernel through
/// `lower_custom_expr` (rather than the hand-built IR) and confirm parity
/// with the stack VM.
///
/// This is the critical proof that `Expr -> KernelIR` lowering is correct
/// for a real production kernel. Together with the
/// `address_kernel_parity_random` test (stack VM vs hand-lowered IR), it
/// establishes the full chain: `Expr` -> stack VM == `Expr` -> auto-lowered IR.
#[test]
fn address_kernel_lowered_parity_random() {
    let desc = build_address_descriptor();
    let KernelShape::Custom { expr, num_inputs } = &desc.shape else {
        panic!("expected Custom shape");
    };
    let ir = lower_custom_expr(expr, *num_inputs, desc.degree, BindingOrder::LowToHigh);
    assert!(ir.is_valid(), "lowered IR failed validation");

    for seed in 0..256u64 {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);

        let challenges: Vec<Fr> = (0..6).map(|_| Fr::random(&mut rng)).collect();
        let lo: Vec<Fr> = (0..12).map(|_| Fr::random(&mut rng)).collect();
        let hi: Vec<Fr> = (0..12).map(|_| Fr::random(&mut rng)).collect();

        let kernel: CpuKernel<Fr> = compile_with_challenges(&desc, &challenges);
        let mut out_stack = vec![Fr::zero(); 2];
        kernel.evaluate(&lo, &hi, &mut out_stack);

        let mut out_ir = vec![Fr::zero(); 2];
        evaluate_ir_pair(&ir, &lo, &hi, &challenges, &mut out_ir);

        assert_eq!(
            out_stack, out_ir,
            "mismatch at seed {seed}: stack={out_stack:?}, ir={out_ir:?}"
        );
    }
}

/// Same as above but for the booleanity kernel.
#[test]
fn booleanity_lowered_parity_random() {
    let b = ExprBuilder::new();
    let h = b.opening(0);
    let gamma = b.challenge(0);
    let expr = b.build(gamma * (h * h - h));
    let desc = KernelDescriptor {
        shape: KernelShape::Custom {
            expr: expr.clone(),
            num_inputs: 1,
        },
        degree: 2,
        tensor_split: None,
    };

    let ir = lower_custom_expr(&expr, 1, desc.degree, BindingOrder::LowToHigh);
    assert!(ir.is_valid());

    for seed in 0..64u64 {
        let mut rng = ChaCha20Rng::seed_from_u64(seed * 17 + 3);
        let challenges = vec![Fr::random(&mut rng)];
        let lo = vec![Fr::random(&mut rng)];
        let hi = vec![Fr::random(&mut rng)];

        let kernel = compile_with_challenges::<Fr>(&desc, &challenges);
        let mut out_stack = vec![Fr::zero(); 2];
        kernel.evaluate(&lo, &hi, &mut out_stack);

        let mut out_ir = vec![Fr::zero(); 2];
        evaluate_ir_pair(&ir, &lo, &hi, &challenges, &mut out_ir);

        assert_eq!(out_stack, out_ir, "mismatch at seed {seed}");
    }
}

/// `CpuBackend::compile_kernel_ir` end-to-end: lower an Expr, hand the IR to
/// the backend trait, and confirm the resulting `CpuKernel` matches the
/// stack-VM compiler on random inputs.
///
/// This validates the full ComputeBackend trait integration —
/// `lower_custom_expr` + `compile_kernel_ir` should be bit-identical to
/// `compile_kernel_with_challenges` on the same descriptor.
#[test]
fn compute_backend_compile_kernel_ir_parity() {
    let desc = build_address_descriptor();
    let KernelShape::Custom { expr, num_inputs } = &desc.shape else {
        panic!("expected Custom shape");
    };
    let ir = lower_custom_expr(expr, *num_inputs, desc.degree, BindingOrder::LowToHigh);

    let backend = CpuBackend;

    for seed in 0..64u64 {
        let mut rng = ChaCha20Rng::seed_from_u64(seed * 31 + 7);

        let challenges: Vec<Fr> = (0..6).map(|_| Fr::random(&mut rng)).collect();
        let lo: Vec<Fr> = (0..12).map(|_| Fr::random(&mut rng)).collect();
        let hi: Vec<Fr> = (0..12).map(|_| Fr::random(&mut rng)).collect();

        let stack_kernel: CpuKernel<Fr> =
            backend.compile_kernel_with_challenges(&desc, &challenges);
        let ir_kernel: CpuKernel<Fr> = backend.compile_kernel_ir(&ir, &challenges);

        let mut out_stack = vec![Fr::zero(); 2];
        stack_kernel.evaluate(&lo, &hi, &mut out_stack);

        let mut out_ir = vec![Fr::zero(); 2];
        ir_kernel.evaluate(&lo, &hi, &mut out_ir);

        assert_eq!(out_stack, out_ir, "mismatch at seed {seed}");
    }
}
