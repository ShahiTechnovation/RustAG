//! A tamper-evident, hash-chained audit log - the SOC 2 groundwork.
//!
//! Each entry commits to the hash of the previous entry, so the whole history
//! is a hash chain. Any insertion, deletion, or edit anywhere in the log breaks
//! the chain from that point forward, and [`AuditLog::verify`] reports the exact
//! index where it broke. The chain is genesis-anchored at the all-zero hash.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Domain tag mixed into every audit-entry hash.
const AUDIT_DOMAIN: &[u8] = b"rustag.audit.v1";

/// The all-zero genesis predecessor hash (hex).
fn genesis_hash() -> String {
    hex::encode([0u8; 32])
}

/// One append-only audit record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    /// Monotonic sequence number, starting at 0.
    pub seq: u64,
    /// When the action occurred.
    pub timestamp: DateTime<Utc>,
    /// Who performed it (tenant id, user id, "system", ...).
    pub actor: String,
    /// What was done (free-form action label).
    pub action: String,
    /// Structured details (opaque to the log).
    pub details: serde_json::Value,
    /// Hash of the previous entry (hex); genesis = all-zero.
    pub prev_hash: String,
    /// This entry's hash (hex), covering all fields above plus `prev_hash`.
    pub hash: String,
}

impl AuditEntry {
    /// Recompute this entry's hash from its content and predecessor.
    fn compute_hash(
        seq: u64,
        timestamp: &DateTime<Utc>,
        actor: &str,
        action: &str,
        details: &serde_json::Value,
        prev_hash: &str,
    ) -> String {
        let mut h = Sha256::new();
        h.update(AUDIT_DOMAIN);
        h.update(seq.to_le_bytes());
        h.update(timestamp.timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        write_field(&mut h, actor.as_bytes());
        write_field(&mut h, action.as_bytes());
        // serde_json::to_vec is deterministic for a given Value (object key
        // order is preserved as inserted / parsed), which is sufficient here
        // because we hash the Value we ourselves stored.
        let detail_bytes = serde_json::to_vec(details).unwrap_or_default();
        write_field(&mut h, &detail_bytes);
        write_field(&mut h, prev_hash.as_bytes());
        hex::encode(h.finalize())
    }

    /// Recompute the expected hash for this entry given its stored fields.
    fn recompute(&self) -> String {
        Self::compute_hash(
            self.seq,
            &self.timestamp,
            &self.actor,
            &self.action,
            &self.details,
            &self.prev_hash,
        )
    }
}

fn write_field(h: &mut Sha256, bytes: &[u8]) {
    h.update((bytes.len() as u64).to_le_bytes());
    h.update(bytes);
}

/// An append-only, tamper-evident chain of [`AuditEntry`]s.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    /// An empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an action, linking it to the current head, and return the new
    /// entry's hash.
    pub fn append(
        &mut self,
        actor: impl Into<String>,
        action: impl Into<String>,
        details: serde_json::Value,
    ) -> String {
        let seq = self.entries.len() as u64;
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(genesis_hash);
        let actor = actor.into();
        let action = action.into();
        let timestamp = Utc::now();
        let hash = AuditEntry::compute_hash(seq, &timestamp, &actor, &action, &details, &prev_hash);
        let entry = AuditEntry {
            seq,
            timestamp,
            actor,
            action,
            details,
            prev_hash,
            hash: hash.clone(),
        };
        self.entries.push(entry);
        hash
    }

    /// All entries, oldest first.
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// The current head hash (genesis if empty).
    pub fn head(&self) -> String {
        self.entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(genesis_hash)
    }

    /// Verify the chain. `Ok(())` if intact; `Err(index)` at the first entry
    /// whose stored hash, predecessor link, or sequence number is inconsistent.
    pub fn verify(&self) -> std::result::Result<(), usize> {
        let mut expected_prev = genesis_hash();
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.seq != i as u64
                || entry.prev_hash != expected_prev
                || entry.hash != entry.recompute()
            {
                return Err(i);
            }
            expected_prev = entry.hash.clone();
        }
        Ok(())
    }

    /// Whether the chain is fully intact.
    pub fn is_intact(&self) -> bool {
        self.verify().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fresh_log_is_intact_and_genesis_anchored() {
        let log = AuditLog::new();
        assert!(log.is_intact());
        assert_eq!(log.head(), hex::encode([0u8; 32]));
    }

    #[test]
    fn appends_chain_and_verify() {
        let mut log = AuditLog::new();
        log.append("tenant-a", "stagenet.create", json!({"name": "alpha"}));
        log.append("tenant-a", "attest.create", json!({"slot": 10}));
        log.append("system", "stagenet.sleep", json!({}));
        assert_eq!(log.entries().len(), 3);
        assert!(log.verify().is_ok());
        // Each entry links to the previous one's hash.
        assert_eq!(log.entries()[1].prev_hash, log.entries()[0].hash);
        assert_eq!(log.entries()[2].prev_hash, log.entries()[1].hash);
    }

    #[test]
    fn editing_an_entry_is_detected() {
        let mut log = AuditLog::new();
        log.append("a", "x", json!({"v": 1}));
        log.append("b", "y", json!({"v": 2}));
        log.append("c", "z", json!({"v": 3}));
        // Tamper with the middle entry's details without updating its hash.
        log.entries[1].details = json!({"v": 999});
        assert_eq!(log.verify(), Err(1));
    }

    #[test]
    fn deleting_an_entry_is_detected() {
        let mut log = AuditLog::new();
        log.append("a", "x", json!({}));
        log.append("b", "y", json!({}));
        log.append("c", "z", json!({}));
        // Remove the middle entry: seq numbers and links no longer line up.
        log.entries.remove(1);
        assert!(log.verify().is_err());
    }

    #[test]
    fn log_json_roundtrips_and_stays_intact() {
        let mut log = AuditLog::new();
        log.append("a", "x", json!({"k": "v"}));
        log.append("b", "y", json!({"n": 2}));
        let json_str = serde_json::to_string(&log).unwrap();
        let back: AuditLog = serde_json::from_str(&json_str).unwrap();
        assert!(back.is_intact());
        assert_eq!(back.head(), log.head());
    }
}
