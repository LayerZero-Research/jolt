//! Lowering from [`Expr`] to [`KernelIR`].
//!
//! Companion to `Notes/jolt-compute-backend-walk-the-walk-2026-04-21.md` §B.7
//! Step 4. Walks the symbolic expression DAG once and emits a flat
//! register-machine body that backends consume directly.
//!
//! # Output contract
//!
//! The lowered IR matches the stack VM's evaluation contract
//! (`jolt-cpu/src/custom.rs::kernel_from_ops`):
//!
//! - Grid `{0, 2, 3, ..., degree}` (skipping `t = 1`); slot 0 maps to `t = 0`,
//!   slot `k ≥ 1` maps to `t = k + 1`.
//! - At slot `t`, each opening `Var::Opening(i)` evaluates to
//!   `lo[i] + t · (hi[i] - lo[i])`; constants and challenges are
//!   slot-independent.
//! - At slot 0 (`t = 0`), opening values reduce to `lo[i]` directly (no
//!   diff-multiplication).
//!
//! # Strategy
//!
//! 1. **Pass 1** (scan): walk the arena once to collect the unique set of
//!    opening / challenge / constant references. This bounds the preamble
//!    register footprint and avoids redundant `LoadPair` / `LoadChallenge`
//!    / `Const` ops.
//! 2. **Preamble** (slot-independent): emit one `LoadPair` per opening, one
//!    `LoadChallenge` per challenge, one `Const` per constant value. If the
//!    grid contains `t ≥ 2` (i.e. `degree ≥ 2`), additionally hoist
//!    `diff[i] = hi[i] - lo[i]` via `Sub` so that per-slot interpolation is a
//!    single `Fma`.
//! 3. **Per-slot body**: for each output slot, materialize the interpolated
//!    opening values (`Fma(diff, t, lo)` for `t ≥ 2`, or just `lo` for
//!    `t = 0`), then walk the DAG in post-order, allocating one fresh
//!    register per non-leaf node and memoizing by `ExprId` so shared
//!    subtrees emit ops only once. The root register is written via
//!    `StoreSlot { slot, src: root_reg }`.
//!
//! `Neg(x)` lowers to `Sub { lhs: const_zero, rhs: x }` since [`KernelOp`]
//! has no negation primitive. A `Const(0)` register is preallocated whenever
//! the expression contains any `Neg`.

use std::collections::{BTreeMap, BTreeSet};

use crate::expr::{Expr, ExprId, ExprNode, Var};
use crate::kernel_ir::{BindingOrder, BufId, ConstVal, KernelIR, KernelIteration, KernelOp, RegId};

