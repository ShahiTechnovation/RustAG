//! RustAG verifiable staging attestation (Phase 3, P3.1).
//!
//! This crate turns "we tested our program against staged mainnet state" from
//! an unverifiable claim into a cryptographic artifact:
//!
//! - [`state_root`] / [`MerkleTree`] commit to an exact account set with a
//!   SHA-256 Merkle root, plus per-account inclusion proofs.
//! - [`AttestationManifest`] is the signable claim (what stagenet, what state
//!   root, which programs, which transaction outcomes).
//! - [`Attestation`] binds a manifest to an Ed25519 [`solana_keypair::Keypair`]
//!   and verifies **offline** with nothing but the account set and the pubkey.
//! - [`AuditLog`] is a tamper-evident, hash-chained activity log (SOC 2
//!   groundwork) where any edit/insert/delete is detectable.
//!
//! ```
//! use rustag_attest::{Attestation, AttestationManifest};
//! use solana_keypair::Keypair;
//! # use rustag_core::AccountEntry;
//!
//! # fn demo(accounts: &[AccountEntry]) {
//! let manifest = AttestationManifest::build(
//!     uuid::Uuid::new_v4(), "my-stagenet", "https://rpc...", "mainnet-beta",
//!     1234, accounts, vec!["MyProgram...".into()], &[],
//! );
//! let key = Keypair::new();
//! let attestation = Attestation::create(manifest, &key);
//!
//! // Anyone can verify, offline, that this exact state was attested:
//! let report = attestation.verify_against(accounts).unwrap();
//! assert!(report.is_valid());
//! # }
//! ```

mod attestation;
mod audit;
mod error;
mod evidence;
mod manifest;
mod merkle;
mod state;

pub use attestation::{Attestation, VerificationReport};
pub use audit::{AuditEntry, AuditLog};
pub use error::{AttestError, Result};
pub use evidence::{
    payload_hash, EvidenceBundle, EvidenceCheck, EvidenceManifest, FidelityGrade, InputProvenance,
    EVIDENCE_SCHEMA,
};
pub use manifest::{AttestationManifest, MANIFEST_SCHEMA};
pub use merkle::{
    decode_hash, hash_leaf, hash_nodes, verify_proof, Hash32, MerkleProof, MerkleTree, EMPTY_ROOT,
};
pub use state::{
    account_leaf_hash, encode_account_leaf, encode_tx_result, state_root, state_tree,
    tx_results_root,
};
