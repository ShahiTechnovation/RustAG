//! Keccak-256 hashing primitives, matching `spl-account-compression`.
//!
//! On-chain, Solana's account-compression program hashes nodes with
//! `solana_program::keccak` (Keccak-256, *not* NIST SHA3-256). Using the same
//! function here means a root computed off-chain by this crate is byte-for-byte
//! identical to the root the on-chain program would hold for the same leaves.

use sha3::{Digest, Keccak256};

/// A 32-byte Merkle node.
pub type Node = [u8; 32];

/// Keccak-256 of an arbitrary byte string (e.g. to derive a leaf from data).
pub fn keccak256(data: &[u8]) -> Node {
    Keccak256::digest(data).into()
}

/// Hash two child nodes into their parent: `keccak256(left || right)`.
pub fn hash_pair(left: &Node, right: &Node) -> Node {
    let mut h = Keccak256::new();
    h.update(left);
    h.update(right);
    h.finalize().into()
}

/// Precompute the empty-subtree root at every level `0..=max_depth`.
///
/// `empty[0]` is the all-zero leaf; `empty[i] = keccak(empty[i-1], empty[i-1])`.
/// These are the canonical "no data here" nodes that a sparse tree uses for
/// unfilled subtrees - identical to the on-chain program's constants.
pub fn empty_node_table(max_depth: u32) -> Vec<Node> {
    let mut table = Vec::with_capacity(max_depth as usize + 1);
    table.push([0u8; 32]);
    for level in 1..=max_depth as usize {
        let prev = table[level - 1];
        table.push(hash_pair(&prev, &prev));
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_table_grows_by_self_hashing() {
        let t = empty_node_table(3);
        assert_eq!(t.len(), 4);
        assert_eq!(t[0], [0u8; 32]);
        assert_eq!(t[1], hash_pair(&t[0], &t[0]));
        assert_eq!(t[2], hash_pair(&t[1], &t[1]));
        assert_eq!(t[3], hash_pair(&t[2], &t[2]));
    }

    #[test]
    fn hash_pair_is_order_sensitive() {
        let a = keccak256(b"a");
        let b = keccak256(b"b");
        assert_ne!(hash_pair(&a, &b), hash_pair(&b, &a));
    }
}
