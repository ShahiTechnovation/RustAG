//! The RustAG staging environment: a LiteSVM runtime wrapped with lazy mainnet
//! mirroring, dirty/clean tracking, persistence, and overrides.

use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashSet;
use litesvm::LiteSVM;
use moka::future::Cache;
use solana_account::Account;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_hash::Hash;
use solana_message::VersionedMessage;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_transaction::versioned::VersionedTransaction;
use uuid::Uuid;

use rustag_mirror::{registry, AccountCategory, MainnetMirror, RemoteAccount};

use crate::account_state::{AccountEntry, AccountSync};
use crate::account_store::{AccountStore, StagenetRecord, TransactionRecord};
use crate::config::StagenetConfig;
use crate::error::{Result, RustagError};
use crate::metrics::MetricsSnapshot;

/// SPL token account `amount` field offset (mint[32] + owner[32]).
const SPL_TOKEN_AMOUNT_OFFSET: usize = 64;

/// The outcome of executing or simulating a transaction.
#[derive(Debug, Clone)]
pub struct TxOutcome {
    pub signature: Signature,
    pub success: bool,
    pub err: Option<String>,
    pub compute_units: u64,
    pub fee: u64,
    pub logs: Vec<String>,
}

impl TxOutcome {
    /// The transaction signature as a base58 string.
    pub fn signature_string(&self) -> String {
        self.signature.to_string()
    }
}

/// Parameters for the generic account override API.
#[derive(Debug, Clone, Default)]
pub struct AccountOverride {
    pub lamports: Option<u64>,
    pub data: Option<Vec<u8>>,
    pub owner: Option<Pubkey>,
    pub executable: Option<bool>,
}

/// A persistent, mainnet-mirroring staging environment for Solana programs.
pub struct Stagenet {
    /// The LiteSVM runtime - executes transactions.
    svm: LiteSVM,
    /// Persisted account/transaction state.
    store: Arc<AccountStore>,
    /// Mainnet RPC fetcher.
    mirror: Arc<MainnetMirror>,
    /// Hot in-memory cache of account entries (sync state + category).
    cache: Cache<Pubkey, AccountEntry>,
    /// Accounts modified locally (frozen from mainnet sync).
    dirty_set: DashSet<Pubkey>,
    /// Stagenet identity.
    id: Uuid,
    /// Stagenet configuration.
    config: StagenetConfig,
    /// Monotonic slot counter, advanced once per processed transaction.
    slot: u64,
}

impl Stagenet {
    // --- construction ---------------------------------------------------

    /// Create a brand-new stagenet and persist its record.
    pub async fn create(config: StagenetConfig, store: Arc<AccountStore>) -> Result<Self> {
        let sn = Self::build(Uuid::new_v4(), config, store)?;
        sn.persist_record().await?;
        Ok(sn)
    }

    /// Reopen an existing stagenet, rehydrating its accounts into the SVM.
    pub async fn reopen(
        id: Uuid,
        config: StagenetConfig,
        store: Arc<AccountStore>,
    ) -> Result<Self> {
        let mut sn = Self::build(id, config, store)?;
        sn.rehydrate().await?;
        sn.store.touch_stagenet(&id).await?;
        Ok(sn)
    }

    /// An offline, in-memory stagenet for local tests (mirror disabled).
    pub async fn local(name: &str) -> Result<Self> {
        let store = Arc::new(AccountStore::in_memory().await?);
        Self::create(StagenetConfig::local(name), store).await
    }

    /// An in-memory stagenet wired to a real mainnet endpoint (lazy fetch on).
    pub async fn local_with_mainnet(name: &str, mainnet_rpc: &str) -> Result<Self> {
        let store = Arc::new(AccountStore::in_memory().await?);
        let config = StagenetConfig {
            mainnet_rpc: mainnet_rpc.to_string(),
            mirror_enabled: true,
            ..StagenetConfig::local(name)
        };
        Self::create(config, store).await
    }

