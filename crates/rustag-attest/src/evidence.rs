//! `EvidenceBundle` - the signed, offline-verifiable proof of what a *proposed*
//! privileged transaction does before it is signed or deployed.
//!
//! Where [`crate::Attestation`] proves "this staging state existed", an
//! [`EvidenceBundle`] proves the stronger, action-shaped claim a multisig signer
//! actually needs: *"rehearsing this exact payload against this exact
//! (cross-checked) mainnet closure produces this diff, trips these alarms, at
//! this compute - and you can re-execute it yourself to confirm."*
//!
//! The bundle is the output of a [`rustag-rehearse`](https://docs.rs/rustag-rehearse)
//! run. This crate owns only the *artifact*: its canonical signing digest,
//! Ed25519 binding, and the offline input/signature checks. The
//! re-execution check (restore the closure, replay the payload, reproduce
//! `post_state_root`) lives in `rustag-rehearse`, which has the runtime.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_signer::Signer;

use rustag_core::AccountEntry;

use crate::error::{AttestError, Result};
use crate::state::state_root;

/// Schema identifier embedded in every bundle.
pub const EVIDENCE_SCHEMA: &str = "groundtruth.evidence/v1";
/// Domain tag mixed into the evidence signing digest (distinct from the
/// attestation-manifest domain, so the two artifact types can never collide).
const EVIDENCE_DIGEST_DOMAIN: &[u8] = b"groundtruth.evidence.v1";

/// How faithfully the recorded result can be reproduced by a third party.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FidelityGrade {
    /// Deterministically re-executable: a self-contained offline checkpoint +
    /// journal whose replay reproduces `post_state_root` bit-for-bit.
    A,
    /// A signed *observation* of an external engine (e.g. a surfnet run) that is
    /// not offline-self-contained.
    B,
}

/// Provenance of the mainnet state a rehearsal read - how many independent RPCs
/// agreed on the closure, bounding (never proving) the input's truthfulness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InputProvenance {
    /// RPC endpoints the closure was cross-fetched from (redacted host forms).
    pub endpoints: Vec<String>,
    /// The slot each endpoint reported (parallel to `endpoints`).
    pub slots: Vec<u64>,
    /// Whether all endpoints agreed byte-for-byte on the closure.
    pub full_agreement: bool,
}

/// The signable claim describing a rehearsal's inputs and result.
///
/// All hash fields are lowercase hex. `findings` is opaque JSON (the semantic
/// diff, invariant alarms, and exploit-scan report produced by `rustag-sim`),
/// carried verbatim and bound into the signature by its canonical bytes - so
/// this crate never has to depend on the simulation types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceManifest {
    /// Schema version (`groundtruth.evidence/v1`).
    pub schema: String,
    /// Version of the tool that produced the bundle.
    pub tool_version: String,
    /// SHA-256 (hex) of the canonical payload bytes that were rehearsed.
    pub payload_hash: String,
    /// The on-chain proposal account this payload came from, if any (base58).
    pub proposal_account: Option<String>,
    /// Program ids the payload invoked.
    pub program_ids: Vec<String>,
    /// Network the closure was mirrored from (e.g. `mainnet-beta`).
    pub network: String,
    /// The mainnet source (redacted RPC url, or `"offline"`).
    pub input_source: String,
    /// Cross-fetch provenance of the mainnet closure.
    pub provenance: InputProvenance,
    /// How faithfully the result can be reproduced.
    pub fidelity_grade: FidelityGrade,
    /// The execution engine (e.g. `litesvm-0.12`).
    pub engine: String,
    /// Merkle root over the closure the payload executed *against* (hex).
    pub pre_state_root: String,
    /// Merkle root over the state the payload produced (hex).
    pub post_state_root: String,
    /// Number of accounts in the committed closure.
    pub closure_account_count: usize,
    /// Compute units the payload consumed.
    pub compute_units: u64,
    /// Whether the payload executed successfully.
    pub success: bool,
    /// The semantic diff + invariant alarms + exploit-scan report (opaque JSON).
    pub findings: serde_json::Value,
    /// Hash of the previous bundle in this subject's chain of custody, if any.
    pub prev_bundle_hash: Option<String>,
    /// When the bundle was produced.
    pub created_at: DateTime<Utc>,
}

