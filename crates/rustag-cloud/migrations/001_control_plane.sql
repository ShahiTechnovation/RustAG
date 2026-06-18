-- RustAG cloud control-plane schema (SQLite; Postgres-portable).
--
-- This is the *control plane* DB: tenants, API keys, and the registry of hosted
-- stagenets with their allocated ports and process status. Each hosted stagenet
-- runs as an isolated child process with its OWN per-stagenet data-plane DB
-- (Phase 1 `.rustag/db.sqlite`) inside its working directory — so the blast
-- radius of one tenant's stagenet is a single process + a single file.
--
-- Portable to Postgres/Redis in production: same DDL, swap the pool. Redis fronts
-- the slug→port routing table for the reverse proxy under load.

CREATE TABLE IF NOT EXISTS tenants (
    id          TEXT PRIMARY KEY,                 -- UUID v4
    name        TEXT NOT NULL,
    email       TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_tenants_email ON tenants(email);

CREATE TABLE IF NOT EXISTS api_keys (
    id          TEXT PRIMARY KEY,                 -- UUID v4
    tenant_id   TEXT NOT NULL,
    key_hash    TEXT NOT NULL,                    -- sha256 hex of the plaintext key
    label       TEXT,
    created_at  TEXT NOT NULL,
    last_used   TEXT,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant ON api_keys(tenant_id);

CREATE TABLE IF NOT EXISTS cloud_stagenets (
    id          TEXT PRIMARY KEY,                 -- UUID v4
    tenant_id   TEXT NOT NULL,
    slug        TEXT NOT NULL,                    -- subdomain/path segment, unique
    name        TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'creating', -- creating | running | stopped | error
    rpc_port    INTEGER NOT NULL,
    ws_port     INTEGER NOT NULL,
    api_port    INTEGER NOT NULL,
    mainnet_rpc TEXT NOT NULL,
    pid         INTEGER,                          -- child process id when running
    work_dir    TEXT NOT NULL,                    -- isolated per-stagenet data dir
    created_at  TEXT NOT NULL,
    last_active TEXT,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_cloud_stagenets_slug ON cloud_stagenets(slug);
CREATE INDEX IF NOT EXISTS idx_cloud_stagenets_tenant ON cloud_stagenets(tenant_id);
