//! `rustag override` - set account state via the running stagenet.

use std::str::FromStr;

use anyhow::{bail, Result};
use clap::Args;
use serde_json::json;
use solana_pubkey::Pubkey;

use super::{api_base, config_from_record, connection_hint, ok, open_store, resolve_record};

#[derive(Args)]
pub struct OverrideArgs {
    #[arg(short, long)]
    pub stagenet: Option<String>,
    /// Account to override.
    #[arg(long)]
    pub pubkey: String,
    /// Set the lamport balance.
    #[arg(long)]
    pub lamports: Option<u64>,
    /// Set an SPL token account's amount.
    #[arg(long = "token-balance")]
    pub token_balance: Option<u64>,
}

pub async fn run(args: OverrideArgs) -> Result<()> {
    Pubkey::from_str(&args.pubkey)
        .map_err(|_| anyhow::anyhow!("invalid pubkey: {}", args.pubkey))?;
    if args.lamports.is_none() && args.token_balance.is_none() {
        bail!("nothing to override - pass --lamports or --token-balance");
    }

    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let config = config_from_record(&record)?;

    let url = format!("{}/api/override", api_base(&config));
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&json!({
            "pubkey": args.pubkey,
            "lamports": args.lamports,
            "tokenBalance": args.token_balance,
        }))
        .send()
        .await
        .map_err(|_| connection_hint(&record.name))?;

    if !resp.status().is_success() {
        bail!("override failed: {}", resp.text().await.unwrap_or_default());
    }
    if let Some(amount) = args.token_balance {
        ok(format!("Token balance of {} set to {amount}", args.pubkey));
    } else if let Some(lamports) = args.lamports {
        ok(format!(
            "Balance of {} set to {lamports} lamports",
            args.pubkey
        ));
    }
    Ok(())
}