impl EvidenceManifest {
    /// The 32-byte digest that is actually signed.
    ///
    /// Built from a fixed field order with explicit length prefixes rather than
    /// from JSON, because JSON key/whitespace ordering is not canonical and must
    /// never affect what a signature commits to. `findings` is bound by its
    /// canonical serialized bytes (serde_json sorts object keys by default).
    pub fn signing_digest(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(EVIDENCE_DIGEST_DOMAIN);
        write_field(&mut h, self.schema.as_bytes());
        write_field(&mut h, self.tool_version.as_bytes());
        write_field(&mut h, self.payload_hash.as_bytes());
        write_opt(&mut h, self.proposal_account.as_deref());
        h.update((self.program_ids.len() as u64).to_le_bytes());
        for p in &self.program_ids {
            write_field(&mut h, p.as_bytes());
        }
        write_field(&mut h, self.network.as_bytes());
        write_field(&mut h, self.input_source.as_bytes());
        // Provenance: endpoints, slots, agreement flag.
        h.update((self.provenance.endpoints.len() as u64).to_le_bytes());
        for e in &self.provenance.endpoints {
            write_field(&mut h, e.as_bytes());
        }
        for s in &self.provenance.slots {
            h.update(s.to_le_bytes());
        }
        h.update([self.provenance.full_agreement as u8]);
        h.update([match self.fidelity_grade {
            FidelityGrade::A => b'A',
            FidelityGrade::B => b'B',
        }]);
        write_field(&mut h, self.engine.as_bytes());
        write_field(&mut h, self.pre_state_root.as_bytes());
        write_field(&mut h, self.post_state_root.as_bytes());
        h.update((self.closure_account_count as u64).to_le_bytes());
        h.update(self.compute_units.to_le_bytes());
        h.update([self.success as u8]);
        // Bind the opaque findings by their canonical bytes.
        let findings_bytes = serde_json::to_vec(&self.findings).unwrap_or_default();
        write_field(&mut h, &findings_bytes);
        write_opt(&mut h, self.prev_bundle_hash.as_deref());
        h.update(self.created_at.timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        h.finalize().into()
    }
}

/// A manifest plus the Ed25519 signature that binds a rehearser to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceBundle {
    /// The signed claim.
    pub manifest: EvidenceManifest,
    /// Base58 public key of the rehearser.
    pub attester: String,
    /// Base58 Ed25519 signature over `manifest.signing_digest()`.
    pub signature: String,
}

impl EvidenceBundle {
    /// Sign `manifest` with `signer`, producing the artifact.
    pub fn create(manifest: EvidenceManifest, signer: &Keypair) -> Self {
        let digest = manifest.signing_digest();
        let signature = signer.sign_message(&digest);
        Self {
            manifest,
            attester: signer.pubkey().to_string(),
            signature: signature.to_string(),
        }
    }

    /// The bundle's own content hash (hex) - the link a successor bundle stores
    /// in `prev_bundle_hash` to form a per-subject chain of custody.
    pub fn bundle_hash(&self) -> String {
        hex::encode(self.manifest.signing_digest())
    }

    fn attester_pubkey(&self) -> Result<Pubkey> {
        Pubkey::from_str(&self.attester).map_err(|_| AttestError::BadPubkey(self.attester.clone()))
    }

    fn parse_signature(&self) -> Result<Signature> {
        let bytes = bs58::decode(&self.signature)
            .into_vec()
            .map_err(|_| AttestError::BadSignature)?;
        Signature::try_from(bytes.as_slice()).map_err(|_| AttestError::BadSignature)
    }

    /// Verify the signature: does `attester` actually attest this manifest?
    pub fn verify_signature(&self) -> Result<bool> {
        let pubkey = self.attester_pubkey()?;
        let signature = self.parse_signature()?;
        Ok(signature.verify(&pubkey.to_bytes(), &self.manifest.signing_digest()))
    }

    /// Confirm the bundle's `pre_state_root` is the Merkle root of `closure` -
    /// i.e. the payload really was rehearsed against *this* set of accounts. A
    /// signer runs this against a closure re-fetched from their **own** RPC, so
    /// a compromised proposer cannot substitute the state that was rehearsed.
    pub fn verify_pre_state(&self, closure: &[AccountEntry]) -> EvidenceCheck {
        let actual = hex::encode(state_root(closure));
        EvidenceCheck {
            signature_valid: self.verify_signature().unwrap_or(false),
            pre_state_matches: actual == self.manifest.pre_state_root,
            account_count_matches: closure.len() == self.manifest.closure_account_count,
            expected_pre_state_root: self.manifest.pre_state_root.clone(),
            actual_pre_state_root: actual,
            attester: self.attester.clone(),
        }
    }

    /// Serialize to pretty JSON for an artifact file.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| AttestError::Serialization(e.to_string()))
    }

    /// Parse from JSON.
    pub fn from_json(s: &str) -> Result<Self> {
        serde_json::from_str(s).map_err(|e| AttestError::Serialization(e.to_string()))
    }
}

/// The outcome of the offline checks a signer can run without a runtime.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceCheck {
    /// The Ed25519 signature is valid for `attester` over the manifest.
    pub signature_valid: bool,
    /// The recomputed closure root equals the bundle's `pre_state_root`.
    pub pre_state_matches: bool,
    /// The supplied closure size equals the bundle's claimed count.
    pub account_count_matches: bool,
    /// The root the bundle claims to have rehearsed against.
    pub expected_pre_state_root: String,
    /// The root recomputed from the supplied closure.
    pub actual_pre_state_root: String,
    /// The rehearser whose signature was checked.
    pub attester: String,
}

