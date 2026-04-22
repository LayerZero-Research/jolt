//! Lower-level register-machine kernel IR for backend codegen.
//!
//! [`KernelIR`] is a flat, register-machine representation of one
//! composition-reduce kernel body. Backends consume this IR directly: CPU walks
//! it as an interpreter or generates straight-line Rust code; Metal walks it as
//! an MSL emitter. The IR is the contract between [`Expr`](crate::Expr) (and
//! its `Custom` `KernelShape` parent) and backend codegen.
//!
//! # Why this exists
//!
//! Today's `KernelShape::Custom` is an `Expr` AST that each backend walks
//! independently. Metal's codegen produces near-optimal SSA-form MSL with
//! pre-computed diffs and incremental interpolation
//! (`crates/jolt-metal/src/compiler.rs::generate_custom_body`). CPU's codegen
//! produces a stack-VM closure with per-call heap allocation and dispatch
//! overhead (`crates/jolt-cpu/src/custom.rs::kernel_from_ops`).
//!
//! Lifting the implicit "register IR" Metal already uses into a first-class
//! type lets both backends consume the same representation and removes the
//! asymmetry. See `Notes/jolt-compute-backend-walk-the-walk-2026-04-21.md`
//! Appendix A for the full motivation.
//!
//! # Design
//!
//! - **Field-agnostic.** Constants are `i128`, lifted to `F` at backend time
//!   via `F::from_i128`. Field-sized values (challenges, gamma powers) enter
//!   the IR via [`KernelOp::LoadChallenge`] and are resolved at compile time
//!   from a backend-supplied `&[F]`.
//!
//! - **Register-typed, not stack-typed.** Every op names its operands and
//!   destination explicitly. There is no implicit stack to push/pop, so the
//!   interpreter fits in a fixed-size stack-allocated register file.
//!
//! - **Iteration model is explicit.** [`KernelIteration`] tells the backend
//!   how the body is dispatched: per-pair (with a [`BindingOrder`]), per-
//!   element, etc. Loads are typed to the iteration model:
//!   [`KernelOp::LoadPair`] for `PerPair`, [`KernelOp::LoadOne`] for
//!   `PerElement`.

/// Register identifier in the per-iteration register file.
///
/// Backends interpret this as an opaque index. [`KernelIR::num_registers`]
/// upper-bounds the maximum `RegId + 1`.
pub type RegId = u16;

/// Buffer identifier — index into the kernel's input buffer slice.
///
/// Valid range: `0..ir.num_inputs`.
pub type BufId = u16;

/// How the IR body is dispatched across the input data.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KernelIteration {
    /// Body runs once per pair index `i`. Loads use [`KernelOp::LoadPair`].
    ///
    /// `BindingOrder` determines the gather pattern:
    /// - `LowToHigh`: `lo = buf[2i]`, `hi = buf[2i+1]` (interleaved)
    /// - `HighToLow`: `lo = buf[i]`, `hi = buf[i + n_pairs]` (split-half)
    PerPair { binding_order: BindingOrder },
    /// Body runs once per element index `i`. Loads use [`KernelOp::LoadOne`].
    PerElement,
}

/// Variable binding order for [`KernelIteration::PerPair`].
///
/// Mirrors `jolt_compute::BindingOrder`. Duplicated here to keep `jolt-ir`
/// independent of `jolt-compute` (the IR may be consumed by tools that don't
/// link the compute crate).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum BindingOrder {
    /// Interleaved layout. Pairs `(buf[2i], buf[2i+1])`.
    #[default]
    LowToHigh,
    /// Split-half layout. Pairs `(buf[i], buf[i + n_pairs])`.
    HighToLow,
}

/// Compile-time-known constant value.
///
/// Lowered to a field element at backend time via `F::from_i128`. Field-sized
/// values (challenges, gamma powers) do not use this enum — they enter via
/// [`KernelOp::LoadChallenge`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConstVal {
    /// Small integer constant. Lifted via `F::from_i128`.
    I128(i128),
}

