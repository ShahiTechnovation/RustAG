//! Content-addressed snapshots of a stagenet's full account state.

use chrono::{DateTime, Utc};
use rustag_attest::state_root;
use rustag_core::{AccountEntry, Stagenet};
use serde::Serialize;
use uuid::Uuid;

use crate::error::Result;

/// An immutable snapshot of every account in a stagenet at a point in time,
/// content-addressed by its Merkle [`state_root`](rustag_attest::state_root).
///
/// Restoring a checkpoint rebuilds an isolated, offline stagenet carrying
/// exactly the captured state - the basis of time-travel debugging and
/// security-audit replays.
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// Unique id for this checkpoint.
    pub id: Uuid,
    /// The stagenet this snapshot was taken from.
    pub stagenet_id: Uuid,
    /// The stagenet's slot at capture time.
    pub slot: u64,
    /// Every account, exactly as persisted.
    pub accounts: Vec<AccountEntry>,
    /// Hex Merkle root committing to `accounts` (the content address).
    pub state_root: String,
    /// When the snapshot was taken.
    pub created_at: DateTime<Utc>,
}

impl Checkpoint {
    /// Capture the current state of `stagenet`.
    pub async fn capture(stagenet: &Stagenet) -> Result<Self> {
        let accounts = stagenet.export_accounts().await?;
        let state_root = hex::encode(state_root(&accounts));
        Ok(Self {
            id: Uuid::new_v4(),
            stagenet_id: stagenet.id(),
            slot: stagenet.current_slot(),
            accounts,
            state_root,
            created_at: Utc::now(),
        })
    }

    /// Rebuild an isolated, offline stagenet carrying this checkpoint's state.
    ///
    /// The restored stagenet has its own fresh id and database and mirroring
    /// disabled, so replays never touch mainnet or the original. Its slot
    /// counter starts at zero (slot does not affect the account state root).
    pub async fn restore(&self, name: &str) -> Result<Stagenet> {
        let mut sn = Stagenet::local(name).await?;
        sn.import_accounts(&self.accounts).await?;
        Ok(sn)
    }

    /// Number of accounts captured.
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// A small, serializable summary (no account payloads) for reports/JSON.
    pub fn summary(&self) -> CheckpointSummary {
        CheckpointSummary {
            id: self.id,
            stagenet_id: self.stagenet_id,
            slot: self.slot,
            account_count: self.accounts.len(),
            state_root: self.state_root.clone(),
            created_at: self.created_at,
        }
    }
}

/// A lightweight, serializable description of a [`Checkpoint`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointSummary {
    pub id: Uuid,
    pub stagenet_id: Uuid,
    pub slot: u64,
    pub account_count: usize,
    pub state_root: String,
    pub created_at: DateTime<Utc>,
}
