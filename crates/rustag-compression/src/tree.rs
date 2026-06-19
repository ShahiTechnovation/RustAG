//! An off-chain concurrent Merkle tree, semantically matching the on-chain
//! `spl-account-compression` program.
//!
//! Solana stores compressed account/NFT state as the *root* of a Merkle tree;
//! the leaves live off-chain. The on-chain program accepts an update to a leaf
//! if the caller supplies a proof that authenticates against a **recent** root —
//! not necessarily the very latest — which is what lets many writers update the
//! same tree concurrently without their proofs going stale on every block. This
//! crate reproduces that behavior off-chain so teams can:
//!
//! - compute the exact root their on-chain tree will hold,
//! - generate and verify proofs in their test suite, and
//! - check whether a (possibly stale) proof would still be accepted on-chain.
//!
//! The tree keeps the full leaf set in memory (it is a *testing* tool, not the
//! on-chain account), which makes it an authoritative oracle: every proof and
//! root is recomputed from ground truth, and the changelog/root-history mirror
//! the on-chain ring buffers used for stale-proof fast-forwarding.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::error::{CompressionError, Result};
use crate::hash::{empty_node_table, hash_pair, Node};

/// One recorded modification, mirroring the on-chain changelog ring buffer.
#[derive(Debug, Clone)]
struct ChangeLog {
    /// Monotonic sequence number of this change.
    seq: u64,
    /// The leaf index that changed.
    index: u64,
    /// Node values along the changed leaf's path, `path[level]` at `level`
    /// `0..max_depth` (`path[0]` is the new leaf value itself), *after* the
    /// change. Used to fast-forward stale proofs.
    path: Vec<Node>,
    /// The tree root produced by this change.
    root: Node,
}

/// A Merkle inclusion proof for a single leaf.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerklePath {
    /// Index of the proven leaf.
    pub leaf_index: u64,
    /// The leaf value being proven.
    pub leaf: Node,
    /// The root this proof was generated against.
    pub root: Node,
    /// Sibling hashes from the leaf level up to (but excluding) the root.
    pub siblings: Vec<Node>,
}

/// An off-chain concurrent Merkle tree (keccak-256, sparse, with root history).
#[derive(Debug, Clone)]
pub struct ConcurrentMerkleTree {
    max_depth: u32,
    max_buffer_size: usize,
    /// Authoritative, dense leaf storage (`leaves.len()` == number appended).
    leaves: Vec<Node>,
    /// Precomputed empty-subtree roots, indexed by level.
    empty_nodes: Vec<Node>,
    /// Recent changes, oldest-first, capped at `max_buffer_size`.
    changelog: VecDeque<ChangeLog>,
    /// The root of the empty tree (before any change).
    genesis_root: Node,
    /// Next changelog sequence number to assign.
    next_seq: u64,
}

impl ConcurrentMerkleTree {
    /// Create an empty tree of capacity `2^max_depth` with a root-history /
    /// changelog window of `max_buffer_size` recent changes.
    pub fn new(max_depth: u32, max_buffer_size: usize) -> Result<Self> {
        if !(1..=30).contains(&max_depth) {
            return Err(CompressionError::UnsupportedDepth(max_depth));
        }
        let empty_nodes = empty_node_table(max_depth);
        let genesis_root = empty_nodes[max_depth as usize];
        Ok(Self {
            max_depth,
            max_buffer_size: max_buffer_size.max(1),
            leaves: Vec::new(),
            empty_nodes,
            changelog: VecDeque::new(),
            genesis_root,
            next_seq: 0,
        })
    }

    /// Maximum number of leaves this tree can hold (`2^max_depth`).
    pub fn capacity(&self) -> u64 {
        1u64 << self.max_depth
    }

    /// Number of leaves appended so far.
    pub fn len(&self) -> u64 {
        self.leaves.len() as u64
    }

    /// Whether no leaves have been appended.
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// The tree's configured depth.
    pub fn max_depth(&self) -> u32 {
        self.max_depth
    }

    /// The current Merkle root.
    pub fn root(&self) -> Node {
        self.node_at(self.max_depth, 0)
    }

    /// The compute node at `(level, idx)`, treating unfilled subtrees as empty.
    fn node_at(&self, level: u32, idx: u64) -> Node {
        if level == 0 {
            return self
                .leaves
                .get(idx as usize)
                .copied()
                .unwrap_or(self.empty_nodes[0]);
        }
        // If the leftmost leaf of this subtree is beyond what we've filled, the
        // whole subtree is empty — return the precomputed constant (the prune
        // that keeps this O(filled · depth) instead of O(2^depth)).
        let first_leaf = idx << level;
        if first_leaf >= self.leaves.len() as u64 {
            return self.empty_nodes[level as usize];
        }
        let left = self.node_at(level - 1, idx * 2);
        let right = self.node_at(level - 1, idx * 2 + 1);
        hash_pair(&left, &right)
    }

