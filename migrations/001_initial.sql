-- RustAG Phase 1 schema (SQLite).
--
-- Designed to be portable to Postgres in Phase 2: no SQLite-only column types,
-- timestamps stored as ISO-8601 TEXT, booleans as INTEGER(0/1).

CREATE TABLE IF NOT EXISTS stagenets (
    id          TEXT PRIMARY KEY,                 -- UUID v4
    name        TEXT NOT NULL,
    network     TEXT NOT NULL DEFAULT 'mainnet-beta',
    rpc_port    INTEGER NOT NULL,
    ws_port     INTEGER NOT NULL DEFAULT 0,
    api_port    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,                     -- ISO-8601
    last_active TEXT,
    config_json TEXT NOT NULL                      -- serialized StagenetConfig
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_stagenets_name ON stagenets(name);

CREATE TABLE IF NOT EXISTS accounts (
    pubkey         TEXT NOT NULL,
    stagenet_id    TEXT NOT NULL,
    lamports       INTEGER NOT NULL DEFAULT 0,
    data           BLOB NOT NULL DEFAULT (x''),
    owner          TEXT NOT NULL,
    executable     INTEGER NOT NULL DEFAULT 0,
    rent_epoch     INTEGER NOT NULL DEFAULT 0,
    sync_state     TEXT NOT NULL DEFAULT 'Unknown', -- 'Unknown' | 'Clean' | 'Dirty' | 'Pinned'
    category       TEXT,                            -- 'Oracle' | 'Program' | 'TokenMint' | 'Data' | NULL
    fetched_at     TEXT,
    modified_at    TEXT,
    PRIMARY KEY (pubkey, stagenet_id),
    FOREIGN KEY (stagenet_id) REFERENCES stagenets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_accounts_stagenet ON accounts(stagenet_id);
CREATE INDEX IF NOT EXISTS idx_accounts_sync_state ON accounts(stagenet_id, sync_state);
CREATE INDEX IF NOT EXISTS idx_accounts_category ON accounts(stagenet_id, category);

CREATE TABLE IF NOT EXISTS transactions (
    signature      TEXT NOT NULL,
    stagenet_id    TEXT NOT NULL,
    slot           INTEGER NOT NULL DEFAULT 0,
    success        INTEGER NOT NULL DEFAULT 1,
    fee            INTEGER NOT NULL DEFAULT 0,
    compute_units  INTEGER,
    programs       TEXT,                            -- JSON array of program IDs invoked
    logs           TEXT,                            -- JSON array of log messages
    err            TEXT,                            -- error string when success = 0
    created_at     TEXT NOT NULL,
    PRIMARY KEY (signature, stagenet_id),
    FOREIGN KEY (stagenet_id) REFERENCES stagenets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_transactions_stagenet
    ON transactions(stagenet_id, created_at DESC);
