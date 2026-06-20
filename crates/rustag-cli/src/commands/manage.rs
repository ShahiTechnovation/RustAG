//! `rustag stop` / `status` / `list`.

use anyhow::Result;
use clap::Args;
use console::style;

use super::{
    api_base, config_from_record, info, is_running, ok, open_store, pid_file, resolve_record,
    short_id, warn,
};

#[derive(Args)]
pub struct StopArgs {
    #[arg(short, long)]
    pub stagenet: Option<String>,
}

#[derive(Args)]
pub struct StatusArgs {
    #[arg(short, long)]
    pub stagenet: Option<String>,
}

pub async fn stop(args: StopArgs) -> Result<()> {
    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let pid_path = pid_file(&record.name);

    let Ok(pid_str) = std::fs::read_to_string(&pid_path) else {
        warn(format!(
            "stagenet '{}' does not appear to be running",
            record.name
        ));
        return Ok(());
    };
    // Validate the PID is numeric before signalling anything.
    let Ok(pid) = pid_str.trim().parse::<u32>() else {
        let _ = std::fs::remove_file(&pid_path);
        warn(format!("ignoring malformed PID file for '{}'", record.name));
        return Ok(());
    };
    let pid = pid.to_string();

    // On Windows, filter by image name so a recycled PID belonging to some other
    // process is never force-killed.
    let killed = if cfg!(windows) {
        std::process::Command::new("taskkill")
            .args(["/PID", &pid, "/F", "/FI", "IMAGENAME eq rustag.exe"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        std::process::Command::new("kill")
            .arg(&pid)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    let _ = std::fs::remove_file(&pid_path);
    if killed {
        ok(format!("stopped stagenet '{}' (pid {pid})", record.name));
    } else {
        warn(format!(
            "could not signal pid {pid}; removed stale PID file for '{}'",
            record.name
        ));
    }
    Ok(())
}

pub async fn status(args: StatusArgs) -> Result<()> {
    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let config = config_from_record(&record)?;

    let accounts = store.count_accounts(&record.id).await?;
    let transactions = store.count_transactions(&record.id).await?;
    let running = is_running(&config).await;

    println!();
    println!("  {}", style(&record.name).bold().underlined());
    info(format!("id:           {}", short_id(record.id)));
    info(format!(
        "status:       {}",
        if running {
            style("running").green().to_string()
        } else {
            style("stopped").dim().to_string()
        }
    ));
    info(format!("rpc:          {}", config.rpc_url()));
    info(format!("rest api:     {}", api_base(&config)));
    info(format!("mainnet rpc:  {}", config.mainnet_rpc));
    info(format!(
        "mirror:       {}",
        if config.mirror_enabled {
            "enabled"
        } else {
            "disabled"
        }
    ));
    info(format!("accounts:     {accounts}"));
    info(format!("transactions: {transactions}"));
    println!();
    Ok(())
}

pub async fn list() -> Result<()> {
    let store = open_store().await?;
    let records = store.list_stagenets().await?;
    if records.is_empty() {
        info("no stagenets yet - run `rustag create <name>`");
        return Ok(());
    }
    println!();
    println!(
        "  {:<24} {:<10} {:<26} {}",
        style("NAME").bold(),
        style("ID").bold(),
        style("RPC").bold(),
        style("STATUS").bold()
    );
    for record in records {
        let config = config_from_record(&record)?;
        let running = is_running(&config).await;
        println!(
            "  {:<24} {:<10} {:<26} {}",
            record.name,
            short_id(record.id),
            config.rpc_url(),
            if running {
                style("running").green().to_string()
            } else {
                style("stopped").dim().to_string()
            }
        );
    }
    println!();
    Ok(())
}
