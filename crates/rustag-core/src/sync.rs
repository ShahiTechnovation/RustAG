//! Background oracle sync loop.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::stagenet::Stagenet;

/// Spawn a background task that re-fetches CLEAN oracle accounts from mainnet on
/// the given interval, keeping Pyth prices fresh. DIRTY accounts are
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

/// Spawn the analytics sampler: every `interval`, capture a
/// [`MetricsSnapshot`](crate::MetricsSnapshot) and persist it as one row per
/// series, pruning old rows to a bounded retention. The first sample is taken
/// immediately so the dashboard has a baseline point on startup.
pub fn spawn_metrics_sampler(
    stagenet: Arc<RwLock<Stagenet>>,
    interval: Duration,
) -> JoinHandle<()> {
    /// Keep the most-recent N metric rows per stagenet (~ several days at 60s).
    const RETENTION_ROWS: i64 = 20_000;
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval.max(Duration::from_secs(1)));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let sampled = {
                let sn = stagenet.read().await;
                sn.collect_metrics()
                    .await
                    .map(|snap| (snap, sn.store(), sn.id()))
            };
            let (snapshot, store, id) = match sampled {
                Ok(triple) => triple,
                Err(e) => {
                    tracing::warn!(error = %e, "metrics sample failed; will retry");
                    continue;
                }
            };
            if let Err(e) = store.insert_metrics(&id, &snapshot.into_points()).await {
                tracing::warn!(error = %e, "metrics persist failed");
            }
            if let Err(e) = store.prune_metrics(&id, RETENTION_ROWS).await {
                tracing::debug!(error = %e, "metrics prune failed");
            }
        }
    })
}
