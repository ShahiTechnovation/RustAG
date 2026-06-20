//! Integration tests for the RustAG core runtime.
//!
//! Network-free tests run by default. The mainnet test is `#[ignore]`d; run it
//! with `cargo test -- --include-ignored` (optionally set `RUSTAG_MAINNET_RPC`).

use std::str::FromStr;

use rustag_core::{AccountOverride, AccountSync, Stagenet};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::instruction::transfer;
use solana_transaction::Transaction;

#[tokio::test]
async fn airdrop_and_check_balance() {
    let mut sn = Stagenet::local("test").await.unwrap();
    let wallet = Keypair::new();

    sn.airdrop(&wallet.pubkey(), 1_000_000_000).await.unwrap();

    let balance = sn.get_balance(&wallet.pubkey()).await.unwrap();
    assert_eq!(balance, 1_000_000_000);
    assert!(sn.is_dirty(&wallet.pubkey()));
}

#[tokio::test]
async fn dirty_tracking_after_transaction() {
    let mut sn = Stagenet::local("test").await.unwrap();
    let payer = Keypair::new();
    let receiver = Keypair::new();

    sn.airdrop(&payer.pubkey(), 2_000_000_000).await.unwrap();

    let ix = transfer(&payer.pubkey(), &receiver.pubkey(), 1_000_000_000);
    let msg = Message::new(&[ix], Some(&payer.pubkey()));
    let tx = Transaction::new(&[&payer], msg, sn.latest_blockhash());

    let outcome = sn.send_transaction(tx.into()).await.unwrap();
    assert!(outcome.success, "tx failed: {:?}", outcome.err);

    // Both writable accounts are now DIRTY (frozen from mainnet sync).
    assert!(sn.is_dirty(&payer.pubkey()));
    assert!(sn.is_dirty(&receiver.pubkey()));

    // Balances reflect the transfer (+ fees on the payer).
    let payer_balance = sn.get_balance(&payer.pubkey()).await.unwrap();
    assert!(
        payer_balance < 1_000_000_000,
        "payer paid fees: {payer_balance}"
    );
    let receiver_balance = sn.get_balance(&receiver.pubkey()).await.unwrap();
    assert_eq!(receiver_balance, 1_000_000_000);

    // The transaction was indexed.
    let txs = sn.store().list_transactions(&sn.id(), 10).await.unwrap();
    assert_eq!(txs.len(), 1);
    assert!(txs[0].success);
}

#[tokio::test]
async fn override_account_and_impersonation() {
    let mut sn = Stagenet::local("test").await.unwrap();
    let acct = Pubkey::new_unique();

    sn.override_account(
        &acct,
        AccountOverride {
            lamports: Some(42),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(sn.get_balance(&acct).await.unwrap(), 42);
    assert!(sn.is_dirty(&acct));

    // Impersonation toggles signature verification.
    assert!(sn.sigverify_enabled());
    sn.enable_impersonation();
    assert!(!sn.sigverify_enabled());
    sn.disable_impersonation();
    assert!(sn.sigverify_enabled());
}

#[tokio::test]
async fn failed_transaction_is_recorded_without_dirtying() {
    let mut sn = Stagenet::local("test").await.unwrap();
    let payer = Keypair::new();
    let receiver = Keypair::new();

    // Payer has 1 SOL but tries to send 5 SOL.
    sn.airdrop(&payer.pubkey(), 1_000_000_000).await.unwrap();
    let ix = transfer(&payer.pubkey(), &receiver.pubkey(), 5_000_000_000);
    let msg = Message::new(&[ix], Some(&payer.pubkey()));
    let tx = Transaction::new(&[&payer], msg, sn.latest_blockhash());

    let outcome = sn.send_transaction(tx.into()).await.unwrap();
    assert!(!outcome.success);
    // Receiver was never created.
    assert!(!sn.is_dirty(&receiver.pubkey()));
    let txs = sn.store().list_transactions(&sn.id(), 10).await.unwrap();
    assert_eq!(txs.len(), 1);
    assert!(!txs[0].success);
}

#[tokio::test]
#[ignore = "hits mainnet RPC; run with --include-ignored"]
async fn lazy_account_fetch_from_mainnet() {
    let rpc = std::env::var("RUSTAG_MAINNET_RPC")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    // The Pyth SOL/USD price feed - always exists, always has data.
    let sol_usd_pyth = Pubkey::from_str("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG").unwrap();

    let mut sn = Stagenet::local_with_mainnet("test", &rpc).await.unwrap();

    // First access fetches from mainnet and caches as CLEAN.
    let account = sn.get_account(&sol_usd_pyth).await.unwrap();
    assert!(account.is_some(), "Pyth SOL/USD must exist on mainnet");

    let entry = sn
        .store()
        .get_account(&sn.id(), &sol_usd_pyth)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(entry.sync_state, AccountSync::Clean { .. }));
}
