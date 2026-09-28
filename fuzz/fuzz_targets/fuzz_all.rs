#![no_main]

//! Every target in one instrumented binary. `JOLT_FUZZ_TARGET` selects one
//! (the campaign always sets it); without it the first input byte picks the
//! target, so `cargo fuzz run fuzz_all` fuzzes all of them together.

use jolt_akita_fuzz::targets::{by_name, Target, ALL};
use std::sync::OnceLock;

static TARGET: OnceLock<Option<Target>> = OnceLock::new();

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let selected = TARGET.get_or_init(|| {
        std::env::var("JOLT_FUZZ_TARGET")
            .ok()
            .map(|name| by_name(&name).unwrap_or_else(|| panic!("unknown JOLT_FUZZ_TARGET {name}")))
    });
    match (selected, data.split_first()) {
        (Some(run), _) => run(data),
        (None, Some((&index, rest))) => (ALL[usize::from(index) % ALL.len()].1)(rest),
        (None, None) => {}
    }
});
