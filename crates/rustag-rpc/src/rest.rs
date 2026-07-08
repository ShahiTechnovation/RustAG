//! REST API consumed by the Next.js dashboard.

use std::str::FromStr;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use solana_transaction::versioned::VersionedTransaction;
use uuid::Uuid;

use rustag_core::AccountOverride;

use crate::state::{AppState, MAX_DEMO_AIRDROP_LAMPORTS};
use crate::types::encode_account_rich;

/// Build the REST router (mounted under `/api`).
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/stagenet", get(stagenet_info))
        .route("/api/accounts", get(list_accounts))
        .route("/api/accounts/{pubkey}", get(get_account))
        .route("/api/transactions", get(list_transactions))
        .route("/api/airdrop", post(airdrop))
        .route("/api/override", post(override_account))
        .route("/api/preload", post(preload))
        // --- Phase 2: scheduler, analytics, simulation ---
        .route("/api/schedules", get(list_schedules).post(create_schedule))
        .route("/api/schedules/{id}", delete(delete_schedule))
        .route("/api/schedules/{id}/toggle", post(toggle_schedule))
        .route("/api/metrics", get(get_metrics))
        .route("/api/simulate", post(simulate))
        .with_state(state)
}

type ApiResult = std::result::Result<Json<Value>, (StatusCode, String)>;

fn server_err<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

/// 403 for a route disabled while the server runs in public-demo mode. Keeps the
/// shared stagenet un-griefable and the upstream RPC key un-drainable without
/// blocking the read + airdrop + simulate experience a reviewer actually wants.
fn demo_forbidden(action: &str) -> (StatusCode, String) {
    (
        StatusCode::FORBIDDEN,
        format!("{action} is disabled on the public demo (reads, capped airdrops, and simulate stay available)"),
    )
}

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

async fn stagenet_info(State(state): State<AppState>) -> ApiResult {
    let sn = state.stagenet.read().await;
    let store = sn.store();
    let id = sn.id();
    let config = sn.config().clone();
    let slot = sn.current_slot();
    let dirty = sn.dirty_count();
    drop(sn);

    let accounts = store.count_accounts(&id).await.map_err(server_err)?;
    let transactions = store.count_transactions(&id).await.map_err(server_err)?;

    Ok(Json(json!({
        "id": id.to_string(),
        "name": config.name,
        "network": config.network,
        "slot": slot,
        "rpcUrl": config.rpc_url(),
        "wsUrl": config.ws_url(),
        "mirrorEnabled": config.mirror_enabled,
        // Redact the upstream RPC credential: `mainnet_rpc` carries the paid
        // Helius/Alchemy `?api-key=` (or a path-embedded key) and this response
        // is served to every browser. Never surface it on a read path.
        "mainnetRpc": rustag_core::redact_url(&config.mainnet_rpc),
        "accounts": accounts,
        "transactions": transactions,
        "dirtyAccounts": dirty,
        // Let the dashboard adapt (hide override/preload, show a "live demo"
        // badge) when the backend is a public, capped-interactive instance.
        "demoMode": state.demo_mode,
    })))
}

