//! Error type for the RPC/REST server.

use std::net::SocketAddr;

/// Errors raised while starting or running the servers.
#[derive(Debug, thiserror::Error)]
pub enum RpcServerError {
    #[error("failed to bind {0}: {1}")]
    Bind(SocketAddr, #[source] std::io::Error),

    #[error("server error: {0}")]
    Serve(#[source] std::io::Error),

    #[error(transparent)]
    Core(#[from] rustag_core::RustagError),
}
