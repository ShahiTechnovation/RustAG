//! REST API consumed by the Next.js dashboard.

use std::str::FromStr;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use solana_pubkey::Pubkey;

use rustag_core::AccountOverride;

use crate::state::AppState;
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
        .with_state(state)
}

type ApiResult = std::result::Result<Json<Value>, (StatusCode, String)>;

fn server_err<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
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
        "mainnetRpc": config.mainnet_rpc,
        "accounts": accounts,
        "transactions": transactions,
        "dirtyAccounts": dirty,
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
    let lamports = (body.sol * 1_000_000_000.0) as u64;
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
