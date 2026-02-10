use storage::{Database, FilterOptions, ListOptions, NewSession, SessionUpdate};

fn new_session(track: &str, car: &str, started_at: &str) -> NewSession {
    NewSession {
        track_name: track.to_string(),
        car_name: car.to_string(),
        session_type: "practice".to_string(),
        started_at: started_at.to_string(),
        raw_session_type: None,
    }
}

async fn setup_db() -> (Database, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();
    (db, dir)
}

async fn seed_sessions(db: &Database) {
    // Insert diverse sessions for filter testing
    let sessions = vec![
        (
            "Spa-Francorchamps",
            "Porsche 911 GT3 R",
            "2025-01-15T10:00:00.000Z",
        ),
        (
            "Spa-Francorchamps",
            "BMW M4 GT3",
            "2025-01-20T14:00:00.000Z",
        ),
        ("Monza", "Porsche 911 GT3 R", "2025-02-01T09:00:00.000Z"),
        ("Monza", "Ferrari 296 GT3", "2025-02-10T16:00:00.000Z"),
        (
            "Silverstone",
            "Porsche 911 GT3 R",
            "2025-03-01T11:00:00.000Z",
        ),
    ];

    for (i, (track, car, started_at)) in sessions.iter().enumerate() {
        let session = db
            .insert_session(&new_session(track, car, started_at))
            .await
            .unwrap();
        // Give some sessions best lap times
        if i < 4 {
            db.update_session(
                &session.id,
                SessionUpdate {
                    status: Some("completed".to_string()),
                    best_lap_time_ms: Some(120000 - (i as i64 * 1000)),
                    lap_count: Some((i as i32) + 5),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }
    }
}

#[tokio::test]
async fn test_filter_by_track() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    let filters = FilterOptions {
        track: Some("Spa-Francorchamps".to_string()),
        ..Default::default()
    };

    let sessions = db
        .list_sessions(ListOptions::default(), &filters)
        .await
        .unwrap();

    assert_eq!(sessions.len(), 2);
    for s in &sessions {
        assert_eq!(s.track_name, "Spa-Francorchamps");
    }
}

#[tokio::test]
async fn test_filter_by_car() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    let filters = FilterOptions {
        car: Some("Porsche 911 GT3 R".to_string()),
        ..Default::default()
    };

    let sessions = db
        .list_sessions(ListOptions::default(), &filters)
        .await
        .unwrap();

    assert_eq!(sessions.len(), 3);
    for s in &sessions {
        assert_eq!(s.car_name, "Porsche 911 GT3 R");
    }
}

#[tokio::test]
async fn test_filter_by_date_range() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    let filters = FilterOptions {
        date_start: Some("2025-02-01T00:00:00.000Z".to_string()),
        date_end: Some("2025-02-28T23:59:59.000Z".to_string()),
        ..Default::default()
    };

    let sessions = db
        .list_sessions(ListOptions::default(), &filters)
        .await
        .unwrap();

    assert_eq!(sessions.len(), 2);
    for s in &sessions {
        assert_eq!(s.track_name, "Monza");
    }
}

#[tokio::test]
async fn test_combined_filters_use_and_logic() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    let filters = FilterOptions {
        track: Some("Spa-Francorchamps".to_string()),
        car: Some("Porsche 911 GT3 R".to_string()),
        ..Default::default()
    };

    let sessions = db
        .list_sessions(ListOptions::default(), &filters)
        .await
        .unwrap();

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].track_name, "Spa-Francorchamps");
    assert_eq!(sessions[0].car_name, "Porsche 911 GT3 R");
}

#[tokio::test]
async fn test_pagination_with_filters() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    // Get first page of 2
    let page1 = db
        .list_sessions(
            ListOptions {
                limit: 2,
                offset: 0,
            },
            &FilterOptions::default(),
        )
        .await
        .unwrap();
    assert_eq!(page1.len(), 2);

    // Get second page of 2
    let page2 = db
        .list_sessions(
            ListOptions {
                limit: 2,
                offset: 2,
            },
            &FilterOptions::default(),
        )
        .await
        .unwrap();
    assert_eq!(page2.len(), 2);

    // Get third page (only 1 left)
    let page3 = db
        .list_sessions(
            ListOptions {
                limit: 2,
                offset: 4,
            },
            &FilterOptions::default(),
        )
        .await
        .unwrap();
    assert_eq!(page3.len(), 1);

    // No duplicate IDs across pages
    let mut all_ids: Vec<String> = Vec::new();
    all_ids.extend(page1.iter().map(|s| s.id.clone()));
    all_ids.extend(page2.iter().map(|s| s.id.clone()));
    all_ids.extend(page3.iter().map(|s| s.id.clone()));
    let unique_count = {
        let mut seen = std::collections::HashSet::new();
        all_ids.iter().filter(|id| seen.insert(id.as_str())).count()
    };
    assert_eq!(unique_count, 5);
}

