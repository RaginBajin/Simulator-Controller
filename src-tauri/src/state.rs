use std::path::PathBuf;

use storage::Database;

/// Application state shared across Tauri commands.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(db: Database, data_dir: PathBuf) -> Self {
        Self { db, data_dir }
    }
}
