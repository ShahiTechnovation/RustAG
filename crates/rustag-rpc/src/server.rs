//! Server wiring: spins up the JSON-RPC, WebSocket, and REST servers plus the
//! background oracle-sync task.

use std::net::{Ipv4Addr, SocketAddr};
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
    let (rpc_port, ws_port, api_port, oracle_secs) = {
        let sn = stagenet.read().await;
        let c = sn.config();
        (c.rpc_port, c.ws_port, c.api_port, c.oracle_sync_interval)
    };

    let state = AppState::new(Arc::clone(&stagenet));

    // Keep CLEAN oracle accounts fresh in the background.
    rustag_core::spawn_oracle_sync(Arc::clone(&stagenet), Duration::from_secs(oracle_secs));

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

    let addrs = ServerAddrs {
        rpc: SocketAddr::from((Ipv4Addr::LOCALHOST, rpc_port)),
        ws: SocketAddr::from((Ipv4Addr::LOCALHOST, ws_port)),
        api: SocketAddr::from((Ipv4Addr::LOCALHOST, api_port)),
    };

    tracing::info!(
        rpc = %addrs.rpc,
        ws = %addrs.ws,
        api = %addrs.api,
        "RustAG servers starting — point your cluster URL at the RPC address"
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