/// One operation in the per-iteration body.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KernelOp {
    /// `regs[dst_lo] = lo[buf]; regs[dst_hi] = hi[buf];`
    /// Valid only with [`KernelIteration::PerPair`].
    LoadPair {
        buf: BufId,
        dst_lo: RegId,
        dst_hi: RegId,
    },
    /// `regs[dst] = elem[buf]` where `elem` is the per-element view.
    /// Valid only with [`KernelIteration::PerElement`].
    LoadOne { buf: BufId, dst: RegId },
    /// `regs[dst] = challenges[idx]` where `challenges` is supplied by the
    /// backend at compile time.
    LoadChallenge { idx: u8, dst: RegId },
    /// `regs[dst] = F::from_i128(value)`.
    Const { value: ConstVal, dst: RegId },
    /// `regs[dst] = regs[lhs] + regs[rhs]`
    Add { lhs: RegId, rhs: RegId, dst: RegId },
    /// `regs[dst] = regs[lhs] - regs[rhs]`
    Sub { lhs: RegId, rhs: RegId, dst: RegId },
    /// `regs[dst] = regs[lhs] * regs[rhs]`
    Mul { lhs: RegId, rhs: RegId, dst: RegId },
    /// `regs[dst] = regs[a] * regs[b] + regs[c]` (fused multiply-add).
    Fma {
        a: RegId,
        b: RegId,
        c: RegId,
        dst: RegId,
    },
    /// `out[slot] = regs[src]` where `out` is the kernel's evaluation output
    /// (length = `ir.num_evals`).
    StoreSlot { slot: u8, src: RegId },
}

/// A complete kernel description suitable for backend lowering.
///
/// # Example
///
/// Booleanity check `γ · (h² − h)` evaluated on the standard grid `{0, 2}`:
///
/// ```
/// use jolt_ir::{
///     BindingOrder, KernelIR, KernelIteration, KernelOp, ConstVal,
/// };
///
/// // Registers:
/// //   r0, r1: lo, hi for opening 0 (h)
/// //   r2:    challenge 0 (gamma)
/// //   r3:    diff = hi - lo
/// //   r4:    p = interpolated value of h
/// //   r5:    p * p
/// //   r6:    p * p - p
/// //   r7:    gamma * (p * p - p) = output
/// //   r8:    constant 2 for slot-1 interpolation
/// let ir = KernelIR {
///     num_inputs: 1,
///     num_registers: 9,
///     num_evals: 2,
///     iteration: KernelIteration::PerPair { binding_order: BindingOrder::LowToHigh },
///     ops: vec![
///         // Preamble
///         KernelOp::LoadPair { buf: 0, dst_lo: 0, dst_hi: 1 },
///         KernelOp::LoadChallenge { idx: 0, dst: 2 },
///         KernelOp::Sub { lhs: 1, rhs: 0, dst: 3 },        // diff
///         // Slot 0: t = 0, p = lo
///         KernelOp::Mul { lhs: 0, rhs: 0, dst: 5 },        // p * p
///         KernelOp::Sub { lhs: 5, rhs: 0, dst: 6 },        // p * p - p
///         KernelOp::Mul { lhs: 2, rhs: 6, dst: 7 },        // gamma * (...)
///         KernelOp::StoreSlot { slot: 0, src: 7 },
///         // Slot 1: t = 2, p = lo + 2 * diff = Fma(diff, 2, lo)
///         KernelOp::Const { value: ConstVal::I128(2), dst: 8 },
///         KernelOp::Fma { a: 3, b: 8, c: 0, dst: 4 },
///         KernelOp::Mul { lhs: 4, rhs: 4, dst: 5 },
///         KernelOp::Sub { lhs: 5, rhs: 4, dst: 6 },
///         KernelOp::Mul { lhs: 2, rhs: 6, dst: 7 },
///         KernelOp::StoreSlot { slot: 1, src: 7 },
///     ],
/// };
/// assert!(ir.is_valid());
/// ```
#[derive(Clone, Debug)]
pub struct KernelIR {
    /// Number of input buffers. Loads must reference `buf < num_inputs`.
    pub num_inputs: u16,
    /// Total register file size. All `RegId`s in `ops` must be `< num_registers`.
    pub num_registers: u16,
    /// Number of evaluation slots in the output buffer. `StoreSlot.slot` must
    /// be `< num_evals`.
    pub num_evals: u8,
    /// Iteration model.
    pub iteration: KernelIteration,
    /// Body, executed once per iteration unit.
    pub ops: Vec<KernelOp>,
}

