//! `rustag schedule` - manage recurring on-chain activities (Phase 2).
//!
//! Talks to a running stagenet's REST API. An activity pairs a schedule
//! expression (`@every 30s` or a 5-field cron) with an action (airdrop, signed
//! transfer, or replay of a pre-signed transaction).

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use serde_json::{json, Value};

use super::{api_base, config_from_record, connection_hint, info, ok, open_store, resolve_record};

#[derive(Args)]
pub struct ScheduleArgs {
    #[arg(short, long)]
    pub stagenet: Option<String>,
    #[command(subcommand)]
    pub cmd: ScheduleCmd,
}

#[derive(Subcommand)]
pub enum ScheduleCmd {
    /// Add a recurring activity.
    Add(AddArgs),
    /// List all activities and their last run status.
    List,
    /// Remove an activity by id.
    Rm {
        /// Activity id (from `schedule list`).
        id: String,
    },
    /// Enable or disable an activity.
    Toggle {
        /// Activity id.
        id: String,
        /// Disable instead of enable.
        #[arg(long)]
        off: bool,
    },
}

#[derive(Args)]
pub struct AddArgs {
    /// Human-friendly activity name.
    pub name: String,
    /// Schedule: `@every 30s` / `@hourly` / 5-field cron `*/5 * * * *`.
    pub schedule: String,
    /// Airdrop action: the recipient wallet address.
    #[arg(long)]
    pub airdrop: Option<String>,
    /// Transfer action: base58 secret key of the sender wallet.
    #[arg(long)]
    pub transfer_from: Option<String>,
    /// Transfer recipient (used with --transfer-from).
    #[arg(long)]
    pub to: Option<String>,
    /// SOL amount for airdrop/transfer actions.
    #[arg(long, default_value_t = 1.0)]
    pub sol: f64,
    /// Raw signed transaction (base64) to replay on the schedule.
    #[arg(long)]
    pub raw_tx: Option<String>,
}

pub async fn run(args: ScheduleArgs) -> Result<()> {
    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let config = config_from_record(&record)?;
    let base = api_base(&config);
    let client = reqwest::Client::new();

    match args.cmd {
        ScheduleCmd::Add(add) => {
            let action = build_action(&add)?;
            let url = format!("{base}/api/schedules");
            let resp = client
                .post(&url)
                .json(&json!({ "name": add.name, "schedule": add.schedule, "action": action }))
                .send()
                .await
                .map_err(|_| connection_hint(&record.name))?;
            if !resp.status().is_success() {
                bail!(
                    "could not add activity: {}",
                    resp.text().await.unwrap_or_default()
                );
            }
            let body: Value = resp.json().await.unwrap_or(Value::Null);
            ok(format!(
                "Added activity '{}' ({})",
                add.name,
                body.get("id").and_then(|v| v.as_str()).unwrap_or("?")
            ));
            info(format!("schedule: {}", add.schedule));
        }
        ScheduleCmd::List => {
            let url = format!("{base}/api/schedules");
            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|_| connection_hint(&record.name))?;
            let body: Value = resp.json().await.unwrap_or(Value::Null);
            print_schedules(&body);
        }
        ScheduleCmd::Rm { id } => {
            let url = format!("{base}/api/schedules/{id}");
            let resp = client
                .delete(&url)
                .send()
                .await
                .map_err(|_| connection_hint(&record.name))?;
            if !resp.status().is_success() {
                bail!(
                    "could not remove activity: {}",
                    resp.text().await.unwrap_or_default()
                );
            }
            ok(format!("Removed activity {id}"));
        }
        ScheduleCmd::Toggle { id, off } => {
            let url = format!("{base}/api/schedules/{id}/toggle");
            let resp = client
                .post(&url)
                .json(&json!({ "enabled": !off }))
                .send()
                .await
                .map_err(|_| connection_hint(&record.name))?;
            if !resp.status().is_success() {
                bail!(
                    "could not toggle activity: {}",
                    resp.text().await.unwrap_or_default()
                );
            }
            ok(format!(
                "Activity {id} {}",
                if off { "disabled" } else { "enabled" }
            ));
        }
    }
    Ok(())
}

fn build_action(add: &AddArgs) -> Result<Value> {
    if let Some(pubkey) = &add.airdrop {
        Ok(json!({ "type": "airdrop", "pubkey": pubkey, "sol": add.sol }))
    } else if let (Some(secret), Some(to)) = (&add.transfer_from, &add.to) {
        Ok(json!({ "type": "transfer", "secret_key": secret, "to": to, "sol": add.sol }))
    } else if let Some(raw) = &add.raw_tx {
        Ok(json!({ "type": "raw_transaction", "transaction_base64": raw }))
    } else {
        bail!("specify an action: --airdrop <pubkey>, --transfer-from <secret> --to <pubkey>, or --raw-tx <base64>")
    }
}

fn print_schedules(body: &Value) {
    let schedules = body.get("schedules").and_then(|v| v.as_array());
    let Some(schedules) = schedules else {
        info("no activities");
        return;
    };
    if schedules.is_empty() {
        info("no activities - add one with `rustag schedule add`");
        return;
    }
    println!();
    println!(
        "  {:<10} {:<18} {:<16} {:<8} {:<6} {}",
        style("ID").bold(),
        style("NAME").bold(),
        style("SCHEDULE").bold(),
        style("ENABLED").bold(),
        style("RUNS").bold(),
        style("LAST STATUS").bold(),
    );
    for s in schedules {
        let id = s.get("id").and_then(|v| v.as_str()).unwrap_or("?");
        let short = id.chars().take(8).collect::<String>();
        println!(
            "  {:<10} {:<18} {:<16} {:<8} {:<6} {}",
            short,
            truncate(s.get("name").and_then(|v| v.as_str()).unwrap_or("-"), 18),
            truncate(
                s.get("schedule").and_then(|v| v.as_str()).unwrap_or("-"),
                16
            ),
            s.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false),
            s.get("runCount").and_then(|v| v.as_i64()).unwrap_or(0),
            s.get("lastStatus").and_then(|v| v.as_str()).unwrap_or("-"),
        );
    }
    println!();
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
