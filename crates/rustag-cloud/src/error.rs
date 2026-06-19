//! Control-plane error type with an axum `IntoResponse` mapping.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Errors raised by the control plane.
#[derive(Debug, thiserror::Error)]
pub enum CloudError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    Conflict(String),

    #[error("missing or invalid API key")]
    Unauthorized,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("no free ports available in the configured range")]
    NoPorts,

    #[error("orchestrator error: {0}")]
    Orchestrator(String),

    #[error("upstream stagenet error: {0}")]
    Upstream(String),
}

impl IntoResponse for CloudError {
    fn into_response(self) -> Response {
        let status = match &self {
            CloudError::NotFound(_) => StatusCode::NOT_FOUND,
            CloudError::Conflict(_) => StatusCode::CONFLICT,
            CloudError::Unauthorized => StatusCode::UNAUTHORIZED,
            CloudError::BadRequest(_) => StatusCode::BAD_REQUEST,
            CloudError::NoPorts => StatusCode::SERVICE_UNAVAILABLE,
            CloudError::Upstream(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = %self, "control-plane internal error");
        }
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

/// Convenience result alias for the control plane.
pub type Result<T> = std::result::Result<T, CloudError>;
