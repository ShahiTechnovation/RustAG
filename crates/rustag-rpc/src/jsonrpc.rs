//! Solana-compatible JSON-RPC surface, served over HTTP via axum.
//!
//! Implements the subset of the Solana JSON-RPC API needed for a wallet or
//! `@solana/web3.js` `Connection` to point at a stagenet and Just Work:
//! account reads, balances, blockhash, airdrops, send/simulate, and the status
//! queries clients use to confirm transactions.

use std::str::FromStr;

use axum::extract::State;
use axum::Json;
use base64::Engine;
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use solana_transaction::versioned::VersionedTransaction;

use crate::state::{AppState, MAX_DEMO_AIRDROP_LAMPORTS};
use crate::types::{encode_account_base64, with_context, API_VERSION, GENESIS_HASH};

/// A JSON-RPC error `(code, message)`.
type RpcResult = std::result::Result<Value, (i64, String)>;

const INVALID_PARAMS: i64 = -32602;
const METHOD_NOT_FOUND: i64 = -32601;
const SERVER_ERROR: i64 = -32000;

/// axum entry point: handles a single request or a batch.
pub async fn handle(State(state): State<AppState>, Json(body): Json<Value>) -> Json<Value> {
    let response = match body {
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(dispatch_one(&state, item).await);
            }
            Value::Array(out)
        }
        other => dispatch_one(&state, other).await,
    };
    Json(response)
}

async fn dispatch_one(state: &AppState, request: Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = request.get("params").cloned().unwrap_or(Value::Null);
    let empty = Vec::new();
    let params = params.as_array().unwrap_or(&empty);

    let result = route(state, method, params).await;
    match result {
        Ok(value) => json!({ "jsonrpc": "2.0", "id": id, "result": value }),
        Err((code, message)) => {
            json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
        }
    }
}

pub(crate) async fn route(state: &AppState, method: &str, params: &[Value]) -> RpcResult {
    match method {
        "getHealth" => Ok(json!("ok")),
        "getVersion" => Ok(json!({ "solana-core": API_VERSION, "feature-set": 0u32 })),
        "getGenesisHash" => Ok(json!(GENESIS_HASH)),
        "getIdentity" => Ok(json!({ "identity": GENESIS_HASH })),
        "getSlot" => Ok(json!(slot(state).await)),
        "getBlockHeight" => Ok(json!(slot(state).await)),
        "getEpochInfo" => get_epoch_info(state).await,
        "getLatestBlockhash" => get_latest_blockhash(state).await,
        "isBlockhashValid" => is_blockhash_valid(state).await,
        "getMinimumBalanceForRentExemption" => get_min_balance(state, params).await,
        "getBalance" => get_balance(state, params).await,
        "getAccountInfo" => get_account_info(state, params).await,
        "getMultipleAccounts" => get_multiple_accounts(state, params).await,
        "getProgramAccounts" => get_program_accounts(state, params).await,
        "getTokenAccountBalance" => get_token_account_balance(state, params).await,
        "requestAirdrop" => request_airdrop(state, params).await,
        "sendTransaction" => send_transaction(state, params).await,
        "simulateTransaction" => simulate_transaction(state, params).await,
        "getSignatureStatuses" => get_signature_statuses(state, params).await,
        "getTransaction" => get_transaction(state, params).await,
        "getFeeForMessage" => Ok(with_context(slot(state).await, json!(5000))),
        other => Err((METHOD_NOT_FOUND, format!("method not found: {other}"))),
    }
}

// --- helpers ----------------------------------------------------------------

async fn slot(state: &AppState) -> u64 {
    state.stagenet.read().await.current_slot()
}

/// A stored `err` is a JSON-encoded `TransactionError` (or `None` on success).
/// Parse it back into the structured object Solana clients expect; `null` means
/// the transaction succeeded.
pub(crate) fn err_value(err: &Option<String>) -> Value {
    match err {
        Some(s) => serde_json::from_str(s).unwrap_or(Value::String(s.clone())),
        None => Value::Null,
    }
}

