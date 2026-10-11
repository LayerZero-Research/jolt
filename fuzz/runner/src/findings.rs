//! Deduplicated, bounded storage of findings; originals are never deleted.

use crate::libfuzzer::sha1_hex;
use crate::runner::job::Phase;
use crate::store::{now, read_json, write_json};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

const MAX_SAMPLES: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Timeout,
    Oom,
    Leak,
    Asan,
    Panic,
    Signal,
    Crash,
    /// A fuzzing job that stopped making progress after initialization
    /// without libFuzzer reporting a timeout.
    Hang,
}

impl Display for FindingKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self {
            FindingKind::Timeout => "timeout",
            FindingKind::Oom => "oom",
            FindingKind::Leak => "leak",
            FindingKind::Asan => "asan",
            FindingKind::Panic => "panic",
            FindingKind::Signal => "signal",
            FindingKind::Crash => "crash",
            FindingKind::Hang => "hang",
        })
    }
}

/// Where a finding was first seen.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Context {
    pub host: String,
    pub campaign_id: String,
    pub build_id: String,
    pub phase: Phase,
}

/// A stored copy of the input that produced a sample.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredInput {
    /// File name inside the finding directory.
    pub input: String,
    pub sha1: String,
    /// libFuzzer's artifact file name.
    pub artifact: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub at: u64,
    pub lane: String,
    pub report: String,
    #[serde(flatten)]
    pub stored: Option<StoredInput>,
}

/// `findings/<id>/meta.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FindingMeta {
    pub id: String,
    pub signature: String,
    pub kind: FindingKind,
    pub target: String,
    pub lanes: Vec<String>,
    pub first_seen: u64,
    pub last_seen: u64,
    pub count: u64,
    pub samples: Vec<Sample>,
    /// The fresh-process replay verdict; `None` until a replay completes.
    pub reproducible: Option<String>,
    #[serde(flatten)]
    pub context: Context,
}

impl FindingMeta {
    /// Stored input file names, oldest first.
    pub fn inputs(&self) -> impl Iterator<Item = &str> {
        self.samples
            .iter()
            .filter_map(|sample| sample.stored.as_ref())
            .map(|stored| stored.input.as_str())
    }
}

pub struct Occurrence<'a> {
    pub id: &'a str,
    pub signature: &'a str,
    pub kind: FindingKind,
    pub lane: &'a str,
    pub target: &'a str,
    pub artifact: Option<&'a Path>,
    pub report: &'a [String],
    pub context: Context,
}

/// Store one occurrence; returns `(meta, is_new_signature)`.
pub fn record(findings: &Path, occurrence: Occurrence<'_>) -> IoResult<(FindingMeta, bool)> {
    let directory = findings.join(occurrence.id);
    fs::create_dir_all(&directory)?;
    let meta_path = directory.join("meta.json");
    let existing: Option<FindingMeta> = read_json(&meta_path);
    let new = existing.is_none();
    let time = now();
    let mut meta = existing.unwrap_or_else(|| FindingMeta {
        id: occurrence.id.to_string(),
        signature: occurrence.signature.to_string(),
        kind: occurrence.kind,
        target: occurrence.target.to_string(),
        lanes: Vec::new(),
        first_seen: time,
        last_seen: time,
        count: 0,
        samples: Vec::new(),
        reproducible: None,
        context: occurrence.context,
    });
    meta.count += 1;
    meta.last_seen = time;
    if !meta.lanes.iter().any(|lane| lane == occurrence.lane) {
        meta.lanes.push(occurrence.lane.to_string());
    }
    if meta.samples.len() < MAX_SAMPLES {
        let index = meta.samples.len();
        let report = format!("sample-{index}.txt");
        let stored = match occurrence.artifact.filter(|path| path.is_file()) {
            Some(artifact) => {
                let data = fs::read(artifact)?;
                let input = format!("sample-{index}.input");
                fs::write(directory.join(&input), &data)?;
                Some(StoredInput {
                    input,
                    sha1: sha1_hex(&data),
                    artifact: artifact
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(str::to_string),
                })
            }
            None => None,
        };
        fs::write(directory.join(&report), occurrence.report.join("\n") + "\n")?;
        meta.samples.push(Sample {
            at: time,
            lane: occurrence.lane.to_string(),
            report,
            stored,
        });
    }
    write_json(&meta_path, &meta)?;
    Ok((meta, new))
}

/// Move a crashing input out of the corpus so restarts do not loop on it.
/// libFuzzer names corpus files by the SHA-1 of their contents.
pub fn quarantine_corpus_copy(
    artifact: Option<&Path>,
    corpus: &Path,
    quarantine: &Path,
) -> Option<PathBuf> {
    let data = fs::read(artifact?).ok()?;
    let digest = sha1_hex(&data);
    let candidate = corpus.join(&digest);
    if !candidate.is_file() {
        return None;
    }
    fs::create_dir_all(quarantine).ok()?;
    let destination = quarantine.join(&digest);
    fs::rename(&candidate, &destination).ok()?;
    Some(destination)
}

/// Quarantine a failing input that has no corpus copy to move, so seed
/// staging still skips it.
pub fn quarantine_artifact(artifact: Option<&Path>, quarantine: &Path) -> Option<PathBuf> {
    let data = fs::read(artifact?).ok()?;
    fs::create_dir_all(quarantine).ok()?;
    let destination = quarantine.join(sha1_hex(&data));
    fs::write(&destination, data).ok()?;
    Some(destination)
}

pub fn summarize(findings: &Path) -> Vec<FindingMeta> {
    let mut out: Vec<FindingMeta> = fs::read_dir(findings)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| read_json(&entry.path().join("meta.json")))
                .collect()
        })
        .unwrap_or_default();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("jolt-fuzz-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn records_bounded_samples_and_counts_all() {
        let root = scratch("findings");
        let artifact = root.join("crash-1");
        fs::write(&artifact, b"input").unwrap();
        for index in 0..(MAX_SAMPLES + 3) {
            let (meta, new) = record(
                &root.join("findings"),
                Occurrence {
                    id: "panic-planning-1",
                    signature: "sig",
                    kind: FindingKind::Panic,
                    lane: "planning",
                    target: "planning",
                    artifact: Some(&artifact),
                    report: &["line".to_string()],
                    context: Context {
                        host: "h".into(),
                        campaign_id: "c".into(),
                        build_id: "b".into(),
                        phase: Phase::Fuzz,
                    },
                },
            )
            .unwrap();
            assert_eq!(new, index == 0);
            assert_eq!(meta.count, index as u64 + 1);
        }
        let meta: FindingMeta =
            read_json(&root.join("findings/panic-planning-1/meta.json")).unwrap();
        assert_eq!(meta.samples.len(), MAX_SAMPLES);
        assert_eq!(meta.context.host, "h");
        assert_eq!(
            fs::read(root.join("findings/panic-planning-1/sample-0.input")).unwrap(),
            b"input"
        );
    }

    #[test]
    fn quarantines_the_corpus_copy_by_content_hash() {
        let root = scratch("quarantine");
        let corpus = root.join("corpus");
        fs::create_dir_all(&corpus).unwrap();
        fs::write(corpus.join(sha1_hex(b"boom")), b"boom").unwrap();
        let artifact = root.join("crash-2");
        fs::write(&artifact, b"boom").unwrap();
        let moved = quarantine_corpus_copy(Some(&artifact), &corpus, &root.join("q")).unwrap();
        assert!(moved.is_file());
        assert!(!corpus.join(sha1_hex(b"boom")).exists());
        assert!(quarantine_corpus_copy(Some(&artifact), &corpus, &root.join("q")).is_none());
    }
}
