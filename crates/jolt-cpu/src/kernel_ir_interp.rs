//! CPU interpreter for [`KernelIR`].
//!
//! Path A from `Notes/jolt-compute-backend-walk-the-walk-2026-04-21.md` §A.6:
//! replace the per-call `Vec<F>` stack with a stack-allocated register file,
//! and replace `Vec<CompiledOp<F>>` with `&[KernelOp]`.
//!
//! # Per-pair cost vs. the current stack VM
//! (`crates/jolt-cpu/src/custom.rs::kernel_from_ops`)
//!
//! - **Heap allocations:** 0 (was 1 `Vec::with_capacity` per pair).
//! - **Op dispatches:** `ir.ops.len()` (was `degree * expr.len()` because the
//!   stack VM re-evaluates the entire expression once per output slot).
//! - **Indirect calls:** 0 — [`evaluate_ir_pair`] is a concrete function, not
//!   a `Box<dyn Fn>`.
//! - **Materialized field constants:** baked at IR-construction time via
//!   [`KernelOp::Const`]; the `i128` -> `F` lift happens at op dispatch time
//!   (matches the stack VM, can be hoisted to compile time later).

use jolt_field::Field;
use jolt_ir::{ConstVal, KernelIR, KernelIteration, KernelOp};

/// Maximum number of registers per pair. Sized to fit any kernel currently
/// in use without heap allocation, while keeping the stack frame bounded
/// (~8KB for BN254 Fr).
pub const MAX_REGS: usize = 256;

/// Evaluate a [`KernelIR`] body for one pair `(lo, hi)`.
///
/// Contract (must match `crates/jolt-cpu/src/custom.rs::kernel_from_ops`):
/// - `lo` and `hi` are paired views over the kernel's input buffers; each
///   has length >= `ir.num_inputs`.
/// - `challenges` provides the values for [`KernelOp::LoadChallenge`].
/// - `out` receives `ir.num_evals` output values, indexed by
///   [`KernelOp::StoreSlot`] `slot`.
///
/// # Panics (debug builds only)
///
/// - `ir.iteration` is not `PerPair`.
/// - Buffer / register / slot indices are out of range.
/// - `ir.num_registers > MAX_REGS`.
///
/// In release builds these conditions trigger UB-adjacent behavior (out-of-
/// bounds reads/writes) — the IR is expected to have passed `is_valid()` at
/// build time.
#[inline]
pub fn evaluate_ir_pair<F: Field>(
    ir: &KernelIR,
    lo: &[F],
    hi: &[F],
    challenges: &[F],
    out: &mut [F],
) {
    debug_assert!(
        matches!(ir.iteration, KernelIteration::PerPair { .. }),
        "evaluate_ir_pair requires PerPair iteration"
    );
    debug_assert!(ir.num_inputs as usize <= lo.len());
    debug_assert!(ir.num_inputs as usize <= hi.len());
    debug_assert!(ir.num_evals as usize <= out.len());
    debug_assert!(
        (ir.num_registers as usize) <= MAX_REGS,
        "kernel needs {} registers, MAX_REGS={MAX_REGS}",
        ir.num_registers
    );

    let mut regs: [F; MAX_REGS] = [F::zero(); MAX_REGS];

    for op in &ir.ops {
        match *op {
            KernelOp::LoadPair {
                buf,
                dst_lo,
                dst_hi,
            } => {
                regs[dst_lo as usize] = lo[buf as usize];
                regs[dst_hi as usize] = hi[buf as usize];
            }
            KernelOp::LoadOne { .. } => {
                debug_assert!(false, "LoadOne is invalid in PerPair iteration");
            }
            KernelOp::LoadChallenge { idx, dst } => {
                regs[dst as usize] = challenges[idx as usize];
            }
            KernelOp::Const { value, dst } => {
                let ConstVal::I128(v) = value;
                regs[dst as usize] = F::from_i128(v);
            }
            KernelOp::Add { lhs, rhs, dst } => {
                regs[dst as usize] = regs[lhs as usize] + regs[rhs as usize];
            }
            KernelOp::Sub { lhs, rhs, dst } => {
                regs[dst as usize] = regs[lhs as usize] - regs[rhs as usize];
            }
            KernelOp::Mul { lhs, rhs, dst } => {
                regs[dst as usize] = regs[lhs as usize] * regs[rhs as usize];
            }
            KernelOp::Fma { a, b, c, dst } => {
                regs[dst as usize] = regs[a as usize] * regs[b as usize] + regs[c as usize];
            }
            KernelOp::StoreSlot { slot, src } => {
                out[slot as usize] = regs[src as usize];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jolt_field::Fr;
    use jolt_ir::{BindingOrder, KernelIR, KernelIteration, KernelOp};
    use num_traits::Zero;

    /// Hand-lowered booleanity check: `γ · (h² − h)` evaluated at `t ∈ {0, 2}`.
    /// Matches the doctest example in `kernel_ir.rs`.
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

    #[test]
    fn booleanity_matches_handcomputed() {
        let ir = booleanity_ir();
        let lo = [Fr::from_u64(3)];
        let hi = [Fr::from_u64(7)];
        let challenges = [Fr::from_u64(11)];
        let mut out = [Fr::zero(); 2];

        evaluate_ir_pair(&ir, &lo, &hi, &challenges, &mut out);

        // t=0: h=3, γ·(9-3) = 11·6 = 66
        assert_eq!(out[0], Fr::from_u64(66));
        // t=2: h=3+2·4=11, γ·(121-11) = 11·110 = 1210
        assert_eq!(out[1], Fr::from_u64(1210));
    }

    #[test]
    fn empty_inputs_no_panic() {
        // Smallest possible IR: just stores a constant.
        let ir = KernelIR {
            num_inputs: 0,
            num_registers: 1,
            num_evals: 1,
            iteration: KernelIteration::PerPair {
                binding_order: BindingOrder::LowToHigh,
            },
            ops: vec![
                KernelOp::Const {
                    value: ConstVal::I128(42),
                    dst: 0,
                },
                KernelOp::StoreSlot { slot: 0, src: 0 },
            ],
        };
        let mut out = [Fr::zero(); 1];
        evaluate_ir_pair::<Fr>(&ir, &[], &[], &[], &mut out);
        assert_eq!(out[0], Fr::from_u64(42));
    }
}