/// Lower a custom-shape [`Expr`] into a flat [`KernelIR`].
///
/// `num_inputs` is the number of input buffers the expression reads (must
/// match the parent [`KernelDescriptor::num_inputs`](crate::KernelDescriptor)).
/// `degree` controls the output slot count: `num_evals = degree`, with the
/// grid `{0, 2, ..., degree}`.
///
/// # Panics
///
/// - if any `Var::Opening(i)` has `i >= num_inputs` or `i >= u16::MAX`
/// - if any `Var::Challenge(c)` has `c >= u8::MAX` (the IR encodes the
///   challenge index in `u8`; widen if a real kernel hits this).
/// - if `degree == 0`.
pub fn lower_custom_expr(
    expr: &Expr,
    num_inputs: usize,
    degree: usize,
    binding_order: BindingOrder,
) -> KernelIR {
    assert!(degree > 0, "degree must be positive");
    assert!(num_inputs <= u16::MAX as usize, "num_inputs out of range");

    // Pass 1: scan the arena for unique opening / challenge / constant uses.
    let arena = expr.arena();
    let n_nodes = arena.len();

    let mut openings: BTreeSet<u32> = BTreeSet::new();
    let mut challenges: BTreeSet<u32> = BTreeSet::new();
    let mut constants: BTreeSet<i128> = BTreeSet::new();
    let mut needs_neg = false;

    for i in 0..n_nodes {
        match arena.get(ExprId(i as u32)) {
            ExprNode::Constant(v) => {
                let _newly_inserted = constants.insert(v);
            }
            ExprNode::Var(Var::Opening(o)) => {
                assert!((o as usize) < num_inputs, "opening index out of range");
                let _newly_inserted = openings.insert(o);
            }
            ExprNode::Var(Var::Challenge(c)) => {
                assert!(c < u8::MAX as u32, "challenge index out of u8 range");
                let _newly_inserted = challenges.insert(c);
            }
            ExprNode::Neg(_) => {
                needs_neg = true;
            }
            _ => {}
        }
    }

    let needs_diffs = degree >= 2;
    let grid_t_values: Vec<u64> = (1..degree).map(|k| (k + 1) as u64).collect();
    for &t in &grid_t_values {
        let _newly_inserted = constants.insert(t as i128);
    }
    if needs_neg {
        let _newly_inserted = constants.insert(0);
    }

    let mut alloc = RegAllocator::default();

    let lo_reg: BTreeMap<u32, RegId> = openings.iter().map(|&o| (o, alloc.next())).collect();
    let hi_reg: BTreeMap<u32, RegId> = openings.iter().map(|&o| (o, alloc.next())).collect();
    let chal_reg: BTreeMap<u32, RegId> = challenges.iter().map(|&c| (c, alloc.next())).collect();
    let const_reg: BTreeMap<i128, RegId> = constants.iter().map(|&v| (v, alloc.next())).collect();
    let diff_reg: BTreeMap<u32, RegId> = if needs_diffs {
        openings.iter().map(|&o| (o, alloc.next())).collect()
    } else {
        BTreeMap::new()
    };

    let mut ops: Vec<KernelOp> = Vec::with_capacity(8 + n_nodes * 2);

    for (&o, &lo_r) in &lo_reg {
        let hi_r = hi_reg[&o];
        ops.push(KernelOp::LoadPair {
            buf: o as BufId,
            dst_lo: lo_r,
            dst_hi: hi_r,
        });
    }
    for (&c, &r) in &chal_reg {
        ops.push(KernelOp::LoadChallenge {
            idx: c as u8,
            dst: r,
        });
    }
    for (&v, &r) in &const_reg {
        ops.push(KernelOp::Const {
            value: ConstVal::I128(v),
            dst: r,
        });
    }
    if needs_diffs {
        for (&o, &diff_r) in &diff_reg {
            ops.push(KernelOp::Sub {
                lhs: hi_reg[&o],
                rhs: lo_reg[&o],
                dst: diff_r,
            });
        }
    }

    for slot in 0..degree {
        let t = if slot == 0 { 0 } else { slot + 1 };

        let opening_value: BTreeMap<u32, RegId> = if t == 0 {
            lo_reg.clone()
        } else {
            let t_reg = const_reg[&(t as i128)];
            openings
                .iter()
                .map(|&o| {
                    let p_reg = alloc.next();
                    ops.push(KernelOp::Fma {
                        a: diff_reg[&o],
                        b: t_reg,
                        c: lo_reg[&o],
                        dst: p_reg,
                    });
                    (o, p_reg)
                })
                .collect()
        };

        let mut node_reg: Vec<Option<RegId>> = vec![None; n_nodes];
        let root = lower_node(
            expr,
            expr.root(),
            &opening_value,
            &chal_reg,
            &const_reg,
            &mut node_reg,
            &mut alloc,
            &mut ops,
        );

        ops.push(KernelOp::StoreSlot {
            slot: slot as u8,
            src: root,
        });
    }

    KernelIR {
        num_inputs: num_inputs as u16,
        num_registers: alloc.count(),
        num_evals: degree as u8,
        iteration: KernelIteration::PerPair { binding_order },
        ops,
    }
}

#[expect(clippy::too_many_arguments, reason = "internal recursive helper")]
fn lower_node(
    expr: &Expr,
    id: ExprId,
    opening_value: &BTreeMap<u32, RegId>,
    chal_reg: &BTreeMap<u32, RegId>,
    const_reg: &BTreeMap<i128, RegId>,
    node_reg: &mut [Option<RegId>],
    alloc: &mut RegAllocator,
    ops: &mut Vec<KernelOp>,
) -> RegId {
    if let Some(reg) = node_reg[id.index()] {
        return reg;
    }

    let reg = match expr.get(id) {
        ExprNode::Constant(v) => const_reg[&v],
        ExprNode::Var(Var::Opening(o)) => opening_value[&o],
        ExprNode::Var(Var::Challenge(c)) => chal_reg[&c],
        ExprNode::Neg(inner) => {
            let inner_reg = lower_node(
                expr,
                inner,
                opening_value,
                chal_reg,
                const_reg,
                node_reg,
                alloc,
                ops,
            );
            let zero_reg = const_reg[&0];
            let dst = alloc.next();
            ops.push(KernelOp::Sub {
                lhs: zero_reg,
                rhs: inner_reg,
                dst,
            });
            dst
        }
        ExprNode::Add(lhs, rhs) => emit_binop(
            expr,
            lhs,
            rhs,
            opening_value,
            chal_reg,
            const_reg,
            node_reg,
            alloc,
            ops,
            |a, b, dst| KernelOp::Add {
                lhs: a,
                rhs: b,
                dst,
            },
        ),
        ExprNode::Sub(lhs, rhs) => emit_binop(
            expr,
            lhs,
            rhs,
            opening_value,
            chal_reg,
            const_reg,
            node_reg,
            alloc,
            ops,
            |a, b, dst| KernelOp::Sub {
                lhs: a,
                rhs: b,
                dst,
            },
        ),
        ExprNode::Mul(lhs, rhs) => emit_binop(
            expr,
            lhs,
            rhs,
            opening_value,
            chal_reg,
            const_reg,
            node_reg,
            alloc,
            ops,
            |a, b, dst| KernelOp::Mul {
                lhs: a,
                rhs: b,
                dst,
            },
        ),
    };

    node_reg[id.index()] = Some(reg);
    reg
}

