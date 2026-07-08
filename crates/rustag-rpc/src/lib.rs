//! RustAG RPC - a Solana-compatible JSON-RPC + WebSocket server, plus a REST API
//! for the dashboard, all backed by a single [`rustag_core::Stagenet`].
//!
//! Point any Solana client (`@solana/web3.js`, `anchor`, the `solana` CLI) at the
//! JSON-RPC address and it talks to your stagenet as if it were a cluster.

mod error;
mod jsonrpc;
mod rest;
mod server;
mod state;
mod types;
mod ws;

pub use error::RpcServerError;
pub use server::{serve, ServerAddrs};
pub use state::{demo_mode_enabled, AppState};
