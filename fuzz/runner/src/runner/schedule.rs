//! Admission against the budget, lane choice, and corpus compaction.

use super::job::{JobKind, Merge};
use super::{corpus_names, Runner};
use crate::registry::Lane;
use crate::store::{now, snapshot};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::io::Result as IoResult;
use std::time::Instant;

impl Runner {
    /// Fuzzing time per job: the configured slice, lengthened so re-executing
    /// the corpus at startup (which grows with the corpus) stays under ~10%.
    pub(super) fn slice_for(&self, lane: &Lane) -> u64 {
        let init = self
            .state
            .lanes
            .get(&lane.name())
            .map_or(0, |state| state.init_seconds);
        self.options.slice_s.max(init.saturating_mul(10))
    }

    /// Memory a job of `lane` is expected to use, for admission against the
    /// budget: 1.5x the lane's observed peak plus 512 MiB, never above its
    /// per-worker limit (which the cgroup scope or libFuzzer still enforces).
    /// Reserving every worker's full cap idled most CPUs whenever the budget
    /// was smaller than `workers x cap`.
    fn reservation(&self, lane: &Lane) -> u64 {
        let cap = lane.rss_limit_mb;
        match self
            .state
            .lanes
            .get(&lane.name())
            .map(|state| state.peak_rss_mb)
        {
            Some(peak) if peak > 0 => (peak * 3 / 2 + 512).min(cap),
            _ => cap,
        }
    }

    fn fits(&self, lane: &Lane) -> bool {
        if self.jobs.is_empty() {
            return true; // a small budget still makes progress
        }
        let cpus: u64 = self.jobs.values().map(|job| job.lane.threads).sum();
        let memory: u64 = self
            .jobs
            .values()
            .map(|job| self.reservation(&job.lane))
            .sum();
        cpus + lane.threads <= self.budget.cpus
            && memory + self.reservation(lane) <= self.budget.memory_mb
    }

    /// Whether `lane`'s startup baseline is queued or running. A lane fuzzes
    /// only after its own baseline; a compaction, which runs every variant
    /// of a target, only after all of the target's.
    fn awaiting_baseline(&self, lane: &Lane) -> bool {
        let name = lane.name();
        self.baseline_pending
            .iter()
            .any(|pending| pending.name() == name)
            || self
                .jobs
                .values()
                .any(|job| matches!(job.kind, JobKind::Baseline) && job.lane.name() == name)
    }

    fn choose(&self) -> Option<Lane> {
        let running: HashMap<String, u64> = self
            .jobs
            .values()
            .filter(|job| matches!(job.kind, JobKind::Fuzz))
            .fold(HashMap::new(), |mut acc, job| {
                *acc.entry(job.lane.name()).or_default() += 1;
                acc
            });
        let time = now();
        let mut best: Option<(f64, Lane)> = None;
        for lane in &self.lanes {
            let state = self
                .state
                .lanes
                .get(&lane.name())
                .cloned()
                .unwrap_or_default();
            if self.awaiting_baseline(lane) || state.backoff_until > time {
                continue;
            }
            let projected = state.cpu_seconds
                + (running.get(&lane.name()).copied().unwrap_or(0)
                    * self.slice_for(lane)
                    * lane.threads) as f64;
            let score = projected / lane.weight;
            if best
                .as_ref()
                .is_none_or(|(best_score, _)| score < *best_score)
            {
                best = Some((score, lane.clone()));
            }
        }
        // Head-of-line: if the most under-served lane does not fit yet, wait
        // for capacity rather than backfilling with lanes that need less, which
        // starved heavy lanes indefinitely.
        best.map(|(_, lane)| lane).filter(|lane| self.fits(lane))
    }

