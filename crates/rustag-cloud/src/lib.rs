//! RustAG cloud control plane.
//!
//! A multi-tenant orchestrator that hosts many stagenets behind one HTTP
//! endpoint. Tenants authenticate with API keys; each stagenet runs as an
//! isolated child `rustag` process with its own ports and data directory, and is
//! reachable through a reverse proxy at `/{slug}/rpc` and `/{slug}/api/*` (in
//! production, fronted by a per-subdomain router so each project gets
//! `my-project.stagesvm.dev`).
//!
//! Endpoints:
//! - `POST /v1/signup`            - create a tenant, returns the first API key.
//! - `POST /v1/api-keys`          - issue another API key (auth).
//! - `POST /v1/stagenets`         - create + start a stagenet (auth).
//! - `GET  /v1/stagenets`         - list the tenant's stagenets (auth).
//! - `GET  /v1/stagenets/{slug}`  - one stagenet (auth).
//! - `DELETE /v1/stagenets/{slug}`- stop + delete a stagenet (auth).
//! - `POST /{slug}/rpc`           - JSON-RPC proxy to the stagenet.
//! - `ANY  /{slug}/api/*`         - REST proxy to the stagenet.

mod auth;
mod config;
mod error;
mod orchestrator;
mod proxy;
mod store;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::auth::ApiKeyAuth;
use crate::orchestrator::Orchestrator;
use crate::store::{CloudStagenet, ControlPlaneStore};

pub use config::CloudConfig;
pub use error::{CloudError, Result};
pub use store::{CloudStagenet as StagenetRecord, Tenant};

/// The control-plane crate version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Shared state for every handler.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<ControlPlaneStore>,
    pub orch: Arc<Orchestrator>,
    pub http: reqwest::Client,
    pub config: Arc<CloudConfig>,
}

/// Build the control-plane router.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/signup", post(signup))
        .route("/v1/api-keys", post(create_api_key))
        .route("/v1/stagenets", post(create_stagenet).get(list_stagenets))
        .route(
            "/v1/stagenets/{slug}",
            get(get_stagenet).delete(delete_stagenet),
        )
        .route("/{slug}/rpc", post(proxy::proxy_rpc))
        .route("/{slug}/api/{*rest}", any(proxy::proxy_api))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Connect, build, and serve the control plane until shutdown.
pub async fn run(config: CloudConfig) -> anyhow::Result<()> {
    let store = Arc::new(ControlPlaneStore::connect(&config.control_db).await?);
    // Reconcile state left by a previous process: stagenets we no longer
    // supervise are marked stopped (their child handles died with the old process).
    match store.reset_running_to_stopped().await {
        Ok(n) if n > 0 => tracing::warn!(reconciled = n, "marked orphaned stagenets as stopped"),
        Ok(_) => {}
        Err(e) => tracing::warn!(error = %e, "startup reconciliation failed"),
    }
    std::fs::create_dir_all(&config.data_root).ok();
    let config = Arc::new(config);
    let orch = Arc::new(Orchestrator::new(Arc::clone(&config)));
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let state = AppState {
        store,
        orch,
        http,
        config: Arc::clone(&config),
    };

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "rustag-cloud control plane listening");
    axum::serve(listener, app(state)).await?;
    Ok(())
}

// --- handlers ---------------------------------------------------------------

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "rustag-cloud", "version": version() }))
}

#[derive(Deserialize)]
struct SignupBody {
    name: String,
    email: String,
}

async fn signup(
    State(state): State<AppState>,
    Json(body): Json<SignupBody>,
) -> Result<Json<Value>> {
    if body.email.trim().is_empty() {
        return Err(CloudError::BadRequest("email is required".into()));
    }
    let (tenant, api_key) = state
        .store
        .create_tenant(body.name.trim(), body.email.trim())
        .await?;
    Ok(Json(json!({
        "tenant": tenant,
        "apiKey": api_key,
        "note": "store this API key now - it is shown only once",
    })))
}

