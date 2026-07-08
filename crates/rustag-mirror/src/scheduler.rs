//! Sync cadence configuration.
//!
//! The actual background loop lives in `rustag-core`, which owns both the
//! account store and the SVM it needs to refresh. This module just carries the
//! intervals so the mirror remains a pure read-side with no dependency on the
//! core crate (avoiding a dependency cycle).

use std::time::Duration;

/// How often each class of CLEAN account is re-fetched from mainnet.
#[derive(Debug, Clone, Copy)]
pub struct SyncIntervals {
    /// Oracle accounts (Pyth) - kept fresh aggressively.
    pub oracle: Duration,
    /// Everything else - synced lazily.
    pub default: Duration,
}

impl Default for SyncIntervals {
    fn default() -> Self {
        Self {
            oracle: Duration::from_secs(30),
            default: Duration::from_secs(300),
        }
    }
}

impl SyncIntervals {
    /// Build from interval values expressed in seconds.
    pub fn from_secs(oracle_secs: u64, default_secs: u64) -> Self {
        Self {
            oracle: Duration::from_secs(oracle_secs.max(1)),
            default: Duration::from_secs(default_secs.max(1)),
        }
    }
}
