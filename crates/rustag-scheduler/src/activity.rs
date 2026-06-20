//! Scheduled activities: what a schedule *does* when it fires.
//!
//! An [`Action`] is a small, serializable description of an on-chain effect.
//! Together with a [`Schedule`](crate::Schedule) it forms a recurring activity
//! that simulates realistic, ongoing usage of a stagenet - periodic swaps,
//! deposits, liquidations, faucet top-ups, etc.
//!
//! Three primitives cover the common cases:
//! - [`Action::Airdrop`] - credit SOL to a wallet (no signer needed).
//! - [`Action::Transfer`] - build + sign a SOL transfer each fire.
//! - [`Action::RawTransaction`] - replay a pre-signed transaction blob. Because
//!   a stagenet's blockhash never expires, a signed transaction can be resubmitted
//!   indefinitely - ideal for replaying a real swap/deposit instruction sequence.

use base64::Engine;
use serde::{Deserialize, Serialize};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::instruction::transfer;
use solana_transaction::versioned::VersionedTransaction;
use solana_transaction::Transaction;

use rustag_core::Stagenet;

use crate::error::{Result, SchedulerError};

const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;

/// A single on-chain effect a schedule performs when it fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Airdrop `sol` SOL to `pubkey`. Unlimited and free - no signer required.
    Airdrop { pubkey: String, sol: f64 },

    /// Sign and send a SOL transfer of `sol` from the wallet identified by the
    /// base58-encoded `secret_key` to `to`.
    ///
    /// `secret_key` is a base58-encoded 64-byte keypair (the Phantom export /
    /// `Keypair::to_base58_string` format). This is a *staging* convenience -
    /// never schedule a mainnet secret here.
    Transfer {
        secret_key: String,
        to: String,
        sol: f64,
    },

    /// Resubmit a base64-encoded, pre-signed transaction every time the schedule
    /// fires. The most flexible action - capture any real instruction sequence
    /// (a Jupiter swap, a lending deposit) once and replay it on a cadence.
    RawTransaction { transaction_base64: String },
}

impl Action {
    /// Execute this action against `sn`, returning the resulting transaction
    /// signature (base58) on success.
    pub async fn execute(&self, sn: &mut Stagenet) -> Result<Option<String>> {
        match self {
            Action::Airdrop { pubkey, sol } => {
                let pk = parse_pubkey(pubkey)?;
                let sig = sn.airdrop_with_record(&pk, sol_to_lamports(*sol)).await?;
                Ok(Some(sig.to_string()))
            }
            Action::Transfer {
                secret_key,
                to,
                sol,
            } => {
                let keypair = parse_keypair(secret_key)?;
                let to_pk = parse_pubkey(to)?;
                let blockhash = sn.latest_blockhash();
                let ix = transfer(&keypair.pubkey(), &to_pk, sol_to_lamports(*sol));
                let msg = Message::new(&[ix], Some(&keypair.pubkey()));
                let tx: VersionedTransaction = Transaction::new(&[&keypair], msg, blockhash).into();
                let outcome = sn.send_transaction(tx).await?;
                Ok(Some(outcome.signature_string()))
            }
            Action::RawTransaction { transaction_base64 } => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(transaction_base64.trim())
                    .map_err(|e| SchedulerError::Action(format!("base64 decode: {e}")))?;
                let tx: VersionedTransaction = bincode::deserialize(&bytes)
                    .map_err(|e| SchedulerError::Action(format!("deserialize transaction: {e}")))?;
                let outcome = sn.send_transaction(tx).await?;
                Ok(Some(outcome.signature_string()))
            }
        }
    }

    /// Validate that this action is well-formed without executing it (used by the
    /// create path so invalid activities are rejected before being persisted).
    pub fn validate(&self) -> Result<()> {
        match self {
            Action::Airdrop { pubkey, .. } => {
                parse_pubkey(pubkey)?;
            }
            Action::Transfer { secret_key, to, .. } => {
                parse_keypair(secret_key)?;
                parse_pubkey(to)?;
            }
            Action::RawTransaction { transaction_base64 } => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(transaction_base64.trim())
                    .map_err(|e| SchedulerError::Action(format!("base64 decode: {e}")))?;
                bincode::deserialize::<VersionedTransaction>(&bytes)
                    .map_err(|e| SchedulerError::Action(format!("deserialize transaction: {e}")))?;
            }
        }
        Ok(())
    }
}

fn sol_to_lamports(sol: f64) -> u64 {
    (sol.max(0.0) * LAMPORTS_PER_SOL) as u64
}

fn parse_pubkey(s: &str) -> Result<Pubkey> {
    s.trim()
        .parse::<Pubkey>()
        .map_err(|_| SchedulerError::Action(format!("invalid pubkey: {s}")))
}

fn parse_keypair(secret_base58: &str) -> Result<Keypair> {
    Keypair::try_from_base58_string(secret_base58.trim())
        .map_err(|e| SchedulerError::Action(format!("invalid secret key: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_json_roundtrip() {
        let a = Action::Airdrop {
            pubkey: "So11111111111111111111111111111111111111112".into(),
            sol: 1.5,
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("\"type\":\"airdrop\""));
        let back: Action = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, Action::Airdrop { sol, .. } if sol == 1.5));
        a.validate().unwrap();
    }

    #[tokio::test]
    async fn airdrop_action_executes() {
        let mut sn = Stagenet::local("sched-action").await.unwrap();
        let wallet = Pubkey::new_unique();
        let action = Action::Airdrop {
            pubkey: wallet.to_string(),
            sol: 2.0,
        };
        let sig = action.execute(&mut sn).await.unwrap();
        assert!(sig.is_some());
        assert_eq!(sn.get_balance(&wallet).await.unwrap(), 2_000_000_000);
    }

    #[test]
    fn invalid_action_rejected() {
        let bad = Action::Airdrop {
            pubkey: "not-a-pubkey".into(),
            sol: 1.0,
        };
        assert!(bad.validate().is_err());
    }
}