fn param_str(
    params: &[Value],
    idx: usize,
    what: &str,
) -> std::result::Result<String, (i64, String)> {
    params
        .get(idx)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or((INVALID_PARAMS, format!("expected {what} at param {idx}")))
}

fn parse_pubkey(s: &str) -> std::result::Result<Pubkey, (i64, String)> {
    Pubkey::from_str(s).map_err(|_| (INVALID_PARAMS, format!("invalid pubkey: {s}")))
}

fn encoding_of(params: &[Value], idx: usize) -> Option<String> {
    params
        .get(idx)
        .and_then(|c| c.get("encoding"))
        .and_then(|e| e.as_str())
        .map(|s| s.to_string())
}

/// Decode a transaction blob. Honors the `encoding` config. When unspecified we
/// follow the Solana default (base58) and only fall back to base64 if base58
/// fails - base64 blobs contain `+`/`/`/`=`, which are not valid base58, so this
/// avoids the misparse a base64-first order could cause.
fn decode_transaction(
    blob: &str,
    encoding: Option<&str>,
) -> std::result::Result<VersionedTransaction, (i64, String)> {
    let bytes = match encoding {
        Some("base64") => base64::engine::general_purpose::STANDARD
            .decode(blob)
            .map_err(|e| (INVALID_PARAMS, format!("base64 decode: {e}")))?,
        Some("base58") => bs58::decode(blob)
            .into_vec()
            .map_err(|e| (INVALID_PARAMS, format!("base58 decode: {e}")))?,
        _ => bs58::decode(blob)
            .into_vec()
            .or_else(|_| base64::engine::general_purpose::STANDARD.decode(blob))
            .map_err(|e| (INVALID_PARAMS, format!("decode: {e}")))?,
    };
    bincode::deserialize::<VersionedTransaction>(&bytes)
        .map_err(|e| (INVALID_PARAMS, format!("deserialize transaction: {e}")))
}

// --- method implementations -------------------------------------------------

async fn get_epoch_info(state: &AppState) -> RpcResult {
    let slot = slot(state).await;
    Ok(json!({
        "absoluteSlot": slot,
        "blockHeight": slot,
        "epoch": 0,
        "slotIndex": slot,
        "slotsInEpoch": 432_000,
        "transactionCount": slot,
    }))
}

async fn get_latest_blockhash(state: &AppState) -> RpcResult {
    let sn = state.stagenet.read().await;
    let slot = sn.current_slot();
    Ok(with_context(
        slot,
        json!({
            "blockhash": sn.latest_blockhash().to_string(),
            "lastValidBlockHeight": slot + 150,
        }),
    ))
}

async fn is_blockhash_valid(state: &AppState) -> RpcResult {
    // The stagenet blockhash never expires, so any well-formed request is valid.
    Ok(with_context(slot(state).await, json!(true)))
}

async fn get_min_balance(state: &AppState, params: &[Value]) -> RpcResult {
    let len = params.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let sn = state.stagenet.read().await;
    Ok(json!(sn.minimum_balance_for_rent_exemption(len)))
}

async fn get_balance(state: &AppState, params: &[Value]) -> RpcResult {
    let pubkey = parse_pubkey(&param_str(params, 0, "pubkey")?)?;
    let mut sn = state.stagenet.write().await;
    let balance = sn
        .get_balance(&pubkey)
        .await
        .map_err(|e| (SERVER_ERROR, e.to_string()))?;
    Ok(with_context(sn.current_slot(), json!(balance)))
}

async fn get_account_info(state: &AppState, params: &[Value]) -> RpcResult {
    let pubkey = parse_pubkey(&param_str(params, 0, "pubkey")?)?;
    let mut sn = state.stagenet.write().await;
    let account = sn
        .get_account(&pubkey)
        .await
        .map_err(|e| (SERVER_ERROR, e.to_string()))?;
    let value = account
        .map(|e| encode_account_base64(&e))
        .unwrap_or(Value::Null);
    Ok(with_context(sn.current_slot(), value))
}

