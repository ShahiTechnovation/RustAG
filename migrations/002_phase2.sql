-- RustAG Phase 2 schema additions (SQLite; Postgres-portable).
--
-- Adds two subsystems on top of the Phase 1 schema:
--   * `schedules` — the Activity Scheduler (recurring on-chain actions).
--   * `metrics`   — the Analytics time-series (one row per sample per series).
--
-- As in Phase 1: no SQLite-only column types, timestamps as ISO-8601 TEXT,
-- booleans as INTEGER(0/1), so the same DDL migrates cleanly to Postgres /
-- TimescaleDB (where `metrics` becomes a hypertable on `recorded_at`).

CREATE TABLE IF NOT EXISTS schedules (
    id             TEXT PRIMARY KEY,               -- UUID v4
    stagenet_id    TEXT NOT NULL,
    name           TEXT NOT NULL,
    schedule       TEXT NOT NULL,                  -- "@every 30s" | 5-field cron
    action_json    TEXT NOT NULL,                  -- serialized Activity action
    enabled        INTEGER NOT NULL DEFAULT 1,
    run_count      INTEGER NOT NULL DEFAULT 0,
    last_run       TEXT,                           -- ISO-8601, NULL until first fire
    last_status    TEXT,                           -- "ok" | error string
    last_signature TEXT,                           -- signature of the last fired tx
    created_at     TEXT NOT NULL,
    FOREIGN KEY (stagenet_id) REFERENCES stagenets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_schedules_stagenet ON schedules(stagenet_id);
CREATE INDEX IF NOT EXISTS idx_schedules_enabled  ON schedules(stagenet_id, enabled);

CREATE TABLE IF NOT EXISTS metrics (
    stagenet_id  TEXT NOT NULL,
    series       TEXT NOT NULL,                    -- e.g. 'tx_total', 'accounts', 'dirty', 'cu_total'
    value        REAL NOT NULL,
    recorded_at  TEXT NOT NULL,                    -- ISO-8601 (the hypertable time column)
    FOREIGN KEY (stagenet_id) REFERENCES stagenets(id) ON DELETE CASCADE
);

-- Time-ordered reads per (stagenet, series) drive every analytics query.
CREATE INDEX IF NOT EXISTS idx_metrics_series
    ON metrics(stagenet_id, series, recorded_at DESC);
