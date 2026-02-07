use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::StorageError;
use crate::types::*;

pub async fn insert_lap(pool: &SqlitePool, new: &NewLap) -> Result<LapSummary, StorageError> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO lap_summaries (id, session_id, lap_number, lap_time_ms, is_valid, completion_status)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&id)
    .bind(&new.session_id)
    .bind(new.lap_number)
    .bind(new.lap_time_ms)
    .bind(new.is_valid)
    .bind(&new.completion_status)
    .execute(pool)
    .await?;

    let lap = sqlx::query_as::<_, LapSummary>("SELECT * FROM lap_summaries WHERE id = $1")
        .bind(&id)
        .fetch_one(pool)
        .await?;

    Ok(lap)
}

pub async fn get_laps_for_session(pool: &SqlitePool, session_id: &str) -> Result<Vec<LapSummary>, StorageError> {
    let laps = sqlx::query_as::<_, LapSummary>(
        "SELECT * FROM lap_summaries WHERE session_id = $1 ORDER BY lap_number ASC"
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(laps)
}
