use std::time::Instant;
use storage::{
    Database, DebriefUpdate, FilterOptions, ListOptions, NewAiDebrief, NewLap, NewSession,
    SessionUpdate,
};

fn new_session(track: &str) -> NewSession {
    NewSession {
        track_name: track.to_string(),
        car_name: "Porsche 911 GT3 R".to_string(),
        session_type: "practice".to_string(),
        started_at: chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
    }
}

fn new_session_with_time(track: &str, offset_hours: i64) -> NewSession {
    let time = chrono::Utc::now() - chrono::Duration::hours(offset_hours);
    NewSession {
        track_name: track.to_string(),
        car_name: "Porsche 911 GT3 R".to_string(),
        session_type: "practice".to_string(),
        started_at: time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
    }
}

async fn setup_db() -> (Database, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();
    (db, dir)
}

// -- AC #1: Session List Query Returns Sorted Results --

#[tokio::test]
async fn test_list_sessions_sorted_newest_first() {
    let (db, _dir) = setup_db().await;

    // Insert sessions with different times (older first)
    let _s1 = db
        .insert_session(&new_session_with_time("Spa", 3))
        .await
        .unwrap();
    let _s2 = db
        .insert_session(&new_session_with_time("Monza", 2))
        .await
        .unwrap();
    let _s3 = db
        .insert_session(&new_session_with_time("Silverstone", 1))
        .await
        .unwrap();

    let sessions = db
        .list_sessions(ListOptions::default(), &FilterOptions::default())
        .await
        .unwrap();
    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0].track_name, "Silverstone"); // newest
    assert_eq!(sessions[1].track_name, "Monza");
    assert_eq!(sessions[2].track_name, "Spa"); // oldest
}

#[tokio::test]
async fn test_list_sessions_excludes_deleted() {
    let (db, _dir) = setup_db().await;

    let s1 = db.insert_session(&new_session("Spa")).await.unwrap();
    let _s2 = db.insert_session(&new_session("Monza")).await.unwrap();

    db.soft_delete_session(&s1.id).await.unwrap();

    let sessions = db
        .list_sessions(ListOptions::default(), &FilterOptions::default())
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].track_name, "Monza");
}

#[tokio::test]
async fn test_list_1000_sessions_performance() {
    let (db, _dir) = setup_db().await;

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

// -- AC #2: Full Session Data Retrieval --

#[tokio::test]
async fn test_get_session_detail_includes_laps_and_debrief() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();

    // Add laps
    for i in 1..=3 {
        db.insert_lap(&NewLap {
            session_id: session.id.clone(),
            lap_number: i,
            lap_time_ms: 120000 + (i as i64 * 100),
            is_valid: true,
            completion_status: "complete".to_string(),
        })
        .await
        .unwrap();
    }

    // Add debrief
    db.insert_debrief(&NewAiDebrief {
        session_id: session.id.clone(),
        coaching_text: Some("Great session!".to_string()),
        insights_json: Some(r#"{"key": "value"}"#.to_string()),
        recommendations_json: Some(r#"["brake later"]"#.to_string()),
        provider_name: Some("claude".to_string()),
        model_name: Some("claude-3-5-sonnet".to_string()),
        trigger_type: Some("automatic".to_string()),
        data_range_from_ms: None,
        data_range_to_ms: None,
    })
    .await
    .unwrap();

    let detail = db.get_session_detail(&session.id).await.unwrap();
    assert_eq!(detail.session.id, session.id);
    assert_eq!(detail.laps.len(), 3);
    assert!(detail.debrief.is_some());
    assert_eq!(
        detail.debrief.unwrap().coaching_text.unwrap(),
        "Great session!"
    );
    assert!(!detail.has_telemetry); // No telemetry path set
}

#[tokio::test]
async fn test_full_session_detail_loads_under_200ms() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();

    // Add 50 laps
    for i in 1..=50 {
        db.insert_lap(&NewLap {
            session_id: session.id.clone(),
            lap_number: i,
            lap_time_ms: 120000 + (i as i64 * 100),
            is_valid: true,
            completion_status: "complete".to_string(),
        })
        .await
        .unwrap();
    }

    db.insert_debrief(&NewAiDebrief {
        session_id: session.id.clone(),
        coaching_text: Some("Session analysis".to_string()),
        insights_json: None,
        recommendations_json: None,
        provider_name: Some("claude".to_string()),
        model_name: Some("claude-3-5-sonnet".to_string()),
        trigger_type: Some("automatic".to_string()),
        data_range_from_ms: None,
        data_range_to_ms: None,
    })
    .await
    .unwrap();

    let start = Instant::now();
    let _detail = db.get_session_detail(&session.id).await.unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 200,
        "Full session detail load took {}ms, expected <200ms",
        elapsed.as_millis()
    );
}

// -- AC #3: Session Soft Delete --

#[tokio::test]
async fn test_soft_delete_sets_status_and_deleted_at() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();
    assert_eq!(session.status, "active");

    let result = db.soft_delete_session(&session.id).await.unwrap();
    assert_eq!(result.session_id, session.id);
    assert!(!result.deleted_at.is_empty());

    let deleted = db.get_session(&session.id).await.unwrap().unwrap();
    assert_eq!(deleted.status, "deleted");
    assert_eq!(deleted.previous_status.as_deref(), Some("active"));
    assert!(deleted.deleted_at.is_some());
}

#[tokio::test]
async fn test_soft_delete_already_deleted_fails() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();
    db.soft_delete_session(&session.id).await.unwrap();

    let result = db.soft_delete_session(&session.id).await;
    assert!(result.is_err());
}

