//! Akita PCS adapter.

mod commitment_scheme;
mod packed_layout;
mod packed_poly;
mod wrappers;

pub use commitment_scheme::{Fp128OneHot32Config, JoltAkitaCommitmentScheme};
pub use wrappers::JoltToAkitaTranscript;

#[cfg(test)]
mod tests;
