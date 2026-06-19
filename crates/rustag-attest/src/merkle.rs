//! A binary SHA-256 Merkle tree with inclusion proofs.
//!
//! Leaves and internal nodes are domain-separated (`0x00` for leaves, `0x01`
//! for nodes) to prevent second-preimage attacks where a 64-byte internal node
//! could be reinterpreted as two leaves. When a layer has an odd number of
//! nodes the final node is duplicated (hashed with itself), the standard
//! "Bitcoin-style" promotion — documented here because it affects proof shape.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{AttestError, Result};

/// A 32-byte hash digest.
pub type Hash32 = [u8; 32];

/// Root of the empty tree (zero leaves). Distinct from any real leaf hash
/// because a leaf is always `SHA-256(0x00 || payload)`, which is never all-zero
/// for any input we can realistically hit.
pub const EMPTY_ROOT: Hash32 = [0u8; 32];

const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;

/// Hash a leaf payload: `SHA-256(0x00 || payload)`.
pub fn hash_leaf(payload: &[u8]) -> Hash32 {
    let mut h = Sha256::new();
    h.update([LEAF_PREFIX]);
    h.update(payload);
    h.finalize().into()
}

/// Hash two child hashes into their parent: `SHA-256(0x01 || left || right)`.
pub fn hash_nodes(left: &Hash32, right: &Hash32) -> Hash32 {
    let mut h = Sha256::new();
    h.update([NODE_PREFIX]);
    h.update(left);
    h.update(right);
    h.finalize().into()
}

/// Decode a hex string into a 32-byte hash.
pub fn decode_hash(hex_str: &str) -> Result<Hash32> {
    let bytes = hex::decode(hex_str).map_err(|_| AttestError::BadHex(hex_str.to_string()))?;
    Hash32::try_from(bytes.as_slice()).map_err(|_| AttestError::BadHex(hex_str.to_string()))
}

/// A computed Merkle tree over a fixed set of leaf hashes.
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// `layers[0]` is the leaves; each subsequent layer is the parent layer;
    /// `layers.last()` is `[root]` (or empty for a zero-leaf tree).
    layers: Vec<Vec<Hash32>>,
}

impl MerkleTree {
    /// Build a tree from pre-hashed leaves.
    pub fn from_leaves(leaves: Vec<Hash32>) -> Self {
        if leaves.is_empty() {
            return Self {
                layers: vec![Vec::new()],
            };
        }
        let mut layers = vec![leaves];
        while layers.last().expect("non-empty").len() > 1 {
            let prev = layers.last().expect("non-empty");
            let mut next = Vec::with_capacity(prev.len().div_ceil(2));
            let mut i = 0;
            while i < prev.len() {
                let left = prev[i];
                // Duplicate the last node when the layer is odd.
                let right = if i + 1 < prev.len() {
                    prev[i + 1]
                } else {
                    prev[i]
                };
                next.push(hash_nodes(&left, &right));
                i += 2;
            }
            layers.push(next);
        }
        Self { layers }
    }

    /// Build a tree by hashing raw payloads as leaves.
    pub fn from_payloads<I, P>(payloads: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        Self::from_leaves(
            payloads
                .into_iter()
                .map(|p| hash_leaf(p.as_ref()))
                .collect(),
        )
    }

    /// The Merkle root. `EMPTY_ROOT` for a zero-leaf tree.
    pub fn root(&self) -> Hash32 {
        match self.layers.last() {
            Some(top) if top.len() == 1 => top[0],
            _ => EMPTY_ROOT,
        }
    }

    /// Number of leaves in the tree.
    pub fn leaf_count(&self) -> usize {
        self.layers.first().map(Vec::len).unwrap_or(0)
    }

    /// The leaf hash at `index`, if it exists.
    pub fn leaf(&self, index: usize) -> Option<Hash32> {
        self.layers.first().and_then(|l| l.get(index).copied())
    }

