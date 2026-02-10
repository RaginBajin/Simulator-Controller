//! Gap marker queries for telemetry data gaps

use crate::error::StorageError;
use crate::types::{GapMarker, NewGapMarker};
use sqlx::SqlitePool;

/// Inserts a new gap marker into the database.
pub async fn insert_gap_marker(
    pool: &SqlitePool,
    gap: &NewGapMarker,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO gap_markers (session_id, start_time, end_time, duration_ms, reason, lap_position)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&gap.session_id)
    .bind(gap.start_time)
    .bind(gap.end_time)
    .bind(gap.duration_ms)
    .bind(&gap.reason)
    .bind(gap.lap_position)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Retrieves all gap markers for a specific session.
pub async fn get_gap_markers_for_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<GapMarker>, StorageError> {
    let gaps = sqlx::query_as::<_, GapMarker>(
        "SELECT id, session_id, start_time, end_time, duration_ms, reason, lap_position, created_at
         FROM gap_markers
         WHERE session_id = ?
         ORDER BY start_time ASC"
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(gaps)
}

/// Updates session gap summary statistics (gap_count and total_gap_duration_ms).
pub async fn update_session_gap_summary(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE sessions
         SET gap_count = (SELECT COUNT(*) FROM gap_markers WHERE session_id = ?),
             total_gap_duration_ms = (SELECT COALESCE(SUM(duration_ms), 0) FROM gap_markers WHERE session_id = ?),
             updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(session_id)
    .bind(session_id)
    .bind(session_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Marks a session as partial (disconnected) with a disconnected_at timestamp.
pub async fn mark_session_partial(
    pool: &SqlitePool,
    session_id: &str,
    disconnected_at: &str,
) -> Result<(), StorageError> {
    let result = sqlx::query(
        "UPDATE sessions
         SET status = 'partial',
             disconnected_at = ?,
             updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(disconnected_at)
    .bind(session_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(StorageError::NotFound(session_id.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    async fn setup_test_db() -> SqlitePool {
        let connect_options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connect_options)
            .await
            .unwrap();

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        pool
    }

    async fn create_test_session(pool: &SqlitePool, session_id: &str) {
        sqlx::query(
            "INSERT INTO sessions (id, track_name, car_name, session_type, started_at)
             VALUES (?, 'Spa', 'GT3', 'practice', datetime('now'))"
        )
        .bind(session_id)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_insert_gap_marker() {
        let pool = setup_test_db().await;
        create_test_session(&pool, "session-1").await;

        let gap = NewGapMarker {
            session_id: "session-1".to_string(),
            start_time: 1000,
            end_time: 1500,
            duration_ms: 500,
            reason: "stall".to_string(),
            lap_position: Some(1234.5),
        };

        let id = insert_gap_marker(&pool, &gap).await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_get_gap_markers_for_session() {
        let pool = setup_test_db().await;
        create_test_session(&pool, "session-1").await;

        let gap1 = NewGapMarker {
            session_id: "session-1".to_string(),
            start_time: 1000,
            end_time: 1300,
            duration_ms: 300,
            reason: "stall".to_string(),
            lap_position: Some(500.0),
        };

        let gap2 = NewGapMarker {
            session_id: "session-1".to_string(),
            start_time: 2000,
            end_time: 2500,
            duration_ms: 500,
            reason: "disconnect".to_string(),
            lap_position: None,
        };

        insert_gap_marker(&pool, &gap1).await.unwrap();
        insert_gap_marker(&pool, &gap2).await.unwrap();

        let gaps = get_gap_markers_for_session(&pool, "session-1")
            .await
            .unwrap();

        assert_eq!(gaps.len(), 2);
        assert_eq!(gaps[0].duration_ms, 300);
        assert_eq!(gaps[0].reason, "stall");
        assert_eq!(gaps[1].duration_ms, 500);
        assert_eq!(gaps[1].reason, "disconnect");
    }

    #[tokio::test]
    async fn test_get_gap_markers_empty() {
        let pool = setup_test_db().await;
        create_test_session(&pool, "session-1").await;

        let gaps = get_gap_markers_for_session(&pool, "session-1")
            .await
            .unwrap();

        assert_eq!(gaps.len(), 0);
    }

    #[tokio::test]
    async fn test_update_session_gap_summary() {
        let pool = setup_test_db().await;
        create_test_session(&pool, "session-1").await;

        let gap1 = NewGapMarker {
            session_id: "session-1".to_string(),
            start_time: 1000,
            end_time: 1300,
            duration_ms: 300,
            reason: "stall".to_string(),
            lap_position: None,
        };

        let gap2 = NewGapMarker {
            session_id: "session-1".to_string(),
            start_time: 2000,
            end_time: 2700,
            duration_ms: 700,
            reason: "disconnect".to_string(),
            lap_position: None,
        };

        insert_gap_marker(&pool, &gap1).await.unwrap();
        insert_gap_marker(&pool, &gap2).await.unwrap();

        update_session_gap_summary(&pool, "session-1")
            .await
            .unwrap();

        let session: (i32, i64) = sqlx::query_as(
            "SELECT gap_count, total_gap_duration_ms
             FROM sessions
             WHERE id = ?"
        )
        .bind("session-1")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(session.0, 2);
        assert_eq!(session.1, 1000);
    }

    #[tokio::test]
    async fn test_mark_session_partial() {
        let pool = setup_test_db().await;
        create_test_session(&pool, "session-1").await;

        let disconnected_at = "2024-01-15T10:30:00Z";
        mark_session_partial(&pool, "session-1", disconnected_at)
            .await
            .unwrap();

        let session: (String, Option<String>) = sqlx::query_as(
            "SELECT status, disconnected_at
             FROM sessions
             WHERE id = ?"
        )
        .bind("session-1")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(session.0, "partial");
        assert_eq!(session.1.unwrap(), disconnected_at);
    }

    #[tokio::test]
    async fn test_mark_session_partial_not_found() {
        let pool = setup_test_db().await;

        let result = mark_session_partial(&pool, "nonexistent", "2024-01-15T10:30:00Z").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));
    }
}
