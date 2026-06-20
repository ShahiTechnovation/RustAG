//! Differential execution: run one transaction sequence through two backends
//! and report the first observable divergence.
//!
//! Solana now has real client diversity (Agave, Frankendancer, and Firedancer
//! to come). Subtle behavioral differences between clients are a live consensus
//! concern, so a staging tool should be able to *detect* divergence rather than
//! assume a single execution path. This harness compares two [`Stagenet`]
//! backends step by step on `(success, compute_units, error, state_root)` and
//! stops at the first difference.
//!
//! The reference backend is LiteSVM (both sides here). A second backend pointed
//! at a real Firedancer RPC is a `Stagenet`-shaped extension point - the
//! *divergence-detection logic below is the part that has to be correct*, and it
//! is exercised by the tests today.

use rustag_attest::state_root;
use rustag_core::Stagenet;
use serde::Serialize;
use solana_transaction::versioned::VersionedTransaction;

use crate::error::Result;

/// A single observed difference between the two backends.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Divergence {
    /// Step (transaction index) at which it occurred.
    pub step: usize,
    /// Which observable diverged (`success`, `computeUnits`, `error`, `stateRoot`).
    pub field: String,
    /// The left backend's value, stringified.
    pub left: String,
    /// The right backend's value, stringified.
    pub right: String,
}

/// The outcome of a differential run.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DivergenceReport {
    /// Label for the run.
    pub label: String,
    /// Number of steps executed before stopping.
    pub steps: usize,
    /// The first divergence found, if any.
    pub first_divergence: Option<Divergence>,
}

impl DivergenceReport {
    /// Whether the two backends agreed on every observable.
    pub fn agreed(&self) -> bool {
        self.first_divergence.is_none()
    }
}

/// Run `txs` against both `left` and `right`, comparing each transaction's
/// outcome and the resulting state root, stopping at the first divergence.
pub async fn differential(
    label: impl Into<String>,
    left: &mut Stagenet,
    right: &mut Stagenet,
    txs: Vec<VersionedTransaction>,
) -> Result<DivergenceReport> {
    let label = label.into();
    let mut steps = 0;

    for (step, tx) in txs.into_iter().enumerate() {
        steps = step + 1;
        let lo = left.send_transaction(tx.clone()).await?;
        let ro = right.send_transaction(tx).await?;

        if let Some(div) = compare_field(step, "success", lo.success, ro.success) {
            return Ok(report(label, steps, Some(div)));
        }
        if let Some(div) = compare_field(step, "computeUnits", lo.compute_units, ro.compute_units) {
            return Ok(report(label, steps, Some(div)));
        }
        if let Some(div) = compare_field(
            step,
            "error",
            lo.err.clone().unwrap_or_default(),
            ro.err.clone().unwrap_or_default(),
        ) {
            return Ok(report(label, steps, Some(div)));
        }

        // Compare the full state root after each step - the strongest check.
        let lr = hex::encode(state_root(&left.export_accounts().await?));
        let rr = hex::encode(state_root(&right.export_accounts().await?));
        if let Some(div) = compare_field(step, "stateRoot", lr, rr) {
            return Ok(report(label, steps, Some(div)));
        }
    }

    Ok(report(label, steps, None))
}

fn compare_field<T: PartialEq + std::fmt::Display>(
    step: usize,
    field: &str,
    left: T,
    right: T,
) -> Option<Divergence> {
    (left != right).then(|| Divergence {
        step,
        field: field.to_string(),
        left: left.to_string(),
        right: right.to_string(),
    })
}

fn report(label: String, steps: usize, first_divergence: Option<Divergence>) -> DivergenceReport {
    DivergenceReport {
        label,
        steps,
        first_divergence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_signer::Signer;
    use solana_system_interface::instruction::transfer;
    use solana_transaction::Transaction;

    async fn funded(name: &str, lamports: u64) -> (Stagenet, Keypair) {
        let mut sn = Stagenet::local(name).await.unwrap();
        let payer = Keypair::new();
        sn.airdrop(&payer.pubkey(), lamports).await.unwrap();
        (sn, payer)
    }

    #[tokio::test]
    async fn identical_backends_do_not_diverge() {
        // Two backends seeded with the same payer/state run the same workload.
        let payer = Keypair::new();
        let mut left = Stagenet::local("diff-l").await.unwrap();
        let mut right = Stagenet::local("diff-r").await.unwrap();
        left.airdrop(&payer.pubkey(), 10_000_000_000).await.unwrap();
        right
            .airdrop(&payer.pubkey(), 10_000_000_000)
            .await
            .unwrap();

        let bh = left.latest_blockhash();
        let txs: Vec<VersionedTransaction> = (0..4)
            .map(|i| {
                // Deterministic recipients so both sides see identical txs.
                let to = solana_pubkey::Pubkey::new_from_array([i as u8 + 9; 32]);
                let ix = transfer(&payer.pubkey(), &to, 100_000_000);
                let msg = Message::new(&[ix], Some(&payer.pubkey()));
                Transaction::new(&[&payer], msg, bh).into()
            })
            .collect();

        let report = differential("agree", &mut left, &mut right, txs)
            .await
            .unwrap();
        assert!(report.agreed(), "{report:?}");
        assert_eq!(report.steps, 4);
    }

    #[tokio::test]
    async fn different_starting_state_diverges() {
        // Left can afford the transfer; right cannot → success diverges.
        let (mut left, payer) = funded("diff-rich", 5_000_000_000).await;
        let mut right = Stagenet::local("diff-poor").await.unwrap();
        // Right uses the SAME payer key but with too little balance.
        right.airdrop(&payer.pubkey(), 100_000).await.unwrap();

        let bh = left.latest_blockhash();
        let to = solana_pubkey::Pubkey::new_from_array([42; 32]);
        let ix = transfer(&payer.pubkey(), &to, 1_000_000_000);
        let msg = Message::new(&[ix], Some(&payer.pubkey()));
        let tx: VersionedTransaction = Transaction::new(&[&payer], msg, bh).into();

        let report = differential("disagree", &mut left, &mut right, vec![tx])
            .await
            .unwrap();
        assert!(!report.agreed());
        let div = report.first_divergence.unwrap();
        assert_eq!(div.step, 0);
        assert_eq!(div.field, "success");
    }
}
