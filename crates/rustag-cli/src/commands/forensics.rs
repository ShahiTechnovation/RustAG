//! `rustag forensics` - deterministic re-execution of a historical transaction.
//!
//! The forensics command is the post-incident counterpart to `rustag rehearse`.
//! Where `rehearse` looks *forward* (what will this proposal do?), `forensics`
//! looks *backward* (what did this historical transaction actually do, and could
//! a patched program have stopped it?).
//!
//! ## Workflow
//!
//! 1. Fetch the historical transaction by signature.
//! 2. Fetch the pre-state accounts the transaction touched (best-effort from
//!    mainnet RPC; not perfect due to state drift, but sufficient for analysis).
//! 3. Reconstruct the Clock sysvar at the transaction's slot/blockTime.
//! 4. Re-execute the transaction in a sealed offline sandbox.
//! 5. Emit a signed EvidenceBundle with the forensic diff.
//!
//! ## Counterfactual Mode (--patch)
//!
//! With `--patch <ELF_PATH>`, replace the deployed program bytecode with a
//! candidate fix before re-executing. The output shows whether the patched
//! program would have *prevented* the incident (BLOCKED) or not (REPRODUCED).

use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use clap::Args;
use console::style;

use rustag_core::{Stagenet};
use rustag_rehearse::{to_portable, RehearsalOptions, SealedRehearsal};
use rustag_sim::Policy;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

use super::{fail, info, ok, warn};

#[derive(Args)]
pub struct ForensicsArgs {
    /// The transaction signature to re-execute (base58).
    pub signature: String,

    /// Mainnet RPC to fetch the transaction and its account state from.
    #[arg(long, env = "RUSTAG_MAINNET_RPC")]
    pub rpc: Option<String>,

    /// Path to a candidate ELF binary to substitute for the deployed program
    /// (counterfactual analysis: would this fix have prevented the incident?).
    #[arg(long)]
    pub patch: Option<PathBuf>,

    /// The program ID to patch (required when --patch is given).
    #[arg(long)]
    pub patch_program: Option<String>,

    /// Path to a Solana JSON keypair to sign the evidence bundle with.
    #[arg(long)]
    pub signer: Option<PathBuf>,

    /// Where to write the signed EvidenceBundle JSON.
    #[arg(long, default_value = "forensics-bundle.json")]
    pub out: PathBuf,

    /// Where to write the portable pre-state closure.
    #[arg(long, default_value = "forensics-closure.json")]
    pub closure: PathBuf,

    /// Print raw JSON output (machine-readable).
    #[arg(long)]
    pub json: bool,
}

