use serde::Serialize;
use storage::{DeleteResult, FilterOptions, IntegrityReport, ListOptions, RestoreResult, SessionDetail, SessionStats, SessionSummary};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn get_sessions(
    state: State<'_, AppState>,
    track: Option<String>,
    car: Option<String>,
    date_start: Option<String>,
    date_end: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<SessionSummary>, AppError> {
    let filters = FilterOptions {
        track,
        car,
        date_start,
        date_end,
    };
    let opts = ListOptions {
        limit: limit.unwrap_or(50),
        offset: offset.unwrap_or(0),
    };
    state
        .db
        .list_sessions(opts, &filters)
        .await
        .map_err(AppError::from)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptionsResponse {
    pub tracks: Vec<String>,
    pub cars: Vec<String>,
}

#[tauri::command]
pub async fn get_filter_options(
    state: State<'_, AppState>,
) -> Result<FilterOptionsResponse, AppError> {
    let tracks = state.db.get_distinct_tracks().await.map_err(AppError::from)?;
    let cars = state.db.get_distinct_cars().await.map_err(AppError::from)?;
    Ok(FilterOptionsResponse { tracks, cars })
}

#[tauri::command]
pub async fn get_session_stats(
    state: State<'_, AppState>,
    track: Option<String>,
    car: Option<String>,
    date_start: Option<String>,
    date_end: Option<String>,
) -> Result<SessionStats, AppError> {
    let filters = FilterOptions {
        track,
        car,
        date_start,
        date_end,
    };
    state
        .db
        .get_session_stats(&filters)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn get_session_data(
    id: String,
    state: State<'_, AppState>,
) -> Result<SessionDetail, AppError> {
    state
        .db
        .get_session_detail(&id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn delete_session(
    id: String,
    state: State<'_, AppState>,
) -> Result<DeleteResult, AppError> {
    state
        .db
        .soft_delete_session(&id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn restore_session(
    id: String,
    state: State<'_, AppState>,
) -> Result<RestoreResult, AppError> {
    state
        .db
        .restore_session(&id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn validate_session(
    id: String,
    state: State<'_, AppState>,
) -> Result<IntegrityReport, AppError> {
    state
        .db
        .validate_session_integrity(&id)
        .await
        .map_err(AppError::from)
}
