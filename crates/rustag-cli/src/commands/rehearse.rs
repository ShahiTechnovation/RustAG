//! `rustag rehearse` - sealed pre-execution rehearsal of a privileged payload.
//!
//! Rehearse a proposed Solana transaction against faithful mainnet state in a
//! sealed, deterministic sandbox and emit a signed, offline-verifiable
//! `EvidenceBundle`: exactly what it changes, which invariants it trips, at what
//! compute. This is the GroundTruth wedge (see `docs/PIVOT_PLAN.md`).

use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use clap::Args;
use console::style;

use rustag_core::Stagenet;
use rustag_rehearse::{to_portable, RehearsalOptions, SealedRehearsal};
use rustag_sim::{Policy, Severity};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

use super::{info, ok, warn};

#[derive(Args)]
pub struct RehearseArgs {
    /// Squads v4 VaultTransaction proposal pubkey to fetch and rehearse from mainnet.
    /// This is the primary GroundTruth workflow: paste the proposal address, get
    /// a signed evidence bundle back.
    #[arg(long, conflicts_with = "payload", conflicts_with = "demo")]
    pub proposal: Option<String>,

    /// Base64 of a bincode-serialized `VersionedTransaction` to rehearse directly.
    #[arg(long, alias = "message", conflicts_with = "demo")]
    pub payload: Option<String>,

    /// Mainnet RPC to lazily fetch the touched account closure from.
    /// Required when using --proposal (to fetch the Squads account and closure).
    #[arg(long, env = "RUSTAG_MAINNET_RPC")]
    pub rpc: Option<String>,

    /// Rehearse fully offline (no mirror); the payload must be self-contained.
    #[arg(long)]
    pub offline: bool,

    /// Run a built-in, self-contained demo (an ownership-takeover payload) that
    /// needs no network - the fastest way to see a signed bundle end to end.
    #[arg(long)]
    pub demo: bool,

    /// Path to a Solana JSON keypair (64-byte array) to sign the bundle with.
    /// If omitted, an ephemeral key is generated and its pubkey printed.
    #[arg(long, alias = "keypair")]
    pub signer: Option<PathBuf>,

    /// Where to write the signed EvidenceBundle JSON.
    #[arg(long, default_value = "groundtruth-bundle.json")]
    pub out: PathBuf,

    /// Where to write the portable pre-state closure (for offline verification).
    #[arg(long, default_value = "groundtruth-closure.json")]
    pub closure: PathBuf,

    /// Exit non-zero if any alarm reaches this severity (a CI gate).
    #[arg(long, value_parser = parse_severity)]
    pub fail_on: Option<Severity>,
}

pub async fn run(args: RehearseArgs) -> Result<()> {
    let signer = load_or_generate_keypair(args.signer.as_ref())?;

    // Build the working stagenet and the payload to rehearse.
    let (working, payload, opts) = if args.demo {
        build_demo().await?
    } else if let Some(ref proposal_str) = args.proposal {
        build_from_proposal(proposal_str, &args).await?
    } else {
        build_from_args(&args).await?
    };

    println!();
    info(format!("rehearsing with attester {}", style(signer.pubkey()).cyan()));

    let rehearsal = SealedRehearsal::run(working, payload, opts, &Policy::standard(), &signer)
        .await
        .map_err(|e| anyhow!("rehearsal failed: {e}"))?;

    render(&rehearsal);

    // Persist the bundle and its closure.
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

    println!();
    ok(format!("bundle written to {}", args.out.display()));
    ok(format!("closure written to {}", args.closure.display()));
    info(format!(
        "verify offline with: rustag verify {} --closure {}",
        args.out.display(),
        args.closure.display()
    ));

    // CI gate.
    if let Some(threshold) = args.fail_on {
        if let Some(max) = rehearsal.max_alarm_severity() {
            if max >= threshold {
                println!();
                bail!("rehearsal raised a {max:?} alarm (>= --fail-on {threshold:?})");
            }
        }
    }
    Ok(())
}

