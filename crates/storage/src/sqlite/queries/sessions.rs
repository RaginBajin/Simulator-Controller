use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::StorageError;
use crate::types::*;

pub async fn insert_session(pool: &SqlitePool, new: &NewSession) -> Result<Session, StorageError> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    sqlx::query(
        "INSERT INTO sessions (id, track_name, car_name, session_type, started_at, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'active', $6, $6)"
    )
    .bind(&id)
    .bind(&new.track_name)
    .bind(&new.car_name)
    .bind(&new.session_type)
    .bind(&new.started_at)
    .bind(&now)
    .execute(pool)
    .await?;

    get_session(pool, &id)
        .await?
        .ok_or_else(|| StorageError::NotFound(id))
}

pub async fn get_session(pool: &SqlitePool, id: &str) -> Result<Option<Session>, StorageError> {
    let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(session)
}

pub async fn list_sessions(
    pool: &SqlitePool,
    opts: ListOptions,
    filters: &FilterOptions,
) -> Result<Vec<SessionSummary>, StorageError> {
    let mut sql = String::from(
        "SELECT id, track_name, car_name, session_type, started_at, ended_at, lap_count, best_lap_time_ms, status, integrity_status, import_source
         FROM sessions
         WHERE status != 'deleted'"
    );

    let mut bind_idx = 0u32;
    let mut binds: Vec<String> = Vec::new();

    if let Some(track) = &filters.track {
        bind_idx += 1;
        sql.push_str(&format!(" AND track_name = ${}", bind_idx));
        binds.push(track.clone());
    }
    if let Some(car) = &filters.car {
        bind_idx += 1;
        sql.push_str(&format!(" AND car_name = ${}", bind_idx));
        binds.push(car.clone());
    }
    if let Some(date_start) = &filters.date_start {
        bind_idx += 1;
        sql.push_str(&format!(" AND started_at >= ${}", bind_idx));
        binds.push(date_start.clone());
    }
    if let Some(date_end) = &filters.date_end {
        bind_idx += 1;
        sql.push_str(&format!(" AND started_at <= ${}", bind_idx));
        binds.push(date_end.clone());
    }

    bind_idx += 1;
    let limit_idx = bind_idx;
    bind_idx += 1;
    let offset_idx = bind_idx;
    sql.push_str(&format!(
        " ORDER BY started_at DESC LIMIT ${} OFFSET ${}",
        limit_idx, offset_idx
    ));

    let mut query = sqlx::query_as::<_, SessionSummary>(&sql);
    for b in &binds {
        query = query.bind(b);
    }
    query = query.bind(opts.limit).bind(opts.offset);

    let sessions = query.fetch_all(pool).await?;
    Ok(sessions)
}

