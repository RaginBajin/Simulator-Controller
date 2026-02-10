mod commands;
mod error;
mod events;
mod state;

use tauri::Manager;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use state::AppState;
use storage::Database;

const TRASH_MAX_AGE_DAYS: u32 = 30;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().map_err(|e| {
                error!("Failed to resolve app_data_dir: {}", e);
                e
            })?;

            let db_path = app_data_dir.join("sessions.db");

            let db = tauri::async_runtime::block_on(async { Database::init(&db_path).await })
                .map_err(|e| {
                    error!(
                        "Failed to initialize database at {}: {}",
                        db_path.display(),
                        e
                    );
                    e
                })?;

            // Clean up orphaned .parquet.tmp files from previous crashes
            let telemetry_dir = app_data_dir.join("telemetry");
            if let Ok(cleaned) = storage::cleanup_orphaned_temps(&telemetry_dir) {
                if cleaned > 0 {
                    info!("Cleaned up {} orphaned telemetry temp files", cleaned);
                }
            }

            app.manage(AppState::new(db, app_data_dir.clone()));

            // Run trash cleanup asynchronously (does not block startup)
            let cleanup_state = app.state::<AppState>().inner().clone();
            let cleanup_data_dir = app_data_dir.clone();
            tauri::async_runtime::spawn(async move {
                cleanup_expired_trash(&cleanup_state, &cleanup_data_dir).await;
            });

            // Run integrity validation on unvalidated sessions (does not block startup)
            let validation_state = app.state::<AppState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                match validation_state.db.validate_unvalidated_sessions().await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Validated {} previously unvalidated sessions", count);
                        }
                    }
                    Err(e) => {
                        warn!("Startup validation failed: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session::get_sessions,
            commands::session::get_session_data,
            commands::session::delete_session,
            commands::session::restore_session,
            commands::session::get_filter_options,
            commands::session::get_session_stats,
            commands::session::validate_session,
            commands::telemetry::get_chart_data,
            commands::import::import_session,
            commands::import::import_session_batch,
            commands::import::get_supported_import_formats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn cleanup_expired_trash(state: &AppState, _data_dir: &std::path::Path) {
    match state
        .db
        .find_expired_deleted_sessions(TRASH_MAX_AGE_DAYS)
        .await
    {
        Ok(expired) => {
            if expired.is_empty() {
                return;
            }
            info!(
                "Found {} expired deleted sessions to clean up",
                expired.len()
            );
            let mut cleaned = 0u32;
            for session in &expired {
                // Remove Parquet file from .trash if it exists
                if let Some(ref path_str) = session.telemetry_path {
                    let path = std::path::Path::new(path_str);
                    if path.exists() {
                        if let Err(e) = std::fs::remove_file(path) {
                            warn!("Failed to remove trash file {}: {}", path_str, e);
                        }
                    }
                }
                // Permanently delete session record (CASCADE removes laps and debriefs)
                if let Err(e) = state.db.permanently_delete_session(&session.id).await {
                    error!("Failed to permanently delete session {}: {}", session.id, e);
                } else {
                    info!(
                        "Permanently deleted expired session: {} ({})",
                        session.id, session.track_name
                    );
                    cleaned += 1;
                }
            }
            info!(
                "Trash cleanup complete: permanently deleted {} sessions",
                cleaned
            );
        }
        Err(e) => {
            error!("Trash cleanup failed: {}", e);
        }
    }
}
