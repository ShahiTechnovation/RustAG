//! Scheduler error type.

/// Errors raised while parsing, validating, or executing scheduled activities.
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    /// A schedule expression could not be parsed (`@every ...` or 5-field cron).
    #[error("invalid schedule expression: {0}")]
    Schedule(String),

    /// An activity action was malformed (bad pubkey, key, or transaction blob).
    #[error("invalid activity action: {0}")]
    Action(String),

    /// The underlying stagenet runtime returned an error while executing.
    #[error(transparent)]
    Core(#[from] rustag_core::RustagError),

    /// Action (de)serialization failed.
    #[error("action serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Convenience result alias for the scheduler crate.
pub type Result<T> = std::result::Result<T, SchedulerError>;
