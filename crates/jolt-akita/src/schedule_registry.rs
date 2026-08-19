//! Preprocessing-provisioned grouped schedule rows.
//!
//! A grouped row is keyed on the frozen profile of every precommitted group, so
//! an advice batch row cannot be emitted offline: each precommit layout follows
//! the program's `max_{un,}trusted_advice_size`, which is only known at
//! preprocessing. The checked-in catalogs therefore cover scalar rows and the
//! fixture grouped rows, and the rows for a program's actual advice capacities
//! are planned once here, at preprocessing, and installed for the rest of the
//! process.
//!
//! Preprocessing owns the resulting [`RegisteredRows`] and carries it for the
//! life of the proving session. Because the
//! [`CommitmentConfig`](akita_config::CommitmentConfig) resolution hooks in
//! [`crate::configs`] are associated functions with no receiver, they cannot
//! read a preprocessing object directly; provisioning therefore also publishes
//! the rows into a process-wide union that the hooks consult. Rows are
//! addressed by exact row digest and exact committed profiles, so several
//! programs with different advice capacities can be live at once without
//! aliasing.
//!
//! A hook miss falls through to the checked-in catalog and never to the
//! planner, so planner DP stays off the prove/verify path: it runs only in
//! [`provision`], which preprocessing calls before any commit.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use akita_config::{policy_of, CommitmentConfig, ResolvedScheduleRow};
use akita_pcs::AkitaError;
use akita_types::sis::HonestFoldPolicySpec;
use akita_types::{
    schedule_row_digest, AkitaScheduleLookupKey, CommittedGroupBatchProfile, CommittedGroupProfile,
    FoldSchedule, OpeningScheduleSelection, PolynomialGroupLayout, ScheduleRowDigest,
};

/// Upper bound on rows one config may hold.
///
/// The honest sweep is one row per reachable final arity per advice presence
/// combination: the widest family declares 32 arities (`K256_NUM_VARS`) and two
/// advice objects of differing capacity reach 3 combinations, so 96 is the
/// honest ceiling. This leaves headroom above it as a backstop against a caller
/// installing an unbounded set.
pub const MAX_REGISTERED_ROWS: usize = 128;

/// One config's provisioned rows, addressable by the two keys the resolution
/// hooks use: the public row digest (verifier boundary) and the lookup key or
/// exact profiles (honest-prover boundary).
#[derive(Clone, Debug, Default)]
pub struct RegisteredRows {
    by_digest: HashMap<ScheduleRowDigest, ResolvedScheduleRow>,
}

impl RegisteredRows {
    pub fn is_empty(&self) -> bool {
        self.by_digest.is_empty()
    }

    pub fn len(&self) -> usize {
        self.by_digest.len()
    }

    /// The installed rows, in unspecified order.
    pub fn rows(&self) -> impl Iterator<Item = &ResolvedScheduleRow> {
        self.by_digest.values()
    }

    fn insert(&mut self, row: ResolvedScheduleRow) -> Result<(), AkitaError> {
        let digest = row.selection().row_digest;
        if self.by_digest.contains_key(&digest) {
            return Err(AkitaError::InvalidSetup(
                "duplicate schedule row digest in provisioned rows".to_owned(),
            ));
        }
        let _ = self.by_digest.insert(digest, row);
        Ok(())
    }

    fn by_selection(&self, selection: OpeningScheduleSelection) -> Option<&ResolvedScheduleRow> {
        self.by_digest.get(&selection.row_digest)
    }

    fn by_key(&self, key: &AkitaScheduleLookupKey) -> Option<&ResolvedScheduleRow> {
        self.by_digest.values().find(|row| {
            let profiles = row.profiles();
            profiles.final_group.group == key.final_group
                && profiles.precommitteds == key.precommitteds
        })
    }

    fn by_profiles(&self, profiles: &CommittedGroupBatchProfile) -> Option<&ResolvedScheduleRow> {
        self.by_digest
            .values()
            .find(|row| row.profiles() == profiles)
    }

