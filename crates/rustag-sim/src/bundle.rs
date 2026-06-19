//! MEV / Jito-style atomic bundle simulation.
//!
//! A Jito bundle is a sequence of transactions that lands **all-or-nothing**: if
//! any transaction fails, none of the bundle's effects persist. That atomicity
//! is exactly what makes bundles useful — and dangerous — for DeFi: a searcher
//! can assume every transaction either all execute in order or none do.
//!
//! This module reproduces the semantics on an isolated fork. [`simulate_bundle`]
//! is read-only (the base stagenet is never touched); [`land_bundle`] commits the
//! bundle to a stagenet *only if* it lands atomically. Tips are accounted by the
//! net lamport inflow to the configured Jito tip accounts across the bundle.

use std::time::Instant;

use solana_pubkey::Pubkey;
use solana_transaction::versioned::VersionedTransaction;

use rustag_core::Stagenet;

use crate::error::Result;
use crate::report::TxResult;

/// The eight canonical Jito tip-payment accounts on mainnet. A bundle pays its
/// priority by transferring lamports to one of these; callers can override the
/// set with [`simulate_bundle_with_tips`].
pub const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTidu8gZyy3L6ezeL8aXXzqZUMHzhvqXxh6e",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

/// Parse the default Jito tip accounts into pubkeys.
pub fn default_tip_accounts() -> Vec<Pubkey> {
    JITO_TIP_ACCOUNTS
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// The result of simulating or landing a bundle.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleReport {
    /// Bundle label.
    pub label: String,
    /// Whether every transaction succeeded — i.e. the bundle would land.
    pub landed: bool,
    /// Number of transactions in the bundle.
    pub total: usize,
    /// Number actually executed (stops at the first failure).
    pub executed: usize,
    /// Index of the first failing transaction, if any.
    pub failed_at: Option<usize>,
    /// Why the first failure occurred.
    pub failure_reason: Option<String>,
    /// Sum of compute units across executed transactions.
    pub total_compute_units: u64,
    /// Sum of fees across executed transactions.
    pub total_fees: u64,
    /// Net lamports paid to the configured Jito tip accounts.
    pub tip_lamports: u64,
    /// Per-transaction outcomes (executed transactions only).
    pub outcomes: Vec<TxResult>,
    /// Wall-clock duration of the simulation.
    pub duration_ms: u128,
}

impl BundleReport {
    /// Effective tip per compute unit (priority signal), or 0 if no CU spent.
    pub fn tip_per_cu(&self) -> f64 {
        if self.total_compute_units == 0 {
            0.0
        } else {
            self.tip_lamports as f64 / self.total_compute_units as f64
        }
    }
}

/// Simulate a bundle against an **isolated fork** of `base` with the default
/// Jito tip accounts. The base stagenet is never mutated.
pub async fn simulate_bundle(
    base: &Stagenet,
    label: impl Into<String>,
    txs: Vec<VersionedTransaction>,
) -> Result<BundleReport> {
    let tips = default_tip_accounts();
    simulate_bundle_with_tips(base, label, txs, &tips).await
}

/// Simulate a bundle against an isolated fork of `base`, accounting tips paid to
/// `tip_accounts`.
pub async fn simulate_bundle_with_tips(
    base: &Stagenet,
    label: impl Into<String>,
    txs: Vec<VersionedTransaction>,
    tip_accounts: &[Pubkey],
) -> Result<BundleReport> {
    let label = label.into();
    let mut fork = base.fork(&format!("bundle-{label}")).await?;
    run_bundle(&mut fork, label, txs, tip_accounts).await
}

/// Land a bundle onto `stagenet` atomically: it is first simulated on a fork,
/// and only if it lands are the transactions committed to `stagenet`. A
/// non-landing bundle leaves `stagenet` untouched (matching Jito semantics).
pub async fn land_bundle(
    stagenet: &mut Stagenet,
    label: impl Into<String>,
    txs: Vec<VersionedTransaction>,
    tip_accounts: &[Pubkey],
) -> Result<BundleReport> {
    let label = label.into();
    // Dry-run on a fork first so a failing bundle never partially mutates state.
    let dry = {
        let mut fork = stagenet.fork(&format!("bundle-dry-{label}")).await?;
        run_bundle(&mut fork, label.clone(), txs.clone(), tip_accounts).await?
    };
    if !dry.landed {
        return Ok(dry);
    }
    // Guaranteed to succeed identically: commit to the real stagenet.
    run_bundle(stagenet, label, txs, tip_accounts).await
}

