//! A serializable form of an account closure, so a rehearsal's `pre`-state can
//! be written next to its bundle and re-loaded by a verifier.
//!
//! The account's sync-state/category/stagenet bookkeeping is intentionally
//! dropped: it does not affect the Merkle state root (only the
//! consensus-visible fields do), so a round-trip through [`PortableAccount`]
//! reproduces the same `pre_state_root` a bundle commits to.

use std::str::FromStr;

use rustag_core::{AccountEntry, AccountSync};
use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;
use uuid::Uuid;

use crate::{RehearseError, Result};

/// A closure account in a portable, self-describing JSON form.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableAccount {
    /// Account address (base58).
    pub pubkey: String,
    /// Owning program (base58).
    pub owner: String,
    /// Lamport balance.
    pub lamports: u64,
    /// Whether the account is executable.
    pub executable: bool,
    /// Rent epoch.
    pub rent_epoch: u64,
    /// Account data, hex-encoded.
    pub data_hex: String,
}

impl PortableAccount {
    /// Project a live account entry into portable form.
    pub fn from_entry(entry: &AccountEntry) -> Self {
        Self {
            pubkey: entry.pubkey.to_string(),
            owner: entry.owner.to_string(),
            lamports: entry.lamports,
            executable: entry.executable,
            rent_epoch: entry.rent_epoch,
            data_hex: hex::encode(&entry.data),
        }
    }

    /// Rebuild a live account entry. `sync_state`/`category`/`stagenet_id` are
    /// set to neutral defaults - they do not participate in the state root.
    pub fn to_entry(&self) -> Result<AccountEntry> {
        let pubkey = Pubkey::from_str(&self.pubkey)
            .map_err(|_| RehearseError::Other(format!("bad pubkey: {}", self.pubkey)))?;
        let owner = Pubkey::from_str(&self.owner)
            .map_err(|_| RehearseError::Other(format!("bad owner: {}", self.owner)))?;
        let data = hex::decode(&self.data_hex)
            .map_err(|e| RehearseError::Other(format!("bad account data hex: {e}")))?;
        Ok(AccountEntry {
            pubkey,
            data,
            owner,
            lamports: self.lamports,
            executable: self.executable,
            rent_epoch: self.rent_epoch,
            sync_state: AccountSync::clean_now(),
            category: None,
            stagenet_id: Uuid::nil(),
        })
    }
}

/// Encode a closure to portable accounts.
pub fn to_portable(closure: &[AccountEntry]) -> Vec<PortableAccount> {
    closure.iter().map(PortableAccount::from_entry).collect()
}

/// Decode a closure from portable accounts.
pub fn from_portable(accounts: &[PortableAccount]) -> Result<Vec<AccountEntry>> {
    accounts.iter().map(PortableAccount::to_entry).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustag_attest::state_root;

    #[test]
    fn portable_roundtrip_preserves_state_root() {
        let entry = AccountEntry {
            pubkey: Pubkey::new_from_array([3; 32]),
            data: vec![1, 2, 3, 4, 5],
            owner: Pubkey::new_from_array([9; 32]),
            lamports: 42_000,
            executable: false,
            rent_epoch: 7,
            sync_state: AccountSync::clean_now(),
            category: None,
            stagenet_id: Uuid::nil(),
        };
        let portable = to_portable(&[entry.clone()]);
        let back = from_portable(&portable).unwrap();
        assert_eq!(state_root(&[entry]), state_root(&back));
    }
}
