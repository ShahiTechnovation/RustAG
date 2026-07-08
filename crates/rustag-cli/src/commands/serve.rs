//! `rustag serve` - one-shot container entrypoint.
//!
//! Creates the stagenet on first boot (idempotent across restarts) and then runs
//! its servers, so a hosting platform (Render/Fly/Railway) can run a single
//! long-lived process without a separate `create` step. Pair with
//! `RUSTAG_BIND_HOST=0.0.0.0`, `$PORT`, `RUSTAG_MAINNET_RPC`, and
//! `RUSTAG_DEMO_MODE=1` for a safe public demo.

use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use tokio::sync::RwLock;
use uuid::Uuid;

use rustag_core::{AccountStore, Stagenet, StagenetConfig};
use rustag_scheduler::{register_activity, Action};

use super::{config_from_record, db_path, info, ok, open_store, short_id, warn};

/// Registry targets preloaded by default so the demo never boots with an empty
/// account table (all resolve in `rustag_core::registry`).
const DEFAULT_PRELOAD: [&str; 3] = ["pyth", "raydium", "spl-token"];

#[derive(Args)]
pub struct ServeArgs {
    /// Stagenet name (created on first run, reused thereafter).
    #[arg(default_value = "demo")]
    pub name: String,
    /// Programs/oracles to preload so the demo boots with real mainnet data.
    /// Defaults to `pyth raydium spl-token` when omitted.
    #[arg(long, num_args = 0..)]
    pub preload: Vec<String>,
    /// Mainnet RPC endpoint for the lazy mirror (required for real data; falls
    /// back to the rate-limited public endpoint if unset).
    #[arg(long, env = "RUSTAG_MAINNET_RPC")]
    pub mainnet_rpc: Option<String>,
}

pub async fn run(args: ServeArgs) -> Result<()> {
    let store = open_store().await?;

    let targets = if args.preload.is_empty() {
        DEFAULT_PRELOAD.iter().map(|s| s.to_string()).collect()
    } else {
        args.preload.clone()
    };

    // Create-if-needed: idempotent so a container restart reuses the persisted
    // stagenet (accounts, txs, schedules) on a mounted volume.
    let (mut sn, config) = match store.get_stagenet_by_name(&args.name).await? {
        Some(record) => {
            info(format!("Reusing existing stagenet '{}'", args.name));
            let config = config_from_record(&record)?;
            let sn = Stagenet::reopen(record.id, config.clone(), Arc::clone(&store)).await?;
            (sn, config)
        }
        None => {
            let config = StagenetConfig {
                name: args.name.clone(),
                mainnet_rpc: args
                    .mainnet_rpc
                    .clone()
                    .unwrap_or_else(|| rustag_core::DEFAULT_MAINNET_RPC.to_string()),
                preload: targets.clone(),
                db_path: db_path(),
                ..StagenetConfig::default()
            };
            let sn = Stagenet::create(config.clone(), Arc::clone(&store)).await?;
            ok(format!(
                "Created stagenet '{}' (id: {})",
                config.name,
                short_id(sn.id())
            ));
            (sn, config)
        }
    };

    // Preload from the persisted config so the account table is never empty.
    if config.mirror_enabled && !config.preload.is_empty() {
        super::start::preload(&mut sn, &config.preload).await;
    }

    // On a public demo, seed a small recurring activity so a passive reviewer
    // sees the slot advance and the transaction feed move on its own.
    if rustag_rpc::demo_mode_enabled() {
        if let Err(e) = seed_demo_activity(&store, sn.id()).await {
            warn(format!("could not seed demo heartbeat: {e}"));
        }
    }

    let stagenet = Arc::new(RwLock::new(sn));
    ok(format!("Serving stagenet '{}'", config.name));
    info("REST API bind is controlled by RUSTAG_BIND_HOST / $PORT (see docs).");

    rustag_rpc::serve(stagenet).await.map_err(Into::into)
}

/// Name of the seeded heartbeat activity (also its idempotency key).
const DEMO_HEARTBEAT_NAME: &str = "demo-heartbeat";

/// Seed a tiny recurring airdrop so the demo's slot advances and its transaction
/// feed stays populated for a passive reviewer. Idempotent: only registers the
/// activity when it is not already present (survives container restarts).
async fn seed_demo_activity(store: &AccountStore, stagenet_id: Uuid) -> Result<()> {
    let existing = store.list_schedules(&stagenet_id, false).await?;
    if existing.iter().any(|s| s.name == DEMO_HEARTBEAT_NAME) {
        return Ok(());
    }
    let action = Action::Airdrop {
        // A fixed, deterministic demo faucet wallet - any 32 bytes is a valid
        // airdrop target, and a stable address means the demo always credits the
        // same visible wallet.
        pubkey: bs58::encode([7u8; 32]).into_string(),
        sol: 0.01,
    };
    register_activity(
        store,
        stagenet_id,
        DEMO_HEARTBEAT_NAME,
        "@every 30s",
        &action,
    )
    .await?;
    info("Seeded demo heartbeat (airdrop @every 30s) so the feed stays live.");
    Ok(())
}
