//! Property-based invariant fuzzing.
//!
//! A fuzz run drives a deterministic, seeded sequence of transactions against an
//! isolated fork and, after each one, checks a set of caller-supplied
//! [`Invariant`]s — properties that must hold no matter what the program does
//! ("the vault's owner never changes", "the treasury never drops below X").
//! Any violation is captured together with the seed needed to reproduce it, so
//! a failing run is always replayable.

use std::collections::BTreeMap;

use solana_pubkey::Pubkey;
use solana_transaction::versioned::VersionedTransaction;

use rustag_core::{AccountEntry, Stagenet, TxOutcome};

use crate::error::Result;

/// A small, deterministic xorshift PRNG so fuzz runs are fully reproducible from
/// a seed (no `rand`, no entropy, no platform variance).
#[derive(Debug, Clone)]
pub struct FuzzRng {
    state: u64,
}

impl FuzzRng {
    /// Seed the RNG. A zero seed is mapped to a fixed non-zero constant so the
    /// generator never degenerates.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed },
        }
    }

    /// Next 64-bit value (xorshift64*).
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// A value in `[lo, hi)`. Returns `lo` if the range is empty.
    pub fn gen_range(&mut self, lo: u64, hi: u64) -> u64 {
        if hi <= lo {
            return lo;
        }
        lo + self.next_u64() % (hi - lo)
    }

    /// Choose a random element, or `None` if `items` is empty.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            let i = (self.next_u64() % items.len() as u64) as usize;
            Some(&items[i])
        }
    }
}

/// A snapshot of the tracked state, handed to each invariant after a step.
#[derive(Debug, Clone)]
pub struct FuzzObservation {
    /// Step index (0-based).
    pub step: usize,
    /// The run's seed (for reproduction).
    pub seed: u64,
    /// Outcome of the transaction at this step.
    pub last_outcome: TxOutcome,
    /// Current state of each tracked account (`None` if it does not exist).
    pub accounts: BTreeMap<Pubkey, Option<AccountEntry>>,
}

impl FuzzObservation {
    /// Sum of lamports across the tracked accounts (a conservation check input).
    pub fn total_tracked_lamports(&self) -> u128 {
        self.accounts
            .values()
            .filter_map(|a| a.as_ref().map(|e| e.lamports as u128))
            .sum()
    }

    /// The tracked account's current entry, if present.
    pub fn account(&self, pubkey: &Pubkey) -> Option<&AccountEntry> {
        self.accounts.get(pubkey).and_then(|a| a.as_ref())
    }
}

/// A property that must hold after every step. Return `Err(message)` to flag a
/// violation.
pub type Invariant = Box<dyn Fn(&FuzzObservation) -> std::result::Result<(), String> + Send + Sync>;

/// A recorded invariant violation.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    /// Step at which it occurred.
    pub step: usize,
    /// The run's seed (reproduces the exact sequence).
    pub seed: u64,
    /// Which invariant was violated.
    pub invariant: String,
    /// Human-readable detail from the invariant.
    pub message: String,
    /// Signature of the transaction at the violating step.
    pub signature: String,
}

/// The result of a fuzz run.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FuzzReport {
    /// Run label.
    pub label: String,
    /// Seed used.
    pub seed: u64,
    /// Number of steps actually executed.
    pub steps_run: usize,
    /// Number of transactions that succeeded.
    pub succeeded: usize,
    /// Violations found, in order.
    pub violations: Vec<Violation>,
}

