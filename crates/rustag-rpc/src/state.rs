//! Shared server state.

use std::sync::Arc;

use tokio::sync::RwLock;

use rustag_core::Stagenet;

/// State shared by every JSON-RPC, REST, and WebSocket handler.
///
/// The stagenet is behind an async `RwLock` because most operations need
/// `&mut Stagenet` (the lazy mirror mutates the SVM, cache, and store).
#[derive(Clone)]
pub struct AppState {
    pub stagenet: Arc<RwLock<Stagenet>>,
}

impl AppState {
    pub fn new(stagenet: Arc<RwLock<Stagenet>>) -> Self {
        Self { stagenet }
    }
}
