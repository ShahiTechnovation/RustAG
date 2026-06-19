//! Error types for the compression crate.

/// All fallible concurrent-Merkle-tree operations return this.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CompressionError {
    /// `append` was called on a tree already holding `1 << max_depth` leaves.
    #[error("tree is full ({capacity} leaves)")]
    TreeFull { capacity: u64 },

    /// A leaf index referenced a position that has not been appended yet.
    #[error("leaf index {index} out of range (len {len})")]
    LeafOutOfRange { index: u64, len: u64 },

    /// A proof referenced a root that is no longer within the root-history
    /// window — the proof is too stale to fast-forward.
    #[error("proof root is not within the recent root-history window")]
    StaleRoot,

    /// The (fast-forwarded) proof did not reproduce the current root for the
    /// supplied previous leaf — the proof or previous leaf is wrong.
    #[error("invalid proof: does not authenticate the previous leaf against the current root")]
    InvalidProof,

    /// The proof had the wrong number of sibling nodes for the tree depth.
    #[error("proof length {got} does not match tree depth {expected}")]
    ProofLength { got: usize, expected: usize },

    /// `max_depth` was outside the supported `1..=30` range.
    #[error("unsupported max_depth {0} (must be 1..=30)")]
    UnsupportedDepth(u32),
}

/// Convenience alias for results in this crate.
pub type Result<T> = std::result::Result<T, CompressionError>;