    /// Append a leaf, returning the new root. Errors if the tree is full.
    pub fn append(&mut self, leaf: Node) -> Result<Node> {
        if self.len() >= self.capacity() {
            return Err(CompressionError::TreeFull {
                capacity: self.capacity(),
            });
        }
        let index = self.leaves.len() as u64;
        self.leaves.push(leaf);
        Ok(self.record_change(index))
    }

    /// Overwrite the leaf at `index` authoritatively (no proof required), the
    /// privileged "admin" path. Returns the new root.
    pub fn update_leaf(&mut self, index: u64, leaf: Node) -> Result<Node> {
        if index >= self.len() {
            return Err(CompressionError::LeafOutOfRange {
                index,
                len: self.len(),
            });
        }
        self.leaves[index as usize] = leaf;
        Ok(self.record_change(index))
    }

    /// Replace a leaf the way the on-chain program would: the caller proves the
    /// `previous_leaf` against a *recent* root, the proof is fast-forwarded over
    /// any intervening changes, and the update is applied only if it still
    /// authenticates against the current root.
    pub fn replace_leaf(
        &mut self,
        proof_root: Node,
        previous_leaf: Node,
        new_leaf: Node,
        index: u64,
        proof: &[Node],
    ) -> Result<Node> {
        if index >= self.len() {
            return Err(CompressionError::LeafOutOfRange {
                index,
                len: self.len(),
            });
        }
        if proof.len() != self.max_depth as usize {
            return Err(CompressionError::ProofLength {
                got: proof.len(),
                expected: self.max_depth as usize,
            });
        }
        let patched = self.fast_forward(proof_root, proof, index)?;
        if !verify_path(&self.root(), &previous_leaf, index, &patched) {
            return Err(CompressionError::InvalidProof);
        }
        self.leaves[index as usize] = new_leaf;
        Ok(self.record_change(index))
    }

    /// Fast-forward a proof built against `proof_root` to the current root by
    /// replaying every changelog entry recorded after it.
    fn fast_forward(&self, proof_root: Node, proof: &[Node], index: u64) -> Result<Vec<Node>> {
        // Sequence number after which changes must be applied.
        let after_seq: i128 = if proof_root == self.current_root_marker() {
            // Proof is already against the latest root: nothing to apply.
            return Ok(proof.to_vec());
        } else if proof_root == self.genesis_root {
            // Genesis is only reachable if we haven't trimmed past the start.
            match self.changelog.front() {
                Some(front) if front.seq == 0 => -1,
                _ => return Err(CompressionError::StaleRoot),
            }
        } else {
            match self.changelog.iter().find(|c| c.root == proof_root) {
                Some(c) => c.seq as i128,
                None => return Err(CompressionError::StaleRoot),
            }
        };

        let mut patched = proof.to_vec();
        for change in self
            .changelog
            .iter()
            .filter(|c| (c.seq as i128) > after_seq)
        {
            apply_change_to_proof(&mut patched, index, change);
        }
        Ok(patched)
    }

    /// The most recent root (latest changelog entry, or genesis if untouched).
    fn current_root_marker(&self) -> Node {
        self.changelog
            .back()
            .map(|c| c.root)
            .unwrap_or(self.genesis_root)
    }

    /// Record a change at `index`: capture the new path + root into the
    /// changelog ring buffer and return the new root.
    fn record_change(&mut self, index: u64) -> Node {
        let root = self.root();
        let path: Vec<Node> = (0..self.max_depth)
            .map(|level| self.node_at(level, index >> level))
            .collect();
        self.changelog.push_back(ChangeLog {
            seq: self.next_seq,
            index,
            path,
            root,
        });
        self.next_seq += 1;
        while self.changelog.len() > self.max_buffer_size {
            self.changelog.pop_front();
        }
        root
    }

    /// Produce an inclusion proof for the leaf at `index` against the current
    /// root.
    pub fn prove(&self, index: u64) -> Result<MerklePath> {
        if index >= self.len() {
            return Err(CompressionError::LeafOutOfRange {
                index,
                len: self.len(),
            });
        }
        let siblings = (0..self.max_depth)
            .map(|level| self.node_at(level, (index >> level) ^ 1))
            .collect();
        Ok(MerklePath {
            leaf_index: index,
            leaf: self.leaves[index as usize],
            root: self.root(),
            siblings,
        })
    }