    fn build(id: Uuid, config: StagenetConfig, store: Arc<AccountStore>) -> Result<Self> {
        let mirror = Arc::new(MainnetMirror::new(&config.mainnet_rpc, config.max_rps)?);
        let cache = Cache::builder().max_capacity(50_000).build();
        Ok(Self {
            svm: LiteSVM::new(),
            store,
            mirror,
            cache,
            dirty_set: DashSet::new(),
            id,
            config,
            slot: 0,
        })
    }

    async fn persist_record(&self) -> Result<()> {
        let config_json = serde_json::to_string(&self.config)
            .map_err(|e| RustagError::Serialization(e.to_string()))?;
        let now = chrono::Utc::now();
        let rec = StagenetRecord {
            id: self.id,
            name: self.config.name.clone(),
            network: self.config.network.clone(),
            rpc_port: self.config.rpc_port,
            ws_port: self.config.ws_port,
            api_port: self.config.api_port,
            created_at: now,
            last_active: Some(now),
            config_json,
        };
        self.store.upsert_stagenet(&rec).await
    }

    async fn rehydrate(&mut self) -> Result<()> {
        let mut offset = 0i64;
        const PAGE: i64 = 1000;
        loop {
            let batch = self.store.list_accounts(&self.id, PAGE, offset).await?;
            let n = batch.len();
            for entry in &batch {
                self.svm
                    .set_account_no_checks(entry.pubkey, entry.to_shared_data());
                if !entry.is_syncable() {
                    self.dirty_set.insert(entry.pubkey);
                }
                self.cache.insert(entry.pubkey, entry.clone()).await;
            }
            offset += n as i64;
            if (n as i64) < PAGE {
                break;
            }
        }
        Ok(())
    }

    // --- accessors ------------------------------------------------------

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn config(&self) -> &StagenetConfig {
        &self.config
    }

    pub fn store(&self) -> Arc<AccountStore> {
        Arc::clone(&self.store)
    }

    pub fn mirror(&self) -> Arc<MainnetMirror> {
        Arc::clone(&self.mirror)
    }

    /// The latest blockhash, for transaction construction.
    pub fn latest_blockhash(&self) -> Hash {
        self.svm.latest_blockhash()
    }

    /// The current slot (advances once per processed transaction).
    pub fn current_slot(&self) -> u64 {
        self.slot
    }

