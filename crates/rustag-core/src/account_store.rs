//! SQLite-backed persistence for stagenets, accounts, and transactions.
//!
//! Uses the runtime `sqlx::query` API (not the compile-time-checked `query!`
//! macro) so a fresh checkout builds with no live database or `DATABASE_URL`.

use std::path::Path;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use solana_pubkey::Pubkey;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::account_state::{AccountEntry, AccountSync};
use crate::error::{Result, RustagError};
use rustag_mirror::AccountCategory;

/// Embedded migrations (compiled in at build time from `/migrations`).
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

/// Persisted metadata about a stagenet.
#[derive(Debug, Clone)]
pub struct StagenetRecord {
    pub id: Uuid,
    pub name: String,
    pub network: String,
    pub rpc_port: u16,
    pub ws_port: u16,
    pub api_port: u16,
    pub created_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
    pub config_json: String,
}

/// A persisted transaction record (powers the dashboard tx feed + `logs`).
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub signature: String,
    pub stagenet_id: Uuid,
    pub slot: u64,
    pub success: bool,
    pub fee: u64,
    pub compute_units: Option<u64>,
    pub programs: Vec<String>,
    pub logs: Vec<String>,
    pub err: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Handle to the persistence layer.
#[derive(Clone)]
pub struct AccountStore {
    pool: SqlitePool,
}

impl AccountStore {
    /// Connect to `db_path` (or `:memory:`), creating and migrating as needed.
    pub async fn connect(db_path: &str) -> Result<Self> {
        let pool = if db_path == ":memory:" {
            // A single shared connection — multiple connections would each get a
            // separate in-memory database.
            let opts = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
            // The single connection IS the database. Keep it pinned and never
            // reaped — otherwise sqlx's idle/lifetime reaper would drop it and
            // silently wipe all in-memory state.
            SqlitePoolOptions::new()
                .min_connections(1)
                .max_connections(1)
                .idle_timeout(None)
                .max_lifetime(None)
                .connect_with(opts)
                .await?
        } else {
            if let Some(parent) = Path::new(db_path).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let opts = SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .foreign_keys(true);
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(opts)
                .await?
        };

        MIGRATOR.run(&pool).await?;
        Ok(Self { pool })
    }

    /// Convenience constructor for an ephemeral in-memory store (tests).
    pub async fn in_memory() -> Result<Self> {
        Self::connect(":memory:").await
    }

    // --- stagenets ------------------------------------------------------

