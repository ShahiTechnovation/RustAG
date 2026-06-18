//! Control-plane configuration (environment-driven).

use std::path::PathBuf;

/// Runtime configuration for the cloud control plane.
#[derive(Debug, Clone)]
pub struct CloudConfig {
    /// Address the control-plane HTTP server binds to.
    pub bind_addr: String,
    /// SQLite path (or `:memory:`) for the control-plane database.
    pub control_db: String,
    /// Root directory under which each stagenet gets an isolated working dir.
    pub data_root: PathBuf,
    /// Path to the `rustag` binary used to run hosted stagenets.
    pub rustag_bin: String,
    /// Base domain used to render per-stagenet URLs (display only).
    pub base_domain: String,
    /// Inclusive start of the port pool allocated to hosted stagenets.
    pub port_start: u16,
    /// Exclusive end of the port pool.
    pub port_end: u16,
    /// Default mainnet RPC endpoint passed to hosted stagenets.
    pub default_mainnet_rpc: String,
    /// Seconds to wait for a new stagenet to report healthy before giving up.
    pub start_timeout_secs: u64,
    /// Max active stagenets a single tenant may own (resource-exhaustion guard).
    pub max_stagenets_per_tenant: usize,
}

impl CloudConfig {
    /// Build a config from environment variables, applying sensible defaults.
    pub fn from_env() -> Self {
        let var = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        Self {
            bind_addr: var("RUSTAG_CLOUD_BIND", "127.0.0.1:8080"),
            control_db: var("RUSTAG_CLOUD_DB", "rustag-cloud.sqlite"),
            data_root: PathBuf::from(var("RUSTAG_CLOUD_DATA", ".rustag-cloud")),
            rustag_bin: var("RUSTAG_BIN", "rustag"),
            base_domain: var("RUSTAG_CLOUD_DOMAIN", "localhost"),
            port_start: var("RUSTAG_CLOUD_PORT_START", "20000")
                .parse()
                .unwrap_or(20000),
            port_end: var("RUSTAG_CLOUD_PORT_END", "21000")
                .parse()
                .unwrap_or(21000),
            default_mainnet_rpc: var("RUSTAG_MAINNET_RPC", rustag_core::DEFAULT_MAINNET_RPC),
            start_timeout_secs: var("RUSTAG_CLOUD_START_TIMEOUT", "20")
                .parse()
                .unwrap_or(20),
            max_stagenets_per_tenant: var("RUSTAG_CLOUD_MAX_PER_TENANT", "25")
                .parse()
                .unwrap_or(25),
        }
    }
}
