//! `jolt-fuzz prepare`: build instrumented targets and package a distribution.
//!
//! Run from the source tree (`cargo run --release -p jolt-fuzz-runner --
//! prepare --out DIR`); seeds come from `jolt-fuzz-dev` (built here). A plain
//! `cargo build --release` produces no fuzzing
//! instrumentation; this command runs `cargo fuzz build` (nightly, libFuzzer,
//! AddressSanitizer, SanitizerCoverage, debug assertions, overflow checks)
//! and copies the results with everything the campaign needs offline.

use crate::libfuzzer::BINARY;
use crate::registry;
use crate::store::{files, now, read_json, sha256_hex, which, write_json, Machine};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Prepare {
    pub dist: PathBuf,
    pub sanitizer: String,
    /// Build with libFuzzer's comparison tracing. Off speeds up campaigns over
    /// large statements, whose time goes to proving kernels whose comparisons
    /// do not guide input selection.
    pub trace_compares: bool,
    pub skip_build: bool,
}

/// `BUILD-INFO.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildInfo {
    pub build_id: String,
    pub created: u64,
    pub git_commit: String,
    pub git_branch: String,
    pub git_dirty: bool,
    pub rustc: String,
    pub cargo_fuzz: String,
    pub target_triple: String,
    /// `None` when `--skip-build` packaged a build no earlier BUILD-INFO
    /// describes.
    pub sanitizer: Option<String>,
    /// `None` when `--skip-build` packaged a build no earlier BUILD-INFO
    /// describes.
    pub trace_compares: Option<bool>,
    pub flags: String,
    pub cargo_lock_sha256: String,
    pub symbolizer: Option<PathBuf>,
    pub targets: BTreeSet<String>,
    pub build_host: Machine,
}

#[derive(Deserialize)]
struct Metadata {
    target_directory: PathBuf,
}

