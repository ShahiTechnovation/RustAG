//! Errors for the rehearsal engine, unifying the crates it orchestrates.

/// An error from a sealed rehearsal.
#[derive(Debug, thiserror::Error)]
pub enum RehearseError {
    /// A failure in the staging runtime.
    #[error(transparent)]
    Core(#[from] rustag_core::RustagError),
    /// A failure in checkpoint/journal replay.
    #[error(transparent)]
    Replay(#[from] rustag_replay::ReplayError),
    /// A failure in the attestation/evidence layer.
    #[error(transparent)]
    Attest(#[from] rustag_attest::AttestError),
    /// A failure in the simulation/analysis layer.
    #[error(transparent)]
    Sim(#[from] rustag_sim::SimError),
    /// Any other rehearsal failure (encoding, decoding, etc.).
    #[error("rehearsal: {0}")]
    Other(String),
}

/// A rehearsal result.
pub type Result<T> = std::result::Result<T, RehearseError>;