pub async fn list_deleted_sessions(pool: &SqlitePool) -> Result<Vec<SessionSummary>, StorageError> {
    let sessions = sqlx::query_as::<_, SessionSummary>(
        "SELECT id, track_name, car_name, session_type, started_at, ended_at, lap_count, best_lap_time_ms, status, integrity_status, import_source
         FROM sessions
         WHERE status = 'deleted'
         ORDER BY deleted_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(sessions)
}

pub async fn update_session(
    pool: &SqlitePool,
    id: &str,
    update: SessionUpdate,
) -> Result<Session, StorageError> {
    let existing = get_session(pool, id).await?;
    if existing.is_none() {
        return Err(StorageError::NotFound(id.to_string()));
    }
    let existing = existing.unwrap();

    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    let status = update.status.unwrap_or(existing.status);
    let ended_at = update.ended_at.or(existing.ended_at);
    let lap_count = update.lap_count.unwrap_or(existing.lap_count);
    let best_lap_time_ms = update.best_lap_time_ms.or(existing.best_lap_time_ms);

    sqlx::query(
        "UPDATE sessions SET status = $1, ended_at = $2, lap_count = $3, best_lap_time_ms = $4, updated_at = $5
         WHERE id = $6"
    )
    .bind(&status)
    .bind(&ended_at)
    .bind(lap_count)
    .bind(best_lap_time_ms)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?;

    get_session(pool, id)
        .await?
        .ok_or_else(|| StorageError::NotFound(id.to_string()))
}

pub async fn soft_delete_session(
    pool: &SqlitePool,
    id: &str,
) -> Result<DeleteResult, StorageError> {
    let session = get_session(pool, id)
        .await?
        .ok_or_else(|| StorageError::NotFound(id.to_string()))?;

    if session.status == "deleted" {
        return Err(StorageError::AlreadyDeleted(id.to_string()));
    }

    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    sqlx::query(
        "UPDATE sessions SET previous_status = status, status = 'deleted', deleted_at = $1, updated_at = $1
         WHERE id = $2"
    )
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(DeleteResult {
        session_id: id.to_string(),
        deleted_at: now,
    })
}

pub async fn restore_session(pool: &SqlitePool, id: &str) -> Result<RestoreResult, StorageError> {
    let session = get_session(pool, id)
        .await?
        .ok_or_else(|| StorageError::NotFound(id.to_string()))?;

    if session.status != "deleted" {
        return Err(StorageError::NotDeleted(id.to_string()));
    }

    // Check 30-day window -- deleted_at must be present for a deleted session
    let deleted_at = session.deleted_at.as_ref().ok_or_else(|| {
        StorageError::InvalidInput(format!(
            "Session {} is marked deleted but has no deleted_at timestamp",
            id
        ))
    })?;

    let deleted_time = chrono::DateTime::parse_from_rfc3339(deleted_at)
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(deleted_at, "%Y-%m-%dT%H:%M:%S%.3fZ")
                .map(|dt| dt.and_utc().fixed_offset())
        })
        .map_err(|_| {
            StorageError::InvalidInput(format!("Invalid deleted_at timestamp: {}", deleted_at))
        })?;

    let days_since = (chrono::Utc::now() - deleted_time.with_timezone(&chrono::Utc)).num_days();
    if days_since > 30 {
        return Err(StorageError::RestoreExpired(id.to_string()));
    }

    let restored_status = session
        .previous_status
        .unwrap_or_else(|| "completed".to_string());
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    sqlx::query(
        "UPDATE sessions SET status = $1, deleted_at = NULL, previous_status = NULL, updated_at = $2
         WHERE id = $3"
    )
    .bind(&restored_status)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(RestoreResult {
        session_id: id.to_string(),
        restored_status,
    })
}

pub async fn get_session_detail(
    pool: &SqlitePool,
    id: &str,
) -> Result<SessionDetail, StorageError> {
    let session = get_session(pool, id)
        .await?
        .ok_or_else(|| StorageError::NotFound(id.to_string()))?;

    let laps = crate::sqlite::queries::laps::get_laps_for_session(pool, id).await?;
    let debrief = crate::sqlite::queries::ai_results::get_debrief_for_session(pool, id).await?;

    let has_telemetry = session
        .telemetry_path
        .as_ref()
        .is_some_and(|p| !p.is_empty());

    Ok(SessionDetail {
        session,
        laps,
        debrief,
        has_telemetry,
    })
}

/// Find sessions deleted more than `max_age_days` ago for permanent cleanup.
pub async fn find_expired_deleted_sessions(
    pool: &SqlitePool,
    max_age_days: u32,
) -> Result<Vec<Session>, StorageError> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(max_age_days as i64))
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    let sessions = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE status = 'deleted' AND deleted_at IS NOT NULL AND deleted_at < $1"
    )
    .bind(&cutoff)
    .fetch_all(pool)
    .await?;

    Ok(sessions)
}