    /// Produce an inclusion proof for the leaf at `index`.
    pub fn proof(&self, index: usize) -> Result<MerkleProof> {
        if index >= self.leaf_count() {
            return Err(AttestError::LeafOutOfRange(index));
        }
        let mut siblings = Vec::new();
        let mut idx = index;
        // Walk every layer except the root layer, collecting the sibling.
        for layer in &self.layers[..self.layers.len() - 1] {
            let sibling = if idx % 2 == 0 {
                // Even index: sibling is to the right, duplicated if absent.
                if idx + 1 < layer.len() {
                    layer[idx + 1]
                } else {
                    layer[idx]
                }
            } else {
                layer[idx - 1]
            };
            siblings.push(hex::encode(sibling));
            idx /= 2;
        }
        Ok(MerkleProof {
            leaf_index: index,
            siblings,
        })
    }
}

/// A Merkle inclusion proof: the sibling hashes from a leaf up to the root.
///
/// The direction (left/right) at each level is derivable from `leaf_index`, so
/// it is not stored — keeping the artifact compact and unambiguous.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleProof {
    /// Position of the proven leaf within the leaf layer.
    pub leaf_index: usize,
    /// Sibling hashes, leaf-adjacent first, hex-encoded.
    pub siblings: Vec<String>,
}

impl MerkleProof {
    /// Recompute the root this proof implies for `leaf`, or `None` on bad hex.
    pub fn compute_root(&self, leaf: &Hash32) -> Option<Hash32> {
        let mut acc = *leaf;
        let mut idx = self.leaf_index;
        for sibling_hex in &self.siblings {
            let sibling = decode_hash(sibling_hex).ok()?;
            acc = if idx % 2 == 0 {
                hash_nodes(&acc, &sibling)
            } else {
                hash_nodes(&sibling, &acc)
            };
            idx /= 2;
        }
        Some(acc)
    }
}

/// Verify that `leaf` is included under `root` per `proof`.
pub fn verify_proof(root: &Hash32, leaf: &Hash32, proof: &MerkleProof) -> bool {
    proof.compute_root(leaf).as_ref() == Some(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaves(n: usize) -> Vec<Hash32> {
        (0..n).map(|i| hash_leaf(&[i as u8])).collect()
    }

    #[test]
    fn empty_tree_has_empty_root() {
        let t = MerkleTree::from_leaves(Vec::new());
        assert_eq!(t.root(), EMPTY_ROOT);
        assert_eq!(t.leaf_count(), 0);
    }

    #[test]
    fn single_leaf_root_is_the_leaf() {
        let l = leaves(1);
        let t = MerkleTree::from_leaves(l.clone());
        assert_eq!(t.root(), l[0]);
        let proof = t.proof(0).unwrap();
        assert!(proof.siblings.is_empty());
        assert!(verify_proof(&t.root(), &l[0], &proof));
    }

    #[test]
    fn root_is_deterministic_and_order_sensitive() {
        let a = MerkleTree::from_leaves(leaves(5)).root();
        let b = MerkleTree::from_leaves(leaves(5)).root();
        assert_eq!(a, b);
        let mut rev = leaves(5);
        rev.reverse();
        assert_ne!(MerkleTree::from_leaves(rev).root(), a);
    }

    #[test]
    fn proofs_verify_for_every_leaf_at_various_sizes() {
        for n in [2usize, 3, 4, 7, 8, 9, 16, 31] {
            let l = leaves(n);
            let t = MerkleTree::from_leaves(l.clone());
            let root = t.root();
            for (i, leaf) in l.iter().enumerate() {
                let proof = t.proof(i).unwrap();
                assert!(verify_proof(&root, leaf, &proof), "n={n} i={i}");
            }
        }
    }

    #[test]
    fn tampered_leaf_fails_verification() {
        let l = leaves(8);
        let t = MerkleTree::from_leaves(l.clone());
        let root = t.root();
        let proof = t.proof(3).unwrap();
        let forged = hash_leaf(&[99]);
        assert!(!verify_proof(&root, &forged, &proof));
    }

    #[test]
    fn out_of_range_proof_errors() {
        let t = MerkleTree::from_leaves(leaves(4));
        assert!(matches!(t.proof(4), Err(AttestError::LeafOutOfRange(4))));
    }

    #[test]
    fn proof_serde_roundtrip() {
        let t = MerkleTree::from_leaves(leaves(6));
        let proof = t.proof(2).unwrap();
        let json = serde_json::to_string(&proof).unwrap();
        let back: MerkleProof = serde_json::from_str(&json).unwrap();
        assert_eq!(proof, back);
    }
}