    /// Whether `root` is the current root or any retained historical root —
    /// i.e. a proof against it could be fast-forwarded.
    pub fn is_recent_root(&self, root: &Node) -> bool {
        *root == self.current_root_marker()
            || (*root == self.genesis_root && self.changelog.front().map(|c| c.seq) == Some(0))
            || self.changelog.iter().any(|c| c.root == *root)
    }

    /// The retained root history, oldest-first (excludes genesis).
    pub fn root_history(&self) -> Vec<Node> {
        self.changelog.iter().map(|c| c.root).collect()
    }

    /// The canopy layer at `canopy_depth`: the `2^canopy_depth` subtree roots
    /// the on-chain program would cache to shorten proofs. Provided as a utility
    /// for callers that model canopy costs; proofs from [`prove`](Self::prove)
    /// are always full-depth.
    pub fn canopy_layer(&self, canopy_depth: u32) -> Vec<Node> {
        let canopy_depth = canopy_depth.min(self.max_depth);
        let level = self.max_depth - canopy_depth;
        (0..(1u64 << canopy_depth))
            .map(|i| self.node_at(level, i))
            .collect()
    }
}

/// Fold a leaf and its sibling path into a root, MSB-last (index bit `k`
/// selects whether the sibling is on the left at level `k`).
pub fn verify_path(root: &Node, leaf: &Node, index: u64, siblings: &[Node]) -> bool {
    let mut node = *leaf;
    let mut idx = index;
    for sibling in siblings {
        node = if idx & 1 == 0 {
            hash_pair(&node, sibling)
        } else {
            hash_pair(sibling, &node)
        };
        idx >>= 1;
    }
    node == *root
}

