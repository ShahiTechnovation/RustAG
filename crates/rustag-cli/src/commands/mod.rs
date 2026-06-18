//! CLI command implementations and shared helpers.

pub mod airdrop;
pub mod create;
pub mod logs;
pub mod manage;
pub mod metrics;
pub mod overrides;
pub mod preload;
pub mod schedule;
pub mod start;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use console::style;
use uuid::Uuid;

use rustag_core::{AccountStore, StagenetConfig, StagenetRecord};

/// The project-local data directory (`./.rustag`).
pub fn data_dir() -> PathBuf {
    PathBuf::from(".rustag")
}

/// Path to the shared SQLite registry/database.
pub fn db_path() -> String {
    data_dir()
        .join("db.sqlite")
        .to_string_lossy()
        .replace('\\', "/")
}

/// Path to a stagenet's PID file.
pub fn pid_file(name: &str) -> PathBuf {
    data_dir().join(format!("{name}.pid"))
}

/// Open (creating if needed) the shared account store.
pub async fn open_store() -> Result<Arc<AccountStore>> {
    let store = AccountStore::connect(&db_path())
        .await
        .context("failed to open the RustAG database")?;
    Ok(Arc::new(store))
}

/// Resolve which stagenet a command targets.
pub async fn resolve_record(store: &AccountStore, name: Option<&str>) -> Result<StagenetRecord> {
    match name {
        Some(name) => store
            .get_stagenet_by_name(name)
            .await?
            .with_context(|| format!("stagenet '{name}' not found (run `rustag create {name}`)")),
        None => {
            let mut all = store.list_stagenets().await?;
            match all.len() {
                0 => bail!("no stagenets exist yet — run `rustag create <name>`"),
                1 => Ok(all.remove(0)),
                _ => bail!("multiple stagenets exist — specify one with `--stagenet <name>`"),
            }
        }
    }
}

/// Parse the [`StagenetConfig`] stored in a record.
pub fn config_from_record(record: &StagenetRecord) -> Result<StagenetConfig> {
    serde_json::from_str(&record.config_json).context("stored stagenet config is corrupt")
}

/// REST API base URL for a stagenet.
pub fn api_base(config: &StagenetConfig) -> String {
    format!("http://127.0.0.1:{}", config.api_port)
}

/// First 8 characters of a UUID, for display.
pub fn short_id(id: Uuid) -> String {
    id.to_string().chars().take(8).collect()
}

// --- pretty output ----------------------------------------------------------

pub fn ok(msg: impl AsRef<str>) {
    println!("  {} {}", style("✓").green().bold(), msg.as_ref());
}

pub fn info(msg: impl AsRef<str>) {
    println!("  {} {}", style("•").cyan(), msg.as_ref());
}

pub fn warn(msg: impl AsRef<str>) {
    println!("  {} {}", style("!").yellow().bold(), msg.as_ref());
}

/// Friendly "is the stagenet running?" error for client commands.
pub fn connection_hint(name: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "could not reach the stagenet's REST API — is it running? Start it with `rustag start {name}`"
    )
}

/// Probe a stagenet's REST health endpoint to detect whether it is running.
pub async fn is_running(config: &StagenetConfig) -> bool {
    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(800))
        .build()
    else {
        return false;
    };
    client
        .get(format!("{}/api/health", api_base(config)))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
