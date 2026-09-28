//! Liveness-first fuzzing of Jolt with the Akita PCS, independent of the
//! fuzzing engine.
//!
//! Each target is a `pub fn run(data: &[u8])` in [`targets`]; the single
//! libFuzzer entry point `fuzz_targets/fuzz_all.rs` forwards to one of them, so
//! the developer tool can replay and sweep the same code without
//! instrumentation.

// Host-side inline registrations: guests that call these inlines trace and
// prove only when the host links them.
extern crate jolt_inlines_keccak256;
extern crate jolt_inlines_sha2;

pub mod artifacts;
pub mod env;
pub mod gen;
pub mod input;
pub mod liveness;
pub mod opening;
pub mod programs;
pub mod shape;
pub mod stats;
pub mod targets;
pub mod transport;
