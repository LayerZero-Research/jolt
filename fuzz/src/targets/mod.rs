//! Engine-independent target entry points.

pub mod grid;
pub mod planning;
pub mod program;

/// A target's engine-independent entry point.
pub type Target = fn(&[u8]);

/// Every target, keyed by its campaign name (`JOLT_FUZZ_TARGET`).
pub const ALL: &[(&str, Target)] = &[
    ("planning", planning::run),
    ("grid", grid::run),
    ("program", program::run),
];

/// Structured seeds a target contributes beyond the generic ones.
pub fn seeds(name: &str) -> Vec<(String, Vec<u8>)> {
    match name {
        "planning" => planning::seeds(),
        "grid" => grid::seeds(),
        "program" => program::seeds(),
        _ => Vec::new(),
    }
}

pub fn by_name(name: &str) -> Option<Target> {
    ALL.iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, run)| *run)
}
