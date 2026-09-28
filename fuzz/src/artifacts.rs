//! The schedule artifacts every target shares, loaded once per process the
//! way application preprocessing does (`AkitaScheduleArtifacts::from_directory`).

use std::sync::{Arc, OnceLock};

use jolt_akita::AkitaScheduleArtifacts;

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