// -- AC #4: Session Restore (Undo Delete) --

#[tokio::test]
async fn test_restore_session_restores_previous_status() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();
    db.update_session(
        &session.id,
        SessionUpdate {
            status: Some("completed".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    db.soft_delete_session(&session.id).await.unwrap();

    let result = db.restore_session(&session.id).await.unwrap();
    assert_eq!(result.session_id, session.id);
    assert_eq!(result.restored_status, "completed");

    let restored = db.get_session(&session.id).await.unwrap().unwrap();
    assert_eq!(restored.status, "completed");
    assert!(restored.deleted_at.is_none());
    assert!(restored.previous_status.is_none());
}

#[tokio::test]
async fn test_restore_non_deleted_session_fails() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();

    let result = db.restore_session(&session.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_restore_expired_session_fails() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();

    // Manually set deleted_at to 31 days ago
    let old_date = (chrono::Utc::now() - chrono::Duration::days(31))
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    sqlx::query(
        "UPDATE sessions SET status = 'deleted', previous_status = 'active', deleted_at = $1 WHERE id = $2",
    )
    .bind(&old_date)
    .bind(&session.id)
    .execute(db.pool_for_testing())
    .await
    .unwrap();

    let result = db.restore_session(&session.id).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, storage::StorageError::RestoreExpired(_)),
        "Expected RestoreExpired, got: {:?}",
        err
    );
}

// -- AC #5: Debrief Data Storage --

#[tokio::test]
async fn test_insert_and_retrieve_debrief() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();

    let debrief = db
        .insert_debrief(&NewAiDebrief {
            session_id: session.id.clone(),
            coaching_text: Some("You should brake later at turn 1.".to_string()),
            insights_json: Some(r#"{"braking_points": "late"}"#.to_string()),
            recommendations_json: Some(r#"["focus on T1 entry"]"#.to_string()),
            provider_name: Some("claude".to_string()),
            model_name: Some("claude-3-5-sonnet".to_string()),
            trigger_type: Some("automatic".to_string()),
            data_range_from_ms: None,
            data_range_to_ms: None,
        })
        .await
        .unwrap();

    assert_eq!(debrief.session_id, session.id);
    assert_eq!(
        debrief.coaching_text.as_deref(),
        Some("You should brake later at turn 1.")
    );

    let fetched = db.get_debrief_for_session(&session.id).await.unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().id, debrief.id);
}

#[tokio::test]
async fn test_update_debrief() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();

    let debrief = db
        .insert_debrief(&NewAiDebrief {
            session_id: session.id.clone(),
            coaching_text: Some("Initial analysis".to_string()),
            insights_json: None,
            recommendations_json: None,
            provider_name: Some("claude".to_string()),
            model_name: None,
            trigger_type: Some("automatic".to_string()),
            data_range_from_ms: None,
            data_range_to_ms: None,
        })
        .await
        .unwrap();

    let updated = db
        .update_debrief(
            &debrief.id,
            DebriefUpdate {
                coaching_text: Some("Updated analysis with more detail".to_string()),
                insights_json: Some(r#"{"new": "insight"}"#.to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(
        updated.coaching_text.as_deref(),
        Some("Updated analysis with more detail")
    );
    assert_eq!(
        updated.insights_json.as_deref(),
        Some(r#"{"new": "insight"}"#)
    );
}

// -- AC #7: Trash Cleanup Process --

#[tokio::test]
async fn test_cleanup_removes_expired_sessions() {
    let (db, _dir) = setup_db().await;

    let session = db.insert_session(&new_session("Spa")).await.unwrap();

    // Add laps and debrief
    db.insert_lap(&NewLap {
        session_id: session.id.clone(),
        lap_number: 1,
        lap_time_ms: 120000,
        is_valid: true,
        completion_status: "complete".to_string(),
    })
    .await
    .unwrap();

    db.insert_debrief(&NewAiDebrief {
        session_id: session.id.clone(),
        coaching_text: Some("test".to_string()),
        insights_json: None,
        recommendations_json: None,
        provider_name: None,
        model_name: None,
        trigger_type: Some("automatic".to_string()),
        data_range_from_ms: None,
        data_range_to_ms: None,
    })
    .await
    .unwrap();

    // Manually set deleted_at to 31 days ago
    let old_date = (chrono::Utc::now() - chrono::Duration::days(31))
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    sqlx::query(
        "UPDATE sessions SET status = 'deleted', previous_status = 'active', deleted_at = $1 WHERE id = $2",
    )
    .bind(&old_date)
    .bind(&session.id)
    .execute(db.pool_for_testing())
    .await
    .unwrap();

    // Find expired
    let expired = db.find_expired_deleted_sessions(30).await.unwrap();
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].id, session.id);

    // Permanently delete
    db.permanently_delete_session(&session.id).await.unwrap();

    // Verify cascade: session, laps, debriefs all gone
    let session = db.get_session(&session.id).await.unwrap();
    assert!(session.is_none());

    let laps = db.get_laps_for_session(&expired[0].id).await.unwrap();
    assert!(laps.is_empty());

    let debrief = db.get_debrief_for_session(&expired[0].id).await.unwrap();
    assert!(debrief.is_none());
}

#[tokio::test]
async fn test_list_deleted_sessions() {
    let (db, _dir) = setup_db().await;

    let s1 = db.insert_session(&new_session("Spa")).await.unwrap();
    let _s2 = db.insert_session(&new_session("Monza")).await.unwrap();

    db.soft_delete_session(&s1.id).await.unwrap();

    let deleted = db.list_deleted_sessions().await.unwrap();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].track_name, "Spa");
}