    /// Stable digest over the installed rows, order-independent. Prover and
    /// verifier compare this to prove they provisioned the same set before the
    /// grouped statement is bound.
    pub fn set_digest(&self) -> [u8; 32] {
        let mut digests: Vec<[u8; 32]> = self
            .by_digest
            .keys()
            .map(|digest| *digest.as_bytes())
            .collect();
        digests.sort_unstable();
        let mut bytes = Vec::with_capacity(40 + 32 * digests.len());
        bytes.extend_from_slice(b"jolt-akita/provisioned-rows/v1");
        bytes.extend_from_slice(&(digests.len() as u64).to_le_bytes());
        for digest in &digests {
            bytes.extend_from_slice(digest);
        }
        akita_types::instance_descriptor::digest_descriptor_bytes(&bytes)
    }
}

type Registry = RwLock<HashMap<TypeId, RegisteredRows>>;

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

fn poisoned() -> AkitaError {
    AkitaError::InvalidSetup("schedule registry lock poisoned".to_owned())
}

/// The rows installed for `Cfg`, or an empty set.
pub fn registered_rows<Cfg: CommitmentConfig + 'static>() -> Result<RegisteredRows, AkitaError> {
    let guard = registry().read().map_err(|_| poisoned())?;
    Ok(guard.get(&TypeId::of::<Cfg>()).cloned().unwrap_or_default())
}

/// Resolve one public selection against `Cfg`'s installed rows.
///
/// This is the verifier boundary: digest lookup only, never a planner call.
pub fn lookup_selection<Cfg: CommitmentConfig + 'static>(
    selection: OpeningScheduleSelection,
) -> Option<ResolvedScheduleRow> {
    let guard = registry().read().ok()?;
    guard
        .get(&TypeId::of::<Cfg>())?
        .by_selection(selection)
        .cloned()
}

/// Resolve one lookup key against `Cfg`'s installed rows.
pub fn lookup_key<Cfg: CommitmentConfig + 'static>(
    key: &AkitaScheduleLookupKey,
) -> Option<ResolvedScheduleRow> {
    let guard = registry().read().ok()?;
    guard.get(&TypeId::of::<Cfg>())?.by_key(key).cloned()
}

/// Resolve one exact committed-profile batch against `Cfg`'s installed rows.
pub fn lookup_profiles<Cfg: CommitmentConfig + 'static>(
    profiles: &CommittedGroupBatchProfile,
) -> Option<ResolvedScheduleRow> {
    let guard = registry().read().ok()?;
    guard
        .get(&TypeId::of::<Cfg>())?
        .by_profiles(profiles)
        .cloned()
}

/// Resolve `key` against the checked-in catalog alone.
///
/// `Cfg::resolve_catalog_row_for_key` consults the registry first, so it cannot
/// answer "does the catalog already cover this?" once anything is installed —
/// it would report a row this module itself planned, making provisioning
/// non-idempotent.
pub(crate) fn catalog_only_row<Cfg: CommitmentConfig>(
    key: &AkitaScheduleLookupKey,
) -> Result<ResolvedScheduleRow, AkitaError> {
    akita_schedules::resolve_generated_catalog_row_for_key(
        key,
        &policy_of::<Cfg>(),
        Cfg::ring_challenge_config,
        Cfg::schedule_catalog(),
    )
}

