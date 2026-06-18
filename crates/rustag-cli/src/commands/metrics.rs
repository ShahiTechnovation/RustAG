//! `rustag metrics` — show analytics time-series for a running stagenet.

use anyhow::Result;
use clap::Args;
use console::style;
use serde_json::Value;

use super::{api_base, config_from_record, connection_hint, info, open_store, resolve_record};

#[derive(Args)]
pub struct MetricsArgs {
    #[arg(short, long)]
    pub stagenet: Option<String>,
    /// Restrict to one series (e.g. `tvl_lamports`, `transactions`, `accounts`).
    #[arg(long)]
    pub series: Option<String>,
    /// How many of the most-recent points to fetch per series.
    #[arg(long, default_value_t = 20)]
    pub limit: i64,
}

pub async fn run(args: MetricsArgs) -> Result<()> {
    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let config = config_from_record(&record)?;

    let mut url = format!("{}/api/metrics?limit={}", api_base(&config), args.limit);
    if let Some(series) = &args.series {
        url.push_str(&format!("&series={series}"));
    }
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|_| connection_hint(&record.name))?;
    let body: Value = resp.json().await.unwrap_or(Value::Null);

    let Some(metrics) = body.get("metrics").and_then(|v| v.as_object()) else {
        info("no metrics yet — the analytics sampler captures one point per minute");
        return Ok(());
    };
    println!();
    for (series, points) in metrics {
        let arr = points.as_array().cloned().unwrap_or_default();
        let latest = arr
            .last()
            .and_then(|p| p.get("v"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        println!(
            "  {:<18} {} points, latest = {}",
            style(series).cyan().bold(),
            arr.len(),
            format_value(series, latest),
        );
    }
    println!();
    Ok(())
}

fn format_value(series: &str, value: f64) -> String {
    if series == "tvl_lamports" {
        format!("{:.4} SOL", value / 1_000_000_000.0)
    } else {
        format!("{value:.0}")
    }
}
