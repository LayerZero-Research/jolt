use clap::{Parser, ValueEnum};
use eyre::{bail, Context, Result};
use guest::{PreparedStatelessInput, ValidationOutput};
use jolt_sdk::{host::Program, F};
use stateless::{StatelessInput, UncompressedPublicKey};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(about = "Run the canonical stateless Ethereum validator with Jolt")]
struct Args {
    /// Path to an upstream-style JSON fixture, raw StatelessInput JSON, or
    /// prepared postcard input. Directories are treated as fixture sets for
    /// tracing mode.
    input: PathBuf,

    /// Whether to trace or run full prove+verify.
    #[arg(long, value_enum, default_value_t = Mode::Analyze)]
    mode: Mode,

    /// Directory used by Jolt to cache compiled guest artifacts.
    #[arg(long, default_value = "/tmp/jolt-guest-targets")]
    target_dir: PathBuf,

    /// Optional output path for the prepared postcard input. Only valid for a
    /// single input file.
    #[arg(long)]
    write_prepared: Option<PathBuf>,

    /// Limit the number of fixtures traced when the input is a directory.
    #[arg(long)]
    limit: Option<usize>,

    /// Only trace fixtures whose file name contains one of these substrings.
    #[arg(long)]
    filter: Vec<String>,

    /// Continue tracing remaining fixtures after a per-file failure.
    #[arg(long, default_value_t = false)]
    keep_going: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Mode {
    Analyze,
    Prove,
}

#[derive(Debug, serde::Deserialize)]
struct StatelessValidatorFixture {
    name: String,
    stateless_input: StatelessInput,
    success: bool,
}

#[derive(Debug)]
struct LoadedInput {
    name: String,
    expected_success: Option<bool>,
    prepared: PreparedStatelessInput,
}

#[derive(Debug, Clone)]
struct AnalysisResult {
    name: String,
    cycles: usize,
    padded_cycles: usize,
    success: bool,
}

fn compile_guest(target_dir: &Path) -> Program {
    let target_dir = target_dir.to_string_lossy();
    let compile_start = Instant::now();
    let program = guest::compile_stateless_validate(&target_dir);
    info!("guest compile time: {:?}", compile_start.elapsed());
    program
}

fn recover_signers(stateless_input: &StatelessInput) -> Result<Vec<UncompressedPublicKey>> {
    stateless_input
        .block
        .body
        .transactions
        .iter()
        .enumerate()
        .map(|(i, tx)| {
            let key = tx
                .signature()
                .recover_from_prehash(&tx.signature_hash())
                .with_context(|| format!("failed to recover signer for tx #{i}"))?;
            let encoded = key.to_encoded_point(false);
            let public_key: [u8; 65] = encoded
                .as_bytes()
                .try_into()
                .map_err(|_| eyre::eyre!("unexpected uncompressed public key length"))?;
            Ok(UncompressedPublicKey(public_key))
        })
        .collect()
}

fn load_json_input(bytes: &[u8]) -> Result<LoadedInput> {
    if let Ok(fixture) = serde_json::from_slice::<StatelessValidatorFixture>(bytes) {
        let prepared = PreparedStatelessInput {
            public_keys: recover_signers(&fixture.stateless_input)?,
            stateless_input: fixture.stateless_input,
        };
        return Ok(LoadedInput {
            name: fixture.name,
            expected_success: Some(fixture.success),
            prepared,
        });
    }

    let stateless_input: StatelessInput = serde_json::from_slice(bytes)
        .wrap_err("failed to parse JSON as an upstream fixture or StatelessInput")?;
    let name = format!("block-{}", hex::encode(stateless_input.block.hash_slow().0));
    let prepared = PreparedStatelessInput {
        public_keys: recover_signers(&stateless_input)?,
        stateless_input,
    };
    Ok(LoadedInput {
        name,
        expected_success: None,
        prepared,
    })
}

fn load_input(path: &Path) -> Result<LoadedInput> {
    let bytes =
        fs::read(path).wrap_err_with(|| format!("failed to read input {}", path.display()))?;
    if path.extension().and_then(std::ffi::OsStr::to_str) == Some("json") {
        return load_json_input(&bytes);
    }

    let prepared: PreparedStatelessInput =
        postcard::from_bytes(&bytes).wrap_err("failed to parse postcard input")?;
    let name = format!(
        "block-{}",
        hex::encode(prepared.stateless_input.block.hash_slow().0)
    );
    Ok(LoadedInput {
        name,
        expected_success: None,
        prepared,
    })
}

fn check_expected_success(expected_success: Option<bool>, output: ValidationOutput) -> Result<()> {
    if let Some(expected) = expected_success {
        if expected != output.success {
            bail!(
                "validation success mismatch: expected {expected}, got {}",
                output.success
            );
        }
    }
    Ok(())
}

fn analyze_single(
    program: &Program,
    name: &str,
    input_bytes: &[u8],
    expected_success: Option<bool>,
) -> Result<AnalysisResult> {
    let guest_input =
        postcard::to_stdvec(&input_bytes).wrap_err("failed to serialize guest input")?;
    let start = Instant::now();
    let summary = program.clone().trace_analyze::<F>(&guest_input, &[], &[]);
    let elapsed = start.elapsed();
    let total_cycles = summary.trace.len();
    let padded_cycles = if total_cycles == 0 {
        1
    } else {
        total_cycles.next_power_of_two()
    };
    let output: ValidationOutput =
        postcard::from_bytes(&summary.io_device.outputs).wrap_err("failed to decode output")?;
    check_expected_success(expected_success, output)?;

    info!(
        "analysis complete: name={name} cycles={total_cycles} padded_cycles={padded_cycles} success={} block_hash=0x{} elapsed={elapsed:?}",
        output.success,
        hex::encode(output.block_hash),
    );
    Ok(AnalysisResult {
        name: name.to_owned(),
        cycles: total_cycles,
        padded_cycles,
        success: output.success,
    })
}

fn supported_input_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("json") | Some("bin")
    )
}

