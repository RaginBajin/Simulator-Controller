use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use storage::Database;
use telemetry_engine::ConnectionManager;

/// Application state shared across Tauri commands.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub data_dir: PathBuf,
    pub connection_manager: Arc<Mutex<ConnectionManager>>,
}

impl AppState {
    pub fn new(db: Database, data_dir: PathBuf) -> Self {
        Self {
            db,
            data_dir,
            connection_manager: Arc::new(Mutex::new(ConnectionManager::new())),
        }
    }
}
