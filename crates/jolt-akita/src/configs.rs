//! Jolt-local Akita commitment configs.
//!
//! Each config delegates every policy decision (field, ring, decomposition,
//! SIS profile, chunking) to its upstream proof-optimized preset and
//! overrides the schedule catalog and setup sizing hooks,
//! so the generated schedule tables for Jolt's `OneHotTrace` shapes live in this
//! crate (see [`crate::schedules`]) while the planner policy keeps one
//! upstream owner. The catalog is identity-validated against the config's
//! policy on every lookup, so a policy/table drift hard-errors instead of
//! silently planning a different schedule.

use akita_config::CommitmentConfig;
use akita_pcs::AkitaError;
use akita_planner::GeneratedScheduleTable;
use akita_types::{
    setup_matrix_envelope_for_schedule, AkitaScheduleLookupKey, SetupMatrixEnvelope,
};

/// Sizes a production OneHotTrace setup directly from the checked-in Jolt catalog.
///
/// `Some` means the requested maximum shape itself is catalog-backed. Smaller
/// catalog rows are included because setup matrices are shared prefix views
/// and planned footprints are not monotone in either layout dimension.
///
/// WARNING: the catalog holds single-group rows only, but the shared trace setup
/// also backs the precommitted groups of a fused multi-group root fold (see
/// [`crate::multi_group`]), whose schedule this table cannot model. Sizing from
/// catalog rows alone could therefore land *below* the fold's requirement and
/// fail closed with `InvalidSetup` at prove time. The result is floored at the
/// base preset's own envelope so the catalog can only ever widen the setup,
/// never narrow it. Over-sizing is free: matrices are seeded from a flat index,
/// so a shorter setup is a prefix of a longer one and commitments are unchanged.
fn catalog_setup_envelope<Cfg: CommitmentConfig, Base: CommitmentConfig>(
    table: GeneratedScheduleTable,
    max_num_vars: usize,
    max_num_batched_polys: usize,
) -> Result<Option<SetupMatrixEnvelope>, AkitaError> {
    let requested_shape_is_catalogued = table.entries.iter().any(|entry| {
        entry.root.precommitted_groups.is_empty()
            && entry.root.final_group.layout.num_vars() == max_num_vars
            && entry.root.final_group.layout.num_polynomials() == max_num_batched_polys
    });
    if !requested_shape_is_catalogued {
        return Ok(None);
    }

    let mut envelope = Base::max_setup_matrix_size(max_num_vars, max_num_batched_polys)?;
    for entry in table.entries.iter().filter(|entry| {
        entry.root.precommitted_groups.is_empty()
            && entry.root.final_group.layout.num_vars() <= max_num_vars
            && entry.root.final_group.layout.num_polynomials() <= max_num_batched_polys
    }) {
        let schedule = Cfg::runtime_schedule(AkitaScheduleLookupKey::single(
            entry.root.final_group.layout,
        ))?;
        let entry_envelope = setup_matrix_envelope_for_schedule(&schedule)?;
        envelope.max_setup_len = envelope.max_setup_len.max(entry_envelope.max_setup_len);
    }
    Ok(Some(envelope))
}

