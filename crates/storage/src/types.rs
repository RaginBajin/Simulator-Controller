use serde::{Deserialize, Serialize};

/// Full session record from the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub track_name: String,
    pub car_name: String,
    pub session_type: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub lap_count: i32,
    pub best_lap_time_ms: Option<i64>,
    pub status: String,
    pub telemetry_checksum: Option<String>,
    pub telemetry_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub previous_status: Option<String>,
    pub integrity_status: Option<String>,
    pub integrity_details: Option<String>,
    pub integrity_validated_at: Option<String>,
    pub import_source: Option<String>,
    pub import_format: Option<String>,
}

/// Input for creating a new session (excludes auto-generated fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSession {
    pub track_name: String,
    pub car_name: String,
    pub session_type: String,
    pub started_at: String,
}

/// Partial update fields for an existing session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdate {
    pub status: Option<String>,
    pub ended_at: Option<String>,
    pub lap_count: Option<i32>,
    pub best_lap_time_ms: Option<i64>,
}

/// Lightweight session summary for list queries.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub track_name: String,
    pub car_name: String,
    pub session_type: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub lap_count: i32,
    pub best_lap_time_ms: Option<i64>,
    pub status: String,
    pub integrity_status: Option<String>,
    pub import_source: Option<String>,
}

/// Composite session detail: session + laps + debrief.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub session: Session,
    pub laps: Vec<LapSummary>,
    pub debrief: Option<AiDebrief>,
    pub has_telemetry: bool,
}

/// Full lap summary record from the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LapSummary {
    pub id: String,
    pub session_id: String,
    pub lap_number: i32,
    pub lap_time_ms: i64,
    pub is_valid: bool,
    pub completion_status: String,
    pub created_at: String,
}

/// Input for creating a new lap summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewLap {
    pub session_id: String,
    pub lap_number: i32,
    pub lap_time_ms: i64,
    pub is_valid: bool,
    pub completion_status: String,
}

/// AI debrief record from the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AiDebrief {
    pub id: String,
    pub session_id: String,
    pub coaching_text: Option<String>,
    pub insights_json: Option<String>,
    pub recommendations_json: Option<String>,
    pub provider_name: Option<String>,
    pub model_name: Option<String>,
    pub trigger_type: Option<String>,
    pub data_range_from_ms: Option<i64>,
    pub data_range_to_ms: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a new AI debrief.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAiDebrief {
    pub session_id: String,
    pub coaching_text: Option<String>,
    pub insights_json: Option<String>,
    pub recommendations_json: Option<String>,
    pub provider_name: Option<String>,
    pub model_name: Option<String>,
    pub trigger_type: Option<String>,
    pub data_range_from_ms: Option<i64>,
    pub data_range_to_ms: Option<i64>,
}

/// Partial update fields for a debrief.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebriefUpdate {
    pub coaching_text: Option<String>,
    pub insights_json: Option<String>,
    pub recommendations_json: Option<String>,
}

/// Result of a soft delete operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResult {
    pub session_id: String,
    pub deleted_at: String,
}

/// Result of a restore operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub session_id: String,
    pub restored_status: String,
}

/// Options for listing sessions with pagination.
#[derive(Debug, Clone)]
pub struct ListOptions {
    pub limit: i64,
    pub offset: i64,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

/// Filter options for session list queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptions {
    pub track: Option<String>,
    pub car: Option<String>,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
}

/// Aggregate statistics for the current filter set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    pub total_sessions: i64,
    pub best_lap_time_ms: Option<i64>,
    pub latest_session_date: Option<String>,
}
