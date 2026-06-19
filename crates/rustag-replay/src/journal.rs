//! An ordered, replayable log of the transactions applied to a stagenet.
//!
//! Each entry stores the transaction verbatim (bincode → base64, the canonical
//! Solana wire encoding) plus the state root it produced, so the whole history
//! can be deterministically re-applied to a restored checkpoint and the result
//! cross-checked against the recorded roots.

use base64::Engine;
use rustag_attest::state_root;
use rustag_core::{Stagenet, TxOutcome};
use serde::{Deserialize, Serialize};
use solana_transaction::versioned::VersionedTransaction;

use crate::error::{ReplayError, Result};

fn b64() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

/// Encode a transaction to canonical base64 text.
pub fn encode_tx(tx: &VersionedTransaction) -> Result<String> {
    let bytes = bincode::serialize(tx).map_err(|e| ReplayError::Codec(e.to_string()))?;
    Ok(b64().encode(bytes))
}

/// Decode a transaction from canonical base64 text.
pub fn decode_tx(s: &str) -> Result<VersionedTransaction> {
    let bytes = b64()
        .decode(s)
        .map_err(|e| ReplayError::Codec(e.to_string()))?;
    bincode::deserialize(&bytes).map_err(|e| ReplayError::Codec(e.to_string()))
}

/// One journalled, executed transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    /// Position in the journal, starting at 0.
    pub seq: u64,
    /// Human-friendly label.
    pub label: String,
    /// The transaction, bincode+base64 encoded.
    pub tx_base64: String,
    /// Whether it executed successfully.
    pub success: bool,
    /// The transaction's signature (base58).
    pub signature: String,
    /// Hex state root of the stagenet *after* this transaction.
    pub state_root_after: String,
}

/// An ordered list of executed transactions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Journal {
    entries: Vec<JournalEntry>,
}

impl Journal {
    /// An empty journal.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of recorded transactions.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the journal is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All entries, in order.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Append an already-executed transaction with its resulting state root.
    pub fn push(
        &mut self,
        label: impl Into<String>,
        tx: &VersionedTransaction,
        outcome: &TxOutcome,
        state_root_after: impl Into<String>,
    ) -> Result<()> {
        self.entries.push(JournalEntry {
            seq: self.entries.len() as u64,
            label: label.into(),
            tx_base64: encode_tx(tx)?,
            success: outcome.success,
            signature: outcome.signature_string(),
            state_root_after: state_root_after.into(),
        });
        Ok(())
    }

    /// Re-apply every journalled transaction to `stagenet`, returning each
    /// outcome in order. The caller is responsible for `stagenet` being at the
    /// correct starting state (e.g. a restored checkpoint).
    pub async fn replay_onto(&self, stagenet: &mut Stagenet) -> Result<Vec<TxOutcome>> {
        let mut outcomes = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            let tx = decode_tx(&entry.tx_base64)?;
            outcomes.push(stagenet.send_transaction(tx).await?);
        }
        Ok(outcomes)
    }
}

/// Execute `tx` against `stagenet`, record it in `journal` together with the
/// resulting state root, and return the outcome. This is the one-call way to
/// keep a journal in lockstep with a live stagenet.
///
/// For the recorded `state_root_after` to be reproducible under replay, run
/// against an **offline (mirror-disabled)** stagenet: on a mirror-enabled
/// stagenet, `send_transaction` may lazily fetch and persist read-only mainnet
/// accounts, which both move the recorded root and are non-deterministic across
/// runs. Checkpoints restore into offline forks
/// ([`Checkpoint::restore`](crate::Checkpoint::restore)), so a journal captured
/// offline replays bit-for-bit.
pub async fn execute_and_record(
    journal: &mut Journal,
    stagenet: &mut Stagenet,
    label: impl Into<String>,
    tx: VersionedTransaction,
) -> Result<TxOutcome> {
    let outcome = stagenet.send_transaction(tx.clone()).await?;
    let accounts = stagenet.export_accounts().await?;
    let root = hex::encode(state_root(&accounts));
    journal.push(label, &tx, &outcome, root)?;
    Ok(outcome)
}
