//! Background oracle sync loop.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::stagenet::Stagenet;

/// Spawn a background task that re-fetches CLEAN oracle accounts from mainnet on
/// the given interval, keeping Pyth/Switchboard prices fresh. DIRTY accounts are
/// never touched. The task runs until its [`JoinHandle`] is aborted.
pub fn spawn_oracle_sync(stagenet: Arc<RwLock<Stagenet>>, interval: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval.max(Duration::from_secs(1)));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        // The first tick fires immediately; skip it so we don't sync before any
        // oracle has been preloaded.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let mut sn = stagenet.write().await;
            match sn.refresh_clean_oracles().await {
                Ok(n) if n > 0 => tracing::debug!(refreshed = n, "background oracle sync"),
                Ok(_) => {}
                Err(e) => tracing::warn!(error = %e, "oracle sync failed; will retry"),
            }
        }
    })
}