    /// The minimum lamport balance for an account of `data_len` to be rent-exempt.
    pub fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        self.svm.minimum_balance_for_rent_exemption(data_len)
    }

    /// Whether `pubkey` has been locally modified.
    pub fn is_dirty(&self, pubkey: &Pubkey) -> bool {
        self.dirty_set.contains(pubkey)
    }

    /// Number of locally-modified accounts.
    pub fn dirty_count(&self) -> usize {
        self.dirty_set.len()
    }

    // --- snapshot / fork (Phase 2 simulation framework) -----------------

    /// Export every persisted account in this stagenet (a paginated read from
    /// the store). The returned entries carry this stagenet's id;
    /// [`import_accounts`](Self::import_accounts) re-homes them on load.
    pub async fn export_accounts(&self) -> Result<Vec<AccountEntry>> {
        let mut out = Vec::new();
        let mut offset = 0i64;
        const PAGE: i64 = 1000;
        loop {
            let batch = self.store.list_accounts(&self.id, PAGE, offset).await?;
            let n = batch.len();
            out.extend(batch);
            offset += n as i64;
            if (n as i64) < PAGE {
                break;
            }
        }
        Ok(out)
    }

    /// Load account entries into this stagenet, preserving each account's sync
    /// state (dirty/pinned accounts stay frozen from mainnet sync). Entries are
    /// re-homed to this stagenet's id so a snapshot from one stagenet can seed
    /// another.
    pub async fn import_accounts(&mut self, entries: &[AccountEntry]) -> Result<()> {
        for entry in entries {
            let mut e = entry.clone();
            e.stagenet_id = self.id;
            self.svm.set_account_no_checks(e.pubkey, e.to_shared_data());
            if !e.is_syncable() {
                self.dirty_set.insert(e.pubkey);
            }
            self.cache.insert(e.pubkey, e.clone()).await;
            self.store.upsert_account(&e).await?;
        }
        Ok(())
    }

    /// Fork this stagenet into a fresh, **offline in-memory** copy carrying the
    /// same account state. The fork has mirroring disabled and its own database,
    /// so transactions run against it never touch mainnet or the original - the
    /// basis of the simulation framework ("what if N users act at once?").
    pub async fn fork(&self, name: &str) -> Result<Stagenet> {
        let snapshot = self.export_accounts().await?;
        let mut fork = Stagenet::local(name).await?;
        fork.import_accounts(&snapshot).await?;
        fork.slot = self.slot;
        Ok(fork)
    }

    /// Apply a streamed mainnet account update from the real-time mirror.
    /// Returns `true` if applied, `false` if skipped because the account is
    /// locally modified (dirty or pinned - both live in the dirty-set).
    pub async fn apply_realtime_update(&mut self, remote: RemoteAccount) -> Result<bool> {
        if self.dirty_set.contains(&remote.pubkey) {
            return Ok(false);
        }
        let category = registry::categorize(&remote.pubkey);
        let entry = AccountEntry::from_remote(remote, self.id, category);
        self.load_clean(entry).await?;
        Ok(true)
    }

    // --- analytics ------------------------------------------------------

    /// Capture a point-in-time analytics snapshot of stagenet-level gauges.
    pub async fn collect_metrics(&self) -> Result<MetricsSnapshot> {
        let accounts = self.store.count_accounts(&self.id).await?;
        let transactions = self.store.count_transactions(&self.id).await?;
        let tvl_lamports = self.store.sum_account_lamports(&self.id).await?;
        Ok(MetricsSnapshot {
            accounts,
            transactions,
            dirty_accounts: self.dirty_set.len() as i64,
            slot: self.slot,
            tvl_lamports,
            recorded_at: chrono::Utc::now(),
        })
    }

    // --- the lazy mirror ------------------------------------------------

    /// Pre-load every account a transaction touches that we don't already have.
    /// This is the lazy fetch - "mainnet on demand". Fetch failures are logged
    /// and tolerated so a transaction can still proceed.
    #[tracing::instrument(skip(self, account_keys), fields(stagenet = %self.id))]
    pub async fn pre_load_accounts_for_tx(&mut self, account_keys: &[Pubkey]) -> Result<()> {
        if !self.config.mirror_enabled {
            return Ok(());
        }
        let mut to_fetch: Vec<Pubkey> = Vec::new();
        for key in account_keys {
            if self.dirty_set.contains(key) {
                continue; // locally modified - never re-fetch
            }
            if self.svm.get_account(key).is_some() {
                continue; // already loaded
            }
            to_fetch.push(*key);
        }
        if to_fetch.is_empty() {
            return Ok(());
        }

        match self.mirror.fetch_multiple(&to_fetch).await {
            Ok(results) => {
                for (key, remote) in to_fetch.iter().zip(results) {
                    if let Some(remote) = remote {
                        let category = registry::categorize(key);
                        let entry = AccountEntry::from_remote(remote, self.id, category);
                        self.load_clean(entry).await?;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "mainnet pre-load failed; continuing with local state");
            }
        }
        Ok(())
    }

    async fn load_clean(&mut self, entry: AccountEntry) -> Result<()> {
        self.svm
            .set_account_no_checks(entry.pubkey, entry.to_shared_data());
        self.cache.insert(entry.pubkey, entry.clone()).await;
        self.store.upsert_account(&entry).await?;
        Ok(())
    }

    /// Pre-load all accounts a transaction references - the static keys *and*,
    /// for v0 transactions, the accounts resolved through address lookup tables -
    /// and return the complete set of writable account keys for dirty tracking.
    async fn prepare_accounts(&mut self, message: &VersionedMessage) -> Result<Vec<Pubkey>> {
        let static_keys = message.static_account_keys().to_vec();
        self.pre_load_accounts_for_tx(&static_keys).await?;
        let mut writable = writable_static_keys(message);
        writable.extend(self.preload_lookup_tables(message).await?);
        Ok(writable)
    }

    /// Resolve and pre-load address-lookup-table accounts for a v0 message,
    /// returning the writable LUT-resolved keys. Without this, a v0 DeFi
    /// transaction would fail in the SVM with `LookupTableAccountNotFound`.
    async fn preload_lookup_tables(&mut self, message: &VersionedMessage) -> Result<Vec<Pubkey>> {
        let VersionedMessage::V0(v0) = message else {
            return Ok(Vec::new());
        };
        if v0.address_table_lookups.is_empty() {
            return Ok(Vec::new());
        }

        // 1. Fetch the lookup-table accounts themselves.
        let lut_keys: Vec<Pubkey> = v0
            .address_table_lookups
            .iter()
            .map(|l| l.account_key)
            .collect();
        self.pre_load_accounts_for_tx(&lut_keys).await?;

        // 2. Resolve the referenced indexes into concrete addresses.
        let mut writable = Vec::new();
        let mut resolved = Vec::new();
        for lookup in &v0.address_table_lookups {
            let Some(account) = self.svm.get_account(&lookup.account_key) else {
                continue;
            };
            let Ok(table) = AddressLookupTable::deserialize(&account.data) else {
                continue;
            };
            for &index in &lookup.writable_indexes {
                if let Some(address) = table.addresses.get(index as usize) {
                    writable.push(*address);
                    resolved.push(*address);
                }
            }
            for &index in &lookup.readonly_indexes {
                if let Some(address) = table.addresses.get(index as usize) {
                    resolved.push(*address);
                }
            }
        }

        // 3. Pre-load the resolved accounts so the SVM can execute the tx.
        self.pre_load_accounts_for_tx(&resolved).await?;
        Ok(writable)
    }

    /// Read an account, lazily fetching it from mainnet on first access.
    #[tracing::instrument(skip(self), fields(stagenet = %self.id, pubkey = %pubkey))]
    pub async fn get_account(&mut self, pubkey: &Pubkey) -> Result<Option<AccountEntry>> {
        if self.svm.get_account(pubkey).is_some() {
            let sync = self.sync_state_of(pubkey).await;
            let category = self.category_of(pubkey).await;
            return Ok(Some(self.entry_from_svm(*pubkey, sync, category)));
        }
        if self.dirty_set.contains(pubkey) || !self.config.mirror_enabled {
            return Ok(None);
        }
        let Some(remote) = self.mirror.fetch_one(pubkey).await? else {
            return Ok(None);
        };
        let category = registry::categorize(pubkey);
        let entry = AccountEntry::from_remote(remote, self.id, category);
        self.load_clean(entry.clone()).await?;
        Ok(Some(entry))
    }

    /// Read an account's lamport balance (lazily fetching if needed).
    pub async fn get_balance(&mut self, pubkey: &Pubkey) -> Result<u64> {
        Ok(self
            .get_account(pubkey)
            .await?
            .map(|e| e.lamports)
            .unwrap_or(0))
    }

    // --- execution ------------------------------------------------------

    /// Execute a transaction: pre-load referenced accounts, run it, mark the
    /// writable accounts dirty, and index it.
    #[tracing::instrument(skip(self, tx), fields(stagenet = %self.id))]
    pub async fn send_transaction(&mut self, tx: VersionedTransaction) -> Result<TxOutcome> {
        let programs = program_ids(&tx.message);
        let signature = tx.signatures.first().copied().unwrap_or_default();

        // Pre-load every referenced account (static + lookup-table-resolved) and
        // collect the full writable set so dirty tracking covers v0 LUT writes.
        let writable = self.prepare_accounts(&tx.message).await?;

        let outcome = match self.svm.send_transaction(tx) {
            Ok(meta) => TxOutcome {
                signature,
                success: true,
                err: None,
                compute_units: meta.compute_units_consumed,
                fee: meta.fee,
                logs: meta.logs,
            },
            Err(fail) => TxOutcome {
                signature,
                success: false,
                err: Some(
                    serde_json::to_string(&fail.err).unwrap_or_else(|_| format!("{:?}", fail.err)),
                ),
                compute_units: fail.meta.compute_units_consumed,
                fee: fail.meta.fee,
                logs: fail.meta.logs,
            },
        };

        if outcome.success {
            for key in &writable {
                self.mark_dirty_from_svm(*key).await?;
            }
        }
        self.slot = self.slot.saturating_add(1);
        self.index_transaction(&outcome, &programs).await?;

        tracing::info!(
            signature = %outcome.signature,
            success = outcome.success,
            compute_units = outcome.compute_units,
            "transaction processed"
        );
        Ok(outcome)
    }

    /// Simulate a transaction without committing state.
    #[tracing::instrument(skip(self, tx), fields(stagenet = %self.id))]
    pub async fn simulate_transaction(&mut self, tx: VersionedTransaction) -> Result<TxOutcome> {
        let signature = tx.signatures.first().copied().unwrap_or_default();
        let _ = self.prepare_accounts(&tx.message).await?;

        Ok(match self.svm.simulate_transaction(tx) {
            Ok(info) => TxOutcome {
                signature,
                success: true,
                err: None,
                compute_units: info.meta.compute_units_consumed,
                fee: info.meta.fee,
                logs: info.meta.logs,
            },
            Err(fail) => TxOutcome {
                signature,
                success: false,
                err: Some(
                    serde_json::to_string(&fail.err).unwrap_or_else(|_| format!("{:?}", fail.err)),
                ),
                compute_units: fail.meta.compute_units_consumed,
                fee: fail.meta.fee,
                logs: fail.meta.logs,
            },
        })
    }

    /// Unlimited airdrop - the headline feature against the faucet problem.
    ///
    /// Uses LiteSVM's airdrop (a real system transfer) when possible, and falls
    /// back to crediting the balance directly when that would fail (e.g. an
    /// amount below the rent-exemption minimum for a brand-new account). A test
    /// faucet should always succeed.
    #[tracing::instrument(skip(self), fields(stagenet = %self.id, pubkey = %pubkey, lamports))]
    pub async fn airdrop(&mut self, pubkey: &Pubkey, lamports: u64) -> Result<()> {
        if self.svm.airdrop(pubkey, lamports).is_err() {
            let mut account = self.svm.get_account(pubkey).unwrap_or_else(|| Account {
                lamports: 0,
                data: Vec::new(),
                owner: Pubkey::default(),
                executable: false,
                rent_epoch: 0,
            });
            account.lamports = account.lamports.saturating_add(lamports);
            self.svm.set_account_no_checks(*pubkey, account.into());
        }
        self.mark_dirty_from_svm(*pubkey).await?;
        tracing::info!(sol = lamports as f64 / 1_000_000_000.0, "airdrop executed");
        Ok(())
    }

    /// Airdrop and index a synthetic confirmed transaction, returning its
    /// signature. Lets `requestAirdrop` over JSON-RPC be confirmed by clients.
    pub async fn airdrop_with_record(
        &mut self,
        pubkey: &Pubkey,
        lamports: u64,
    ) -> Result<Signature> {
        self.airdrop(pubkey, lamports).await?;
        // Advance the slot first so the synthetic signature and the persisted
        // record agree on the same slot.
        self.slot = self.slot.saturating_add(1);
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(pubkey.as_ref());
        bytes[32..40].copy_from_slice(&self.slot.to_le_bytes());
        let signature = Signature::from(bytes);
        let rec = TransactionRecord {
            signature: signature.to_string(),
            stagenet_id: self.id,
            slot: self.slot,
            success: true,
            fee: 0,
            compute_units: Some(0),
            programs: vec!["airdrop".to_string()],
            logs: vec![format!("airdrop {lamports} lamports")],
            err: None,
            created_at: chrono::Utc::now(),
        };
        self.store.insert_transaction(&rec).await?;
        Ok(signature)
    }

    // --- overrides ------------------------------------------------------

    /// Override arbitrary account fields, pinning the account from sync.
    #[tracing::instrument(skip(self, ov), fields(stagenet = %self.id, pubkey = %pubkey))]
    pub async fn override_account(&mut self, pubkey: &Pubkey, ov: AccountOverride) -> Result<()> {
        let mut account = self.svm.get_account(pubkey).unwrap_or_else(|| Account {
            lamports: 0,
            data: Vec::new(),
            owner: Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        });
        if let Some(l) = ov.lamports {
            account.lamports = l;
        }
        if let Some(d) = ov.data {
            account.data = d;
        }
        if let Some(o) = ov.owner {
            account.owner = o;
        }
        if let Some(e) = ov.executable {
            account.executable = e;
        }
        self.pin_account(*pubkey, account).await
    }

    /// Override an SPL token account's `amount` field (Pinned).
    #[tracing::instrument(skip(self), fields(stagenet = %self.id, token_account = %token_account))]
    pub async fn override_token_balance(
        &mut self,
        token_account: &Pubkey,
        amount: u64,
    ) -> Result<()> {
        // Make sure it's loaded (lazily, if mirroring is on).
        self.get_account(token_account).await?;
        let mut account = self
            .svm
            .get_account(token_account)
            .ok_or(RustagError::AccountNotFound(*token_account))?;
        if account.data.len() < SPL_TOKEN_AMOUNT_OFFSET + 8 {
            return Err(RustagError::InvalidTokenAccount(*token_account));
        }
        account.data[SPL_TOKEN_AMOUNT_OFFSET..SPL_TOKEN_AMOUNT_OFFSET + 8]
            .copy_from_slice(&amount.to_le_bytes());
        self.pin_account(*token_account, account).await
    }

    async fn pin_account(&mut self, pubkey: Pubkey, account: Account) -> Result<()> {
        self.svm
            .set_account_no_checks(pubkey, account.clone().into());
        self.dirty_set.insert(pubkey);
        let category = self.category_of(&pubkey).await;
        let entry = AccountEntry {
            pubkey,
            data: account.data,
            owner: account.owner,
            lamports: account.lamports,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            sync_state: AccountSync::Pinned,
            category,
            stagenet_id: self.id,
        };
        self.cache.insert(pubkey, entry.clone()).await;
        self.store.upsert_account(&entry).await?;
        Ok(())
    }

    // --- impersonation --------------------------------------------------

    /// Enable impersonation: stop verifying signatures so transactions can be
    /// submitted as if signed by any account (e.g. a whale wallet).
    pub fn enable_impersonation(&mut self) {
        self.set_sigverify(false);
    }

    /// Re-enable signature verification.
    pub fn disable_impersonation(&mut self) {
        self.set_sigverify(true);
    }

    /// Whether signature verification is currently enabled.
    pub fn sigverify_enabled(&self) -> bool {
        self.svm.get_sigverify()
    }

    fn set_sigverify(&mut self, enabled: bool) {
        // `with_sigverify` consumes and returns the SVM but only flips a flag,
        // so all account/program state is preserved across the swap.
        let svm = std::mem::take(&mut self.svm);
        self.svm = svm.with_sigverify(enabled);
    }

    // --- preload + sync -------------------------------------------------

    /// Preload a set of known accounts from mainnet, tagging their categories.
    #[tracing::instrument(skip(self, entries), fields(stagenet = %self.id, count = entries.len()))]
    pub async fn preload(&mut self, entries: &[(Pubkey, AccountCategory)]) -> Result<usize> {
        if !self.config.mirror_enabled {
            return Err(RustagError::MirrorDisabled);
        }
        let pubkeys: Vec<Pubkey> = entries.iter().map(|(p, _)| *p).collect();
        let results = self.mirror.fetch_multiple(&pubkeys).await?;
        let mut loaded = 0;
        for ((pubkey, category), remote) in entries.iter().zip(results) {
            if let Some(remote) = remote {
                let entry = AccountEntry::from_remote(remote, self.id, Some(*category));
                debug_assert_eq!(entry.pubkey, *pubkey);
                self.load_clean(entry).await?;
                loaded += 1;
            } else {
                tracing::warn!(pubkey = %pubkey, "preload: account not found on mainnet");
            }
        }
        Ok(loaded)
    }

    /// Re-fetch all CLEAN oracle accounts from mainnet. Called on an interval by
    /// the background sync loop. Returns the number of accounts refreshed.
    #[tracing::instrument(skip(self), fields(stagenet = %self.id))]
    pub async fn refresh_clean_oracles(&mut self) -> Result<usize> {
        if !self.config.mirror_enabled {
            return Ok(0);
        }
        let clean = self
            .store
            .get_clean_accounts_by_category(&self.id, AccountCategory::Oracle)
            .await?;
        if clean.is_empty() {
            return Ok(0);
        }
        let pubkeys: Vec<Pubkey> = clean.iter().map(|e| e.pubkey).collect();
        let results = self.mirror.fetch_multiple(&pubkeys).await?;
        let mut refreshed = 0;
        for (pubkey, remote) in pubkeys.iter().zip(results) {
            if self.dirty_set.contains(pubkey) {
                continue; // became dirty between query and fetch
            }
            if let Some(remote) = remote {
                let entry =
                    AccountEntry::from_remote(remote, self.id, Some(AccountCategory::Oracle));
                self.load_clean(entry).await?;
                refreshed += 1;
            }
        }
        tracing::debug!(refreshed, "oracle sync completed");
        Ok(refreshed)
    }

    // --- internals ------------------------------------------------------

    async fn mark_dirty_from_svm(&mut self, pubkey: Pubkey) -> Result<()> {
        let category = self.category_of(&pubkey).await;
        let entry = self.entry_from_svm(pubkey, AccountSync::dirty_now(), category);
        self.dirty_set.insert(pubkey);
        self.cache.insert(pubkey, entry.clone()).await;
        self.store.upsert_account(&entry).await?;
        Ok(())
    }

    fn entry_from_svm(
        &self,
        pubkey: Pubkey,
        sync_state: AccountSync,
        category: Option<AccountCategory>,
    ) -> AccountEntry {
        let account = self.svm.get_account(&pubkey).unwrap_or_default();
        AccountEntry {
            pubkey,
            data: account.data,
            owner: account.owner,
            lamports: account.lamports,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            sync_state,
            category,
            stagenet_id: self.id,
        }
    }

    async fn sync_state_of(&self, pubkey: &Pubkey) -> AccountSync {
        if let Some(entry) = self.cache.get(pubkey).await {
            return entry.sync_state;
        }
        if self.dirty_set.contains(pubkey) {
            return AccountSync::dirty_now();
        }
        AccountSync::clean_now()
    }

    async fn category_of(&self, pubkey: &Pubkey) -> Option<AccountCategory> {
        if let Some(entry) = self.cache.get(pubkey).await {
            if entry.category.is_some() {
                return entry.category;
            }
        }
        registry::categorize(pubkey)
    }

    async fn index_transaction(&self, outcome: &TxOutcome, programs: &[String]) -> Result<()> {
        let rec = TransactionRecord {
            signature: outcome.signature.to_string(),
            stagenet_id: self.id,
            slot: self.slot,
            success: outcome.success,
            fee: outcome.fee,
            compute_units: Some(outcome.compute_units),
            programs: programs.to_vec(),
            logs: outcome.logs.clone(),
            err: outcome.err.clone(),
            created_at: chrono::Utc::now(),
        };
        self.store.insert_transaction(&rec).await
    }
}

