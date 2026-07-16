//! Pre/post **invariant policy** for pre-execution rehearsal.
//!
//! Where [`crate::fuzz`] evaluates invariants across a *fuzzing loop*, this
//! module evaluates a policy over a single `(pre, post)` account-state pair: the
//! state a proposed privileged payload executed *against* versus the state it
//! *produced*. Each rule that trips emits an [`Alarm`]. The design center is a
//! powerful **zero-config** default ([`Policy::standard`]) that catches the
//! Drift-shaped failure - a hidden authority rotation or a new durable-nonce
//! account buried in a routine-looking multisig payload - without the signer
//! having to know which accounts to watch in advance.

use std::collections::HashMap;

use rustag_core::{fidelity, AccountEntry};
use serde::Serialize;
use solana_pubkey::{pubkey, Pubkey};

use crate::exploit::Severity;

/// The System program - owner of durable-nonce accounts.
const SYSTEM_PROGRAM: Pubkey = pubkey!("11111111111111111111111111111111");
/// Serialized size of an initialized durable-nonce account.
const NONCE_ACCOUNT_LEN: usize = 80;

/// A tripped invariant: something the payload changed that a signer should see
/// before approving.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alarm {
    /// Stable rule identifier (e.g. `upgrade-authority-rotated`).
    pub rule: String,
    /// How serious the change is.
    pub severity: Severity,
    /// The account the alarm is about, if specific (base58).
    pub account: Option<String>,
    /// Human-readable explanation.
    pub detail: String,
}

/// A `(pre, post)` view keyed by pubkey, handed to each rule.
pub struct PrePost<'a> {
    pre: HashMap<Pubkey, &'a AccountEntry>,
    post: HashMap<Pubkey, &'a AccountEntry>,
}

impl<'a> PrePost<'a> {
    /// Build a view from two account sets.
    pub fn new(pre: &'a [AccountEntry], post: &'a [AccountEntry]) -> Self {
        Self {
            pre: pre.iter().map(|e| (e.pubkey, e)).collect(),
            post: post.iter().map(|e| (e.pubkey, e)).collect(),
        }
    }

    /// The account's pre-state, if present.
    pub fn pre(&self, pubkey: &Pubkey) -> Option<&AccountEntry> {
        self.pre.get(pubkey).copied()
    }

    /// The account's post-state, if present.
    pub fn post(&self, pubkey: &Pubkey) -> Option<&AccountEntry> {
        self.post.get(pubkey).copied()
    }

    /// Accounts present in `post` but not `pre` (newly created).
    pub fn added(&self) -> impl Iterator<Item = &AccountEntry> {
        self.post
            .iter()
            .filter(|(k, _)| !self.pre.contains_key(*k))
            .map(|(_, v)| *v)
    }

    /// Accounts present in both, so pre/post can be compared.
    pub fn common(&self) -> impl Iterator<Item = (&AccountEntry, &AccountEntry)> {
        self.pre.iter().filter_map(move |(k, pre)| {
            self.post.get(k).map(|post| (*pre, *post))
        })
    }
}

/// A rule: given a pre/post view, produce zero or more alarms.
pub type PolicyRule = Box<dyn Fn(&PrePost) -> Vec<Alarm> + Send + Sync>;

/// An ordered set of named invariant rules.
#[derive(Default)]
pub struct Policy {
    rules: Vec<(String, PolicyRule)>,
}

impl Policy {
    /// An empty policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a named rule.
    pub fn rule(mut self, name: impl Into<String>, rule: PolicyRule) -> Self {
        self.rules.push((name.into(), rule));
        self
    }

    /// The zero-config default: the universal, parameter-free checks that catch
    /// the most dangerous privileged-payload changes without any prior knowledge
    /// of the protocol's accounts.
    pub fn standard() -> Self {
        Self::new()
            .rule("upgrade-authority", any_upgrade_authority_rotation())
            .rule("owner-change", any_owner_change())
            .rule("new-nonce-account", no_new_nonce_account())
            .rule("program-freeze", program_freeze_guard())
            .rule("nonce-authority-combo", nonce_authority_combo())
            .rule("large-sol-drain", large_sol_drain(80.0))
    }

    /// Number of rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the policy has no rules.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Evaluate every rule over `(pre, post)` and collect all alarms.
    pub fn evaluate(&self, pre: &[AccountEntry], post: &[AccountEntry]) -> Vec<Alarm> {
        let view = PrePost::new(pre, post);
        self.rules
            .iter()
            .flat_map(|(_, rule)| rule(&view))
            .collect()
    }
}

// --- universal (parameter-free) rules -----------------------------------

