//! `rustag doctor` - preflight diagnostics.
//!
//! Answers the "it doesn't work" questions before they become GitHub issues: is
//! the data directory writable, is the database openable, is the configured
//! mainnet RPC reachable, and are the stagenet's ports free (or held by the
//! running stagenet itself). Exits non-zero if any hard check fails.

use std::net::TcpListener;
use std::time::Duration;

use anyhow::{bail, Result};
use clap::Args;
use console::style;

use super::{config_from_record, data_dir, db_path, fail, info, is_running, ok, open_store, warn};
use rustag_core::StagenetConfig;

#[derive(Args)]
pub struct DoctorArgs {
    /// Check a specific stagenet (default: every registered one, or the create
    /// defaults if none exist yet).
    #[arg(short, long)]
    pub stagenet: Option<String>,
}

pub async fn run(args: DoctorArgs) -> Result<()> {
    println!();
    println!("  {}", style("rustag doctor").bold().underlined());
    let mut failures = 0usize;

    // --- environment checks (independent of any stagenet) -------------------
    if !check_data_dir() {
        failures += 1;
    }
    if !check_db().await {
        failures += 1;
    }

    // --- per-stagenet checks ------------------------------------------------
    let store = open_store().await.ok();
    let records = match (&store, &args.stagenet) {
        (Some(store), Some(name)) => store
            .get_stagenet_by_name(name)
            .await
            .ok()
            .flatten()
            .into_iter()
            .collect(),
        (Some(store), None) => store.list_stagenets().await.unwrap_or_default(),
        (None, _) => Vec::new(),
    };

    if records.is_empty() {
        let cfg = StagenetConfig::default();
        info("no stagenets registered - checking `rustag create` defaults");
        failures += check_stagenet(&cfg).await;
    } else {
        for record in &records {
            match config_from_record(record) {
                Ok(cfg) => failures += check_stagenet(&cfg).await,
                Err(e) => {
                    fail(format!("stagenet '{}': corrupt config ({e})", record.name));
                    failures += 1;
                }
            }
        }
    }

    println!();
    if failures == 0 {
        ok("all checks passed");
        Ok(())
    } else {
        bail!("{failures} check(s) failed - see above");
    }
}

/// The data directory exists and is writable (create it, write+remove a probe).
fn check_data_dir() -> bool {
    let dir = data_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        fail(format!(
            "data directory not creatable ({}): {e}",
            dir.display()
        ));
        return false;
    }
    let probe = dir.join(".doctor-write-probe");
    match std::fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            ok(format!("data directory writable: {}", dir.display()));
            true
        }
        Err(e) => {
            fail(format!(
                "data directory not writable ({}): {e}",
                dir.display()
            ));
            false
        }
    }
}

/// The database opens and answers a trivial query.
async fn check_db() -> bool {
    match open_store().await {
        Ok(store) => match store.list_stagenets().await {
            Ok(records) => {
                ok(format!(
                    "database OK ({}): {} stagenet(s) registered",
                    db_path(),
                    records.len()
                ));
                true
            }
            Err(e) => {
                fail(format!("database opened but query failed: {e}"));
                false
            }
        },
        Err(e) => {
            fail(format!("cannot open database ({}): {e}", db_path()));
            false
        }
    }
}

/// Run the per-stagenet checks; returns the number of hard failures.
async fn check_stagenet(cfg: &StagenetConfig) -> usize {
    let mut failures = 0usize;
    info(format!("stagenet '{}':", cfg.name));

    // Mainnet RPC reachability (only meaningful when the mirror is on).
    if cfg.mirror_enabled {
        if !check_mainnet(&cfg.mainnet_rpc).await {
            failures += 1;
        }
    } else {
        info("    mirror disabled - skipping mainnet RPC check");
    }

    // Port availability. If the stagenet is running, its ports are *expected* to
    // be in use (by itself); if it's stopped, they must be free or `start` fails.
    let running = is_running(cfg).await;
    for (label, port) in [
        ("rpc", cfg.rpc_port),
        ("ws", cfg.ws_port),
        ("api", cfg.api_port),
    ] {
        let free = port_free(port);
        if running {
            if free {
                warn(format!(
                    "    port {port} ({label}) is free but the stagenet is marked running"
                ));
            } else {
                ok(format!(
                    "    port {port} ({label}) in use by the running stagenet"
                ));
            }
        } else if free {
            ok(format!("    port {port} ({label}) free"));
        } else {
            fail(format!(
                "    port {port} ({label}) is in use - `rustag start` would fail"
            ));
            failures += 1;
        }
    }
    failures
}

/// POST `getHealth` to the mainnet RPC; success → reachable, non-2xx → warn (the
/// endpoint is up but rate-limited/erroring), transport error → fail.
async fn check_mainnet(rpc: &str) -> bool {
    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    else {
        fail("    could not build an HTTP client for the mainnet check");
        return false;
    };
    let body = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "getHealth" });
    let shown = redact_url(rpc);
    match client.post(rpc).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {
            ok(format!("    mainnet RPC reachable: {shown}"));
            true
        }
        Ok(resp) => {
            warn(format!(
                "    mainnet RPC returned {} (up but degraded?): {shown}",
                resp.status()
            ));
            // Up-but-degraded is not a hard failure - the host is reachable.
            true
        }
        Err(e) => {
            fail(format!("    mainnet RPC unreachable: {shown} ({e})"));
            false
        }
    }
}

/// Whether a TCP port on loopback can be bound (i.e. is free).
fn port_free(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Drop the query string from a URL before printing it - mainnet/WS endpoints
/// carry `?api-key=...`, which must never reach logs or terminal output.
fn redact_url(url: &str) -> String {
    match url.split_once('?') {
        Some((base, _)) => format!("{base}?<redacted>"),
        None => url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_query_string() {
        assert_eq!(
            redact_url("https://rpc.example.com/?api-key=secret123"),
            "https://rpc.example.com/?<redacted>"
        );
        assert_eq!(
            redact_url("https://api.mainnet-beta.solana.com"),
            "https://api.mainnet-beta.solana.com"
        );
    }

    #[test]
    fn port_free_detects_a_bound_port() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        // Held by `listener`, so it must read as not free.
        assert!(!port_free(port));
        drop(listener);
        // Released - now bindable again.
        assert!(port_free(port));
    }
}