/// Delegates a [`CommitmentConfig`] to an upstream preset, overriding its
/// schedule catalog and catalog-backed setup sizing. `get_params_for_prove`
/// re-derives the single-group lookup key through the public layout API and
/// resolves it against the checked-in catalog. The fused stage-8 root fold
/// (see [`crate::multi_group`]) submits a multi-group layout, which the
/// catalogs do not cover, so every fused prove falls back to the base preset's
/// DP planning.
macro_rules! delegate_preset {
    ($(#[$doc:meta])* $name:ident, $base:ty, $catalog:expr) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name;

        impl CommitmentConfig for $name {
            type Field = <$base as CommitmentConfig>::Field;
            type ExtField = <$base as CommitmentConfig>::ExtField;
            const D: usize = <$base as CommitmentConfig>::D;

            fn decomposition() -> akita_types::DecompositionParams {
                <$base>::decomposition()
            }

            fn ring_challenge_config(
                d: usize,
            ) -> Result<akita_challenges::SparseChallengeConfig, akita_pcs::AkitaError>
            {
                <$base>::ring_challenge_config(d)
            }

            fn fold_challenge_shape_at_level(
                inputs: akita_types::AkitaScheduleInputs,
            ) -> akita_challenges::TensorChallengeShape {
                <$base>::fold_challenge_shape_at_level(inputs)
            }

            fn sis_modulus_profile() -> akita_types::SisModulusProfileId {
                <$base>::sis_modulus_profile()
            }

            fn ring_subfield_embedding_norm_bound() -> u32 {
                <$base>::ring_subfield_embedding_norm_bound()
            }

            fn max_setup_matrix_size(
                max_num_vars: usize,
                max_num_batched_polys: usize,
            ) -> Result<akita_types::SetupMatrixEnvelope, akita_pcs::AkitaError> {
                if max_num_batched_polys == 0 {
                    return Err(akita_pcs::AkitaError::InvalidSetup(
                        "max_num_batched_polys must be at least 1".to_string(),
                    ));
                }
                if let Some(envelope) = catalog_setup_envelope::<Self, $base>(
                    $catalog,
                    max_num_vars,
                    max_num_batched_polys,
                )? {
                    return Ok(envelope);
                }
                <$base>::max_setup_matrix_size(max_num_vars, max_num_batched_polys)
            }

            fn basis_range() -> (u32, u32) {
                <$base>::basis_range()
            }

            fn onehot_chunk_size() -> usize {
                <$base>::onehot_chunk_size()
            }

            fn chunked_witness_cfg() -> akita_types::ChunkedWitnessCfg {
                <$base>::chunked_witness_cfg()
            }

            fn recursive_setup_planning() -> bool {
                <$base>::recursive_setup_planning()
            }

            fn supports_multi_group_final_commit() -> bool {
                <$base>::supports_multi_group_final_commit()
            }

            fn schedule_catalog() -> Option<akita_planner::GeneratedScheduleTable> {
                Some($catalog)
            }

            fn get_params_for_prove(
                layout: &akita_types::OpeningClaimsLayout,
            ) -> Result<akita_types::FoldSchedule, akita_pcs::AkitaError> {
                if layout.num_groups() == 1 {
                    layout.check()?;
                    Self::runtime_schedule(akita_types::AkitaScheduleLookupKey::single(
                        layout.root_final_group_layout()?,
                    ))
                } else {
                    <$base>::get_params_for_prove(layout)
                }
            }
        }
    };
}

delegate_preset!(
    /// `D64OneHotK16` with the Jolt-generated K=16 schedule catalog.
    JoltD64OneHotK16,
    akita_config::proof_optimized::fp128::D64OneHotK16,
    crate::schedules::jolt_fp128_d64_onehot_k16_table()
);

delegate_preset!(
    /// `D64OneHot` (K=256) with the Jolt-generated large-trace catalog.
    JoltD64OneHotK256,
    akita_config::proof_optimized::fp128::D64OneHot,
    crate::schedules::jolt_fp128_d64_onehot_k256_table()
);

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "catalog setup tests should fail loudly on malformed schedules"
)]
mod tests {
    use super::*;

    type K16Base = akita_config::proof_optimized::fp128::D64OneHotK16;
    type K256Base = akita_config::proof_optimized::fp128::D64OneHot;

    #[test]
    fn production_one_hot_trace_shapes_use_catalog_setup_sizing() {
        let k16 = crate::schedules::jolt_fp128_d64_onehot_k16_table();
        assert!(
            catalog_setup_envelope::<JoltD64OneHotK16, K16Base>(k16, 28, 81)
                .unwrap()
                .is_some()
        );

        let k256 = crate::schedules::jolt_fp128_d64_onehot_k256_table();
        assert!(
            catalog_setup_envelope::<JoltD64OneHotK256, K256Base>(k256, 38, 41)
                .unwrap()
                .is_some()
        );
    }

    /// The shared trace setup also backs the precommitted groups of a fused
    /// multi-group root fold, whose schedule the single-group catalog cannot
    /// model. Sizing from catalog rows alone could land below the fold's
    /// requirement, so the catalog path must never size under the base preset.
    #[test]
    fn catalog_sizing_never_undercuts_the_base_preset() {
        for (num_vars, num_polys) in [(28usize, 81usize), (24, 49), (20, 1)] {
            let table = crate::schedules::jolt_fp128_d64_onehot_k16_table();
            let Some(envelope) =
                catalog_setup_envelope::<JoltD64OneHotK16, K16Base>(table, num_vars, num_polys)
                    .unwrap()
            else {
                continue;
            };
            let base = K16Base::max_setup_matrix_size(num_vars, num_polys).unwrap();
            assert!(
                envelope.max_setup_len >= base.max_setup_len,
                "catalog sized {} below the base preset's {} at ({num_vars}, {num_polys})",
                envelope.max_setup_len,
                base.max_setup_len,
            );
        }
    }
}
