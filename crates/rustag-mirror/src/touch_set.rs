//! **TouchSet resolver**: given a proposed `VersionedMessage`, compute the
//! complete set of accounts (the *closure*) the transaction will touch at
//! runtime — including accounts that are not statically listed in the message
//! but are required for faithful execution.
//!
//! The static account keys of a Solana message are necessary but not sufficient:
//!
//! 1. **v0 Address Lookup Tables.** The message may reference lookup table (LUT)
//!    accounts whose on-chain data contains additional pubkeys.
//! 2. **ProgramData PDAs.** An upgradeable program's executable ELF does not live
//!    in the program account — it lives in a `ProgramData` PDA. Without
//!    following that pointer the program cannot execute and its upgrade authority
//!    (the single most security-relevant field) is invisible.
//! 3. **Clock sysvar.** Time-dependent logic (vesting, staleness, auctions) reads
//!    the Clock sysvar; rehearsal fidelity requires it in the closure.

use std::collections::HashSet;

use solana_pubkey::{pubkey, Pubkey};

use crate::error::MirrorError;
use crate::fetcher::MainnetMirror;

/// The Clock sysvar pubkey.
const CLOCK_SYSVAR: Pubkey = pubkey!("SysvarC1ock11111111111111111111111111111111");

/// The BPF upgradeable loader program id.
const BPF_LOADER_UPGRADEABLE: Pubkey = pubkey!("BPFLoaderUpgradeab1e11111111111111111111111");

/// Tag byte for `UpgradeableLoaderState::Program`.
const TAG_PROGRAM: [u8; 4] = [2, 0, 0, 0];
/// Serialized size of a `Program` account: 4-byte tag + 32-byte pubkey.
const PROGRAM_LEN: usize = 4 + 32;

/// Serialized size of an Address Lookup Table header before the addresses begin.
/// u32 deactivation_slot_placeholder (unused, always exists) = 4 bytes,
/// but the real layout is: 4 (discriminator) + 8 (deactivation_slot) + 8 (last_extended_slot)
/// + 1 (last_extended_slot_start_index) + 1 (has_authority) + 32 (authority) + 2 (padding) = 56
const ALT_HEADER_LEN: usize = 56;

/// Resolves the full account closure a `VersionedMessage` will touch at runtime.
pub struct TouchSetResolver<'a> {
    mirror: &'a MainnetMirror,
}

impl<'a> TouchSetResolver<'a> {
    /// Create a resolver backed by the given mainnet mirror.
    pub fn new(mirror: &'a MainnetMirror) -> Self {
        Self { mirror }
    }

    /// Resolve the full closure of accounts a message will touch.
    ///
    /// Returns a deduplicated, sorted list of pubkeys including:
    /// - All static account keys from the message
    /// - All keys resolved from v0 Address Lookup Tables
    /// - ProgramData PDAs for any upgradeable programs invoked
    /// - The Clock sysvar
    #[tracing::instrument(skip(self, static_keys), fields(static_count = static_keys.len()))]
    pub async fn resolve(
        &self,
        static_keys: &[Pubkey],
        address_table_lookups: &[AddressTableLookup],
    ) -> Result<Vec<Pubkey>, MirrorError> {
        let mut closure: HashSet<Pubkey> = static_keys.iter().copied().collect();

        // 1. Resolve v0 Address Lookup Table keys.
        for lookup in address_table_lookups {
            let resolved = self.resolve_alt(&lookup.table_key, lookup).await?;
            closure.extend(resolved);
        }

        // 2. For each program invoked, if it is an upgradeable program, resolve
        //    its ProgramData PDA so the ELF and upgrade authority are visible.
        let programs: Vec<Pubkey> = closure.iter().copied().collect();
        for program_id in &programs {
            if let Some(programdata_address) =
                self.resolve_programdata(program_id).await?
            {
                tracing::debug!(
                    program = %program_id,
                    programdata = %programdata_address,
                    "resolved ProgramData PDA for upgradeable program"
                );
                closure.insert(programdata_address);
            }
        }

        // 3. Always include the Clock sysvar.
        closure.insert(CLOCK_SYSVAR);

        let mut result: Vec<Pubkey> = closure.into_iter().collect();
        result.sort_by_key(|k| k.to_bytes());
        Ok(result)
    }

