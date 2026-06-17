//! On-demand mainnet account fetcher (raw JSON-RPC over `reqwest`).

use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;

use base64::Engine;
use serde::Deserialize;
use solana_pubkey::Pubkey;

use crate::account::RemoteAccount;
use crate::error::MirrorError;
use crate::rate_limiter::RpcRateLimiter;

/// Maximum number of accounts a single `getMultipleAccounts` call accepts.
pub const MAX_ACCOUNTS_PER_REQUEST: usize = 100;

/// Fetches account state from a mainnet RPC endpoint.
pub struct MainnetMirror {
    http: reqwest::Client,
    endpoint: String,
    limiter: RpcRateLimiter,
    /// Pubkeys treated as oracles (re-synced frequently by the scheduler).
    oracle_registry: HashSet<Pubkey>,
}

impl MainnetMirror {
    /// Build a mirror pointing at `endpoint`, capped at `max_rps` requests/sec.
    pub fn new(endpoint: impl Into<String>, max_rps: u32) -> Result<Self, MirrorError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("rustag-mirror/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            http,
            endpoint: endpoint.into(),
            limiter: RpcRateLimiter::new(max_rps),
            oracle_registry: crate::registry::oracle_pubkeys().into_iter().collect(),
        })
    }

    /// The configured mainnet endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Whether `pubkey` is a known oracle (eligible for aggressive re-sync).
    pub fn is_oracle(&self, pubkey: &Pubkey) -> bool {
        self.oracle_registry.contains(pubkey)
    }

    /// Fetch a single account, returning `None` if it does not exist on mainnet.
    pub async fn fetch_one(&self, pubkey: &Pubkey) -> Result<Option<RemoteAccount>, MirrorError> {
        Ok(self
            .fetch_multiple(std::slice::from_ref(pubkey))
            .await?
            .into_iter()
            .next()
            .flatten())
    }

    /// Fetch many accounts in `getMultipleAccounts` batches of up to
    /// [`MAX_ACCOUNTS_PER_REQUEST`]. The returned vector is aligned 1:1 with
    /// `pubkeys`; a `None` element means the account does not exist on mainnet.
    #[tracing::instrument(skip(self, pubkeys), fields(count = pubkeys.len()))]
    pub async fn fetch_multiple(
        &self,
        pubkeys: &[Pubkey],
    ) -> Result<Vec<Option<RemoteAccount>>, MirrorError> {
        let mut out = Vec::with_capacity(pubkeys.len());
        for chunk in pubkeys.chunks(MAX_ACCOUNTS_PER_REQUEST) {
            self.limiter.acquire().await;
            let mut batch = self.fetch_chunk(chunk).await?;
            out.append(&mut batch);
        }
        Ok(out)
    }

    async fn fetch_chunk(
        &self,
        chunk: &[Pubkey],
    ) -> Result<Vec<Option<RemoteAccount>>, MirrorError> {
        let keys: Vec<String> = chunk.iter().map(|k| k.to_string()).collect();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getMultipleAccounts",
            "params": [keys, { "encoding": "base64", "commitment": "confirmed" }],
        });

        let body = self
            .http
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let envelope: RpcEnvelope<GetMultipleAccountsResult> = serde_json::from_str(&body)
            .map_err(|e| MirrorError::InvalidResponse(format!("{e}: {}", truncate(&body))))?;

        if let Some(err) = envelope.error {
            return Err(MirrorError::Rpc {
                code: err.code,
                message: err.message,
            });
        }

        let value = envelope
            .result
            .ok_or_else(|| MirrorError::InvalidResponse("missing `result`".into()))?
            .value;

        if value.len() != chunk.len() {
            return Err(MirrorError::InvalidResponse(format!(
                "expected {} accounts, got {}",
                chunk.len(),
                value.len()
            )));
        }

        chunk
            .iter()
            .zip(value)
            .map(|(pubkey, account)| Self::decode_account(*pubkey, account))
            .collect()
    }

    fn decode_account(
        pubkey: Pubkey,
        account: Option<UiAccount>,
    ) -> Result<Option<RemoteAccount>, MirrorError> {
        let Some(account) = account else {
            return Ok(None);
        };

        let (encoded, encoding) = &account.data;
        if encoding != "base64" {
            return Err(MirrorError::Decode(format!(
                "unexpected data encoding `{encoding}`"
            )));
        }
        let data = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| MirrorError::Decode(e.to_string()))?;

        let owner = Pubkey::from_str(&account.owner)
            .map_err(|_| MirrorError::InvalidPubkey(account.owner.clone()))?;

        Ok(Some(RemoteAccount {
            pubkey,
            lamports: account.lamports,
            data,
            owner,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        }))
    }
}

fn truncate(s: &str) -> String {
    const MAX: usize = 200;
    if s.len() <= MAX {
        s.to_string()
    } else {
        // Slice on a char boundary so non-ASCII error bodies don't panic.
        let end = (0..=MAX)
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(0);
        format!("{}...", &s[..end])
    }
}

// --- JSON-RPC wire types ----------------------------------------------------

#[derive(Deserialize)]
struct RpcEnvelope<T> {
    result: Option<T>,
    error: Option<RpcErrorObject>,
}

#[derive(Deserialize)]
struct RpcErrorObject {
    code: i64,
    message: String,
}

#[derive(Deserialize)]
struct GetMultipleAccountsResult {
    value: Vec<Option<UiAccount>>,
}

#[derive(Deserialize)]
struct UiAccount {
    lamports: u64,
    owner: String,
    /// `[<base64>, "base64"]`
    data: (String, String),
    #[serde(default)]
    executable: bool,
    #[serde(rename = "rentEpoch", default)]
    rent_epoch: u64,
}
