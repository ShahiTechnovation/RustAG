//! RustAG ingest pipeline.
//!
//! Fetches account state from mainnet RPC endpoints on demand. The mirror is
//! intentionally dependency-light: it talks raw JSON-RPC over `reqwest` instead
//! of pulling `solana-rpc-client`, which would fork the Solana crate versions
//! that [`litesvm`](https://docs.rs/litesvm) unifies on.
//!
//! The mirror knows nothing about local dirty/clean tracking — that lives in
//! `rustag-core`. It is a pure read-side: "give me the current mainnet state of
//! these pubkeys."
//!
//! ## GroundTruth ingest components
//!
//! - [`MainnetMirror`] — single-endpoint lazy account fetcher.
//! - [`TouchSetResolver`](touch_set::TouchSetResolver) — resolves the full
//!   account closure a `VersionedMessage` will touch (static keys + v0 ALTs +
//!   ProgramData PDAs + Clock sysvar).
//! - [`SquadsDecoder`](squads::SquadsDecoder) — fetches and decodes Squads v4
//!   `VaultTransaction` proposals from mainnet.
//! - [`MultiRpcFetcher`](multi_rpc::MultiRpcFetcher) — N-of-M cross-RPC fetch
//!   with [`InputProvenance`](multi_rpc::InputProvenance) tracking.

mod account;
mod error;
mod fetcher;
pub mod forward_recorder;
pub mod multi_rpc;
mod rate_limiter;
#[cfg(feature = "realtime")]
mod realtime;
pub mod registry;
mod scheduler;
pub mod squads;
pub mod touch_set;

pub use account::{AccountCategory, RemoteAccount};
pub use error::MirrorError;
pub use fetcher::{MainnetMirror, MAX_ACCOUNTS_PER_REQUEST};
pub use forward_recorder::{ForwardRecorder, RecordedCorpus, RecordedTransaction};
pub use multi_rpc::{InputProvenance, MultiRpcFetcher};
pub use rate_limiter::RpcRateLimiter;
#[cfg(feature = "realtime")]
pub use realtime::RealtimeMirror;
pub use scheduler::SyncIntervals;
pub use squads::{ProposedPayload, SquadsDecoder, SQUADS_V4_PROGRAM};
pub use touch_set::{AddressTableLookup, TouchSetResolver};