/// Patch one proof for `target_index` by a single changelog entry.
///
/// A change to leaf `c.index` alters exactly one sibling on `target_index`'s
/// path: the sibling at the level just below where the two leaves' paths merge,
/// which is the highest bit at which the indices differ. (A change to the target
/// leaf itself touches none of its siblings, so it is a no-op here — the stale
/// `previous_leaf` will simply fail the final authentication.)
fn apply_change_to_proof(proof: &mut [Node], target_index: u64, c: &ChangeLog) {
    if c.index == target_index {
        return;
    }
    let diff = c.index ^ target_index;
    let level = 63 - diff.leading_zeros(); // highest differing bit, 0-based
    if let (Some(slot), Some(value)) = (
        proof.get_mut(level as usize),
        c.path.get(level as usize).copied(),
    ) {
        *slot = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::keccak256;

    fn leaf(n: u64) -> Node {
        keccak256(&n.to_le_bytes())
    }

    #[test]
    fn empty_tree_root_is_genesis() {
        let t = ConcurrentMerkleTree::new(5, 8).unwrap();
        assert_eq!(t.root(), t.genesis_root);
        assert!(t.is_empty());
        assert_eq!(t.capacity(), 32);
    }

    #[test]
    fn unsupported_depth_rejected() {
        assert_eq!(
            ConcurrentMerkleTree::new(0, 8).unwrap_err(),
            CompressionError::UnsupportedDepth(0)
        );
        assert_eq!(
            ConcurrentMerkleTree::new(31, 8).unwrap_err(),
            CompressionError::UnsupportedDepth(31)
        );
    }

    #[test]
    fn appends_produce_verifiable_proofs() {
        let mut t = ConcurrentMerkleTree::new(6, 16).unwrap();
        for i in 0..20 {
            t.append(leaf(i)).unwrap();
        }
        let root = t.root();
        for i in 0..20 {
            let p = t.prove(i).unwrap();
            assert_eq!(p.root, root);
            assert!(
                verify_path(&root, &p.leaf, p.leaf_index, &p.siblings),
                "i={i}"
            );
        }
    }

    #[test]
    fn tree_fills_and_then_rejects() {
        let mut t = ConcurrentMerkleTree::new(2, 8).unwrap(); // capacity 4
        for i in 0..4 {
            t.append(leaf(i)).unwrap();
        }
        assert_eq!(
            t.append(leaf(99)),
            Err(CompressionError::TreeFull { capacity: 4 })
        );
    }

    #[test]
    fn update_changes_root_and_proofs_follow() {
        let mut t = ConcurrentMerkleTree::new(5, 16).unwrap();
        for i in 0..10 {
            t.append(leaf(i)).unwrap();
        }
        let before = t.root();
        t.update_leaf(3, leaf(999)).unwrap();
        let after = t.root();
        assert_ne!(before, after);
        let p = t.prove(3).unwrap();
        assert_eq!(p.leaf, leaf(999));
        assert!(verify_path(&after, &p.leaf, 3, &p.siblings));
    }

    #[test]
    fn replace_with_fresh_proof_succeeds() {
        let mut t = ConcurrentMerkleTree::new(6, 32).unwrap();
        for i in 0..30 {
            t.append(leaf(i)).unwrap();
        }
        let p = t.prove(7).unwrap();
        let new_root = t
            .replace_leaf(p.root, p.leaf, leaf(7000), 7, &p.siblings)
            .unwrap();
        assert_eq!(new_root, t.root());
        assert_eq!(t.prove(7).unwrap().leaf, leaf(7000));
    }

    #[test]
    fn stale_proof_for_untouched_leaf_fast_forwards() {
        // The core concurrency property: a proof captured for leaf 7, then made
        // stale by updates to *other* leaves, still authenticates after
        // fast-forwarding over the changelog.
        let mut t = ConcurrentMerkleTree::new(7, 64).unwrap();
        for i in 0..50 {
            t.append(leaf(i)).unwrap();
        }
        let stale = t.prove(7).unwrap();

        // Mutate several other leaves, advancing the root each time.
        for i in [1u64, 20, 33, 41, 8] {
            t.update_leaf(i, leaf(i + 100_000)).unwrap();
        }
        assert_ne!(t.root(), stale.root);

        // The stale proof still lets us replace leaf 7.
        let new_root = t
            .replace_leaf(stale.root, stale.leaf, leaf(7777), 7, &stale.siblings)
            .unwrap();
        assert_eq!(new_root, t.root());
        assert!(verify_path(
            &new_root,
            &leaf(7777),
            7,
            &t.prove(7).unwrap().siblings
        ));
    }

    #[test]
    fn stale_proof_whose_leaf_was_changed_is_rejected() {
        let mut t = ConcurrentMerkleTree::new(6, 32).unwrap();
        for i in 0..20 {
            t.append(leaf(i)).unwrap();
        }
        let stale = t.prove(9).unwrap();
        // Someone else already changed leaf 9.
        t.update_leaf(9, leaf(123)).unwrap();
        // Our stale previous_leaf no longer matches → rejected.
        assert_eq!(
            t.replace_leaf(stale.root, stale.leaf, leaf(456), 9, &stale.siblings),
            Err(CompressionError::InvalidProof)
        );
    }

    #[test]
    fn proof_root_outside_window_is_stale() {
        let mut t = ConcurrentMerkleTree::new(6, 2).unwrap(); // tiny window
        for i in 0..10 {
            t.append(leaf(i)).unwrap();
        }
        let stale = t.prove(2).unwrap();
        // Push the proof's root out of the 2-entry window.
        for i in [0u64, 1, 3, 4] {
            t.update_leaf(i, leaf(i + 1)).unwrap();
        }
        assert_eq!(
            t.replace_leaf(stale.root, stale.leaf, leaf(9), 2, &stale.siblings),
            Err(CompressionError::StaleRoot)
        );
    }

    #[test]
    fn wrong_proof_length_rejected() {
        let mut t = ConcurrentMerkleTree::new(5, 8).unwrap();
        t.append(leaf(0)).unwrap();
        let err = t.replace_leaf(t.root(), leaf(0), leaf(1), 0, &[[0u8; 32]; 2]);
        assert_eq!(
            err,
            Err(CompressionError::ProofLength {
                got: 2,
                expected: 5
            })
        );
    }

    #[test]
    fn fast_forward_matches_fresh_proof_property() {
        // Oracle test: after arbitrary updates to other leaves, the
        // fast-forwarded stale proof must equal the freshly generated proof.
        let mut t = ConcurrentMerkleTree::new(7, 64).unwrap();
        for i in 0..64 {
            t.append(leaf(i)).unwrap();
        }
        let target = 19u64;
        let stale = t.prove(target).unwrap();
        for i in [3u64, 55, 0, 40, 19_000 % 64, 2, 61] {
            if i != target {
                t.update_leaf(i, leaf(i + 500)).unwrap();
            }
        }
        let fresh = t.prove(target).unwrap();
        let patched = t.fast_forward(stale.root, &stale.siblings, target).unwrap();
        assert_eq!(patched, fresh.siblings);
    }

    #[test]
    fn canopy_layer_hashes_up_to_the_root() {
        let mut t = ConcurrentMerkleTree::new(6, 16).unwrap();
        for i in 0..40 {
            t.append(leaf(i)).unwrap();
        }
        let canopy = t.canopy_layer(2); // 4 subtree roots at level 4
        assert_eq!(canopy.len(), 4);
        // Hash the canopy pairwise up to the root and compare.
        let l = hash_pair(&canopy[0], &canopy[1]);
        let r = hash_pair(&canopy[2], &canopy[3]);
        assert_eq!(hash_pair(&l, &r), t.root());
    }
}
