//! `rustag start` — open a stagenet and run its servers.

use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::RwLock;

use rustag_core::{registry, Stagenet};

use super::{config_from_record, info, ok, open_store, pid_file, resolve_record, short_id, warn};

#[derive(Args)]
pub struct StartArgs {
    /// Stagenet name (optional if only one exists).
    pub name: Option<String>,
    /// Extra programs/oracles to preload on startup (e.g. `jupiter pyth`).
    #[arg(long, num_args = 0..)]
    pub preload: Vec<String>,
}

pub async fn run(args: StartArgs) -> Result<()> {
    let store = open_store().await?;
    let record = resolve_record(&store, args.name.as_deref()).await?;
    let config = config_from_record(&record)?;

    // Refuse to start a second instance over a live one (it would bind-fail on
    // the same ports and clobber the running instance's PID file).
    if super::is_running(&config).await {
        anyhow::bail!(
            "stagenet '{}' is already running at {}",
            config.name,
            config.rpc_url()
        );
    }

    let mut sn = Stagenet::reopen(record.id, config.clone(), Arc::clone(&store)).await?;
    ok(format!(
        "Opened stagenet '{}' (id: {})",
        config.name,
        short_id(record.id)
    ));

    // Preload from config + CLI flags.
    let mut targets = config.preload.clone();
    targets.extend(args.preload.clone());
    if !targets.is_empty() {
        if config.mirror_enabled {
            preload(&mut sn, &targets).await;
        } else {
            warn("mirror disabled — skipping preload");
        }
    }

    let stagenet = Arc::new(RwLock::new(sn));

    // Write a PID file so `rustag stop` can find us.
    let pid_path = pid_file(&config.name);
    let _ = std::fs::write(&pid_path, std::process::id().to_string());

    ok(format!("RPC endpoint: {}", config.rpc_url()));
    ok(format!("WebSocket:    {}", config.ws_url()));
    ok(format!(
        "REST API:     http://127.0.0.1:{}",
        config.api_port
    ));
    info("Point your cluster URL at the RPC endpoint. Press Ctrl-C to stop.");

    let result = tokio::select! {
        r = rustag_rpc::serve(Arc::clone(&stagenet)) => r.context("server error"),
        _ = tokio::signal::ctrl_c() => {
            info("shutting down");
            Ok(())
        }
    };

    let _ = std::fs::remove_file(&pid_path);
    result
}

async fn preload(sn: &mut Stagenet, targets: &[String]) {
    let mut entries = Vec::new();
    for name in targets {
        match registry::resolve(name) {
            Some(mut e) => entries.append(&mut e),
            None => warn(format!("unknown preload target: {name}")),
        }
    }
    if entries.is_empty() {
        return;
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(format!(
        "Preloading {} accounts from mainnet...",
        entries.len()
    ));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    match sn.preload(&entries).await {
        Ok(loaded) => {
            pb.finish_and_clear();
            ok(format!("Preloaded {loaded} accounts from mainnet"));
        }
        Err(e) => {
            pb.finish_and_clear();
            warn(format!("preload failed: {e}"));
        }
    }
}
