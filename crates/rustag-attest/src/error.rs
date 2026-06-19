//! Error types for the attestation crate.

/// All fallible attestation operations return this.
#[derive(Debug, thiserror::Error)]
pub enum AttestError {
    /// A Merkle proof referenced a leaf index that does not exist in the tree.
    #[error("merkle leaf index out of range: {0}")]
    LeafOutOfRange(usize),

    /// The attester field was not a valid base58 Solana public key.
    #[error("invalid attester pubkey: {0}")]
    BadPubkey(String),

    /// The signature field was not valid base58 / not 64 bytes.
    #[error("invalid signature encoding")]
    BadSignature,

    /// A hex-encoded hash could not be decoded into 32 bytes.
    #[error("invalid hex hash: {0}")]
    BadHex(String),

    /// JSON (de)serialization of an attestation artifact failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// A filesystem error occurred reading/writing an artifact or key file.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience alias for results in this crate.
pub type Result<T> = std::result::Result<T, AttestError>;
