//! Shared server state.

use std::sync::Arc;

use tokio::sync::RwLock;

use rustag_core::Stagenet;

/// Maximum airdrop honored while in public-demo mode, in lamports (100 SOL).
/// Enough for any realistic test, small enough that it cannot be used to poison
/// the aggregate-balance (TVL) metric the dashboard charts. Only enforced when
/// [`AppState::demo_mode`] is set; local/CLI use stays unlimited.
pub const MAX_DEMO_AIRDROP_LAMPORTS: u64 = 100 * 1_000_000_000;

/// State shared by every JSON-RPC, REST, and WebSocket handler.
///
/// The stagenet is behind an async `RwLock` because most operations need
/// `&mut Stagenet` (the lazy mirror mutates the SVM, cache, and store).
#[derive(Clone)]
pub struct AppState {
    pub stagenet: Arc<RwLock<Stagenet>>,
    /// When set (via `RUSTAG_DEMO_MODE`), the server runs in public-demo mode:
    /// state-mutating and mainnet-quota-draining routes (`override`, `preload`,
    /// schedule writes) are refused, and airdrops are capped
    /// ([`MAX_DEMO_AIRDROP_LAMPORTS`]). Reads, capped airdrops, and the
    /// fork-isolated `simulate` stay live so an anonymous visitor gets a real,
    /// interactive experience without being able to grief the shared stagenet or
    /// exhaust the upstream RPC key.
    pub demo_mode: bool,
}

impl AppState {
    pub fn new(stagenet: Arc<RwLock<Stagenet>>) -> Self {
        Self {
            stagenet,
            demo_mode: demo_mode_enabled(),
        }
    }
}

/// Whether the server is in public-demo mode, read from `RUSTAG_DEMO_MODE`. The
/// single source of truth so the CLI (`rustag serve`) and the request handlers
/// agree on when demo affordances (capped airdrops, gated writes, the seeded
/// heartbeat) apply.
pub fn demo_mode_enabled() -> bool {
    std::env::var("RUSTAG_DEMO_MODE")
        .ok()
        .is_some_and(|v| is_truthy(&v))
}

/// Interpret common truthy strings (`1`, `true`, `yes`, `on`) case-insensitively.
fn is_truthy(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::is_truthy;

    #[test]
    fn truthy_values_enable_demo_mode() {
        for v in ["1", "true", "TRUE", "  Yes ", "on"] {
            assert!(is_truthy(v), "{v:?} should be truthy");
        }
    }

    #[test]
    fn falsy_values_leave_demo_mode_off() {
        for v in ["", "0", "false", "no", "off", "maybe"] {
            assert!(!is_truthy(v), "{v:?} should be falsy");
        }
    }
}
