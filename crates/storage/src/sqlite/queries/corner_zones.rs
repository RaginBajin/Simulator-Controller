//! Query functions for corner zones storage.
//!
//! Stores corner boundaries identified from best lap telemetry.

use crate::error::StorageError;
use sqlx::SqlitePool;

/// Type alias for corner zone query results to reduce complexity
type CornerZoneQueryRow = (i64, String, i32, String, f64, f64, Option<f64>, Option<f64>);

/// Corner zone record from database.
#[derive(Debug, Clone)]
pub struct CornerZone {
    pub id: i64,
    pub session_id: String,
    pub corner_id: i32,
    pub name: String,
    pub start_distance: f64,
    pub end_distance: f64,
    pub brake_onset_distance: Option<f64>,
    pub apex_distance: Option<f64>,
}

/// Insert a corner zone for a session.
///
/// Uses UPSERT to ensure idempotency (one entry per corner per session).
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `session_id` - Session identifier
/// * `corner_id` - Sequential corner ID (1, 2, 3...)
/// * `name` - Corner name (T1, T2, T3...)
/// * `start_distance` - Lap distance where corner starts (meters)
/// * `end_distance` - Lap distance where corner ends (meters)
/// * `brake_onset_distance` - Optional lap distance where braking begins (meters)
/// * `apex_distance` - Optional lap distance at apex (min speed point, meters)
///
/// # Returns
/// The ID of the inserted/updated record
#[allow(clippy::too_many_arguments)]
pub async fn insert_corner_zone(
    pool: &SqlitePool,
    session_id: &str,
    corner_id: i32,
    name: &str,
    start_distance: f64,
    end_distance: f64,
    brake_onset_distance: Option<f64>,
    apex_distance: Option<f64>,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        r#"
        INSERT INTO corner_zones (session_id, corner_id, name, start_distance, end_distance, brake_onset_distance, apex_distance)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(session_id, corner_id)
        DO UPDATE SET
            name = ?3,
            start_distance = ?4,
            end_distance = ?5,
            brake_onset_distance = ?6,
            apex_distance = ?7
        "#,
    )
    .bind(session_id)
    .bind(corner_id)
    .bind(name)
    .bind(start_distance)
    .bind(end_distance)
    .bind(brake_onset_distance)
    .bind(apex_distance)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Get all corner zones for a session, ordered by corner ID.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `session_id` - Session identifier
///
/// # Returns
/// Vector of corner zones in corner ID order
pub async fn get_corner_zones(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<CornerZone>, StorageError> {
    let rows: Vec<CornerZoneQueryRow> = sqlx::query_as(
        r#"
        SELECT id, session_id, corner_id, name, start_distance, end_distance, brake_onset_distance, apex_distance
        FROM corner_zones
        WHERE session_id = ?1
        ORDER BY corner_id
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                session_id,
                corner_id,
                name,
                start_distance,
                end_distance,
                brake_onset_distance,
                apex_distance,
            )| CornerZone {
                id,
                session_id,
                corner_id,
                name,
                start_distance,
                end_distance,
                brake_onset_distance,
                apex_distance,
            },
        )
        .collect())
}

/// Get a specific corner zone by corner ID.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `session_id` - Session identifier
/// * `corner_id` - Corner ID to retrieve
///
/// # Returns
/// Corner zone if found, None otherwise
pub async fn get_corner_zone(
    pool: &SqlitePool,
    session_id: &str,
    corner_id: i32,
) -> Result<Option<CornerZone>, StorageError> {
    let row: Option<CornerZoneQueryRow> = sqlx::query_as(
        r#"
        SELECT id, session_id, corner_id, name, start_distance, end_distance, brake_onset_distance, apex_distance
        FROM corner_zones
        WHERE session_id = ?1 AND corner_id = ?2
        "#,
    )
    .bind(session_id)
    .bind(corner_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(
        |(
            id,
            session_id,
            corner_id,
            name,
            start_distance,
            end_distance,
            brake_onset_distance,
            apex_distance,
        )| CornerZone {
            id,
            session_id,
            corner_id,
            name,
            start_distance,
            end_distance,
            brake_onset_distance,
            apex_distance,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use tempfile::TempDir;

    async fn setup_test_db() -> (Database, TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::init(&db_path).await.unwrap();

        // Insert a test session
        let new_session = crate::types::NewSession {
            track_name: "Test Track".to_string(),
            car_name: "Test Car".to_string(),
            session_type: "practice".to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
        };
        let session = db.insert_session(&new_session).await.unwrap();

        (db, temp_dir, session.id)
    }

    #[tokio::test]
    async fn test_insert_and_get_corner_zone() {
        let (db, _temp, session_id) = setup_test_db().await;

        // Insert corner zone
        let id = insert_corner_zone(
            db.pool(),
            &session_id,
            1,
            "T1",
            100.0,
            250.0,
            Some(100.0),
            Some(175.0),
        )
        .await
        .unwrap();

        assert!(id > 0, "Should return valid ID");

        // Retrieve corner zone
        let corner = get_corner_zone(db.pool(), &session_id, 1)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(corner.corner_id, 1);
        assert_eq!(corner.name, "T1");
        assert_eq!(corner.start_distance, 100.0);
        assert_eq!(corner.end_distance, 250.0);
        assert_eq!(corner.brake_onset_distance, Some(100.0));
        assert_eq!(corner.apex_distance, Some(175.0));
    }

    #[tokio::test]
    async fn test_get_all_corner_zones_ordered() {
        let (db, _temp, session_id) = setup_test_db().await;

        // Insert multiple corners out of order
        insert_corner_zone(db.pool(), &session_id, 3, "T3", 500.0, 650.0, None, None)
            .await
            .unwrap();
        insert_corner_zone(db.pool(), &session_id, 1, "T1", 100.0, 250.0, None, None)
            .await
            .unwrap();
        insert_corner_zone(db.pool(), &session_id, 2, "T2", 300.0, 450.0, None, None)
            .await
            .unwrap();

        // Retrieve all - should be in corner ID order
        let corners = get_corner_zones(db.pool(), &session_id).await.unwrap();

        assert_eq!(corners.len(), 3);
        assert_eq!(corners[0].corner_id, 1);
        assert_eq!(corners[1].corner_id, 2);
        assert_eq!(corners[2].corner_id, 3);
    }

    #[tokio::test]
    async fn test_upsert_idempotency() {
        let (db, _temp, session_id) = setup_test_db().await;

        // Insert first version
        insert_corner_zone(db.pool(), &session_id, 1, "T1", 100.0, 250.0, None, None)
            .await
            .unwrap();

        // Insert updated version (should replace)
        insert_corner_zone(
            db.pool(),
            &session_id,
            1,
            "Turn 1",
            105.0,
            255.0,
            Some(105.0),
            Some(180.0),
        )
        .await
        .unwrap();

        // Verify latest version is stored
        let corner = get_corner_zone(db.pool(), &session_id, 1)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(corner.name, "Turn 1", "Should have updated name");
        assert_eq!(corner.start_distance, 105.0, "Should have updated start");
        assert_eq!(
            corner.apex_distance,
            Some(180.0),
            "Should have updated apex"
        );
    }
}
