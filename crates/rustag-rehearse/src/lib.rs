//! GroundTruth **sealed pre-execution rehearsal**.
//!
//! This is the wedge of the [pivot](../../../docs/PIVOT_PLAN.md): turn a *proposed*
//! privileged Solana payload - a Squads multisig proposal, a program upgrade, a
//! treasury move - into a signed, offline-verifiable [`EvidenceBundle`] that says
//! exactly what it will do before anyone signs it.
//!
//! ## The two-pass algorithm ([`SealedRehearsal::run`])
//!
//! 1. **Discovery.** On a working stagenet with impersonation on (so an unsigned
//!    proposal can execute), simulate the payload to fault in the full account
//!    closure it touches, then dereference every upgradeable program it invokes
//!    (so the real bytecode runs and its upgrade authority is visible).
//! 2. **Execute.** [`Checkpoint::capture`] freezes the fully-populated *pre*-state
//!    (self-contained, content-addressed). Restore it into an isolated offline
//!    fork, [`execute_and_record`] the payload into a [`Journal`], and read the
//!    *post*-state.
//!
//! Because the checkpoint is offline and self-contained, an independent replay
//! reproduces the exact post-state root - so the bundle is **Grade A**
//! (deterministically re-executable by any verifier). The diff is then decoded
//! into human claims ([`rustag_sim::decode_changes`]), evaluated against an
//! invariant [`Policy`], and scanned for exploit shapes, and the whole thing is
//! Ed25519-signed.
//!
//! A signer independently confirms the bundle with [`verify_bundle`], which
//! re-executes the payload against a closure re-fetched from their *own* RPC -
//! so a compromised proposer UI cannot slip a different payload past them.

use std::collections::HashSet;

use rustag_attest::{
    payload_hash, state_root, EvidenceBundle, EvidenceCheck, EvidenceManifest, FidelityGrade,
    InputProvenance, EVIDENCE_SCHEMA,
};
use rustag_core::{AccountEntry, Stagenet};
use rustag_replay::{diff_accounts, execute_and_record, Checkpoint, Journal};
use rustag_sim::{decode_changes, scan_outcomes, Alarm, Policy, SemanticChange};
use solana_keypair::Keypair;
use solana_message::VersionedMessage;
use solana_pubkey::Pubkey;
use solana_transaction::versioned::VersionedTransaction;

mod error;
mod portable;
pub use error::{RehearseError, Result};
pub use portable::{from_portable, to_portable, PortableAccount};

/// The engine identifier stamped into every bundle's `environment.engine`.
const ENGINE: &str = "litesvm-0.12";

/// Inputs to a rehearsal beyond the payload itself.
#[derive(Debug, Clone, Default)]
pub struct RehearsalOptions {
    /// The on-chain proposal account this payload was decoded from, if any.
    pub proposal_account: Option<String>,
    /// Network being rehearsed (e.g. `mainnet-beta`, or `offline`).
    pub network: String,
    /// Mainnet source (redacted RPC url, or `offline`).
    pub input_source: String,
    /// Cross-fetch provenance of the closure.
    pub provenance: InputProvenance,
    /// Pin the clock at `(slot, unix_timestamp)` for time-dependent logic.
    pub clock: Option<(u64, i64)>,
    /// Link to the previous bundle in this subject's chain of custody.
    pub prev_bundle_hash: Option<String>,
}

impl RehearsalOptions {
    /// Options for an offline (already-mirrored / in-memory) rehearsal.
    pub fn offline() -> Self {
        Self {
            network: "offline".to_string(),
            input_source: "offline".to_string(),
            ..Default::default()
        }
    }
}

/// The result of a rehearsal: the signed bundle plus what a caller needs to
/// display or hand to a verifier.
pub struct Rehearsal {
    /// The signed, offline-verifiable evidence bundle.
    pub bundle: EvidenceBundle,
    /// The exact pre-state closure the payload executed against (what a verifier
    /// re-fetches and checks against `pre_state_root`).
    pub closure: Vec<AccountEntry>,
    /// Whether the bundle is Grade A (deterministically re-executable).
    pub grade_a: bool,
    /// The invariant alarms raised, in rule order.
    pub alarms: Vec<Alarm>,
    /// The decoded, human-legible account changes.
    pub semantic_diff: Vec<SemanticChange>,
}

impl Rehearsal {
    /// Number of invariant alarms raised.
    pub fn alarm_count(&self) -> usize {
        self.alarms.len()
    }

    /// The most severe alarm severity raised, if any.
    pub fn max_alarm_severity(&self) -> Option<rustag_sim::Severity> {
        self.alarms.iter().map(|a| a.severity).max()
    }
}

/// The sealed two-pass rehearsal.
pub struct SealedRehearsal;

