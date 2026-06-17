//! Backpressure for outbound mainnet RPC calls.

use std::num::NonZeroU32;

use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};

/// A simple requests-per-second limiter wrapping [`governor`].
///
/// Free mainnet RPC tiers (Helius, public endpoint) rate-limit aggressively;
/// exceeding their quota gets the whole stagenet throttled. This keeps us under
/// the configured ceiling.
pub struct RpcRateLimiter {
    inner: DefaultDirectRateLimiter,
}

impl RpcRateLimiter {
    /// Build a limiter allowing at most `max_rps` requests per second
    /// (clamped to at least 1, with a small burst allowance).
    pub fn new(max_rps: u32) -> Self {
        let rps = NonZeroU32::new(max_rps.max(1)).expect("max_rps clamped to >= 1");
        let quota = Quota::per_second(rps).allow_burst(rps);
        Self {
            inner: RateLimiter::direct(quota),
        }
    }

    /// Wait until the limiter permits another request.
    pub async fn acquire(&self) {
        self.inner.until_ready().await;
    }
}