/// Plan one grouped row and wrap it as a [`ResolvedScheduleRow`].
///
/// Mirrors the offline emitter's grouped solve
/// (`crate::schedules::emit::regen_group_batch`): the same planner, policy, and
/// per-precommit honest-fold policies, so a row planned here is the row the
/// emitter would have written for the same key.
fn plan_row<Cfg: CommitmentConfig>(
    key: &AkitaScheduleLookupKey,
    precommitted_honest_fold_policies: &[HonestFoldPolicySpec],
) -> Result<ResolvedScheduleRow, AkitaError> {
    let policy = policy_of::<Cfg>();
    let schedule = akita_planner::find_schedule(
        key,
        Cfg::root_honest_fold_policy(),
        precommitted_honest_fold_policies,
        &policy,
        Cfg::ring_challenge_config,
    )?
    .schedule;
    schedule.validate_structure()?;
    reject_setup_prefix_contributions(&schedule)?;

    let profiles = CommittedGroupBatchProfile {
        final_group: CommittedGroupProfile::try_from_params(
            key.final_group,
            &schedule.root.params.final_group.commitment,
        )?,
        precommitteds: key.precommitteds.clone(),
    };
    let selection = OpeningScheduleSelection {
        row_digest: schedule_row_digest(&profiles, &schedule)?,
    };
    ResolvedScheduleRow::try_new(selection, profiles, schedule, &policy)
}

/// Jolt's shape guard admits only direct schedules: a level carrying an
/// incoming setup prefix is rejected there, after the row is already bound into
/// the statement. Catch it at provisioning instead, where the error is
/// actionable.
fn reject_setup_prefix_contributions(schedule: &FoldSchedule) -> Result<(), AkitaError> {
    if schedule
        .recursive_folds
        .iter()
        .any(|fold| fold.params.incoming_setup_prefix.is_some())
    {
        return Err(AkitaError::InvalidSetup(
            "provisioned schedule carries a recursive setup-prefix contribution, which Jolt's \
             shape guard does not admit"
                .to_owned(),
        ));
    }
    Ok(())
}

