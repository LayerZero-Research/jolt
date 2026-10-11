//! Persistent campaign directory: lock, identity, state, and bounded logs.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env::consts::{ARCH, OS};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Result as IoResult, Seek, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> IoResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_file_name(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("state"),
        std::process::id()
    ));
    let mut file = File::create(&tmp)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(tmp, path)
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

pub fn hostname() -> String {
    let mut buffer = [0u8; 256];
    // SAFETY: the buffer is valid for its full length.
    let ok = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) } == 0;
    if ok {
        let end = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
        String::from_utf8_lossy(&buffer[..end]).into_owned()
    } else {
        "unknown".into()
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn which(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|path| path.is_file())
    })
}

/// Recreate `from` under `to`, hard-linking files when `link` (falling back
/// to a copy) and copying them otherwise. Entries removed while the walk
/// runs are skipped, so it is safe on directories a campaign is writing.
pub fn snapshot(from: &Path, to: &Path, link: bool) -> IoResult<()> {
    fs::create_dir_all(to)?;
    let entries = match fs::read_dir(from) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        entries => entries?,
    };
    for entry in entries.flatten() {
        let (source, target) = (entry.path(), to.join(entry.file_name()));
        let result = match entry.file_type() {
            Ok(kind) if kind.is_dir() => snapshot(&source, &target, link),
            Ok(_) if link => {
                fs::hard_link(&source, &target).or_else(|_| fs::copy(&source, &target).map(drop))
            }
            Ok(_) => fs::copy(&source, &target).map(drop),
            Err(error) => Err(error),
        };
        match result {
            Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
            _ => {}
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Machine {
    pub hostname: String,
    /// SHA-256 of the systemd machine id, which `machine-id(5)` asks not to
    /// expose verbatim.
    pub machine_id_sha256: Option<String>,
    pub os: String,
    pub arch: String,
    pub cpu_model: String,
}

impl Machine {
    pub fn current() -> Self {
        let machine_id_sha256 = ["/etc/machine-id", "/var/lib/dbus/machine-id"]
            .iter()
            .find_map(|path| fs::read_to_string(path).ok())
            .map(|id| sha256_hex(id.trim().as_bytes()));
        let cpu_model = fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|text| {
                text.lines()
                    .find(|line| line.starts_with("model name"))
                    .and_then(|line| line.split_once(':'))
                    .map(|(_, model)| model.trim().to_string())
            })
            .unwrap_or_default();
        Self {
            hostname: hostname(),
            machine_id_sha256,
            os: OS.into(),
            arch: ARCH.into(),
            cpu_model,
        }
    }

    fn same_host(&self, other: &Self) -> bool {
        self.machine_id_sha256 == other.machine_id_sha256 && self.hostname == other.hostname
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Adoption {
    pub at: u64,
    pub machine: Machine,
}

/// `campaign.json`: the identity an output directory keeps across sessions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Campaign {
    pub campaign_id: String,
    pub created: u64,
    pub machine: Machine,
    pub builds: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adopted_by: Vec<Adoption>,
}

/// Layout of one campaign output directory.
pub struct Store {
    pub root: PathBuf,
    pub corpus: PathBuf,
    pub findings: PathBuf,
    pub quarantine: PathBuf,
    pub logs: PathBuf,
    pub stats: PathBuf,
    pub exports: PathBuf,
    /// Scratch space for workers (`TMPDIR`), so no campaign file lands in
    /// the system temporary directory.
    pub tmp: PathBuf,
    pub state_path: PathBuf,
    pub live_path: PathBuf,
    pub campaign_path: PathBuf,
    lock: Option<File>,
}

impl Store {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            corpus: root.join("corpus"),
            findings: root.join("findings"),
            quarantine: root.join("quarantine"),
            logs: root.join("logs"),
            stats: root.join("stats"),
            exports: root.join("exports"),
            tmp: root.join("tmp"),
            state_path: root.join("state.json"),
            live_path: root.join("live.json"),
            campaign_path: root.join("campaign.json"),
            lock: None,
        }
    }

    pub fn ensure(&self) -> IoResult<()> {
        for dir in [
            &self.corpus,
            &self.findings,
            &self.quarantine,
            &self.logs,
            &self.stats,
            &self.exports,
            &self.tmp,
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    /// Exclusive advisory lock; a second runner on the same directory fails.
    pub fn lock(&mut self) -> Result<(), String> {
        fs::create_dir_all(&self.root)
            .map_err(|e| format!("create {}: {e}", self.root.display()))?;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(".lock"))
            .map_err(|e| format!("open lock: {e}"))?;
        if file.try_lock().is_err() {
            let mut holder = String::new();
            let _ = file.read_to_string(&mut holder);
            return Err(format!(
                "{} is in use by another runner ({})",
                self.root.display(),
                holder.trim()
            ));
        }
        file.set_len(0).map_err(|e| e.to_string())?;
        file.rewind().map_err(|e| e.to_string())?;
        writeln!(
            file,
            "pid={} host={} since={}",
            std::process::id(),
            hostname(),
            now()
        )
        .map_err(|e| e.to_string())?;
        self.lock = Some(file);
        Ok(())
    }

    pub fn is_locked(&self) -> bool {
        match File::open(self.root.join(".lock")) {
            Ok(file) => match file.try_lock_shared() {
                Ok(()) => {
                    let _ = file.unlock();
                    false
                }
                Err(_) => true,
            },
            Err(_) => false,
        }
    }

    /// Create or resume the campaign identity; refuse silent machine changes.
    pub fn campaign(&self, build_id: &str, adopt: bool) -> Result<Campaign, String> {
        let machine = Machine::current();
        let path = &self.campaign_path;
        let mut campaign = match fs::read(path) {
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let mut random = [0u8; 6];
                File::open("/dev/urandom")
                    .and_then(|mut file| file.read_exact(&mut random))
                    .map_err(|e| format!("read /dev/urandom: {e}"))?;
                let campaign = Campaign {
                    campaign_id: random.iter().map(|b| format!("{b:02x}")).collect(),
                    created: now(),
                    machine,
                    builds: vec![build_id.to_string()],
                    adopted_by: Vec::new(),
                };
                write_json(path, &campaign).map_err(|e| e.to_string())?;
                return Ok(campaign);
            }
            Err(error) => return Err(format!("read {}: {error}", path.display())),
            Ok(bytes) => serde_json::from_slice::<Campaign>(&bytes)
                .map_err(|e| format!("parse {}: {e}", path.display()))?,
        };
        if !campaign.machine.same_host(&machine) {
            if !adopt {
                return Err(format!(
                    "{} was created on {}; pass --adopt to continue it here, or use a fresh --output",
                    self.root.display(),
                    campaign.machine.hostname,
                ));
            }
            campaign.adopted_by.push(Adoption {
                at: now(),
                machine: machine.clone(),
            });
            campaign.machine = machine;
        }
        if !campaign.builds.iter().any(|b| b == build_id) {
            campaign.builds.push(build_id.to_string());
        }
        write_json(path, &campaign).map_err(|e| e.to_string())?;
        Ok(campaign)
    }
}

