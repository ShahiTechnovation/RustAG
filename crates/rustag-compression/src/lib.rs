//! RustAG state / ZK compression testing (Phase 3, P3.4).
//!
//! An off-chain, authoritative simulator of Solana's `spl-account-compression`
//! concurrent Merkle tree. Teams building compressed-NFT / compressed-account
//! programs can use it to compute the exact on-chain root, generate and verify
//! proofs in their test suites, and check whether a stale proof would still be
//! accepted on-chain (the concurrency property).
//!
//! ```
//! use rustag_compression::{ConcurrentMerkleTree, keccak256, verify_path};
//!
//! let mut tree = ConcurrentMerkleTree::new(14, 64).unwrap();
//! let root = tree.append(keccak256(b"first compressed leaf")).unwrap();
//!
//! let proof = tree.prove(0).unwrap();
//! assert!(verify_path(&root, &proof.leaf, proof.leaf_index, &proof.siblings));
//! ```

mod error;
mod hash;
mod tree;

pub use error::{CompressionError, Result};
pub use hash::{empty_node_table, hash_pair, keccak256, Node};
pub use tree::{verify_path, ConcurrentMerkleTree, MerklePath};
