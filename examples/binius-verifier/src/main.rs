use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use jolt_sdk::MemoryConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

const CONSTRAINT_SYSTEM_FILE: &str = "constraint_system.bin";
const PUBLIC_FILE: &str = "public.bin";
const NON_PUBLIC_FILE: &str = "non_public.bin";
const PROOF_FILE: &str = "proof.bin";
const LOG_INV_RATE_FILE: &str = "log_inv_rate.txt";
const DEFAULT_TRACE_FILE: &str = "/tmp/binius-verifier.trace";
const DEFAULT_LOG_INV_RATE: u32 = 1;
const DEFAULT_KECCAK_MESSAGE_LEN: usize = 1 << 20;

const OUTPUT_SIZE: u64 = 4096;
const STACK_SIZE: u64 = 33_554_432;
const HEAP_SIZE: u64 = 268_435_456;
const MAX_TRACE_LENGTH: u64 = 134_217_728;
const INPUT_SIZE_SLACK: usize = 1024;
const MIN_INPUT_SIZE: usize = 4096;
const STATUS_OK: u32 = 1;
const LARGE_BLOB_WARNING_BYTES: usize = 128 * 1024 * 1024;

fn get_guest_src_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let guest_src_dir = manifest_dir.join("guest").join("src");

    guest_src_dir.canonicalize().unwrap_or(guest_src_dir)
}

fn get_binius_manifest_path() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let binius_manifest = manifest_dir.join("../../../binius64").join("Cargo.toml");

    binius_manifest.canonicalize().unwrap_or(binius_manifest)
}

#[derive(Parser)]
#[command(author, version, about = "Trace a Binius verifier inside a Jolt guest")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum BiniusExample {
    Sha256,
    Keccak,
}

impl BiniusExample {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Keccak => "keccak",
        }
    }

    fn max_len_flag(self) -> &'static str {
        match self {
            Self::Sha256 | Self::Keccak => "--max-len-bytes",
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Binius artifacts and a proof in a sibling `binius64` checkout
    Generate {
        /// Directory where generated artifacts should be written
        #[arg(long, value_name = "DIRECTORY")]
        artifacts_dir: PathBuf,
        /// Binius example to generate
        #[arg(long, value_enum, default_value_t = BiniusExample::Keccak)]
        example: BiniusExample,
        /// Random message length in bytes for the Binius example
        #[arg(long, value_name = "BYTES", default_value_t = DEFAULT_KECCAK_MESSAGE_LEN)]
        message_len: usize,
        /// Circuit capacity in bytes, defaults to `message_len`
        #[arg(long, value_name = "BYTES")]
        max_len_bytes: Option<usize>,
        /// Inverse-rate log used by the Binius prover and verifier
        #[arg(long, value_name = "LOG_INV_RATE", default_value_t = DEFAULT_LOG_INV_RATE)]
        log_inv_rate: u32,
    },
    /// Trace guest-side Binius verification from prebuilt artifacts
    Trace {
        /// Directory containing Binius verifier artifacts
        #[arg(long, value_name = "DIRECTORY")]
        artifacts_dir: PathBuf,
        /// Override the inverse-rate log used by Binius verifier setup
        #[arg(long, value_name = "LOG_INV_RATE")]
        log_inv_rate: Option<u32>,
        /// Write the full execution trace to disk instead of running measurement-only mode
        #[arg(short = 'd', long = "disk", default_value_t = false)]
        trace_to_file: bool,
        /// Output path used when `--disk` is enabled
        #[arg(long, value_name = "PATH")]
        trace_file: Option<PathBuf>,
        /// Guest heap size in bytes
        #[arg(long, value_name = "BYTES", default_value_t = HEAP_SIZE)]
        heap_size: u64,
        /// Guest stack size in bytes
        #[arg(long, value_name = "BYTES", default_value_t = STACK_SIZE)]
        stack_size: u64,
        /// Guest output buffer size in bytes
        #[arg(long, value_name = "BYTES", default_value_t = OUTPUT_SIZE)]
        max_output_size: u64,
        /// Max trace length compiled into the guest macro
        #[arg(long, value_name = "CYCLES", default_value_t = MAX_TRACE_LENGTH)]
        max_trace_length: u64,
    },
}