#[derive(Deserialize)]
struct ApiKeyBody {
    label: Option<String>,
}

async fn create_api_key(
    State(state): State<AppState>,
    ApiKeyAuth(tenant): ApiKeyAuth,
    Json(body): Json<ApiKeyBody>,
) -> Result<Json<Value>> {
    let key = state
        .store
        .issue_api_key(&tenant.id, body.label.as_deref())
        .await?;
    Ok(Json(json!({ "apiKey": key, "note": "shown only once" })))
}

#[derive(Deserialize)]
struct CreateStagenetBody {
    name: String,
    slug: Option<String>,
    #[serde(rename = "mainnetRpc")]
    mainnet_rpc: Option<String>,
}

async fn create_stagenet(
    State(state): State<AppState>,
    ApiKeyAuth(tenant): ApiKeyAuth,
    Json(body): Json<CreateStagenetBody>,
) -> Result<Json<Value>> {
    let slug = slugify(body.slug.as_deref().unwrap_or(&body.name));
    if slug.is_empty() {
        return Err(CloudError::BadRequest(
            "could not derive a slug from the name".into(),
        ));
    }
    if state.store.slug_exists(&slug).await? {
        return Err(CloudError::Conflict(format!("slug '{slug}' is taken")));
    }
    // Per-tenant quota: cap resource exhaustion from a single account.
    let existing = state.store.list_stagenets(&tenant.id).await?;
    let active = existing.iter().filter(|s| s.status != "error").count();
    if active >= state.config.max_stagenets_per_tenant {
        return Err(CloudError::BadRequest(format!(
            "stagenet quota reached ({} max per tenant)",
            state.config.max_stagenets_per_tenant
        )));
    }
    let mainnet = body
        .mainnet_rpc
        .unwrap_or_else(|| state.config.default_mainnet_rpc.clone());
    let rec = state
        .orch
        .create_and_start(&state.store, tenant.id, &slug, body.name.trim(), &mainnet)
        .await?;
    Ok(Json(stagenet_json(&rec, &state.config)))
}

async fn list_stagenets(
    State(state): State<AppState>,
    ApiKeyAuth(tenant): ApiKeyAuth,
) -> Result<Json<Value>> {
    let records = state.store.list_stagenets(&tenant.id).await?;
    let out: Vec<Value> = records
        .iter()
        .map(|r| stagenet_json(r, &state.config))
        .collect();
    Ok(Json(json!({ "stagenets": out })))
}

async fn get_stagenet(
    State(state): State<AppState>,
    ApiKeyAuth(tenant): ApiKeyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Value>> {
    let rec = owned_stagenet(&state, &tenant.id, &slug).await?;
    Ok(Json(stagenet_json(&rec, &state.config)))
}

async fn delete_stagenet(
    State(state): State<AppState>,
    ApiKeyAuth(tenant): ApiKeyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Value>> {
    // Verify ownership before touching the process or the row.
    owned_stagenet(&state, &tenant.id, &slug).await?;
    state.orch.stop(&slug).await?;
    state.store.delete_stagenet(&slug).await?;
    Ok(Json(json!({ "ok": true, "slug": slug })))
}

// --- helpers ----------------------------------------------------------------

/// Fetch a stagenet, returning `NotFound` if it does not belong to `tenant_id`
/// (so one tenant can never probe another's slugs).
async fn owned_stagenet(
    state: &AppState,
    tenant_id: &uuid::Uuid,
    slug: &str,
) -> Result<CloudStagenet> {
    let rec = state
        .store
        .get_stagenet(slug)
        .await?
        .filter(|r| &r.tenant_id == tenant_id)
        .ok_or_else(|| CloudError::NotFound(format!("stagenet '{slug}'")))?;
    Ok(rec)
}

fn stagenet_json(rec: &CloudStagenet, config: &CloudConfig) -> Value {
    let base = public_base(config);
    let mut value = serde_json::to_value(rec).unwrap_or_else(|_| json!({}));
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "rpcUrl".into(),
            json!(format!("http://{base}/{}/rpc", rec.slug)),
        );
        obj.insert(
            "apiUrl".into(),
            json!(format!("http://{base}/{}/api", rec.slug)),
        );
        obj.insert(
            "directRpc".into(),
            json!(format!("http://127.0.0.1:{}", rec.rpc_port)),
        );
    }
    value
}

