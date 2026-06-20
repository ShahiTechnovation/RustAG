//! `rustag preload` - load known mainnet programs/oracles via the running stagenet.

use anyhow::{bail, Result};
use clap::Args;
use serde_json::json;

use rustag_core::registry;

use super::{
    api_base, config_from_record, connection_hint, info, ok, open_store, resolve_record, warn,
};

#[derive(Args)]
pub struct PreloadArgs {
    #[arg(short, long)]
    pub stagenet: Option<String>,
    /// Programs/oracles to load (e.g. `jupiter pyth raydium`). Empty lists options.
    pub programs: Vec<String>,
}

pub async fn run(args: PreloadArgs) -> Result<()> {
    if args.programs.is_empty() {
        info(format!(
            "available targets: {}",
            registry::available().join(", ")
        ));
        return Ok(());
    }

    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let config = config_from_record(&record)?;

    let url = format!("{}/api/preload", api_base(&config));
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&json!({ "programs": args.programs }))
        .send()
        .await
        .map_err(|_| connection_hint(&record.name))?;

    if !resp.status().is_success() {
        bail!("preload failed: {}", resp.text().await.unwrap_or_default());
    }

    let body: serde_json::Value = resp.json().await.unwrap_or_default();
    let loaded = body.get("loaded").and_then(|v| v.as_u64()).unwrap_or(0);
    ok(format!("Preloaded {loaded} accounts from mainnet"));
    if let Some(unknown) = body.get("unknown").and_then(|v| v.as_array()) {
        for u in unknown {
            if let Some(name) = u.as_str() {
                warn(format!("unknown target: {name}"));
            }
        }
    }
    Ok(())
}
