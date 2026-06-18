//! Reverse proxy: routes `/{slug}/rpc` and `/{slug}/api/*` to the hosted
//! stagenet's child process, so every stagenet is reachable through the single
//! control-plane endpoint (and, in production, a per-subdomain router).

use axum::body::Bytes;
use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::error::{CloudError, Result};
use crate::AppState;

/// Forward a JSON-RPC request to a stagenet's RPC port.
pub async fn proxy_rpc(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    body: Bytes,
) -> Result<Response> {
    let sn = state
        .store
        .get_stagenet(&slug)
        .await?
        .ok_or_else(|| CloudError::NotFound(slug.clone()))?;
    if sn.status != "running" {
        return Err(CloudError::Upstream(format!(
            "stagenet '{slug}' is {} (not running)",
            sn.status
        )));
    }
    let url = format!("http://127.0.0.1:{}/", sn.rpc_port);
    let resp = state
        .http
        .post(url)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| CloudError::Upstream(e.to_string()))?;
    relay(resp).await
}

/// Forward a REST request to a stagenet's REST API port, preserving method,
/// query string, and body.
pub async fn proxy_api(
    State(state): State<AppState>,
    Path((slug, rest)): Path<(String, String)>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    body: Bytes,
) -> Result<Response> {
    let sn = state
        .store
        .get_stagenet(&slug)
        .await?
        .ok_or_else(|| CloudError::NotFound(slug.clone()))?;
    let query = uri.query().map(|q| format!("?{q}")).unwrap_or_default();
    let url = format!("http://127.0.0.1:{}/api/{rest}{query}", sn.api_port);
    let rmethod =
        reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap_or(reqwest::Method::GET);
    let mut req = state.http.request(rmethod, url);
    if !body.is_empty() {
        req = req.header("content-type", "application/json").body(body);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| CloudError::Upstream(e.to_string()))?;
    relay(resp).await
}

/// Relay an upstream stagenet response back to the client, preserving status
/// code and content type.
async fn relay(resp: reqwest::Response) -> Result<Response> {
    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json")
        .to_string();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| CloudError::Upstream(e.to_string()))?;
    let mut response = (status, bytes).into_response();
    if let Ok(value) = HeaderValue::from_str(&content_type) {
        response
            .headers_mut()
            .insert(axum::http::header::CONTENT_TYPE, value);
    }
    Ok(response)
}
