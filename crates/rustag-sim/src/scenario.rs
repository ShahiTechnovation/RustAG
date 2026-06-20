//! Running scenarios against forked stagenets.
//!
//! Every function here runs transactions against an **isolated fork** so the
//! base stagenet is never mutated. A scenario is just a list of transactions;
//! [`stress`] is sugar for "build one transaction per actor", and [`compare`]
//! forks the base once per variant and reports them side by side.

use std::time::Instant;

use solana_transaction::versioned::VersionedTransaction;

use rustag_core::Stagenet;

use crate::error::Result;
use crate::report::{ComparisonReport, ScenarioReport};

/// Run a sequence of transactions against `fork`, recording every outcome.
///
/// The caller owns the fork (create one with [`Stagenet::fork`]), so this never
/// touches mainnet or the original stagenet. Use [`fork_and_replay`] to fork +
/// run in one step.
pub async fn replay(
    fork: &mut Stagenet,
    label: impl Into<String>,
    txs: Vec<VersionedTransaction>,
) -> Result<ScenarioReport> {
    let start = Instant::now();
    let mut report = ScenarioReport::new(label, txs.len());
    for (index, tx) in txs.into_iter().enumerate() {
        let outcome = fork.send_transaction(tx).await?;
        report.record(index, &outcome);
    }
    report.duration_ms = start.elapsed().as_millis();
    Ok(report)
}

/// Build and run one transaction per actor against `fork`.
///
/// Models "what if `actors` users all act at once?" - e.g. a thundering herd of
/// liquidations or swaps. `build(i)` produces the i-th actor's transaction.
pub async fn stress<F>(
    fork: &mut Stagenet,
    label: impl Into<String>,
    actors: usize,
    mut build: F,
) -> Result<ScenarioReport>
where
    F: FnMut(usize) -> VersionedTransaction,
{
    let txs: Vec<VersionedTransaction> = (0..actors).map(&mut build).collect();
    replay(fork, label, txs).await
}

/// Fork `base` into a fresh in-memory copy and replay `txs` against it.
pub async fn fork_and_replay(
    base: &Stagenet,
    label: impl Into<String>,
    txs: Vec<VersionedTransaction>,
) -> Result<ScenarioReport> {
    let label = label.into();
    let mut fork = base.fork(&format!("sim-{label}")).await?;
    replay(&mut fork, label, txs).await
}

/// Fork `base` once per variant and run each variant's transactions against its
/// own fork, returning every report for side-by-side comparison.
///
/// Each variant is fully isolated, so this answers "given the same starting
/// state, which of these N transaction sets behaves best?".
pub async fn compare(
    base: &Stagenet,
    variants: Vec<(String, Vec<VersionedTransaction>)>,
) -> Result<ComparisonReport> {
    let mut reports = Vec::with_capacity(variants.len());
    for (label, txs) in variants {
        reports.push(fork_and_replay(base, label, txs).await?);
    }
    Ok(ComparisonReport { variants: reports })
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_signer::Signer;
    use solana_system_interface::instruction::transfer;
    use solana_transaction::Transaction;

    #[tokio::test]
    async fn fork_isolates_and_replays() {
        // Base stagenet with a funded payer.
        let mut base = Stagenet::local("sim-base").await.unwrap();
        let payer = Keypair::new();
        base.airdrop(&payer.pubkey(), 10_000_000_000).await.unwrap();

        // Fork and run 5 transfers of 1 SOL each.
        let blockhash = base.latest_blockhash();
        let mut fork = base.fork("sim-fork").await.unwrap();
        let txs: Vec<VersionedTransaction> = (0..5)
            .map(|_| {
                let to = Keypair::new().pubkey();
                let ix = transfer(&payer.pubkey(), &to, 1_000_000_000);
                let msg = Message::new(&[ix], Some(&payer.pubkey()));
                Transaction::new(&[&payer], msg, blockhash).into()
            })
            .collect();
        let report = replay(&mut fork, "transfers", txs).await.unwrap();
        assert_eq!(report.total, 5);
        assert_eq!(report.succeeded, 5);
        assert_eq!(report.success_rate(), 1.0);

        // The base is untouched: payer still has the full 10 SOL.
        let mut base = base;
        assert_eq!(
            base.get_balance(&payer.pubkey()).await.unwrap(),
            10_000_000_000
        );
    }

    #[tokio::test]
    async fn stress_surfaces_failures() {
        // Payer funded with only 3 SOL; 5 actors each try to send 1 SOL.
        let mut base = Stagenet::local("sim-stress").await.unwrap();
        let payer = Keypair::new();
        base.airdrop(&payer.pubkey(), 3_000_000_000).await.unwrap();
        let blockhash = base.latest_blockhash();

        let mut fork = base.fork("stress-fork").await.unwrap();
        let report = stress(&mut fork, "herd", 5, |_| {
            let to = Keypair::new().pubkey();
            let ix = transfer(&payer.pubkey(), &to, 1_000_000_000);
            let msg = Message::new(&[ix], Some(&payer.pubkey()));
            Transaction::new(&[&payer], msg, blockhash).into()
        })
        .await
        .unwrap();

        assert_eq!(report.total, 5);
        // Not all can succeed with only 3 SOL of balance + fees.
        assert!(report.succeeded >= 1 && report.succeeded < 5);
        assert!(report.failed >= 1);
    }
}
