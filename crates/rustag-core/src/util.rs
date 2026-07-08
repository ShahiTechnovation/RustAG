//! Small shared helpers used across the workspace.

/// Reduce a URL to just `scheme://host[:port]` before it lands in a shareable
/// artifact (an API response, a log line, an attestation, terminal output).
///
/// RPC endpoints embed API keys either in the query (`?api-key=...`) *or* in the
/// path (`/v2/<KEY>`, `/<KEY>`), so dropping only the query string is not enough;
/// we drop the path too and keep only the authority. This is the single source
/// of truth for credential redaction: the CLI, REST server, and cloud control
/// plane all call it on their read paths.
pub fn redact_url(url: &str) -> String {
    if let Some((scheme, rest)) = url.split_once("://") {
        let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
        if !authority.is_empty() {
            return format!("{scheme}://{authority}");
        }
    }
    // Non-URL input: at minimum strip any query string.
    match url.split_once('?') {
        Some((base, _)) => format!("{base}?<redacted>"),
        None => url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::redact_url;

    #[test]
    fn redacts_query_string_api_keys() {
        assert_eq!(
            redact_url("https://mainnet.helius-rpc.com/?api-key=secret123"),
            "https://mainnet.helius-rpc.com"
        );
    }

    #[test]
    fn redacts_path_embedded_api_keys() {
        // Alchemy-style keys live in the path, not the query.
        assert_eq!(
            redact_url("https://solana-mainnet.g.alchemy.com/v2/SECRET_KEY"),
            "https://solana-mainnet.g.alchemy.com"
        );
        assert_eq!(
            redact_url("https://rpc.example.com/SECRET_KEY"),
            "https://rpc.example.com"
        );
    }

    #[test]
    fn preserves_keyless_host_and_port() {
        assert_eq!(
            redact_url("https://api.mainnet-beta.solana.com"),
            "https://api.mainnet-beta.solana.com"
        );
        assert_eq!(redact_url("http://127.0.0.1:8899"), "http://127.0.0.1:8899");
    }
}
