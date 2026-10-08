//! The schedule artifacts every target shares, loaded once per process the
//! way application preprocessing does (`AkitaScheduleArtifacts::from_directory`).

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

use akita_schedules::ValidatedScheduleCatalog;
use jolt_akita::{AkitaChunkProfile, AkitaScheduleArtifacts};

pub fn shared() -> Arc<AkitaScheduleArtifacts> {
    static ARTIFACTS: OnceLock<Arc<AkitaScheduleArtifacts>> = OnceLock::new();
    Arc::clone(ARTIFACTS.get_or_init(|| {
        let dir = crate::env::artifacts_dir();
        Arc::new(
            AkitaScheduleArtifacts::from_directory(&dir).unwrap_or_else(|error| {
                panic!("load schedule artifacts from {}: {error}", dir.display())
            }),
        )
    }))
}

/// The validated catalogs grouped-row provisioning reads for one trace family.
pub struct Catalogs {
    pub dense: ValidatedScheduleCatalog,
    pub full_dense: ValidatedScheduleCatalog,
    pub one_hot: ValidatedScheduleCatalog,
    /// `(num_vars, num_polys)` of every final group `one_hot` lists.
    pub one_hot_rows: BTreeSet<(usize, usize)>,
}

/// [`Catalogs`] for `(k, profile)`, parsed once per process.
pub fn catalogs(k: usize, profile: AkitaChunkProfile) -> Arc<Catalogs> {
    type Cache = Mutex<HashMap<(usize, AkitaChunkProfile), Arc<Catalogs>>>;
    static CACHE: OnceLock<Cache> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(Cache::default)
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Arc::clone(cache.entry((k, profile)).or_insert_with(|| {
        let artifacts = shared();
        let load = |result: Result<ValidatedScheduleCatalog, _>, name: &str| {
            result.unwrap_or_else(|error| panic!("load {name} catalog: {error}"))
        };
        let one_hot = load(
            artifacts.one_hot_catalog_for_profile(k, profile),
            &format!("K={k} {profile:?}"),
        );
        let one_hot_rows = one_hot
            .rows()
            .map(|row| {
                let group = row.profiles().final_group.group;
                (group.num_vars(), group.num_polynomials())
            })
            .collect();
        Arc::new(Catalogs {
            dense: load(artifacts.dense_catalog(), "dense"),
            full_dense: load(artifacts.full_dense_catalog(), "full dense"),
            one_hot,
            one_hot_rows,
        })
    }))
}
