use std::path::PathBuf;

use sqlx::SqlitePool;
use tracing::{info, warn};

use crate::error::StorageError;
use crate::types::Session;

use super::checksum::validate_checksum_with_details;
use super::lap_time::validate_lap_times;
use super::monotonicity::{
    compute_lap_sample_counts, has_excessive_violations, validate_lap_distance_monotonicity,
};
use super::range_check::{has_excessive_range_violations, validate_channel_ranges};
use super::{
    IntegrityReport, STATUS_CHECKSUM_FAILED, STATUS_DISTANCE_ANOMALY, STATUS_NOT_VALIDATED,
    STATUS_RANGE_VIOLATION, STATUS_VALID,
};

/// Run the full validation suite for a session and persist results.
pub async fn validate_session_integrity(
    pool: &SqlitePool,
    session: &Session,
) -> Result<IntegrityReport, StorageError> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    let mut checksum_valid: Option<bool> = None;
    let mut monotonicity_violations = Vec::new();
    let mut range_violations = std::collections::HashMap::new();
    let mut total_samples: usize = 0;
    let mut lap_sample_counts = std::collections::HashMap::new();
    let mut telemetry_read_failed = false;

    // Telemetry-based validations (only if telemetry exists)
    if let (Some(ref path_str), Some(ref expected_checksum)) =
        (&session.telemetry_path, &session.telemetry_checksum)
    {
        let path = PathBuf::from(path_str);
        if path.exists() {
            // Checksum validation
            let (is_valid, _actual) =
                validate_checksum_with_details(&path, expected_checksum, &session.id)?;
            checksum_valid = Some(is_valid);

            // If checksum passes, run data validations
            if is_valid {
                match crate::parquet::read_telemetry(&path, None) {
                    Ok(batch) => {
                        total_samples = batch.num_rows();
                        lap_sample_counts = compute_lap_sample_counts(&batch);
                        monotonicity_violations = validate_lap_distance_monotonicity(&batch);
                        range_violations = validate_channel_ranges(&batch);
                    }
                    Err(e) => {
                        warn!(
                            session_id = session.id,
                            error = %e,
                            "Failed to read telemetry for validation, retaining not_validated status"
                        );
                        telemetry_read_failed = true;
                    }
                }
            }
        }
    }

    // Lap time validation
    let laps = crate::sqlite::queries::laps::get_laps_for_session(pool, &session.id).await?;
    let lap_time_violations = validate_lap_times(&laps);

    // Flag invalid laps in the database
    for violation in &lap_time_violations {
        sqlx::query(
            "UPDATE lap_summaries SET completion_status = 'invalid' WHERE session_id = $1 AND lap_number = $2",
        )
        .bind(&session.id)
        .bind(violation.lap_number)
        .execute(pool)
        .await?;
    }

    // Determine overall status (worst wins)
    let status = if checksum_valid == Some(false) {
        STATUS_CHECKSUM_FAILED.to_string()
    } else if telemetry_read_failed {
        // Telemetry exists but couldn't be read -- don't mark as valid
        STATUS_NOT_VALIDATED.to_string()
    } else if has_excessive_range_violations(&range_violations) {
        STATUS_RANGE_VIOLATION.to_string()
    } else if has_excessive_violations(&monotonicity_violations, &lap_sample_counts, total_samples)
    {
        STATUS_DISTANCE_ANOMALY.to_string()
    } else {
        STATUS_VALID.to_string()
    };

    let report = IntegrityReport {
        status: status.clone(),
        checksum_valid,
        monotonicity_violations,
        range_violations,
        lap_time_violations,
        validated_at: now.clone(),
    };

    // Persist results
    let details_json = serde_json::to_string(&report).unwrap_or_default();
    sqlx::query(
        "UPDATE sessions SET integrity_status = $1, integrity_details = $2, integrity_validated_at = $3, updated_at = $3 WHERE id = $4",
    )
    .bind(&status)
    .bind(&details_json)
    .bind(&now)
    .bind(&session.id)
    .execute(pool)
    .await?;

    Ok(report)
}

/// Find sessions that haven't been validated yet and run validation on each.
pub async fn validate_unvalidated_sessions(pool: &SqlitePool) -> Result<u32, StorageError> {
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE (integrity_status = 'not_validated' OR integrity_status IS NULL) AND telemetry_path IS NOT NULL AND status != 'deleted'"
    )
    .fetch_all(pool)
    .await?;

    if sessions.is_empty() {
        return Ok(0);
    }

    info!(
        "Starting integrity validation for {} unvalidated sessions",
        sessions.len()
    );

    let mut validated = 0u32;
    for (i, session) in sessions.iter().enumerate() {
        info!(
            "Validating session {} of {}: {} ({})",
            i + 1,
            sessions.len(),
            session.id,
            session.track_name
        );
        match validate_session_integrity(pool, session).await {
            Ok(report) => {
                info!(
                    session_id = session.id,
                    status = report.status,
                    "Session validation complete"
                );
                validated += 1;
            }
            Err(e) => {
                warn!(
                    session_id = session.id,
                    error = %e,
                    "Session validation failed"
                );
            }
        }
    }

    info!(
        "Integrity validation complete: {} of {} sessions validated",
        validated,
        sessions.len()
    );

    Ok(validated)
}