/// Execute a bundle sequentially against `target`, stopping at the first
/// failure, and measure tips by the net inflow to `tip_accounts`.
async fn run_bundle(
    target: &mut Stagenet,
    label: String,
    txs: Vec<VersionedTransaction>,
    tip_accounts: &[Pubkey],
) -> Result<BundleReport> {
    let start = Instant::now();
    let total = txs.len();

    let mut pre_tips: u128 = 0;
    for acct in tip_accounts {
        pre_tips += target.get_balance(acct).await? as u128;
    }

    let mut outcomes = Vec::with_capacity(total);
    let mut total_cu = 0u64;
    let mut total_fees = 0u64;
    let mut failed_at = None;
    let mut failure_reason = None;

    for (index, tx) in txs.into_iter().enumerate() {
        let outcome = target.send_transaction(tx).await?;
        total_cu = total_cu.saturating_add(outcome.compute_units);
        total_fees = total_fees.saturating_add(outcome.fee);
        let success = outcome.success;
        outcomes.push(TxResult {
            index,
            signature: outcome.signature_string(),
            success: outcome.success,
            err: outcome.err.clone(),
            compute_units: outcome.compute_units,
            fee: outcome.fee,
        });
        if !success {
            failed_at = Some(index);
            failure_reason = outcome.err.clone();
            break;
        }
    }

    let mut post_tips: u128 = 0;
    for acct in tip_accounts {
        post_tips += target.get_balance(acct).await? as u128;
    }
    let tip_lamports = post_tips.saturating_sub(pre_tips).min(u64::MAX as u128) as u64;

    Ok(BundleReport {
        label,
        landed: failed_at.is_none() && total > 0,
        total,
        executed: outcomes.len(),
        failed_at,
        failure_reason,
        total_compute_units: total_cu,
        total_fees,
        tip_lamports,
        outcomes,
        duration_ms: start.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_signer::Signer;
    use solana_system_interface::instruction::transfer;
    use solana_transaction::Transaction;

    #[test]
    fn all_default_tip_accounts_are_valid_pubkeys() {
        assert_eq!(default_tip_accounts().len(), JITO_TIP_ACCOUNTS.len());
    }

    fn tx(
        payer: &Keypair,
        to: &Pubkey,
        lamports: u64,
        bh: solana_hash::Hash,
    ) -> VersionedTransaction {
        let ix = transfer(&payer.pubkey(), to, lamports);
        let msg = Message::new(&[ix], Some(&payer.pubkey()));
        Transaction::new(&[payer], msg, bh).into()
    }

    #[tokio::test]
    async fn landing_bundle_succeeds_and_accounts_tips() {
        let mut base = Stagenet::local("bundle-ok").await.unwrap();
        let payer = Keypair::new();
        base.airdrop(&payer.pubkey(), 10_000_000_000).await.unwrap();
        let tip = default_tip_accounts()[0];
        // On mainnet the Jito tip accounts already exist (and are rent-exempt);
        // model that so a small tip transfer isn't rejected for rent.
        base.airdrop(&tip, 1_000_000_000).await.unwrap();
        let bh = base.latest_blockhash();

        let txs = vec![
            tx(&payer, &Keypair::new().pubkey(), 1_000_000_000, bh),
            tx(&payer, &tip, 50_000, bh), // the tip payment
        ];
        let report = simulate_bundle(&base, "searcher", txs).await.unwrap();
        assert!(report.landed, "{report:?}");
        assert_eq!(report.executed, 2);
        assert_eq!(report.tip_lamports, 50_000);
        assert!(report.failed_at.is_none());
    }

    #[tokio::test]
    async fn failing_bundle_does_not_land_or_mutate_base() {
        let mut base = Stagenet::local("bundle-fail").await.unwrap();
        let payer = Keypair::new();
        base.airdrop(&payer.pubkey(), 1_500_000_000).await.unwrap();
        let bh = base.latest_blockhash();

        // Two 1-SOL transfers but only ~1.5 SOL of balance: the second fails.
        let txs = vec![
            tx(&payer, &Keypair::new().pubkey(), 1_000_000_000, bh),
            tx(&payer, &Keypair::new().pubkey(), 1_000_000_000, bh),
        ];
        let report = simulate_bundle(&base, "greedy", txs).await.unwrap();
        assert!(!report.landed);
        assert_eq!(report.failed_at, Some(1));

        // Base is untouched: payer still holds the full original balance.
        let mut base = base;
        assert_eq!(
            base.get_balance(&payer.pubkey()).await.unwrap(),
            1_500_000_000
        );
    }

    #[tokio::test]
    async fn land_bundle_commits_only_when_it_lands() {
        let mut sn = Stagenet::local("bundle-land").await.unwrap();
        let payer = Keypair::new();
        sn.airdrop(&payer.pubkey(), 1_200_000_000).await.unwrap();
        let bh = sn.latest_blockhash();

        // A bundle that cannot land must leave state untouched.
        let doomed = vec![
            tx(&payer, &Keypair::new().pubkey(), 1_000_000_000, bh),
            tx(&payer, &Keypair::new().pubkey(), 1_000_000_000, bh),
        ];
        let report = land_bundle(&mut sn, "doomed", doomed, &default_tip_accounts())
            .await
            .unwrap();
        assert!(!report.landed);
        assert_eq!(
            sn.get_balance(&payer.pubkey()).await.unwrap(),
            1_200_000_000
        );

        // A bundle that lands must commit.
        let recipient = Keypair::new().pubkey();
        let good = vec![tx(&payer, &recipient, 500_000_000, bh)];
        let report = land_bundle(&mut sn, "good", good, &default_tip_accounts())
            .await
            .unwrap();
        assert!(report.landed);
        assert_eq!(sn.get_balance(&recipient).await.unwrap(), 500_000_000);
    }
}
