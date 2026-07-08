//! Plain account types exchanged between the mirror and the core runtime.

use solana_pubkey::Pubkey;

/// A snapshot of an account as it exists on mainnet right now.
///
/// This is deliberately a dumb data holder with no notion of local
/// dirty/clean state - `rustag-core` wraps it into its own `AccountEntry`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAccount {
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
}

/// Coarse classification of a known account, used to decide sync cadence and to
/// drive the dashboard. Oracles are re-synced aggressively; everything else is
/// synced lazily.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AccountCategory {
    /// A price feed (e.g. Pyth). Synced frequently.
    Oracle,
    /// An executable program account.
    Program,
    /// An SPL token mint.
    TokenMint,
    /// Any other data account.
    Data,
}

impl AccountCategory {
    /// Stable string form persisted in the database and exposed over the API.
    pub fn label(&self) -> &'static str {
        match self {
            AccountCategory::Oracle => "Oracle",
            AccountCategory::Program => "Program",
            AccountCategory::TokenMint => "TokenMint",
            AccountCategory::Data => "Data",
        }
    }

    /// Parse the [`AccountCategory::label`] form back into a category.
    pub fn from_label(s: &str) -> Option<Self> {
        match s {
            "Oracle" => Some(AccountCategory::Oracle),
            "Program" => Some(AccountCategory::Program),
            "TokenMint" => Some(AccountCategory::TokenMint),
            "Data" => Some(AccountCategory::Data),
            _ => None,
        }
    }
}