fn collect_input_paths(
    input: &Path,
    filter: &[String],
    limit: Option<usize>,
) -> Result<Vec<PathBuf>> {
    if input.is_file() {
        return Ok(vec![input.to_path_buf()]);
    }

    if !input.is_dir() {
        bail!(
            "input path is neither a file nor a directory: {}",
            input.display()
        );
    }

    let mut paths: Vec<PathBuf> = fs::read_dir(input)
        .wrap_err_with(|| format!("failed to read directory {}", input.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .wrap_err_with(|| format!("failed to list directory {}", input.display()))?
        .into_iter()
        .filter(|path| path.is_file() && supported_input_file(path))
        .filter(|path| {
            if filter.is_empty() {
                return true;
            }
            let file_name = path.file_name().and_then(OsStr::to_str).unwrap_or_default();
            filter.iter().any(|needle| file_name.contains(needle))
        })
        .collect();
    paths.sort();

    if let Some(limit) = limit {
        paths.truncate(limit);
    }

    if paths.is_empty() {
        bail!(
            "no supported fixture files found in {} with the current filters",
            input.display()
        );
    }

    Ok(paths)
}

fn log_loaded_input(loaded: &LoadedInput, input_bytes: &[u8]) {
    let block_hash = loaded.prepared.stateless_input.block.hash_slow().0;
    let tx_count = loaded
        .prepared
        .stateless_input
        .block
        .body
        .transactions
        .len();
    info!(
        "loaded input: name={} block_hash=0x{} tx_count={} input_bytes={} expected_success={:?}",
        loaded.name,
        hex::encode(block_hash),
        tx_count,
        input_bytes.len(),
        loaded.expected_success,
    );
}

fn write_prepared_input(path: &Path, input_bytes: &[u8]) -> Result<()> {
    fs::write(path, input_bytes).wrap_err_with(|| format!("failed to write {}", path.display()))?;
    info!("wrote prepared input to {}", path.display());
    Ok(())
}

fn log_analysis_summary(results: &[AnalysisResult], elapsed: Duration) {
    if results.len() <= 1 {
        return;
    }

    let total_cycles = results
        .iter()
        .map(|result| result.cycles as u128)
        .sum::<u128>();
    let total_padded_cycles = results
        .iter()
        .map(|result| result.padded_cycles as u128)
        .sum::<u128>();
    let avg_cycles = total_cycles as f64 / results.len() as f64;
    let slowest = results
        .iter()
        .max_by_key(|result| result.cycles)
        .expect("results is non-empty");
    let fastest = results
        .iter()
        .min_by_key(|result| result.cycles)
        .expect("results is non-empty");
    let successes = results.iter().filter(|result| result.success).count();

    info!(
        "analysis summary: fixtures={} successes={} total_cycles={} total_padded_cycles={} avg_cycles={avg_cycles:.2} fastest={}({}) slowest={}({}) elapsed={elapsed:?}",
        results.len(),
        successes,
        total_cycles,
        total_padded_cycles,
        fastest.name,
        fastest.cycles,
        slowest.name,
        slowest.cycles,
    );
}

fn analyze_inputs(args: &Args) -> Result<()> {
    if args.input.is_dir() && args.write_prepared.is_some() {
        bail!("--write-prepared only supports a single input file");
    }

    let input_paths = collect_input_paths(&args.input, &args.filter, args.limit)?;
    let program = compile_guest(&args.target_dir);
    let batch_start = Instant::now();
    let mut results = Vec::with_capacity(input_paths.len());
    let mut first_error = None;

    for input_path in input_paths {
        let outcome = (|| -> Result<AnalysisResult> {
            let loaded = load_input(&input_path)?;
            let input_bytes = postcard::to_stdvec(&loaded.prepared)
                .wrap_err("failed to serialize prepared input")?;
            log_loaded_input(&loaded, &input_bytes);

            if let Some(path) = &args.write_prepared {
                write_prepared_input(path, &input_bytes)?;
            }

            analyze_single(
                &program,
                &loaded.name,
                &input_bytes,
                loaded.expected_success,
            )
        })();

        match outcome {
            Ok(result) => results.push(result),
            Err(error) => {
                if !args.keep_going {
                    return Err(error);
                }
                info!("analysis failed for {}: {error}", input_path.display());
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    log_analysis_summary(&results, batch_start.elapsed());

    if let Some(error) = first_error {
        return Err(error);
    }

    Ok(())
}

fn prove(
    target_dir: &Path,
    name: &str,
    input_bytes: &[u8],
    expected_success: Option<bool>,
) -> Result<()> {
    let target_dir = target_dir.to_string_lossy();
    let mut program = guest::compile_stateless_validate(&target_dir);
    let shared_preprocessing = guest::preprocess_shared_stateless_validate(&mut program)?;
    let prover_preprocessing =
        guest::preprocess_prover_stateless_validate(shared_preprocessing.clone());
    let verifier_setup = prover_preprocessing.generators.to_verifier_setup();
    let verifier_preprocessing =
        guest::preprocess_verifier_stateless_validate(shared_preprocessing, verifier_setup, None);

    let prove = guest::build_prover_stateless_validate(program, prover_preprocessing);
    let verify = guest::build_verifier_stateless_validate(verifier_preprocessing);

    let start = Instant::now();
    let (output, proof, program_io): (ValidationOutput, _, _) = prove(input_bytes);
    let prove_elapsed = start.elapsed();
    check_expected_success(expected_success, output)?;

    let start = Instant::now();
    let valid = verify(input_bytes, output, program_io.panic, proof);
    let verify_elapsed = start.elapsed();
    if !valid {
        bail!("proof verification failed");
    }

    info!(
        "prove+verify complete: name={name} success={} block_hash=0x{} prove_elapsed={prove_elapsed:?} verify_elapsed={verify_elapsed:?}",
        output.success,
        hex::encode(output.block_hash),
    );
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    match args.mode {
        Mode::Analyze => analyze_inputs(&args)?,
        Mode::Prove => {
            if args.input.is_dir() {
                bail!("prove mode only supports a single input file");
            }
            if args.limit.is_some() || !args.filter.is_empty() || args.keep_going {
                bail!("--limit, --filter, and --keep-going are only supported in analyze mode");
            }

            let loaded = load_input(&args.input)?;
            let input_bytes = postcard::to_stdvec(&loaded.prepared)
                .wrap_err("failed to serialize prepared input")?;
            log_loaded_input(&loaded, &input_bytes);

            if let Some(path) = &args.write_prepared {
                write_prepared_input(path, &input_bytes)?;
            }

            prove(
                &args.target_dir,
                &loaded.name,
                &input_bytes,
                loaded.expected_success,
            )?;
        }
    }

    Ok(())
}