struct ArtifactBundle {
    log_inv_rate: u32,
    guest_input_bytes: Vec<u8>,
    packed_blob_len: usize,
    constraint_system_len: usize,
    public_len: usize,
    proof_len: usize,
}

fn status_message(status: u32) -> &'static str {
    match status {
        STATUS_OK => "verification succeeded",
        10 => "failed to deserialize log_inv_rate",
        11 => "failed to deserialize constraint system",
        12 => "failed to deserialize public values",
        13 => "failed to deserialize proof",
        20 => "artifact blob has trailing input bytes",
        21 => "proof challenger type did not match StdChallenger",
        30 => "Binius verifier setup failed inside the guest",
        40 => "Binius proof verification failed inside the guest",
        _ => "unknown guest status",
    }
}

fn artifact_path(artifacts_dir: &Path, file_name: &str) -> PathBuf {
    artifacts_dir.join(file_name)
}

fn run_command(program: &str, args: &[String]) -> Result<()> {
    info!("Running command: {} {}", program, args.join(" "));
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("failed to spawn `{program}`"))?;

    if !status.success() {
        bail!("command `{program}` failed with status {status}");
    }

    Ok(())
}

fn write_log_inv_rate(artifacts_dir: &Path, log_inv_rate: u32) -> Result<()> {
    let path = artifact_path(artifacts_dir, LOG_INV_RATE_FILE);
    fs::write(&path, format!("{log_inv_rate}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn generate_artifacts(
    artifacts_dir: &Path,
    example: BiniusExample,
    message_len: usize,
    max_len_bytes: Option<usize>,
    log_inv_rate: u32,
) -> Result<()> {
    fs::create_dir_all(artifacts_dir)
        .with_context(|| format!("failed to create {}", artifacts_dir.display()))?;

    let max_len_bytes = max_len_bytes.unwrap_or(message_len);
    let manifest_path = get_binius_manifest_path();

    let constraint_system_path = artifact_path(artifacts_dir, CONSTRAINT_SYSTEM_FILE);
    let public_path = artifact_path(artifacts_dir, PUBLIC_FILE);
    let non_public_path = artifact_path(artifacts_dir, NON_PUBLIC_FILE);
    let proof_path = artifact_path(artifacts_dir, PROOF_FILE);

    let save_args = vec![
        "run".to_string(),
        "--release".to_string(),
        "--manifest-path".to_string(),
        manifest_path.display().to_string(),
        "-p".to_string(),
        "binius-examples".to_string(),
        "--example".to_string(),
        example.as_str().to_string(),
        "--".to_string(),
        "save".to_string(),
        "--message-len".to_string(),
        message_len.to_string(),
        example.max_len_flag().to_string(),
        max_len_bytes.to_string(),
        "--cs-path".to_string(),
        constraint_system_path.display().to_string(),
        "--pub-witness-path".to_string(),
        public_path.display().to_string(),
        "--non-pub-data-path".to_string(),
        non_public_path.display().to_string(),
    ];
    run_command("cargo", &save_args)?;

    let prover_args = vec![
        "run".to_string(),
        "--release".to_string(),
        "--manifest-path".to_string(),
        manifest_path.display().to_string(),
        "-p".to_string(),
        "binius-examples".to_string(),
        "--bin".to_string(),
        "prover".to_string(),
        "--".to_string(),
        "--cs-path".to_string(),
        constraint_system_path.display().to_string(),
        "--pub-witness-path".to_string(),
        public_path.display().to_string(),
        "--non-pub-data-path".to_string(),
        non_public_path.display().to_string(),
        "--proof-path".to_string(),
        proof_path.display().to_string(),
        "--log-inv-rate".to_string(),
        log_inv_rate.to_string(),
    ];
    run_command("cargo", &prover_args)?;

    let verifier_args = vec![
        "run".to_string(),
        "--release".to_string(),
        "--manifest-path".to_string(),
        manifest_path.display().to_string(),
        "-p".to_string(),
        "binius-examples".to_string(),
        "--bin".to_string(),
        "verifier".to_string(),
        "--".to_string(),
        "--cs-path".to_string(),
        constraint_system_path.display().to_string(),
        "--pub-witness-path".to_string(),
        public_path.display().to_string(),
        "--proof-path".to_string(),
        proof_path.display().to_string(),
        "--log-inv-rate".to_string(),
        log_inv_rate.to_string(),
    ];
    run_command("cargo", &verifier_args)?;

    write_log_inv_rate(artifacts_dir, log_inv_rate)?;
    info!(
        "Generated {} artifacts in {} with message_len={} and max_len_bytes={}",
        example.as_str(),
        artifacts_dir.display(),
        message_len,
        max_len_bytes
    );

    Ok(())
}

fn resolve_log_inv_rate(artifacts_dir: &Path, cli_override: Option<u32>) -> Result<u32> {
    if let Some(log_inv_rate) = cli_override {
        return Ok(log_inv_rate);
    }

    let path = artifact_path(artifacts_dir, LOG_INV_RATE_FILE);
    let raw = fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read {} and no --log-inv-rate override was provided",
            path.display()
        )
    })?;

    raw.trim()
        .parse::<u32>()
        .with_context(|| format!("failed to parse {} as a u32 log_inv_rate", path.display()))
}

