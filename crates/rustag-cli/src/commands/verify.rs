//! `rustag verify` - verify a staging attestation offline (Phase 3, P3.1).

use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use clap::Args;
use console::style;

use rustag_attest::Attestation;

use super::{fail, info, load_all_accounts, ok, open_store, resolve_record};

#[derive(Args)]
pub struct VerifyArgs {
    /// Path to the attestation JSON to verify.
    pub file: PathBuf,
    /// Verify the state root against a stagenet's current accounts (defaults to
    /// the stagenet named in the attestation, if it exists locally).
    #[arg(short, long)]
    pub stagenet: Option<String>,
    /// Only check the signature; skip recomputing the state root.
    #[arg(long)]
    pub signature_only: bool,
}

pub async fn run(args: VerifyArgs) -> Result<()> {
    let raw = std::fs::read_to_string(&args.file)
        .with_context(|| format!("failed to read {}", args.file.display()))?;
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
