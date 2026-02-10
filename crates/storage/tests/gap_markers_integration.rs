//! Integration tests for gap marker operations

use storage::sqlite::Database;
use storage::types::{NewGapMarker, NewSession};
use tempfile::TempDir;

async fn setup_test_db() -> (Database, TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();
    (db, temp_dir)
}

async fn create_test_session(db: &Database, session_id: &str) {
    let new_session = NewSession {
        track_name: "Spa-Francorchamps".to_string(),
        car_name: "Porsche 911 GT3 R".to_string(),
        session_type: "practice".to_string(),
        started_at: "2024-01-15T10:00:00Z".to_string(),
    };

    // Manually insert with specific ID for testing
    let pool = db.pool_for_testing();
    sqlx::query(
        "INSERT INTO sessions (id, track_name, car_name, session_type, started_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(session_id)
    .bind(&new_session.track_name)
    .bind(&new_session.car_name)
    .bind(&new_session.session_type)
    .bind(&new_session.started_at)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn test_insert_and_retrieve_gap_markers() {
    let (db, _temp) = setup_test_db().await;
    create_test_session(&db, "session-1").await;

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
        end_time: 3000,
        duration_ms: 1000,
        reason: "disconnect".to_string(),
        lap_position: None,
    };

    db.insert_gap_marker(&gap1).await.unwrap();
    db.insert_gap_marker(&gap2).await.unwrap();

    let gaps = db.get_gap_markers_for_session("session-1").await.unwrap();

    assert_eq!(gaps.len(), 2);
    assert_eq!(gaps[0].duration_ms, 300);
    assert_eq!(gaps[0].reason, "stall");
    assert_eq!(gaps[1].duration_ms, 1000);
    assert_eq!(gaps[1].reason, "disconnect");
}

#[tokio::test]
async fn test_update_session_gap_summary() {
    let (db, _temp) = setup_test_db().await;
    create_test_session(&db, "session-1").await;

    let gap1 = NewGapMarker {
        session_id: "session-1".to_string(),
        start_time: 1000,
        end_time: 1400,
        duration_ms: 400,
        reason: "stall".to_string(),
        lap_position: None,
    };

    let gap2 = NewGapMarker {
        session_id: "session-1".to_string(),
        start_time: 2000,
        end_time: 2600,
        duration_ms: 600,
        reason: "disconnect".to_string(),
        lap_position: None,
    };

    db.insert_gap_marker(&gap1).await.unwrap();
    db.insert_gap_marker(&gap2).await.unwrap();

    db.update_session_gap_summary("session-1").await.unwrap();

    let session = db.get_session("session-1").await.unwrap().unwrap();
    assert_eq!(session.gap_count, 2);
    assert_eq!(session.total_gap_duration_ms, 1000);
}

#[tokio::test]
async fn test_mark_session_partial() {
    let (db, _temp) = setup_test_db().await;
    create_test_session(&db, "session-1").await;

    let disconnected_at = "2024-01-15T10:30:00Z";
    db.mark_session_partial("session-1", disconnected_at)
        .await
        .unwrap();

    let session = db.get_session("session-1").await.unwrap().unwrap();
    assert_eq!(session.status, "partial");
    assert_eq!(session.disconnected_at.unwrap(), disconnected_at);
}

#[tokio::test]
async fn test_gap_markers_with_session_detail() {
    let (db, _temp) = setup_test_db().await;
    create_test_session(&db, "session-1").await;

    let gap = NewGapMarker {
        session_id: "session-1".to_string(),
        start_time: 1000,
        end_time: 1700,
        duration_ms: 700,
        reason: "disconnect".to_string(),
        lap_position: Some(1234.5),
    };

    db.insert_gap_marker(&gap).await.unwrap();
    db.update_session_gap_summary("session-1").await.unwrap();

    let session_detail = db.get_session_detail("session-1").await.unwrap();
    assert_eq!(session_detail.session.gap_count, 1);
    assert_eq!(session_detail.session.total_gap_duration_ms, 700);

    let gaps = db.get_gap_markers_for_session("session-1").await.unwrap();
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].lap_position, Some(1234.5));
}

#[tokio::test]
async fn test_partial_session_appears_in_session_list() {
    let (db, _temp) = setup_test_db().await;
    create_test_session(&db, "session-1").await;
    create_test_session(&db, "session-2").await;

    db.mark_session_partial("session-1", "2024-01-15T10:30:00Z")
        .await
        .unwrap();

    let sessions = db
        .list_sessions(
            storage::types::ListOptions {
                limit: 10,
                offset: 0,
            },
            &storage::types::FilterOptions::default(),
        )
        .await
        .unwrap();

    // Both sessions should be in the list
    assert_eq!(sessions.len(), 2);

    // Find the partial session
    let partial_session = sessions.iter().find(|s| s.id == "session-1").unwrap();
    assert_eq!(partial_session.status, "partial");
}
