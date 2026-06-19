//! `rustag attest` — produce a signed, verifiable attestation of the state a
//! stagenet was tested against (Phase 3, P3.1).
//!
//! Operates offline against the persisted store: it commits to the exact account
//! set RustAG holds for the stagenet, signs the manifest with a local attester
//! key, and writes a `*.attestation.json` artifact that anyone can verify with
//! `rustag verify` — no server, no network.

use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Args;
use solana_keypair::{read_keypair_file, write_keypair_file, Keypair};

use rustag_attest::{Attestation, AttestationManifest};

use super::{
    config_from_record, data_dir, info, load_all_accounts, load_outcomes, ok, open_store,
    redact_url, resolve_record, warn,
};

#[derive(Args)]
pub struct AttestArgs {
    /// Stagenet to attest (defaults to the only one, if unambiguous).
    #[arg(short, long)]
    pub stagenet: Option<String>,
    /// Where to write the attestation JSON (default: `.rustag/<name>.attestation.json`).
    #[arg(short, long)]
    pub out: Option<PathBuf>,
    /// Attester keypair file (Solana JSON format). Created if missing
    /// (default: `.rustag/attest-key.json`).
    #[arg(short, long)]
    pub key: Option<PathBuf>,
    /// Program id(s) the test run exercised. Repeatable. If omitted, the
    /// distinct program ids from recorded transactions are used.
    #[arg(short, long = "program")]
    pub programs: Vec<String>,
    /// Slot to record in the manifest (default: number of recorded transactions).
    #[arg(long)]
    pub slot: Option<u64>,
    /// Cap on recorded transactions folded into the tx-results root.
    #[arg(long, default_value_t = 100_000)]
    pub tx_limit: i64,
}

pub async fn run(args: AttestArgs) -> Result<()> {
    let store = open_store().await?;
    let record = resolve_record(&store, args.stagenet.as_deref()).await?;
    let config = config_from_record(&record)?;

    let accounts = load_all_accounts(&store, &record.id).await?;
    if accounts.is_empty() {
        warn("this stagenet has no accounts yet — the attestation will commit to an empty state");
    }
    let outcomes = load_outcomes(&store, &record.id, args.tx_limit).await?;

    // Programs: explicit flags win; otherwise derive the distinct program ids
    // from the persisted transaction records.
    let programs: Vec<String> = if args.programs.is_empty() {
        let mut set = BTreeSet::new();
        for rec in store.list_transactions(&record.id, args.tx_limit).await? {
            set.extend(rec.programs);
        }
        set.into_iter().collect()
    } else {
        args.programs.clone()
    };

    let source = if config.mirror_enabled {
        redact_url(&config.mainnet_rpc)
    } else {
        "offline".to_string()
    };
    let slot = args.slot.unwrap_or(outcomes.len() as u64);

    let manifest = AttestationManifest::build(
        record.id,
        &record.name,
        source,
        &record.network,
        slot,
        &accounts,
        programs.clone(),
        &outcomes,
    );

    let key_path = args
        .key
        .unwrap_or_else(|| data_dir().join("attest-key.json"));
    let keypair = load_or_create_key(&key_path)?;
    let attestation = Attestation::create(manifest, &keypair);

    let out_path = args
        .out
        .unwrap_or_else(|| data_dir().join(format!("{}.attestation.json", record.name)));
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let json = attestation
        .to_json()
        .map_err(|e| anyhow!("failed to serialize attestation: {e}"))?;
    std::fs::write(&out_path, json)
        .with_context(|| format!("failed to write {}", out_path.display()))?;

    ok(format!("attestation written: {}", out_path.display()));
    info(format!("attester    {}", attestation.attester));
    info(format!("state root  {}", attestation.manifest.state_root));
    info(format!(
        "accounts    {}   transactions {}   programs {}",
        attestation.manifest.account_count,
        attestation.manifest.tx_count,
        programs.len()
    ));
    info("verify with: rustag verify <file> --stagenet <name>");
    Ok(())
}

/// Load the attester keypair, generating and persisting one on first use.
fn load_or_create_key(path: &PathBuf) -> Result<Keypair> {
    if path.exists() {
        read_keypair_file(path)
            .map_err(|e| anyhow!("failed to read attester key {}: {e}", path.display()))
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let keypair = Keypair::new();
        write_keypair_file(&keypair, path)
            .map_err(|e| anyhow!("failed to create attester key {}: {e}", path.display()))?;
        ok(format!("generated attester key: {}", path.display()));
        Ok(keypair)
    }
}
