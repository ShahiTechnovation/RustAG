//! Real-time mainnet account push via the standard `accountSubscribe` WebSocket.
//!
//! Phase 1 keeps CLEAN oracle accounts fresh by polling mainnet every 30s. This
//! module is the Phase 2 *push* path: it opens a WebSocket to a mainnet RPC,
//! `accountSubscribe`s to a set of pubkeys, and forwards every
//! `accountNotification` as a [`RemoteAccount`] over an `mpsc` channel — the
//! moment mainnet changes, the stagenet sees it (sub-second, not 30s).
//!
//! It speaks the *standard* Solana pub/sub protocol, which Geyser/Yellowstone-
//! backed providers (Helius, Triton) serve, so one implementation points at any
//! endpoint. A native Yellowstone gRPC source is a drop-in alternative: it would
//! satisfy the exact same contract — produce [`RemoteAccount`]s into an
//! `mpsc::Sender` — and `rustag_core::spawn_realtime_apply` consumes either.
//!
//! Enabled by the `realtime` cargo feature.

use std::collections::HashMap;
use std::str::FromStr;

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::account::RemoteAccount;
use crate::error::MirrorError;

/// A real-time account subscriber over the `accountSubscribe` WebSocket API.
pub struct RealtimeMirror;

impl RealtimeMirror {
    /// Connect to `ws_url`, subscribe to every pubkey in `pubkeys`, and forward
    /// decoded account updates to `updates` until the socket closes or the
    /// receiver is dropped. Returns `Err` only on a connection/transport failure
    /// so the caller can reconnect.
    pub async fn run(
        ws_url: &str,
        pubkeys: Vec<Pubkey>,
        updates: mpsc::Sender<RemoteAccount>,
    ) -> Result<(), MirrorError> {
        let (ws, _resp) = tokio_tungstenite::connect_async(ws_url)
            .await
            .map_err(|e| MirrorError::WebSocket(e.to_string()))?;
        let (mut sink, mut stream) = ws.split();

        // request-id -> pubkey, resolved to subscription-id -> pubkey on ack.
        let mut req_to_key: HashMap<u64, Pubkey> = HashMap::new();
        for (i, pk) in pubkeys.iter().enumerate() {
            let id = i as u64 + 1;
            req_to_key.insert(id, *pk);
            let req = json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "accountSubscribe",
                "params": [pk.to_string(), { "encoding": "base64", "commitment": "confirmed" }],
            });
            sink.send(Message::Text(req.to_string()))
                .await
                .map_err(|e| MirrorError::WebSocket(e.to_string()))?;
        }
        tracing::info!(count = pubkeys.len(), "realtime mirror subscribed");

        let mut sub_to_key: HashMap<u64, Pubkey> = HashMap::new();

        while let Some(msg) = stream.next().await {
            let msg = msg.map_err(|e| MirrorError::WebSocket(e.to_string()))?;
            let text = match &msg {
                Message::Text(t) => t.as_str().to_string(),
                Message::Close(_) => break,
                Message::Ping(_) | Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => {
                    continue
                }
            };
            let Ok(value) = serde_json::from_str::<Value>(&text) else {
                continue;
            };

            // Subscription ack: { id, result: <subId> }.
            if let (Some(id), Some(sub)) = (
                value.get("id").and_then(|v| v.as_u64()),
                value.get("result").and_then(|v| v.as_u64()),
            ) {
                if let Some(pk) = req_to_key.get(&id) {
                    sub_to_key.insert(sub, *pk);
                }
                continue;
            }

            // Account notification: route by subscription id and forward.
            if value.get("method").and_then(|m| m.as_str()) == Some("accountNotification") {
                let params = &value["params"];
                let Some(sub) = params.get("subscription").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Some(pubkey) = sub_to_key.get(&sub).copied() else {
                    continue;
                };
                if let Some(remote) = decode_value(pubkey, &params["result"]["value"]) {
                    // A dropped receiver means the consumer is gone — stop cleanly.
                    if updates.send(remote).await.is_err() {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Decode an `accountNotification` `value` object into a [`RemoteAccount`].
fn decode_value(pubkey: Pubkey, value: &Value) -> Option<RemoteAccount> {
    if value.is_null() {
        return None;
    }
    let lamports = value.get("lamports")?.as_u64()?;
    let owner = Pubkey::from_str(value.get("owner")?.as_str()?).ok()?;
    let data_b64 = value
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let data = base64::engine::general_purpose::STANDARD
        .decode(data_b64)
        .ok()?;
    let executable = value
        .get("executable")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let rent_epoch = value.get("rentEpoch").and_then(|v| v.as_u64()).unwrap_or(0);
    Some(RemoteAccount {
        pubkey,
        lamports,
        data,
        owner,
        executable,
        rent_epoch,
    })
}
