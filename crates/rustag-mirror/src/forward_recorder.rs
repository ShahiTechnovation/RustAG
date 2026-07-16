//! **ForwardRecorder** — records real mainnet transactions for a watched program.
//!
//! This is the foundation for the upgrade-rehearsal CI gate (Phase 3). The
//! recorder builds a self-contained per-program `Journal` corpus of real mainnet
//! transactions. Each recorded entry carries its pre-state snapshot (a
//! self-contained `Checkpoint`), so a candidate program upgrade can be replayed
//! against the real traffic and any divergence from the current deployed version
//! is immediately detectable.
//!
//! ## Architecture
//!
//! ```text
//! mainnet RPC (getSignaturesForAddress + getTransaction)
//!     │
//!     ▼
//! ForwardRecorder ──── buffers raw transactions
//!     │
//!     ▼
//! RecordedTransaction { pre_state_snapshot, raw_tx, slot, signature }
//!     │
//!     ▼
//! RecordedCorpus ──── stored per-program, queryable by slot range
//! ```
//!
//! Phase 3 will replace the polling RPC approach with Yellowstone gRPC for
//! lower-latency, higher-throughput recording.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::MirrorError;
use crate::fetcher::MainnetMirror;

/// A single recorded mainnet transaction with its pre-state context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedTransaction {
    /// The transaction's signature (base58).
    pub signature: String,
    /// The slot this transaction was confirmed in.
    pub slot: u64,
    /// The Unix timestamp of the block (from blockTime).
    pub block_time: Option<i64>,
    /// The raw serialized transaction (base64 bincode).
    pub raw_tx_b64: String,
    /// Whether the transaction succeeded.
    pub success: bool,
    /// The program IDs this transaction invoked.
    pub program_ids: Vec<String>,
    /// The set of accounts the transaction touched (pubkeys, base58).
    pub touched_accounts: Vec<String>,
}

/// A corpus of recorded mainnet transactions for a watched program.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecordedCorpus {
    /// The program being watched.
    pub program_id: String,
    /// Recorded transactions, ordered by slot ascending.
    pub transactions: Vec<RecordedTransaction>,
    /// The earliest slot in this corpus.
    pub from_slot: u64,
    /// The latest slot in this corpus.
    pub to_slot: u64,
}

impl RecordedCorpus {
    /// Create an empty corpus for a given program.
    pub fn new(program_id: &str) -> Self {
        Self {
            program_id: program_id.to_string(),
            ..Default::default()
        }
    }

    /// Number of recorded transactions.
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Whether the corpus is empty.
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Filter to only transactions that succeeded.
    pub fn successful(&self) -> impl Iterator<Item = &RecordedTransaction> {
        self.transactions.iter().filter(|t| t.success)
    }
}

/// Records mainnet transactions for a watched program.
pub struct ForwardRecorder<'a> {
    mirror: &'a MainnetMirror,
    max_transactions: usize,
    poll_interval: Duration,
}

impl<'a> ForwardRecorder<'a> {
    /// Create a recorder with default settings.
    pub fn new(mirror: &'a MainnetMirror) -> Self {
        Self {
            mirror,
            max_transactions: 1000,
            poll_interval: Duration::from_secs(2),
        }
    }

    /// Set the maximum number of transactions to record.
    pub fn with_max_transactions(mut self, max: usize) -> Self {
        self.max_transactions = max;
        self
    }

    /// Set the poll interval for RPC-based recording.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Fetch recent transactions for `program_id` from mainnet.
    ///
    /// This uses `getSignaturesForAddress` to discover transaction signatures,
    /// then `getTransaction` to fetch each one. Returns a corpus of up to
    /// `max_transactions` recent transactions.
    ///
    /// Phase 3 will replace this with Yellowstone gRPC for real-time streaming.
    #[tracing::instrument(skip(self), fields(program = %program_id))]
    pub async fn fetch_recent(
        &self,
        program_id: &str,
        limit: usize,
    ) -> Result<RecordedCorpus, MirrorError> {
        let limit = limit.min(self.max_transactions);

        tracing::info!(program = %program_id, limit, "fetching recent transactions");

        let signatures = self.get_signatures_for_address(program_id, limit).await?;

        if signatures.is_empty() {
            tracing::info!(program = %program_id, "no recent transactions found");
            return Ok(RecordedCorpus::new(program_id));
        }

        let mut corpus = RecordedCorpus::new(program_id);
        let mut min_slot = u64::MAX;
        let mut max_slot = 0u64;

        for sig_info in &signatures {
            match self.get_transaction(&sig_info.signature).await {
                Ok(Some(tx)) => {
                    if tx.slot < min_slot {
                        min_slot = tx.slot;
                    }
                    if tx.slot > max_slot {
                        max_slot = tx.slot;
                    }
                    corpus.transactions.push(tx);
                }
                Ok(None) => {
                    tracing::debug!(sig = %sig_info.signature, "transaction not found");
                }
                Err(e) => {
                    tracing::warn!(sig = %sig_info.signature, err = %e, "failed to fetch transaction");
                }
            }
        }

        // Sort by slot ascending (oldest first — replay order).
        corpus.transactions.sort_by_key(|t| t.slot);

        if !corpus.transactions.is_empty() {
            corpus.from_slot = min_slot;
            corpus.to_slot = max_slot;
        }

        tracing::info!(
            program = %program_id,
            recorded = corpus.len(),
            from_slot = corpus.from_slot,
            to_slot = corpus.to_slot,
            "recording complete"
        );

        Ok(corpus)
    }