#[tokio::test]
async fn test_get_distinct_tracks() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    let tracks = db.get_distinct_tracks().await.unwrap();
    assert_eq!(tracks.len(), 3);
    assert!(tracks.contains(&"Monza".to_string()));
    assert!(tracks.contains(&"Silverstone".to_string()));
    assert!(tracks.contains(&"Spa-Francorchamps".to_string()));
}

#[tokio::test]
async fn test_get_distinct_cars() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    let cars = db.get_distinct_cars().await.unwrap();
    assert_eq!(cars.len(), 3);
    assert!(cars.contains(&"BMW M4 GT3".to_string()));
    assert!(cars.contains(&"Ferrari 296 GT3".to_string()));
    assert!(cars.contains(&"Porsche 911 GT3 R".to_string()));
}

#[tokio::test]
async fn test_session_stats_no_filters() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    let stats = db
        .get_session_stats(&FilterOptions::default())
        .await
        .unwrap();

    assert_eq!(stats.total_sessions, 5);
    assert!(stats.best_lap_time_ms.is_some());
    assert!(stats.latest_session_date.is_some());
}

#[tokio::test]
async fn test_session_stats_with_track_filter() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    let filters = FilterOptions {
        track: Some("Monza".to_string()),
        ..Default::default()
    };

    let stats = db.get_session_stats(&filters).await.unwrap();
    assert_eq!(stats.total_sessions, 2);
}

#[tokio::test]
async fn test_filter_returns_empty_for_no_match() {
    let (db, _dir) = setup_db().await;
    seed_sessions(&db).await;

    let filters = FilterOptions {
        track: Some("Nurburgring".to_string()),
        ..Default::default()
    };

    let sessions = db
        .list_sessions(ListOptions::default(), &filters)
        .await
        .unwrap();
    assert!(sessions.is_empty());

    let stats = db.get_session_stats(&filters).await.unwrap();
    assert_eq!(stats.total_sessions, 0);
    assert!(stats.best_lap_time_ms.is_none());
}

#[tokio::test]
async fn test_filter_by_session_type() {
    let (db, _dir) = setup_db().await;

    // Insert sessions with different types
    let mut race_session = new_session("Spa", "Porsche 911", "2025-01-01T10:00:00.000Z");
    race_session.session_type = "race".to_string();
    db.insert_session(&race_session).await.unwrap();

    let mut qualifying_session = new_session("Spa", "Porsche 911", "2025-01-02T10:00:00.000Z");
    qualifying_session.session_type = "qualifying".to_string();
    db.insert_session(&qualifying_session).await.unwrap();

    let mut practice_session = new_session("Spa", "Porsche 911", "2025-01-03T10:00:00.000Z");
    practice_session.session_type = "practice".to_string();
    db.insert_session(&practice_session).await.unwrap();

    // Filter by race
    let filters = FilterOptions {
        session_type: Some("race".to_string()),
        ..Default::default()
    };
    let sessions = db
        .list_sessions(ListOptions::default(), &filters)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_type, "race");

    // Filter by qualifying
    let filters = FilterOptions {
        session_type: Some("qualifying".to_string()),
        ..Default::default()
    };
    let sessions = db
        .list_sessions(ListOptions::default(), &filters)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_type, "qualifying");

    // Filter by practice
    let filters = FilterOptions {
        session_type: Some("practice".to_string()),
        ..Default::default()
    };
    let sessions = db
        .list_sessions(ListOptions::default(), &filters)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_type, "practice");
}

#[tokio::test]
async fn test_distinct_excludes_deleted_sessions() {
    let (db, _dir) = setup_db().await;

    let s1 = db
        .insert_session(&new_session(
            "UniqueTrack",
            "UniqueCar",
            "2025-01-01T00:00:00.000Z",
        ))
        .await
        .unwrap();

    // Verify it shows in distinct lists
    let tracks = db.get_distinct_tracks().await.unwrap();
    assert!(tracks.contains(&"UniqueTrack".to_string()));

    // Delete it
    db.soft_delete_session(&s1.id).await.unwrap();

    // Should no longer appear
    let tracks = db.get_distinct_tracks().await.unwrap();
    assert!(!tracks.contains(&"UniqueTrack".to_string()));

    let cars = db.get_distinct_cars().await.unwrap();
    assert!(!cars.contains(&"UniqueCar".to_string()));
}