/// Plan and install the grouped rows for one frozen precommit profile across
/// `final_num_vars`, then return the installed set.
///
/// A key the checked-in catalog already covers is skipped: the catalog row is
/// authoritative and resolution falls through to it. Only the remainder is
/// planned.
///
/// Installing is idempotent for an identical set and an error for a different
/// one — two advice capacities in one process would otherwise silently
/// reinterpret each other's rows.
pub fn provision<Cfg: CommitmentConfig + 'static>(
    precommitted_combinations: &[Vec<CommittedGroupProfile>],
    precommitted_honest_fold_policy: HonestFoldPolicySpec,
    final_num_vars: impl IntoIterator<Item = usize>,
) -> Result<RegisteredRows, AkitaError> {
    if precommitted_combinations.iter().any(Vec::is_empty) {
        return Err(AkitaError::InvalidSetup(
            "a grouped row must have at least one precommitted group".to_owned(),
        ));
    }
    let final_arities: Vec<usize> = final_num_vars.into_iter().collect();
    // Every reachable presence combination gets its own row at every final
    // arity: which advice objects a proof actually carries is a runtime fact,
    // so a missing combination would only surface as an unresolvable row.
    let keys: Vec<AkitaScheduleLookupKey> = precommitted_combinations
        .iter()
        .flat_map(|precommitteds| {
            final_arities
                .iter()
                .map(|num_vars| AkitaScheduleLookupKey {
                    final_group: PolynomialGroupLayout::new(*num_vars, 1),
                    precommitteds: precommitteds.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect();
    if keys.len() > MAX_REGISTERED_ROWS {
        return Err(AkitaError::InvalidSetup(format!(
            "provisioning {} rows exceeds the {MAX_REGISTERED_ROWS}-row cap",
            keys.len()
        )));
    }

    // Planner solves are memory-bound (each holds a large suffix DP cache), so
    // the worker count stays on akita's offline default rather than host-wide
    // parallelism — this runs inside a process that may already hold a trace.
    let workers = akita_planner::emit::offline_planning_worker_count(keys.len());
    let planned = akita_planner::emit::bounded_parallel_filter_map(&keys, workers, |key| {
        if catalog_only_row::<Cfg>(key).is_ok() {
            return Ok(None);
        }
        let policies = vec![precommitted_honest_fold_policy; key.precommitteds.len()];
        match plan_row::<Cfg>(key, &policies) {
            Ok(row) => Ok(Some(row)),
            // Adding a precommit raises the fold-count floor, so a final arity
            // near the bottom of a family's scalar range can admit no grouped
            // schedule at all. That arity simply has no row; a proof at it
            // fails its own shape validation. Any other planner failure is a
            // real one and propagates.
            Err(AkitaError::UnsupportedSchedule(reason)) => {
                tracing::debug!(
                    final_num_vars = key.final_group.num_vars(),
                    %reason,
                    "no grouped schedule at this final arity; skipping"
                );
                Ok(None)
            }
            Err(error) => Err(error.to_string()),
        }
    })
    .map_err(AkitaError::InvalidSetup)?;

    let mut rows = RegisteredRows::default();
    for row in planned {
        rows.insert(row)?;
    }
    publish::<Cfg>(rows)
}

/// Publish `rows` so the static resolution hooks can see them, and hand the
/// caller its own set back to own.
///
/// The hooks are `CommitmentConfig` associated functions with no receiver, so
/// they cannot read a preprocessing object; the ambient map is how a
/// preprocessing-owned row set becomes visible to them. It is a union, not a
/// single set: rows are addressed by exact row digest and exact committed
/// profiles, so two programs with different advice capacities cannot alias each
/// other and both can be live in one process.
///
/// Re-publishing an identical row is a no-op. The same digest carrying a
/// *different* schedule is a genuine collision and an error.
fn publish<Cfg: CommitmentConfig + 'static>(
    rows: RegisteredRows,
) -> Result<RegisteredRows, AkitaError> {
    let mut guard = registry().write().map_err(|_| poisoned())?;
    let ambient = guard.entry(TypeId::of::<Cfg>()).or_default();
    for (digest, row) in &rows.by_digest {
        match ambient.by_digest.get(digest) {
            Some(existing) if existing.profiles() != row.profiles() => {
                return Err(AkitaError::InvalidSetup(
                    "schedule row digest collision: same identity, different profiles".to_owned(),
                ));
            }
            Some(_) => {}
            None => {
                let _ = ambient.by_digest.insert(*digest, row.clone());
            }
        }
    }
    Ok(rows)
}

/// The frozen profile the independent trusted-advice commit will produce for a
/// dense group of `layout`.
///
/// Read from the checked-in dense catalog, not planned: the catalog row is what
/// the commit actually resolves, so it — not a fresh solve — is the value a
/// grouped row must be keyed on. The offline emitter deliberately does the
/// opposite (`crate::schedules::emit`, which must not read the catalog it is
/// regenerating); `runtime_and_generated_dense_profiles_agree` pins the two
/// together.
pub fn dense_precommit_profile(
    layout: PolynomialGroupLayout,
) -> Result<CommittedGroupProfile, AkitaError> {
    crate::configs::JoltDense::profile_without_precommitted_groups(layout)
}

/// The advice layouts a program can precommit, in canonical public batch
/// order. `None` means that advice kind cannot appear in any proof for this
/// program, so no row is planned for it.
///
/// Both layouts are fixed by the program's advice capacities, which are known at
/// preprocessing even though whether a given proof carries each object is only
/// decided at proving time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdvicePrecommitLayouts {
    pub untrusted: Option<PolynomialGroupLayout>,
    pub trusted: Option<PolynomialGroupLayout>,
}

impl AdvicePrecommitLayouts {
    /// Every ordered non-empty presence combination, as frozen dense profiles.
    ///
    /// A proof carries whichever advice objects its inputs actually populate, so
    /// all of `[untrusted]`, `[trusted]`, and `[untrusted, trusted]` are
    /// reachable when both capacities are nonzero.
    ///
    /// Deduplicated by profile sequence: a grouped row is keyed on the frozen
    /// profiles alone, never on which advice kind produced them (the role is
    /// bound in the statement transcript, not in the schedule). Two advice
    /// objects of equal arity therefore share one single-precommit row, and
    /// emitting it twice would be a duplicate row identity.
    fn precommit_combinations(self) -> Result<Vec<Vec<CommittedGroupProfile>>, AkitaError> {
        let untrusted = self.untrusted.map(dense_precommit_profile).transpose()?;
        let trusted = self.trusted.map(dense_precommit_profile).transpose()?;
        let mut combinations: Vec<Vec<CommittedGroupProfile>> = Vec::with_capacity(3);
        let mut push_unique = |combination: Vec<CommittedGroupProfile>| {
            if !combinations.contains(&combination) {
                combinations.push(combination);
            }
        };
        if let Some(untrusted) = untrusted {
            push_unique(vec![untrusted]);
        }
        if let Some(trusted) = trusted {
            push_unique(vec![trusted]);
        }
        if let (Some(untrusted), Some(trusted)) = (untrusted, trusted) {
            push_unique(vec![untrusted, trusted]);
        }
        Ok(combinations)
    }
}

/// Plan and install the grouped rows batching the program's advice
/// precommits with every final `OneHotTrace` arity in `final_num_vars`.
///
/// The advice layouts are fixed by the program's advice capacities, so this runs
/// once at preprocessing; the trace length is not known then, hence the sweep
/// over the family's whole arity range.
pub fn provision_advice<Cfg: CommitmentConfig + 'static>(
    layouts: AdvicePrecommitLayouts,
    final_num_vars: impl IntoIterator<Item = usize>,
) -> Result<RegisteredRows, AkitaError> {
    let combinations = layouts.precommit_combinations()?;
    if combinations.is_empty() {
        return Ok(RegisteredRows::default());
    }
    provision::<Cfg>(
        &combinations,
        crate::configs::JoltDense::root_honest_fold_policy(),
        final_num_vars,
    )
}

/// Provision the grouped advice rows for the `OneHotTrace` family selected by
/// `one_hot_k`, over that family's whole reachable final-arity range.
///
/// Preprocessing calls this once, before building the packed setup: setup
/// sizing folds in the provisioned rows, so a row installed later would not be
/// covered by the setup matrices.
///
/// Each physical arity is the advice object's own — the caller derives it
/// from the matching `max_*_advice_size` through the same packing plan the commit
/// uses, so this crate stays free of the advice layout.
pub fn provision_advice_for_k(
    untrusted_physical_vars: Option<usize>,
    trusted_physical_vars: Option<usize>,
    one_hot_k: usize,
    max_final_num_vars: usize,
) -> Result<RegisteredRows, AkitaError> {
    let layouts = AdvicePrecommitLayouts {
        untrusted: untrusted_physical_vars.map(|vars| PolynomialGroupLayout::new(vars, 1)),
        trusted: trusted_physical_vars.map(|vars| PolynomialGroupLayout::new(vars, 1)),
    };
    // The family's declared range is what the catalog covers; `max_final_num_vars`
    // is what this program can actually reach, from its padded trace-length
    // bound. Sweeping past it plans rows no proof of this program can select.
    let (min, declared_max) = match one_hot_k {
        crate::AKITA_ONE_HOT_K256 => crate::schedules::emit::K256_NUM_VARS,
        crate::AKITA_ONE_HOT_K16 => crate::schedules::emit::K16_NUM_VARS,
        other => {
            return Err(AkitaError::InvalidSetup(format!(
                "unsupported one-hot K {other} for grouped advice provisioning"
            )))
        }
    };
    let max = declared_max.min(max_final_num_vars);
    if max < min {
        return Ok(RegisteredRows::default());
    }
    match one_hot_k {
        crate::AKITA_ONE_HOT_K256 => {
            provision_advice::<crate::configs::JoltOneHotK256>(layouts, min..=max)
        }
        crate::AKITA_ONE_HOT_K16 => {
            provision_advice::<crate::configs::JoltOneHotK16>(layouts, min..=max)
        }
        _ => unreachable!("one-hot K was validated above"),
    }
}

/// Drop every installed row. Tests only: production installs once per process.
#[cfg(test)]
pub fn reset_for_tests() {
    if let Ok(mut guard) = registry().write() {
        guard.clear();
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests"
)]
mod tests {
    use super::*;
    use crate::configs::{JoltDense, JoltOneHotK256};
    use crate::schedules::emit;

    /// Runtime reads the dense catalog; the emitter runs a fresh planner solve.
    /// A divergence would key grouped rows on a prefix no commit can produce.
    #[test]
    fn runtime_and_generated_dense_profiles_agree() {
        for physical_vars in emit::DENSE_NUM_VARS.0..=emit::DENSE_NUM_VARS.1 {
            let layout = PolynomialGroupLayout::new(physical_vars, 1);
            let from_catalog = dense_precommit_profile(layout).unwrap_or_else(|error| {
                panic!("dense catalog must cover {physical_vars} physical vars: {error}")
            });
            let from_planner =
                emit::planned_profile_without_precommitted_groups::<JoltDense>(layout)
                    .unwrap_or_else(|error| {
                        panic!("planner must solve {physical_vars} vars: {error}")
                    });
            assert_eq!(
                from_catalog, from_planner,
                "dense profile for {physical_vars} vars diverges between catalog and planner"
            );
        }
    }

    #[test]
    fn dense_catalog_covers_every_legal_advice_arity() {
        for physical_vars in emit::DENSE_NUM_VARS.0..=emit::DENSE_NUM_VARS.1 {
            assert!(
                dense_precommit_profile(PolynomialGroupLayout::new(physical_vars, 1)).is_ok(),
                "dense catalog must cover {physical_vars} physical vars"
            );
        }
    }

    #[test]
    fn provisioning_more_rows_than_the_cap_is_rejected() {
        let profile =
            dense_precommit_profile(PolynomialGroupLayout::new(emit::DENSE_NUM_VARS.0, 1)).unwrap();
        let error = provision::<JoltOneHotK256>(
            &[vec![profile]],
            JoltDense::root_honest_fold_policy(),
            0..=MAX_REGISTERED_ROWS,
        )
        .expect_err("exceeding the row cap must be rejected");
        assert!(
            format!("{error}").contains("cap"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn empty_registry_resolves_nothing() {
        let selection = OpeningScheduleSelection::default();
        assert!(lookup_selection::<JoltOneHotK256>(selection).is_none());
    }

    /// The fixture family's arities are small, so it is the one family whose
    /// grouped rows are cheap enough to plan end to end inside a test.
    mod fixture {
        use super::*;
        use crate::configs::JoltOneHotK16;
        use crate::schedules::emit::{FIXTURE_K16_FINAL_NUM_VARS, FIXTURE_TRUSTED_ADVICE_GROUP};

        fn trusted_only(trusted: PolynomialGroupLayout) -> AdvicePrecommitLayouts {
            AdvicePrecommitLayouts {
                untrusted: None,
                trusted: Some(trusted),
            }
        }

        fn fixture_keys(
            final_num_vars: impl IntoIterator<Item = usize>,
        ) -> (CommittedGroupProfile, Vec<AkitaScheduleLookupKey>) {
            let profile = dense_precommit_profile(FIXTURE_TRUSTED_ADVICE_GROUP).unwrap();
            let keys = final_num_vars
                .into_iter()
                .map(|num_vars| AkitaScheduleLookupKey {
                    final_group: PolynomialGroupLayout::new(num_vars, 1),
                    precommitteds: vec![profile],
                })
                .collect();
            (profile, keys)
        }

        /// No grouped advice row is checked in, so preprocessing plans every
        /// one. A cataloged grouped row would split one behavior across two
        /// sources: a table for the guessed capacities, the planner for the rest.
        #[test]
        fn no_grouped_advice_row_is_cataloged_and_provisioning_plans_them_all() {
            reset_for_tests();
            let (_, keys) =
                fixture_keys(FIXTURE_K16_FINAL_NUM_VARS.0..=FIXTURE_K16_FINAL_NUM_VARS.1);
            for key in &keys {
                assert!(
                    catalog_only_row::<JoltOneHotK16>(key).is_err(),
                    "a grouped advice row must never be checked in: {key:?}"
                );
            }

            let rows = provision_advice::<JoltOneHotK16>(
                trusted_only(FIXTURE_TRUSTED_ADVICE_GROUP),
                FIXTURE_K16_FINAL_NUM_VARS.0..=FIXTURE_K16_FINAL_NUM_VARS.1,
            )
            .expect("provisioning must plan the whole range");
            assert_eq!(
                rows.len(),
                keys.len(),
                "every final arity must be planned, since none is cataloged"
            );
            for key in &keys {
                let resolved = JoltOneHotK16::resolve_catalog_row_for_key(key)
                    .expect("a provisioned row must resolve through the hook");
                assert_eq!(resolved.profiles().precommitteds, key.precommitteds);
            }
            reset_for_tests();
        }

        /// Final arity 27 is outside the fixture catalog's 22..=26 range, so it
        /// exercises the planned-row path end to end.
        #[test]
        fn a_provisioned_row_resolves_through_every_hook() {
            reset_for_tests();
            let (profile, keys) = fixture_keys([27]);
            let key = keys.first().expect("one key");

            assert!(
                JoltOneHotK16::resolve_catalog_row_for_key(key).is_err(),
                "arity 27 must not be cataloged, or this test proves nothing"
            );

            let rows = provision::<JoltOneHotK16>(
                &[vec![profile]],
                JoltDense::root_honest_fold_policy(),
                [27],
            )
            .expect("provisioning an uncataloged key must plan it");
            assert_eq!(rows.len(), 1);

            let by_key = JoltOneHotK16::resolve_catalog_row_for_key(key)
                .expect("the provisioned row must now resolve by key");
            let by_profiles = JoltOneHotK16::resolve_catalog_row_for_profiles(by_key.profiles())
                .expect("the provisioned row must resolve by exact profiles");
            let by_selection = JoltOneHotK16::resolve_schedule_selection(by_key.selection())
                .expect("the provisioned row must resolve by public selection");

            assert_eq!(by_key.selection(), by_profiles.selection());
            assert_eq!(by_key.selection(), by_selection.selection());
            reset_for_tests();
        }

        /// The capability this module exists for: a program whose trusted
        /// advice capacity has no checked-in grouped row still gets a complete,
        /// resolvable row set. Every final arity is uncataloged at this prefix,
        /// so the whole range is planned.
        #[test]
        fn a_new_advice_capacity_provisions_and_resolves_every_final_arity() {
            reset_for_tests();
            let uncataloged_trusted =
                PolynomialGroupLayout::new(FIXTURE_TRUSTED_ADVICE_GROUP.num_vars() + 1, 1);
            let range = FIXTURE_K16_FINAL_NUM_VARS.0..=FIXTURE_K16_FINAL_NUM_VARS.1;
            let expected = range.clone().count();

            let rows =
                provision_advice::<JoltOneHotK16>(trusted_only(uncataloged_trusted), range.clone())
                    .expect("a previously unseen advice capacity must provision");
            assert_eq!(
                rows.len(),
                expected,
                "every final arity at an uncataloged prefix must be planned"
            );

            let profile = dense_precommit_profile(uncataloged_trusted).unwrap();
            for num_vars in range {
                let key = AkitaScheduleLookupKey {
                    final_group: PolynomialGroupLayout::new(num_vars, 1),
                    precommitteds: vec![profile],
                };
                let row = JoltOneHotK16::resolve_catalog_row_for_key(&key)
                    .unwrap_or_else(|error| panic!("arity {num_vars} must resolve: {error}"));
                // The verifier only ever sees the public selection.
                let by_selection = JoltOneHotK16::resolve_schedule_selection(row.selection())
                    .unwrap_or_else(|error| {
                        panic!("arity {num_vars} must resolve by selection: {error}")
                    });
                assert_eq!(by_selection.profiles(), row.profiles());
            }
            reset_for_tests();
        }

        #[test]
        fn republishing_an_identical_row_set_is_a_no_op() {
            reset_for_tests();
            let (profile, _) = fixture_keys([27]);
            let combinations = [vec![profile]];
            let policy = JoltDense::root_honest_fold_policy();
            let first = provision::<JoltOneHotK16>(&combinations, policy, [27])
                .expect("first provisioning");
            let second = provision::<JoltOneHotK16>(&combinations, policy, [27])
                .expect("re-provisioning an identical set must succeed");
            assert_eq!(first.set_digest(), second.set_digest());
            reset_for_tests();
        }

        /// Two programs with different trusted-advice capacities are live in one
        /// process. Their rows carry distinct digests and distinct prefix
        /// profiles, so neither can resolve the other's row.
        #[test]
        fn two_advice_capacities_coexist_without_aliasing() {
            reset_for_tests();
            let small = FIXTURE_TRUSTED_ADVICE_GROUP;
            let large = PolynomialGroupLayout::new(small.num_vars() + 1, 1);

            let small_rows =
                provision_advice::<JoltOneHotK16>(trusted_only(small), [27]).expect("small");
            let large_rows =
                provision_advice::<JoltOneHotK16>(trusted_only(large), [27]).expect("large");
            assert_ne!(
                small_rows.set_digest(),
                large_rows.set_digest(),
                "different capacities must produce different row sets"
            );

            for (layout, own) in [(small, &small_rows), (large, &large_rows)] {
                let profile = dense_precommit_profile(layout).unwrap();
                let key = AkitaScheduleLookupKey {
                    final_group: PolynomialGroupLayout::new(27, 1),
                    precommitteds: vec![profile],
                };
                let resolved = JoltOneHotK16::resolve_catalog_row_for_key(&key)
                    .unwrap_or_else(|error| panic!("{layout:?} must still resolve: {error}"));
                assert_eq!(
                    resolved.profiles().precommitteds,
                    vec![profile],
                    "resolved row must carry its own prefix, not the other capacity's"
                );
                assert!(
                    own.rows()
                        .any(|row| row.selection() == resolved.selection()),
                    "each capacity must resolve to a row from its own set"
                );
            }
            reset_for_tests();
        }

        #[test]
        fn a_digest_collision_with_different_profiles_is_rejected() {
            reset_for_tests();
            let (profile, _) = fixture_keys([27]);
            let mut rows = provision::<JoltOneHotK16>(
                &[vec![profile]],
                JoltDense::root_honest_fold_policy(),
                [27],
            )
            .expect("provisioning must succeed");

            // Re-key a genuinely different row under an already-published
            // digest: the identity says "same row", the profiles say otherwise.
            let other = plan_row::<JoltOneHotK16>(
                &AkitaScheduleLookupKey {
                    final_group: PolynomialGroupLayout::new(28, 1),
                    precommitteds: vec![profile],
                },
                &[JoltDense::root_honest_fold_policy()],
            )
            .expect("plan a second row");
            let stolen_digest = rows.by_digest.keys().next().copied().expect("one row");
            rows.by_digest.clear();
            let _ = rows.by_digest.insert(stolen_digest, other);

            let error =
                publish::<JoltOneHotK16>(rows).expect_err("a digest collision must be rejected");
            assert!(
                format!("{error}").contains("collision"),
                "unexpected error: {error}"
            );
            reset_for_tests();
        }

        #[test]
        fn setup_capacity_covers_a_provisioned_row() {
            reset_for_tests();
            let (profile, _) = fixture_keys([27]);
            let baseline = JoltOneHotK16::setup_matrix_capacity(27, 2)
                .expect("baseline capacity")
                .num_field_elements;
            let _ = provision::<JoltOneHotK16>(
                &[vec![profile]],
                JoltDense::root_honest_fold_policy(),
                [27],
            )
            .expect("provisioning must succeed");
            let provisioned = JoltOneHotK16::setup_matrix_capacity(27, 2)
                .expect("capacity after provisioning")
                .num_field_elements;
            assert!(
                provisioned >= baseline,
                "provisioned capacity {provisioned} must cover the baseline {baseline}"
            );
            reset_for_tests();
        }
    }
}