/// Print the semantic diff, alarms, grade, and compute of a rehearsal.
fn render(rehearsal: &rustag_rehearse::Rehearsal) {
    let m = &rehearsal.bundle.manifest;
    println!();
    info(format!(
        "fidelity    {}   engine {}",
        match m.fidelity_grade {
            rustag_attest::FidelityGrade::A => style("Grade A (re-executable)").green(),
            rustag_attest::FidelityGrade::B => style("Grade B (observed)").yellow(),
        },
        m.engine
    ));
    info(format!(
        "pre  root   {}",
        style(short_root(&m.pre_state_root)).dim()
    ));
    info(format!(
        "post root   {}",
        style(short_root(&m.post_state_root)).dim()
    ));
    info(format!(
        "compute     {} CU   success {}",
        m.compute_units, m.success
    ));

    println!();
    if rehearsal.semantic_diff.is_empty() {
        info("no state changes");
    } else {
        info(format!("changes ({}):", rehearsal.semantic_diff.len()));
        for change in &rehearsal.semantic_diff {
            println!("      {}", describe_change(change));
        }
    }

    println!();
    if rehearsal.alarms.is_empty() {
        ok("no invariant alarms");
    } else {
        warn(format!("{} invariant alarm(s):", rehearsal.alarms.len()));
        for alarm in &rehearsal.alarms {
            let sev = style(format!("{:?}", alarm.severity)).red().bold();
            println!("      [{sev}] {} - {}", alarm.rule, alarm.detail);
        }
    }
}

fn describe_change(change: &rustag_sim::SemanticChange) -> String {
    serde_json::to_string(change).unwrap_or_else(|_| format!("{change:?}"))
}

fn short_root(root: &str) -> String {
    if root.len() > 16 {
        format!("{}…{}", &root[..8], &root[root.len() - 8..])
    } else {
        root.to_string()
    }
}

/// Build the working stagenet + payload for a real rehearsal from CLI args.
async fn build_from_args(args: &RehearseArgs) -> Result<(Stagenet, VersionedTransaction, RehearsalOptions)> {
    let raw = args
        .payload
        .as_ref()
        .ok_or_else(|| anyhow!("provide --proposal <SQUADS_PUBKEY>, --payload <base64>, or use --demo"))?;
    let payload = decode_payload(raw)?;

    let (working, opts) = if args.offline {
        (
            Stagenet::local("groundtruth-rehearse").await?,
            RehearsalOptions::offline(),
        )
    } else {
        let rpc = args
            .rpc
            .as_ref()
            .ok_or_else(|| anyhow!("provide --rpc <url> (or RUSTAG_MAINNET_RPC), or --offline"))?;
        let working = Stagenet::local_with_mainnet("groundtruth-rehearse", rpc).await?;
        let opts = RehearsalOptions {
            network: "mainnet-beta".to_string(),
            input_source: rustag_core::redact_url(rpc),
            ..Default::default()
        };
        (working, opts)
    };
    Ok((working, payload, opts))
}

