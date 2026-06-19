//! `rustag tree` — build an off-chain concurrent Merkle tree and print its root
//! and proofs (Phase 3, P3.4).
//!
//! A standalone utility for teams working with compressed accounts/NFTs: compute
//! the exact `spl-account-compression` root for a set of leaves and generate
//! inclusion proofs, with no stagenet or network required.

use anyhow::{bail, Result};
use clap::Args;

use rustag_compression::{keccak256, verify_path, ConcurrentMerkleTree, Node};

use super::{info, ok};

#[derive(Args)]
pub struct TreeArgs {
    /// Tree depth (capacity is `2^depth`).
    #[arg(long, default_value_t = 14)]
    pub depth: u32,
    /// Root-history / changelog window size.
    #[arg(long, default_value_t = 64)]
    pub buffer: usize,
    /// Leaves to append, in order. A 64-char hex string is taken as a raw
    /// 32-byte node; anything else is keccak-256 hashed. Repeatable.
    #[arg(long = "leaf")]
    pub leaves: Vec<String>,
    /// Print an inclusion proof for this leaf index.
    #[arg(long)]
    pub prove: Option<u64>,
}

pub async fn run(args: TreeArgs) -> Result<()> {
    let mut tree = ConcurrentMerkleTree::new(args.depth, args.buffer)
        .map_err(|e| anyhow::anyhow!("invalid tree parameters: {e}"))?;

    for (i, leaf) in args.leaves.iter().enumerate() {
        let node = parse_leaf(leaf);
        tree.append(node)
            .map_err(|e| anyhow::anyhow!("failed to append leaf {i}: {e}"))?;
    }

    println!();
    ok(format!("root        {}", hex::encode(tree.root())));
    info(format!(
        "depth {}   leaves {}   capacity {}   buffer {}",
        args.depth,
        tree.len(),
        tree.capacity(),
        args.buffer
    ));

    if let Some(index) = args.prove {
        if index >= tree.len() {
            bail!(
                "cannot prove leaf {index}: only {} leaf/leaves appended",
                tree.len()
            );
        }
        let proof = tree
            .prove(index)
            .map_err(|e| anyhow::anyhow!("failed to build proof: {e}"))?;
        let valid = verify_path(&proof.root, &proof.leaf, proof.leaf_index, &proof.siblings);
        println!();
        info(format!(
            "proof for leaf {index} ({} siblings):",
            proof.siblings.len()
        ));
        for (level, sibling) in proof.siblings.iter().enumerate() {
            info(format!("  level {level:>2}  {}", hex::encode(sibling)));
        }
        if valid {
            ok("proof verifies against the root");
        } else {
            bail!("internal error: generated proof did not verify");
        }
    }
    Ok(())
}

/// Parse a leaf argument: a 64-char hex string is a raw node, otherwise the
/// UTF-8 bytes are keccak-256 hashed (the common way to derive a leaf from data).
fn parse_leaf(s: &str) -> Node {
    if s.len() == 64 {
        if let Ok(bytes) = hex::decode(s) {
            if let Ok(node) = Node::try_from(bytes.as_slice()) {
                return node;
            }
        }
    }
    keccak256(s.as_bytes())
}
