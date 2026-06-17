//! Akita PCS adapter.

mod commitment_scheme;
mod wrappers;

pub use commitment_scheme::{Fp128Dense128Config, JoltAkitaCommitmentScheme};
pub use wrappers::JoltToAkitaTranscript;