/// Build a payload from a Squads v4 proposal pubkey fetched from mainnet.
async fn build_from_proposal(
    proposal_str: &str,
    args: &RehearseArgs,
) -> Result<(Stagenet, VersionedTransaction, RehearsalOptions)> {
    use rustag_mirror::{MainnetMirror, SquadsDecoder};
    use solana_pubkey::Pubkey;
    use std::str::FromStr;

    let rpc = args
        .rpc
        .as_ref()
        .ok_or_else(|| anyhow!("--rpc <url> is required when using --proposal"))?;

    let proposal_pubkey = Pubkey::from_str(proposal_str)
        .map_err(|_| anyhow!("invalid proposal pubkey: {proposal_str}"))?;

    info(format!("fetching Squads proposal {} from mainnet", style(proposal_str).cyan()));

    let mirror = MainnetMirror::new(rpc, 10)
        .map_err(|e| anyhow!("failed to create mainnet mirror: {e}"))?;
    let decoder = SquadsDecoder::new(&mirror);
    let proposed = decoder
        .decode_proposal(&proposal_pubkey)
        .await
        .map_err(|e| anyhow!("failed to decode Squads proposal: {e}"))?;

    info(format!(
        "proposal: multisig {}, vault {}, {}/{} approvals",
        style(&proposed.multisig.to_string()[..8]).dim(),
        proposed.vault_index,
        proposed.approval_count,
        proposed.threshold,
    ));

    // Decode the raw message bytes from the Squads proposal.
    // Squads v4 VaultTransaction embeds a `TransactionMessage` in Borsh format.
    // We deserialize it as a VersionedTransaction with a synthetic signature.
    let payload = decode_squads_message(&proposed.message_bytes)?;

    let working = Stagenet::local_with_mainnet("groundtruth-rehearse", rpc).await?;
    let opts = RehearsalOptions {
        network: "mainnet-beta".to_string(),
        input_source: rustag_core::redact_url(rpc),
        proposal_account: Some(proposal_str.to_string()),
        ..Default::default()
    };
    Ok((working, payload, opts))
}

/// Decode Squads VaultTransaction message bytes into a VersionedTransaction.
/// The message bytes from Squads are in Borsh/bincode format.
fn decode_squads_message(message_bytes: &[u8]) -> Result<VersionedTransaction> {
    // Try direct bincode deserialization first.
    if let Ok(tx) = bincode::deserialize::<VersionedTransaction>(message_bytes) {
        return Ok(tx);
    }
    // Try base64 → bincode (in case the bytes are base64-encoded).
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(message_bytes) {
        if let Ok(tx) = bincode::deserialize::<VersionedTransaction>(&decoded) {
            return Ok(tx);
        }
    }
    Err(anyhow!(
        "could not decode the Squads proposal message — try manually passing --payload <base64>"
    ))
}

/// A self-contained demo: fund a victim account, then rehearse a payload that
/// reassigns its owning program to an attacker-controlled program - a routine-
/// looking transaction that quietly takes over an account.
async fn build_demo() -> Result<(Stagenet, VersionedTransaction, RehearsalOptions)> {
    use solana_message::Message;
    use solana_transaction::Transaction;

    let mut sn = Stagenet::local("groundtruth-demo").await?;
    let payer = Keypair::new();
    let victim = Keypair::new();
    sn.airdrop(&payer.pubkey(), 10_000_000_000).await?;
    sn.airdrop(&victim.pubkey(), 1_000_000_000).await?;

    // The attacker-controlled program the victim account is being reassigned to.
    let attacker_program = Pubkey::new_from_array([7; 32]);
    let ix = solana_system_interface::instruction::assign(&victim.pubkey(), &attacker_program);
    let msg = Message::new(&[ix], Some(&payer.pubkey()));
    let bh = sn.latest_blockhash();
    let payload: VersionedTransaction = Transaction::new(&[&payer, &victim], msg, bh).into();

    info("demo: rehearsing an account-ownership takeover (System::Assign)");
    Ok((sn, payload, RehearsalOptions::offline()))
}

fn decode_payload(base64_tx: &str) -> Result<VersionedTransaction> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_tx.trim())
        .context("--message is not valid base64")?;
    bincode::deserialize(&bytes).context("--message is not a bincode VersionedTransaction")
}

/// Load a Solana JSON keypair from `path`, or generate an ephemeral one.
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
                "no --keypair given; signing with an ephemeral key {}",
                kp.pubkey()
            ));
            Ok(kp)
        }
    }
}

fn parse_severity(s: &str) -> std::result::Result<Severity, String> {
    match s.to_ascii_lowercase().as_str() {
        "info" => Ok(Severity::Info),
        "low" => Ok(Severity::Low),
        "medium" => Ok(Severity::Medium),
        "high" => Ok(Severity::High),
        "critical" => Ok(Severity::Critical),
        other => Err(format!("unknown severity '{other}'")),
    }
}
