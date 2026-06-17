//! Registry of well-known mainnet programs and oracle price feeds.
//!
//! Every pubkey here was verified to exist on mainnet-beta at authoring time.
//! Preloading these gives a stagenet real DeFi state without manual setup —
//! the core value proposition of RustAG.

use std::str::FromStr;

use solana_pubkey::Pubkey;

use crate::account::AccountCategory;

// --- Programs (executable accounts) ----------------------------------------
const JUPITER_V6: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
const RAYDIUM_AMM_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const ORCA_WHIRLPOOLS: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
const MARINADE: &str = "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD";
const SPL_TOKEN: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const ASSOCIATED_TOKEN: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const TOKEN_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const METAPLEX_TOKEN_METADATA: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
const METAPLEX_CORE: &str = "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d";

// --- Oracle price feeds ----------------------------------------------------
const PYTH_SOL_USD: &str = "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG";
const PYTH_ETH_USD: &str = "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB";
const PYTH_USDC_USD: &str = "Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD";
const SWITCHBOARD_USDT_USD: &str = "8SXvChNYFhRq4EZuZvnhjrB3jJRQCv4k3P4W6hesH3Ee";

const PROGRAMS: &[&str] = &[
    JUPITER_V6,
    RAYDIUM_AMM_V4,
    ORCA_WHIRLPOOLS,
    MARINADE,
    SPL_TOKEN,
    ASSOCIATED_TOKEN,
    TOKEN_2022,
    METAPLEX_TOKEN_METADATA,
    METAPLEX_CORE,
];

const ORACLES: &[&str] = &[
    PYTH_SOL_USD,
    PYTH_ETH_USD,
    PYTH_USDC_USD,
    SWITCHBOARD_USDT_USD,
];

/// Parse a registry constant. Safe to unwrap: every constant is validated by the
/// `registry_pubkeys_parse` test, so a malformed entry fails CI, never prod.
fn pk(s: &str) -> Pubkey {
    Pubkey::from_str(s).expect("registry pubkey is valid base58 (checked by test)")
}

/// All oracle pubkeys (used by the fetcher to flag accounts for fast re-sync).
pub fn oracle_pubkeys() -> Vec<Pubkey> {
    ORACLES.iter().map(|s| pk(s)).collect()
}

/// Every known account paired with its category.
pub fn all_entries() -> Vec<(Pubkey, AccountCategory)> {
    PROGRAMS
        .iter()
        .map(|s| (pk(s), AccountCategory::Program))
        .chain(ORACLES.iter().map(|s| (pk(s), AccountCategory::Oracle)))
        .collect()
}

/// Classify a pubkey if it is a known program or oracle.
pub fn categorize(pubkey: &Pubkey) -> Option<AccountCategory> {
    all_entries()
        .into_iter()
        .find(|(p, _)| p == pubkey)
        .map(|(_, c)| c)
}

/// Resolve a human-friendly preload name (e.g. `jupiter`, `pyth`) to the set of
/// accounts it should load. Returns `None` for unknown names.
pub fn resolve(name: &str) -> Option<Vec<(Pubkey, AccountCategory)>> {
    let one = |s: &str, c: AccountCategory| Some(vec![(pk(s), c)]);
    match name.trim().to_lowercase().as_str() {
        "jupiter" | "jupiter-v6" => one(JUPITER_V6, AccountCategory::Program),
        "raydium" | "raydium-amm" | "raydium-v4" => one(RAYDIUM_AMM_V4, AccountCategory::Program),
        "orca" | "orca-whirlpools" | "whirlpool" => one(ORCA_WHIRLPOOLS, AccountCategory::Program),
        "marinade" => one(MARINADE, AccountCategory::Program),
        "spl-token" | "token" => Some(vec![
            (pk(SPL_TOKEN), AccountCategory::Program),
            (pk(ASSOCIATED_TOKEN), AccountCategory::Program),
        ]),
        "token-2022" | "token2022" => one(TOKEN_2022, AccountCategory::Program),
        "metaplex" => Some(vec![
            (pk(METAPLEX_TOKEN_METADATA), AccountCategory::Program),
            (pk(METAPLEX_CORE), AccountCategory::Program),
        ]),
        "pyth" => Some(vec![
            (pk(PYTH_SOL_USD), AccountCategory::Oracle),
            (pk(PYTH_ETH_USD), AccountCategory::Oracle),
            (pk(PYTH_USDC_USD), AccountCategory::Oracle),
        ]),
        "switchboard" => one(SWITCHBOARD_USDT_USD, AccountCategory::Oracle),
        _ => None,
    }
}

/// Names accepted by [`resolve`], for CLI help and validation.
pub fn available() -> &'static [&'static str] {
    &[
        "jupiter",
        "pyth",
        "switchboard",
        "raydium",
        "orca",
        "marinade",
        "spl-token",
        "token-2022",
        "metaplex",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_pubkeys_parse() {
        for s in PROGRAMS.iter().chain(ORACLES.iter()) {
            Pubkey::from_str(s).unwrap_or_else(|_| panic!("invalid registry pubkey: {s}"));
        }
    }

    #[test]
    fn resolve_known_names() {
        assert!(resolve("jupiter").is_some());
        assert!(resolve("PYTH").is_some()); // case-insensitive
        assert_eq!(resolve("pyth").unwrap().len(), 3);
        assert!(resolve("nope").is_none());
    }

    #[test]
    fn categorize_oracles_and_programs() {
        assert_eq!(categorize(&pk(PYTH_SOL_USD)), Some(AccountCategory::Oracle));
        assert_eq!(categorize(&pk(JUPITER_V6)), Some(AccountCategory::Program));
        assert_eq!(categorize(&Pubkey::new_unique()), None);
    }
}
