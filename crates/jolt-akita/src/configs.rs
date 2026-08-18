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
    commit_only_setup_field_elements, setup_matrix_capacity_for_schedule, AkitaScheduleLookupKey,
    SetupMatrixCapacity,
};

fn dp_planned_schedule<Cfg: CommitmentConfig>(
    key: &AkitaScheduleLookupKey,
) -> Result<akita_types::FoldSchedule, AkitaError> {
    let planned = akita_planner::find_schedule(
        key,
        Cfg::root_honest_fold_policy(),
        &[],
        &akita_config::policy_of::<Cfg>(),
        Cfg::ring_challenge_config,
    )?;
    planned.schedule.validate_structure()?;
    Ok(planned.schedule)
}

/// Folds one row's footprint into `capacity`.
///
/// A prefix group is committed independently before the final grouped key
/// exists. Its A/B footprint therefore belongs in the setup even when the
/// complete grouped key exceeds this setup's total capacity, so prefix
/// accounting runs before the whole-key filter.
fn fold_row_capacity(
    capacity: &mut SetupMatrixCapacity,
    key: &AkitaScheduleLookupKey,
    schedule: impl FnOnce() -> Result<akita_types::FoldSchedule, AkitaError>,
    max_num_vars: usize,
    max_num_batched_polys: usize,
) -> Result<(), AkitaError> {
    for precommitted in &key.precommitteds {
        if AkitaScheduleLookupKey::single(precommitted.group)
            .fits_setup_capacity(max_num_vars, max_num_batched_polys)?
        {
            let commit_only = commit_only_setup_field_elements(
                &precommitted.inner_commit_matrix,
                &precommitted.outer_commit_matrix,
                precommitted.outer_slice_count,
            )?;
            capacity.num_field_elements = capacity.num_field_elements.max(commit_only);
        }
    }

    if !key.fits_setup_capacity(max_num_vars, max_num_batched_polys)? {
        return Ok(());
    }
    let row_capacity = setup_matrix_capacity_for_schedule(&schedule()?)?;
    capacity.num_field_elements = capacity
        .num_field_elements
        .max(row_capacity.num_field_elements);
    Ok(())
}

/// Sizes a production OneHotTrace setup from the requested fallback row, every
/// checked-in Jolt row that fits the advertised extent, and every row
/// preprocessing provisioned for this config.
///
/// Setup footprints are not monotone in either layout dimension. The DP row
/// for the maximum shape therefore cannot stand in for smaller catalog rows,
/// even when the maximum shape itself is not catalog-backed.
///
/// Provisioned rows are sized here rather than at commit time because setup
/// construction precedes every commit: a grouped row planned by
/// [`crate::schedule_registry::provision`] must already be installed when this
/// runs, or its matrices would be missing from the setup.
fn catalog_setup_capacity<Cfg: CommitmentConfig + 'static>(
    table: &GeneratedScheduleTable,
    max_num_vars: usize,
    max_num_batched_polys: usize,
) -> Result<SetupMatrixCapacity, AkitaError> {
    let fallback_key = AkitaScheduleLookupKey::single(
        akita_types::OpeningClaimsLayout::new(max_num_vars, max_num_batched_polys)?
            .root_final_group_layout()?,
    );
    let mut capacity =
        setup_matrix_capacity_for_schedule(&dp_planned_schedule::<Cfg>(&fallback_key)?)?;
    for entry in table.entries {
        let key = entry.to_runtime_lookup_key();
        fold_row_capacity(
            &mut capacity,
            &key,
            // Catalog-only: this loop is already iterating the catalog, and
            // routing through the registry-aware hook would re-enter sizing.
            || Ok(crate::schedule_registry::catalog_only_row::<Cfg>(&key)?.into_schedule()),
            max_num_vars,
            max_num_batched_polys,
        )?;
    }
    for row in crate::schedule_registry::registered_rows::<Cfg>()?.rows() {
        let profiles = row.profiles();
        let key = AkitaScheduleLookupKey {
            final_group: profiles.final_group.group,
            precommitteds: profiles.precommitteds.clone(),
        };
        fold_row_capacity(
            &mut capacity,
            &key,
            || Ok(row.schedule().clone()),
            max_num_vars,
            max_num_batched_polys,
        )?;
    }
    Ok(capacity)
}

