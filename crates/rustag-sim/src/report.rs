//! Simulation results: per-transaction outcomes and aggregate statistics.

use serde::Serialize;

use rustag_core::TxOutcome;

/// The outcome of a single transaction within a scenario.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TxResult {
    pub index: usize,
    pub signature: String,
    pub success: bool,
    pub err: Option<String>,
    pub compute_units: u64,
    pub fee: u64,
}

/// The result of running one scenario (a sequence of transactions against a
/// single fork): per-tx outcomes plus aggregate statistics.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioReport {
    pub label: String,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub total_compute_units: u64,
    pub max_compute_units: u64,
    pub total_fees: u64,
    pub duration_ms: u128,
    pub outcomes: Vec<TxResult>,
}

impl ScenarioReport {
    pub(crate) fn new(label: impl Into<String>, capacity: usize) -> Self {
        Self {
            label: label.into(),
            total: 0,
            succeeded: 0,
            failed: 0,
            total_compute_units: 0,
            max_compute_units: 0,
            total_fees: 0,
            duration_ms: 0,
            outcomes: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn record(&mut self, index: usize, outcome: &TxOutcome) {
        self.total += 1;
        if outcome.success {
            self.succeeded += 1;
        } else {
            self.failed += 1;
        }
        self.total_compute_units = self
            .total_compute_units
            .saturating_add(outcome.compute_units);
        self.max_compute_units = self.max_compute_units.max(outcome.compute_units);
        self.total_fees = self.total_fees.saturating_add(outcome.fee);
        self.outcomes.push(TxResult {
            index,
            signature: outcome.signature_string(),
            success: outcome.success,
            err: outcome.err.clone(),
            compute_units: outcome.compute_units,
            fee: outcome.fee,
        });
    }

    /// Fraction of transactions that succeeded, in `0.0..=1.0`.
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.succeeded as f64 / self.total as f64
        }
    }

    /// Mean compute units per transaction.
    pub fn mean_compute_units(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.total_compute_units as f64 / self.total as f64
        }
    }
}

/// The result of comparing several scenarios run against independent forks of
/// the same base stagenet ("fork, replay N variants, compare outcomes").
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonReport {
    pub variants: Vec<ScenarioReport>,
}

impl ComparisonReport {
    /// The variant with the highest success rate (ties broken by lower mean CU).
    pub fn best_by_success_rate(&self) -> Option<&ScenarioReport> {
        self.variants.iter().max_by(|a, b| {
            a.success_rate()
                .partial_cmp(&b.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    b.mean_compute_units()
                        .partial_cmp(&a.mean_compute_units())
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
        })
    }
}
