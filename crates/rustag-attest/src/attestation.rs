//! The signed attestation artifact and its offline verification.

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_signer::Signer;

use rustag_core::AccountEntry;

use crate::error::{AttestError, Result};
use crate::manifest::AttestationManifest;
use crate::state::state_root;

/// A manifest plus the Ed25519 signature that binds an attester to it.
///
/// This is the artifact written to `*.attestation.json`. It verifies offline
/// with nothing but its own contents and the account set it claims to commit to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attestation {
    /// The signed claim.
    pub manifest: AttestationManifest,
    /// Base58 public key of the attester.
    pub attester: String,
    /// Base58 Ed25519 signature over `manifest.signing_digest()`.
    pub signature: String,
}

impl Attestation {
    /// Sign `manifest` with `signer`, producing the artifact.
    pub fn create(manifest: AttestationManifest, signer: &Keypair) -> Self {
        let digest = manifest.signing_digest();
        let signature = signer.sign_message(&digest);
        Self {
            manifest,
            attester: signer.pubkey().to_string(),
            signature: signature.to_string(),
        }
    }

    /// Parse the attester field into a public key.
    pub fn attester_pubkey(&self) -> Result<Pubkey> {
        Pubkey::from_str(&self.attester).map_err(|_| AttestError::BadPubkey(self.attester.clone()))
    }

    /// Parse the base58 signature field into a [`Signature`].
    fn parse_signature(&self) -> Result<Signature> {
        let bytes = bs58::decode(&self.signature)
            .into_vec()
            .map_err(|_| AttestError::BadSignature)?;
        Signature::try_from(bytes.as_slice()).map_err(|_| AttestError::BadSignature)
    }

    /// Verify only the signature: does `attester` actually attest this manifest?
    pub fn verify_signature(&self) -> Result<bool> {
        let pubkey = self.attester_pubkey()?;
        let signature = self.parse_signature()?;
        let digest = self.manifest.signing_digest();
        Ok(signature.verify(&pubkey.to_bytes(), &digest))
    }

    /// Full verification against a concrete account set: recompute the state
    /// root from `accounts`, confirm it matches the manifest, and check the
    /// signature. This is what a reviewer runs to confirm "this program was
    /// tested against *exactly* this state".
    pub fn verify_against(&self, accounts: &[AccountEntry]) -> Result<VerificationReport> {
        let actual_root = hex::encode(state_root(accounts));
        Ok(VerificationReport {
            signature_valid: self.verify_signature()?,
            state_root_matches: actual_root == self.manifest.state_root,
            account_count_matches: accounts.len() == self.manifest.account_count,
            expected_state_root: self.manifest.state_root.clone(),
            actual_state_root: actual_root,
            attester: self.attester.clone(),
        })
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

/// The outcome of verifying an attestation against an account set.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReport {
    /// The Ed25519 signature is valid for `attester` over the manifest.
    pub signature_valid: bool,
    /// The recomputed state root equals the manifest's claimed root.
    pub state_root_matches: bool,
    /// The supplied account count equals the manifest's claimed count.
    pub account_count_matches: bool,
    /// The root the manifest claims.
    pub expected_state_root: String,
    /// The root recomputed from the supplied accounts.
    pub actual_state_root: String,
    /// The attester whose signature was checked.
    pub attester: String,
}

impl VerificationReport {
    /// Whether every check passed - the attestation is fully valid.
    pub fn is_valid(&self) -> bool {
        self.signature_valid && self.state_root_matches && self.account_count_matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustag_core::{AccountEntry, AccountSync};
    use uuid::Uuid;

    fn accounts() -> Vec<AccountEntry> {
        (0u8..4)
            .map(|i| AccountEntry {
                pubkey: Pubkey::new_from_array([i + 1; 32]),
                data: vec![i; (i as usize) + 1],
                owner: Pubkey::new_from_array([0xAA; 32]),
                lamports: 1000 * (i as u64 + 1),
                executable: false,
                rent_epoch: 0,
                sync_state: AccountSync::clean_now(),
                category: None,
                stagenet_id: Uuid::nil(),
            })
            .collect()
    }

    fn manifest_for(accs: &[AccountEntry]) -> AttestationManifest {
        AttestationManifest::build(
            Uuid::from_u128(7),
            "demo",
            "offline",
            "mainnet-beta",
            100,
            accs,
            vec!["MyProgram1111111111111111111111111111111111".to_string()],
            &[],
        )
    }

    #[test]
    fn create_and_verify_roundtrip() {
        let accs = accounts();
        let kp = Keypair::new();
        let att = Attestation::create(manifest_for(&accs), &kp);

        assert!(att.verify_signature().unwrap());
        let report = att.verify_against(&accs).unwrap();
        assert!(report.is_valid(), "{report:?}");
        assert_eq!(report.attester, kp.pubkey().to_string());
    }

    #[test]
    fn tampered_manifest_breaks_the_signature() {
        let accs = accounts();
        let kp = Keypair::new();
        let mut att = Attestation::create(manifest_for(&accs), &kp);
        att.manifest.slot = 999; // forge a field after signing
        assert!(!att.verify_signature().unwrap());
    }

    #[test]
    fn mismatched_state_set_fails_verification() {
        let accs = accounts();
        let kp = Keypair::new();
        let att = Attestation::create(manifest_for(&accs), &kp);

        let mut other = accs.clone();
        other[0].lamports += 1; // a different state than was attested
        let report = att.verify_against(&other).unwrap();
        assert!(report.signature_valid);
        assert!(!report.state_root_matches);
        assert!(!report.is_valid());
    }

    #[test]
    fn json_artifact_roundtrips_and_still_verifies() {
        let accs = accounts();
        let kp = Keypair::new();
        let att = Attestation::create(manifest_for(&accs), &kp);
        let json = att.to_json().unwrap();
        let back = Attestation::from_json(&json).unwrap();
        assert_eq!(att, back);
        assert!(back.verify_against(&accs).unwrap().is_valid());
    }

    #[test]
    fn a_different_signer_is_rejected() {
        let accs = accounts();
        let real = Keypair::new();
        let attacker = Keypair::new();
        let mut att = Attestation::create(manifest_for(&accs), &real);
        // Claim the attacker signed it, keep the real signature.
        att.attester = attacker.pubkey().to_string();
        assert!(!att.verify_signature().unwrap());
    }
}
