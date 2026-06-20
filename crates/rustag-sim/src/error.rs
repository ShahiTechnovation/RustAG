//! Simulation error type.

/// Errors raised while running a simulation. A *failed transaction* is not an
/// error - it is a recorded outcome in the report; an [`SimError`] means the
/// harness itself could not run (e.g. forking the base stagenet failed).
#[derive(Debug, thiserror::Error)]
pub enum SimError {
    /// The underlying stagenet runtime returned an error.
    #[error(transparent)]
    Core(#[from] rustag_core::RustagError),
}

/// Convenience result alias for the simulation crate.
pub type Result<T> = std::result::Result<T, SimError>;