#[derive(Deserialize)]
struct Pagination {
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list_accounts(State(state): State<AppState>, Query(p): Query<Pagination>) -> ApiResult {
    let sn = state.stagenet.read().await;
    let store = sn.store();
    let id = sn.id();
    drop(sn);
    let limit = p.limit.unwrap_or(100).clamp(1, 1000);
    let offset = p.offset.unwrap_or(0).max(0);
    let accounts = store
        .list_accounts(&id, limit, offset)
        .await
        .map_err(server_err)?;
    let out: Vec<Value> = accounts.iter().map(encode_account_rich).collect();
    Ok(Json(json!({ "accounts": out })))
}

async fn get_account(State(state): State<AppState>, Path(pubkey): Path<String>) -> ApiResult {
    let pubkey = Pubkey::from_str(&pubkey)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid pubkey".to_string()))?;

    // On the public demo, serve only accounts that are already mirrored (the
    // preloaded set, oracle refreshes, and demo activity). Lazily mirroring an
    // arbitrary pubkey would fan out to a live mainnet fetch, letting a crawler
    // drain the upstream RPC quota and grow the table without bound.
    if state.demo_mode {
        let sn = state.stagenet.read().await;
        let (store, id) = (sn.store(), sn.id());
        drop(sn);
        return match store.get_account(&id, &pubkey).await.map_err(server_err)? {
            Some(e) => Ok(Json(encode_account_rich(&e))),
            None => Err((
                StatusCode::NOT_FOUND,
                "account not mirrored (arbitrary mainnet fetches are disabled on the public demo)"
                    .to_string(),
            )),
        };
    }

    let mut sn = state.stagenet.write().await;
    let account = sn.get_account(&pubkey).await.map_err(server_err)?;
    match account {
        Some(e) => Ok(Json(encode_account_rich(&e))),
        None => Err((StatusCode::NOT_FOUND, "account not found".to_string())),
    }
}

#[derive(Deserialize)]
struct TxQuery {
    limit: Option<i64>,
}

async fn list_transactions(State(state): State<AppState>, Query(q): Query<TxQuery>) -> ApiResult {
    let sn = state.stagenet.read().await;
    let store = sn.store();
    let id = sn.id();
    drop(sn);
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    let txs = store
        .list_transactions(&id, limit)
        .await
        .map_err(server_err)?;
    let out: Vec<Value> = txs
        .iter()
        .map(|t| {
            json!({
                "signature": t.signature,
                "slot": t.slot,
                "success": t.success,
                "fee": t.fee,
                "computeUnits": t.compute_units,
                "programs": t.programs,
                "logs": t.logs,
                "err": t.err,
                "createdAt": t.created_at.to_rfc3339(),
            })
        })
        .collect();
    Ok(Json(json!({ "transactions": out })))
}

#[derive(Deserialize)]
struct AirdropBody {
    pubkey: String,
    /// Amount in SOL.
    sol: f64,
}

async fn airdrop(State(state): State<AppState>, Json(body): Json<AirdropBody>) -> ApiResult {
    let pubkey = Pubkey::from_str(&body.pubkey)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid pubkey".to_string()))?;
    // `sol as u64` saturates (negative/NaN -> 0, huge -> u64::MAX), so the cap
    // below also fences off nonsense amounts on the public demo.
    let lamports = (body.sol * 1_000_000_000.0) as u64;
    if state.demo_mode && lamports > MAX_DEMO_AIRDROP_LAMPORTS {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "airdrop is capped at {} SOL on the public demo",
                MAX_DEMO_AIRDROP_LAMPORTS / 1_000_000_000
            ),
        ));
    }
    let mut sn = state.stagenet.write().await;
    let signature = sn
        .airdrop_with_record(&pubkey, lamports)
        .await
        .map_err(server_err)?;
    Ok(Json(
        json!({ "signature": signature.to_string(), "lamports": lamports }),
    ))
}

#[derive(Deserialize)]
struct OverrideBody {
    pubkey: String,
    lamports: Option<u64>,
    #[serde(rename = "tokenBalance")]
    token_balance: Option<u64>,
}

