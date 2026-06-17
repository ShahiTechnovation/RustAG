//! Minimal Solana-style WebSocket pub/sub.
//!
//! Phase 1 implements `accountSubscribe` and `signatureSubscribe` via polling
//! (the spec calls for an eventual Geyser/gRPC upgrade in Phase 2).
//! `signatureSubscribe` is what `@solana/web3.js` uses to confirm transactions,
//! so it is required for `sendAndConfirmTransaction` to work. Unary JSON-RPC
//! requests received over the socket are delegated to the same dispatcher as the
//! HTTP endpoint, so a client can use one connection for everything.

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use solana_pubkey::Pubkey;

use crate::jsonrpc::err_value;
use crate::state::AppState;
use crate::types::encode_account_base64;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// An active subscription on a connection.
enum Subscription {
    /// Push the account whenever its (lamports, len) fingerprint changes.
    Account {
        pubkey: Pubkey,
        last: Option<String>,
    },
    /// Fire once when the signature's transaction is found, then auto-cancel.
    Signature { signature: String },
}

/// axum handler: upgrade the connection and run the pub/sub loop.
pub async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| run(socket, state))
}

async fn run(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut subs: HashMap<u64, Subscription> = HashMap::new();
    let mut next_id: u64 = 1;
    let mut ticker = tokio::time::interval(POLL_INTERVAL);

    loop {
        tokio::select! {
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if let Some(reply) = handle_text(&state, &text, &mut subs, &mut next_id).await {
                            if sender.send(Message::Text(reply.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}      // ignore binary/ping/pong
                    Some(Err(_)) => break,
                }
            }
            _ = ticker.tick() => {
                let notes = poll_subscriptions(&state, &mut subs).await;
                for note in notes {
                    if sender.send(Message::Text(note.into())).await.is_err() {
                        return;
                    }
                }
            }
        }
    }
}

async fn handle_text(
    state: &AppState,
    text: &str,
    subs: &mut HashMap<u64, Subscription>,
    next_id: &mut u64,
) -> Option<String> {
    let req: Value = serde_json::from_str(text).ok()?;
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params: Vec<Value> = req
        .get("params")
        .and_then(|p| p.as_array().cloned())
        .unwrap_or_default();

    let ok =
        |result: Value| Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string());

    match method {
        "accountSubscribe" => {
            let Some(pubkey) = params
                .first()
                .and_then(|v| v.as_str())
                .and_then(|s| Pubkey::from_str(s).ok())
            else {
                return Some(invalid(&id, "invalid pubkey"));
            };
            let sub_id = take_id(next_id);
            subs.insert(sub_id, Subscription::Account { pubkey, last: None });
            ok(json!(sub_id))
        }
        "signatureSubscribe" => {
            let Some(signature) = params.first().and_then(|v| v.as_str()) else {
                return Some(invalid(&id, "invalid signature"));
            };
            let sub_id = take_id(next_id);
            subs.insert(
                sub_id,
                Subscription::Signature {
                    signature: signature.to_string(),
                },
            );
            ok(json!(sub_id))
        }
        "slotSubscribe" => {
            // Accepted for compatibility; this stagenet emits no slot stream.
            ok(json!(take_id(next_id)))
        }
        "accountUnsubscribe" | "signatureUnsubscribe" | "slotUnsubscribe" => {
            if let Some(sub_id) = params.first().and_then(|v| v.as_u64()) {
                subs.remove(&sub_id);
            }
            ok(json!(true))
        }
        other => {
            let result = crate::jsonrpc::route(state, other, &params).await;
            Some(match result {
                Ok(value) => json!({ "jsonrpc": "2.0", "id": id, "result": value }).to_string(),
                Err((code, message)) => {
                    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
                        .to_string()
                }
            })
        }
    }
}

fn take_id(next_id: &mut u64) -> u64 {
    let id = *next_id;
    *next_id += 1;
    id
}

fn invalid(id: &Value, message: &str) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": message } })
        .to_string()
}

async fn poll_subscriptions(
    state: &AppState,
    subs: &mut HashMap<u64, Subscription>,
) -> Vec<String> {
    if subs.is_empty() {
        return Vec::new();
    }
    let mut sn = state.stagenet.write().await;
    let slot = sn.current_slot();
    let store = sn.store();
    let id = sn.id();

    let mut notes = Vec::new();
    let mut fired_signatures = Vec::new();

    for (sub_id, sub) in subs.iter_mut() {
        match sub {
            Subscription::Account { pubkey, last } => {
                let Ok(account) = sn.get_account(pubkey).await else {
                    continue;
                };
                let value = account
                    .as_ref()
                    .map(encode_account_base64)
                    .unwrap_or(Value::Null);
                let fingerprint = account
                    .as_ref()
                    .map(|e| format!("{}:{}", e.lamports, e.data.len()));
                if *last != fingerprint {
                    *last = fingerprint;
                    notes.push(
                        json!({
                            "jsonrpc": "2.0",
                            "method": "accountNotification",
                            "params": {
                                "result": { "context": { "slot": slot }, "value": value },
                                "subscription": sub_id,
                            },
                        })
                        .to_string(),
                    );
                }
            }
            Subscription::Signature { signature } => {
                if let Ok(Some(rec)) = store.get_transaction(&id, signature).await {
                    notes.push(
                        json!({
                            "jsonrpc": "2.0",
                            "method": "signatureNotification",
                            "params": {
                                "result": {
                                    "context": { "slot": slot },
                                    "value": { "err": err_value(&rec.err) },
                                },
                                "subscription": sub_id,
                            },
                        })
                        .to_string(),
                    );
                    fired_signatures.push(*sub_id);
                }
            }
        }
    }

    // Signature subscriptions are one-shot: drop them once they have fired.
    for sub_id in fired_signatures {
        subs.remove(&sub_id);
    }
    notes
}
