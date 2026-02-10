use std::collections::HashMap;
use std::path::{Path, PathBuf};

use arrow::record_batch::RecordBatch;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{ConnectOptions, Executor, SqlitePool};
use tracing::info;

use crate::error::StorageError;
use crate::parquet;
use crate::types::*;

const MAX_CONNECTIONS: u32 = 5;

/// Core database handle wrapping a SQLite connection pool.
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create or open a SQLite database at the given path, run migrations, and return a ready handle.
    pub async fn init(path: &Path) -> Result<Self, StorageError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("Initializing database at: {}", path.display());

        let connect_options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .disable_statement_logging();

        let pool = SqlitePoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    conn.execute("PRAGMA foreign_keys = ON").await?;
                    conn.execute("PRAGMA journal_mode = WAL").await?;
                    Ok(())
                })
            })
            .connect_with(connect_options)
            .await?;

        // Run embedded migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        info!("Database initialized successfully");

        Ok(Self { pool })
    }

    // -- Session operations --

    pub async fn insert_session(&self, new: &NewSession) -> Result<Session, StorageError> {
        crate::sqlite::queries::sessions::insert_session(&self.pool, new).await
    }

    pub async fn get_session(&self, id: &str) -> Result<Option<Session>, StorageError> {
        crate::sqlite::queries::sessions::get_session(&self.pool, id).await
    }

    pub async fn list_sessions(
        &self,
        opts: ListOptions,
        filters: &FilterOptions,
    ) -> Result<Vec<SessionSummary>, StorageError> {
        crate::sqlite::queries::sessions::list_sessions(&self.pool, opts, filters).await
    }

    pub async fn update_session(
        &self,
        id: &str,
        update: SessionUpdate,
    ) -> Result<Session, StorageError> {
        crate::sqlite::queries::sessions::update_session(&self.pool, id, update).await
    }

    // -- Lap operations --

    pub async fn insert_lap(&self, new: &NewLap) -> Result<LapSummary, StorageError> {
        crate::sqlite::queries::laps::insert_lap(&self.pool, new).await
    }

    pub async fn get_laps_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<LapSummary>, StorageError> {
        crate::sqlite::queries::laps::get_laps_for_session(&self.pool, session_id).await
    }

    // -- Telemetry operations --

    /// Write telemetry data to Parquet, compute checksum, and update the session record.
    pub async fn write_telemetry(
        &self,
        session_id: &str,
        data_dir: &Path,
        batch: &RecordBatch,
        file_metadata: HashMap<String, String>,
    ) -> Result<parquet::TelemetryWriteResult, StorageError> {
        // Verify session exists
        let session = self.get_session(session_id).await?;
        if session.is_none() {
            return Err(StorageError::NotFound(session_id.to_string()));
        }

        // Write the Parquet file (atomic write with retry)
        let result = parquet::write_telemetry(session_id, data_dir, batch, file_metadata).await?;

        // Update session record with checksum and path
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();

        sqlx::query(
            "UPDATE sessions SET telemetry_checksum = $1, telemetry_path = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(&result.checksum)
        .bind(result.path.to_str())
        .bind(&now)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(result)
    }

    /// Read telemetry data for a session, validating the checksum.
    pub async fn read_telemetry(&self, session_id: &str) -> Result<RecordBatch, StorageError> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(session_id.to_string()))?;

        let path_str = session.telemetry_path.ok_or_else(|| {
            StorageError::NotFound(format!("No telemetry for session {}", session_id))
        })?;
        let path = PathBuf::from(&path_str);

        parquet::read_telemetry(&path, session.telemetry_checksum.as_deref())
    }

    /// Read a windowed slice of telemetry data by lap distance range.
    pub async fn read_telemetry_window(
        &self,
        session_id: &str,
        start_distance: f64,
        end_distance: f64,
    ) -> Result<RecordBatch, StorageError> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(session_id.to_string()))?;

        let path_str = session.telemetry_path.ok_or_else(|| {
            StorageError::NotFound(format!("No telemetry for session {}", session_id))
        })?;
        let path = PathBuf::from(&path_str);

        parquet::read_telemetry_window(
            &path,
            start_distance,
            end_distance,
            session.telemetry_checksum.as_deref(),
        )
    }

    /// Validate the telemetry file checksum for a session.
    pub async fn validate_telemetry(&self, session_id: &str) -> Result<bool, StorageError> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(session_id.to_string()))?;

        let path_str = session.telemetry_path.ok_or_else(|| {
            StorageError::NotFound(format!("No telemetry for session {}", session_id))
        })?;
        let checksum = session.telemetry_checksum.ok_or_else(|| {
            StorageError::NotFound(format!("No checksum for session {}", session_id))
        })?;

        parquet::validate_checksum(Path::new(&path_str), &checksum)
    }

    // -- Soft delete / restore operations --

    pub async fn soft_delete_session(&self, id: &str) -> Result<DeleteResult, StorageError> {
        crate::sqlite::queries::sessions::soft_delete_session(&self.pool, id).await
    }

    pub async fn restore_session(&self, id: &str) -> Result<RestoreResult, StorageError> {
        crate::sqlite::queries::sessions::restore_session(&self.pool, id).await
    }

    pub async fn list_deleted_sessions(&self) -> Result<Vec<SessionSummary>, StorageError> {
        crate::sqlite::queries::sessions::list_deleted_sessions(&self.pool).await
    }

    // -- Filter / stats queries --

    pub async fn get_distinct_tracks(&self) -> Result<Vec<String>, StorageError> {
        crate::sqlite::queries::sessions::get_distinct_tracks(&self.pool).await
    }

    pub async fn get_distinct_cars(&self) -> Result<Vec<String>, StorageError> {
        crate::sqlite::queries::sessions::get_distinct_cars(&self.pool).await
    }

    pub async fn get_session_stats(
        &self,
        filters: &FilterOptions,
    ) -> Result<SessionStats, StorageError> {
        crate::sqlite::queries::sessions::get_session_stats(&self.pool, filters).await
    }

    // -- Session detail (composite) --

    pub async fn get_session_detail(&self, id: &str) -> Result<SessionDetail, StorageError> {
        crate::sqlite::queries::sessions::get_session_detail(&self.pool, id).await
    }

    // -- AI debrief operations --

    pub async fn insert_debrief(&self, new: &NewAiDebrief) -> Result<AiDebrief, StorageError> {
        crate::sqlite::queries::ai_results::insert_debrief(&self.pool, new).await
    }

    pub async fn get_debrief_for_session(
        &self,
        session_id: &str,
    ) -> Result<Option<AiDebrief>, StorageError> {
        crate::sqlite::queries::ai_results::get_debrief_for_session(&self.pool, session_id).await
    }

    pub async fn update_debrief(
        &self,
        id: &str,
        update: DebriefUpdate,
    ) -> Result<AiDebrief, StorageError> {
        crate::sqlite::queries::ai_results::update_debrief(&self.pool, id, update).await
    }

    // -- Trash cleanup --

    pub async fn find_expired_deleted_sessions(
        &self,
        max_age_days: u32,
    ) -> Result<Vec<Session>, StorageError> {
        crate::sqlite::queries::sessions::find_expired_deleted_sessions(&self.pool, max_age_days)
            .await
    }

    pub async fn permanently_delete_session(&self, id: &str) -> Result<(), StorageError> {
        crate::sqlite::queries::sessions::permanently_delete_session(&self.pool, id).await
    }

    // -- Import operations --

    pub async fn find_duplicate_session(
        &self,
        track: &str,
        car: &str,
        started_at: &str,
        tolerance_secs: i64,
    ) -> Result<Option<String>, StorageError> {
        crate::sqlite::queries::sessions::find_duplicate_session(
            &self.pool,
            track,
            car,
            started_at,
            tolerance_secs,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn insert_imported_session(
        &self,
        id: &str,
        track_name: &str,
        car_name: &str,
        session_type: &str,
        started_at: &str,
        ended_at: Option<&str>,
        lap_count: i32,
        best_lap_time_ms: Option<i64>,
        import_source: &str,
        import_format: &str,
        raw_session_type: Option<&str>,
    ) -> Result<Session, StorageError> {
        crate::sqlite::queries::sessions::insert_imported_session(
            &self.pool,
            id,
            track_name,
            car_name,
            session_type,
            started_at,
            ended_at,
            lap_count,
            best_lap_time_ms,
            import_source,
            import_format,
            raw_session_type,
        )
        .await
    }

    // -- Integrity validation --

    /// Run the full validation suite for a single session.
    pub async fn validate_session_integrity(
        &self,
        session_id: &str,
    ) -> Result<crate::validation::IntegrityReport, StorageError> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(session_id.to_string()))?;

        crate::validation::orchestrator::validate_session_integrity(&self.pool, &session).await
    }

    /// Find and validate all sessions that haven't been validated yet.
    pub async fn validate_unvalidated_sessions(&self) -> Result<u32, StorageError> {
        crate::validation::orchestrator::validate_unvalidated_sessions(&self.pool).await
    }

    /// Expose pool for query functions that need direct pool access.
    #[allow(dead_code)]
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Expose pool for integration tests that need raw SQL access.
    #[cfg(feature = "test-utils")]
    pub fn pool_for_testing(&self) -> &SqlitePool {
        &self.pool
    }
}
