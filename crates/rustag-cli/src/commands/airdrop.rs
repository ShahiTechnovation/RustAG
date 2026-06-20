//! `rustag airdrop` - credit SOL to a wallet via the running stagenet.

use std::str::FromStr;

use anyhow::{bail, Result};
use clap::Args;
use serde_json::json;
use solana_pubkey::Pubkey;

use super::{api_base, config_from_record, connection_hint, ok, open_store, resolve_record};

#[derive(Args)]
pub struct AirdropArgs {
    #[arg(short, long)]
    pub stagenet: Option<String>,
    /// Recipient wallet address.
    pub pubkey: String,
    /// Amount in SOL.
    pub amount: f64,
}

pub async fn run(args: AirdropArgs) -> Result<()> {
    Pubkey::from_str(&args.pubkey)
        .map_err(|_| anyhow::anyhow!("invalid pubkey: {}", args.pubkey))?;

    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let config = config_from_record(&record)?;

    let url = format!("{}/api/airdrop", api_base(&config));
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&json!({ "pubkey": args.pubkey, "sol": args.amount }))
        .send()
        .await
        .map_err(|_| connection_hint(&record.name))?;

    if !resp.status().is_success() {
        bail!("airdrop failed: {}", resp.text().await.unwrap_or_default());
    }
    ok(format!("Airdropped {} SOL to {}", args.amount, args.pubkey));
    Ok(())
}
