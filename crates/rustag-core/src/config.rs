//! Stagenet configuration.

use serde::{Deserialize, Serialize};

/// Default public mainnet endpoint (rate-limited; override in production).
pub const DEFAULT_MAINNET_RPC: &str = "https://api.mainnet-beta.solana.com";

/// Per-stagenet configuration. Serialized into the `stagenets.config_json`
/// column so a stagenet can be fully reconstructed after a restart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagenetConfig {
    /// Human-friendly stagenet name.
    pub name: String,
    /// Network being mirrored (informational).
    pub network: String,
    /// JSON-RPC port.
    pub rpc_port: u16,
    /// WebSocket port.
    pub ws_port: u16,
    /// REST API port (for the dashboard).
    pub api_port: u16,
    /// Mainnet RPC endpoint used by the lazy mirror.
    pub mainnet_rpc: String,
    /// Whether lazy mainnet mirroring is enabled.
    pub mirror_enabled: bool,
    /// Re-sync interval for CLEAN oracle accounts (seconds).
    pub oracle_sync_interval: u64,
    /// Re-sync interval for other CLEAN accounts (seconds).
    pub default_sync_interval: u64,
    /// Max mainnet RPC requests per second.
    pub max_rps: u32,
    /// Soft cap on accounts per stagenet.
    pub max_accounts: u64,
    /// Compute-unit ceiling per transaction (Solana's real limit).
    pub max_compute_units: u64,
    /// Programs/oracles to preload on startup.
    pub preload: Vec<String>,
    /// SQLite database path.
    pub db_path: String,
    /// CORS origins allowed to call the REST API.
    pub cors_origins: Vec<String>,

    // --- Phase 2 ---------------------------------------------------------
    /// Enable the real-time push mirror (Yellowstone gRPC / `accountSubscribe`
    /// WebSocket) instead of relying solely on the 30s polling loop.
    #[serde(default)]
    pub realtime_enabled: bool,
    /// Mainnet WebSocket endpoint for the real-time mirror (e.g. a Helius
    /// `wss://...` URL). Required when `realtime_enabled` is true.
    #[serde(default)]
    pub realtime_ws: Option<String>,
    /// Run the Activity Scheduler background loop on startup.
    #[serde(default = "default_true")]
    pub scheduler_enabled: bool,
    /// Capture analytics metric samples on startup.
    #[serde(default = "default_true")]
    pub metrics_enabled: bool,
    /// How often the analytics sampler captures a snapshot (seconds).
    #[serde(default = "default_metrics_interval")]
    pub metrics_interval: u64,
}

fn default_true() -> bool {
    true
}

fn default_metrics_interval() -> u64 {
    60
}

impl Default for StagenetConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            network: "mainnet-beta".to_string(),
            rpc_port: 8899,
            ws_port: 8900,
            api_port: 9000,
            mainnet_rpc: DEFAULT_MAINNET_RPC.to_string(),
            mirror_enabled: true,
            oracle_sync_interval: 30,
            default_sync_interval: 300,
            max_rps: 10,
            max_accounts: 50_000,
            max_compute_units: 1_400_000,
            preload: Vec::new(),
            db_path: ".rustag/db.sqlite".to_string(),
            cors_origins: vec!["http://localhost:3000".to_string()],
            realtime_enabled: false,
            realtime_ws: None,
            scheduler_enabled: true,
            metrics_enabled: true,
            metrics_interval: 60,
        }
    }
}

impl StagenetConfig {
    /// A config suitable for offline local tests: mirror disabled, in-memory DB.
    pub fn local(name: &str) -> Self {
        Self {
            name: name.to_string(),
            mirror_enabled: false,
            db_path: ":memory:".to_string(),
            ..Default::default()
        }
    }

    /// The JSON-RPC URL a client should point at.
    pub fn rpc_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.rpc_port)
    }

    /// The WebSocket URL a client should point at.
    pub fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.ws_port)
    }
}