fn load_artifacts(artifacts_dir: &Path, cli_log_inv_rate: Option<u32>) -> Result<ArtifactBundle> {
    let log_inv_rate = resolve_log_inv_rate(artifacts_dir, cli_log_inv_rate)?;

    let constraint_system_path = artifact_path(artifacts_dir, CONSTRAINT_SYSTEM_FILE);
    let public_path = artifact_path(artifacts_dir, PUBLIC_FILE);
    let proof_path = artifact_path(artifacts_dir, PROOF_FILE);

    let constraint_system_bytes = fs::read(&constraint_system_path).with_context(|| {
        format!(
            "failed to read constraint system artifact from {}",
            constraint_system_path.display()
        )
    })?;
    let public_bytes = fs::read(&public_path).with_context(|| {
        format!(
            "failed to read public witness artifact from {}",
            public_path.display()
        )
    })?;
    let proof_bytes = fs::read(&proof_path).with_context(|| {
        format!(
            "failed to read proof artifact from {}",
            proof_path.display()
        )
    })?;

    let mut packed_blob = Vec::with_capacity(
        4 + constraint_system_bytes.len() + public_bytes.len() + proof_bytes.len(),
    );
    packed_blob.extend_from_slice(&log_inv_rate.to_le_bytes());
    packed_blob.extend_from_slice(&constraint_system_bytes);
    packed_blob.extend_from_slice(&public_bytes);
    packed_blob.extend_from_slice(&proof_bytes);
    let guest_input_bytes = postcard::to_stdvec(&packed_blob.as_slice())
        .context("failed to postcard-serialize the packed Binius artifact blob")?;

    Ok(ArtifactBundle {
        log_inv_rate,
        guest_input_bytes,
        packed_blob_len: packed_blob.len(),
        constraint_system_len: constraint_system_bytes.len(),
        public_len: public_bytes.len(),
        proof_len: proof_bytes.len(),
    })
}

fn calculate_max_input_size(input_len: usize) -> u64 {
    let desired = input_len
        .saturating_add(INPUT_SIZE_SLACK)
        .max(MIN_INPUT_SIZE);

    u64::try_from(desired).unwrap_or(u64::MAX)
}

fn guest_memory_config(
    input_len: usize,
    heap_size: u64,
    stack_size: u64,
    max_output_size: u64,
) -> MemoryConfig {
    MemoryConfig {
        max_input_size: calculate_max_input_size(input_len),
        max_output_size,
        max_untrusted_advice_size: 0,
        max_trusted_advice_size: 0,
        heap_size,
        stack_size,
        program_size: None,
    }
}

fn generate_provable_macro(
    memory_config: MemoryConfig,
    max_trace_length: u64,
    guest_src_dir: &Path,
) -> Result<()> {
    let provable_macro_path = guest_src_dir.join("provable_macro.rs");
    let macro_content = format!(
        r#"macro_rules! provable_with_config {{
    ($item: item) => {{
        #[jolt::provable(
            max_input_size = {},
            max_output_size = {},
            max_untrusted_advice_size = {},
            max_trusted_advice_size = {},
            heap_size = {},
            stack_size = {},
            max_trace_length = {}
        )]
        $item
    }};
}}"#,
        memory_config.max_input_size,
        memory_config.max_output_size,
        memory_config.max_untrusted_advice_size,
        memory_config.max_trusted_advice_size,
        memory_config.heap_size,
        memory_config.stack_size,
        max_trace_length
    );

    fs::write(&provable_macro_path, macro_content).with_context(|| {
        format!(
            "failed to write generated provable macro to {}",
            provable_macro_path.display()
        )
    })
}

