use std::time::Instant;
use storage::{Database, FilterOptions, ListOptions, NewLap, NewSession, SessionUpdate};

fn new_session() -> NewSession {
    NewSession {
        track_name: "Spa-Francorchamps".to_string(),
        car_name: "Porsche 911 GT3 R".to_string(),
        session_type: "practice".to_string(),
        started_at: chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
    }
}

#[tokio::test]
async fn test_insert_and_get_session() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Database::init(&db_path).await.unwrap();
    let session = db.insert_session(&new_session()).await.unwrap();

    assert_eq!(session.track_name, "Spa-Francorchamps");
    assert_eq!(session.car_name, "Porsche 911 GT3 R");
    assert_eq!(session.session_type, "practice");
    assert_eq!(session.status, "active");
    assert_eq!(session.lap_count, 0);
    assert!(session.best_lap_time_ms.is_none());
    assert!(session.ended_at.is_none());

    let fetched = db.get_session(&session.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, session.id);
    assert_eq!(fetched.track_name, "Spa-Francorchamps");
}

#[tokio::test]
async fn test_session_persists_across_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Open, insert, close (drop)
    let session_id;
    {
        let db = Database::init(&db_path).await.unwrap();
        let session = db.insert_session(&new_session()).await.unwrap();
        session_id = session.id;
    }

    // Reopen, verify data persists
    let db = Database::init(&db_path).await.unwrap();
    let sessions = db
        .list_sessions(ListOptions::default(), &FilterOptions::default())
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, session_id);
    assert_eq!(sessions[0].track_name, "Spa-Francorchamps");
}

#[tokio::test]
async fn test_list_sessions_performance_1000() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    // Insert 1000 sessions
    for i in 0..1000 {
        let session = NewSession {
            track_name: format!("Track {}", i),
            car_name: format!("Car {}", i),
            session_type: "practice".to_string(),
            started_at: chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        };
        db.insert_session(&session).await.unwrap();
    }

    // Query and time it
    let start = Instant::now();
    let sessions = db
        .list_sessions(
            ListOptions {
                limit: 1000,
                offset: 0,
            },
            &FilterOptions::default(),
        )
        .await
        .unwrap();
    let elapsed = start.elapsed();

    assert_eq!(sessions.len(), 1000);
    assert!(
        elapsed.as_millis() < 100,
        "Query took {}ms, expected <100ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_insert_session_with_laps() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    let session = db.insert_session(&new_session()).await.unwrap();

    // Insert laps
    for i in 1..=5 {
        let lap = NewLap {
            session_id: session.id.clone(),
            lap_number: i,
            lap_time_ms: 120000 + (i as i64 * 100),
            is_valid: true,
            completion_status: "complete".to_string(),
        };
        db.insert_lap(&lap).await.unwrap();
    }

    let laps = db.get_laps_for_session(&session.id).await.unwrap();
    assert_eq!(laps.len(), 5);
    assert_eq!(laps[0].lap_number, 1);
    assert_eq!(laps[4].lap_number, 5);
    assert_eq!(laps[0].session_id, session.id);
}

#[tokio::test]
async fn test_update_session_status() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    let session = db.insert_session(&new_session()).await.unwrap();
    assert_eq!(session.status, "active");

    let updated = db
        .update_session(
            &session.id,
            SessionUpdate {
                status: Some("completed".to_string()),
                ended_at: Some(
                    chrono::Utc::now()
                        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                        .to_string(),
                ),
                lap_count: Some(10),
                best_lap_time_ms: Some(119500),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.status, "completed");
    assert!(updated.ended_at.is_some());
    assert_eq!(updated.lap_count, 10);
    assert_eq!(updated.best_lap_time_ms, Some(119500));
    assert_ne!(updated.updated_at, session.updated_at);
}

#[tokio::test]
async fn test_migration_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Initialize twice - should not error
    let _db1 = Database::init(&db_path).await.unwrap();
    let db2 = Database::init(&db_path).await.unwrap();

    // Should still work
    let session = db2.insert_session(&new_session()).await.unwrap();
    assert_eq!(session.track_name, "Spa-Francorchamps");
}

#[tokio::test]
async fn test_foreign_key_enforcement() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    // Inserting a lap for a non-existent session should fail
    let lap = NewLap {
        session_id: "non-existent-id".to_string(),
        lap_number: 1,
        lap_time_ms: 120000,
        is_valid: true,
        completion_status: "complete".to_string(),
    };

    let result = db.insert_lap(&lap).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_sessions_excludes_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    let session = db.insert_session(&new_session()).await.unwrap();
    db.update_session(
        &session.id,
        SessionUpdate {
            status: Some("deleted".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let sessions = db
        .list_sessions(ListOptions::default(), &FilterOptions::default())
        .await
        .unwrap();
    assert_eq!(sessions.len(), 0);
}
