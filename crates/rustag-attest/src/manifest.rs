//! The attestation manifest — the human-readable, signable claim about what a
//! stagenet proved.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use rustag_core::{AccountEntry, TxOutcome};

use crate::state::{state_root, tx_results_root};

/// Schema identifier embedded in every manifest, so a verifier can refuse a
/// version it does not understand.
pub const MANIFEST_SCHEMA: &str = "rustag.attestation/v1";

/// Domain tag mixed into the manifest signing digest.
const MANIFEST_DIGEST_DOMAIN: &[u8] = b"rustag.attestation.manifest.v1";

/// A signable claim describing the state and outcomes a stagenet test produced.
///
/// All hash fields are lowercase hex. The struct is the *content*; the
/// [`crate::Attestation`] wraps it with an attester pubkey and a signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestationManifest {
    /// Schema version (`rustag.attestation/v1`).
    pub schema: String,
    /// The stagenet this attestation is about.
    pub stagenet_id: Uuid,
    /// Human-friendly stagenet name (informational).
    pub stagenet_name: String,
    /// Version of the tool that produced the attestation.
    pub tool_version: String,
    /// The mainnet source the staged state was derived from (RPC URL or
    /// `"offline"` when no mirror was used).
    pub mainnet_source: String,
    /// Network being mirrored (e.g. `mainnet-beta`).
    pub network: String,
    /// Stagenet slot at attestation time.
    pub slot: u64,
    /// Number of accounts committed to by `state_root`.
    pub account_count: usize,
    /// Merkle root over the full, pubkey-sorted account set (hex).
    pub state_root: String,
    /// Program ids that were exercised by the attested test run.
    pub programs: Vec<String>,
    /// Number of transaction outcomes committed to by `tx_results_root`.
    pub tx_count: usize,
    /// Merkle root over the ordered transaction outcomes (hex).
    pub tx_results_root: String,
    /// When the manifest was produced.
    pub created_at: DateTime<Utc>,
}

impl AttestationManifest {
    /// Construct a manifest from a stagenet's account set and test outcomes.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        stagenet_id: Uuid,
        stagenet_name: impl Into<String>,
        mainnet_source: impl Into<String>,
        network: impl Into<String>,
        slot: u64,
        accounts: &[AccountEntry],
        programs: Vec<String>,
        outcomes: &[TxOutcome],
    ) -> Self {
        Self {
            schema: MANIFEST_SCHEMA.to_string(),
            stagenet_id,
            stagenet_name: stagenet_name.into(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            mainnet_source: mainnet_source.into(),
            network: network.into(),
            slot,
            account_count: accounts.len(),
            state_root: hex::encode(state_root(accounts)),
            programs,
            tx_count: outcomes.len(),
            tx_results_root: hex::encode(tx_results_root(outcomes)),
            created_at: Utc::now(),
        }
    }

    /// The 32-byte digest that is actually signed.
    ///
    /// Built from a fixed field order with explicit separators rather than from
    /// JSON, because JSON key/whitespace ordering is not canonical and must
    /// never affect what a signature commits to.
    pub fn signing_digest(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(MANIFEST_DIGEST_DOMAIN);
        write_field(&mut h, self.schema.as_bytes());
        h.update(self.stagenet_id.as_bytes());
        write_field(&mut h, self.stagenet_name.as_bytes());
        write_field(&mut h, self.tool_version.as_bytes());
        write_field(&mut h, self.mainnet_source.as_bytes());
        write_field(&mut h, self.network.as_bytes());
        h.update(self.slot.to_le_bytes());
        h.update((self.account_count as u64).to_le_bytes());
        write_field(&mut h, self.state_root.as_bytes());
        h.update((self.programs.len() as u64).to_le_bytes());
        for program in &self.programs {
            write_field(&mut h, program.as_bytes());
        }
        h.update((self.tx_count as u64).to_le_bytes());
        write_field(&mut h, self.tx_results_root.as_bytes());
        // Bind the full-precision timestamp (nanoseconds), not whole seconds, so
        // the signature commits to exactly the `created_at` stored in the JSON.
        h.update(
            self.created_at
                .timestamp_nanos_opt()
                .unwrap_or(0)
                .to_le_bytes(),
        );
        h.finalize().into()
    }
}

/// Length-prefix a variable-length field so concatenation is unambiguous
/// (no two distinct field sequences can hash to the same byte stream).
fn write_field(h: &mut Sha256, bytes: &[u8]) {
    h.update((bytes.len() as u64).to_le_bytes());
    h.update(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> AttestationManifest {
        AttestationManifest {
            schema: MANIFEST_SCHEMA.to_string(),
            stagenet_id: Uuid::from_u128(1),
            stagenet_name: "demo".to_string(),
            tool_version: "0.1.0".to_string(),
            mainnet_source: "offline".to_string(),
            network: "mainnet-beta".to_string(),
            slot: 42,
            account_count: 3,
            state_root: "aa".repeat(32),
            programs: vec!["Prog1".to_string(), "Prog2".to_string()],
            tx_count: 2,
            tx_results_root: "bb".repeat(32),
            created_at: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        }
    }

    #[test]
    fn digest_is_stable_for_identical_manifests() {
        assert_eq!(manifest().signing_digest(), manifest().signing_digest());
    }

    #[test]
    fn digest_changes_when_any_field_changes() {
        let base = manifest().signing_digest();
        let mut m = manifest();
        m.slot = 43;
        assert_ne!(m.signing_digest(), base);

        let mut m = manifest();
        m.programs.push("Prog3".to_string());
        assert_ne!(m.signing_digest(), base);

        let mut m = manifest();
        m.state_root = "cc".repeat(32);
        assert_ne!(m.signing_digest(), base);
    }

    #[test]
    fn field_boundaries_are_unambiguous() {
        // "ab"+"c" must not digest the same as "a"+"bc": length prefixing
        // guarantees it.
        let mut m1 = manifest();
        m1.stagenet_name = "ab".to_string();
        m1.mainnet_source = "c".to_string();
        let mut m2 = manifest();
        m2.stagenet_name = "a".to_string();
        m2.mainnet_source = "bc".to_string();
        assert_ne!(m1.signing_digest(), m2.signing_digest());
    }

    #[test]
    fn manifest_json_roundtrip() {
        let m = manifest();
        let json = serde_json::to_string_pretty(&m).unwrap();
        let back: AttestationManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }
}
