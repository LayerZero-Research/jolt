//! Job kinds and the libFuzzer process each one runs.

use super::{Message, Runner, LOG_BYTES, STARTUP_GRACE_S, TAIL_LINES};
use crate::libfuzzer::{self, Status, BINARY};
use crate::registry::Lane;
use crate::store::{now, RotatingLog};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{BufRead, BufReader, Result as IoResult};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Memory a hard-limited worker may use beyond its libFuzzer `-rss_limit_mb`,
/// so libFuzzer reports an out-of-memory input before the cgroup kills it.
const HARD_MEMORY_HEADROOM_MB: u64 = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Fuzz,
    Baseline,
    Replay,
    Merge,
}

impl Display for Phase {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self {
            Phase::Fuzz => "fuzz",
            Phase::Baseline => "baseline",
            Phase::Replay => "replay",
            Phase::Merge => "merge",
        })
    }
}

/// A finding's stored input, executed once in a fresh process.
pub struct Replay {
    pub finding_id: String,
    pub input: PathBuf,
}

/// One step of a target's corpus minimization: libFuzzer `-merge=1` of
/// `work/snapshot/` into `work/merged/` under one variant's environment.
/// Every variant runs a step into the same `merged/`, which keeps each input
/// that adds coverage under any of them.
#[derive(Clone)]
pub struct Merge {
    pub work: PathBuf,
    /// Corpus file names the snapshot holds.
    pub snapshot: Vec<String>,
    /// Variants still to merge after this step's.
    pub remaining: Vec<Lane>,
    pub started: Instant,
}

pub enum JobKind {
    Fuzz,
    /// Every non-quarantined shipped seed once (`-runs=0`).
    Baseline,
    Replay(Replay),
    Merge(Merge),
}

impl JobKind {
    pub fn phase(&self) -> Phase {
        match self {
            JobKind::Fuzz => Phase::Fuzz,
            JobKind::Baseline => Phase::Baseline,
            JobKind::Replay(_) => Phase::Replay,
            JobKind::Merge(_) => Phase::Merge,
        }
    }
}

/// Why and when the runner signalled a job to stop.
pub enum Stop {
    /// The job outlasted its wall-clock deadline.
    Watchdog(Instant),
    /// The campaign is shutting down.
    Shutdown(Instant),
}

impl Stop {
    pub fn at(&self) -> Instant {
        match self {
            Stop::Watchdog(at) | Stop::Shutdown(at) => *at,
        }
    }
}

pub struct Job {
    pub(super) lane: Lane,
    pub(super) kind: JobKind,
    pub(super) child: Child,
    pub(super) started: Instant,
    pub(super) deadline: Instant,
    log: RotatingLog,
    pub(super) artifacts: PathBuf,
    pub(super) stats_file: PathBuf,
    pub(super) status: Status,
    pub(super) tail: VecDeque<String>,
    artifact: Option<PathBuf>,
    pub(super) stop: Option<Stop>,
    pub(super) output_closed: bool,
    pub(super) exited_at: Option<Instant>,
    pub(super) inited_after: Option<u64>,
}

impl Job {
    pub(super) fn consume(&mut self, line: String) {
        self.log.line(&line);
        libfuzzer::parse_status(&line, &mut self.status);
        if self.status.inited && self.inited_after.is_none() {
            self.inited_after = Some(self.started.elapsed().as_secs());
        }
        if let Some(path) = libfuzzer::artifact_path(&line) {
            self.artifact = Some(if path.is_absolute() {
                path
            } else {
                self.artifacts.join(path)
            });
        }
        if self.tail.len() == TAIL_LINES {
            self.tail.pop_front();
        }
        self.tail.push_back(line);
    }

    pub(super) fn signal(&self, signal: libc::c_int) {
        // SAFETY: the job is its own process-group leader.
        unsafe {
            libc::killpg(self.child.id() as libc::pid_t, signal);
        }
    }

    /// The failure artifact libFuzzer reported, or one it wrote silently.
    pub(super) fn failure_artifact(&self) -> Option<PathBuf> {
        self.artifact.clone().or_else(|| {
            std::fs::read_dir(&self.artifacts)
                .ok()?
                .flatten()
                .map(|entry| entry.path())
                .find(|path| path.is_file() && libfuzzer::is_failure_artifact(path))
        })
    }
}

