//! Error types for the replay crate.

use uuid::Uuid;

/// All fallible replay operations return this.
#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    /// An underlying stagenet operation failed.
    #[error("core error: {0}")]
    Core(#[from] rustag_core::RustagError),

    /// A journalled transaction could not be encoded or decoded.
    #[error("transaction codec error: {0}")]
    Codec(String),

    /// A lineage operation referenced a branch id that does not exist.
    #[error("unknown branch: {0}")]
    UnknownBranch(Uuid),

    /// A timeline index was out of range.
    #[error("checkpoint index {0} out of range")]
    CheckpointOutOfRange(usize),
}

/// Convenience alias for results in this crate.
pub type Result<T> = std::result::Result<T, ReplayError>;