fn trace_guest_verifier(
    artifacts: &ArtifactBundle,
    trace_to_file: bool,
    trace_file: Option<PathBuf>,
    heap_size: u64,
    stack_size: u64,
    max_output_size: u64,
    max_trace_length: u64,
) -> Result<()> {
    let guest_src_dir = get_guest_src_dir();
    let mut memory_config = guest_memory_config(
        artifacts.guest_input_bytes.len(),
        heap_size,
        stack_size,
        max_output_size,
    );
    generate_provable_macro(memory_config, max_trace_length, &guest_src_dir)?;

    info!(
        "Loaded Binius artifacts: log_inv_rate={}, constraint_system={} bytes, public={} bytes, proof={} bytes, packed_blob={} bytes, guest_input={} bytes",
        artifacts.log_inv_rate,
        artifacts.constraint_system_len,
        artifacts.public_len,
        artifacts.proof_len,
        artifacts.packed_blob_len,
        artifacts.guest_input_bytes.len()
    );
    if artifacts.packed_blob_len >= LARGE_BLOB_WARNING_BYTES {
        info!(
            "Large artifact blob detected ({} MiB). Guest-side deserialization can dominate runtime for very large examples.",
            artifacts.packed_blob_len / (1024 * 1024)
        );
    }
    info!(
        "Tracing with memory_config: input={}, output={}, heap={}, stack={}, max_trace_length={}",
        memory_config.max_input_size,
        memory_config.max_output_size,
        memory_config.heap_size,
        memory_config.stack_size,
        max_trace_length
    );

    let mut program = jolt_sdk::host::Program::new("binius-verifier-guest");
    program.set_func("verify_binius_proof");
    program.set_std(true);
    program.set_memory_config(memory_config);
    program.build(jolt_sdk::host::DEFAULT_TARGET_DIR);

    let elf_contents = program
        .get_elf_contents()
        .context("guest build succeeded but ELF contents were unavailable")?;
    let (_, _, program_size, _) = jolt_sdk::guest::program::decode(&elf_contents);

    memory_config.program_size = Some(program_size);
    let mut guest_program = jolt_sdk::guest::program::Program::new(&elf_contents, &memory_config);
    guest_program.elf = program.elf;

    let guest_output = if trace_to_file {
        let trace_path = trace_file.unwrap_or_else(|| PathBuf::from(DEFAULT_TRACE_FILE));
        let (_, io_device) =
            guest_program.trace_to_file(&artifacts.guest_input_bytes, &[], &[], &trace_path);
        info!("Trace written to {}", trace_path.display());
        postcard::from_bytes::<u32>(&io_device.outputs)
            .context("failed to decode guest output after trace-to-file")?
    } else {
        let (_, io_device) = guest_program.run(&artifacts.guest_input_bytes, &[], &[]);
        postcard::from_bytes::<u32>(&io_device.outputs)
            .context("failed to decode guest output after trace-only execution")?
    };

    if guest_output != STATUS_OK {
        bail!(
            "guest-side Binius verification failed with status {} ({})",
            guest_output,
            status_message(guest_output)
        );
    }

    info!("Guest-side Binius verification succeeded");
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Generate {
            artifacts_dir,
            example,
            message_len,
            max_len_bytes,
            log_inv_rate,
        } => generate_artifacts(
            &artifacts_dir,
            example,
            message_len,
            max_len_bytes,
            log_inv_rate,
        ),
        Commands::Trace {
            artifacts_dir,
            log_inv_rate,
            trace_to_file,
            trace_file,
            heap_size,
            stack_size,
            max_output_size,
            max_trace_length,
        } => {
            let artifacts = load_artifacts(&artifacts_dir, log_inv_rate)?;
            trace_guest_verifier(
                &artifacts,
                trace_to_file,
                trace_file,
                heap_size,
                stack_size,
                max_output_size,
                max_trace_length,
            )
        }
    }
}