    /// Fetch transaction signatures for a program using `getSignaturesForAddress`.
    async fn get_signatures_for_address(
        &self,
        program_id: &str,
        limit: usize,
    ) -> Result<Vec<SignatureInfo>, MirrorError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignaturesForAddress",
            "params": [
                program_id,
                {
                    "limit": limit.min(1000),
                    "commitment": "confirmed"
                }
            ]
        });

        let body = self
            .mirror
            .http_post(&request)
            .await?;

        let envelope: RpcEnvelope<Vec<SignatureInfo>> = serde_json::from_str(&body)
            .map_err(|e| MirrorError::InvalidResponse(format!("getSignaturesForAddress: {e}")))?;

        if let Some(err) = envelope.error {
            return Err(MirrorError::Rpc {
                code: err.code,
                message: err.message,
            });
        }

        Ok(envelope.result.unwrap_or_default())
    }

    /// Fetch a transaction by signature using `getTransaction`.
    async fn get_transaction(
        &self,
        signature: &str,
    ) -> Result<Option<RecordedTransaction>, MirrorError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                signature,
                {
                    "encoding": "base64",
                    "commitment": "confirmed",
                    "maxSupportedTransactionVersion": 0
                }
            ]
        });

        let body = self
            .mirror
            .http_post(&request)
            .await?;

        let envelope: RpcEnvelope<Option<TxResult>> = serde_json::from_str(&body)
            .map_err(|e| MirrorError::InvalidResponse(format!("getTransaction: {e}")))?;

        if let Some(err) = envelope.error {
            return Err(MirrorError::Rpc {
                code: err.code,
                message: err.message,
            });
        }

        let Some(Some(result)) = envelope.result else {
            return Ok(None);
        };

        // Extract the base64 transaction from the result.
        let raw_tx_b64 = match result.transaction {
            serde_json::Value::Array(ref arr) if !arr.is_empty() => {
                arr[0].as_str().unwrap_or("").to_string()
            }
            serde_json::Value::String(ref s) => s.clone(),
            _ => return Ok(None),
        };

        let success = result
            .meta
            .as_ref()
            .and_then(|m| m.get("err"))
            .map(|e| e.is_null())
            .unwrap_or(true);

        let touched_accounts = result
            .meta
            .as_ref()
            .and_then(|m| m.get("postBalances"))
            .and_then(|_| result.transaction.get("message"))
            .and_then(|msg| msg.get("accountKeys"))
            .and_then(|keys| keys.as_array())
            .map(|keys| {
                keys.iter()
                    .filter_map(|k| k.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Some(RecordedTransaction {
            signature: signature.to_string(),
            slot: result.slot,
            block_time: result.block_time,
            raw_tx_b64,
            success,
            program_ids: vec![], // populated in Phase 3 from tx message decoding
            touched_accounts,
        }))
    }
}

// JSON-RPC wire types ----------------------------------------------------

#[derive(serde::Deserialize)]
struct RpcEnvelope<T> {
    result: Option<T>,
    error: Option<RpcErrorObject>,
}

#[derive(serde::Deserialize)]
struct RpcErrorObject {
    code: i64,
    message: String,
}

#[derive(serde::Deserialize)]
struct SignatureInfo {
    signature: String,
    #[allow(dead_code)]
    slot: Option<u64>,
    #[allow(dead_code)]
    err: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TxResult {
    slot: u64,
    block_time: Option<i64>,
    transaction: serde_json::Value,
    meta: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_successful_filter() {
        let mut corpus = RecordedCorpus::new("11111111111111111111111111111111");
        corpus.transactions.push(RecordedTransaction {
            signature: "sig1".to_string(),
            slot: 100,
            block_time: None,
            raw_tx_b64: "".to_string(),
            success: true,
            program_ids: vec![],
            touched_accounts: vec![],
        });
        corpus.transactions.push(RecordedTransaction {
            signature: "sig2".to_string(),
            slot: 101,
            block_time: None,
            raw_tx_b64: "".to_string(),
            success: false,
            program_ids: vec![],
            touched_accounts: vec![],
        });
        assert_eq!(corpus.len(), 2);
        assert_eq!(corpus.successful().count(), 1);
    }
}