    /// Insert (or replace) a stagenet record.
    pub async fn upsert_stagenet(&self, rec: &StagenetRecord) -> Result<()> {
        sqlx::query(
            "INSERT INTO stagenets
                (id, name, network, rpc_port, ws_port, api_port, created_at, last_active, config_json)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                network = excluded.network,
                rpc_port = excluded.rpc_port,
                ws_port = excluded.ws_port,
                api_port = excluded.api_port,
                last_active = excluded.last_active,
                config_json = excluded.config_json",
        )
        .bind(rec.id.to_string())
        .bind(&rec.name)
        .bind(&rec.network)
        .bind(rec.rpc_port as i64)
        .bind(rec.ws_port as i64)
        .bind(rec.api_port as i64)
        .bind(rec.created_at.to_rfc3339())
        .bind(rec.last_active.map(|t| t.to_rfc3339()))
        .bind(&rec.config_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Look up a stagenet by id.
    pub async fn get_stagenet(&self, id: &Uuid) -> Result<Option<StagenetRecord>> {
        let row = sqlx::query("SELECT * FROM stagenets WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        row.map(parse_stagenet_row).transpose()
    }

    /// Look up a stagenet by name.
    pub async fn get_stagenet_by_name(&self, name: &str) -> Result<Option<StagenetRecord>> {
        let row = sqlx::query("SELECT * FROM stagenets WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;
        row.map(parse_stagenet_row).transpose()
    }

    /// List all stagenets (newest first).
    pub async fn list_stagenets(&self) -> Result<Vec<StagenetRecord>> {
        let rows = sqlx::query("SELECT * FROM stagenets ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(parse_stagenet_row).collect()
    }

    /// Stamp a stagenet as active right now.
    pub async fn touch_stagenet(&self, id: &Uuid) -> Result<()> {
        sqlx::query("UPDATE stagenets SET last_active = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete a stagenet and (via cascade) its accounts and transactions.
    pub async fn delete_stagenet(&self, id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM stagenets WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- accounts -------------------------------------------------------

    /// Insert or update an account entry.
    pub async fn upsert_account(&self, entry: &AccountEntry) -> Result<()> {
        sqlx::query(
            "INSERT INTO accounts
                (pubkey, stagenet_id, lamports, data, owner, executable, rent_epoch,
                 sync_state, category, fetched_at, modified_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(pubkey, stagenet_id) DO UPDATE SET
                lamports = excluded.lamports,
                data = excluded.data,
                owner = excluded.owner,
                executable = excluded.executable,
                rent_epoch = excluded.rent_epoch,
                sync_state = excluded.sync_state,
                category = excluded.category,
                fetched_at = excluded.fetched_at,
                modified_at = excluded.modified_at",
        )
        .bind(entry.pubkey.to_string())
        .bind(entry.stagenet_id.to_string())
        .bind(entry.lamports as i64)
        .bind(entry.data.as_slice())
        .bind(entry.owner.to_string())
        .bind(entry.executable as i64)
        .bind(entry.rent_epoch as i64)
        .bind(entry.sync_state.label())
        .bind(entry.category.map(|c| c.label().to_string()))
        .bind(entry.sync_state.fetched_at().map(|t| t.to_rfc3339()))
        .bind(entry.sync_state.modified_at().map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Fetch a single account entry.
    pub async fn get_account(
        &self,
        stagenet_id: &Uuid,
        pubkey: &Pubkey,
    ) -> Result<Option<AccountEntry>> {
        let row = sqlx::query("SELECT * FROM accounts WHERE stagenet_id = ? AND pubkey = ?")
            .bind(stagenet_id.to_string())
            .bind(pubkey.to_string())
            .fetch_optional(&self.pool)
            .await?;
        row.map(parse_account_row).transpose()
    }

    /// List accounts for a stagenet (most-recently-touched first), paginated.
    pub async fn list_accounts(
        &self,
        stagenet_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AccountEntry>> {
        let rows = sqlx::query(
            "SELECT * FROM accounts WHERE stagenet_id = ?
             ORDER BY COALESCE(modified_at, fetched_at) DESC, pubkey
             LIMIT ? OFFSET ?",
        )
        .bind(stagenet_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(parse_account_row).collect()
    }

    /// All CLEAN accounts of a given category (used by the oracle sync loop).
    pub async fn get_clean_accounts_by_category(
        &self,
        stagenet_id: &Uuid,
        category: AccountCategory,
    ) -> Result<Vec<AccountEntry>> {
        let rows = sqlx::query(
            "SELECT * FROM accounts
             WHERE stagenet_id = ? AND sync_state = 'Clean' AND category = ?",
        )
        .bind(stagenet_id.to_string())
        .bind(category.label())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(parse_account_row).collect()
    }

    /// All accounts owned by `owner` (powers `getProgramAccounts`).
    pub async fn get_program_accounts(
        &self,
        stagenet_id: &Uuid,
        owner: &Pubkey,
        limit: i64,
    ) -> Result<Vec<AccountEntry>> {
        let rows =
            sqlx::query("SELECT * FROM accounts WHERE stagenet_id = ? AND owner = ? LIMIT ?")
                .bind(stagenet_id.to_string())
                .bind(owner.to_string())
                .bind(limit)
                .fetch_all(&self.pool)
                .await?;
        rows.into_iter().map(parse_account_row).collect()
    }

    /// Count accounts in a stagenet.
    pub async fn count_accounts(&self, stagenet_id: &Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) AS n FROM accounts WHERE stagenet_id = ?")
            .bind(stagenet_id.to_string())
            .fetch_one(&self.pool)
            .await?;
        Ok(row.try_get::<i64, _>("n")?)
    }

    // --- transactions ---------------------------------------------------

    /// Insert (or replace) a transaction record.
    pub async fn insert_transaction(&self, rec: &TransactionRecord) -> Result<()> {
        let programs = serde_json::to_string(&rec.programs)
            .map_err(|e| RustagError::Serialization(e.to_string()))?;
        let logs = serde_json::to_string(&rec.logs)
            .map_err(|e| RustagError::Serialization(e.to_string()))?;
        sqlx::query(
            "INSERT INTO transactions
                (signature, stagenet_id, slot, success, fee, compute_units, programs, logs, err, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(signature, stagenet_id) DO UPDATE SET
                slot = excluded.slot,
                success = excluded.success,
                fee = excluded.fee,
                compute_units = excluded.compute_units,
                programs = excluded.programs,
                logs = excluded.logs,
                err = excluded.err",
        )
        .bind(&rec.signature)
        .bind(rec.stagenet_id.to_string())
        .bind(rec.slot as i64)
        .bind(rec.success as i64)
        .bind(rec.fee as i64)
        .bind(rec.compute_units.map(|c| c as i64))
        .bind(programs)
        .bind(logs)
        .bind(rec.err.as_deref())
        .bind(rec.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// List recent transactions for a stagenet (newest first).
    pub async fn list_transactions(
        &self,
        stagenet_id: &Uuid,
        limit: i64,
    ) -> Result<Vec<TransactionRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM transactions WHERE stagenet_id = ?
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(stagenet_id.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(parse_transaction_row).collect()
    }

    /// Look up a single transaction by signature.
    pub async fn get_transaction(
        &self,
        stagenet_id: &Uuid,
        signature: &str,
    ) -> Result<Option<TransactionRecord>> {
        let row = sqlx::query("SELECT * FROM transactions WHERE stagenet_id = ? AND signature = ?")
            .bind(stagenet_id.to_string())
            .bind(signature)
            .fetch_optional(&self.pool)
            .await?;
        row.map(parse_transaction_row).transpose()
    }

    /// Count transactions in a stagenet.
    pub async fn count_transactions(&self, stagenet_id: &Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) AS n FROM transactions WHERE stagenet_id = ?")
            .bind(stagenet_id.to_string())
            .fetch_one(&self.pool)
            .await?;
        Ok(row.try_get::<i64, _>("n")?)
    }
}

// --- row -> domain mapping --------------------------------------------------

fn parse_ts(s: Option<String>) -> Option<DateTime<Utc>> {
    s.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|d| d.with_timezone(&Utc))
    })
}

fn parse_pubkey(s: &str) -> Result<Pubkey> {
    Pubkey::from_str(s).map_err(|_| RustagError::InvalidPubkey(s.to_string()))
}

fn parse_stagenet_row(row: sqlx::sqlite::SqliteRow) -> Result<StagenetRecord> {
    let id_str: String = row.try_get("id")?;
    let id = Uuid::parse_str(&id_str).map_err(|e| RustagError::Serialization(e.to_string()))?;
    Ok(StagenetRecord {
        id,
        name: row.try_get("name")?,
        network: row.try_get("network")?,
        rpc_port: row.try_get::<i64, _>("rpc_port")? as u16,
        ws_port: row.try_get::<i64, _>("ws_port")? as u16,
        api_port: row.try_get::<i64, _>("api_port")? as u16,
        created_at: parse_ts(row.try_get("created_at")?).unwrap_or_else(Utc::now),
        last_active: parse_ts(row.try_get("last_active")?),
        config_json: row.try_get("config_json")?,
    })
}

fn parse_account_row(row: sqlx::sqlite::SqliteRow) -> Result<AccountEntry> {
    let pubkey = parse_pubkey(&row.try_get::<String, _>("pubkey")?)?;
    let owner = parse_pubkey(&row.try_get::<String, _>("owner")?)?;
    let stagenet_id = Uuid::parse_str(&row.try_get::<String, _>("stagenet_id")?)
        .map_err(|e| RustagError::Serialization(e.to_string()))?;
    let label: String = row.try_get("sync_state")?;
    let fetched_at = parse_ts(row.try_get("fetched_at")?);
    let modified_at = parse_ts(row.try_get("modified_at")?);
    let category = row
        .try_get::<Option<String>, _>("category")?
        .and_then(|c| AccountCategory::from_label(&c));

    Ok(AccountEntry {
        pubkey,
        data: row.try_get::<Vec<u8>, _>("data")?,
        owner,
        lamports: row.try_get::<i64, _>("lamports")? as u64,
        executable: row.try_get::<i64, _>("executable")? != 0,
        rent_epoch: row.try_get::<i64, _>("rent_epoch")? as u64,
        sync_state: AccountSync::from_parts(&label, fetched_at, modified_at),
        category,
        stagenet_id,
    })
}

fn parse_transaction_row(row: sqlx::sqlite::SqliteRow) -> Result<TransactionRecord> {
    let stagenet_id = Uuid::parse_str(&row.try_get::<String, _>("stagenet_id")?)
        .map_err(|e| RustagError::Serialization(e.to_string()))?;
    let programs: Vec<String> = row
        .try_get::<Option<String>, _>("programs")?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let logs: Vec<String> = row
        .try_get::<Option<String>, _>("logs")?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    Ok(TransactionRecord {
        signature: row.try_get("signature")?,
        stagenet_id,
        slot: row.try_get::<i64, _>("slot")? as u64,
        success: row.try_get::<i64, _>("success")? != 0,
        fee: row.try_get::<i64, _>("fee")? as u64,
        compute_units: row
            .try_get::<Option<i64>, _>("compute_units")?
            .map(|c| c as u64),
        programs,
        logs,
        err: row.try_get("err")?,
        created_at: parse_ts(row.try_get("created_at")?).unwrap_or_else(Utc::now),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn account_crud_and_sync_query() {
        let store = AccountStore::in_memory().await.unwrap();
        let sid = Uuid::new_v4();

        // FK requires the stagenet to exist first.
        store
            .upsert_stagenet(&StagenetRecord {
                id: sid,
                name: "t".into(),
                network: "mainnet-beta".into(),
                rpc_port: 8899,
                ws_port: 8900,
                api_port: 9000,
                created_at: Utc::now(),
                last_active: None,
                config_json: "{}".into(),
            })
            .await
            .unwrap();

        let mut entry = AccountEntry {
            pubkey: Pubkey::new_unique(),
            data: vec![9, 8, 7],
            owner: Pubkey::new_unique(),
            lamports: 12_345,
            executable: false,
            rent_epoch: u64::MAX, // must survive the i64 round-trip
            sync_state: AccountSync::clean_now(),
            category: Some(AccountCategory::Oracle),
            stagenet_id: sid,
        };
        store.upsert_account(&entry).await.unwrap();

        let got = store
            .get_account(&sid, &entry.pubkey)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.lamports, 12_345);
        assert_eq!(got.rent_epoch, u64::MAX);
        assert_eq!(got.data, vec![9, 8, 7]);
        assert_eq!(got.category, Some(AccountCategory::Oracle));
        assert!(matches!(got.sync_state, AccountSync::Clean { .. }));

        // Clean oracle shows up in the sync query.
        let clean = store
            .get_clean_accounts_by_category(&sid, AccountCategory::Oracle)
            .await
            .unwrap();
        assert_eq!(clean.len(), 1);

        // Dirty it: no longer syncable.
        entry.mark_dirty();
        store.upsert_account(&entry).await.unwrap();
        let clean = store
            .get_clean_accounts_by_category(&sid, AccountCategory::Oracle)
            .await
            .unwrap();
        assert_eq!(clean.len(), 0);
        assert_eq!(store.count_accounts(&sid).await.unwrap(), 1);
    }
}
