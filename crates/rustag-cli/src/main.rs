//! RustAG command-line interface.
//!
//! `rustag start <name>` runs a stagenet (JSON-RPC + WS + REST). The other
//! commands either operate on the local stagenet registry (`create`, `list`) or
//! talk to a running stagenet's REST API (`airdrop`, `override`, `preload`,
//! `logs`, `status`).

mod commands;

use clap::{Parser, Subcommand};

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
}

#[derive(Subcommand)]
enum Command {
    /// Create a new staging environment.
    Create(commands::create::CreateArgs),
    /// Start a stagenet (runs the JSON-RPC, WebSocket, and REST servers).
    Start(commands::start::StartArgs),
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let cli = Cli::parse();
    match cli.command {
        Command::Create(args) => commands::create::run(args).await,
        Command::Start(args) => commands::start::run(args).await,
        Command::Stop(args) => commands::manage::stop(args).await,
        Command::Status(args) => commands::manage::status(args).await,
        Command::List => commands::manage::list().await,
        Command::Airdrop(args) => commands::airdrop::run(args).await,
        Command::Override(args) => commands::overrides::run(args).await,
        Command::Preload(args) => commands::preload::run(args).await,
        Command::Logs(args) => commands::logs::run(args).await,
    }
}

fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "rustag=info,rustag_core=info,rustag_rpc=info,tower_http=warn".into());
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}