/// Flag any account whose upgrade authority changed - the Drift-shaped move,
/// detected across *every* ProgramData account in the closure with no prior
/// registration required.
pub fn any_upgrade_authority_rotation() -> PolicyRule {
    Box::new(|view| {
        let mut alarms = Vec::new();
        for (pre, post) in view.common() {
            let before = fidelity::parse_upgrade_authority(&pre.data);
            let after = fidelity::parse_upgrade_authority(&post.data);
            // Only compare when both sides are well-formed ProgramData records.
            if let (Some(b), Some(a)) = (before, after) {
                if b != a {
                    alarms.push(Alarm {
                        rule: "upgrade-authority-rotated".to_string(),
                        severity: Severity::Critical,
                        account: Some(post.pubkey.to_string()),
                        detail: format!(
                            "upgrade authority changed from {} to {}",
                            b.map(|p| p.to_string()).unwrap_or_else(|| "none".into()),
                            a.map(|p| p.to_string()).unwrap_or_else(|| "none".into()),
                        ),
                    });
                }
            }
        }
        alarms
    })
}

/// Flag any existing account whose `owner` program changed - almost always a
/// takeover (e.g. reassigning a vault to an attacker-controlled program).
pub fn any_owner_change() -> PolicyRule {
    Box::new(|view| {
        let mut alarms = Vec::new();
        for (pre, post) in view.common() {
            if pre.owner != post.owner {
                alarms.push(Alarm {
                    rule: "owner-changed".to_string(),
                    severity: Severity::High,
                    account: Some(post.pubkey.to_string()),
                    detail: format!("account owner changed from {} to {}", pre.owner, post.owner),
                });
            }
        }
        alarms
    })
}

/// Flag any newly created durable-nonce account (a System-owned, 80-byte
/// account) - the primitive behind the Drift blind-signing / replay vector.
pub fn no_new_nonce_account() -> PolicyRule {
    Box::new(|view| {
        view.added()
            .filter(|e| e.owner == SYSTEM_PROGRAM && e.data.len() == NONCE_ACCOUNT_LEN)
            .map(|e| Alarm {
                rule: "new-nonce-account".to_string(),
                severity: Severity::High,
                account: Some(e.pubkey.to_string()),
                detail: "a new durable-nonce account was created (replay/blind-sign vector)"
                    .to_string(),
            })
            .collect()
    })
}

// --- parametric rules ---------------------------------------------------

/// Require a specific account keep at least `floor` lamports (a treasury guard).
pub fn balance_floor(account: Pubkey, floor: u64) -> PolicyRule {
    Box::new(move |view| match view.post(&account) {
        Some(post) if post.lamports < floor => vec![Alarm {
            rule: "balance-below-floor".to_string(),
            severity: Severity::High,
            account: Some(account.to_string()),
            detail: format!("balance {} fell below floor {floor}", post.lamports),
        }],
        _ => Vec::new(),
    })
}

/// Require a specific account keep a specific `owner` (an oracle/config guard).
pub fn owner_unchanged(account: Pubkey, expected_owner: Pubkey) -> PolicyRule {
    Box::new(move |view| match view.post(&account) {
        Some(post) if post.owner != expected_owner => vec![Alarm {
            rule: "owner-unexpected".to_string(),
            severity: Severity::High,
            account: Some(account.to_string()),
            detail: format!("owner is {} (expected {expected_owner})", post.owner),
        }],
        _ => Vec::new(),
    })
}

/// Require a specific byte range of an account's data stay immutable (a config
/// guard - e.g. an authority field or a fee parameter at a known offset).
pub fn config_bytes_immutable(account: Pubkey, range: std::ops::Range<usize>) -> PolicyRule {
    Box::new(move |view| {
        let (Some(pre), Some(post)) = (view.pre(&account), view.post(&account)) else {
            return Vec::new();
        };
        let (Some(a), Some(b)) = (pre.data.get(range.clone()), post.data.get(range.clone())) else {
            return Vec::new();
        };
        if a != b {
            vec![Alarm {
                rule: "config-bytes-mutated".to_string(),
                severity: Severity::High,
                account: Some(account.to_string()),
                detail: format!("protected bytes {range:?} changed"),
            }]
        } else {
            Vec::new()
        }
    })
}

/// Alert when any program's upgrade authority is set to `None` in the payload,
/// permanently freezing the program. While sometimes intentional, this is
/// irreversible and should always be flagged for explicit signer acknowledgement.
pub fn program_freeze_guard() -> PolicyRule {
    Box::new(|view| {
        let mut alarms = Vec::new();
        for (pre, post) in view.common() {
            let before = fidelity::parse_upgrade_authority(&pre.data);
            let after = fidelity::parse_upgrade_authority(&post.data);
            if let (Some(Some(_)), Some(None)) = (before, after) {
                alarms.push(Alarm {
                    rule: "program-frozen".to_string(),
                    severity: Severity::Critical,
                    account: Some(post.pubkey.to_string()),
                    detail: "program upgrade authority set to None — program is now PERMANENTLY immutable".to_string(),
                });
            }
        }
        alarms
    })
}

