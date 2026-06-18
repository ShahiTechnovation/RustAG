//! RustAG simulation framework.
//!
//! Fork a stagenet into an isolated, in-memory copy, replay or stress-test
//! transactions against it, and compare outcomes — without ever mutating the
//! original or touching mainnet. This answers questions like:
//!
//! - *"What if 1,000 users liquidate simultaneously?"* — [`stress`].
//! - *"Given the same starting state, which of these strategies behaves best?"*
//!   — [`compare`].
//! - *"Replay these N transactions and tell me the success rate and CU spend."*
//!   — [`replay`] / [`fork_and_replay`].
//!
//! ```no_run
//! # async fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! use rustag_core::Stagenet;
//! use rustag_sim::stress;
//!
//! let base = Stagenet::local("base").await?;
//! let mut fork = base.fork("herd").await?;
//! let report = stress(&mut fork, "liquidations", 1_000, |_i| {
//!     // build the i-th actor's transaction here
//!     # unreachable!()
//! }).await?;
//! println!("success rate: {:.1}%", report.success_rate() * 100.0);
//! # Ok(()) }
//! ```

mod error;
mod report;
mod scenario;

pub use error::{Result, SimError};
pub use report::{ComparisonReport, ScenarioReport, TxResult};
pub use scenario::{compare, fork_and_replay, replay, stress};
