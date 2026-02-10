//! Query functions for derived metrics storage.
//!
//! Stores and retrieves computed derived metrics results in JSON format.

use crate::error::StorageError;
use serde_json::Value as JsonValue;
use sqlx::SqlitePool;

/// Insert or update derived metrics for a session.
///
/// Uses UPSERT (INSERT OR REPLACE) to ensure idempotency.
/// Re-computing metrics for the same session will overwrite previous results.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `session_id` - Session identifier
/// * `metric_type` - Type of metric ('brake_count', 'trail_braking', 'tire_degradation', 'corner_segmentation')
/// * `data` - JSON-serialized metric data
///
/// # Returns
/// The ID of the inserted/updated record
pub async fn insert_derived_metrics(
    pool: &SqlitePool,
    session_id: &str,
    metric_type: &str,
    data: &JsonValue,
) -> Result<i64, StorageError> {
    let data_json = serde_json::to_string(data)?;

    let result = sqlx::query(
        r#"
        INSERT INTO derived_metrics (session_id, metric_type, data)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(session_id, metric_type)
        DO UPDATE SET data = ?3, computed_at = datetime('now')
        "#,
    )
    .bind(session_id)
    .bind(metric_type)
    .bind(&data_json)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Get derived metrics for a session by metric type.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `session_id` - Session identifier
/// * `metric_type` - Type of metric to retrieve
///
/// # Returns
/// JSON data if found, None if not found
pub async fn get_derived_metrics(
    pool: &SqlitePool,
    session_id: &str,
    metric_type: &str,
) -> Result<Option<JsonValue>, StorageError> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT data FROM derived_metrics
        WHERE session_id = ?1 AND metric_type = ?2
        "#,
    )
    .bind(session_id)
    .bind(metric_type)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((data,)) => {
            let json: JsonValue = serde_json::from_str(&data)?;
            Ok(Some(json))
        }
        None => Ok(None),
    }
}

/// Get all derived metrics for a session.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `session_id` - Session identifier
///
/// # Returns
/// Vector of (metric_type, data) tuples
pub async fn get_all_derived_metrics(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<(String, JsonValue)>, StorageError> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT metric_type, data FROM derived_metrics
        WHERE session_id = ?1
        ORDER BY metric_type
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for (metric_type, data) in rows {
        let json: JsonValue = serde_json::from_str(&data)?;
        results.push((metric_type, json));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use serde_json::json;
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
    async fn test_insert_and_get_derived_metrics() {
        let (db, _temp, session_id) = setup_test_db().await;

        let test_data = json!({
            "brake_count": 12,
            "avg_pressure": 0.75
        });

        // Insert metrics
        let id = insert_derived_metrics(
            db.pool(),
            &session_id,
            "brake_count",
            &test_data,
        )
        .await
        .unwrap();

        assert!(id > 0, "Should return valid ID");

        // Retrieve metrics
        let retrieved = get_derived_metrics(
            db.pool(),
            &session_id,
            "brake_count",
        )
        .await
        .unwrap();

        assert!(retrieved.is_some(), "Should find inserted metrics");
        assert_eq!(retrieved.unwrap(), test_data, "Retrieved data should match");
    }

    #[tokio::test]
    async fn test_upsert_idempotency() {
        let (db, _temp, session_id) = setup_test_db().await;

        let data_v1 = json!({"count": 10});
        let data_v2 = json!({"count": 15});

        // Insert first version
        insert_derived_metrics(
            db.pool(),
            &session_id,
            "brake_count",
            &data_v1,
        )
        .await
        .unwrap();

        // Insert second version (should replace)
        insert_derived_metrics(
            db.pool(),
            &session_id,
            "brake_count",
            &data_v2,
        )
        .await
        .unwrap();

        // Verify latest version is stored
        let retrieved = get_derived_metrics(
            db.pool(),
            &session_id,
            "brake_count",
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(retrieved, data_v2, "Should have latest version");
    }

    #[tokio::test]
    async fn test_get_all_derived_metrics() {
        let (db, _temp, session_id) = setup_test_db().await;

        // Insert multiple metric types
        insert_derived_metrics(
            db.pool(),
            &session_id,
            "brake_count",
            &json!({"count": 12}),
        )
        .await
        .unwrap();

        insert_derived_metrics(
            db.pool(),
            &session_id,
            "tire_degradation",
            &json!({"severity": "mild"}),
        )
        .await
        .unwrap();

        // Retrieve all
        let all_metrics = get_all_derived_metrics(
            db.pool(),
            &session_id,
        )
        .await
        .unwrap();

        assert_eq!(all_metrics.len(), 2, "Should retrieve both metrics");
        assert_eq!(all_metrics[0].0, "brake_count");
        assert_eq!(all_metrics[1].0, "tire_degradation");
    }
}