    /// Resolve the pubkeys referenced by a v0 Address Lookup Table.
    async fn resolve_alt(
        &self,
        table_key: &Pubkey,
        lookup: &AddressTableLookup,
    ) -> Result<Vec<Pubkey>, MirrorError> {
        let account = self.mirror.fetch_one(table_key).await?.ok_or_else(|| {
            MirrorError::InvalidResponse(format!(
                "address lookup table {table_key} not found on mainnet"
            ))
        })?;

        let addresses = parse_alt_addresses(&account.data)?;

        let mut resolved = Vec::new();
        for &index in lookup
            .writable_indexes
            .iter()
            .chain(lookup.readonly_indexes.iter())
        {
            let idx = index as usize;
            if idx >= addresses.len() {
                return Err(MirrorError::InvalidResponse(format!(
                    "ALT {table_key} index {idx} out of range (table has {} entries)",
                    addresses.len()
                )));
            }
            resolved.push(addresses[idx]);
        }

        tracing::debug!(
            table = %table_key,
            resolved_count = resolved.len(),
            "resolved v0 Address Lookup Table"
        );
        Ok(resolved)
    }

    /// If `program_id` is an upgradeable program, fetch its account and resolve
    /// the ProgramData address. Returns `None` if the program is not upgradeable
    /// or does not exist.
    async fn resolve_programdata(
        &self,
        program_id: &Pubkey,
    ) -> Result<Option<Pubkey>, MirrorError> {
        let account = match self.mirror.fetch_one(program_id).await? {
            Some(a) => a,
            None => return Ok(None),
        };

        if account.owner != BPF_LOADER_UPGRADEABLE {
            return Ok(None);
        }

        // Parse the Program account to find its ProgramData address.
        if account.data.len() >= PROGRAM_LEN && account.data[0..4] == TAG_PROGRAM {
            let bytes: [u8; 32] = account.data[4..PROGRAM_LEN]
                .try_into()
                .map_err(|_| MirrorError::Decode("ProgramData address parse failed".into()))?;
            return Ok(Some(Pubkey::new_from_array(bytes)));
        }

        Ok(None)
    }
}

/// A simplified address table lookup descriptor (mirrors the Solana SDK type).
#[derive(Debug, Clone)]
pub struct AddressTableLookup {
    /// The pubkey of the on-chain address lookup table account.
    pub table_key: Pubkey,
    /// Indexes into the table for writable accounts.
    pub writable_indexes: Vec<u8>,
    /// Indexes into the table for readonly accounts.
    pub readonly_indexes: Vec<u8>,
}

/// Parse the list of addresses from an Address Lookup Table account's data.
fn parse_alt_addresses(data: &[u8]) -> Result<Vec<Pubkey>, MirrorError> {
    if data.len() < ALT_HEADER_LEN {
        return Err(MirrorError::Decode(format!(
            "ALT data too short: {} bytes (need at least {ALT_HEADER_LEN})",
            data.len()
        )));
    }

    let address_data = &data[ALT_HEADER_LEN..];
    if address_data.len() % 32 != 0 {
        return Err(MirrorError::Decode(format!(
            "ALT address data length {} is not a multiple of 32",
            address_data.len()
        )));
    }

    let addresses: Vec<Pubkey> = address_data
        .chunks_exact(32)
        .map(|chunk| {
            let bytes: [u8; 32] = chunk.try_into().expect("chunk is exactly 32 bytes");
            Pubkey::new_from_array(bytes)
        })
        .collect();

    Ok(addresses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_alt_addresses_empty_table() {
        // Header + 0 addresses.
        let data = vec![0u8; ALT_HEADER_LEN];
        let addrs = parse_alt_addresses(&data).unwrap();
        assert!(addrs.is_empty());
    }

    #[test]
    fn parse_alt_addresses_two_entries() {
        let mut data = vec![0u8; ALT_HEADER_LEN];
        let key1 = Pubkey::new_from_array([1; 32]);
        let key2 = Pubkey::new_from_array([2; 32]);
        data.extend_from_slice(&key1.to_bytes());
        data.extend_from_slice(&key2.to_bytes());
        let addrs = parse_alt_addresses(&data).unwrap();
        assert_eq!(addrs, vec![key1, key2]);
    }

    #[test]
    fn parse_alt_addresses_rejects_short_data() {
        let data = vec![0u8; 10];
        assert!(parse_alt_addresses(&data).is_err());
    }

    #[test]
    fn parse_alt_addresses_rejects_misaligned_data() {
        let mut data = vec![0u8; ALT_HEADER_LEN];
        data.extend_from_slice(&[0u8; 17]); // not a multiple of 32
        assert!(parse_alt_addresses(&data).is_err());
    }
}
