//! What a finished job means: findings, baselines, replays, compactions, and
//! failure backoff.

use super::job::{Job, JobKind, Merge, Replay, Stop};
use super::{corpus_names, Baseline, HarnessStats, Runner, MAX_BACKOFF_S, STARTUP_GRACE_S};
use crate::findings::{self, Context, FindingKind, FindingMeta, Occurrence};
use crate::libfuzzer;
use crate::store::{count_files, now, read_json, write_json};
use std::collections::HashSet;
use std::path::Path;

fn last_lines(tail: &[String], count: usize) -> String {
    tail[tail.len().saturating_sub(count)..].join(" | ")
}

impl Runner {
    pub(super) fn finish(&mut self, job: Job, code: i32) {
        match &job.kind {
            JobKind::Fuzz | JobKind::Baseline => self.finish_fuzz(&job, code),
            JobKind::Replay(replay) => self.finish_replay(&job, replay, code),
            JobKind::Merge(merge) => self.finish_merge(&job, merge, code),
        }
        let _ = std::fs::remove_dir_all(&job.artifacts);
    }

    fn merge_harness_stats(&mut self, job: &Job) {
        if let Some(stats) = read_json::<HarnessStats>(&job.stats_file) {
            let harness = &mut self.lane_state(&job.lane.name()).harness;
            for (key, value) in stats.counters {
                *harness.counters.entry(key).or_default() += value;
            }
            for (key, value) in stats.seconds {
                *harness.seconds.entry(key).or_default() += value;
            }
        }
        let _ = std::fs::remove_file(&job.stats_file);
    }

    /// A fuzzing or baseline job: account its work, then triage a failure.
    fn finish_fuzz(&mut self, job: &Job, code: i32) {
        let name = job.lane.name();
        let baseline = matches!(job.kind, JobKind::Baseline);
        let watchdog = matches!(job.stop, Some(Stop::Watchdog(_)));
        let elapsed = job.started.elapsed().as_secs_f64();
        self.merge_harness_stats(job);
        {
            let state = self.lane_state(&name);
            state.cpu_seconds += elapsed * job.lane.threads as f64;
            state.cov = state.cov.max(job.status.cov);
            state.ft = state.ft.max(job.status.ft);
            state.peak_rss_mb = state.peak_rss_mb.max(job.status.rss_mb);
            if baseline {
                // A baseline cut short by the runner is neither passed nor
                // failed; it runs again on resume.
                if code == 0 || job.stop.is_none() {
                    state.baseline = Some(Baseline {
                        ok: code == 0,
                        seconds: elapsed.round(),
                        at: now(),
                        execs: job.status.execs,
                    });
                }
            } else {
                state.execs += job.status.execs;
                match job.inited_after {
                    Some(init) => state.init_seconds = init,
                    // Stopped while still re-executing the corpus: that took
                    // at least this long, so the next job gets more time.
                    None if watchdog => {
                        state.init_seconds = state.init_seconds.max(elapsed as u64);
                    }
                    None => {}
                }
            }
        }
        if code == 0 {
            self.lane_state(&name).consecutive_failures = 0;
            if baseline {
                self.event(&format!(
                    "baseline {name}: ok ({} seeds, {elapsed:.0}s)",
                    job.status.execs
                ));
            }
            return;
        }
        let artifact = job.failure_artifact();
        if self.stopping && job.stop.is_some() && artifact.is_none() {
            return;
        }
        let tail: Vec<String> = job.tail.iter().cloned().collect();
        let failure_output = tail
            .iter()
            .any(|line| line.contains("panicked at") || line.contains("ERROR:"));
        if artifact.is_some() || failure_output {
            let kind = libfuzzer::classify(&tail, artifact.as_deref());
            self.record_finding(job, kind, artifact.as_deref(), &tail);
        } else if watchdog && baseline {
            self.event(&format!(
                "baseline {name}: outlasted its seed-count deadline ({elapsed:.0}s); it reruns on resume"
            ));
        } else if watchdog && !job.status.inited {
            let init = self.lane_state(&name).init_seconds;
            self.event(&format!(
                "{name}: corpus re-execution outlasted the deadline ({elapsed:.0}s); next job allows {}s",
                init.saturating_mul(2)
            ));
        } else if watchdog {
            // libFuzzer writes the running input only when its own -timeout
            // fires, so a job stuck past that has no artifact to keep.
            let mut report = tail;
            report.push(format!(
                "jolt-fuzz: no libFuzzer progress for {elapsed:.0}s after INITED; stopped by the runner's watchdog; the input being executed was not identified"
            ));
            self.record_finding(job, FindingKind::Hang, None, &report);
        } else if !job.status.inited && elapsed < STARTUP_GRACE_S as f64 {
            self.lane_state(&name).startup_failures += 1;
            self.backoff(
                &name,
                &format!("startup failure (exit {code}): {}", last_lines(&tail, 5)),
            );
        } else {
            self.lane_state(&name).infra_failures += 1;
            let reason = if code < 0 {
                format!("killed by signal {}", -code)
            } else {
                format!("exit {code}")
            };
            self.backoff(
                &name,
                &format!(
                    "infrastructure failure ({reason}, no artifact; possibly an OS OOM kill): {}",
                    last_lines(&tail, 3)
                ),
            );
        }
    }

