//! Process orchestration: each hosted stagenet runs as an isolated child
//! `rustag` process with its own working directory and allocated ports.
//!
//! Isolation model: the `rustag` CLI reads/writes `./.rustag` relative to its
//! working directory, so giving each stagenet a distinct `current_dir` gives it
//! a private data-plane database and PID file. The control plane keeps the live
//! [`Child`] handles so it can stop them, and records the pid for restart-time
//! reconciliation.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::CloudConfig;
use crate::error::{CloudError, Result};
use crate::store::{CloudStagenet, ControlPlaneStore};

/// Spawns and supervises hosted stagenet processes.
pub struct Orchestrator {
    config: Arc<CloudConfig>,
    children: Mutex<HashMap<String, Child>>,
    /// Serializes port allocation + reservation so two concurrent creates can
    /// never pick the same port triple (the allocate→insert TOCTOU window).
    alloc_lock: Mutex<()>,
    http: reqwest::Client,
}

impl Orchestrator {
    pub fn new(config: Arc<CloudConfig>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            config,
            children: Mutex::new(HashMap::new()),
            alloc_lock: Mutex::new(()),
            http,
        }
    }

    /// Allocate ports, create the per-stagenet data dir, run `rustag create`,
    /// launch `rustag start`, and wait for it to report healthy.
    pub async fn create_and_start(
        &self,
        store: &ControlPlaneStore,
        tenant_id: Uuid,
        slug: &str,
        name: &str,
        mainnet_rpc: &str,
    ) -> Result<CloudStagenet> {
        let work_dir = self.config.data_root.join(slug);
        std::fs::create_dir_all(&work_dir)?;

        // Reserve ports + slug atomically: allocate and persist the row under a
        // single lock so concurrent creates can't collide on the same ports, and
        // a duplicate slug surfaces as a clean 409 (not a 500).
        let (rpc, ws, api) = {
            let _guard = self.alloc_lock.lock().await;
            let (rpc, ws, api) = self.allocate_ports(store).await?;
            let rec = CloudStagenet {
                id: Uuid::new_v4(),
                tenant_id,
                slug: slug.to_string(),
                name: name.to_string(),
                status: "creating".to_string(),
                rpc_port: rpc,
                ws_port: ws,
                api_port: api,
                mainnet_rpc: mainnet_rpc.to_string(),
                pid: None,
                work_dir: work_dir.to_string_lossy().to_string(),
                created_at: Utc::now(),
                last_active: None,
            };
            store.insert_stagenet(&rec).await?;
            (rpc, ws, api)
        };

        // 1. Register the stagenet in its isolated working directory.
        let create_status = Command::new(&self.config.rustag_bin)
            .args([
                "create",
                slug,
                "--rpc-port",
                &rpc.to_string(),
                "--ws-port",
                &ws.to_string(),
                "--api-port",
                &api.to_string(),
                "--mainnet-rpc",
                mainnet_rpc,
            ])
            .current_dir(&work_dir)
            .status()
            .await
            .map_err(|e| CloudError::Orchestrator(format!("spawn `rustag create`: {e}")))?;
        if !create_status.success() {
            // Free the reserved ports + slug so a failed create doesn't leak them.
            store.delete_stagenet(slug).await.ok();
            return Err(CloudError::Orchestrator(
                "`rustag create` failed".to_string(),
            ));
        }

        // 2. Launch the long-running stagenet server. `kill_on_drop` ensures the
        //    child is reaped if this Orchestrator (and its children map) is
        //    dropped during a graceful shutdown, rather than orphaned.
        let child = Command::new(&self.config.rustag_bin)
            .args(["start", slug])
            .current_dir(&work_dir)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| CloudError::Orchestrator(format!("spawn `rustag start`: {e}")))?;
        let pid = child.id().map(|p| p as i64);
        self.children.lock().await.insert(slug.to_string(), child);

        // 3. Wait for the stagenet's REST health endpoint to come up.
        if self.await_healthy(api).await {
            store.set_status(slug, "running", pid).await?;
        } else {
            self.stop(slug).await.ok();
            store.delete_stagenet(slug).await.ok();
            return Err(CloudError::Orchestrator(
                "stagenet did not become healthy within the start timeout".to_string(),
            ));
        }

        store
            .get_stagenet(slug)
            .await?
            .ok_or_else(|| CloudError::NotFound(slug.to_string()))
    }

    /// Stop a stagenet's child process (best-effort).
    pub async fn stop(&self, slug: &str) -> Result<()> {
        let child = self.children.lock().await.remove(slug);
        if let Some(mut child) = child {
            let _ = child.start_kill();
            let _ = child.wait().await;
        }
        Ok(())
    }

    async fn await_healthy(&self, api_port: u16) -> bool {
        let url = format!("http://127.0.0.1:{api_port}/api/health");
        let attempts = self.config.start_timeout_secs.saturating_mul(4).max(4);
        for _ in 0..attempts {
            if let Ok(resp) = self.http.get(&url).send().await {
                if resp.status().is_success() {
                    return true;
                }
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        false
    }

    /// Find a free `(rpc, ws, api)` port triple in the configured range.
    async fn allocate_ports(&self, store: &ControlPlaneStore) -> Result<(u16, u16, u16)> {
        let used: HashSet<u16> = store.used_ports().await?.into_iter().collect();
        let mut base = self.config.port_start;
        while base.saturating_add(2) < self.config.port_end {
            let triple = (base, base + 1, base + 2);
            if ![triple.0, triple.1, triple.2]
                .iter()
                .any(|p| used.contains(p))
            {
                return Ok(triple);
            }
            base = base.saturating_add(10);
        }
        Err(CloudError::NoPorts)
    }
}