/// The host:port (or domain) clients should address the proxy at.
fn public_base(config: &CloudConfig) -> String {
    if config.base_domain == "localhost" {
        config.bind_addr.clone()
    } else {
        config.base_domain.clone()
    }
}

/// Turn an arbitrary name into a URL-safe slug.
fn slugify(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    let mut prev_dash = false;
    for ch in input.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_normalizes() {
        assert_eq!(slugify("My Project"), "my-project");
        assert_eq!(slugify("  Hello!! World  "), "hello-world");
        assert_eq!(slugify("already-good"), "already-good");
        assert_eq!(slugify("!!!"), "");
    }

    #[tokio::test]
    async fn signup_and_auth_roundtrip() {
        let store = ControlPlaneStore::connect(":memory:").await.unwrap();
        let (tenant, key) = store.create_tenant("Acme", "a@acme.dev").await.unwrap();
        // The key authenticates back to the same tenant.
        let resolved = store.tenant_by_key(&key).await.unwrap().unwrap();
        assert_eq!(resolved.id, tenant.id);
        // A bad key resolves to nothing.
        assert!(store.tenant_by_key("rk_nope").await.unwrap().is_none());
        // Duplicate email is rejected.
        assert!(store.create_tenant("Acme2", "a@acme.dev").await.is_err());
    }

    /// Isolation canary (spec 2.8 / docs/threat-model.md): prove tenant A cannot
    /// read, enumerate, or reach tenant B's stagenet. This must stay green - it is
    /// the single test that backs the "zero cross-tenant access" SLO. It exercises
    /// the *real* `owned_stagenet` guard, not a reimplementation of it.
    #[tokio::test]
    async fn cross_tenant_isolation_is_enforced() {
        let store = Arc::new(ControlPlaneStore::connect(":memory:").await.unwrap());
        let config = Arc::new(CloudConfig::from_env());
        let state = AppState {
            store: Arc::clone(&store),
            orch: Arc::new(Orchestrator::new(Arc::clone(&config))),
            http: reqwest::Client::new(),
            config,
        };

        let (alice, _) = store.create_tenant("Alice", "alice@a.dev").await.unwrap();
        let (bob, _) = store.create_tenant("Bob", "bob@b.dev").await.unwrap();

        // Alice owns a stagenet.
        let rec = CloudStagenet {
            id: uuid::Uuid::new_v4(),
            tenant_id: alice.id,
            slug: "alice-proj".to_string(),
            name: "alice-proj".to_string(),
            status: "running".to_string(),
            rpc_port: 20000,
            ws_port: 20001,
            api_port: 20002,
            mainnet_rpc: "https://example.invalid".to_string(),
            pid: None,
            work_dir: "/tmp/alice-proj".to_string(),
            created_at: chrono::Utc::now(),
            last_active: None,
        };
        store.insert_stagenet(&rec).await.unwrap();

        // Alice can read her own stagenet through the guard.
        assert!(owned_stagenet(&state, &alice.id, "alice-proj")
            .await
            .is_ok());

        // Bob cannot read it - the guard returns NotFound (so Bob can't even
        // distinguish "exists but not yours" from "doesn't exist").
        assert!(matches!(
            owned_stagenet(&state, &bob.id, "alice-proj").await,
            Err(CloudError::NotFound(_))
        ));

        // Bob's listing never includes Alice's stagenet; Alice's does.
        assert_eq!(store.list_stagenets(&bob.id).await.unwrap().len(), 0);
        assert_eq!(store.list_stagenets(&alice.id).await.unwrap().len(), 1);
    }
}
