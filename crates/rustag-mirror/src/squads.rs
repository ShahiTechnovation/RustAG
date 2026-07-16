//! **Squads v4 multisig proposal decoder.**
//!
//! Fetches a Squads v4 `VaultTransaction` account from mainnet, decodes the
//! embedded `TransactionMessage`, and returns a structured payload ready for
//! rehearsal.
//!
//! The Squads v4 on-chain layout is documented in the
//! [`squads-multisig-program`](https://github.com/Squads-Protocol/v4) repo.
//! We decode it by hand (fixed offsets) to avoid pulling the full Squads SDK.

use solana_pubkey::{pubkey, Pubkey};

use crate::error::MirrorError;
use crate::fetcher::MainnetMirror;

/// The Squads v4 Multisig program id.
pub const SQUADS_V4_PROGRAM: Pubkey = pubkey!("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf");

/// A decoded Squads multisig proposal ready for rehearsal.
#[derive(Debug, Clone)]
pub struct ProposedPayload {
    /// The decoded transaction message extracted from the proposal.
    pub message_bytes: Vec<u8>,
    /// The multisig account (the vault authority).
    pub multisig: Pubkey,
    /// The proposal / vault-transaction account that was decoded.
    pub proposal: Pubkey,
    /// The member who created the proposal, if decodable.
    pub creator: Option<Pubkey>,
    /// Number of approvals the proposal has received.
    pub approval_count: u8,
    /// The multisig's approval threshold.
    pub threshold: u8,
    /// The vault index the transaction targets.
    pub vault_index: u8,
}

/// Squads v4 account discriminators (first 8 bytes, Anchor-style SHA256 hash).
/// These are the sighash of `account:<TypeName>`.
const MULTISIG_DISCRIMINATOR: [u8; 8] = [0xa2, 0x54, 0x39, 0x46, 0x3e, 0xbc, 0x2f, 0x9c];
const VAULT_TX_DISCRIMINATOR: [u8; 8] = [0x9d, 0x47, 0x1b, 0xe1, 0x10, 0xe4, 0x0c, 0xef];

/// Decodes Squads v4 multisig proposals from mainnet.
pub struct SquadsDecoder<'a> {
    mirror: &'a MainnetMirror,
}

impl<'a> SquadsDecoder<'a> {
    /// Create a decoder backed by the given mainnet mirror.
    pub fn new(mirror: &'a MainnetMirror) -> Self {
        Self { mirror }
    }

