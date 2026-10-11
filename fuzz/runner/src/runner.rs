//! The unattended campaign loop.
//!
//! Workers are runner-managed: each job is one libFuzzer process for one lane,
//! running until it records a failure or its time slice ends. libFuzzer's own
//! `-fork`/`-jobs` modes are never used, so worker count and Akita's internal
//! Rayon threads are the only two concurrency settings, accounted together.

pub mod job;
mod schedule;
mod signals;
mod triage;

use crate::findings;
use crate::libfuzzer::{self, Status};
use crate::registry::Lane;
use crate::resources::Budget;
use crate::store::{count_files, now, read_json, which, write_json, Campaign, RotatingLog, Store};
use job::{Job, Phase, Replay, Stop};
use libc::{SIGINT, SIGKILL, SIGTERM};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::Result as IoResult;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

const TAIL_LINES: usize = 400;
const LOG_BYTES: u64 = 4 << 20;
const STATE_INTERVAL: Duration = Duration::from_secs(15);
const CORPUS_COUNT_INTERVAL: Duration = Duration::from_secs(300);
const STARTUP_GRACE_S: u64 = 120;
const KILL_GRACE: Duration = Duration::from_secs(30);
const MAX_BACKOFF_S: u64 = 1800;

pub struct Options {
    pub slice_s: u64,
    pub duration_s: Option<u64>,
    pub skip_baseline: bool,
    pub replay_new_findings: bool,
    pub extra_args: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Baseline {
    pub ok: bool,
    pub seconds: f64,
    pub at: u64,
    pub execs: u64,
}

/// Harness reach counters and timings: one worker's stats file
/// (`JOLT_FUZZ_STATS_FILE`), or a lane's totals over completed jobs.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HarnessStats {
    pub counters: BTreeMap<String, f64>,
    pub seconds: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LaneState {
    pub cpu_seconds: f64,
    pub jobs: u64,
    pub execs: u64,
    pub findings: u64,
    pub startup_failures: u64,
    pub infra_failures: u64,
    pub consecutive_failures: u64,
    pub backoff_until: u64,
    pub cov: u64,
    pub ft: u64,
    pub corpus_files: u64,
    /// Largest RSS a finished job of this lane reported.
    pub peak_rss_mb: u64,
    /// Seconds the last fuzzing job spent re-executing the corpus before
    /// mutating (libFuzzer `INITED`).
    pub init_seconds: u64,
    /// Corpus size after the last compaction (0 if never compacted).
    pub compacted_corpus_files: u64,
    pub compactions: u64,
    /// Unix time before which no compaction is attempted.
    pub next_compaction_at: u64,
    pub baseline: Option<Baseline>,
    pub harness: HarnessStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub started: u64,
    pub build_id: String,
    pub budget: Budget,
    pub stopped: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub lanes: BTreeMap<String, LaneState>,
    pub sessions: Vec<Session>,
    pub updated: u64,
}

/// `live.json`: the running session, rewritten every few seconds.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Live {
    pub pid: u32,
    pub started: u64,
    pub updated: u64,
    pub stopping: bool,
    pub budget: Budget,
    pub jobs: Vec<LiveJob>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiveJob {
    pub job: u64,
    pub lane: String,
    pub purpose: Phase,
    pub pid: u32,
    pub running_s: u64,
    pub status: Status,
    pub stats_file: PathBuf,
}

enum Message {
    Line(u64, String),
    Closed(u64),
}

pub struct Runner {
    dist: PathBuf,
    store: Store,
    lanes: Vec<Lane>,
    budget: Budget,
    options: Options,
    campaign: Campaign,
    build_id: String,
    jobs: HashMap<u64, Job>,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    stopping: bool,
    started: Instant,
    started_unix: u64,
    events: RotatingLog,
    sequence: u64,
    state: State,
    symbolizer: Option<PathBuf>,
    pending_replays: VecDeque<(Lane, Replay)>,
    last_state: Instant,
    last_corpus_count: Instant,
    /// Lanes whose startup baseline has not started in this session.
    baseline_pending: VecDeque<Lane>,
}

fn corpus_names(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
                .filter_map(|entry| entry.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

pub fn symbolizer(dist: &Path) -> Option<PathBuf> {
    let bundled = dist.join("bin/llvm-symbolizer");
    if bundled.is_file() {
        Some(bundled)
    } else {
        which("llvm-symbolizer")
    }
}

impl Runner {
    pub fn new(
        dist: PathBuf,
        store: Store,
        lanes: Vec<Lane>,
        budget: Budget,
        options: Options,
        campaign: Campaign,
        build_id: String,
    ) -> IoResult<Self> {
        let mut state: State = read_json(&store.state_path).unwrap_or_default();
        state.sessions.push(Session {
            started: now(),
            build_id: build_id.clone(),
            budget: budget.clone(),
            stopped: None,
        });
        for lane in &lanes {
            state.lanes.entry(lane.name()).or_default();
        }
        let events = RotatingLog::open(&store.root.join("events.log"), LOG_BYTES)?;
        let (sender, receiver) = mpsc::channel();
        let symbolizer = symbolizer(&dist);
        Ok(Self {
            dist,
            store,
            lanes,
            budget,
            options,
            campaign,
            build_id,
            jobs: HashMap::new(),
            sender,
            receiver,
            stopping: false,
            started: Instant::now(),
            started_unix: now(),
            events,
            sequence: 0,
            state,
            symbolizer,
            pending_replays: VecDeque::new(),
            last_state: Instant::now() - STATE_INTERVAL,
            last_corpus_count: Instant::now() - CORPUS_COUNT_INTERVAL,
            baseline_pending: VecDeque::new(),
        })
    }

    fn event(&mut self, message: &str) {
        let line = format!("{} {message}", now());
        self.events.line(&line);
        println!("{line}");
    }

    fn lane_state(&mut self, name: &str) -> &mut LaneState {
        self.state.lanes.entry(name.to_string()).or_default()
    }

    fn write_state(&mut self, force: bool) {
        if !force && self.last_state.elapsed() < STATE_INTERVAL {
            return;
        }
        self.last_state = Instant::now();
        if force || self.last_corpus_count.elapsed() >= CORPUS_COUNT_INTERVAL {
            self.last_corpus_count = Instant::now();
            for lane in self.lanes.clone() {
                let count = count_files(&self.store.corpus.join(&lane.target));
                self.lane_state(&lane.name()).corpus_files = count;
            }
        }
        self.state.updated = now();
        let _ = write_json(&self.store.state_path, &self.state);
        let live = Live {
            pid: std::process::id(),
            started: self.started_unix,
            updated: now(),
            stopping: self.stopping,
            budget: self.budget.clone(),
            jobs: self
                .jobs
                .iter()
                .map(|(id, job)| LiveJob {
                    job: *id,
                    lane: job.lane.name(),
                    purpose: job.kind.phase(),
                    pid: job.child.id(),
                    running_s: job.started.elapsed().as_secs(),
                    status: job.status.clone(),
                    stats_file: job.stats_file.clone(),
                })
                .collect(),
        };
        let _ = write_json(&self.store.live_path, &live);
    }

    /// Write the shipped seeds of `lane`'s target into `dir` as libFuzzer
    /// runs them, skipping quarantined ones; returns how many there are.
    fn stage_seeds(&self, lane: &Lane, dir: &Path) -> IoResult<u64> {
        let seeds = self.dist.join("seeds").join(&lane.target);
        let quarantine = self.store.quarantine.join(&lane.target);
        std::fs::create_dir_all(dir)?;
        let Ok(entries) = std::fs::read_dir(&seeds) else {
            return Ok(0);
        };
        let mut staged = 0;
        for entry in entries.flatten() {
            if !entry.file_type()?.is_file() {
                continue;
            }
            // libFuzzer runs at most `max_len` bytes of an input and names
            // crash artifacts after those bytes, so the staged copy must be
            // the truncated input for a crashing seed to be quarantined.
            let mut data = std::fs::read(entry.path())?;
            data.truncate(lane.max_len as usize);
            let digest = libfuzzer::sha1_hex(&data);
            if quarantine.join(&digest).exists() {
                continue;
            }
            staged += 1;
            if !dir.join(&digest).exists() {
                std::fs::write(dir.join(&digest), data)?;
            }
        }
        Ok(staged)
    }

    fn pump(&mut self, timeout: Duration) {
        let mut next = self.receiver.recv_timeout(timeout);
        loop {
            match next {
                Ok(Message::Line(id, line)) => {
                    if let Some(job) = self.jobs.get_mut(&id) {
                        job.consume(line);
                    }
                }
                Ok(Message::Closed(id)) => {
                    if let Some(job) = self.jobs.get_mut(&id) {
                        job.output_closed = true;
                    }
                }
                Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => return,
            }
            next = self.receiver.recv_timeout(Duration::ZERO);
        }
    }

    fn reap(&mut self) {
        let ids: Vec<u64> = self.jobs.keys().copied().collect();
        for id in ids {
            let Some(job) = self.jobs.get_mut(&id) else {
                continue;
            };
            match job.child.try_wait() {
                // A descendant may keep the pipe open after the worker exits.
                Ok(Some(status))
                    if job.output_closed
                        || job
                            .exited_at
                            .is_some_and(|at| at.elapsed() > Duration::from_secs(10)) =>
                {
                    let code = status
                        .code()
                        .unwrap_or_else(|| -status.signal().unwrap_or(0));
                    if let Some(job) = self.jobs.remove(&id) {
                        self.finish(job, code);
                    }
                }
                Ok(Some(_)) => {
                    // Let the reader drain the pipe first.
                    job.exited_at.get_or_insert_with(Instant::now);
                }
                Ok(None) | Err(_) => {
                    let now = Instant::now();
                    match job.stop.as_ref().map(Stop::at) {
                        None if now > job.deadline => {
                            job.signal(SIGTERM);
                            job.stop = Some(Stop::Watchdog(now));
                            let name = job.lane.name();
                            self.event(&format!(
                                "watchdog: {name} exceeded its wall-clock deadline; terminating"
                            ));
                        }
                        Some(at) if now - at > KILL_GRACE => job.signal(SIGKILL),
                        _ => {}
                    }
                }
            }
        }
    }

    /// Queue replays of findings no earlier session finished replaying.
    fn requeue_replays(&mut self) {
        for meta in findings::summarize(&self.store.findings) {
            if meta.reproducible.is_some() {
                continue;
            }
            let Some(input) = meta.inputs().next() else {
                continue;
            };
            let Some(lane) = meta
                .lanes
                .first()
                .and_then(|name| self.lanes.iter().find(|lane| lane.name() == *name))
            else {
                continue;
            };
            let input = self.store.findings.join(&meta.id).join(input);
            self.pending_replays.push_back((
                lane.clone(),
                Replay {
                    finding_id: meta.id,
                    input,
                },
            ));
        }
    }

    pub fn run(mut self) -> IoResult<()> {
        signals::install();
        for note in self.budget.notes.clone() {
            self.event(&note);
        }
        for lane in self.lanes.clone() {
            self.stage_seeds(&lane, &self.store.corpus.join(&lane.target))?;
        }
        if self.options.replay_new_findings {
            self.requeue_replays();
        }
        if !self.options.skip_baseline {
            self.queue_baselines();
        }
        self.event(&format!(
            "fuzzing {} lanes; slice {}s",
            self.lanes.len(),
            self.options.slice_s
        ));
        while !self.stopping {
            if self
                .options
                .duration_s
                .is_some_and(|limit| self.started.elapsed().as_secs() >= limit)
            {
                self.event("duration reached; stopping");
                self.stopping = true;
                break;
            }
            self.schedule();
            self.pump(Duration::from_secs(1));
            self.reap();
            self.write_state(false);
            if signals::received() > 0 && !self.stopping {
                self.stopping = true;
                self.event("received a stop signal; stopping (send again to force)");
            }
        }
        self.shutdown();
        Ok(())
    }

    fn shutdown(&mut self) {
        for job in self.jobs.values_mut() {
            job.signal(SIGINT);
            job.stop.get_or_insert(Stop::Shutdown(Instant::now()));
        }
        let deadline = Instant::now() + KILL_GRACE;
        while !self.jobs.is_empty() && Instant::now() < deadline && signals::received() < 2 {
            self.pump(Duration::from_millis(500));
            self.reap();
        }
        for job in self.jobs.values() {
            job.signal(SIGKILL);
        }
        while !self.jobs.is_empty() {
            self.pump(Duration::from_millis(200));
            self.reap();
        }
        if let Some(session) = self.state.sessions.last_mut() {
            session.stopped = Some(now());
        }
        self.write_state(true);
        let _ = std::fs::remove_file(&self.store.live_path);
        self.event("stopped; state saved");
    }
}
