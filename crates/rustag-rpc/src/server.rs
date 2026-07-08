//! Server wiring: spins up the JSON-RPC, WebSocket, and REST servers plus the
//! background oracle-sync task.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use axum::routing::{get, post};
use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use rustag_core::Stagenet;

use crate::error::RpcServerError;
use crate::state::AppState;

/// Bound listener addresses, returned so callers can print them.
#[derive(Debug, Clone, Copy)]
pub struct ServerAddrs {
    pub rpc: SocketAddr,
    pub ws: SocketAddr,
    pub api: SocketAddr,
}

/// Start all servers and block until one of them stops (or errors).
pub async fn serve(stagenet: Arc<RwLock<Stagenet>>) -> Result<(), RpcServerError> {
    let (rpc_port, ws_port, api_port, oracle_secs, store, scheduler_on, metrics_on, metrics_secs) = {
        let sn = stagenet.read().await;
        let c = sn.config();
        (
            c.rpc_port,
            c.ws_port,
            c.api_port,
            c.oracle_sync_interval,
            sn.store(),
            c.scheduler_enabled,
            c.metrics_enabled,
            c.metrics_interval,
        )
    };

    let state = AppState::new(Arc::clone(&stagenet));

    // Keep CLEAN oracle accounts fresh in the background.
    rustag_core::spawn_oracle_sync(Arc::clone(&stagenet), Duration::from_secs(oracle_secs));

    // Phase 2 background workers.
    if metrics_on {
        rustag_core::spawn_metrics_sampler(
            Arc::clone(&stagenet),
            Duration::from_secs(metrics_secs),
        );
        tracing::info!(interval_secs = metrics_secs, "analytics sampler enabled");
    }
    if scheduler_on {
        rustag_scheduler::Scheduler::spawn(Arc::clone(&stagenet), Arc::clone(&store));
    }
    spawn_realtime(Arc::clone(&stagenet)).await;

    let rpc_app = Router::new()
        .route("/", post(crate::jsonrpc::handle))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    let ws_app = Router::new()
        .route("/", get(crate::ws::handler))
        .with_state(state.clone());

    let rest_app = crate::rest::router(state.clone())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // The REST/API listener is the only one intended for public exposure (the
    // dashboard talks to it). Its bind host is configurable via RUSTAG_BIND_HOST
    // (set 0.0.0.0 in a container behind a proxy) and its port honors $PORT when
    // a platform such as Render injects one. The JSON-RPC and WebSocket servers
    // stay on loopback so a container never exposes them by default; a public
    // cluster URL is an explicit, separate opt-in.
    let api_host = api_bind_host();
    let api_port = resolve_api_port(api_port, std::env::var("PORT").ok());
    let addrs = ServerAddrs {
        rpc: SocketAddr::from((Ipv4Addr::LOCALHOST, rpc_port)),
        ws: SocketAddr::from((Ipv4Addr::LOCALHOST, ws_port)),
        api: SocketAddr::from((api_host, api_port)),
    };

    tracing::info!(
        rpc = %addrs.rpc,
        ws = %addrs.ws,
        api = %addrs.api,
        "RustAG servers starting - point your cluster URL at the RPC address"
    );

    tokio::try_join!(
        serve_app(addrs.rpc, rpc_app),
        serve_app(addrs.ws, ws_app),
        serve_app(addrs.api, rest_app),
    )?;
    Ok(())
}

async fn serve_app(addr: SocketAddr, app: Router) -> Result<(), RpcServerError> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| RpcServerError::Bind(addr, e))?;
    axum::serve(listener, app)
        .await
        .map_err(RpcServerError::Serve)?;
    Ok(())
}

/// Resolve the bind IP for the public REST/API listener from `RUSTAG_BIND_HOST`.
fn api_bind_host() -> IpAddr {
    parse_bind_host(std::env::var("RUSTAG_BIND_HOST").ok())
}

/// Parse a bind-host override, defaulting to loopback (safe for local dev). Set
/// `RUSTAG_BIND_HOST=0.0.0.0` to expose the API from inside a container behind a
/// platform proxy. An unparseable value falls back to loopback with a warning.
fn parse_bind_host(raw: Option<String>) -> IpAddr {
    let loopback = IpAddr::V4(Ipv4Addr::LOCALHOST);
    match raw {
        Some(v) if !v.trim().is_empty() => v.trim().parse::<IpAddr>().unwrap_or_else(|_| {
            tracing::warn!(value = %v, "RUSTAG_BIND_HOST is not a valid IP address; using 127.0.0.1");
            loopback
        }),
        _ => loopback,
    }
}

/// Prefer a platform-injected `$PORT` (Render/Heroku expose exactly one port and
/// route the public URL to it) over the configured API port; fall back to the
/// configured port when `$PORT` is absent or not a valid `u16`.
fn resolve_api_port(configured: u16, port_env: Option<String>) -> u16 {
    port_env
        .and_then(|p| p.trim().parse::<u16>().ok())
        .unwrap_or(configured)
}

/// Start the real-time push mirror (Phase 2). Subscribes to the oracle registry
/// over `accountSubscribe` and applies pushed updates with a reconnect loop.
/// Only present when built with the `realtime` feature.
#[cfg(feature = "realtime")]
async fn spawn_realtime(stagenet: Arc<RwLock<Stagenet>>) {
    use tokio::sync::mpsc;

    let (enabled, ws) = {
        let sn = stagenet.read().await;
        let c = sn.config();
        (c.realtime_enabled, c.realtime_ws.clone())
    };
    if !enabled {
        return;
    }
    let Some(ws_url) = ws else {
        tracing::warn!("realtime_enabled is set but realtime_ws is empty; using polling only");
        return;
    };

    let pubkeys = rustag_core::registry::oracle_pubkeys();
    let (tx, rx) = mpsc::channel(1024);
    rustag_core::spawn_realtime_apply(Arc::clone(&stagenet), rx);
    tracing::info!(count = pubkeys.len(), "real-time push mirror enabled");

    tokio::spawn(async move {
        loop {
            if let Err(e) =
                rustag_core::RealtimeMirror::run(&ws_url, pubkeys.clone(), tx.clone()).await
            {
                tracing::warn!(error = %e, "realtime mirror disconnected; retrying in 5s");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}

/// No-op when built without the `realtime` feature (polling stays the source of
/// truth). Warns if the config asked for realtime but the binary lacks it.
#[cfg(not(feature = "realtime"))]
async fn spawn_realtime(stagenet: Arc<RwLock<Stagenet>>) {
    if stagenet.read().await.config().realtime_enabled {
        tracing::warn!(
            "realtime_enabled is set but this build lacks the `realtime` feature; \
             rebuild with `--features realtime` for push updates. Using polling only."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_host_defaults_to_loopback() {
        assert_eq!(parse_bind_host(None), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(
            parse_bind_host(Some("  ".to_string())),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        );
    }

    #[test]
    fn bind_host_accepts_any_interface() {
        assert_eq!(
            parse_bind_host(Some("0.0.0.0".to_string())),
            IpAddr::V4(Ipv4Addr::UNSPECIFIED)
        );
    }

    #[test]
    fn bind_host_rejects_garbage_and_falls_back() {
        assert_eq!(
            parse_bind_host(Some("not-an-ip".to_string())),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        );
    }

    #[test]
    fn api_port_prefers_platform_port_env() {
        assert_eq!(resolve_api_port(9000, Some("10000".to_string())), 10000);
        assert_eq!(resolve_api_port(9000, None), 9000);
        assert_eq!(resolve_api_port(9000, Some("bogus".to_string())), 9000);
    }
}
