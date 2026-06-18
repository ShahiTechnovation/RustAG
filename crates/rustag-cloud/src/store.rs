//! Control-plane persistence: tenants, API keys, and hosted-stagenet records.
//!
//! SQLite via the runtime `sqlx::query` API (no compile-time `DATABASE_URL`
//! needed), mirroring the Phase 1 data-plane store. The DDL is Postgres-portable,
//! so production swaps the pool for `PgPool` with the same queries.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::auth::hash_key;
use crate::error::{CloudError, Result};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// A registered tenant (an account that owns API keys and stagenets).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

/// A hosted stagenet record (control-plane view; the data plane lives in the
/// child process's own DB).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudStagenet {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub slug: String,
    pub name: String,
    pub status: String,
    pub rpc_port: u16,
    pub ws_port: u16,
    pub api_port: u16,
    pub mainnet_rpc: String,
    pub pid: Option<i64>,
    pub work_dir: String,
    pub created_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
}

/// Handle to the control-plane database.
#[derive(Clone)]
pub struct ControlPlaneStore {
    pool: SqlitePool,
}

impl ControlPlaneStore {
    /// Connect, creating and migrating the control-plane DB as needed.
    pub async fn connect(db_path: &str) -> Result<Self> {
        let pool = if db_path == ":memory:" {
            let opts = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
            SqlitePoolOptions::new()
                .min_connections(1)
                .max_connections(1)
                .idle_timeout(None)
                .max_lifetime(None)
                .connect_with(opts)
                .await?
        } else {
            if let Some(parent) = std::path::Path::new(db_path).parent() {
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

    // --- tenants & API keys --------------------------------------------

    /// Create a tenant and issue its first API key. Returns `(tenant, plaintext
    /// key)`; the plaintext is shown once and never stored (only its hash is).
    pub async fn create_tenant(&self, name: &str, email: &str) -> Result<(Tenant, String)> {
        if self.tenant_by_email(email).await?.is_some() {
            return Err(CloudError::Conflict(format!(
                "tenant '{email}' already exists"
            )));
        }
        let tenant = Tenant {
            id: Uuid::new_v4(),
            name: name.to_string(),
            email: email.to_string(),
            created_at: Utc::now(),
        };
        sqlx::query("INSERT INTO tenants (id, name, email, created_at) VALUES (?, ?, ?, ?)")
            .bind(tenant.id.to_string())
            .bind(&tenant.name)
            .bind(&tenant.email)
            .bind(tenant.created_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        let key = self.issue_api_key(&tenant.id, Some("default")).await?;
        Ok((tenant, key))
    }

    async fn tenant_by_email(&self, email: &str) -> Result<Option<Tenant>> {
        let row = sqlx::query("SELECT * FROM tenants WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        row.map(parse_tenant).transpose()
    }

    /// Issue a new API key for a tenant, returning the plaintext exactly once.
    pub async fn issue_api_key(&self, tenant_id: &Uuid, label: Option<&str>) -> Result<String> {
        // 256 bits of entropy from two v4 UUIDs; stored only as a sha256 hash.
        let plaintext = format!("rk_{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        sqlx::query(
            "INSERT INTO api_keys (id, tenant_id, key_hash, label, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id.to_string())
        .bind(hash_key(&plaintext))
        .bind(label)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(plaintext)
    }

    /// Resolve a plaintext API key to its tenant, stamping `last_used`.
    pub async fn tenant_by_key(&self, plaintext: &str) -> Result<Option<Tenant>> {
        let hash = hash_key(plaintext);
        let row = sqlx::query(
            "SELECT t.* FROM tenants t
             JOIN api_keys k ON k.tenant_id = t.id
             WHERE k.key_hash = ?",
        )
        .bind(&hash)
        .fetch_optional(&self.pool)
        .await?;
        let tenant = row.map(parse_tenant).transpose()?;
        if tenant.is_some() {
            let _ = sqlx::query("UPDATE api_keys SET last_used = ? WHERE key_hash = ?")
                .bind(Utc::now().to_rfc3339())
                .bind(&hash)
                .execute(&self.pool)
                .await;
        }
        Ok(tenant)
    }

    // --- hosted stagenets ----------------------------------------------

    /// Whether a slug is already taken.
    pub async fn slug_exists(&self, slug: &str) -> Result<bool> {
        let row = sqlx::query("SELECT 1 AS x FROM cloud_stagenets WHERE slug = ?")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.is_some())
    }

    /// All ports currently allocated to stagenets (for the port allocator).
    pub async fn used_ports(&self) -> Result<Vec<u16>> {
        let rows = sqlx::query("SELECT rpc_port, ws_port, api_port FROM cloud_stagenets")
            .fetch_all(&self.pool)
            .await?;
        let mut ports = Vec::with_capacity(rows.len() * 3);
        for row in rows {
            ports.push(row.try_get::<i64, _>("rpc_port")? as u16);
            ports.push(row.try_get::<i64, _>("ws_port")? as u16);
            ports.push(row.try_get::<i64, _>("api_port")? as u16);
        }
        Ok(ports)
    }

    /// Insert a new stagenet record.
    pub async fn insert_stagenet(&self, rec: &CloudStagenet) -> Result<()> {
        sqlx::query(
            "INSERT INTO cloud_stagenets
                (id, tenant_id, slug, name, status, rpc_port, ws_port, api_port,
                 mainnet_rpc, pid, work_dir, created_at, last_active)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(rec.id.to_string())
        .bind(rec.tenant_id.to_string())
        .bind(&rec.slug)
        .bind(&rec.name)
        .bind(&rec.status)
        .bind(rec.rpc_port as i64)
        .bind(rec.ws_port as i64)
        .bind(rec.api_port as i64)
        .bind(&rec.mainnet_rpc)
        .bind(rec.pid)
        .bind(&rec.work_dir)
        .bind(rec.created_at.to_rfc3339())
        .bind(rec.last_active.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            // The slug UNIQUE index is the source of truth: a concurrent
            // duplicate create becomes a clean 409, not a 500.
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                CloudError::Conflict(format!("slug '{}' is taken", rec.slug))
            }
            _ => CloudError::Database(e),
        })?;
        Ok(())
    }

    /// Reconcile control-plane state on startup: any stagenet left `running` or
    /// `creating` by a previous process has no live child handle here, so mark it
    /// `stopped`. Returns the number of rows reconciled.
    pub async fn reset_running_to_stopped(&self) -> Result<u64> {
        let res = sqlx::query(
            "UPDATE cloud_stagenets SET status = 'stopped'
             WHERE status IN ('running', 'creating')",
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }

    /// Update a stagenet's status and (optionally) pid.
    pub async fn set_status(&self, slug: &str, status: &str, pid: Option<i64>) -> Result<()> {
        sqlx::query(
            "UPDATE cloud_stagenets SET status = ?, pid = ?, last_active = ? WHERE slug = ?",
        )
        .bind(status)
        .bind(pid)
        .bind(Utc::now().to_rfc3339())
        .bind(slug)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Fetch a stagenet by slug.
    pub async fn get_stagenet(&self, slug: &str) -> Result<Option<CloudStagenet>> {
        let row = sqlx::query("SELECT * FROM cloud_stagenets WHERE slug = ?")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?;
        row.map(parse_stagenet).transpose()
    }

    /// List a tenant's stagenets (newest first).
    pub async fn list_stagenets(&self, tenant_id: &Uuid) -> Result<Vec<CloudStagenet>> {
        let rows = sqlx::query(
            "SELECT * FROM cloud_stagenets WHERE tenant_id = ? ORDER BY created_at DESC",
        )
        .bind(tenant_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(parse_stagenet).collect()
    }

    /// Delete a stagenet record. Returns whether a row was removed.
    pub async fn delete_stagenet(&self, slug: &str) -> Result<bool> {
        let res = sqlx::query("DELETE FROM cloud_stagenets WHERE slug = ?")
            .bind(slug)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}

fn parse_tenant(row: sqlx::sqlite::SqliteRow) -> Result<Tenant> {
    Ok(Tenant {
        id: parse_uuid(&row.try_get::<String, _>("id")?)?,
        name: row.try_get("name")?,
        email: row.try_get("email")?,
        created_at: parse_ts(row.try_get("created_at")?),
    })
}

fn parse_stagenet(row: sqlx::sqlite::SqliteRow) -> Result<CloudStagenet> {
    Ok(CloudStagenet {
        id: parse_uuid(&row.try_get::<String, _>("id")?)?,
        tenant_id: parse_uuid(&row.try_get::<String, _>("tenant_id")?)?,
        slug: row.try_get("slug")?,
        name: row.try_get("name")?,
        status: row.try_get("status")?,
        rpc_port: row.try_get::<i64, _>("rpc_port")? as u16,
        ws_port: row.try_get::<i64, _>("ws_port")? as u16,
        api_port: row.try_get::<i64, _>("api_port")? as u16,
        mainnet_rpc: row.try_get("mainnet_rpc")?,
        pid: row.try_get::<Option<i64>, _>("pid")?,
        work_dir: row.try_get("work_dir")?,
        created_at: parse_ts(row.try_get("created_at")?),
        last_active: row
            .try_get::<Option<String>, _>("last_active")?
            .and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
    })
}

fn parse_uuid(s: &str) -> Result<Uuid> {
    Uuid::parse_str(s).map_err(|e| CloudError::BadRequest(format!("invalid uuid: {e}")))
}

fn parse_ts(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
