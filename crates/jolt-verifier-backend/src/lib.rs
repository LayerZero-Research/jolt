//! Verifier-side scalar arithmetic abstraction.
//!
//! `FieldBackend` lifts the verifier's field-level computation off the concrete
//! [`Field`](jolt_field::Field) so the same verification logic can run against
//! multiple targets:
//!
//! | Backend     | `Scalar`    | What it does                                          |
//! |-------------|-------------|-------------------------------------------------------|
//! | [`Native`]  | `F`         | Direct field ops; identical codegen to native code    |
//! | `Tracing`   | `NodeId`    | Records every op into an AST (recursion / Lean)       |
//! | `R1CSGen`   | `LcId`      | Emits R1CS constraints (replaces hand-rolled R1CS)    |
//!
//! Only [`Native`] is implemented in this crate. Tracing and R1CSGen
//! implementations live downstream; they only need to provide `Scalar` and the
//! trait methods, no other API changes.
//!
//! # Why not just `Field`?
//!
//! The native implementation IS just `Field` with `Scalar = F` and identity
//! wrapping. The trait exists for backends where each operation needs to be
//! observed:
//!
//! - **Tracing** has to know that "this multiplication came from the verifier's
//!   eq evaluation" or "this scalar was sampled from the transcript at round 5".
//! - **R1CSGen** has to emit a fresh R1CS variable for every wrapped scalar
//!   and a constraint for every assertion.
//! - **Provenance** ([`ScalarOrigin`]) lets backends label inputs by source —
//!   public verifier-key data, untrusted proof data, transcript challenges.
//!
//! # Zero overhead
//!
//! [`Native`] is a unit struct. Every method is `#[inline(always)]` and forwards
//! directly to the underlying field operator. Rust's monomorphization erases
//! the trait, producing the same code as if the verifier had been hand-written
//! against `F` from the start.
//!
//! # Helpers
//!
//! [`eq_eval`] and [`univariate_horner`] are free functions that compute the
//! eq polynomial evaluation and a Horner univariate evaluation through the
//! backend. They live here (not in `jolt-poly` / `jolt-sumcheck`) so this
//! crate stays the single source of truth for backend-aware primitives.

#![cfg_attr(not(test), warn(missing_docs))]

mod backend;
mod error;
mod expr_eval;
pub mod helpers;
mod native;
pub mod tracing;

pub use backend::{FieldBackend, ScalarOrigin};
pub use error::BackendError;
pub use expr_eval::evaluate_expr;
pub use helpers::{eq_eval, pow_u64, univariate_horner};
pub use native::Native;
pub use tracing::{replay as replay_trace, AstAssertion, AstGraph, AstNodeId, AstOp, Tracing};