impl SealedRehearsal {
    /// Rehearse `payload` on `working` (an offline in-memory or mirror-enabled,
    /// primed stagenet) under `policy`, signing the resulting bundle with `signer`.
    pub async fn run(
        mut working: Stagenet,
        payload: VersionedTransaction,
        opts: RehearsalOptions,
        policy: &Policy,
        signer: &Keypair,
    ) -> Result<Rehearsal> {
        // Impersonation lets an unsigned proposed payload execute.
        working.enable_impersonation();

        // Pass 1 - discovery: fault in the closure the payload touches.
        let _ = working.simulate_transaction(payload.clone()).await?;

        // Dereference every upgradeable program the payload invokes so its real
        // bytecode runs and its ProgramData (upgrade authority) is in the closure.
        let program_pubkeys = instruction_program_ids(&payload.message);
        for pid in &program_pubkeys {
            working.load_upgradeable_program(pid).await?;
        }

        // Pin the clock, if requested.
        if let Some((slot, ts)) = opts.clock {
            working.sync_clock(slot, ts);
        }

        // Freeze the fully-populated pre-state (self-contained, content-addressed).
        let pre = Checkpoint::capture(&working).await?;

        // Pass 2 - execute on an isolated offline restore, recording it.
        let mut exec = pre.restore("groundtruth-exec").await?;
        exec.enable_impersonation();
        let mut journal = Journal::new();
        let outcome = execute_and_record(&mut journal, &mut exec, "payload", payload.clone()).await?;
        let post_accounts = exec.export_accounts().await?;
        let post_state_root = hex::encode(state_root(&post_accounts));

        // Grade A: an independent impersonated replay reproduces the post root.
        let grade_a = impersonated_replay_reproduces(&pre, &journal, &post_state_root).await?;

        // Analyze the diff.
        let raw_diff = diff_accounts(&pre.accounts, &post_accounts);
        let semantic = decode_changes(&pre.accounts, &post_accounts);
        let alarms = policy.evaluate(&pre.accounts, &post_accounts);
        let scan = scan_outcomes(&[outcome.clone()]);

        let findings = serde_json::json!({
            "semanticDiff": &semantic,
            "alarms": &alarms,
            "scan": &scan,
            "rawDiff": &raw_diff,
        });

        let payload_bytes =
            bincode::serialize(&payload).map_err(|e| RehearseError::Other(e.to_string()))?;

        let manifest = EvidenceManifest {
            schema: EVIDENCE_SCHEMA.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            payload_hash: payload_hash(&payload_bytes),
            proposal_account: opts.proposal_account,
            program_ids: program_pubkeys.iter().map(|p| p.to_string()).collect(),
            network: opts.network,
            input_source: opts.input_source,
            provenance: opts.provenance,
            fidelity_grade: if grade_a {
                FidelityGrade::A
            } else {
                FidelityGrade::B
            },
            engine: ENGINE.to_string(),
            pre_state_root: pre.state_root.clone(),
            post_state_root,
            closure_account_count: pre.accounts.len(),
            compute_units: outcome.compute_units,
            success: outcome.success,
            findings,
            prev_bundle_hash: opts.prev_bundle_hash,
            created_at: chrono::Utc::now(),
        };

        let bundle = EvidenceBundle::create(manifest, signer);
        Ok(Rehearsal {
            bundle,
            closure: pre.accounts,
            grade_a,
            alarms,
            semantic_diff: semantic,
        })
    }
}

/// The outcome of independently verifying a bundle, signer-side.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullVerification {
    /// The offline signature + input-closure checks.
    pub offline: EvidenceCheck,
    /// Whether re-executing the payload reproduced the claimed post-state root.
    pub post_state_matches: bool,
    /// The post-state root this verifier recomputed.
    pub recomputed_post_state_root: String,
}

impl FullVerification {
    /// Whether every check passed - the bundle is fully trustworthy to this
    /// verifier's own view of mainnet.
    pub fn is_valid(&self) -> bool {
        self.offline.is_valid() && self.post_state_matches
    }
}

/// Independently verify a bundle: confirm the signature and that the payload,
/// re-executed against `closure`, reproduces the claimed `post_state_root`.
///
/// The signer passes a `closure` re-fetched from their **own** RPC (and the
/// `payload` decoded from the **on-chain** proposal account), so a compromised
/// proposer cannot make a bundle pass for a payload that does something else.
pub async fn verify_bundle(
    bundle: &EvidenceBundle,
    closure: &[AccountEntry],
    payload: &VersionedTransaction,
) -> Result<FullVerification> {
    let offline = bundle.verify_pre_state(closure);

    // Re-execute the payload against the closure on a fresh isolated runtime.
    let mut sn = Stagenet::local("groundtruth-verify").await?;
    sn.import_accounts(closure).await?;
    sn.enable_impersonation();
    let program_pubkeys = instruction_program_ids(&payload.message);
    for pid in &program_pubkeys {
        sn.load_upgradeable_program(pid).await?;
    }
    sn.send_transaction(payload.clone()).await?;
    let recomputed = hex::encode(state_root(&sn.export_accounts().await?));

    Ok(FullVerification {
        post_state_matches: recomputed == bundle.manifest.post_state_root,
        recomputed_post_state_root: recomputed,
        offline,
    })
}