fn output(command: &mut Command) -> Result<String, String> {
    let result = command.output().map_err(|e| format!("{command:?}: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "{command:?} failed:\n{}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(from)
        .map_err(|e| format!("read {}: {e}", from.display()))?
        .flatten()
    {
        let target = to.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)
                .map_err(|e| format!("copy {}: {e}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

impl Prepare {
    /// Package into a sibling directory and move it over `--out` only once
    /// complete, so a failed prepare leaves the previous distribution intact.
    pub fn run(self) -> Result<(), String> {
        let dist = &self.dist;
        let occupied = std::fs::read_dir(dist).is_ok_and(|mut entries| entries.next().is_some());
        if occupied && !dist.join("BUILD-INFO.json").is_file() {
            return Err(format!(
                "{} is not empty and is not a distribution (no BUILD-INFO.json); refusing to replace it",
                dist.display()
            ));
        }
        let name = dist
            .file_name()
            .ok_or_else(|| format!("--out {} names no directory", dist.display()))?;
        let staging = dist.with_file_name(format!(
            ".{}.preparing-{}",
            name.to_string_lossy(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&staging);
        let info = match self.package(&staging) {
            Ok(info) => info,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&staging);
                return Err(error);
            }
        };
        if dist.exists() {
            std::fs::remove_dir_all(dist).map_err(|e| format!("clear {}: {e}", dist.display()))?;
        }
        std::fs::rename(&staging, dist)
            .map_err(|e| format!("move {} to {}: {e}", staging.display(), dist.display()))?;
        println!(
            "prepared {} (build {}{})",
            dist.display(),
            info.build_id,
            if info.git_dirty { ", dirty tree" } else { "" }
        );
        if info.symbolizer.is_none() {
            println!("note: no llvm-symbolizer found; Rust panics are still symbolized, sanitizer frames will be raw addresses");
        }
        Ok(())
    }

    fn package(&self, dist: &Path) -> Result<BuildInfo, String> {
        let fuzz = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .canonicalize()
            .map_err(|e| format!("fuzz directory: {e}"))?;
        let repo = fuzz.join("..");
        let lanes = registry::load(&fuzz.join("campaign/targets.toml"))?;
        let registered: BTreeSet<String> = registry::targets(&lanes).into_iter().collect();
        let metadata: Metadata = serde_json::from_str(&output(
            Command::new("cargo")
                .args(["metadata", "--format-version", "1", "--no-deps"])
                .current_dir(&fuzz),
        )?)
        .map_err(|e| format!("cargo metadata: {e}"))?;
        let target_dir = metadata.target_directory;
        // The developer tool links the harness library; the runner does not.
        let build_dev = Command::new("cargo")
            .args(["build", "--release", "-p", "jolt-fuzz-dev"])
            .current_dir(&fuzz)
            .status()
            .map_err(|e| format!("build jolt-fuzz-dev: {e}"))?;
        if !build_dev.success() {
            return Err("cargo build -p jolt-fuzz-dev failed".into());
        }
        let dev = target_dir.join("release/jolt-fuzz-dev");
        let library: BTreeSet<String> = output(Command::new(&dev).arg("list"))?
            .lines()
            .map(str::to_string)
            .collect();
        if registered != library {
            return Err(format!(
                "target sets disagree:\n  campaign/targets.toml: {registered:?}\n  jolt_akita_fuzz::targets::ALL: {library:?}"
            ));
        }

        let rustc = output(Command::new("rustc").arg("-vV").current_dir(&fuzz))?;
        let triple = rustc
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .map(str::to_string)
            .ok_or("rustc -vV reported no host")?;
        let (sanitizer, trace_compares) = if self.skip_build {
            read_json::<BuildInfo>(&self.dist.join("BUILD-INFO.json"))
                .map_or((None, None), |previous| {
                    (previous.sanitizer, previous.trace_compares)
                })
        } else {
            let mut build = Command::new("cargo");
            build
                .args([
                    "fuzz",
                    "build",
                    "--release",
                    "--debug-assertions",
                    "--sanitizer",
                    &self.sanitizer,
                    "--target",
                    &triple,
                    BINARY,
                ])
                .current_dir(&fuzz);
            if !self.trace_compares {
                build.arg("--no-trace-compares");
            }
            println!("+ {build:?}");
            let status = build
                .status()
                .map_err(|e| format!("cargo fuzz build: {e}"))?;
            if !status.success() {
                return Err("cargo fuzz build failed".into());
            }
            (Some(self.sanitizer.clone()), Some(self.trace_compares))
        };

        for sub in [
            "bin",
            "campaign",
            "artifacts/schedules",
            "artifacts/guests",
            "artifacts/bundles",
            "seeds",
        ] {
            std::fs::create_dir_all(dist.join(sub)).map_err(|e| e.to_string())?;
        }
        let built = target_dir.join(&triple).join("release");
        std::fs::copy(built.join(BINARY), dist.join("bin").join(BINARY)).map_err(|e| {
            format!("copy instrumented {BINARY}: {e} (was `cargo fuzz build` run?)")
        })?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        std::fs::copy(&exe, dist.join("jolt-fuzz")).map_err(|e| format!("copy runner: {e}"))?;
        std::fs::copy(
            fuzz.join("campaign/targets.toml"),
            dist.join("campaign/targets.toml"),
        )
        .map_err(|e| e.to_string())?;
        std::fs::copy(fuzz.join("README.md"), dist.join("README.md"))
            .map_err(|e| format!("copy README: {e}"))?;
        copy_dir(
            &repo.join("crates/jolt-akita/schedules"),
            &dist.join("artifacts/schedules"),
        )?;
        let symbolizer = which("llvm-symbolizer");
        if let Some(path) = &symbolizer {
            std::fs::copy(path, dist.join("bin/llvm-symbolizer"))
                .map_err(|e| format!("copy symbolizer: {e}"))?;
        }

        let run_dev = |args: &[&OsStr], what: &str| -> Result<(), String> {
            let status = Command::new(&dev)
                .args(args)
                .env("JOLT_FUZZ_ARTIFACTS", dist.join("artifacts/schedules"))
                .env("JOLT_FUZZ_GUESTS", dist.join("artifacts/guests"))
                .status()
                .map_err(|e| format!("jolt-fuzz-dev {what}: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("jolt-fuzz-dev {what} failed"))
            }
        };
        // Guests are built with the `jolt` CLI; bundles are honest proofs of them.
        println!("building guests");
        run_dev(
            &[
                "build-guests".as_ref(),
                dist.join("artifacts/guests").as_os_str(),
            ],
            "build-guests",
        )?;
        println!("proving honest verifier bundles");
        run_dev(
            &[
                "bundles".as_ref(),
                dist.join("artifacts/bundles").as_os_str(),
            ],
            "bundles",
        )?;
        // Seeds use the packaged artifacts so their case selectors match the
        // catalogs the campaign will load.
        println!("generating seeds");
        run_dev(&["seeds".as_ref(), dist.join("seeds").as_os_str()], "seeds")?;

        let git = |args: &[&str]| {
            output(Command::new("git").args(args).current_dir(&repo)).unwrap_or_default()
        };
        let mut manifest = String::new();
        for file in files(dist) {
            manifest.push_str(&format!(
                "{}  {}\n",
                sha256_file(&dist.join(&file))?,
                file.display()
            ));
        }
        let info = BuildInfo {
            build_id: sha256_hex(manifest.as_bytes())[..16].to_string(),
            created: now(),
            git_commit: git(&["rev-parse", "HEAD"]),
            git_branch: git(&["rev-parse", "--abbrev-ref", "HEAD"]),
            git_dirty: !git(&["status", "--porcelain", "--untracked-files=no"]).is_empty(),
            rustc,
            cargo_fuzz: output(
                Command::new("cargo")
                    .args(["fuzz", "--version"])
                    .current_dir(&fuzz),
            )
            .unwrap_or_default(),
            target_triple: triple,
            sanitizer,
            trace_compares,
            flags:
                "cargo fuzz build --release --debug-assertions (profile: debug=1, overflow-checks)"
                    .into(),
            cargo_lock_sha256: sha256_file(&fuzz.join("Cargo.lock"))?,
            symbolizer,
            targets: registered,
            build_host: Machine::current(),
        };
        let info_path = dist.join("BUILD-INFO.json");
        write_json(&info_path, &info).map_err(|e| e.to_string())?;
        manifest.push_str(&format!("{}  BUILD-INFO.json\n", sha256_file(&info_path)?));
        std::fs::write(dist.join("MANIFEST.sha256"), &manifest).map_err(|e| e.to_string())?;
        Ok(info)
    }
}
