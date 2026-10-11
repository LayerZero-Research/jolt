//! Commands that work while a campaign runs, plus triage helpers.

use crate::findings::{self, FindingMeta};
use crate::libfuzzer::{self, BINARY};
use crate::prepare::BuildInfo;
use crate::registry::{self, Lane};
use crate::runner::job::Phase;
use crate::runner::{symbolizer, Live, State};
use crate::store::{directory_bytes, files, now, read_json, sha256_hex, snapshot, Campaign, Store};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::json;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Builder;

fn duration(seconds: f64) -> String {
    let seconds = seconds as u64;
    let (days, hours, minutes) = (seconds / 86400, seconds % 86400 / 3600, seconds % 3600 / 60);
    if days > 0 {
        format!("{days}d{hours:02}h{minutes:02}m")
    } else {
        format!("{hours}h{minutes:02}m")
    }
}

#[expect(clippy::expect_used, reason = "the report is plain serde data")]
pub fn status(store: &Store, lanes: &[Lane], as_json: bool) -> i32 {
    let state: State = read_json(&store.state_path).unwrap_or_default();
    let live: Option<Live> = if store.is_locked() {
        read_json(&store.live_path)
    } else {
        None
    };
    let campaign: Option<Campaign> = read_json(&store.campaign_path);
    let all_findings = findings::summarize(&store.findings);
    let disk = directory_bytes(&store.root);
    if as_json {
        let report = json!({
            "campaign": campaign,
            "running": live.is_some(),
            "live": live,
            "lanes": state.lanes,
            "findings": all_findings,
            "disk_bytes": disk,
        });
        println!("{}", serde_json::to_string_pretty(&report).expect("json"));
        return 0;
    }
    println!(
        "campaign {} on {}  output {}",
        campaign.as_ref().map_or("?", |c| c.campaign_id.as_str()),
        campaign
            .as_ref()
            .map_or("?", |c| c.machine.hostname.as_str()),
        store.root.display()
    );
    match &live {
        Some(live) => println!(
            "RUNNING pid {} for {}; budget {} cpus / {} MiB; hard memory limit: {}",
            live.pid,
            duration(now().saturating_sub(live.started) as f64),
            live.budget.cpus,
            live.budget.memory_mb,
            if live.budget.hard_memory { "yes" } else { "no" }
        ),
        None => println!("not running"),
    }
    println!("disk usage {:.1} MiB\n", disk as f64 / (1 << 20) as f64);
    let jobs = live.as_ref().map_or(&[][..], |live| live.jobs.as_slice());
    let header = format!(
        "{:32} {:>10} {:>12} {:>8} {:>7} {:>8} {:>7} {:>5} {:>5}  baseline",
        "lane", "cpu-time", "execs", "exec/s", "cov", "ft", "corpus", "find", "fail"
    );
    println!("{header}\n{}", "-".repeat(header.len()));
    for lane in lanes {
        let name = lane.name();
        let entry = state.lanes.get(&name).cloned().unwrap_or_default();
        let running: Vec<_> = jobs.iter().filter(|job| job.lane == name).collect();
        let live_execs: u64 = running
            .iter()
            .filter(|job| job.purpose == Phase::Fuzz)
            .map(|job| job.status.execs)
            .sum();
        let eps: u64 = running.iter().map(|job| job.status.exec_per_s).sum();
        let cov = running
            .iter()
            .map(|job| job.status.cov)
            .fold(entry.cov, u64::max);
        let ft = running
            .iter()
            .map(|job| job.status.ft)
            .fold(entry.ft, u64::max);
        let baseline = match &entry.baseline {
            None => "-",
            Some(baseline) if baseline.ok => "ok",
            Some(_) => "FAILED",
        };
        let backoff = if entry.backoff_until > now() {
            format!("  backoff {}s", entry.backoff_until - now())
        } else {
            String::new()
        };
        println!(
            "{name:32} {:>10} {:>12} {eps:>8} {cov:>7} {ft:>8} {:>7} {:>5} {:>5}  {baseline}{backoff}",
            duration(entry.cpu_seconds),
            entry.execs + live_execs,
            entry.corpus_files,
            entry.findings,
            entry.startup_failures + entry.infra_failures,
        );
    }
    println!();
    if all_findings.is_empty() {
        println!("no findings");
    }
    for meta in &all_findings {
        println!(
            "[{}] {}  x{}  reproducible={}  phase={}\n    {}",
            meta.kind,
            meta.id,
            meta.count,
            meta.reproducible.as_deref().unwrap_or("-"),
            meta.context.phase,
            meta.signature
        );
    }
    let reach: Vec<_> = state
        .lanes
        .iter()
        .filter(|(_, lane)| !lane.harness.counters.is_empty())
        .collect();
    if !reach.is_empty() {
        println!("\nharness reach counters (completed jobs):");
        for (name, lane) in reach {
            let text: Vec<String> = lane
                .harness
                .counters
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect();
            println!("  {name}: {}", text.join(", "));
        }
    }
    0
}

