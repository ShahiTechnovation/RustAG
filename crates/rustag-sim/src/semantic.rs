//! Decode raw account-state changes into **human-readable claims** a multisig
//! signer can actually reason about.
//!
//! A raw `StateDiff` says "account X changed". That is not enough to sign
//! against. [`decode_changes`] turns each change into a typed [`SemanticChange`]
//! - "the SPL token balance of X went from 1,000 to 0", "the upgrade authority
//! of program P rotated to Q", "a durable-nonce account was created" - the layer
//! that makes an [`crate::invariants::Alarm`] legible.

use std::collections::BTreeMap;

use rustag_core::{fidelity, AccountEntry};
use serde::Serialize;
use solana_pubkey::{pubkey, Pubkey};

/// SPL Token program (v1).
const SPL_TOKEN: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
/// SPL Token-2022 program.
const SPL_TOKEN_2022: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
/// The System program (owner of durable-nonce accounts).
const SYSTEM_PROGRAM: Pubkey = pubkey!("11111111111111111111111111111111");
/// Offset of the `amount` field in an SPL token account (mint[32] + owner[32]).
const SPL_TOKEN_AMOUNT_OFFSET: usize = 64;
/// Minimum length of an SPL token account (through the `amount` field).
const SPL_TOKEN_MIN_LEN: usize = 72;
/// Offset of the `owner` (authority) field in an SPL token account.
const SPL_TOKEN_AUTHORITY_OFFSET: usize = 32;
/// Offset of the `delegate` field in an SPL token account (after amount).
#[allow(dead_code)]
const SPL_TOKEN_DELEGATE_OFFSET: usize = 72;
/// Offset of the `close_authority` field in an SPL token account.
const SPL_TOKEN_CLOSE_AUTH_OFFSET: usize = 130;
/// Full length of an SPL token account with all optional fields.
const SPL_TOKEN_FULL_LEN: usize = 165;
/// Serialized size of an initialized durable-nonce account.
const NONCE_ACCOUNT_LEN: usize = 80;
/// Tag byte for `UpgradeableLoaderState::ProgramData`.
const TAG_PROGRAM_DATA: [u8; 4] = [3, 0, 0, 0];
/// Offset of the deployment slot in a ProgramData account.
const PROGRAMDATA_SLOT_OFFSET: usize = 4;

/// A single decoded, human-legible change to one account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SemanticChange {
    /// An account was created.
    AccountCreated { account: String, owner: String, lamports: u64 },
    /// A durable-nonce account was created (called out specifically because
    /// durable nonces are the primitive behind blind-signing replay attacks).
    NonceAccountCreated { account: String },
    /// An account was closed / removed.
    AccountClosed { account: String },
    /// An account's owning program changed.
    OwnerChanged { account: String, before: String, after: String },
    /// A program's upgrade authority changed.
    UpgradeAuthority {
        account: String,
        before: Option<String>,
        after: Option<String>,
    },
    /// A program's upgrade authority was set to None (program is now immutable).
    ProgramFrozen { account: String, program: String },
    /// A ProgramData account's deployment slot changed, indicating the program
    /// bytecode was upgraded.
    ProgramUpgraded {
        account: String,
        before_slot: u64,
        after_slot: u64,
    },
    /// An SPL token account's `amount` changed.
    TokenAmount { account: String, before: u64, after: u64 },
    /// An SPL token account's authority (owner/delegate/close authority) changed.
    TokenAuthorityChanged {
        account: String,
        field: String,
        before: String,
        after: String,
    },
    /// An account's SOL balance changed (no other decodable change).
    SolBalance { account: String, before: u64, after: u64 },
    /// The account's data changed in a way we do not specifically decode.
    DataChanged { account: String, before_len: usize, after_len: usize },
}

fn is_token_owner(owner: &Pubkey) -> bool {
    *owner == SPL_TOKEN || *owner == SPL_TOKEN_2022
}

fn token_amount(entry: &AccountEntry) -> Option<u64> {
    if is_token_owner(&entry.owner) && entry.data.len() >= SPL_TOKEN_MIN_LEN {
        let bytes: [u8; 8] = entry.data[SPL_TOKEN_AMOUNT_OFFSET..SPL_TOKEN_MIN_LEN]
            .try_into()
            .ok()?;
        Some(u64::from_le_bytes(bytes))
    } else {
        None
    }
}

