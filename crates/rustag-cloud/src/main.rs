//! `rustag-cloud` — the RustAG cloud control-plane server.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rustag-cloud",
    version,
    about = "Multi-tenant control plane that hosts RustAG stagenets",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the control-plane HTTP server (configured via environment).
    Serve,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => rustag_cloud::run(rustag_cloud::CloudConfig::from_env()).await,
    }
}

fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "rustag_cloud=info,tower_http=warn".into());
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}
