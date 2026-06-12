//! Akita PCS adapter.
//!
//! The module path and exported type names still use `hachi` as a legacy name
//! from the original integration. A future mechanical rename should switch this
//! module and public aliases to `akita`.

mod commitment_scheme;
mod packed_layout;
mod packed_poly;
mod wrappers;

pub use commitment_scheme::{Fp128OneHot32Config, JoltHachiCommitmentScheme};
pub use wrappers::JoltToHachiTranscript;

#[cfg(test)]
mod tests;
