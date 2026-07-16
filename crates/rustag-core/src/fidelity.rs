//! Execution-fidelity helpers: the two things a *pre-execution rehearsal* needs
//! that a plain LiteSVM fork does not have.
//!
//! 1. **ProgramData dereference.** An upgradeable program's executable ELF does
//!    not live in the program account - it lives in a separate `ProgramData`
//!    account referenced by it. Without following that pointer, a real mainnet
//!    program cannot execute in the sandbox, and its *upgrade authority* (the
//!    single most security-relevant field a privileged transaction can touch)
//!    is invisible. [`parse_programdata_address`], [`parse_upgrade_authority`],
//!    and [`programdata_elf`] decode the on-chain `UpgradeableLoaderState`
//!    layout so [`Stagenet::load_upgradeable_program`](crate::Stagenet::load_upgradeable_program)
//!    can wire the ELF into the runtime.
//! 2. **Clock sync.** Vesting, funding windows, staleness checks, and auction
//!    logic read the `Clock` sysvar. [`Stagenet::sync_clock`](crate::Stagenet::sync_clock)
//!    pins it at the target slot / `blockTime` so time-dependent logic rehearses
//!    faithfully.
//!
//! The `UpgradeableLoaderState` layout is decoded by hand (fixed offsets) rather
//! than by pulling `solana-loader-v3-interface`, whose major line does not unify
//! with the SDK crates `litesvm` 0.12 pins - keeping the dependency tree coherent.

use solana_pubkey::{pubkey, Pubkey};

/// The BPF upgradeable loader program id. An account owned by this loader is an
/// upgradeable program whose bytecode lives in a separate `ProgramData` account.
pub const BPF_LOADER_UPGRADEABLE: Pubkey = pubkey!("BPFLoaderUpgradeab1e11111111111111111111111");

/// `bincode` enum discriminant (u32 LE) for `UpgradeableLoaderState::Program`.
const TAG_PROGRAM: [u8; 4] = [2, 0, 0, 0];
/// `bincode` enum discriminant (u32 LE) for `UpgradeableLoaderState::ProgramData`.
const TAG_PROGRAM_DATA: [u8; 4] = [3, 0, 0, 0];

/// Serialized size of a `Program` account: 4-byte tag + 32-byte pubkey.
const PROGRAM_LEN: usize = 4 + 32;
/// Serialized size of the `ProgramData` metadata header that precedes the ELF:
/// 4-byte tag + 8-byte slot + 1-byte `Option` tag + 32-byte authority pubkey.
pub const PROGRAMDATA_HEADER_LEN: usize = 4 + 8 + 1 + 32;
/// Byte offset of the `Option<Pubkey>` presence tag inside a `ProgramData` account.
const AUTHORITY_OPTION_OFFSET: usize = 12;
/// Byte offset of the upgrade-authority pubkey inside a `ProgramData` account
/// (immediately after the 1-byte `Option` presence tag).
pub const UPGRADE_AUTHORITY_OFFSET: usize = 13;

/// Given a `Program` account's data, return the `ProgramData` account address it
/// points at, or `None` if the bytes are not a well-formed `Program` record.
pub fn parse_programdata_address(program_account_data: &[u8]) -> Option<Pubkey> {
    let data = program_account_data;
    if data.len() < PROGRAM_LEN || data[0..4] != TAG_PROGRAM {
        return None;
    }
    let bytes: [u8; 32] = data[4..PROGRAM_LEN].try_into().ok()?;
    Some(Pubkey::new_from_array(bytes))
}