impl EvidenceCheck {
    /// Whether every offline check passed. (Full trust additionally requires the
    /// re-execution check in `rustag-rehearse`, which reproduces `post_state_root`.)
    pub fn is_valid(&self) -> bool {
        self.signature_valid && self.pre_state_matches && self.account_count_matches
    }
}

/// The SHA-256 (hex) of arbitrary payload bytes, used for `payload_hash`.
pub fn payload_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Length-prefix a variable-length field so concatenation is unambiguous.
fn write_field(h: &mut Sha256, bytes: &[u8]) {
    h.update((bytes.len() as u64).to_le_bytes());
    h.update(bytes);
}

/// Length-prefix an optional field with a presence byte, so `None` and
/// `Some("")` never encode identically.
fn write_opt(h: &mut Sha256, value: Option<&str>) {
    match value {
        Some(s) => {
            h.update([1u8]);
            write_field(h, s.as_bytes());
        }
        None => h.update([0u8]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustag_core::AccountSync;
    use uuid::Uuid;

    fn closure() -> Vec<AccountEntry> {
        (0u8..3)
            .map(|i| AccountEntry {
                pubkey: Pubkey::new_from_array([i + 1; 32]),
                data: vec![i; (i as usize) + 1],
                owner: Pubkey::new_from_array([0xAB; 32]),
                lamports: 1000 * (i as u64 + 1),
                executable: false,
                rent_epoch: 0,
                sync_state: AccountSync::clean_now(),
                category: None,
                stagenet_id: Uuid::nil(),
            })
            .collect()
    }

    fn manifest_for(closure: &[AccountEntry]) -> EvidenceManifest {
        EvidenceManifest {
            schema: EVIDENCE_SCHEMA.to_string(),
            tool_version: "0.1.0".to_string(),
            payload_hash: payload_hash(b"rotate-upgrade-authority"),
            proposal_account: Some("Prop1111111111111111111111111111111111111111".to_string()),
            program_ids: vec!["Prog111111111111111111111111111111111111111".to_string()],
            network: "mainnet-beta".to_string(),
            input_source: "offline".to_string(),
            provenance: InputProvenance {
                endpoints: vec!["helius".into(), "triton".into()],
                slots: vec![100, 100],
                full_agreement: true,
            },
            fidelity_grade: FidelityGrade::A,
            engine: "litesvm-0.12".to_string(),
            pre_state_root: hex::encode(state_root(closure)),
            post_state_root: "cc".repeat(32),
            closure_account_count: closure.len(),
            compute_units: 4200,
            success: true,
            findings: serde_json::json!({ "alarms": [{ "rule": "upgrade-authority-rotated" }] }),
            prev_bundle_hash: None,
            created_at: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        }
    }

    #[test]
    fn create_and_verify_roundtrip() {
        let acc = closure();
        let kp = Keypair::new();
        let bundle = EvidenceBundle::create(manifest_for(&acc), &kp);

        assert!(bundle.verify_signature().unwrap());
        let check = bundle.verify_pre_state(&acc);
        assert!(check.is_valid(), "{check:?}");
        assert_eq!(check.attester, kp.pubkey().to_string());
    }

    #[test]
    fn tampering_any_field_breaks_the_signature() {
        let acc = closure();
        let kp = Keypair::new();
        let mut bundle = EvidenceBundle::create(manifest_for(&acc), &kp);
        bundle.manifest.post_state_root = "dd".repeat(32);
        assert!(!bundle.verify_signature().unwrap());
    }

    #[test]
    fn tampering_findings_breaks_the_signature() {
        // The whole point: a lying UI that keeps the state roots but rewrites the
        // human-readable findings must not verify.
        let acc = closure();
        let kp = Keypair::new();
        let mut bundle = EvidenceBundle::create(manifest_for(&acc), &kp);
        bundle.manifest.findings = serde_json::json!({ "alarms": [] });
        assert!(!bundle.verify_signature().unwrap());
    }

    #[test]
    fn substituted_closure_fails_pre_state_check() {
        let acc = closure();
        let kp = Keypair::new();
        let bundle = EvidenceBundle::create(manifest_for(&acc), &kp);

        let mut other = acc.clone();
        other[0].lamports += 1; // a different closure than was rehearsed
        let check = bundle.verify_pre_state(&other);
        assert!(check.signature_valid);
        assert!(!check.pre_state_matches);
        assert!(!check.is_valid());
    }

    #[test]
    fn json_artifact_roundtrips_and_still_verifies() {
        let acc = closure();
        let kp = Keypair::new();
        let bundle = EvidenceBundle::create(manifest_for(&acc), &kp);
        let json = bundle.to_json().unwrap();
        let back = EvidenceBundle::from_json(&json).unwrap();
        assert_eq!(bundle, back);
        assert!(back.verify_pre_state(&acc).is_valid());
    }

    #[test]
    fn digest_is_stable() {
        let acc = closure();
        assert_eq!(
            manifest_for(&acc).signing_digest(),
            manifest_for(&acc).signing_digest()
        );
    }
}
