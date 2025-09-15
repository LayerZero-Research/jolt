use clap::{Args, Parser, Subcommand, ValueEnum};

#[path = "../../benches/e2e_profiling.rs"]
mod e2e_profiling;
use e2e_profiling::{benchmarks, master_benchmark, BenchType};

use std::any::Any;

use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{self, prelude::*, EnvFilter};

struct GlogFormatter;

fn days_to_month_day(days_since_epoch: u64) -> (u8, u8) {
    // Simplified calendar calculation for current year only
    let mut remaining_days = days_since_epoch % 365; // Current year approximation
    let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let mut month = 1;
    for &days_in_month in &month_days {
        if remaining_days >= days_in_month as u64 {
            remaining_days -= days_in_month as u64;
            month += 1;
        } else {
            break;
        }
    }

    let day = remaining_days + 1; // Days are 1-indexed
    (month, day as u8)
}

impl<S, N> FormatEvent<S, N> for GlogFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let level = match *event.metadata().level() {
            tracing::Level::ERROR => "\x1b[31mE\x1b[0m",
            tracing::Level::WARN => "\x1b[33mW\x1b[0m",
            tracing::Level::INFO => "\x1b[32mI\x1b[0m",
            tracing::Level::DEBUG => "\x1b[36mD\x1b[0m",
            tracing::Level::TRACE => "\x1b[37mT\x1b[0m",
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let total_secs = now.as_secs();

        // Calculate days since epoch to get month/day
        let days_since_epoch = total_secs / 86400;
        let (month, day) = days_to_month_day(days_since_epoch);

        // Calculate time of day
        let day_secs = total_secs % 86400;
        let (hours, mins, secs) = (day_secs / 3600, day_secs % 3600 / 60, day_secs % 60);

        let thread_id = format!("{:?}", std::thread::current().id())
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(1);

        let filename = std::path::Path::new(event.metadata().file().unwrap_or("unknown"))
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown");

        write!(
            writer,
            "{}\x1b[90m{:02}{:02} {:02}:{:02}:{:02}.{:03} {} {}:{}]\x1b[0m ",
            level,
            month,
            day,
            hours,
            mins,
            secs,
            now.subsec_millis(),
            thread_id,
            filename,
            event.metadata().line().unwrap_or(0)
        )?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;
        write!(writer, "\n")?;

        Ok(())
    }
}

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Profile(ProfileArgs),
    Benchmark(BenchmarkArgs),
}

#[derive(Args, Debug, Clone)]
struct ProfileArgs {
    /// Output formats
    #[clap(short, long, value_enum)]
    format: Option<Vec<Format>>,

    /// Type of benchmark to run
    #[clap(long, value_enum)]
    name: BenchType,
}

#[derive(Args, Debug)]
struct BenchmarkArgs {
    #[clap(flatten)]
    profile_args: ProfileArgs,

    /// Max trace length to use (as 2^scale)
    #[clap(short, long, default_value_t = 20)]
    scale: usize,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum Format {
    Default,
    Chrome,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Profile(args) => trace_only(args),
        Commands::Benchmark(args) => benchmark_and_trace(args),
    }
}

fn trace(
    args: ProfileArgs,
    benchmark_fn: Vec<(tracing::Span, Box<dyn FnOnce()>)>,
    trace_file: Option<String>,
) {
    let mut layers = Vec::new();

    let log_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let log_layer = tracing_subscriber::fmt::layer()
        .event_format(GlogFormatter)
        .with_filter(log_filter.clone())
        .boxed();
    layers.push(log_layer);

    let mut guards: Vec<Box<dyn Any>> = vec![];

    if let Some(format) = &args.format {
        if format.contains(&Format::Chrome) {
            let (chrome_layer, guard) = if let Some(file) = &trace_file {
                ChromeLayerBuilder::new()
                    .file(file)
                    .include_args(true)
                    .build()
            } else {
                ChromeLayerBuilder::new().include_args(true).build()
            };
            layers.push(chrome_layer.boxed());
            guards.push(Box::new(guard));
            if trace_file.is_some() {
                tracing::info!("Running tracing-chrome. Files will be saved in benchmark-runs/perfetto_traces/ and can be viewed in https://ui.perfetto.dev/");
            } else {
                tracing::info!("Running tracing-chrome. Files will be saved as trace-<some timestamp>.json and can be viewed in https://ui.perfetto.dev/");
            }
        }
    }

    tracing_subscriber::registry().with(layers).init();
    for (span, bench) in benchmark_fn.into_iter() {
        span.to_owned().in_scope(|| {
            bench();
            tracing::info!("Bench Complete");
        });
    }
}

fn trace_only(args: ProfileArgs) {
    trace(args.clone(), benchmarks(args.name), None)
}

fn benchmark_and_trace(args: BenchmarkArgs) {
    // Generate trace filename
    let bench_name = match args.profile_args.name {
        BenchType::Fibonacci => "fibonacci",
        BenchType::Sha2Chain => "sha2_chain",
        BenchType::Sha3Chain => "sha3_chain",
        BenchType::Btreemap => "btreemap",
        BenchType::Sha2 => panic!("Use sha2-chain instead"),
        BenchType::Sha3 => panic!("Use sha3-chain instead"),
    };
    let trace_file = format!(
        "benchmark-runs/perfetto_traces/{}_{}.json",
        bench_name, args.scale
    );
    std::fs::create_dir_all("benchmark-runs/perfetto_traces").ok();

    trace(
        args.profile_args.clone(),
        master_benchmark(args.profile_args.name, args.scale),
        Some(trace_file),
    )
}