/// Two independent impersonated replays from `pre` must agree with each other
/// and reproduce `expected_post` - the Grade-A property.
async fn impersonated_replay_reproduces(
    pre: &Checkpoint,
    journal: &Journal,
    expected_post: &str,
) -> Result<bool> {
    let a = impersonated_replay_root(pre, journal, "groundtruth-ga1").await?;
    let b = impersonated_replay_root(pre, journal, "groundtruth-ga2").await?;
    Ok(a == b && a == expected_post)
}

async fn impersonated_replay_root(
    pre: &Checkpoint,
    journal: &Journal,
    name: &str,
) -> Result<String> {
    let mut sn = pre.restore(name).await?;
    sn.enable_impersonation();
    journal.replay_onto(&mut sn).await?;
    Ok(hex::encode(state_root(&sn.export_accounts().await?)))
}

/// The distinct program ids a message's instructions invoke, in first-seen order.
fn instruction_program_ids(message: &VersionedMessage) -> Vec<Pubkey> {
    let keys = message.static_account_keys();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for ix in message.instructions() {
        if let Some(pubkey) = keys.get(ix.program_id_index as usize) {
            if seen.insert(*pubkey) {
                out.push(*pubkey);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_signer::Signer;
    use solana_system_interface::instruction::transfer;
    use solana_transaction::Transaction;

    /// A funded offline stagenet plus its payer and a watched treasury.
    async fn funded(name: &str) -> (Stagenet, Keypair, Pubkey) {
        let mut sn = Stagenet::local(name).await.unwrap();
        let payer = Keypair::new();
        sn.airdrop(&payer.pubkey(), 10_000_000_000).await.unwrap();
        let treasury = payer.pubkey();
        (sn, payer, treasury)
    }

    fn transfer_payload(payer: &Keypair, to: &Pubkey, lamports: u64, bh: solana_hash::Hash) -> VersionedTransaction {
        let ix = transfer(&payer.pubkey(), to, lamports);
        let msg = Message::new(&[ix], Some(&payer.pubkey()));
        Transaction::new(&[payer], msg, bh).into()
    }

    #[tokio::test]
    async fn rehearsal_is_grade_a_and_verifies_offline() {
        let (working, payer, _) = funded("rh-ok").await;
        let bh = working.latest_blockhash();
        let to = Pubkey::new_from_array([9; 32]);
        let payload = transfer_payload(&payer, &to, 1_000_000_000, bh);

        let signer = Keypair::new();
        let rehearsal = SealedRehearsal::run(
            working,
            payload.clone(),
            RehearsalOptions::offline(),
            &Policy::standard(),
            &signer,
        )
        .await
        .unwrap();

        assert!(rehearsal.grade_a, "a self-contained transfer must be Grade A");
        assert_eq!(rehearsal.bundle.manifest.fidelity_grade, FidelityGrade::A);
        assert!(rehearsal.bundle.verify_signature().unwrap());

        // A verifier reproduces the exact result from the closure + payload.
        let full = verify_bundle(&rehearsal.bundle, &rehearsal.closure, &payload)
            .await
            .unwrap();
        assert!(full.is_valid(), "{full:?}");
    }

    #[tokio::test]
    async fn treasury_floor_breach_raises_an_alarm() {
        let (working, payer, treasury) = funded("rh-alarm").await;
        let bh = working.latest_blockhash();
        let to = Pubkey::new_from_array([9; 32]);
        // Move almost everything out of the treasury.
        let payload = transfer_payload(&payer, &to, 9_500_000_000, bh);

        let policy = Policy::new().rule(
            "treasury-floor",
            rustag_sim::balance_floor(treasury, 5_000_000_000),
        );
        let signer = Keypair::new();
        let rehearsal =
            SealedRehearsal::run(working, payload, RehearsalOptions::offline(), &policy, &signer)
                .await
                .unwrap();

        assert!(rehearsal.alarm_count() >= 1, "the floor breach must alarm");
        let findings = &rehearsal.bundle.manifest.findings;
        let alarms = findings["alarms"].as_array().unwrap();
        assert!(alarms
            .iter()
            .any(|a| a["rule"] == "balance-below-floor"));
    }

    #[tokio::test]
    async fn tampered_closure_fails_verification() {
        let (working, payer, _) = funded("rh-tamper").await;
        let bh = working.latest_blockhash();
        let to = Pubkey::new_from_array([9; 32]);
        let payload = transfer_payload(&payer, &to, 1_000_000_000, bh);

        let signer = Keypair::new();
        let rehearsal = SealedRehearsal::run(
            working,
            payload.clone(),
            RehearsalOptions::offline(),
            &Policy::standard(),
            &signer,
        )
        .await
        .unwrap();

        // Present a closure that differs from what was rehearsed.
        let mut bad = rehearsal.closure.clone();
        bad[0].lamports += 1;
        let full = verify_bundle(&rehearsal.bundle, &bad, &payload).await.unwrap();
        assert!(!full.offline.pre_state_matches);
        assert!(!full.is_valid());
    }
}