pub fn export(
    store: &Store,
    destination: Option<&Path>,
    with_logs: bool,
) -> Result<PathBuf, String> {
    let (host, id) = read_json::<Campaign>(&store.campaign_path).map_or_else(
        || ("unknown".to_string(), "unknown".to_string()),
        |campaign| (campaign.machine.hostname, campaign.campaign_id),
    );
    let name = format!("jolt-fuzz-{host}-{id}-{}.tar.gz", now());
    let path = match destination {
        Some(path) if path.to_string_lossy().ends_with(".tar.gz") => path.to_path_buf(),
        Some(dir) => dir.join(name),
        None => store.exports.join(name),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let partial = path.with_extension("gz.partial");
    let staging = store.tmp.join(format!("export-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    // A running campaign renames and removes corpus files and appends to its
    // logs, so the archive is built from a snapshot: hard links pin the
    // write-once corpus and finding files, and appended files are copied.
    let mut members = vec![
        ("campaign.json", false),
        ("state.json", false),
        ("events.log", false),
        ("corpus", true),
        ("findings", true),
        ("quarantine", true),
    ];
    if with_logs {
        members.push(("logs", false));
    }
    let snapshotted = members.into_iter().try_for_each(|(member, link)| {
        let (source, target) = (store.root.join(member), staging.join(member));
        let copied = if source.is_dir() {
            snapshot(&source, &target, link)
        } else {
            std::fs::create_dir_all(&staging)
                .and_then(|()| std::fs::copy(&source, &target).map(drop))
        };
        match copied {
            Err(error) if error.kind() != ErrorKind::NotFound => {
                Err(format!("snapshot {member}: {error}"))
            }
            _ => Ok(()),
        }
    });
    let result = snapshotted.and_then(|()| {
        let file =
            File::create(&partial).map_err(|e| format!("create {}: {e}", partial.display()))?;
        let mut archive = Builder::new(GzEncoder::new(file, Compression::default()));
        archive.follow_symlinks(false);
        archive
            .append_dir_all(format!("{host}-{id}"), &staging)
            .map_err(|e| format!("archive: {e}"))?;
        archive
            .into_inner()
            .and_then(|gz| gz.finish())
            .map_err(|e| e.to_string())?;
        std::fs::rename(&partial, &path).map_err(|e| format!("rename {}: {e}", partial.display()))
    });
    let _ = std::fs::remove_dir_all(&staging);
    if result.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    result.map(|()| path)
}

fn lane_for<'a>(lanes: &'a [Lane], spec: &str) -> Result<&'a Lane, String> {
    lanes
        .iter()
        .find(|lane| lane.matches(spec))
        .ok_or_else(|| format!("unknown target {spec}"))
}

/// A finding id (its stored samples) or `TARGET[@VARIANT]` plus input files.
fn resolve<'a>(
    store: &Store,
    lanes: &'a [Lane],
    spec: &str,
    inputs: &[PathBuf],
) -> Result<(&'a Lane, Vec<PathBuf>, Option<PathBuf>), String> {
    let directory = store.findings.join(spec);
    if let Some(meta) = read_json::<FindingMeta>(&directory.join("meta.json")) {
        let lane = lane_for(lanes, meta.lanes.first().map_or("", String::as_str))?;
        let paths: Vec<PathBuf> = meta.inputs().map(|input| directory.join(input)).collect();
        if paths.is_empty() {
            return Err(format!(
                "finding {spec} has no stored input; see its sample reports"
            ));
        }
        return Ok((lane, paths, Some(directory)));
    }
    if inputs.is_empty() {
        return Err(format!(
            "{spec} is not a finding id; reproduce TARGET needs input files"
        ));
    }
    Ok((lane_for(lanes, spec)?, inputs.to_vec(), None))
}

fn env_for(dist: &Path, store: &Store, lane: &Lane) -> Vec<(String, String)> {
    let mut env = libfuzzer::environment(
        lane,
        &dist.join("artifacts/schedules"),
        Path::new("/dev/null"),
        &store.tmp,
        symbolizer(dist).as_deref(),
    );
    env.insert("RUST_BACKTRACE".into(), "full".into());
    env.into_iter().collect()
}