// --- message helpers --------------------------------------------------------

/// The writable accounts in a message's static key list, derived from the
/// header. Lookup-table-resolved writable accounts are added separately by
/// `preload_lookup_tables`.
fn writable_static_keys(message: &VersionedMessage) -> Vec<Pubkey> {
    let header = message.header();
    let keys = message.static_account_keys();
    let num_signed = header.num_required_signatures as usize;
    let num_ro_signed = header.num_readonly_signed_accounts as usize;
    let num_ro_unsigned = header.num_readonly_unsigned_accounts as usize;
    let num_writable_signed = num_signed.saturating_sub(num_ro_signed);
    let writable_unsigned_end = keys.len().saturating_sub(num_ro_unsigned);

    keys.iter()
        .enumerate()
        .filter_map(|(i, key)| {
            let writable = if i < num_signed {
                i < num_writable_signed
            } else {
                i < writable_unsigned_end
            };
            writable.then_some(*key)
        })
        .collect()
}

/// Unique program ids invoked by a message's instructions.
fn program_ids(message: &VersionedMessage) -> Vec<String> {
    let keys = message.static_account_keys();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for ix in message.instructions() {
        if let Some(pubkey) = keys.get(ix.program_id_index as usize) {
            let s = pubkey.to_string();
            if seen.insert(s.clone()) {
                out.push(s);
            }
        }
    }
    out
}