#[expect(clippy::too_many_arguments, reason = "internal helper for binary ops")]
#[inline]
fn emit_binop(
    expr: &Expr,
    lhs: ExprId,
    rhs: ExprId,
    opening_value: &BTreeMap<u32, RegId>,
    chal_reg: &BTreeMap<u32, RegId>,
    const_reg: &BTreeMap<i128, RegId>,
    node_reg: &mut [Option<RegId>],
    alloc: &mut RegAllocator,
    ops: &mut Vec<KernelOp>,
    make_op: impl FnOnce(RegId, RegId, RegId) -> KernelOp,
) -> RegId {
    let lhs_reg = lower_node(
        expr,
        lhs,
        opening_value,
        chal_reg,
        const_reg,
        node_reg,
        alloc,
        ops,
    );
    let rhs_reg = lower_node(
        expr,
        rhs,
        opening_value,
        chal_reg,
        const_reg,
        node_reg,
        alloc,
        ops,
    );
    let dst = alloc.next();
    ops.push(make_op(lhs_reg, rhs_reg, dst));
    dst
}

#[derive(Default)]
struct RegAllocator {
    next: u16,
}

impl RegAllocator {
    fn next(&mut self) -> RegId {
        let r = self.next;
        self.next = self.next.checked_add(1).expect("register count overflow");
        r
    }
    fn count(&self) -> u16 {
        self.next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::ExprBuilder;

    #[test]
    fn booleanity_lowering_validates() {
        let b = ExprBuilder::new();
        let h = b.opening(0);
        let gamma = b.challenge(0);
        let expr = b.build(gamma * (h * h - h));

        let ir = lower_custom_expr(&expr, 1, 2, BindingOrder::LowToHigh);
        assert!(ir.is_valid(), "{ir:?}");
        assert_eq!(ir.num_inputs, 1);
        assert_eq!(ir.num_evals, 2);
        // Slot 0 (t=0) and slot 1 (t=2). Both must be written.
        assert!(ir
            .ops
            .iter()
            .any(|op| matches!(op, KernelOp::StoreSlot { slot: 0, .. })));
        assert!(ir
            .ops
            .iter()
            .any(|op| matches!(op, KernelOp::StoreSlot { slot: 1, .. })));
    }

    #[test]
    fn dedup_repeated_constants_and_openings() {
        // x*x + x*2 + x*2 — three references to opening(0) and two to const 2.
        let b = ExprBuilder::new();
        let x = b.opening(0);
        let two_a = b.constant(2);
        let two_b = b.constant(2);
        let expr = b.build(x * x + x * two_a + x * two_b);

        let ir = lower_custom_expr(&expr, 1, 2, BindingOrder::LowToHigh);
        assert!(ir.is_valid());

        let load_pairs = ir
            .ops
            .iter()
            .filter(|op| matches!(op, KernelOp::LoadPair { .. }))
            .count();
        assert_eq!(load_pairs, 1, "opening(0) should be loaded once");

        let const_2_count = ir
            .ops
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    KernelOp::Const {
                        value: ConstVal::I128(2),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(const_2_count, 1, "constant 2 should be materialized once");
    }

    #[test]
    fn neg_lowers_to_sub_zero() {
        let b = ExprBuilder::new();
        let x = b.opening(0);
        let expr = b.build(-x);

        let ir = lower_custom_expr(&expr, 1, 1, BindingOrder::LowToHigh);
        assert!(ir.is_valid());

        let zero_const = ir
            .ops
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    KernelOp::Const {
                        value: ConstVal::I128(0),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(zero_const, 1, "Neg should preallocate Const(0)");
    }

    #[test]
    fn degree_one_no_diff() {
        let b = ExprBuilder::new();
        let x = b.opening(0);
        let expr = b.build(x);

        let ir = lower_custom_expr(&expr, 1, 1, BindingOrder::LowToHigh);
        assert!(ir.is_valid());

        let subs = ir
            .ops
            .iter()
            .filter(|op| matches!(op, KernelOp::Sub { .. }))
            .count();
        assert_eq!(subs, 0, "degree=1 should not hoist diffs");
        assert_eq!(ir.num_evals, 1);
    }

    #[test]
    #[should_panic(expected = "degree must be positive")]
    fn degree_zero_panics() {
        let b = ExprBuilder::new();
        let x = b.opening(0);
        let expr = b.build(x);
        let _ = lower_custom_expr(&expr, 1, 0, BindingOrder::LowToHigh);
    }

    #[test]
    #[should_panic(expected = "opening index out of range")]
    fn opening_out_of_range_panics() {
        let b = ExprBuilder::new();
        let x = b.opening(5);
        let expr = b.build(x);
        let _ = lower_custom_expr(&expr, 1, 1, BindingOrder::LowToHigh);
    }
}