async fn override_account(
    State(state): State<AppState>,
    Json(body): Json<OverrideBody>,
) -> ApiResult {
    if state.demo_mode {
        return Err(demo_forbidden("override"));
    }
    let pubkey = Pubkey::from_str(&body.pubkey)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid pubkey".to_string()))?;
    let mut sn = state.stagenet.write().await;
    if let Some(amount) = body.token_balance {
        sn.override_token_balance(&pubkey, amount)
            .await
            .map_err(server_err)?;
    } else {
        sn.override_account(
            &pubkey,
            AccountOverride {
                lamports: body.lamports,
                ..Default::default()
            },
        )
        .await
        .map_err(server_err)?;
    }
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct PreloadBody {
    programs: Vec<String>,
}

async fn preload(State(state): State<AppState>, Json(body): Json<PreloadBody>) -> ApiResult {
    // Preload fans out to live mainnet `getMultipleAccounts` calls; on the public
    // demo it would let anyone drain the upstream RPC key. The demo already
    // preloads Pyth/Raydium/token at boot, so reviewers never need this route.
    if state.demo_mode {
        return Err(demo_forbidden("preload"));
    }
    let mut entries = Vec::new();
    let mut unknown = Vec::new();
    for name in &body.programs {
        match rustag_core::registry::resolve(name) {
            Some(mut e) => entries.append(&mut e),
            None => unknown.push(name.clone()),
        }
    }
    let mut sn = state.stagenet.write().await;
    let loaded = sn.preload(&entries).await.map_err(server_err)?;
    Ok(Json(json!({ "loaded": loaded, "unknown": unknown })))
}

// --- Phase 2: Activity Scheduler -------------------------------------------

fn schedule_json(s: &rustag_core::ScheduleRecord) -> Value {
    let mut action: Value = serde_json::from_str(&s.action_json).unwrap_or(Value::Null);
    redact_secrets(&mut action);
    json!({
        "id": s.id.to_string(),
        "name": s.name,
        "schedule": s.schedule,
        "action": action,
        "enabled": s.enabled,
        "runCount": s.run_count,
        "lastRun": s.last_run.map(|t| t.to_rfc3339()),
        "lastStatus": s.last_status,
        "lastSignature": s.last_signature,
        "createdAt": s.created_at.to_rfc3339(),
    })
}

/// Mask sensitive fields (e.g. a `Transfer` action's `secret_key`) before a
/// schedule action is serialized into an API response. The full value is kept
/// in storage for execution; only the read path is redacted.
fn redact_secrets(action: &mut Value) {
    if let Some(obj) = action.as_object_mut() {
        if let Some(secret) = obj.get_mut("secret_key") {
            *secret = Value::String("***redacted***".to_string());
        }
    }
}

async fn list_schedules(State(state): State<AppState>) -> ApiResult {
    let sn = state.stagenet.read().await;
    let store = sn.store();
    let id = sn.id();
    drop(sn);
    let schedules = store.list_schedules(&id, false).await.map_err(server_err)?;
    let out: Vec<Value> = schedules.iter().map(schedule_json).collect();
    Ok(Json(json!({ "schedules": out })))
}

#[derive(Deserialize)]
struct CreateScheduleBody {
    name: String,
    schedule: String,
    action: rustag_scheduler::Action,
}

async fn create_schedule(
    State(state): State<AppState>,
    Json(body): Json<CreateScheduleBody>,
) -> ApiResult {
    if state.demo_mode {
        return Err(demo_forbidden("creating schedules"));
    }
    let sn = state.stagenet.read().await;
    let store = sn.store();
    let id = sn.id();
    drop(sn);
    let rec =
        rustag_scheduler::register_activity(&store, id, &body.name, &body.schedule, &body.action)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(schedule_json(&rec)))
}

async fn delete_schedule(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult {
    if state.demo_mode {
        return Err(demo_forbidden("deleting schedules"));
    }
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid schedule id".to_string()))?;
    let sn = state.stagenet.read().await;
    let store = sn.store();
    drop(sn);
    let removed = store.delete_schedule(&uuid).await.map_err(server_err)?;
    Ok(Json(json!({ "ok": removed })))
}

#[derive(Deserialize)]
struct ToggleBody {
    enabled: bool,
}

async fn toggle_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ToggleBody>,
) -> ApiResult {
    if state.demo_mode {
        return Err(demo_forbidden("toggling schedules"));
    }
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid schedule id".to_string()))?;
    let sn = state.stagenet.read().await;
    let store = sn.store();
    drop(sn);
    store
        .set_schedule_enabled(&uuid, body.enabled)
        .await
        .map_err(server_err)?;
    Ok(Json(json!({ "ok": true, "enabled": body.enabled })))
}

