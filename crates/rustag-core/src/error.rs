//! Error types for the RustAG core runtime.

use solana_pubkey::Pubkey;

/// All fallible RustAG core operations return this.
#[derive(Debug, thiserror::Error)]
pub enum RustagError {
    /// The requested account does not exist locally and could not be fetched.
    #[error("account not found: {0}")]
    AccountNotFound(Pubkey),

    /// A mainnet mirror operation failed.
    #[error("mainnet mirror error: {0}")]
    Mirror(#[from] rustag_mirror::MirrorError),

    /// A database operation failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// A database migration failed.
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    /// `LiteSVM` rejected a transaction.
    #[error("transaction failed: {0}")]
    TransactionFailed(String),

    /// `LiteSVM` rejected an airdrop.
    #[error("airdrop failed: {0}")]
    AirdropFailed(String),

    /// A pubkey string could not be parsed.
    #[error("invalid pubkey: {0}")]
    InvalidPubkey(String),

    /// A transaction blob could not be decoded/deserialized.
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    /// An account was not a valid SPL token account.
    #[error("not a valid SPL token account: {0}")]
    InvalidTokenAccount(Pubkey),

    /// A mirror-dependent operation was attempted while the mirror was disabled.
    #[error("mainnet mirror is disabled for this stagenet")]
    MirrorDisabled,

    /// A stagenet with the given name/id was not found.
    #[error("stagenet not found: {0}")]
    StagenetNotFound(String),

    /// Serialization/deserialization of stored metadata failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// A filesystem error occurred (e.g. creating the database directory).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience alias for results in this crate.
pub type Result<T> = std::result::Result<T, RustagError>;