async fn get_multiple_accounts(state: &AppState, params: &[Value]) -> RpcResult {
    let keys = params
        .first()
        .and_then(|v| v.as_array())
        .ok_or((INVALID_PARAMS, "expected pubkey array".to_string()))?;
    let mut sn = state.stagenet.write().await;
    let mut out = Vec::with_capacity(keys.len());
    for key in keys {
        let s = key
            .as_str()
            .ok_or((INVALID_PARAMS, "pubkey must be a string".to_string()))?;
        let pubkey = parse_pubkey(s)?;
        let account = sn
            .get_account(&pubkey)
            .await
            .map_err(|e| (SERVER_ERROR, e.to_string()))?;
        out.push(
            account
                .map(|e| encode_account_base64(&e))
                .unwrap_or(Value::Null),
        );
    }
    let slot = sn.current_slot();
    Ok(with_context(slot, Value::Array(out)))
}

async fn get_program_accounts(state: &AppState, params: &[Value]) -> RpcResult {
    let program = parse_pubkey(&param_str(params, 0, "program id")?)?;
    let sn = state.stagenet.read().await;
    let store = sn.store();
    let id = sn.id();
    drop(sn);
    let accounts = store
        .get_program_accounts(&id, &program, 10_000)
        .await
        .map_err(|e| (SERVER_ERROR, e.to_string()))?;

    // Apply `dataSize` / `memcmp` filters from params[1] if present.
    let empty = Vec::new();
    let filters = params
        .get(1)
        .and_then(|c| c.get("filters"))
        .and_then(|f| f.as_array())
        .unwrap_or(&empty);

    let out: Vec<Value> = accounts
        .iter()
        .filter(|e| passes_filters(&e.data, filters))
        .map(|e| json!({ "pubkey": e.pubkey.to_string(), "account": encode_account_base64(e) }))
        .collect();
    Ok(Value::Array(out))
}

/// Apply Solana `getProgramAccounts` filters (`dataSize`, `memcmp`) to account data.
fn passes_filters(data: &[u8], filters: &[Value]) -> bool {
    filters.iter().all(|filter| {
        if let Some(size) = filter.get("dataSize").and_then(|v| v.as_u64()) {
            return data.len() as u64 == size;
        }
        if let Some(memcmp) = filter.get("memcmp") {
            let offset = memcmp.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let Some(bytes_str) = memcmp.get("bytes").and_then(|v| v.as_str()) else {
                return true;
            };
            let encoding = memcmp.get("encoding").and_then(|v| v.as_str());
            let needle = match encoding {
                Some("base64") => base64::engine::general_purpose::STANDARD
                    .decode(bytes_str)
                    .ok(),
                _ => bs58::decode(bytes_str).into_vec().ok(),
            };
            let Some(needle) = needle else { return false };
            return data.len() >= offset + needle.len()
                && data[offset..offset + needle.len()] == needle[..];
        }
        true // unknown filter shapes are ignored
    })
}

async fn get_token_account_balance(state: &AppState, params: &[Value]) -> RpcResult {
    let pubkey = parse_pubkey(&param_str(params, 0, "token account")?)?;
    let mut sn = state.stagenet.write().await;
    let account = sn
        .get_account(&pubkey)
        .await
        .map_err(|e| (SERVER_ERROR, e.to_string()))?
        .ok_or((SERVER_ERROR, "token account not found".to_string()))?;
    if account.data.len() < 72 {
        return Err((SERVER_ERROR, "not a token account".to_string()));
    }
    let amount = u64::from_le_bytes(account.data[64..72].try_into().unwrap());
    let slot = sn.current_slot();
    Ok(with_context(
        slot,
        json!({
            "amount": amount.to_string(),
            "decimals": 0,
            "uiAmount": amount as f64,
            "uiAmountString": amount.to_string(),
        }),
    ))
}

