//! `rustag verify` - verify a staging attestation or EvidenceBundle offline.

use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use clap::Args;
use console::style;

use rustag_attest::{Attestation, EvidenceBundle};
use rustag_rehearse::{from_portable, verify_bundle, PortableAccount};
use solana_transaction::versioned::VersionedTransaction;

use super::{fail, info, load_all_accounts, ok, open_store, resolve_record};

#[derive(Args)]
pub struct VerifyArgs {
    /// Path to the attestation or EvidenceBundle JSON to verify.
    pub file: PathBuf,
    /// Verify the state root against a stagenet's current accounts (defaults to
    /// the stagenet named in the attestation, if it exists locally).
    #[arg(short, long)]
    pub stagenet: Option<String>,
    /// Only check the signature; skip recomputing the state root.
    #[arg(long)]
    pub signature_only: bool,
    /// EvidenceBundle: portable closure JSON to check `pre_state_root` against
    /// (ideally re-fetched from your own RPC).
    #[arg(long)]
    pub closure: Option<PathBuf>,
    /// EvidenceBundle: base64 payload to re-execute for full verification.
    #[arg(long)]
    pub message: Option<String>,
}

pub async fn run(args: VerifyArgs) -> Result<()> {
    let raw = std::fs::read_to_string(&args.file)
        .with_context(|| format!("failed to read {}", args.file.display()))?;

    // An EvidenceBundle and an Attestation are distinct artifacts; dispatch on
    // the evidence schema so `verify` handles both.
    if let Ok(bundle) = EvidenceBundle::from_json(&raw) {
        if bundle.manifest.schema.starts_with("groundtruth.evidence") {
            return verify_evidence(&args, bundle).await;
        }
    }

    let attestation =
        Attestation::from_json(&raw).map_err(|e| anyhow!("not a valid attestation file: {e}"))?;

    println!();
    info(format!("attester    {}", attestation.attester));
    info(format!(
        "stagenet    {}",
        attestation.manifest.stagenet_name
    ));
    info(format!("state root  {}", attestation.manifest.state_root));
    info(format!(
        "accounts    {}   transactions {}",
        attestation.manifest.account_count, attestation.manifest.tx_count
    ));

    let signature_valid = attestation
        .verify_signature()
        .map_err(|e| anyhow!("signature check failed: {e}"))?;
    print_check("signature", signature_valid);

    if args.signature_only {
        return finish(signature_valid);
    }

    // Verify the committed state root against a local stagenet. If the user
    // *explicitly* named a stagenet, an unresolved target is a hard error - we
    // must never report VALID after silently skipping the state-root check the
    // user asked for. Only the implicit (manifest-derived) target may fall back
    // to signature-only.
    let explicit = args.stagenet.is_some();
    let target = args
        .stagenet
        .clone()
        .or_else(|| Some(attestation.manifest.stagenet_name.clone()));
    let store = open_store().await.ok();

    if let (Some(store), Some(name)) = (&store, &target) {
        match resolve_record(store, Some(name)).await {
            Ok(record) => {
                let accounts = load_all_accounts(store, &record.id).await?;
                let report = attestation
                    .verify_against(&accounts)
                    .map_err(|e| anyhow!("verification failed: {e}"))?;
                print_check("state root matches", report.state_root_matches);
                print_check("account count matches", report.account_count_matches);
                if !report.state_root_matches {
                    info(format!("  expected {}", report.expected_state_root));
                    info(format!("  actual   {}", report.actual_state_root));
                }
                return finish(report.is_valid());
            }
            Err(e) if explicit => {
                bail!("--stagenet '{name}' could not be resolved, so the state root was NOT checked: {e}");
            }
            Err(_) => {} // implicit target: fall through to signature-only below
        }
    } else if explicit {
        bail!(
            "could not open the local store to verify against stagenet '{}'",
            target.unwrap_or_default()
        );
    }

    info("no matching local stagenet found - verified signature only (state root NOT checked)");
    finish(signature_valid)
}

/// Verify a GroundTruth EvidenceBundle: signature, then (if a closure is given)
/// the `pre_state_root`, then (if a payload is also given) full re-execution.
async fn verify_evidence(args: &VerifyArgs, bundle: EvidenceBundle) -> Result<()> {
    let m = &bundle.manifest;
    println!();
    info(format!("attester    {}", bundle.attester));
    info(format!(
        "grade       {:?}   engine {}",
        m.fidelity_grade, m.engine
    ));
    info(format!("pre  root   {}", m.pre_state_root));
    info(format!("post root   {}", m.post_state_root));
    info(format!(
        "programs    {}   compute {} CU",
        m.program_ids.len(),
        m.compute_units
    ));

    let signature_valid = bundle
        .verify_signature()
        .map_err(|e| anyhow!("signature check failed: {e}"))?;
    print_check("signature", signature_valid);

    if args.signature_only {
        return finish_evidence(signature_valid);
    }

    // Without a closure we cannot check the state the bundle rehearsed against.
    let Some(closure_path) = &args.closure else {
        info("no --closure given - verified signature only (state NOT checked)");
        return finish_evidence(signature_valid);
    };

    let closure_raw = std::fs::read_to_string(closure_path)
        .with_context(|| format!("failed to read {}", closure_path.display()))?;
    let portable: Vec<PortableAccount> =
        serde_json::from_str(&closure_raw).context("closure file is not a portable-account array")?;
    let closure = from_portable(&portable).map_err(|e| anyhow!("bad closure: {e}"))?;

    let check = bundle.verify_pre_state(&closure);
    print_check("pre-state matches", check.pre_state_matches);
    print_check("account count matches", check.account_count_matches);
    if !check.pre_state_matches {
        info(format!("  expected {}", check.expected_pre_state_root));
        info(format!("  actual   {}", check.actual_pre_state_root));
    }

    // With the payload too, re-execute for full verification.
    if let Some(message) = &args.message {
        let payload = decode_payload(message)?;
        let full = verify_bundle(&bundle, &closure, &payload)
            .await
            .map_err(|e| anyhow!("re-execution failed: {e}"))?;
        print_check("re-execution reproduces post root", full.post_state_matches);
        if !full.post_state_matches {
            info(format!("  recomputed {}", full.recomputed_post_state_root));
        }
        return finish_evidence(full.is_valid());
    }

    finish_evidence(check.is_valid())
}

fn decode_payload(base64_tx: &str) -> Result<VersionedTransaction> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_tx.trim())
        .context("--message is not valid base64")?;
    bincode::deserialize(&bytes).context("--message is not a bincode VersionedTransaction")
}

fn finish_evidence(valid: bool) -> Result<()> {
    println!();
    if valid {
        ok("EvidenceBundle is VALID");
        Ok(())
    } else {
        bail!("EvidenceBundle is INVALID");
    }
}

fn print_check(label: &str, passed: bool) {
    if passed {
        ok(format!("{label}: {}", style("valid").green()));
    } else {
        fail(format!("{label}: {}", style("INVALID").red().bold()));
    }
}

fn finish(valid: bool) -> Result<()> {
    println!();
    if valid {
        ok("attestation is VALID");
        Ok(())
    } else {
        bail!("attestation is INVALID");
    }
}
