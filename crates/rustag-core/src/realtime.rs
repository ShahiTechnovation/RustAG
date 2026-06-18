//! Real-time mainnet account application.
//!
//! Phase 1 keeps CLEAN oracle accounts fresh by *polling* mainnet every 30s
//! ([`crate::spawn_oracle_sync`]). Phase 2 adds a *push* path: a real-time source
//! (a Yellowstone gRPC stream or the standard `accountSubscribe` WebSocket — see
//! `rustag_mirror::realtime`) streams [`RemoteAccount`] updates over a channel,
//! and this consumer applies them to the stagenet the moment they arrive.
//!
//! The application rule is the same invariant the whole mirror is built on:
//! **never overwrite locally-modified state.** A DIRTY or PINNED account (both
//! live in the stagenet's dirty-set) is skipped; everything else is refreshed.
//! This module is deliberately transport-agnostic — it only knows about a
//! channel of [`RemoteAccount`]s — so the same consumer serves gRPC, WebSocket,
//! or a test harness.

use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use rustag_mirror::RemoteAccount;

use crate::stagenet::Stagenet;

/// Spawn a task that applies streamed mainnet account updates to `stagenet`,
/// respecting dirty/pinned accounts. The task runs until the channel closes or
/// its [`JoinHandle`] is aborted. Returns the number of updates applied via the
/// task's lifetime through tracing; callers typically just hold the handle.
pub fn spawn_realtime_apply(
    stagenet: Arc<RwLock<Stagenet>>,
    mut updates: mpsc::Receiver<RemoteAccount>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut applied: u64 = 0;
        let mut skipped: u64 = 0;
        while let Some(remote) = updates.recv().await {
            let mut sn = stagenet.write().await;
            match sn.apply_realtime_update(remote).await {
                Ok(true) => applied += 1,
                Ok(false) => skipped += 1,
                Err(e) => tracing::warn!(error = %e, "failed to apply realtime account update"),
            }
            if applied % 50 == 0 && applied > 0 {
                tracing::debug!(applied, skipped, "realtime mirror progress");
            }
        }
        tracing::info!(applied, skipped, "realtime mirror stream closed");
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_pubkey::Pubkey;

    #[tokio::test]
    async fn applies_update_but_never_overwrites_dirty() {
        let sn = Stagenet::local("realtime-test").await.unwrap();
        let stagenet = Arc::new(RwLock::new(sn));

        // A fresh (clean) account is applied.
        let key = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        {
            let mut sn = stagenet.write().await;
            let applied = sn
                .apply_realtime_update(RemoteAccount {
                    pubkey: key,
                    lamports: 1_000,
                    data: vec![1, 2, 3],
                    owner,
                    executable: false,
                    rent_epoch: 0,
                })
                .await
                .unwrap();
            assert!(applied, "clean account must be applied");
            assert_eq!(sn.get_balance(&key).await.unwrap(), 1_000);
        }

        // Dirty it locally, then a streamed update must be ignored.
        {
            let mut sn = stagenet.write().await;
            sn.airdrop(&key, 5_000).await.unwrap();
            assert!(sn.is_dirty(&key));
            let applied = sn
                .apply_realtime_update(RemoteAccount {
                    pubkey: key,
                    lamports: 42,
                    data: vec![],
                    owner,
                    executable: false,
                    rent_epoch: 0,
                })
                .await
                .unwrap();
            assert!(!applied, "dirty account must NOT be overwritten");
            assert_eq!(sn.get_balance(&key).await.unwrap(), 6_000);
        }
    }
}
