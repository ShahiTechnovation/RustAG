//! Analytics time-series for a stagenet.
//!
//! The analytics sampler ([`crate::spawn_metrics_sampler`]) periodically captures
//! a [`MetricsSnapshot`] — a set of stagenet-level gauges — and persists it as one
//! [`MetricPoint`] row per series in the `metrics` table. The dashboard then reads
//! these back as time-series (TVL over time, transaction volume, dirty-account
//! growth, ...).
//!
//! Series names are stable strings (the `SERIES_*` constants) so the storage
//! layer, the REST API, and the dashboard all agree on them. In production the
//! `metrics` table becomes a Postgres/TimescaleDB hypertable on `recorded_at`.

use chrono::{DateTime, Utc};

/// Number of accounts mirrored into the stagenet.
pub const SERIES_ACCOUNTS: &str = "accounts";
/// Cumulative number of transactions processed.
pub const SERIES_TRANSACTIONS: &str = "transactions";
/// Number of locally-modified (dirty/pinned) accounts.
pub const SERIES_DIRTY: &str = "dirty_accounts";
/// The current monotonic slot.
pub const SERIES_SLOT: &str = "slot";
/// Total value locked, in lamports (sum of lamports across all accounts).
pub const SERIES_TVL_LAMPORTS: &str = "tvl_lamports";

/// Every series the analytics sampler emits, for API discovery and validation.
pub const ALL_SERIES: &[&str] = &[
    SERIES_ACCOUNTS,
    SERIES_TRANSACTIONS,
    SERIES_DIRTY,
    SERIES_SLOT,
    SERIES_TVL_LAMPORTS,
];

/// A single persisted measurement: `(series, value)` at a point in time.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricPoint {
    pub series: String,
    pub value: f64,
    pub recorded_at: DateTime<Utc>,
}

/// A point-in-time capture of stagenet-level gauges, fanned out into one
/// [`MetricPoint`] per series for storage.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub accounts: i64,
    pub transactions: i64,
    pub dirty_accounts: i64,
    pub slot: u64,
    /// Total value locked, in lamports. Stored as `f64` because "unlimited
    /// airdrops" can push the sum past `i64::MAX`; SQLite `TOTAL()` returns a
    /// REAL so the aggregate never overflows or flips sign.
    pub tvl_lamports: f64,
    pub recorded_at: DateTime<Utc>,
}

impl MetricsSnapshot {
    /// Expand this snapshot into the per-series rows the store persists.
    pub fn into_points(self) -> Vec<MetricPoint> {
        let at = self.recorded_at;
        let p = |series: &str, value: f64| MetricPoint {
            series: series.to_string(),
            value,
            recorded_at: at,
        };
        vec![
            p(SERIES_ACCOUNTS, self.accounts as f64),
            p(SERIES_TRANSACTIONS, self.transactions as f64),
            p(SERIES_DIRTY, self.dirty_accounts as f64),
            p(SERIES_SLOT, self.slot as f64),
            p(SERIES_TVL_LAMPORTS, self.tvl_lamports),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_expands_to_one_point_per_series() {
        let snap = MetricsSnapshot {
            accounts: 12,
            transactions: 7,
            dirty_accounts: 3,
            slot: 99,
            tvl_lamports: 5_000_000_000.0,
            recorded_at: Utc::now(),
        };
        let points = snap.into_points();
        assert_eq!(points.len(), ALL_SERIES.len());
        let tvl = points
            .iter()
            .find(|p| p.series == SERIES_TVL_LAMPORTS)
            .unwrap();
        assert_eq!(tvl.value, 5_000_000_000.0);
    }
}
