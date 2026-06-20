//! Canonical, deterministic encoding of stagenet state into Merkle leaves.
//!
//! The whole value of an attestation rests on this being byte-for-byte
//! reproducible by a third party. Two rules make it so:
//!
//! 1. **Fixed-width, little-endian field encoding** with an explicit `data`
//!    length prefix - no ambiguity, no padding, no platform-endianness leak.
//! 2. **Accounts are sorted by pubkey** before the tree is built, so the root
//!    never depends on the order the accounts happened to be enumerated in.

use rustag_core::{AccountEntry, TxOutcome};

use crate::merkle::{hash_leaf, Hash32, MerkleTree};

/// Domain tag mixed into every account leaf (prevents cross-protocol collisions).
const ACCOUNT_LEAF_DOMAIN: &[u8] = b"rustag.account.v1";
/// Domain tag mixed into every transaction-result leaf.
const TX_RESULT_LEAF_DOMAIN: &[u8] = b"rustag.txresult.v1";

/// Canonically encode a single account into its pre-hash leaf payload.
pub fn encode_account_leaf(entry: &AccountEntry) -> Vec<u8> {
    let mut buf = Vec::with_capacity(ACCOUNT_LEAF_DOMAIN.len() + 32 + 32 + 25 + entry.data.len());
    buf.extend_from_slice(ACCOUNT_LEAF_DOMAIN);
    buf.extend_from_slice(&entry.pubkey.to_bytes());
    buf.extend_from_slice(&entry.owner.to_bytes());
    buf.extend_from_slice(&entry.lamports.to_le_bytes());
    buf.push(entry.executable as u8);
    buf.extend_from_slice(&entry.rent_epoch.to_le_bytes());
    buf.extend_from_slice(&(entry.data.len() as u64).to_le_bytes());
    buf.extend_from_slice(&entry.data);
    buf
}

/// The leaf hash for a single account.
pub fn account_leaf_hash(entry: &AccountEntry) -> Hash32 {
    hash_leaf(&encode_account_leaf(entry))
}

/// Build the Merkle tree committing to a full account set (sorted by pubkey).
pub fn state_tree(entries: &[AccountEntry]) -> MerkleTree {
    let mut sorted: Vec<&AccountEntry> = entries.iter().collect();
    sorted.sort_by_key(|e| e.pubkey.to_bytes());
    MerkleTree::from_leaves(sorted.iter().map(|e| account_leaf_hash(e)).collect())
}

/// The Merkle root committing to a full account set.
pub fn state_root(entries: &[AccountEntry]) -> Hash32 {
    state_tree(entries).root()
}

/// Canonically encode a transaction outcome into its pre-hash leaf payload.
pub fn encode_tx_result(outcome: &TxOutcome) -> Vec<u8> {
    let mut buf = Vec::with_capacity(TX_RESULT_LEAF_DOMAIN.len() + 64 + 17);
    buf.extend_from_slice(TX_RESULT_LEAF_DOMAIN);
    buf.extend_from_slice(outcome.signature.as_array());
    buf.push(outcome.success as u8);
    buf.extend_from_slice(&outcome.compute_units.to_le_bytes());
    buf.extend_from_slice(&outcome.fee.to_le_bytes());
    // Length-prefix the optional error with a presence byte so that `None` and
    // `Some("")` (and any two distinct error strings) never encode identically.
    match &outcome.err {
        Some(err) => {
            buf.push(1);
            buf.extend_from_slice(&(err.len() as u64).to_le_bytes());
            buf.extend_from_slice(err.as_bytes());
        }
        None => buf.push(0),
    }
    buf
}

/// The Merkle root committing to an ordered list of transaction outcomes.
/// Order is significant here (transactions are sequential), so outcomes are
/// **not** sorted.
pub fn tx_results_root(outcomes: &[TxOutcome]) -> Hash32 {
    MerkleTree::from_leaves(
        outcomes
            .iter()
            .map(|o| hash_leaf(&encode_tx_result(o)))
            .collect(),
    )
    .root()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustag_core::{AccountEntry, AccountSync};
    use solana_pubkey::Pubkey;
    use uuid::Uuid;

    fn entry(seed: u8, lamports: u64) -> AccountEntry {
        AccountEntry {
            pubkey: Pubkey::new_from_array([seed; 32]),
            data: vec![seed, seed, seed],
            owner: Pubkey::new_from_array([1; 32]),
            lamports,
            executable: false,
            rent_epoch: 0,
            sync_state: AccountSync::clean_now(),
            category: None,
            stagenet_id: Uuid::nil(),
        }
    }

    #[test]
    fn state_root_is_order_independent() {
        let a = vec![entry(1, 10), entry(2, 20), entry(3, 30)];
        let b = vec![entry(3, 30), entry(1, 10), entry(2, 20)];
        assert_eq!(state_root(&a), state_root(&b));
    }

    #[test]
    fn state_root_changes_when_state_changes() {
        let base = vec![entry(1, 10), entry(2, 20)];
        let mut changed = base.clone();
        changed[0].lamports = 11;
        assert_ne!(state_root(&base), state_root(&changed));
    }

    #[test]
    fn sync_state_does_not_affect_the_root() {
        // The root commits to consensus-visible account fields only - not to our
        // internal dirty/clean bookkeeping, which is not part of mainnet state.
        let mut a = entry(5, 100);
        let mut b = a.clone();
        a.sync_state = AccountSync::clean_now();
        b.sync_state = AccountSync::Pinned;
        assert_eq!(state_root(&[a]), state_root(&[b]));
    }

    #[test]
    fn empty_state_is_the_empty_root() {
        assert_eq!(state_root(&[]), crate::merkle::EMPTY_ROOT);
    }

    #[test]
    fn tx_result_none_and_empty_err_do_not_collide() {
        use rustag_core::TxOutcome;
        use solana_signature::Signature;
        let make = |err: Option<String>| TxOutcome {
            signature: Signature::from([3u8; 64]),
            success: err.is_none(),
            err,
            compute_units: 1,
            fee: 1,
            logs: Vec::new(),
        };
        assert_ne!(
            tx_results_root(&[make(None)]),
            tx_results_root(&[make(Some(String::new()))])
        );
    }
}