async fn request_airdrop(state: &AppState, params: &[Value]) -> RpcResult {
    let pubkey = parse_pubkey(&param_str(params, 0, "pubkey")?)?;
    let lamports = params
        .get(1)
        .and_then(|v| v.as_u64())
        .ok_or((INVALID_PARAMS, "expected lamports at param 1".to_string()))?;
    // Same cap as the REST path so the standard Solana `requestAirdrop` RPC is
    // not a way around the public-demo airdrop limit.
    if state.demo_mode && lamports > MAX_DEMO_AIRDROP_LAMPORTS {
        return Err((
            INVALID_PARAMS,
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
        .map_err(|e| (SERVER_ERROR, e.to_string()))?;
    Ok(json!(signature.to_string()))
}

async fn send_transaction(state: &AppState, params: &[Value]) -> RpcResult {
    let blob = param_str(params, 0, "transaction")?;
    let tx = decode_transaction(&blob, encoding_of(params, 1).as_deref())?;
    let mut sn = state.stagenet.write().await;
    let outcome = sn
        .send_transaction(tx)
        .await
        .map_err(|e| (SERVER_ERROR, e.to_string()))?;
    // Return the signature regardless of runtime success; clients confirm via
    // getSignatureStatuses (which surfaces any error).
    Ok(json!(outcome.signature_string()))
}

async fn simulate_transaction(state: &AppState, params: &[Value]) -> RpcResult {
    let blob = param_str(params, 0, "transaction")?;
    let tx = decode_transaction(&blob, encoding_of(params, 1).as_deref())?;
    let mut sn = state.stagenet.write().await;
    let outcome = sn
        .simulate_transaction(tx)
        .await
        .map_err(|e| (SERVER_ERROR, e.to_string()))?;
    let slot = sn.current_slot();
    Ok(with_context(
        slot,
        json!({
            "err": err_value(&outcome.err),
            "logs": outcome.logs,
            "accounts": Value::Null,
            "unitsConsumed": outcome.compute_units,
            "returnData": Value::Null,
        }),
    ))
}

async fn get_signature_statuses(state: &AppState, params: &[Value]) -> RpcResult {
    let sigs = params
        .first()
        .and_then(|v| v.as_array())
        .ok_or((INVALID_PARAMS, "expected signature array".to_string()))?;
    let sn = state.stagenet.read().await;
    let store = sn.store();
    let id = sn.id();
    let slot = sn.current_slot();
    drop(sn);

    let mut out = Vec::with_capacity(sigs.len());
    for sig in sigs {
        let s = sig
            .as_str()
            .ok_or((INVALID_PARAMS, "signature must be a string".to_string()))?;
        let record = store
            .get_transaction(&id, s)
            .await
            .map_err(|e| (SERVER_ERROR, e.to_string()))?;
        match record {
            Some(rec) => out.push(json!({
                "slot": rec.slot,
                "confirmations": Value::Null,
                "err": err_value(&rec.err),
                "confirmationStatus": "finalized",
            })),
            None => out.push(Value::Null),
        }
    }
    Ok(with_context(slot, Value::Array(out)))
}

async fn get_transaction(state: &AppState, params: &[Value]) -> RpcResult {
    let signature = param_str(params, 0, "signature")?;
    let sn = state.stagenet.read().await;
    let store = sn.store();
    let id = sn.id();
    drop(sn);
    let record = store
        .get_transaction(&id, &signature)
        .await
        .map_err(|e| (SERVER_ERROR, e.to_string()))?;
    Ok(match record {
        Some(rec) => json!({
            "slot": rec.slot,
            "blockTime": Value::Null,
            "meta": {
                "err": err_value(&rec.err),
                "fee": rec.fee,
                "computeUnitsConsumed": rec.compute_units,
                "logMessages": rec.logs,
            },
            "transaction": { "signatures": [rec.signature] },
        }),
        None => Value::Null,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use rustag_core::Stagenet;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_signer::Signer;
    use solana_system_interface::instruction::transfer;
    use solana_transaction::versioned::VersionedTransaction;
    use solana_transaction::Transaction;
    use tokio::sync::RwLock;

    async fn state() -> AppState {
        let sn = Stagenet::local("rpc-test").await.unwrap();
        AppState::new(Arc::new(RwLock::new(sn)))
    }

    #[tokio::test]
    async fn health_and_version() {
        let st = state().await;
        assert_eq!(route(&st, "getHealth", &[]).await.unwrap(), json!("ok"));
        assert!(route(&st, "getVersion", &[]).await.is_ok());
        assert!(route(&st, "nonexistent", &[]).await.is_err());
    }

    #[tokio::test]
    async fn airdrop_and_balance() {
        let st = state().await;
        let kp = Keypair::new();
        let pk = kp.pubkey().to_string();
        let sig = route(&st, "requestAirdrop", &[json!(pk), json!(1_000_000_000u64)])
            .await
            .unwrap();
        assert!(sig.is_string());
        let bal = route(&st, "getBalance", &[json!(pk)]).await.unwrap();
        assert_eq!(bal["value"], json!(1_000_000_000u64));
    }

    #[tokio::test]
    async fn demo_mode_caps_airdrop() {
        let mut st = state().await;
        st.demo_mode = true;
        let pk = Keypair::new().pubkey().to_string();
        // Over the cap: rejected.
        assert!(route(
            &st,
            "requestAirdrop",
            &[json!(pk), json!(MAX_DEMO_AIRDROP_LAMPORTS + 1)]
        )
        .await
        .is_err());
        // At the cap: allowed.
        assert!(route(
            &st,
            "requestAirdrop",
            &[json!(pk), json!(MAX_DEMO_AIRDROP_LAMPORTS)]
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn send_transaction_roundtrip() {
        let st = state().await;
        let payer = Keypair::new();
        route(
            &st,
            "requestAirdrop",
            &[json!(payer.pubkey().to_string()), json!(2_000_000_000u64)],
        )
        .await
        .unwrap();

        let receiver = Keypair::new();
        let blockhash = st.stagenet.read().await.latest_blockhash();
        let ix = transfer(&payer.pubkey(), &receiver.pubkey(), 1_000_000_000);
        let msg = Message::new(&[ix], Some(&payer.pubkey()));
        let tx: VersionedTransaction = Transaction::new(&[&payer], msg, blockhash).into();
        let bytes = bincode::serialize(&tx).unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);

        let sig = route(
            &st,
            "sendTransaction",
            &[json!(b64), json!({"encoding": "base64"})],
        )
        .await
        .unwrap();
        assert!(sig.is_string());

        let statuses = route(&st, "getSignatureStatuses", &[json!([sig])])
            .await
            .unwrap();
        assert!(statuses["value"][0].is_object());

        let bal = route(&st, "getBalance", &[json!(receiver.pubkey().to_string())])
            .await
            .unwrap();
        assert_eq!(bal["value"], json!(1_000_000_000u64));
    }

    #[tokio::test]
    async fn failed_tx_reports_structured_err() {
        let st = state().await;
        let payer = Keypair::new();
        route(
            &st,
            "requestAirdrop",
            &[json!(payer.pubkey().to_string()), json!(1_000_000_000u64)],
        )
        .await
        .unwrap();

        // Spend more than the balance -> the tx fails at runtime.
        let receiver = Keypair::new();
        let blockhash = st.stagenet.read().await.latest_blockhash();
        let ix = transfer(&payer.pubkey(), &receiver.pubkey(), 5_000_000_000);
        let msg = Message::new(&[ix], Some(&payer.pubkey()));
        let tx: VersionedTransaction = Transaction::new(&[&payer], msg, blockhash).into();
        let b64 =
            base64::engine::general_purpose::STANDARD.encode(bincode::serialize(&tx).unwrap());

        let sig = route(
            &st,
            "sendTransaction",
            &[json!(b64), json!({"encoding": "base64"})],
        )
        .await
        .unwrap();

        let statuses = route(&st, "getSignatureStatuses", &[json!([sig])])
            .await
            .unwrap();
        let err = &statuses["value"][0]["err"];
        // Structured (object or string variant), never null for a failed tx, and
        // never the Rust Debug rendering.
        assert!(!err.is_null(), "failed tx must report an err");
        assert!(err.is_object() || err.is_string());
    }
}