    fn record_finding(
        &mut self,
        job: &Job,
        kind: FindingKind,
        artifact: Option<&Path>,
        report: &[String],
    ) {
        let name = job.lane.name();
        let elapsed = job.started.elapsed().as_secs_f64();
        let (id, text) = libfuzzer::signature(kind, &job.lane.target, report);
        let context = Context {
            host: self.campaign.machine.hostname.clone(),
            campaign_id: self.campaign.campaign_id.clone(),
            build_id: self.build_id.clone(),
            phase: job.kind.phase(),
        };
        let recorded = findings::record(
            &self.store.findings,
            Occurrence {
                id: &id,
                signature: &text,
                kind,
                lane: &name,
                target: &job.lane.target,
                artifact,
                report,
                context,
            },
        );
        let (meta, new) = match recorded {
            Ok(result) => result,
            Err(error) => {
                self.event(&format!("could not record finding {id}: {error}"));
                return;
            }
        };
        self.lane_state(&name).findings += 1;
        let quarantine = self.store.quarantine.join(&job.lane.target);
        let moved = findings::quarantine_corpus_copy(
            artifact,
            &self.store.corpus.join(&job.lane.target),
            &quarantine,
        )
        .or_else(|| {
            // `-runs=0` stops at the first failing seed, so a baseline
            // failure is quarantined even once its corpus copy is gone.
            matches!(job.kind, JobKind::Baseline)
                .then(|| findings::quarantine_artifact(artifact, &quarantine))
                .flatten()
        });
        self.event(&format!(
            "{}finding {id} (x{}) in {name} after {elapsed:.0}s: {text}{}",
            if new { "NEW " } else { "" },
            meta.count,
            moved.as_ref().map_or(String::new(), |path| format!(
                "; quarantined corpus input {}",
                path.display()
            ))
        ));
        if new && self.options.replay_new_findings {
            if let Some(input) = meta.inputs().next() {
                self.pending_replays.push_back((
                    job.lane.clone(),
                    Replay {
                        input: self.store.findings.join(&id).join(input),
                        finding_id: id,
                    },
                ));
            }
        }
        if elapsed < 60.0 && !job.status.inited && moved.is_none() {
            self.backoff(
                &name,
                "failure before initialization with no corpus input to quarantine",
            );
        } else {
            self.lane_state(&name).consecutive_failures = 0;
        }
    }

