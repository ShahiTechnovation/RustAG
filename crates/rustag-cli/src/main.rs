//! RustAG command-line interface.
//!
//! `rustag start <name>` runs a stagenet (JSON-RPC + WS + REST). The other
//! commands either operate on the local stagenet registry (`create`, `list`) or
//! talk to a running stagenet's REST API (`airdrop`, `override`, `preload`,
//! `logs`, `status`).

mod commands;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "rustag",
    version,
    about = "A persistent, mainnet-mirroring staging environment for Solana programs",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Log output format. `json` emits structured logs for containers/log
    /// aggregators; `text` is human-readable (default).
    #[arg(long, value_enum, default_value_t = LogFormat::Text, env = "RUSTAG_LOG_FORMAT", global = true)]
    log_format: LogFormat,
}

/// Tracing output format selected by `--log-format` / `RUSTAG_LOG_FORMAT`.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum LogFormat {
    /// Human-readable, colorized output.
    Text,
    /// Newline-delimited JSON (one object per event).
    Json,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new staging environment.
    Create(commands::create::CreateArgs),
    /// Start a stagenet (runs the JSON-RPC, WebSocket, and REST servers).
    Start(commands::start::StartArgs),
    /// Create-if-needed then serve - the one-shot entrypoint for hosted demos.
    Serve(commands::serve::ServeArgs),
    /// Stop a running stagenet (best-effort via its PID file).
    Stop(commands::manage::StopArgs),
    /// Show a stagenet's status.
    Status(commands::manage::StatusArgs),
    /// List all stagenets.
    List,
    /// Airdrop SOL to a wallet (requires the stagenet to be running).
    Airdrop(commands::airdrop::AirdropArgs),
    /// Override account state (requires the stagenet to be running).
    Override(commands::overrides::OverrideArgs),
    /// Preload known mainnet programs/oracles (requires the stagenet running).
    Preload(commands::preload::PreloadArgs),
    /// Tail the transaction log.
    Logs(commands::logs::LogsArgs),
    /// Manage recurring on-chain activities (Phase 2 scheduler).
    Schedule(commands::schedule::ScheduleArgs),
    /// Show analytics time-series for a stagenet (Phase 2).
    Metrics(commands::metrics::MetricsArgs),
    /// Run preflight diagnostics (DB writable, mainnet reachable, ports free).
    Doctor(commands::doctor::DoctorArgs),
    /// Produce a signed, verifiable attestation of staged state (Phase 3).
    Attest(commands::attest::AttestArgs),
    /// Verify a staging attestation offline (Phase 3).
    Verify(commands::verify::VerifyArgs),
    /// Scan recorded transactions for exploit signatures (Phase 3).
    Scan(commands::scan::ScanArgs),
    /// Build an off-chain concurrent Merkle tree and print its root (Phase 3).
    Tree(commands::tree::TreeArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse before initializing tracing so `--log-format` takes effect for all
    // logs (and so `--help`/`--version` exit cleanly without a logger).
    let cli = Cli::parse();
    init_tracing(cli.log_format);
    match cli.command {
        Command::Create(args) => commands::create::run(args).await,
        Command::Start(args) => commands::start::run(args).await,
        Command::Serve(args) => commands::serve::run(args).await,
        Command::Stop(args) => commands::manage::stop(args).await,
        Command::Status(args) => commands::manage::status(args).await,
        Command::List => commands::manage::list().await,
        Command::Airdrop(args) => commands::airdrop::run(args).await,
        Command::Override(args) => commands::overrides::run(args).await,
        Command::Preload(args) => commands::preload::run(args).await,
        Command::Logs(args) => commands::logs::run(args).await,
        Command::Schedule(args) => commands::schedule::run(args).await,
        Command::Metrics(args) => commands::metrics::run(args).await,
        Command::Doctor(args) => commands::doctor::run(args).await,
        Command::Attest(args) => commands::attest::run(args).await,
        Command::Verify(args) => commands::verify::run(args).await,
        Command::Scan(args) => commands::scan::run(args).await,
        Command::Tree(args) => commands::tree::run(args).await,
    }
}

fn init_tracing(format: LogFormat) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "rustag=info,rustag_core=info,rustag_rpc=info,tower_http=warn".into());
    let registry = tracing_subscriber::registry().with(filter);
    match format {
        LogFormat::Json => registry
            .with(tracing_subscriber::fmt::layer().json().with_target(false))
            .init(),
        LogFormat::Text => registry
            .with(tracing_subscriber::fmt::layer().with_target(false))
            .init(),
    }
}