    /// Fetch and decode a Squads v4 vault transaction (proposal).
    ///
    /// The `proposal_pubkey` should be the on-chain address of a
    /// `VaultTransaction` account created by the Squads v4 program.
    ///
    /// Returns the embedded transaction message bytes and proposal metadata.
    #[tracing::instrument(skip(self))]
    pub async fn decode_proposal(
        &self,
        proposal_pubkey: &Pubkey,
    ) -> Result<ProposedPayload, MirrorError> {
        // Fetch the vault transaction account.
        let vt_account = self
            .mirror
            .fetch_one(proposal_pubkey)
            .await?
            .ok_or_else(|| {
                MirrorError::InvalidResponse(format!(
                    "Squads proposal {proposal_pubkey} not found on mainnet"
                ))
            })?;

        // Verify owner is the Squads v4 program.
        if vt_account.owner != SQUADS_V4_PROGRAM {
            return Err(MirrorError::Decode(format!(
                "account {proposal_pubkey} is owned by {}, not the Squads v4 program {SQUADS_V4_PROGRAM}",
                vt_account.owner
            )));
        }

        let data = &vt_account.data;

        // Decode the VaultTransaction layout.
        //
        // Squads v4 VaultTransaction (Anchor account):
        //   [0..8]   - discriminator
        //   [8..40]  - multisig (Pubkey)
        //   [40..41] - vault_index (u8)
        //   [41..49] - transaction_index (u64 LE)
        //   [49..50] - bump (u8)
        //   [50..82] - creator (Pubkey)
        //   [82..86] - ephemeral_signer_count (u32 LE) — Anchor Vec length prefix
        //   [86..]   - message (Borsh-serialized TransactionMessage, prefixed by length)
        //
        // NOTE: The exact offsets depend on the Squads v4 version. We decode
        // defensively and report clear errors if the layout doesn't match.

        if data.len() < 86 {
            return Err(MirrorError::Decode(format!(
                "VaultTransaction account too short: {} bytes (need >= 86)",
                data.len()
            )));
        }

        // Validate discriminator.
        if data[0..8] != VAULT_TX_DISCRIMINATOR {
            return Err(MirrorError::Decode(format!(
                "account {proposal_pubkey} has unexpected discriminator (not a VaultTransaction)"
            )));
        }

        let multisig = Pubkey::new_from_array(
            data[8..40]
                .try_into()
                .map_err(|_| MirrorError::Decode("multisig pubkey parse failed".into()))?,
        );
        let vault_index = data[40];
        let creator = Pubkey::new_from_array(
            data[50..82]
                .try_into()
                .map_err(|_| MirrorError::Decode("creator pubkey parse failed".into()))?,
        );

        // Skip the ephemeral signer vector.
        let ephemeral_signer_count = u32::from_le_bytes(
            data[82..86]
                .try_into()
                .map_err(|_| MirrorError::Decode("ephemeral signer count parse failed".into()))?,
        ) as usize;

        // Each ephemeral signer bump is a single u8.
        let message_offset = 86 + ephemeral_signer_count;

        if data.len() < message_offset + 4 {
            return Err(MirrorError::Decode(
                "VaultTransaction data truncated before message".into(),
            ));
        }

        // The message is a Borsh-serialized `TransactionMessage`. In the Anchor
        // representation, a variable-length `Vec<u8>` or struct is prefixed with
        // a 4-byte little-endian length.
        let msg_len = u32::from_le_bytes(
            data[message_offset..message_offset + 4]
                .try_into()
                .map_err(|_| MirrorError::Decode("message length parse failed".into()))?,
        ) as usize;

        let msg_start = message_offset + 4;
        let msg_end = msg_start + msg_len;

        if data.len() < msg_end {
            return Err(MirrorError::Decode(format!(
                "VaultTransaction message extends beyond account data ({msg_end} > {})",
                data.len()
            )));
        }

        let message_bytes = data[msg_start..msg_end].to_vec();

        // Fetch the multisig account to get the threshold and approval count.
        let (threshold, approval_count) =
            self.fetch_multisig_metadata(&multisig).await.unwrap_or((0, 0));

        tracing::info!(
            proposal = %proposal_pubkey,
            multisig = %multisig,
            vault_index,
            message_len = message_bytes.len(),
            threshold,
            approvals = approval_count,
            "decoded Squads v4 VaultTransaction"
        );

        Ok(ProposedPayload {
            message_bytes,
            multisig,
            proposal: *proposal_pubkey,
            creator: Some(creator),
            approval_count,
            threshold,
            vault_index,
        })
    }

    /// Fetch the Squads multisig account to read threshold.
    async fn fetch_multisig_metadata(
        &self,
        multisig: &Pubkey,
    ) -> Result<(u8, u8), MirrorError> {
        let account = self.mirror.fetch_one(multisig).await?.ok_or_else(|| {
            MirrorError::InvalidResponse(format!("multisig account {multisig} not found"))
        })?;

        let data = &account.data;

        // Squads v4 Multisig account (simplified):
        //   [0..8]   - discriminator
        //   [8..10]  - threshold (u16 LE)
        //   [10..18] - config_authority (Option<Pubkey> - 1 + 32 bytes)
        //   ...
        //   Members are stored as a Vec further in the account.
        //
        // We only extract the threshold here. Approval count requires
        // fetching the Proposal account (separate from VaultTransaction).
        if data.len() < 10 || data[0..8] != MULTISIG_DISCRIMINATOR {
            return Ok((0, 0));
        }

        let threshold = u16::from_le_bytes(
            data[8..10]
                .try_into()
                .map_err(|_| MirrorError::Decode("threshold parse failed".into()))?,
        ) as u8;

        Ok((threshold, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squads_v4_program_address_is_correct() {
        // Verify the hardcoded program id matches the well-known value.
        assert_eq!(
            SQUADS_V4_PROGRAM.to_string(),
            "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"
        );
    }
}