/// Append-only text log capped at `max_bytes` with one rotated backup.
pub struct RotatingLog {
    path: PathBuf,
    file: File,
    written: u64,
    max_bytes: u64,
}

impl RotatingLog {
    pub fn open(path: &Path, max_bytes: u64) -> IoResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let written = file.metadata().map_or(0, |m| m.len());
        Ok(Self {
            path: path.to_path_buf(),
            file,
            written,
            max_bytes,
        })
    }

    pub fn line(&mut self, line: &str) {
        let _ = writeln!(self.file, "{line}");
        self.written += line.len() as u64 + 1;
        if self.written >= self.max_bytes {
            let backup = self.path.with_extension("log.1");
            let _ = fs::rename(&self.path, backup);
            if let Ok(file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
            {
                self.file = file;
                self.written = 0;
            }
        }
    }
}

pub fn directory_bytes(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => directory_bytes(&entry.path()),
            Ok(_) => entry.metadata().map_or(0, |m| m.len()),
            Err(_) => 0,
        })
        .sum()
}

/// Every file under `root`, relative to it, sorted.
pub fn files(root: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else if let Ok(relative) = path.strip_prefix(root) {
                out.push(relative.to_path_buf());
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

pub fn count_files(path: &Path) -> u64 {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
                .count() as u64
        })
        .unwrap_or(0)
}
