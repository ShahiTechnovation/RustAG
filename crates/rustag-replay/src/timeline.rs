//! A timeline of checkpoints plus the journal between them — the time-travel
//! debugging surface.

use rustag_attest::account_leaf_hash;
use rustag_core::{AccountEntry, Stagenet};
use serde::Serialize;
use std::collections::HashMap;

use crate::checkpoint::{Checkpoint, CheckpointSummary};
use crate::error::{ReplayError, Result};
use crate::journal::Journal;

/// An ordered series of [`Checkpoint`]s and the [`Journal`] of transactions
/// applied to the stagenet they were taken from.
#[derive(Debug, Clone, Default)]
pub struct Timeline {
    /// Checkpoints in capture order.
    pub checkpoints: Vec<Checkpoint>,
    /// The transaction journal across the whole timeline.
    pub journal: Journal,
}

impl Timeline {
    /// An empty timeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Capture a checkpoint of `stagenet` and append it, returning its index.
    pub async fn checkpoint(&mut self, stagenet: &Stagenet) -> Result<usize> {
        let cp = Checkpoint::capture(stagenet).await?;
        self.checkpoints.push(cp);
        Ok(self.checkpoints.len() - 1)
    }

    /// The most recent checkpoint.
    pub fn latest(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }

    /// The checkpoint at `index`.
    pub fn at(&self, index: usize) -> Option<&Checkpoint> {
        self.checkpoints.get(index)
    }

    /// Restore an isolated stagenet at checkpoint `index`.
    pub async fn restore(&self, index: usize, name: &str) -> Result<Stagenet> {
        self.checkpoints
            .get(index)
            .ok_or(ReplayError::CheckpointOutOfRange(index))?
            .restore(name)
            .await
    }

    /// The account-level diff between checkpoints `from` and `to`.
    pub fn diff(&self, from: usize, to: usize) -> Result<StateDiff> {
        let a = self
            .checkpoints
            .get(from)
            .ok_or(ReplayError::CheckpointOutOfRange(from))?;
        let b = self
            .checkpoints
            .get(to)
            .ok_or(ReplayError::CheckpointOutOfRange(to))?;
        Ok(diff_accounts(&a.accounts, &b.accounts))
    }

    /// Summaries of every checkpoint, for reporting.
    pub fn summaries(&self) -> Vec<CheckpointSummary> {
        self.checkpoints.iter().map(Checkpoint::summary).collect()
    }
}

/// The account-level difference between two state snapshots.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StateDiff {
    /// Accounts present in `b` but not `a` (base58 pubkeys).
    pub added: Vec<String>,
    /// Accounts present in `a` but not `b`.
    pub removed: Vec<String>,
    /// Accounts present in both whose committed content changed.
    pub changed: Vec<String>,
}

impl StateDiff {
    /// Whether the two snapshots are identical.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// Total number of differing accounts.
    pub fn len(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }
}

/// Compute the account-level diff between two account sets, comparing committed
/// content via each account's canonical Merkle leaf hash (so the diff is exactly
/// what would move the state root).
pub fn diff_accounts(a: &[AccountEntry], b: &[AccountEntry]) -> StateDiff {
    let map_a: HashMap<String, [u8; 32]> = a
        .iter()
        .map(|e| (e.pubkey.to_string(), account_leaf_hash(e)))
        .collect();
    let map_b: HashMap<String, [u8; 32]> = b
        .iter()
        .map(|e| (e.pubkey.to_string(), account_leaf_hash(e)))
        .collect();

    let mut added = Vec::new();
    let mut changed = Vec::new();
    for (key, hash_b) in &map_b {
        match map_a.get(key) {
            None => added.push(key.clone()),
            Some(hash_a) if hash_a != hash_b => changed.push(key.clone()),
            Some(_) => {}
        }
    }
    let mut removed: Vec<String> = map_a
        .keys()
        .filter(|k| !map_b.contains_key(*k))
        .cloned()
        .collect();

    added.sort();
    changed.sort();
    removed.sort();
    StateDiff {
        added,
        removed,
        changed,
    }
}

/// Replay `journal` from `checkpoint` onto a fresh stagenet and return the
/// resulting hex state root.
pub async fn replay_to_root(
    checkpoint: &Checkpoint,
    journal: &Journal,
    name: &str,
) -> Result<String> {
    let mut sn = checkpoint.restore(name).await?;
    journal.replay_onto(&mut sn).await?;
    let accounts = sn.export_accounts().await?;
    Ok(hex::encode(rustag_attest::state_root(&accounts)))
}

/// Verify that replaying `journal` from `checkpoint` is deterministic: two
/// independent replays must yield identical state roots. This is the property a
/// security-audit replay depends on.
pub async fn verify_deterministic(checkpoint: &Checkpoint, journal: &Journal) -> Result<bool> {
    let first = replay_to_root(checkpoint, journal, "determinism-1").await?;
    let second = replay_to_root(checkpoint, journal, "determinism-2").await?;
    Ok(first == second)
}

/// Verify that replaying `journal` from `checkpoint` reproduces the state root
/// the journal *recorded* for its final transaction. Unlike
/// [`verify_deterministic`] (which only proves two replays agree with each
/// other), this proves the replay reproduces the **original** execution, which
/// is what makes a replay admissible as audit evidence.
///
/// This holds when the journal is self-contained — i.e. the `checkpoint` already
/// holds every account the journalled transactions touch. That is exactly the
/// case for offline (mirror-disabled) stagenets, which is how
/// [`Checkpoint::restore`] rebuilds state. For a journal captured against a
/// mirror-enabled stagenet, a transaction may have lazily fetched an account
/// that the checkpoint does not contain; document and capture such journals
/// against an offline stagenet to keep replay reproducible.
///
/// Returns `Ok(None)` for an empty journal (nothing to reproduce).
pub async fn replay_matches_journal(
    checkpoint: &Checkpoint,
    journal: &Journal,
) -> Result<Option<bool>> {
    let Some(expected) = journal.entries().last().map(|e| e.state_root_after.clone()) else {
        return Ok(None);
    };
    let actual = replay_to_root(checkpoint, journal, "journal-match").await?;
    Ok(Some(actual == expected))
}