/// Delegates a [`CommitmentConfig`] to an upstream preset, overriding its
/// schedule catalog and catalog-backed setup sizing.
macro_rules! delegate_preset {
    (
        $(#[$doc:meta])*
        $name:ident,
        $base:ty,
        $root_honest_fold_policy:expr,
        $catalog:expr
    ) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name;

        impl CommitmentConfig for $name {
            type Field = <$base as CommitmentConfig>::Field;
            type ExtField = <$base as CommitmentConfig>::ExtField;
            const D: usize = <$base as CommitmentConfig>::D;
            const RING_DIMENSION_SCHEDULE_MODE: akita_schedules::RingDimensionScheduleMode =
                <$base as CommitmentConfig>::RING_DIMENSION_SCHEDULE_MODE;
            const EXT_DEGREE: usize = <$base as CommitmentConfig>::EXT_DEGREE;

            fn decomposition() -> akita_types::DecompositionParams {
                <$base>::decomposition()
            }

            fn ring_challenge_config(
                d: usize,
            ) -> Result<akita_challenges::SparseChallengeConfig, akita_pcs::AkitaError>
            {
                <$base>::ring_challenge_config(d)
            }

            fn selection_policy() -> akita_schedules::SelectionPolicyId {
                <$base>::selection_policy()
            }

            fn sis_modulus_profile() -> akita_types::SisModulusProfileId {
                <$base>::sis_modulus_profile()
            }

            fn setup_matrix_capacity(
                max_num_vars: usize,
                max_num_batched_polys: usize,
            ) -> Result<akita_types::SetupMatrixCapacity, akita_pcs::AkitaError> {
                if max_num_batched_polys == 0 {
                    return Err(akita_pcs::AkitaError::InvalidSetup(
                        "max_num_batched_polys must be at least 1".to_string(),
                    ));
                }
                if let Some(table) = $catalog {
                    return catalog_setup_capacity::<Self>(
                        &table,
                        max_num_vars,
                        max_num_batched_polys,
                    );
                }
                let key = AkitaScheduleLookupKey::single(
                    akita_types::OpeningClaimsLayout::new(
                        max_num_vars,
                        max_num_batched_polys,
                    )?
                    .root_final_group_layout()?,
                );
                setup_matrix_capacity_for_schedule(&dp_planned_schedule::<Self>(&key)?)
            }

            fn opening_basis_range() -> (u32, u32) {
                <$base>::opening_basis_range()
            }

            fn inner_basis_range() -> (u32, u32) {
                <$base>::inner_basis_range()
            }

            fn root_honest_fold_policy() -> akita_types::sis::HonestFoldPolicySpec {
                $root_honest_fold_policy
            }

            fn chunked_witness_cfg() -> akita_types::ChunkedWitnessCfg {
                <$base>::chunked_witness_cfg()
            }

            fn recursive_setup_planning() -> bool {
                <$base>::recursive_setup_planning()
            }

            fn schedule_catalog() -> Option<akita_schedules::GeneratedScheduleTable> {
                $catalog
            }

            // A grouped row's key contains the frozen precommit profile, which
            // follows the program's advice capacity and so cannot be emitted
            // offline. Preprocessing plans those rows into the registry; every
            // resolution hook therefore checks it before the checked-in
            // catalog. A miss falls through to the catalog, never to the
            // planner, which keeps planner DP off the prove/verify path.
            //
            // The fallback arms replicate the trait's default bodies; they
            // cannot be delegated to because a default body is unreachable from
            // its own override.
            fn resolve_catalog_row_for_key(
                key: &AkitaScheduleLookupKey,
            ) -> Result<akita_schedules::ResolvedScheduleRow, akita_pcs::AkitaError> {
                if let Some(row) = crate::schedule_registry::lookup_key::<Self>(key) {
                    return Ok(row);
                }
                Self::validate_sis_modulus_profile()?;
                akita_schedules::resolve_generated_catalog_row_for_key(
                    key,
                    &akita_config::policy_of::<Self>(),
                    Self::ring_challenge_config,
                    Self::schedule_catalog(),
                )
            }

            fn resolve_catalog_row_for_profiles(
                profiles: &akita_types::CommittedGroupBatchProfile,
            ) -> Result<akita_schedules::ResolvedScheduleRow, akita_pcs::AkitaError> {
                if let Some(row) = crate::schedule_registry::lookup_profiles::<Self>(profiles) {
                    return Ok(row);
                }
                Self::validate_sis_modulus_profile()?;
                profiles.validate(Self::decomposition().field_bits())?;
                akita_schedules::resolve_generated_catalog_row_for_profiles(
                    &AkitaScheduleLookupKey {
                        final_group: profiles.final_group.group,
                        precommitteds: profiles.precommitteds.clone(),
                    },
                    profiles,
                    &akita_config::policy_of::<Self>(),
                    Self::ring_challenge_config,
                    Self::schedule_catalog(),
                )
            }

            fn resolve_schedule_selection(
                selection: akita_types::OpeningScheduleSelection,
            ) -> Result<akita_schedules::ResolvedScheduleRow, akita_pcs::AkitaError> {
                if let Some(row) = crate::schedule_registry::lookup_selection::<Self>(selection) {
                    return Ok(row);
                }
                Self::validate_sis_modulus_profile()?;
                akita_schedules::resolve_generated_schedule_selection(
                    selection,
                    &akita_config::policy_of::<Self>(),
                    Self::ring_challenge_config,
                    Self::schedule_catalog(),
                )
            }
        }
    };
}

