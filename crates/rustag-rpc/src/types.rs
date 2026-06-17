//! Shared response encoders for the Solana-compatible RPC + REST surfaces.

use base64::Engine;
use serde_json::{json, Value};

use rustag_core::AccountEntry;

/// The RPC `apiVersion` we advertise (matches the Agave line we run on).
pub const API_VERSION: &str = "2.1.0";

/// A fixed genesis hash for this stagenet family.
pub const GENESIS_HASH: &str = "RUSTaG1111111111111111111111111111111111111";

/// Encode an account into the Solana JSON-RPC `value` shape (base64 data).
pub fn encode_account_base64(entry: &AccountEntry) -> Value {
    json!({
        "lamports": entry.lamports,
        "owner": entry.owner.to_string(),
        "data": [base64::engine::general_purpose::STANDARD.encode(&entry.data), "base64"],
        "executable": entry.executable,
        "rentEpoch": entry.rent_epoch,
        "space": entry.data.len(),
    })
}

/// Wrap a value in the standard `{ context, value }` envelope.
pub fn with_context(slot: u64, value: Value) -> Value {
    json!({
        "context": { "apiVersion": API_VERSION, "slot": slot },
        "value": value,
    })
}

/// A richer JSON view of an account for the dashboard REST API.
pub fn encode_account_rich(entry: &AccountEntry) -> Value {
    json!({
        "pubkey": entry.pubkey.to_string(),
        "lamports": entry.lamports,
        "sol": entry.sol(),
        "owner": entry.owner.to_string(),
        "executable": entry.executable,
        "rentEpoch": entry.rent_epoch,
        "dataLen": entry.data.len(),
        "dataBase64": base64::engine::general_purpose::STANDARD.encode(&entry.data),
        "syncState": entry.sync_state.label(),
        "category": entry.category.map(|c| c.label()),
    })
}
