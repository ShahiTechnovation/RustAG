//! `rustag create` - register a new staging environment.

use std::sync::Arc;

use anyhow::{bail, Result};
use clap::Args;

use rustag_core::{Stagenet, StagenetConfig};

use super::{db_path, info, ok, open_store, short_id};

#[derive(Args)]
pub struct CreateArgs {
    /// Stagenet name (must be unique).
    pub name: String,
    /// JSON-RPC port.
    #[arg(long, default_value_t = 8899)]
    pub rpc_port: u16,
    /// WebSocket port (defaults to rpc_port + 1).
    #[arg(long)]
    pub ws_port: Option<u16>,
    /// REST API port.
    #[arg(long, default_value_t = 9000)]
    pub api_port: u16,
    /// Mainnet RPC endpoint for the lazy mirror.
    #[arg(long, env = "RUSTAG_MAINNET_RPC")]
    pub mainnet_rpc: Option<String>,
    /// Disable mainnet mirroring (fully offline stagenet).
    #[arg(long, default_value_t = false)]
    pub no_mirror: bool,
}

pub async fn run(args: CreateArgs) -> Result<()> {
    let store = open_store().await?;
    if store.get_stagenet_by_name(&args.name).await?.is_some() {
        bail!("stagenet '{}' already exists", args.name);
    }

    let config = StagenetConfig {
        name: args.name.clone(),
        rpc_port: args.rpc_port,
        ws_port: args
            .ws_port
            .unwrap_or_else(|| args.rpc_port.saturating_add(1)),
        api_port: args.api_port,
        mainnet_rpc: args
            .mainnet_rpc
            .unwrap_or_else(|| rustag_core::DEFAULT_MAINNET_RPC.to_string()),
        mirror_enabled: !args.no_mirror,
        db_path: db_path(),
        ..StagenetConfig::default()
    };

    let sn = Stagenet::create(config.clone(), Arc::clone(&store)).await?;

    ok(format!(
        "Created stagenet '{}' (id: {})",
        config.name,
        short_id(sn.id())
    ));
    ok(format!("RPC endpoint: {}", config.rpc_url()));
    ok(format!("WebSocket:    {}", config.ws_url()));
    ok(format!(
        "REST API:     http://127.0.0.1:{}",
        config.api_port
    ));
    if !config.mirror_enabled {
        info("mainnet mirroring is DISABLED for this stagenet");
    }
    info(format!("Start it with: rustag start {}", config.name));
    Ok(())
}
