//! RustAG core - a persistent, mainnet-mirroring staging environment for Solana
//! programs, built on [`litesvm`](https://docs.rs/litesvm).
//!
//! The central type is [`Stagenet`]: a LiteSVM runtime augmented with
//!
//! - **lazy mainnet mirroring** - accounts are fetched from mainnet on first
//!   access and cached locally ([`Stagenet::pre_load_accounts_for_tx`]),
//! - **dirty/clean tracking** - locally-modified accounts are frozen from
//!   mainnet sync, while CLEAN oracle accounts are refreshed in the background,
//! - **unlimited airdrops** and **state overrides** for fearless testing,
//! - **SQLite persistence** so a stagenet survives restarts.
//!
//! See the workspace `README.md` and `docs/architecture.md` for the full design.

mod account_state;
mod account_store;
mod config;
mod error;
pub mod metrics;
mod realtime;
mod stagenet;
mod sync;

pub use account_state::{AccountEntry, AccountSync};
pub use account_store::{AccountStore, ScheduleRecord, StagenetRecord, TransactionRecord};
pub use config::{StagenetConfig, DEFAULT_MAINNET_RPC};
pub use error::{Result, RustagError};
pub use metrics::{MetricPoint, MetricsSnapshot};
pub use realtime::spawn_realtime_apply;
pub use stagenet::{AccountOverride, Stagenet, TxOutcome};
pub use sync::{spawn_metrics_sampler, spawn_oracle_sync};

// Re-export the mirror surface so downstream crates have a single dependency.
#[cfg(feature = "realtime")]
pub use rustag_mirror::RealtimeMirror;
pub use rustag_mirror::{registry, AccountCategory, MainnetMirror, MirrorError, RemoteAccount};