/// Permanently delete a session record (CASCADE removes laps and debriefs).
pub async fn permanently_delete_session(pool: &SqlitePool, id: &str) -> Result<(), StorageError> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Return distinct track names from non-deleted sessions.
pub async fn get_distinct_tracks(pool: &SqlitePool) -> Result<Vec<String>, StorageError> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT track_name FROM sessions WHERE status != 'deleted' ORDER BY track_name",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Return distinct car names from non-deleted sessions.
pub async fn get_distinct_cars(pool: &SqlitePool) -> Result<Vec<String>, StorageError> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT car_name FROM sessions WHERE status != 'deleted' ORDER BY car_name",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Find a potential duplicate session by track + car + started_at within tolerance.
pub async fn find_duplicate_session(
    pool: &SqlitePool,
    track: &str,
    car: &str,
    started_at: &str,
    tolerance_secs: i64,
) -> Result<Option<String>, StorageError> {
    let session_id = sqlx::query_scalar::<_, String>(
        "SELECT id FROM sessions WHERE track_name = $1 AND car_name = $2 AND ABS(julianday(started_at) - julianday($3)) * 86400 <= $4 AND status != 'deleted' LIMIT 1"
    )
    .bind(track)
    .bind(car)
    .bind(started_at)
    .bind(tolerance_secs)
    .fetch_optional(pool)
    .await?;

    Ok(session_id)
}

/// Insert a session with full import fields (import_source, import_format, ended_at, lap_count, best_lap_time_ms).
#[allow(clippy::too_many_arguments)]
pub async fn insert_imported_session(
    pool: &SqlitePool,
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
) -> Result<Session, StorageError> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    sqlx::query(
        "INSERT INTO sessions (id, track_name, car_name, session_type, started_at, ended_at, lap_count, best_lap_time_ms, status, import_source, import_format, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'completed', $9, $10, $11, $11)"
    )
    .bind(id)
    .bind(track_name)
    .bind(car_name)
    .bind(session_type)
    .bind(started_at)
    .bind(ended_at)
    .bind(lap_count)
    .bind(best_lap_time_ms)
    .bind(import_source)
    .bind(import_format)
    .bind(&now)
    .execute(pool)
    .await?;

    get_session(pool, id)
        .await?
        .ok_or_else(|| StorageError::NotFound(id.to_string()))
}

/// Return aggregate stats for sessions matching the given filters.
pub async fn get_session_stats(
    pool: &SqlitePool,
    filters: &FilterOptions,
) -> Result<SessionStats, StorageError> {
    let mut sql = String::from(
        "SELECT COUNT(*) as total, MIN(best_lap_time_ms) as best_lap, MAX(started_at) as latest
         FROM sessions
         WHERE status != 'deleted'",
    );

    let mut bind_idx = 0u32;
    let mut binds: Vec<String> = Vec::new();

    if let Some(track) = &filters.track {
        bind_idx += 1;
        sql.push_str(&format!(" AND track_name = ${}", bind_idx));
        binds.push(track.clone());
    }
    if let Some(car) = &filters.car {
        bind_idx += 1;
        sql.push_str(&format!(" AND car_name = ${}", bind_idx));
        binds.push(car.clone());
    }
    if let Some(date_start) = &filters.date_start {
        bind_idx += 1;
        sql.push_str(&format!(" AND started_at >= ${}", bind_idx));
        binds.push(date_start.clone());
    }
    if let Some(date_end) = &filters.date_end {
        bind_idx += 1;
        sql.push_str(&format!(" AND started_at <= ${}", bind_idx));
        binds.push(date_end.clone());
    }

    let mut query = sqlx::query_as::<_, (i64, Option<i64>, Option<String>)>(&sql);
    for b in &binds {
        query = query.bind(b);
    }

    let (total_sessions, best_lap_time_ms, latest_session_date) = query.fetch_one(pool).await?;

    Ok(SessionStats {
        total_sessions,
        best_lap_time_ms,
        latest_session_date,
    })
}
