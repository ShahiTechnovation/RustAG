//! Account state machine: the heart of the lazy mainnet mirror.
//!
//! Every account in a stagenet carries a [`AccountSync`] tag that decides
//! whether the background scheduler is allowed to overwrite it from mainnet:
//!
//! - `Unknown`  - never fetched; will be fetched lazily on first access.
//! - `Clean`    - a faithful mainnet copy; the scheduler may re-sync it.
//! - `Dirty`    - modified by a local transaction; frozen from mainnet sync.
//! - `Pinned`   - explicitly set via the override API; immune to everything.

use chrono::{DateTime, Utc};
use solana_account::{Account, AccountSharedData};
use solana_pubkey::Pubkey;
use uuid::Uuid;

use rustag_mirror::{AccountCategory, RemoteAccount};

/// Synchronization state of a single account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountSync {
    /// Never fetched. Will be fetched lazily on first access.
    Unknown,
    /// Fetched from mainnet. May be re-synced by the background scheduler.
    Clean { fetched_at: DateTime<Utc> },
    /// Modified by a local transaction. Never overwritten by mainnet sync.
    Dirty { modified_at: DateTime<Utc> },
    /// Explicitly set by the user via the override API. Immune to everything.
    Pinned,
}

impl AccountSync {
    /// `Clean` stamped at the current time.
    pub fn clean_now() -> Self {
        AccountSync::Clean {
            fetched_at: Utc::now(),
        }
    }

    /// `Dirty` stamped at the current time.
    pub fn dirty_now() -> Self {
        AccountSync::Dirty {
            modified_at: Utc::now(),
        }
    }

    /// Stable string form persisted in the database.
    pub fn label(&self) -> &'static str {
        match self {
            AccountSync::Unknown => "Unknown",
            AccountSync::Clean { .. } => "Clean",
            AccountSync::Dirty { .. } => "Dirty",
            AccountSync::Pinned => "Pinned",
        }
    }

    /// Reconstruct from the persisted label plus optional timestamps.
    pub fn from_parts(
        label: &str,
        fetched_at: Option<DateTime<Utc>>,
        modified_at: Option<DateTime<Utc>>,
    ) -> Self {
        match label {
            "Clean" => AccountSync::Clean {
                fetched_at: fetched_at.unwrap_or_else(Utc::now),
            },
            "Dirty" => AccountSync::Dirty {
                modified_at: modified_at.unwrap_or_else(Utc::now),
            },
            "Pinned" => AccountSync::Pinned,
            _ => AccountSync::Unknown,
        }
    }

    /// The `fetched_at` timestamp, if this is a `Clean` state.
    pub fn fetched_at(&self) -> Option<DateTime<Utc>> {
        match self {
            AccountSync::Clean { fetched_at } => Some(*fetched_at),
            _ => None,
        }
    }

    /// The `modified_at` timestamp, if this is a `Dirty` state.
    pub fn modified_at(&self) -> Option<DateTime<Utc>> {
        match self {
            AccountSync::Dirty { modified_at } => Some(*modified_at),
            _ => None,
        }
    }
}

/// A fully-resolved account record within a stagenet.
#[derive(Debug, Clone)]
pub struct AccountEntry {
    pub pubkey: Pubkey,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub lamports: u64,
    pub executable: bool,
    pub rent_epoch: u64,
    pub sync_state: AccountSync,
    pub category: Option<AccountCategory>,
    pub stagenet_id: Uuid,
}

impl AccountEntry {
    /// Whether the background scheduler is allowed to overwrite this account.
    pub fn is_syncable(&self) -> bool {
        matches!(
            self.sync_state,
            AccountSync::Clean { .. } | AccountSync::Unknown
        )
    }

    /// Mark this account as locally modified (freezes it from mainnet sync).
    pub fn mark_dirty(&mut self) {
        self.sync_state = AccountSync::dirty_now();
    }

    /// Mark this account as a fresh mainnet copy.
    pub fn mark_clean(&mut self) {
        self.sync_state = AccountSync::clean_now();
    }

    /// Pin this account so nothing ever overwrites it.
    pub fn pin(&mut self) {
        self.sync_state = AccountSync::Pinned;
    }

    /// SOL balance as a float, for display.
    pub fn sol(&self) -> f64 {
        self.lamports as f64 / 1_000_000_000.0
    }

    /// Build a `Clean` entry from a freshly-fetched mainnet account.
    pub fn from_remote(
        remote: RemoteAccount,
        stagenet_id: Uuid,
        category: Option<AccountCategory>,
    ) -> Self {
        Self {
            pubkey: remote.pubkey,
            data: remote.data,
            owner: remote.owner,
            lamports: remote.lamports,
            executable: remote.executable,
            rent_epoch: remote.rent_epoch,
            sync_state: AccountSync::clean_now(),
            category,
            stagenet_id,
        }
    }

    /// Convert into a `solana_account::Account` for loading into LiteSVM.
    pub fn to_account(&self) -> Account {
        Account {
            lamports: self.lamports,
            data: self.data.clone(),
            owner: self.owner,
            executable: self.executable,
            rent_epoch: self.rent_epoch,
        }
    }

    /// Convert into LiteSVM's internal `AccountSharedData` representation.
    pub fn to_shared_data(&self) -> AccountSharedData {
        AccountSharedData::from(self.to_account())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_clean_transitions() {
        let mut e = AccountEntry {
            pubkey: Pubkey::new_unique(),
            data: vec![1, 2, 3],
            owner: Pubkey::default(),
            lamports: 42,
            executable: false,
            rent_epoch: 0,
            sync_state: AccountSync::clean_now(),
            category: None,
            stagenet_id: Uuid::nil(),
        };
        assert!(e.is_syncable());

        e.mark_dirty();
        assert!(!e.is_syncable());
        assert!(matches!(e.sync_state, AccountSync::Dirty { .. }));

        e.pin();
        assert!(!e.is_syncable());
        assert_eq!(e.sync_state, AccountSync::Pinned);

        e.mark_clean();
        assert!(e.is_syncable());
    }

    #[test]
    fn sync_label_roundtrip() {
        let now = Utc::now();
        let clean = AccountSync::Clean { fetched_at: now };
        assert_eq!(clean.label(), "Clean");
        let back = AccountSync::from_parts("Clean", Some(now), None);
        assert_eq!(back, clean);

        assert_eq!(
            AccountSync::from_parts("Pinned", None, None),
            AccountSync::Pinned
        );
        assert_eq!(
            AccountSync::from_parts("garbage", None, None),
            AccountSync::Unknown
        );
    }
}