pub fn reproduce(
    dist: &Path,
    store: &Store,
    lanes: &[Lane],
    spec: &str,
    inputs: &[PathBuf],
) -> Result<i32, String> {
    let (lane, paths, _) = resolve(store, lanes, spec, inputs)?;
    let scratch = store.tmp.join(format!("reproduce-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
    let args = libfuzzer::replay_args(&dist.join("bin").join(BINARY), lane, &scratch, &paths);
    let env = env_for(dist, store, lane);
    let shown: Vec<String> = lane.env.iter().map(|(k, v)| format!("{k}={v}")).collect();
    println!("+ {} {}", shown.join(" "), args.join(" "));
    let status = Command::new(&args[0])
        .args(&args[1..])
        .envs(env)
        .status()
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&scratch);
    Ok(status.code().unwrap_or(1))
}

pub fn minimize(
    dist: &Path,
    store: &Store,
    lanes: &[Lane],
    finding: &str,
    seconds: u64,
) -> Result<i32, String> {
    let (lane, paths, directory) = resolve(store, lanes, finding, &[])?;
    let directory = directory.ok_or_else(|| format!("{finding} is not a finding id"))?;
    let work = directory.join("minimize-work");
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let output = directory.join(format!("minimized-{}.input", now()));
    let mut args =
        libfuzzer::base_args(&dist.join("bin").join(BINARY), lane, lane.timeout_s, &work);
    args.extend([
        "-minimize_crash=1".to_string(),
        format!("-max_total_time={seconds}"),
        format!("-exact_artifact_path={}", output.display()),
        paths[0].display().to_string(),
    ]);
    println!("+ {}", args.join(" "));
    let status = Command::new(&args[0])
        .args(&args[1..])
        .envs(env_for(dist, store, lane))
        .current_dir(&work)
        .status()
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&work);
    if output.is_file() {
        println!(
            "minimized input: {} (original kept: {})",
            output.display(),
            paths[0].display()
        );
        Ok(0)
    } else {
        println!(
            "no smaller crashing input (exit {:?}); original kept: {}",
            status.code(),
            paths[0].display()
        );
        Ok(1)
    }
}

/// Static checks of a prepared distribution.
pub fn validate_dist(dist: &Path, lanes: &[Lane]) -> Vec<String> {
    let mut problems = Vec::new();
    if read_json::<BuildInfo>(&dist.join("BUILD-INFO.json")).is_none() {
        problems.push("BUILD-INFO.json is missing or unreadable; run `jolt-fuzz prepare`".into());
    }
    match std::fs::read_to_string(dist.join("MANIFEST.sha256")) {
        Ok(manifest) => {
            let mut listed = BTreeSet::new();
            for line in manifest.lines() {
                let Some((digest, name)) = line.split_once("  ").filter(|(digest, _)| {
                    digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit())
                }) else {
                    problems.push(format!("unparseable MANIFEST.sha256 line: {line:?}"));
                    continue;
                };
                listed.insert(PathBuf::from(name));
                match std::fs::read(dist.join(name)) {
                    Ok(bytes) => {
                        if sha256_hex(&bytes) != digest {
                            problems.push(format!("checksum mismatch: {name}"));
                        }
                    }
                    Err(_) => problems.push(format!("missing {name}")),
                }
            }
            for file in files(dist) {
                if file != Path::new("MANIFEST.sha256") && !listed.contains(&file) {
                    problems.push(format!("{} is not in MANIFEST.sha256", file.display()));
                }
            }
        }
        Err(_) => problems.push("MANIFEST.sha256 is missing".into()),
    }
    if !dist.join("bin").join(BINARY).is_file() {
        problems.push(format!("missing instrumented binary bin/{BINARY}"));
    }
    for target in registry::targets(lanes) {
        if !dist.join("seeds").join(&target).is_dir() {
            problems.push(format!("missing seeds/{target}"));
        }
    }
    for (dir, what) in [
        ("artifacts/guests", "guest ELFs"),
        ("artifacts/bundles", "verifier bundles"),
    ] {
        let present = std::fs::read_dir(dist.join(dir))
            .map(|entries| entries.flatten().next().is_some())
            .unwrap_or(false);
        if !present {
            problems.push(format!("missing {dir} ({what})"));
        }
    }
    let schedules = std::fs::read_dir(dist.join("artifacts/schedules"))
        .map(|e| e.flatten().count())
        .unwrap_or(0);
    if schedules == 0 {
        problems.push("missing artifacts/schedules/*.aks".into());
    }
    problems
}
