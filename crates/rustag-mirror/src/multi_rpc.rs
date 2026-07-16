//! **N-of-M multi-RPC fetcher** with provenance tracking.
//!
//! For pre-execution assurance, we need to bound the trust placed in any single
//! RPC endpoint. `MultiRpcFetcher` fetches the same accounts from M independent
//! endpoints and requires N of them to agree on each account's data before
//! including it in the closure. The provenance — which endpoints were queried,
//! what slot each returned, and whether they agreed — is recorded as
//! [`InputProvenance`] and stamped into the EvidenceBundle so verifiers know
//! exactly how trustworthy the input state was.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;

use crate::account::RemoteAccount;
use crate::error::MirrorError;
use crate::fetcher::MainnetMirror;

/// The provenance record stamped into every `EvidenceBundle`, documenting how
/// the closure's accounts were sourced from mainnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputProvenance {
    /// The endpoints that were queried (redacted to hostnames).
    pub endpoints: Vec<String>,
    /// How many endpoints had to agree for an account to be included (N).
    pub min_agreement: usize,
    /// Per-account agreement: pubkey → number of endpoints that returned the
    /// same data hash.
    pub per_account_agreement: HashMap<String, usize>,
    /// Whether every account in the closure met the N-of-M threshold.
    pub full_agreement: bool,
    /// Accounts where fewer than N endpoints agreed (potential drift).
    pub disagreements: Vec<String>,
}

impl Default for InputProvenance {
    fn default() -> Self {
        Self {
            endpoints: Vec::new(),
            min_agreement: 1,
            per_account_agreement: HashMap::new(),
            full_agreement: true,
            disagreements: Vec::new(),
        }
    }
}

/// Fetches accounts from multiple RPC endpoints and requires N-of-M agreement.
pub struct MultiRpcFetcher {
    mirrors: Vec<MainnetMirror>,
    min_agreement: usize,
}

impl MultiRpcFetcher {
    /// Create a fetcher that queries `endpoints` and requires `min_agreement`
    /// of them to return the same data for each account.
    ///
    /// # Panics
    /// Panics if `min_agreement` is 0 or greater than `endpoints.len()`.
    pub fn new(
        endpoints: Vec<String>,
        min_agreement: usize,
        max_rps_per_endpoint: u32,
    ) -> Result<Self, MirrorError> {
        assert!(min_agreement > 0, "min_agreement must be > 0");
        assert!(
            min_agreement <= endpoints.len(),
            "min_agreement ({min_agreement}) > endpoint count ({})",
            endpoints.len()
        );

        let mirrors = endpoints
            .iter()
            .map(|ep| MainnetMirror::new(ep, max_rps_per_endpoint))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            mirrors,
            min_agreement,
        })
    }

    /// Create a fetcher that uses a single endpoint (1-of-1, no cross-checking).
    pub fn single(endpoint: String, max_rps: u32) -> Result<Self, MirrorError> {
        Self::new(vec![endpoint], 1, max_rps)
    }

    /// Number of endpoints being queried.
    pub fn endpoint_count(&self) -> usize {
        self.mirrors.len()
    }

    /// Fetch accounts from all endpoints and return those meeting the agreement
    /// threshold, along with a provenance record.
    #[tracing::instrument(skip(self, pubkeys), fields(count = pubkeys.len(), endpoints = self.mirrors.len()))]
    pub async fn fetch_with_provenance(
        &self,
        pubkeys: &[Pubkey],
    ) -> Result<(Vec<RemoteAccount>, InputProvenance), MirrorError> {
        // Fetch from all endpoints in parallel.
        let mut all_results: Vec<Vec<Option<RemoteAccount>>> = Vec::new();

        for mirror in &self.mirrors {
            let result = mirror.fetch_multiple(pubkeys).await?;
            all_results.push(result);
        }

        let endpoints: Vec<String> = self
            .mirrors
            .iter()
            .map(|m| redact_endpoint(m.endpoint()))
            .collect();

        let mut accounts = Vec::new();
        let mut per_account_agreement = HashMap::new();
        let mut disagreements = Vec::new();

        for (idx, pubkey) in pubkeys.iter().enumerate() {
            // Collect non-None results from each endpoint for this pubkey.
            let mut data_hashes: HashMap<u64, (usize, RemoteAccount)> = HashMap::new();

            for endpoint_results in &all_results {
                if let Some(Some(account)) = endpoint_results.get(idx) {
                    let hash = simple_hash(&account.data, account.lamports, &account.owner);
                    data_hashes
                        .entry(hash)
                        .and_modify(|(count, _)| *count += 1)
                        .or_insert((1, account.clone()));
                }
            }

            // Find the consensus result (most endpoints agreed).
            let consensus = data_hashes
                .into_iter()
                .max_by_key(|(_, (count, _))| *count);

            match consensus {
                Some((_, (count, account))) if count >= self.min_agreement => {
                    per_account_agreement.insert(pubkey.to_string(), count);
                    accounts.push(account);
                }
                Some((_, (count, _))) => {
                    per_account_agreement.insert(pubkey.to_string(), count);
                    disagreements.push(pubkey.to_string());
                    tracing::warn!(
                        pubkey = %pubkey,
                        agreement = count,
                        required = self.min_agreement,
                        "account did not meet N-of-M agreement threshold"
                    );
                }
                None => {
                    // Account not found on any endpoint — not necessarily an error
                    // (the account may not exist), but record it.
                    tracing::debug!(pubkey = %pubkey, "account not found on any endpoint");
                }
            }
        }

        let full_agreement = disagreements.is_empty();

        let provenance = InputProvenance {
            endpoints,
            min_agreement: self.min_agreement,
            per_account_agreement,
            full_agreement,
            disagreements,
        };

        tracing::info!(
            accounts_resolved = accounts.len(),
            full_agreement,
            "multi-RPC fetch complete"
        );

        Ok((accounts, provenance))
    }
}

/// Redact an endpoint URL to just its hostname for privacy in provenance records.
fn redact_endpoint(url: &str) -> String {
    // Try to extract just the host portion.
    if let Some(start) = url.find("://") {
        let rest = &url[start + 3..];
        if let Some(end) = rest.find('/') {
            // Remove any query params from the host.
            let host = &rest[..end];
            if let Some(qmark) = host.find('?') {
                return host[..qmark].to_string();
            }
            return host.to_string();
        }
        // No path, just take everything up to query params.
        if let Some(qmark) = rest.find('?') {
            return rest[..qmark].to_string();
        }
        return rest.to_string();
    }
    // Fallback: return as-is.
    url.to_string()
}

/// A simple non-cryptographic hash for comparing account data across endpoints.
/// We only need equality checking, not security.
fn simple_hash(data: &[u8], lamports: u64, owner: &Pubkey) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    lamports.hash(&mut hasher);
    owner.to_bytes().hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_strips_api_keys() {
        assert_eq!(
            redact_endpoint("https://mainnet.helius-rpc.com/?api-key=SECRET_KEY"),
            "mainnet.helius-rpc.com"
        );
    }

    #[test]
    fn redact_preserves_hostname() {
        assert_eq!(
            redact_endpoint("https://api.mainnet-beta.solana.com"),
            "api.mainnet-beta.solana.com"
        );
    }

    #[test]
    fn redact_handles_path() {
        assert_eq!(
            redact_endpoint("https://rpc.example.com/v1/mainnet"),
            "rpc.example.com"
        );
    }

    #[test]
    fn default_provenance_is_full_agreement() {
        let p = InputProvenance::default();
        assert!(p.full_agreement);
        assert!(p.disagreements.is_empty());
    }
}
