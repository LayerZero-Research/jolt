#![allow(
    clippy::assertions_on_result_states,
    clippy::from_over_into,
    clippy::len_without_is_empty,
    clippy::needless_range_loop,
    clippy::new_without_default,
    clippy::too_long_first_doc_paragraph,
    long_running_const_eval,
    non_snake_case,
    type_alias_bounds
)]
#[cfg(all(feature = "dory-pcs", feature = "akita-pcs"))]
compile_error!("Features `dory-pcs` and `akita-pcs` are mutually exclusive.");
#[cfg(all(
    any(feature = "host", feature = "prover"),
    not(any(feature = "dory-pcs", feature = "akita-pcs"))
))]
compile_error!("Enable exactly one PCS backend: `dory-pcs` or `akita-pcs`.");
#[cfg(all(feature = "akita-pcs", feature = "zk"))]
compile_error!("`akita-pcs` does not support the `zk` feature yet.");

#[cfg(feature = "host")]
pub mod host;

pub mod curve;
pub mod field;
pub mod guest;
pub mod msm;
pub mod poly;
pub mod subprotocols;
pub mod transcripts;
pub mod utils;
pub mod zkvm;
pub use ark_bn254;

// Re-export AdviceTape type for use in generated code
pub use tracer::emulator::cpu::AdviceTape;
