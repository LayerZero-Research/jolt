//! Grumpkin operations optimized for Jolt zkVM.
//!
//! This module is a thin facade that selects the backend:
//! - default: a minimal no-arkworks Montgomery backend (guest-friendly)
//! - `arkworks` feature: Arkworks-backed types and helpers (host-friendly)

#[cfg(feature = "arkworks")]
pub use crate::backend_arkworks::*;

#[cfg(not(feature = "arkworks"))]
pub use crate::backend_mont::*;