pub async fn run(args: ForensicsArgs) -> Result<()> {
    let rpc = args
        .rpc
        .as_ref()
        .ok_or_else(|| anyhow!("--rpc <url> is required (or set RUSTAG_MAINNET_RPC)"))?;

    let signer = load_or_generate_keypair(args.signer.as_ref())?;

    if !args.json {
        println!();
        info(format!(
            "forensics: re-executing {} from mainnet",
            style(&args.signature).cyan()
        ));
    }

    // Step 1: Fetch the historical transaction.
    let (raw_tx, slot, block_time) =
        fetch_transaction(rpc, &args.signature).await?;

    if !args.json {
        info(format!(
            "transaction found: slot {}, block_time {}",
            slot,
            block_time
                .map(|t| format!("{t}"))
                .unwrap_or_else(|| "unknown".to_string())
        ));
    }

    let payload: VersionedTransaction =
        bincode::deserialize(&raw_tx).context("failed to deserialize transaction")?;

    // Step 2 & 3: Build a stagenet, import accounts, pin the clock.
    let mut working = Stagenet::local_with_mainnet("groundtruth-forensics", rpc).await?;

    // Pin the clock at the transaction's slot and blockTime for time-dependent replay.
    if let Some(ts) = block_time {
        working.sync_clock(slot, ts);
        if !args.json {
            info(format!("clock pinned at slot {slot}, unix_ts {ts}"));
        }
    }

    // Step 4: Apply counterfactual program patch if requested.
    let counterfactual = if let Some(elf_path) = &args.patch {
        let program_id_str = args
            .patch_program
            .as_ref()
            .ok_or_else(|| anyhow!("--patch-program <PROGRAM_ID> is required with --patch"))?;

        let elf_bytes = std::fs::read(elf_path)
            .with_context(|| format!("failed to read ELF patch: {}", elf_path.display()))?;

        if !args.json {
            warn(format!(
                "COUNTERFACTUAL MODE: substituting {} ({} bytes) for program {}",
                elf_path.display(),
                elf_bytes.len(),
                program_id_str
            ));
        }

        Some((program_id_str.clone(), elf_bytes))
    } else {
        None
    };

    // Apply the program patch if in counterfactual mode.
    if let Some((program_id_str, elf_bytes)) = &counterfactual {
        use solana_pubkey::Pubkey;
        use std::str::FromStr;
        let program_id = Pubkey::from_str(program_id_str)
            .map_err(|_| anyhow!("invalid program ID: {program_id_str}"))?;
        // We use load_upgradeable_program + override_account to substitute the ELF.
        // First load the current program from mainnet so the account exists.
        let _ = working.load_upgradeable_program(&program_id).await;
        // Then override the ProgramData account's data with our candidate ELF.
        // The ProgramData address is derived from the program ID.
        use rustag_core::AccountOverride;
        working
            .override_account(
                &program_id,
                AccountOverride {
                    data: Some(elf_bytes.clone()),
                    executable: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| anyhow!("failed to patch program: {e}"))?;
    }

    // Step 5: Run the sealed rehearsal.
    let opts = RehearsalOptions {
        network: "mainnet-beta".to_string(),
        input_source: rustag_core::redact_url(rpc),
        ..Default::default()
    };

    let rehearsal = SealedRehearsal::run(
        working,
        payload.clone(),
        opts,
        &Policy::standard(),
        &signer,
    )
    .await
    .map_err(|e| anyhow!("forensic re-execution failed: {e}"))?;

    // Render results.
    if args.json {
        let output = serde_json::json!({
            "signature": args.signature,
            "slot": slot,
            "blockTime": block_time,
            "counterfactual": counterfactual.is_some(),
            "bundle": rehearsal.bundle,
            "grade": if rehearsal.grade_a { "A" } else { "B" },
            "alarms": rehearsal.alarms,
            "semanticDiff": rehearsal.semantic_diff,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        render_forensics(&rehearsal, &args, &counterfactual);
    }

    // Persist bundle and closure.
    let bundle_json = rehearsal
        .bundle
        .to_json()
        .map_err(|e| anyhow!("failed to serialize bundle: {e}"))?;
    std::fs::write(&args.out, bundle_json)
        .with_context(|| format!("failed to write {}", args.out.display()))?;
    let closure_json = serde_json::to_string_pretty(&to_portable(&rehearsal.closure))
        .context("failed to serialize closure")?;
    std::fs::write(&args.closure, closure_json)
        .with_context(|| format!("failed to write {}", args.closure.display()))?;

    if !args.json {
        println!();
        ok(format!("forensic bundle written to {}", args.out.display()));
        ok(format!("closure written to {}", args.closure.display()));
        info(format!(
            "verify offline with: rustag verify {} --closure {}",
            args.out.display(),
            args.closure.display()
        ));
    }

    // In counterfactual mode, report the verdict.
    if counterfactual.is_some() && !args.json {
        println!();
        if !rehearsal.bundle.manifest.success {
            ok(style("COUNTERFACTUAL VERDICT: BLOCKED ✓ — the patched program would have PREVENTED this transaction from succeeding").green().bold().to_string());
        } else {
            fail(style("COUNTERFACTUAL VERDICT: REPRODUCED ✗ — the patched program did NOT prevent this transaction").red().bold().to_string());
        }
    }

    Ok(())
}

/// Render a forensic rehearsal result to stdout.
fn render_forensics(
    rehearsal: &rustag_rehearse::Rehearsal,
    args: &ForensicsArgs,
    counterfactual: &Option<(String, Vec<u8>)>,
) {
    let m = &rehearsal.bundle.manifest;
    println!();

    if counterfactual.is_some() {
        info(style("=== COUNTERFACTUAL FORENSICS MODE ===").yellow().bold().to_string());
    } else {
        info(style("=== FORENSIC RE-EXECUTION ===").cyan().bold().to_string());
    }

    println!();
    info(format!("signature   {}", style(&args.signature).dim()));
    info(format!(
        "fidelity    {}   engine {}",
        match m.fidelity_grade {
            rustag_attest::FidelityGrade::A => style("Grade A (re-executable)").green(),
            rustag_attest::FidelityGrade::B => style("Grade B (observed)").yellow(),
        },
        m.engine
    ));
    info(format!("pre  root   {}", style(short_root(&m.pre_state_root)).dim()));
    info(format!("post root   {}", style(short_root(&m.post_state_root)).dim()));
    info(format!(
        "compute     {} CU   success {}",
        m.compute_units,
        if m.success {
            style("true").green().to_string()
        } else {
            style("FALSE").red().bold().to_string()
        }
    ));

    println!();
    if rehearsal.semantic_diff.is_empty() {
        info("no state changes");
    } else {
        info(format!("state changes ({}):", rehearsal.semantic_diff.len()));
        for change in &rehearsal.semantic_diff {
            println!(
                "      {}",
                serde_json::to_string(change).unwrap_or_else(|_| format!("{change:?}"))
            );
        }
    }

    println!();
    if rehearsal.alarms.is_empty() {
        ok("no invariant alarms (transaction appears benign in isolation)");
    } else {
        warn(format!("{} invariant alarm(s):", rehearsal.alarms.len()));
        for alarm in &rehearsal.alarms {
            let sev = style(format!("{:?}", alarm.severity)).red().bold();
            println!("      [{sev}] {} - {}", alarm.rule, alarm.detail);
        }
    }
}

fn short_root(root: &str) -> String {
    if root.len() > 16 {
        format!("{}…{}", &root[..8], &root[root.len() - 8..])
    } else {
        root.to_string()
    }
}

/// Fetch a transaction by signature from mainnet RPC.
/// Returns (raw_bytes, slot, block_time).
async fn fetch_transaction(rpc: &str, signature: &str) -> Result<(Vec<u8>, u64, Option<i64>)> {
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(concat!("rustag-cli/", env!("CARGO_PKG_VERSION")))
        .build()?;

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

    let body = http
        .post(rpc)
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    #[derive(serde::Deserialize)]
    struct Envelope {
        result: Option<TxResult>,
        error: Option<RpcError>,
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TxResult {
        slot: u64,
        block_time: Option<i64>,
        transaction: serde_json::Value,
    }
    #[derive(serde::Deserialize)]
    struct RpcError {
        code: i64,
        message: String,
    }

    let envelope: Envelope =
        serde_json::from_str(&body).context("failed to parse getTransaction response")?;

    if let Some(err) = envelope.error {
        bail!("RPC error {}: {}", err.code, err.message);
    }

    let result = envelope
        .result
        .ok_or_else(|| anyhow!("transaction {signature} not found (check the signature and try a confirmed-commitment RPC)"))?;

    // The transaction is returned as [base64_string, "base64"] array.
    let raw_b64 = match &result.transaction {
        serde_json::Value::Array(arr) if !arr.is_empty() => arr[0]
            .as_str()
            .ok_or_else(|| anyhow!("unexpected transaction encoding"))?,
        serde_json::Value::String(s) => s.as_str(),
        _ => bail!("unexpected transaction format"),
    };

    let raw = base64::engine::general_purpose::STANDARD
        .decode(raw_b64)
        .context("failed to decode base64 transaction")?;

    Ok((raw, result.slot, result.block_time))
}

/// Load a keypair from a file, or generate an ephemeral one.
fn load_or_generate_keypair(path: Option<&PathBuf>) -> Result<Keypair> {
    match path {
        Some(path) => {
            let raw = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read keypair {}", path.display()))?;
            let bytes: Vec<u8> =
                serde_json::from_str(&raw).context("keypair file is not a JSON byte array")?;
            Keypair::try_from(bytes.as_slice())
                .map_err(|e| anyhow!("invalid keypair bytes: {e}"))
        }
        None => {
            let kp = Keypair::new();
            warn(format!(
                "no --signer given; signing with an ephemeral key {}",
                kp.pubkey()
            ));
            Ok(kp)
        }
    }
}