impl KernelIR {
    /// Validates that the IR is internally consistent.
    ///
    /// Checks:
    /// - All buffer references are `< num_inputs`
    /// - All register references are `< num_registers`
    /// - All slot writes are `< num_evals`
    /// - Load ops match the iteration model
    /// - Each `slot < num_evals` is written by at least one `StoreSlot`
    /// - No empty body
    pub fn is_valid(&self) -> bool {
        if self.ops.is_empty() {
            return false;
        }

        let n_buf = self.num_inputs;
        let n_reg = self.num_registers;
        let n_eval = self.num_evals;

        let mut slot_written = vec![false; n_eval as usize];
        let is_per_pair = matches!(self.iteration, KernelIteration::PerPair { .. });

        for op in &self.ops {
            let ok = match *op {
                KernelOp::LoadPair {
                    buf,
                    dst_lo,
                    dst_hi,
                } => is_per_pair && buf < n_buf && dst_lo < n_reg && dst_hi < n_reg,
                KernelOp::LoadOne { buf, dst } => !is_per_pair && buf < n_buf && dst < n_reg,
                KernelOp::LoadChallenge { dst, .. } => dst < n_reg,
                KernelOp::Const { dst, .. } => dst < n_reg,
                KernelOp::Add { lhs, rhs, dst }
                | KernelOp::Sub { lhs, rhs, dst }
                | KernelOp::Mul { lhs, rhs, dst } => lhs < n_reg && rhs < n_reg && dst < n_reg,
                KernelOp::Fma { a, b, c, dst } => {
                    a < n_reg && b < n_reg && c < n_reg && dst < n_reg
                }
                KernelOp::StoreSlot { slot, src } => {
                    if (slot as usize) < slot_written.len() && src < n_reg {
                        slot_written[slot as usize] = true;
                        true
                    } else {
                        false
                    }
                }
            };
            if !ok {
                return false;
            }
        }

        slot_written.iter().all(|w| *w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn booleanity_example_validates() {
        let ir = KernelIR {
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
        };
        assert!(ir.is_valid());
    }

    #[test]
    fn empty_body_invalid() {
        let ir = KernelIR {
            num_inputs: 0,
            num_registers: 0,
            num_evals: 1,
            iteration: KernelIteration::PerPair {
                binding_order: BindingOrder::LowToHigh,
            },
            ops: vec![],
        };
        assert!(!ir.is_valid());
    }

    #[test]
    fn missing_slot_write_invalid() {
        let ir = KernelIR {
            num_inputs: 1,
            num_registers: 2,
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
                KernelOp::StoreSlot { slot: 0, src: 0 },
            ],
        };
        assert!(!ir.is_valid());
    }

    #[test]
    fn out_of_range_register_invalid() {
        let ir = KernelIR {
            num_inputs: 1,
            num_registers: 2,
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
                KernelOp::Mul {
                    lhs: 0,
                    rhs: 1,
                    dst: 99,
                },
                KernelOp::StoreSlot { slot: 0, src: 99 },
            ],
        };
        assert!(!ir.is_valid());
    }

    #[test]
    fn load_one_in_per_pair_invalid() {
        let ir = KernelIR {
            num_inputs: 1,
            num_registers: 2,
            num_evals: 1,
            iteration: KernelIteration::PerPair {
                binding_order: BindingOrder::LowToHigh,
            },
            ops: vec![
                KernelOp::LoadOne { buf: 0, dst: 0 },
                KernelOp::StoreSlot { slot: 0, src: 0 },
            ],
        };
        assert!(!ir.is_valid());
    }
}
