//! `rustag logs` - tail the transaction feed from the running stagenet.

use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use clap::Args;
use console::style;
use serde_json::Value;

use super::{api_base, config_from_record, connection_hint, open_store, resolve_record};

#[derive(Args)]
pub struct LogsArgs {
    #[arg(short, long)]
    pub stagenet: Option<String>,
    /// Keep streaming new transactions.
    #[arg(short, long)]
    pub follow: bool,
}

pub async fn run(args: LogsArgs) -> Result<()> {
    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let config = config_from_record(&record)?;
    let url = format!("{}/api/transactions?limit=50", api_base(&config));
    let client = reqwest::Client::new();
    let mut seen: HashSet<String> = HashSet::new();

    loop {
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|_| connection_hint(&record.name))?;
        let body: Value = resp.json().await.unwrap_or_default();
        if let Some(txs) = body.get("transactions").and_then(|v| v.as_array()) {
            // API returns newest-first; print in chronological order.
            for tx in txs.iter().rev() {
                let sig = tx.get("signature").and_then(|s| s.as_str()).unwrap_or("");
                if sig.is_empty() || !seen.insert(sig.to_string()) {
                    continue;
                }
                print_tx(tx);
            }
        }

        if !args.follow {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}

fn print_tx(tx: &Value) {
    let success = tx.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    let cus = tx.get("computeUnits").and_then(|v| v.as_u64()).unwrap_or(0);
    let programs = tx
        .get("programs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|p| p.as_str())
                .map(short_pubkey)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let time = tx
        .get("createdAt")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "--:--:--".to_string());

    let mark = if success {
        style("✓").green().bold()
    } else {
        style("✗").red().bold()
    };
    let programs = if programs.is_empty() {
        "-".to_string()
    } else {
        programs
    };
    println!(
        "  [{}] {} {} (CUs: {})",
        style(time).dim(),
        mark,
        programs,
        cus
    );
}

fn short_pubkey(p: &str) -> String {
    if p.len() > 12 {
        format!("{}…{}", &p[..4], &p[p.len() - 4..])
    } else {
        p.to_string()
    }
}