    /// Snapshot a target's corpus and start the first merge step, under
    /// `variants[0]`; the remaining variants follow as its steps finish.
    fn start_compaction(&mut self, variants: &[Lane]) -> IoResult<()> {
        let Some((first, remaining)) = variants.split_first() else {
            return Ok(());
        };
        let work = self.store.root.join("merge").join(&first.target);
        let _ = std::fs::remove_dir_all(&work);
        // Merge a private snapshot: running jobs replace and remove corpus
        // files (`-reduce_inputs`), which aborts a merge that reads the live
        // directory.
        snapshot(
            &self.store.corpus.join(&first.target),
            &work.join("snapshot"),
            true,
        )?;
        std::fs::create_dir_all(work.join("merged"))?;
        let merge = Merge {
            snapshot: corpus_names(&work.join("snapshot")),
            work,
            remaining: remaining.to_vec(),
            started: Instant::now(),
        };
        self.spawn(first, JobKind::Merge(merge))
    }

    /// Start a corpus merge for any target whose corpus doubled since its
    /// last compaction (and has at least 200 inputs), one merge per target.
    fn schedule_compactions(&mut self) {
        if self.stopping {
            return;
        }
        let time = now();
        let merging: HashSet<String> = self
            .jobs
            .values()
            .filter(|job| matches!(job.kind, JobKind::Merge(_)))
            .map(|job| job.lane.target.clone())
            .collect();
        let mut seen = HashSet::new();
        for lane in self.lanes.clone() {
            if !seen.insert(lane.target.clone()) || merging.contains(&lane.target) {
                continue;
            }
            let state = self
                .state
                .lanes
                .get(&lane.name())
                .cloned()
                .unwrap_or_default();
            let due = state.corpus_files >= 200
                && state.corpus_files >= 2 * state.compacted_corpus_files
                && state.next_compaction_at <= time;
            let variants: Vec<Lane> = self
                .lanes
                .iter()
                .filter(|other| other.target == lane.target)
                .cloned()
                .collect();
            if !due
                || variants
                    .iter()
                    .any(|variant| self.awaiting_baseline(variant))
                || !self.fits(&lane)
            {
                continue;
            }
            match self.start_compaction(&variants) {
                Ok(()) => self.event(&format!(
                    "compacting {} ({} inputs, {} variant steps)",
                    lane.target,
                    state.corpus_files,
                    variants.len()
                )),
                Err(error) => self.event(&format!(
                    "compaction of {} failed to start: {error}",
                    lane.target
                )),
            }
        }
    }

    pub(super) fn schedule(&mut self) {
        while !self.stopping
            && self
                .baseline_pending
                .front()
                .is_some_and(|lane| self.fits(lane))
        {
            let Some(lane) = self.baseline_pending.pop_front() else {
                break;
            };
            if let Err(error) = self.spawn(&lane, JobKind::Baseline) {
                self.event(&format!(
                    "baseline {} failed to start: {error}",
                    lane.name()
                ));
            }
        }
        while !self.stopping
            && self
                .pending_replays
                .front()
                .is_some_and(|(lane, _)| self.fits(lane))
        {
            let Some((lane, replay)) = self.pending_replays.pop_front() else {
                break;
            };
            let id = replay.finding_id.clone();
            if let Err(error) = self.spawn(&lane, JobKind::Replay(replay)) {
                self.event(&format!("replay of {id} failed to start: {error}"));
            }
        }
        self.schedule_compactions();
        while !self.stopping {
            let Some(lane) = self.choose() else {
                return;
            };
            if let Err(error) = self.spawn(&lane, JobKind::Fuzz) {
                self.backoff(&lane.name(), &format!("spawn failed: {error}"));
            }
        }
    }

    /// Queue every lane's startup baseline. Each lane starts fuzzing as soon
    /// as its own baseline finishes; failures become findings.
    pub(super) fn queue_baselines(&mut self) {
        self.event(&format!(
            "baseline: executing shipped seeds for {} lanes",
            self.lanes.len()
        ));
        // Slowest (end-to-end) lanes first so they overlap cheaper lanes' fuzzing.
        let mut lanes = self.lanes.clone();
        lanes.sort_by_key(|lane| Reverse(lane.timeout_s));
        self.baseline_pending = lanes.into();
    }
}
