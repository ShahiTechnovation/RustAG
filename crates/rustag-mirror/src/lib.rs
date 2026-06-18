//! RustAG mainnet mirror.
//!
//! Fetches account state from a mainnet RPC endpoint on demand. The mirror is
//! intentionally dependency-light: it talks raw JSON-RPC over `reqwest` instead
//! of pulling `solana-rpc-client`, which would fork the Solana crate versions
//! that [`litesvm`](https://docs.rs/litesvm) unifies on.
//!
//! The mirror knows nothing about local dirty/clean tracking — that lives in
//! `rustag-core`. It is a pure read-side: "give me the current mainnet state of
//! these pubkeys".

mod account;
mod error;
mod fetcher;
mod rate_limiter;
#[cfg(feature = "realtime")]
mod realtime;
pub mod registry;
mod scheduler;

pub use account::{AccountCategory, RemoteAccount};
pub use error::MirrorError;
pub use fetcher::{MainnetMirror, MAX_ACCOUNTS_PER_REQUEST};
pub use rate_limiter::RpcRateLimiter;
#[cfg(feature = "realtime")]
pub use realtime::RealtimeMirror;
pub use scheduler::SyncIntervals;