/// Decode the changes between two account sets into human-legible claims,
/// sorted by pubkey for stable output.
pub fn decode_changes(pre: &[AccountEntry], post: &[AccountEntry]) -> Vec<SemanticChange> {
    let pre_map: BTreeMap<Pubkey, &AccountEntry> = pre.iter().map(|e| (e.pubkey, e)).collect();
    let post_map: BTreeMap<Pubkey, &AccountEntry> = post.iter().map(|e| (e.pubkey, e)).collect();

    let mut keys: Vec<Pubkey> = pre_map.keys().chain(post_map.keys()).copied().collect();
    keys.sort_by_key(|k| k.to_bytes());
    keys.dedup();

    let mut changes = Vec::new();
    for key in keys {
        match (pre_map.get(&key), post_map.get(&key)) {
            (None, Some(p)) => changes.push(created(p)),
            (Some(_), None) => changes.push(SemanticChange::AccountClosed {
                account: key.to_string(),
            }),
            (Some(a), Some(b)) => {
                if let Some(change) = decode_modified(a, b) {
                    changes.push(change);
                }
            }
            (None, None) => {}
        }
    }
    changes
}

fn created(entry: &AccountEntry) -> SemanticChange {
    if entry.owner == SYSTEM_PROGRAM && entry.data.len() == NONCE_ACCOUNT_LEN {
        SemanticChange::NonceAccountCreated {
            account: entry.pubkey.to_string(),
        }
    } else {
        SemanticChange::AccountCreated {
            account: entry.pubkey.to_string(),
            owner: entry.owner.to_string(),
            lamports: entry.lamports,
        }
    }
}

fn decode_modified(a: &AccountEntry, b: &AccountEntry) -> Option<SemanticChange> {
    let account = a.pubkey.to_string();

    // 1. Owner reassignment is the most security-relevant change.
    if a.owner != b.owner {
        return Some(SemanticChange::OwnerChanged {
            account,
            before: a.owner.to_string(),
            after: b.owner.to_string(),
        });
    }

    // 2. Upgrade authority rotation or program freeze (ProgramData accounts).
    if let (Some(before), Some(after)) = (
        fidelity::parse_upgrade_authority(&a.data),
        fidelity::parse_upgrade_authority(&b.data),
    ) {
        if before != after {
            // Check specifically if the program was frozen (authority → None).
            if after.is_none() && before.is_some() {
                return Some(SemanticChange::ProgramFrozen {
                    account: account.clone(),
                    program: before.unwrap().to_string(),
                });
            }
            return Some(SemanticChange::UpgradeAuthority {
                account,
                before: before.map(|p| p.to_string()),
                after: after.map(|p| p.to_string()),
            });
        }
        // Check if the deployment slot changed (program was upgraded).
        if let (Some(before_slot), Some(after_slot)) =
            (programdata_slot(&a.data), programdata_slot(&b.data))
        {
            if before_slot != after_slot {
                return Some(SemanticChange::ProgramUpgraded {
                    account,
                    before_slot,
                    after_slot,
                });
            }
        }
    }

    // 3. SPL token authority change (owner/delegate/close authority).
    if is_token_owner(&a.owner) && a.data.len() >= SPL_TOKEN_FULL_LEN && b.data.len() >= SPL_TOKEN_FULL_LEN {
        if let Some(change) = decode_token_authority_change(a, b) {
            return Some(change);
        }
    }

    // 4. SPL token amount change.
    if let (Some(before), Some(after)) = (token_amount(a), token_amount(b)) {
        if before != after {
            return Some(SemanticChange::TokenAmount { account, before, after });
        }
    }

    // 5. Any other data change.
    if a.data != b.data {
        return Some(SemanticChange::DataChanged {
            account,
            before_len: a.data.len(),
            after_len: b.data.len(),
        });
    }

    // 6. A pure lamports move.
    if a.lamports != b.lamports {
        return Some(SemanticChange::SolBalance {
            account,
            before: a.lamports,
            after: b.lamports,
        });
    }

    None
}

/// Extract the deployment slot from a ProgramData account.
fn programdata_slot(data: &[u8]) -> Option<u64> {
    if data.len() >= PROGRAMDATA_SLOT_OFFSET + 8 && data[0..4] == TAG_PROGRAM_DATA {
        let bytes: [u8; 8] = data[PROGRAMDATA_SLOT_OFFSET..PROGRAMDATA_SLOT_OFFSET + 8]
            .try_into()
            .ok()?;
        Some(u64::from_le_bytes(bytes))
    } else {
        None
    }
}