// --- Phase 2: Analytics ----------------------------------------------------

#[derive(Deserialize)]
struct MetricsQuery {
    series: Option<String>,
    limit: Option<i64>,
}

async fn get_metrics(State(state): State<AppState>, Query(q): Query<MetricsQuery>) -> ApiResult {
    let sn = state.stagenet.read().await;
    let store = sn.store();
    let id = sn.id();
    drop(sn);
    let limit = q.limit.unwrap_or(500).clamp(1, 10_000);
    let series_list: Vec<&str> = match q.series.as_deref() {
        Some(s) => vec![s],
        None => rustag_core::metrics::ALL_SERIES.to_vec(),
    };
    let mut out = serde_json::Map::new();
    for series in series_list {
        let points = store
            .query_metrics(&id, series, limit)
            .await
            .map_err(server_err)?;
        let arr: Vec<Value> = points
            .iter()
            .map(|p| json!({ "t": p.recorded_at.to_rfc3339(), "v": p.value }))
            .collect();
        out.insert(series.to_string(), Value::Array(arr));
    }
    Ok(Json(json!({ "metrics": Value::Object(out) })))
}

// --- Phase 2: Simulation ---------------------------------------------------

#[derive(Deserialize)]
struct SimulateBody {
    label: Option<String>,
    /// Encoded, signed transactions to replay against a fork of the stagenet.
    transactions: Vec<String>,
    /// `base64` (default) or `base58`.
    encoding: Option<String>,
}

/// Upper bound on transactions per simulation request (keeps the under-lock fork
/// snapshot and the overall replay bounded).
const MAX_SIMULATE_TXS: usize = 5_000;

async fn simulate(State(state): State<AppState>, Json(body): Json<SimulateBody>) -> ApiResult {
    let label = body.label.unwrap_or_else(|| "scenario".to_string());
    if body.transactions.len() > MAX_SIMULATE_TXS {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("too many transactions (max {MAX_SIMULATE_TXS})"),
        ));
    }
    let enc = body.encoding.as_deref();
    let mut txs = Vec::with_capacity(body.transactions.len());
    for blob in &body.transactions {
        txs.push(decode_tx(blob, enc).map_err(|m| (StatusCode::BAD_REQUEST, m))?);
    }

    // Hold the read lock only long enough to snapshot the base into an isolated
    // fork; run the (potentially long) replay against the owned fork with no lock
    // held, so a large batch can't starve writers on the live stagenet.
    let mut fork = {
        let sn = state.stagenet.read().await;
        sn.fork(&format!("sim-{label}")).await.map_err(server_err)?
    };
    let report = rustag_sim::replay(&mut fork, label, txs)
        .await
        .map_err(server_err)?;
    Ok(Json(serde_json::to_value(report).map_err(server_err)?))
}

/// Decode a base64/base58 transaction blob into a [`VersionedTransaction`].
fn decode_tx(
    blob: &str,
    encoding: Option<&str>,
) -> std::result::Result<VersionedTransaction, String> {
    let bytes = match encoding {
        Some("base64") => base64::engine::general_purpose::STANDARD
            .decode(blob)
            .map_err(|e| format!("base64 decode: {e}"))?,
        Some("base58") => bs58::decode(blob)
            .into_vec()
            .map_err(|e| format!("base58 decode: {e}"))?,
        _ => base64::engine::general_purpose::STANDARD
            .decode(blob)
            .or_else(|_| bs58::decode(blob).into_vec())
            .map_err(|e| format!("decode: {e}"))?,
    };
    bincode::deserialize::<VersionedTransaction>(&bytes).map_err(|e| format!("deserialize: {e}"))
}