    /// Chain the next variant's merge step, or after the last one keep the
    /// merged set: move snapshot inputs it dropped to
    /// `corpus-archive/<target>/`. Inputs fuzz jobs added meanwhile stay.
    fn finish_merge(&mut self, job: &Job, merge: &Merge, code: i32) {
        let target = job.lane.target.clone();
        let interrupted = code != 0 && matches!(job.stop, Some(Stop::Shutdown(_)));
        match merge.remaining.split_first() {
            // Stopped by the runner, not a merge failure: compaction becomes
            // due again on the next check.
            _ if interrupted => self.event(&format!("compaction of {target} interrupted")),
            _ if code != 0 => self.compaction_failed(&target, &format!("exit {code}")),
            Some(_) if self.stopping => {
                self.event(&format!("compaction of {target} interrupted"));
            }
            Some((next, remaining)) => {
                let step = Merge {
                    remaining: remaining.to_vec(),
                    ..merge.clone()
                };
                match self.spawn(next, JobKind::Merge(step)) {
                    Ok(()) => return,
                    Err(error) => self.compaction_failed(
                        &target,
                        &format!("step {} failed to start: {error}", next.name()),
                    ),
                }
            }
            None => self.keep_merged(&target, merge),
        }
        let _ = std::fs::remove_dir_all(&merge.work);
    }

    fn compaction_failed(&mut self, target: &str, reason: &str) {
        let retry = now() + 6 * 3600;
        for lane in self.lanes.iter().filter(|lane| lane.target == target) {
            self.state
                .lanes
                .entry(lane.name())
                .or_default()
                .next_compaction_at = retry;
        }
        self.event(&format!(
            "compaction of {target} failed ({reason}); retrying in 6h"
        ));
    }

    fn keep_merged(&mut self, target: &str, merge: &Merge) {
        let kept: HashSet<String> = corpus_names(&merge.work.join("merged"))
            .into_iter()
            .collect();
        let corpus = self.store.corpus.join(target);
        let archive = self.store.root.join("corpus-archive").join(target);
        let _ = std::fs::create_dir_all(&archive);
        let mut archived = 0u64;
        for file in &merge.snapshot {
            if !kept.contains(file)
                && std::fs::rename(corpus.join(file), archive.join(file)).is_ok()
            {
                archived += 1;
            }
        }
        let remaining = count_files(&corpus);
        let next = now() + 3600;
        for lane in self.lanes.iter().filter(|lane| lane.target == target) {
            let state = self.state.lanes.entry(lane.name()).or_default();
            state.compacted_corpus_files = remaining;
            state.corpus_files = remaining;
            state.compactions += 1;
            state.next_compaction_at = next;
        }
        self.event(&format!(
            "compacted {target}: {} -> {} inputs in {:.0}s ({archived} moved to corpus-archive)",
            merge.snapshot.len(),
            merge.snapshot.len() as u64 - archived,
            merge.started.elapsed().as_secs_f64()
        ));
    }

    fn finish_replay(&mut self, job: &Job, replay: &Replay, code: i32) {
        let id = &replay.finding_id;
        let tail: Vec<String> = job.tail.iter().cloned().collect();
        let verdict = match job.stop {
            // No verdict: the finding is replayed again on resume.
            Some(Stop::Shutdown(_)) => {
                self.event(&format!("replay of {id} interrupted"));
                return;
            }
            Some(Stop::Watchdog(_)) => "unknown (the replay outlasted its deadline)".to_string(),
            None if code == 0 => "no (input passed when replayed in a fresh process)".to_string(),
            None => {
                let kind = libfuzzer::classify(&tail, None);
                let (replay_id, _) = libfuzzer::signature(kind, &job.lane.target, &tail);
                if replay_id == *id {
                    "yes".to_string()
                } else {
                    format!("different signature on replay: {replay_id}")
                }
            }
        };
        let directory = self.store.findings.join(id);
        let meta_path = directory.join("meta.json");
        let Some(mut meta) = read_json::<FindingMeta>(&meta_path) else {
            return;
        };
        meta.reproducible = Some(verdict.clone());
        let _ = std::fs::write(directory.join("replay.txt"), tail.join("\n") + "\n");
        let _ = write_json(&meta_path, &meta);
        self.event(&format!("replay of {id}: reproducible={verdict}"));
    }

    pub(super) fn backoff(&mut self, name: &str, reason: &str) {
        let state = self.lane_state(name);
        state.consecutive_failures += 1;
        let delay = (5u64 << (state.consecutive_failures - 1).min(20)).min(MAX_BACKOFF_S);
        state.backoff_until = now() + delay;
        let failures = state.consecutive_failures;
        self.event(&format!(
            "{name}: {reason}; retry in {delay}s (consecutive failures: {failures})"
        ));
    }
}
