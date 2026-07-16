//! `rustag record` - record a program's real mainnet transactions.
//!
//! This command uses the ForwardRecorder to fetch recent mainnet transactions
//! for a watched program and saves them as a self-contained corpus JSON file.
//! The corpus is used by the upgrade-rehearsal CI gate to replay real traffic
//! against candidate program upgrades.
//!
//! ## Workflow
//!
//! ```bash
//! # Record 500 recent transactions for Jupiter v6
//! rustag record --program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 \
//!               --rpc https://mainnet.helius-rpc.com/?api-key=XXX \
//!               --limit 500 \
//!               --out jupiter-corpus.json
//!
//! # Re-run to update the corpus with new transactions
//! rustag record --program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 \
//!               --rpc ... --out jupiter-corpus.json --append
//! ```

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Args;
use console::style;

use rustag_mirror::{ForwardRecorder, MainnetMirror, RecordedCorpus};

use super::{info, ok, warn};

#[derive(Args)]
pub struct RecordArgs {
    /// The program to watch (base58 pubkey).
    #[arg(long)]
    pub program: String,

    /// Mainnet RPC to fetch transactions from.
    #[arg(long, env = "RUSTAG_MAINNET_RPC")]
    pub rpc: Option<String>,

    /// Maximum number of transactions to record.
    #[arg(long, default_value = "200")]
    pub limit: usize,

    /// Where to write the corpus JSON.
    #[arg(long, default_value = "rustag-corpus.json")]
    pub out: PathBuf,

    /// Append to an existing corpus file instead of overwriting.
    #[arg(long)]
    pub append: bool,

    /// Print statistics only (do not save the corpus).
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(args: RecordArgs) -> Result<()> {
    let rpc = args
        .rpc
        .as_ref()
        .ok_or_else(|| anyhow!("--rpc <url> is required (or set RUSTAG_MAINNET_RPC)"))?;

    println!();
    info(format!(
        "recording up to {} transactions for {}",
        args.limit,
        style(&args.program).cyan()
    ));

    let mirror = MainnetMirror::new(rpc, 10)
        .map_err(|e| anyhow!("failed to create mainnet mirror: {e}"))?;

    let recorder = ForwardRecorder::new(&mirror).with_max_transactions(args.limit);

    let corpus = recorder
        .fetch_recent(&args.program, args.limit)
        .await
        .map_err(|e| anyhow!("failed to record transactions: {e}"))?;

    // Merge with existing corpus if --append.
    let final_corpus = if args.append && args.out.exists() {
        let existing_raw = std::fs::read_to_string(&args.out)
            .with_context(|| format!("failed to read {}", args.out.display()))?;
        let mut existing: RecordedCorpus = serde_json::from_str(&existing_raw)
            .context("existing corpus file is corrupt")?;

        let new_sigs: std::collections::HashSet<&str> =
            corpus.transactions.iter().map(|t| t.signature.as_str()).collect();
        let existing_sigs: std::collections::HashSet<&str> =
            existing.transactions.iter().map(|t| t.signature.as_str()).collect();
        let truly_new: Vec<_> = corpus
            .transactions
            .iter()
            .filter(|t| !existing_sigs.contains(t.signature.as_str()))
            .cloned()
            .collect();

        info(format!(
            "appending {} new transactions ({} already in corpus)",
            truly_new.len(),
            new_sigs.intersection(&existing_sigs).count()
        ));

        existing.transactions.extend(truly_new);
        existing.transactions.sort_by_key(|t| t.slot);
        if !existing.transactions.is_empty() {
            existing.from_slot = existing.transactions[0].slot;
            existing.to_slot = existing.transactions.last().unwrap().slot;
        }
        existing
    } else {
        corpus
    };

    println!();
    print_stats(&final_corpus);

    if args.dry_run {
        warn("dry-run mode: corpus not saved");
        return Ok(());
    }

    let json = serde_json::to_string_pretty(&final_corpus)
        .context("failed to serialize corpus")?;
    std::fs::write(&args.out, json)
        .with_context(|| format!("failed to write {}", args.out.display()))?;

    println!();
    ok(format!(
        "{} transactions saved to {}",
        final_corpus.len(),
        args.out.display()
    ));
    info(format!(
        "slot range: {} → {}",
        final_corpus.from_slot, final_corpus.to_slot
    ));

    Ok(())
}

fn print_stats(corpus: &RecordedCorpus) {
    let success_count = corpus.successful().count();
    let fail_count = corpus.len() - success_count;
    let success_rate = if corpus.is_empty() {
        0.0
    } else {
        success_count as f64 / corpus.len() as f64 * 100.0
    };

    info(format!("program     {}", style(&corpus.program_id).cyan()));
    info(format!("total       {} transactions", corpus.len()));
    info(format!(
        "success     {} ({:.1}%)   failed {}",
        success_count, success_rate, fail_count
    ));
    if !corpus.is_empty() {
        info(format!(
            "slots       {} → {}",
            corpus.from_slot, corpus.to_slot
        ));
    }
}
