use std::path::PathBuf;

use storage::import::types::{BatchImportResult, ImportResult};
use tauri::{Emitter, State, Window};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn import_session(
    file_path: String,
    force_duplicate: Option<bool>,
    state: State<'_, AppState>,
) -> Result<ImportResult, AppError> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(AppError {
            code: "FILE_NOT_FOUND".to_string(),
            message: format!("File not found: {}", file_path),
            details: None,
            retryable: false,
        });
    }

    telemetry_engine::import_session(
        &state.db,
        &state.data_dir,
        &path,
        force_duplicate.unwrap_or(false),
    )
    .await
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn import_session_batch(
    file_paths: Vec<String>,
    force_duplicate: Option<bool>,
    state: State<'_, AppState>,
    window: Window,
) -> Result<BatchImportResult, AppError> {
    let paths: Vec<PathBuf> = file_paths.iter().map(PathBuf::from).collect();

    // Validate all files exist upfront before starting the batch import
    for (i, path) in paths.iter().enumerate() {
        if !path.exists() {
            return Err(AppError {
                code: "FILE_NOT_FOUND".to_string(),
                message: format!("File not found: {}", file_paths[i]),
                details: None,
                retryable: false,
            });
        }
    }

    telemetry_engine::import_session_batch(
        &state.db,
        &state.data_dir,
        &paths,
        force_duplicate.unwrap_or(false),
        |progress| {
            let _ = window.emit("import:progress", &progress);
        },
    )
    .await
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn get_supported_import_formats() -> Result<Vec<String>, AppError> {
    Ok(telemetry_engine::supported_import_formats())
}