/// Detect changes to an SPL token account's authority fields.
fn decode_token_authority_change(a: &AccountEntry, b: &AccountEntry) -> Option<SemanticChange> {
    let account = a.pubkey.to_string();

    // Check token owner (authority) field at offset 32.
    let a_auth = extract_pubkey(&a.data, SPL_TOKEN_AUTHORITY_OFFSET);
    let b_auth = extract_pubkey(&b.data, SPL_TOKEN_AUTHORITY_OFFSET);
    if a_auth != b_auth {
        return Some(SemanticChange::TokenAuthorityChanged {
            account,
            field: "owner".to_string(),
            before: a_auth.to_string(),
            after: b_auth.to_string(),
        });
    }

    // Check close authority at offset 130 (if present).
    if a.data.len() >= SPL_TOKEN_CLOSE_AUTH_OFFSET + 32
        && b.data.len() >= SPL_TOKEN_CLOSE_AUTH_OFFSET + 32
    {
        let a_close = extract_pubkey(&a.data, SPL_TOKEN_CLOSE_AUTH_OFFSET);
        let b_close = extract_pubkey(&b.data, SPL_TOKEN_CLOSE_AUTH_OFFSET);
        if a_close != b_close {
            return Some(SemanticChange::TokenAuthorityChanged {
                account,
                field: "closeAuthority".to_string(),
                before: a_close.to_string(),
                after: b_close.to_string(),
            });
        }
    }

    None
}

/// Extract a Pubkey from a byte slice at the given offset.
fn extract_pubkey(data: &[u8], offset: usize) -> Pubkey {
    if data.len() >= offset + 32 {
        let bytes: [u8; 32] = data[offset..offset + 32].try_into().unwrap_or([0; 32]);
        Pubkey::new_from_array(bytes)
    } else {
        Pubkey::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustag_core::AccountSync;
    use uuid::Uuid;

    fn entry(pubkey: Pubkey, owner: Pubkey, lamports: u64, data: Vec<u8>) -> AccountEntry {
        AccountEntry {
            pubkey,
            data,
            owner,
            lamports,
            executable: false,
            rent_epoch: 0,
            sync_state: AccountSync::clean_now(),
            category: None,
            stagenet_id: Uuid::nil(),
        }
    }

    fn token_account(pubkey: Pubkey, amount: u64) -> AccountEntry {
        let mut data = vec![0u8; SPL_TOKEN_MIN_LEN];
        data[SPL_TOKEN_AMOUNT_OFFSET..SPL_TOKEN_MIN_LEN].copy_from_slice(&amount.to_le_bytes());
        entry(pubkey, SPL_TOKEN, 2_039_280, data)
    }

    #[test]
    fn decodes_token_amount_drain() {
        let pk = Pubkey::new_from_array([3; 32]);
        let pre = vec![token_account(pk, 1_000_000)];
        let post = vec![token_account(pk, 0)];
        let changes = decode_changes(&pre, &post);
        assert_eq!(
            changes,
            vec![SemanticChange::TokenAmount {
                account: pk.to_string(),
                before: 1_000_000,
                after: 0
            }]
        );
    }

    #[test]
    fn decodes_nonce_creation_and_close() {
        let nonce_pk = Pubkey::new_from_array([4; 32]);
        let nonce = entry(nonce_pk, SYSTEM_PROGRAM, 1, vec![0u8; NONCE_ACCOUNT_LEN]);
        assert_eq!(
            decode_changes(&[], &[nonce.clone()]),
            vec![SemanticChange::NonceAccountCreated {
                account: nonce_pk.to_string()
            }]
        );
        assert_eq!(
            decode_changes(&[nonce], &[]),
            vec![SemanticChange::AccountClosed {
                account: nonce_pk.to_string()
            }]
        );
    }

    #[test]
    fn decodes_owner_reassignment() {
        let pk = Pubkey::new_from_array([5; 32]);
        let pre = vec![entry(pk, Pubkey::new_from_array([1; 32]), 10, vec![0])];
        let post = vec![entry(pk, Pubkey::new_from_array([2; 32]), 10, vec![0])];
        let changes = decode_changes(&pre, &post);
        assert!(matches!(changes[0], SemanticChange::OwnerChanged { .. }));
    }

    #[test]
    fn pure_sol_move_is_sol_balance() {
        let pk = Pubkey::new_from_array([6; 32]);
        let owner = Pubkey::new_from_array([1; 32]);
        let changes = decode_changes(
            &[entry(pk, owner, 100, vec![9])],
            &[entry(pk, owner, 40, vec![9])],
        );
        assert_eq!(
            changes,
            vec![SemanticChange::SolBalance {
                account: pk.to_string(),
                before: 100,
                after: 40
            }]
        );
    }

    #[test]
    fn identical_state_has_no_changes() {
        let pk = Pubkey::new_from_array([6; 32]);
        let owner = Pubkey::new_from_array([1; 32]);
        let a = vec![entry(pk, owner, 100, vec![9])];
        assert!(decode_changes(&a, &a).is_empty());
    }
}