impl FuzzReport {
    /// Whether no invariant was ever violated.
    pub fn passed(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Run a seeded invariant-fuzz campaign of `steps` transactions against `fork`.
///
/// `build(step, rng)` produces the transaction for each step (use `rng` so the
/// sequence is reproducible). After each step, every named invariant in
/// `invariants` is evaluated against the tracked accounts. If `stop_on_violation`
/// is set, the run halts at the first violation.
// A fuzz campaign genuinely has many orthogonal knobs (fork, label, steps, seed,
// tracked set, invariants, stop policy, tx builder); folding them into a config
// struct would hurt readability more than it helps here.
#[allow(clippy::too_many_arguments)]
pub async fn fuzz<F>(
    fork: &mut Stagenet,
    label: impl Into<String>,
    steps: usize,
    seed: u64,
    tracked: &[Pubkey],
    invariants: &[(&str, Invariant)],
    stop_on_violation: bool,
    mut build: F,
) -> Result<FuzzReport>
where
    F: FnMut(usize, &mut FuzzRng) -> VersionedTransaction,
{
    let label = label.into();
    let mut rng = FuzzRng::new(seed);
    let mut violations = Vec::new();
    let mut succeeded = 0;
    let mut steps_run = 0;

    for step in 0..steps {
        let tx = build(step, &mut rng);
        let outcome = fork.send_transaction(tx).await?;
        if outcome.success {
            succeeded += 1;
        }
        steps_run += 1;

        // Gather the tracked accounts for this observation.
        let mut accounts = BTreeMap::new();
        for pk in tracked {
            accounts.insert(*pk, fork.get_account(pk).await?);
        }
        let obs = FuzzObservation {
            step,
            seed,
            last_outcome: outcome,
            accounts,
        };

        let mut violated = false;
        for (name, invariant) in invariants {
            if let Err(message) = invariant(&obs) {
                violations.push(Violation {
                    step,
                    seed,
                    invariant: (*name).to_string(),
                    message,
                    signature: obs.last_outcome.signature_string(),
                });
                violated = true;
            }
        }
        if violated && stop_on_violation {
            break;
        }
    }

    Ok(FuzzReport {
        label,
        seed,
        steps_run,
        succeeded,
        violations,
    })
}

/// Built-in invariant: the account at `pubkey` must keep `expected_owner`.
pub fn owner_unchanged(pubkey: Pubkey, expected_owner: Pubkey) -> Invariant {
    Box::new(move |obs| match obs.account(&pubkey) {
        Some(entry) if entry.owner != expected_owner => Err(format!(
            "account {pubkey} owner changed to {} (expected {expected_owner})",
            entry.owner
        )),
        _ => Ok(()),
    })
}

/// Built-in invariant: the account at `pubkey` must hold at least `floor`
/// lamports (once it exists).
pub fn balance_floor(pubkey: Pubkey, floor: u64) -> Invariant {
    Box::new(move |obs| match obs.account(&pubkey) {
        Some(entry) if entry.lamports < floor => Err(format!(
            "account {pubkey} balance {} fell below floor {floor}",
            entry.lamports
        )),
        _ => Ok(()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_signer::Signer;
    use solana_system_interface::instruction::transfer;
    use solana_transaction::Transaction;

    #[test]
    fn rng_is_deterministic() {
        let a: Vec<u64> = (0..5)
            .scan(FuzzRng::new(42), |r, _| Some(r.next_u64()))
            .collect();
        let b: Vec<u64> = (0..5)
            .scan(FuzzRng::new(42), |r, _| Some(r.next_u64()))
            .collect();
        assert_eq!(a, b);
        let c: Vec<u64> = (0..5)
            .scan(FuzzRng::new(43), |r, _| Some(r.next_u64()))
            .collect();
        assert_ne!(a, c);
    }

    #[tokio::test]
    async fn invariant_holds_for_safe_transfers() {
        let mut base = Stagenet::local("fuzz-ok").await.unwrap();
        let payer = Keypair::new();
        base.airdrop(&payer.pubkey(), 100_000_000_000)
            .await
            .unwrap();
        let bh = base.latest_blockhash();
        let mut fork = base.fork("fuzz-ok-fork").await.unwrap();

        // The payer keeps a comfortable floor; small random transfers never
        // breach it.
        let invariants: Vec<(&str, Invariant)> =
            vec![("payer-floor", balance_floor(payer.pubkey(), 1_000_000_000))];
        let report = fuzz(
            &mut fork,
            "safe",
            20,
            7,
            &[payer.pubkey()],
            &invariants,
            true,
            |_, rng| {
                let amount = rng.gen_range(1, 1_000_000);
                let to = Keypair::new().pubkey();
                let ix = transfer(&payer.pubkey(), &to, amount);
                let msg = Message::new(&[ix], Some(&payer.pubkey()));
                Transaction::new(&[&payer], msg, bh).into()
            },
        )
        .await
        .unwrap();
        assert!(report.passed(), "{report:?}");
        assert_eq!(report.steps_run, 20);
    }

    #[tokio::test]
    async fn invariant_violation_is_caught_and_reproducible() {
        let mut base = Stagenet::local("fuzz-bad").await.unwrap();
        let payer = Keypair::new();
        base.airdrop(&payer.pubkey(), 5_000_000_000).await.unwrap();
        let bh = base.latest_blockhash();
        let mut fork = base.fork("fuzz-bad-fork").await.unwrap();

        // Floor set absurdly high: the very first transfer drops the payer
        // below it, so the run must flag a violation and stop.
        let invariants: Vec<(&str, Invariant)> = vec![(
            "impossible-floor",
            balance_floor(payer.pubkey(), 4_999_000_000),
        )];
        let report = fuzz(
            &mut fork,
            "drain",
            10,
            1,
            &[payer.pubkey()],
            &invariants,
            true,
            |_, _| {
                let to = Keypair::new().pubkey();
                let ix = transfer(&payer.pubkey(), &to, 2_000_000_000);
                let msg = Message::new(&[ix], Some(&payer.pubkey()));
                Transaction::new(&[&payer], msg, bh).into()
            },
        )
        .await
        .unwrap();
        assert!(!report.passed());
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].invariant, "impossible-floor");
        assert_eq!(report.violations[0].seed, 1);
    }
}