/// Alert when any single account loses more than `max_drain_percent`% of its
/// SOL balance in one payload. Catches treasury drain attacks where the amount
/// might be within a floor but the percentage is suspicious.
pub fn large_sol_drain(max_drain_percent: f64) -> PolicyRule {
    Box::new(move |view| {
        let mut alarms = Vec::new();
        for (pre, post) in view.common() {
            if pre.lamports > 0 && post.lamports < pre.lamports {
                let drained = pre.lamports - post.lamports;
                let percent = (drained as f64 / pre.lamports as f64) * 100.0;
                if percent > max_drain_percent {
                    alarms.push(Alarm {
                        rule: "large-sol-drain".to_string(),
                        severity: Severity::High,
                        account: Some(post.pubkey.to_string()),
                        detail: format!(
                            "account lost {:.1}% of its SOL ({} → {} lamports, drained {})",
                            percent, pre.lamports, post.lamports, drained
                        ),
                    });
                }
            }
        }
        alarms
    })
}

/// The Drift-shaped compound attack: a new durable-nonce account was created
/// AND an authority (upgrade, owner, or token) was rotated in the same payload.
/// This combination — nonce for replay + authority for control — is the exact
/// pattern behind the $285M blind-signing attack.
pub fn nonce_authority_combo() -> PolicyRule {
    Box::new(|view| {
        let has_new_nonce = view
            .added()
            .any(|e| e.owner == SYSTEM_PROGRAM && e.data.len() == NONCE_ACCOUNT_LEN);

        if !has_new_nonce {
            return Vec::new();
        }

        let has_authority_change = view.common().any(|(pre, post)| {
            // Check upgrade authority rotation.
            if let (Some(before), Some(after)) = (
                fidelity::parse_upgrade_authority(&pre.data),
                fidelity::parse_upgrade_authority(&post.data),
            ) {
                if before != after {
                    return true;
                }
            }
            // Check owner change.
            pre.owner != post.owner
        });

        if has_authority_change {
            vec![Alarm {
                rule: "nonce-authority-combo".to_string(),
                severity: Severity::Critical,
                account: None,
                detail: "CRITICAL: a new durable-nonce account was created AND an authority was \
                         rotated in the same payload — this is the exact Drift blind-signing \
                         attack pattern"
                    .to_string(),
            }]
        } else {
            Vec::new()
        }
    })
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

    /// A ProgramData account body with the given authority.
    fn programdata(pubkey: Pubkey, authority: Pubkey) -> AccountEntry {
        let mut data = vec![3, 0, 0, 0];
        data.extend_from_slice(&7u64.to_le_bytes());
        data.push(1);
        data.extend_from_slice(&authority.to_bytes());
        entry(pubkey, fidelity::BPF_LOADER_UPGRADEABLE, 1_000_000, data)
    }

    #[test]
    fn standard_catches_upgrade_authority_rotation() {
        let pd = Pubkey::new_from_array([9; 32]);
        let pre = vec![programdata(pd, Pubkey::new_from_array([1; 32]))];
        let post = vec![programdata(pd, Pubkey::new_from_array([2; 32]))];
        let alarms = Policy::standard().evaluate(&pre, &post);
        assert!(alarms.iter().any(|a| a.rule == "upgrade-authority-rotated"));
        assert!(alarms.iter().any(|a| a.severity == Severity::Critical));
    }

    #[test]
    fn standard_catches_new_nonce_account() {
        let pre: Vec<AccountEntry> = vec![];
        let nonce = entry(
            Pubkey::new_from_array([4; 32]),
            SYSTEM_PROGRAM,
            2_000_000,
            vec![0u8; NONCE_ACCOUNT_LEN],
        );
        let alarms = Policy::standard().evaluate(&pre, &[nonce]);
        assert!(alarms.iter().any(|a| a.rule == "new-nonce-account"));
    }

    #[test]
    fn standard_catches_owner_change() {
        let pk = Pubkey::new_from_array([5; 32]);
        let pre = vec![entry(pk, Pubkey::new_from_array([1; 32]), 10, vec![])];
        let post = vec![entry(pk, Pubkey::new_from_array([2; 32]), 10, vec![])];
        let alarms = Policy::standard().evaluate(&pre, &post);
        assert!(alarms.iter().any(|a| a.rule == "owner-changed"));
    }

    #[test]
    fn benign_change_trips_nothing() {
        let pk = Pubkey::new_from_array([5; 32]);
        let owner = Pubkey::new_from_array([1; 32]);
        let pre = vec![entry(pk, owner, 100, vec![1, 2, 3])];
        let post = vec![entry(pk, owner, 90, vec![1, 2, 3])]; // just a balance move
        assert!(Policy::standard().evaluate(&pre, &post).is_empty());
    }

    #[test]
    fn balance_floor_guards_a_treasury() {
        let pk = Pubkey::new_from_array([7; 32]);
        let owner = Pubkey::new_from_array([1; 32]);
        let pre = vec![entry(pk, owner, 1_000, vec![])];
        let post = vec![entry(pk, owner, 10, vec![])];
        let alarms = Policy::new()
            .rule("floor", balance_floor(pk, 500))
            .evaluate(&pre, &post);
        assert_eq!(alarms.len(), 1);
        assert_eq!(alarms[0].rule, "balance-below-floor");
    }
}
