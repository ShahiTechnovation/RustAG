//! Error type for the mainnet mirror.

/// Errors that can occur while fetching state from a mainnet RPC endpoint.
#[derive(Debug, thiserror::Error)]
pub enum MirrorError {
    /// The HTTP transport failed (DNS, TLS, timeout, connection reset, ...).
    #[error("mainnet RPC transport error: {0}")]
    Http(#[from] reqwest::Error),

    /// The endpoint returned a JSON-RPC `error` object.
    #[error("mainnet RPC error {code}: {message}")]
    Rpc { code: i64, message: String },

    /// The response could not be understood (missing fields, wrong shape).
    #[error("malformed mainnet RPC response: {0}")]
    InvalidResponse(String),

    /// A pubkey returned by the endpoint was not valid base58.
    #[error("invalid pubkey in response: {0}")]
    InvalidPubkey(String),

    /// Account `data` could not be decoded from its wire encoding.
    #[error("failed to decode account data: {0}")]
    Decode(String),

    /// The real-time WebSocket transport failed (connect, send, or stream).
    #[error("realtime websocket error: {0}")]
    WebSocket(String),
}