impl Runner {
    /// The libFuzzer command line for `kind` and its wall-clock budget in
    /// seconds, staging any inputs it reads under `artifacts`.
    fn launch(
        &self,
        lane: &Lane,
        kind: &JobKind,
        artifacts: &Path,
    ) -> IoResult<(Vec<String>, u64)> {
        let binary = self.dist.join("bin").join(BINARY);
        let init = self
            .state
            .lanes
            .get(&lane.name())
            .map_or(0, |state| state.init_seconds);
        Ok(match kind {
            JobKind::Fuzz => {
                let slice = self.slice_for(lane);
                let corpus = self.store.corpus.join(&lane.target);
                let args = libfuzzer::fuzz_args(
                    &binary,
                    lane,
                    &[&corpus],
                    artifacts,
                    slice,
                    false,
                    &self.options.extra_args,
                );
                // Corpus re-execution happens before libFuzzer honors
                // `-max_total_time`, so allow for it on top of the slice.
                (args, slice + init.saturating_mul(2) + 600)
            }
            JobKind::Baseline => {
                // The baseline checks the shipped seeds (honest statements),
                // not the accumulated corpus, whose re-execution grows with
                // every campaign hour. `-runs=0` stops at the first failing
                // seed, so quarantined seeds are left out.
                let scratch = artifacts.join("scratch-corpus");
                std::fs::create_dir_all(&scratch)?;
                let seeds = artifacts.join("seeds");
                let count = self.stage_seeds(lane, &seeds)?;
                let args = libfuzzer::fuzz_args(
                    &binary,
                    lane,
                    &[&scratch, &seeds],
                    artifacts,
                    0,
                    true,
                    &[],
                );
                // libFuzzer stops each seed at `timeout_s`, so the seed count
                // bounds a healthy baseline; a fixed budget cut off lanes
                // with many slow seeds before they finished.
                (args, count.saturating_mul(lane.timeout_s).max(3600))
            }
            JobKind::Replay(replay) => (
                libfuzzer::replay_args(
                    &binary,
                    lane,
                    artifacts,
                    std::slice::from_ref(&replay.input),
                ),
                600,
            ),
            JobKind::Merge(merge) => {
                let mut args = libfuzzer::base_args(&binary, lane, lane.timeout_s, artifacts);
                args.extend([
                    "-merge=1".to_string(),
                    format!("-max_len={}", lane.max_len),
                    merge.work.join("merged").display().to_string(),
                    merge.work.join("snapshot").display().to_string(),
                ]);
                // A merge executes the whole corpus once.
                (args, (6 * 3600u64).max(init.saturating_mul(3)))
            }
        })
    }

    #[expect(clippy::expect_used, reason = "stderr is configured as piped")]
    pub(super) fn spawn(&mut self, lane: &Lane, kind: JobKind) -> IoResult<()> {
        self.sequence += 1;
        let id = self.sequence;
        let job_name = format!("{}-{}-{id}", now(), std::process::id());
        let artifacts = self
            .store
            .root
            .join("artifacts")
            .join(lane.name())
            .join(&job_name);
        std::fs::create_dir_all(&artifacts)?;
        let stats_dir = self.store.stats.join(lane.name());
        std::fs::create_dir_all(&stats_dir)?;
        let stats_file = stats_dir.join(format!("{job_name}.json"));
        let (mut args, budget_s) = self.launch(lane, &kind, &artifacts)?;
        if self.budget.hard_memory {
            let limit = lane.rss_limit_mb + HARD_MEMORY_HEADROOM_MB;
            let mut scoped: Vec<String> = [
                "systemd-run",
                "--user",
                "--scope",
                "--quiet",
                "--collect",
                "-p",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            scoped.extend([
                format!("MemoryMax={limit}M"),
                "-p".into(),
                "MemorySwapMax=0".into(),
                "--".into(),
            ]);
            scoped.extend(args);
            args = scoped;
        }
        let env = libfuzzer::environment(
            lane,
            &self.dist.join("artifacts/schedules"),
            &stats_file,
            &self.store.tmp,
            self.symbolizer.as_deref(),
        );
        let mut log = RotatingLog::open(
            &self
                .store
                .logs
                .join(lane.name())
                .join(format!("{job_name}.log")),
            LOG_BYTES,
        )?;
        log.line(&format!(
            "# {} job {job_name}: {}",
            kind.phase(),
            args.join(" ")
        ));
        let mut child = Command::new(&args[0])
            .args(&args[1..])
            .envs(env)
            .current_dir(&artifacts)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()?;
        let stderr = child.stderr.take().expect("piped stderr");
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).split(b'\n').map_while(Result::ok) {
                if sender
                    .send(Message::Line(
                        id,
                        String::from_utf8_lossy(&line).into_owned(),
                    ))
                    .is_err()
                {
                    return;
                }
            }
            let _ = sender.send(Message::Closed(id));
        });
        let deadline =
            Instant::now() + Duration::from_secs(budget_s + 2 * lane.timeout_s + STARTUP_GRACE_S);
        if matches!(kind, JobKind::Fuzz) {
            self.lane_state(&lane.name()).jobs += 1;
        }
        self.jobs.insert(
            id,
            Job {
                lane: lane.clone(),
                kind,
                child,
                started: Instant::now(),
                deadline,
                log,
                artifacts,
                stats_file,
                status: Status::default(),
                tail: VecDeque::with_capacity(TAIL_LINES),
                artifact: None,
                stop: None,
                output_closed: false,
                exited_at: None,
                inited_after: None,
            },
        );
        Ok(())
    }
}