/// Decode the upgrade authority from a `ProgramData` account's data.
///
/// Returns:
/// - `None` - the bytes are not a well-formed `ProgramData` record.
/// - `Some(None)` - a well-formed record with **no** authority (immutable).
/// - `Some(Some(pk))` - the current upgrade authority.
pub fn parse_upgrade_authority(programdata: &[u8]) -> Option<Option<Pubkey>> {
    let data = programdata;
    if data.len() < PROGRAMDATA_HEADER_LEN || data[0..4] != TAG_PROGRAM_DATA {
        return None;
    }
    match data[AUTHORITY_OPTION_OFFSET] {
        0 => Some(None),
        1 => {
            let bytes: [u8; 32] = data[UPGRADE_AUTHORITY_OFFSET..PROGRAMDATA_HEADER_LEN]
                .try_into()
                .ok()?;
            Some(Some(Pubkey::new_from_array(bytes)))
        }
        _ => None,
    }
}

/// The executable ELF bytes carried by a `ProgramData` account (everything after
/// the fixed metadata header), or an empty slice if the record is malformed or
/// header-only.
pub fn programdata_elf(programdata: &[u8]) -> &[u8] {
    if programdata.len() > PROGRAMDATA_HEADER_LEN && programdata[0..4] == TAG_PROGRAM_DATA {
        &programdata[PROGRAMDATA_HEADER_LEN..]
    } else {
        &[]
    }
}

/// Whether an account owner marks it as an upgradeable-loader program account.
pub fn is_upgradeable_program(owner: &Pubkey) -> bool {
    *owner == BPF_LOADER_UPGRADEABLE
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic `Program` account body pointing at `pd`.
    fn program_body(pd: Pubkey) -> Vec<u8> {
        let mut v = TAG_PROGRAM.to_vec();
        v.extend_from_slice(&pd.to_bytes());
        v
    }

    /// Build a synthetic `ProgramData` account body with an optional authority
    /// and trailing `elf` bytes.
    fn programdata_body(authority: Option<Pubkey>, elf: &[u8]) -> Vec<u8> {
        let mut v = TAG_PROGRAM_DATA.to_vec();
        v.extend_from_slice(&7u64.to_le_bytes()); // slot
        match authority {
            Some(pk) => {
                v.push(1);
                v.extend_from_slice(&pk.to_bytes());
            }
            None => {
                v.push(0);
                v.extend_from_slice(&[0u8; 32]); // fixed-width padding, ignored
            }
        }
        v.extend_from_slice(elf);
        v
    }

    #[test]
    fn programdata_address_roundtrips() {
        let pd = Pubkey::new_from_array([9; 32]);
        assert_eq!(parse_programdata_address(&program_body(pd)), Some(pd));
    }

    #[test]
    fn non_program_bytes_yield_none() {
        assert_eq!(parse_programdata_address(&[0u8; 10]), None);
        assert_eq!(parse_programdata_address(&[3, 0, 0, 0]), None);
    }

    #[test]
    fn upgrade_authority_present_and_absent() {
        let auth = Pubkey::new_from_array([5; 32]);
        assert_eq!(
            parse_upgrade_authority(&programdata_body(Some(auth), b"")),
            Some(Some(auth))
        );
        assert_eq!(
            parse_upgrade_authority(&programdata_body(None, b"")),
            Some(None)
        );
        assert_eq!(parse_upgrade_authority(&[0u8; 10]), None);
    }

    #[test]
    fn elf_is_the_tail_after_the_header() {
        let body = programdata_body(Some(Pubkey::new_from_array([1; 32])), b"ELFBYTES");
        assert_eq!(programdata_elf(&body), b"ELFBYTES");
        // Header-only record has no ELF.
        assert!(programdata_elf(&programdata_body(None, b"")).is_empty());
    }

    #[test]
    fn upgrade_authority_offset_matches_the_header() {
        // A rotated authority must be detectable purely from the fixed offset -
        // this is the invariant the pre-sign alarm relies on.
        let before = Pubkey::new_from_array([1; 32]);
        let after = Pubkey::new_from_array([2; 32]);
        let a = programdata_body(Some(before), b"x");
        let b = programdata_body(Some(after), b"x");
        assert_ne!(
            &a[UPGRADE_AUTHORITY_OFFSET..PROGRAMDATA_HEADER_LEN],
            &b[UPGRADE_AUTHORITY_OFFSET..PROGRAMDATA_HEADER_LEN]
        );
    }
}
