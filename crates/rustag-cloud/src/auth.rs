//! API-key authentication for the control plane.
//!
//! Keys are random `rk_...` strings shown to the user exactly once; only their
//! sha256 hash is stored. Requests authenticate with `Authorization: Bearer
//! <key>`, and the [`ApiKeyAuth`] extractor resolves that to the owning tenant.

use std::fmt::Write as _;

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use sha2::{Digest, Sha256};

use crate::error::CloudError;
use crate::store::Tenant;
use crate::AppState;

/// sha256 hex digest of an API key (what the database stores).
pub fn hash_key(plaintext: &str) -> String {
    let digest = Sha256::digest(plaintext.as_bytes());
    let mut out = String::with_capacity(64);
    for byte in digest {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Extractor that authenticates a request and yields the owning [`Tenant`].
pub struct ApiKeyAuth(pub Tenant);

impl FromRequestParts<AppState> for ApiKeyAuth {
    type Rejection = CloudError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|h| {
                h.strip_prefix("Bearer ")
                    .or_else(|| h.strip_prefix("bearer "))
            })
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or(CloudError::Unauthorized)?;

        let tenant = state
            .store
            .tenant_by_key(token)
            .await?
            .ok_or(CloudError::Unauthorized)?;
        Ok(ApiKeyAuth(tenant))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_hex() {
        let a = hash_key("rk_abc");
        let b = hash_key("rk_abc");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, hash_key("rk_abd"));
    }
}
