//! RustAG time-travel debugging, deterministic replay & branching (Phase 3, P3.2).
//!
//! - [`Checkpoint`] snapshots a stagenet's full account state, content-addressed
//!   by its Merkle [state root](rustag_attest::state_root), and restores it into
//!   an isolated offline stagenet.
//! - [`Journal`] records every applied transaction verbatim so a checkpoint can
//!   be deterministically replayed; [`verify_deterministic`] proves two replays
//!   agree (the basis of audit-grade replay).
//! - [`Timeline`] strings checkpoints together and computes the account-level
//!   [`StateDiff`] between any two - "what changed, and where?".
//! - [`Lineage`] / [`branch_stagenet`] make fork-of-fork first-class, recording
//!   the full ancestry of every branching stagenet.
//!
//! Everything here runs fully offline against [`rustag_core::Stagenet::local`]
//! forks, so it never touches mainnet or the original stagenet.

mod checkpoint;
mod error;
mod journal;
mod lineage;
mod timeline;

pub use checkpoint::{Checkpoint, CheckpointSummary};
pub use error::{ReplayError, Result};
pub use journal::{decode_tx, encode_tx, execute_and_record, Journal, JournalEntry};
pub use lineage::{branch_stagenet, Lineage, LineageNode};
pub use timeline::{
    diff_accounts, replay_matches_journal, replay_to_root, verify_deterministic, StateDiff,
    Timeline,
};

#[cfg(test)]
mod tests {
    use super::*;
    use rustag_core::Stagenet;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_signer::Signer;
    use solana_system_interface::instruction::transfer;
    use solana_transaction::versioned::VersionedTransaction;
    use solana_transaction::Transaction;

    /// Build a signed 1-SOL transfer as a versioned transaction.
    fn transfer_tx(
        payer: &Keypair,
        to: &solana_pubkey::Pubkey,
        lamports: u64,
        bh: solana_hash::Hash,
    ) -> VersionedTransaction {
        let ix = transfer(&payer.pubkey(), to, lamports);
        let msg = Message::new(&[ix], Some(&payer.pubkey()));
        Transaction::new(&[payer], msg, bh).into()
    }

    #[tokio::test]
    async fn checkpoint_restore_preserves_state_root() {
        let mut sn = Stagenet::local("cp-base").await.unwrap();
        let payer = Keypair::new();
        sn.airdrop(&payer.pubkey(), 5_000_000_000).await.unwrap();

        let cp = Checkpoint::capture(&sn).await.unwrap();
        let restored = cp.restore("cp-restored").await.unwrap();
        let restored_accounts = restored.export_accounts().await.unwrap();
        assert_eq!(
            hex::encode(rustag_attest::state_root(&restored_accounts)),
            cp.state_root
        );
    }

    #[tokio::test]
    async fn journal_replay_is_deterministic() {
        let mut sn = Stagenet::local("j-base").await.unwrap();
        let payer = Keypair::new();
        sn.airdrop(&payer.pubkey(), 10_000_000_000).await.unwrap();

        // Snapshot the funded starting point, then journal three transfers.
        let cp = Checkpoint::capture(&sn).await.unwrap();
        let bh = sn.latest_blockhash();
        let mut journal = Journal::new();
        for i in 0..3u64 {
            let to = Keypair::new().pubkey();
            let tx = transfer_tx(&payer, &to, 1_000_000_000 + i, bh);
            execute_and_record(&mut journal, &mut sn, format!("transfer-{i}"), tx)
                .await
                .unwrap();
        }
        assert_eq!(journal.len(), 3);

        // Two independent replays from the checkpoint must agree.
        assert!(verify_deterministic(&cp, &journal).await.unwrap());

        // And the replay reproduces the root the journal *recorded* - proving
        // the replay matches the original execution, not just itself.
        assert_eq!(
            replay_matches_journal(&cp, &journal).await.unwrap(),
            Some(true)
        );

        // And the replayed root matches the live stagenet's final root.
        let live = hex::encode(rustag_attest::state_root(
            &sn.export_accounts().await.unwrap(),
        ));
        let replayed = replay_to_root(&cp, &journal, "replay-check").await.unwrap();
        assert_eq!(live, replayed);
    }

    #[tokio::test]
    async fn timeline_diff_reports_changed_accounts() {
        let mut sn = Stagenet::local("tl-base").await.unwrap();
        let payer = Keypair::new();
        sn.airdrop(&payer.pubkey(), 10_000_000_000).await.unwrap();

        let mut timeline = Timeline::new();
        let t0 = timeline.checkpoint(&sn).await.unwrap();

        // Move 1 SOL to a brand-new recipient.
        let recipient = Keypair::new().pubkey();
        let bh = sn.latest_blockhash();
        let tx = transfer_tx(&payer, &recipient, 1_000_000_000, bh);
        execute_and_record(&mut timeline.journal, &mut sn, "pay", tx)
            .await
            .unwrap();
        let t1 = timeline.checkpoint(&sn).await.unwrap();

        let diff = timeline.diff(t0, t1).unwrap();
        assert!(!diff.is_empty());
        // The recipient is newly present; the payer's balance changed.
        assert!(diff.added.contains(&recipient.to_string()));
        assert!(diff.changed.contains(&payer.pubkey().to_string()));
    }

    #[tokio::test]
    async fn branch_of_branch_is_isolated_with_lineage() {
        let mut base = Stagenet::local("br-base").await.unwrap();
        let payer = Keypair::new();
        base.airdrop(&payer.pubkey(), 10_000_000_000).await.unwrap();

        let (mut lineage, root_id) = Lineage::new_root("base", base.current_slot());

        // Branch the base, then branch the branch (fork-of-fork).
        let (mut branch1, b1) = branch_stagenet(&base, &mut lineage, root_id, "branch-1")
            .await
            .unwrap();
        let (branch2, b2) = branch_stagenet(&branch1, &mut lineage, b1, "branch-2")
            .await
            .unwrap();

        assert_eq!(lineage.depth(b2), 2);
        assert_eq!(lineage.ancestors(b2), vec![b1, root_id]);

        // Mutating branch1 does not affect branch2 (full isolation).
        let mut branch1 = std::mem::replace(&mut branch1, base.fork("scratch").await.unwrap());
        branch1.airdrop(&Keypair::new().pubkey(), 1).await.unwrap();
        let mut branch2 = branch2;
        // branch2 still holds exactly the base's payer balance, untouched.
        assert_eq!(
            branch2.get_balance(&payer.pubkey()).await.unwrap(),
            10_000_000_000
        );
    }
}