delegate_preset!(
    /// Adaptive one-hot config with the Jolt-generated K=16 schedule catalog.
    JoltOneHotK16,
    akita_config::proof_optimized::fp128::OneHot,
    akita_types::sis::HonestFoldPolicySpec::UnitOneHot(
        akita_types::sis::UnitOneHotFoldPolicy::new(128, 1, 16),
    ),
    crate::schedules::jolt_fp128_onehot_k16_table()
);

#[cfg(feature = "akita-test-schedules")]
delegate_preset!(
    /// Nonproduction K=16 config for small trusted-advice grouped fixtures.
    JoltOneHotK16Fixture,
    akita_config::proof_optimized::fp128::OneHot,
    akita_types::sis::HonestFoldPolicySpec::UnitOneHot(
        akita_types::sis::UnitOneHotFoldPolicy::new(128, 1, 16),
    ),
    crate::schedules::jolt_fp128_onehot_k16_fixture_table()
);

delegate_preset!(
    /// Adaptive one-hot config with the Jolt-generated K=256 schedule catalog.
    JoltOneHotK256,
    akita_config::proof_optimized::fp128::OneHot,
    akita_config::proof_optimized::fp128::OneHot::root_honest_fold_policy(),
    crate::schedules::jolt_fp128_onehot_k256_table()
);

delegate_preset!(
    /// Adaptive dense config with the Jolt-generated advice/program byte-object catalog.
    JoltDense,
    akita_config::proof_optimized::fp128::Dense,
    akita_config::proof_optimized::fp128::Dense::root_honest_fold_policy(),
    crate::schedules::jolt_fp128_dense_table()
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_shapes_have_setup_capacities() {
        assert!(JoltDense::setup_matrix_capacity(14, 2).is_ok());
        assert!(JoltOneHotK16::setup_matrix_capacity(34, 1).is_ok());
        assert!(JoltOneHotK256::setup_matrix_capacity(43, 1).is_ok());
    }

    #[test]
    #[expect(clippy::unwrap_used)]
    fn k256_policy_uses_adaptive_dimensions() {
        assert_eq!(JoltOneHotK256::D, 256);
        assert_eq!(JoltOneHotK256::inner_basis_range(), (3, 11));
        assert_eq!(JoltOneHotK256::opening_basis_range(), (3, 6));
        assert!(matches!(
            JoltOneHotK256::RING_DIMENSION_SCHEDULE_MODE,
            akita_schedules::RingDimensionScheduleMode::AdaptiveDimension { .. }
        ));

        let layout = akita_types::OpeningClaimsLayout::new(39, 1).unwrap();
        let row = JoltOneHotK256::resolve_catalog_row_for_opening(&layout).unwrap();
        let schedule = row.schedule();
        let commitment = &schedule.root.params.final_group.commitment;
        assert!([64, 128, 256].contains(&commitment.inner_commit_matrix.ring_dimension()));
        assert!([64, 128].contains(&commitment.outer_commit_matrix.ring_dimension()));
    }
}
