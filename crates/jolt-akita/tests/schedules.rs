#![expect(
    clippy::expect_used,
    reason = "catalog tests should fail loudly when a table or grid is malformed"
)]

//! The Jolt-owned schedule catalogs: coverage and drift guards.

use jolt_akita::schedules::emit::{
    family_specs, keys, K16_NUM_POLYS, K16_NUM_VARS, K256_NUM_POLYS, K256_NUM_VARS,
};
use jolt_akita::schedules::{jolt_fp128_d64_onehot_k16_table, jolt_fp128_d64_onehot_k256_table};
use jolt_akita::PolynomialGroupLayout;

/// Pins the set of reachable keys that miss their checked-in table and so fall
/// back to planner DP at setup. Today that set is exactly the `(16, 81)` K=16
/// shape, which the planner cannot schedule. Every other reachable key must be
/// catalogued. Identity validity is exercised by every Akita e2e.
#[test]
fn catalogs_cover_every_reachable_one_hot_trace_shape() {
    for (table, num_polys, num_vars) in [
        (
            jolt_fp128_d64_onehot_k16_table(),
            K16_NUM_POLYS,
            K16_NUM_VARS,
        ),
        (
            jolt_fp128_d64_onehot_k256_table(),
            K256_NUM_POLYS,
            K256_NUM_VARS,
        ),
    ] {
        let grid = keys(num_polys, num_vars);
        assert!(!grid.is_empty());
        let missing: Vec<_> = grid
            .into_iter()
            .filter(|key| {
                !table.entries.iter().any(|entry| {
                    entry.root.final_group.layout == *key
                        && entry.root.precommitted_groups.is_empty()
                })
            })
            .collect();
        let expected_missing = if num_vars == K16_NUM_VARS {
            vec![PolynomialGroupLayout::new(16, 81)]
        } else {
            Vec::new()
        };
        assert_eq!(
            missing, expected_missing,
            "reachable catalog misses changed; generate a schedule or review the explicit DP fallback set"
        );
        assert_eq!(
            table.identity.key_count,
            table.entries.len(),
            "identity key count must match the table"
        );
    }
}

/// Splits Rust source into a whitespace-insensitive token stream:
/// identifier/number runs stay whole, every other non-whitespace character is
/// its own token. The planner emits unformatted source while the checked-in
/// modules are rustfmt-formatted (outside the `#[rustfmt::skip]` tables), so a
/// byte-for-byte oracle reports pure formatting as drift; token equality
/// detects every semantic change while ignoring layout. The checked-in file's
/// formatting itself is enforced by the workspace `cargo fmt` lane.
fn source_tokens(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in source.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            current.push(ch);
        } else {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            if !ch.is_whitespace() {
                tokens.push(ch.to_string());
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Regenerates both family modules through the planner DP and compares their
/// token streams against the checked-in tables. Slow (re-runs every DP
/// solve) — run explicitly:
/// `cargo nextest run -p jolt-akita catalogs_match_planner --run-ignored all`
#[test]
#[ignore = "regenerates every schedule through the planner DP (minutes)"]
fn catalogs_match_planner_regeneration() {
    for spec in family_specs(std::path::PathBuf::new()) {
        let regenerated =
            akita_planner::emit::emit_family_module(&spec).expect("regeneration must succeed");
        let checked_in = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/schedules")
                .join(format!("{}.rs", spec.module_name)),
        )
        .expect("checked-in table must exist");
        let regenerated = source_tokens(&regenerated);
        let checked_in = source_tokens(&checked_in);
        if let Some(index) = (0..regenerated.len().max(checked_in.len()))
            .find(|&index| regenerated.get(index) != checked_in.get(index))
        {
            let context = |tokens: &[String]| {
                tokens[index.saturating_sub(8)..(index + 8).min(tokens.len())].join(" ")
            };
            assert_eq!(
                regenerated.get(index),
                checked_in.get(index),
                "{} drifted from the planner DP — regenerate via gen_jolt_schedules\n  \
                 first mismatch at token {index}\n  planner:    …{}…\n  checked-in: …{}…",
                spec.module_name,
                context(&regenerated),
                context(&checked_in),
            );
        }
    }
}
