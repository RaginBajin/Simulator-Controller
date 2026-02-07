use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::StorageError;
use crate::types::*;

pub async fn insert_debrief(pool: &SqlitePool, new: &NewAiDebrief) -> Result<AiDebrief, StorageError> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    sqlx::query(
        "INSERT INTO ai_debriefs (id, session_id, coaching_text, insights_json, recommendations_json, provider_name, model_name, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)"
    )
    .bind(&id)
    .bind(&new.session_id)
    .bind(&new.coaching_text)
    .bind(&new.insights_json)
    .bind(&new.recommendations_json)
    .bind(&new.provider_name)
    .bind(&new.model_name)
    .bind(&now)
    .execute(pool)
    .await?;

    let debrief = sqlx::query_as::<_, AiDebrief>("SELECT * FROM ai_debriefs WHERE id = $1")
        .bind(&id)
        .fetch_one(pool)
        .await?;

    Ok(debrief)
}

pub async fn get_debrief_for_session(pool: &SqlitePool, session_id: &str) -> Result<Option<AiDebrief>, StorageError> {
    let debrief = sqlx::query_as::<_, AiDebrief>(
        "SELECT * FROM ai_debriefs WHERE session_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(debrief)
}

pub async fn update_debrief(pool: &SqlitePool, id: &str, update: DebriefUpdate) -> Result<AiDebrief, StorageError> {
    let existing = sqlx::query_as::<_, AiDebrief>("SELECT * FROM ai_debriefs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| StorageError::NotFound(id.to_string()))?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let coaching_text = update.coaching_text.or(existing.coaching_text);
    let insights_json = update.insights_json.or(existing.insights_json);
    let recommendations_json = update.recommendations_json.or(existing.recommendations_json);

    sqlx::query(
        "UPDATE ai_debriefs SET coaching_text = $1, insights_json = $2, recommendations_json = $3, updated_at = $4
         WHERE id = $5"
    )
    .bind(&coaching_text)
    .bind(&insights_json)
    .bind(&recommendations_json)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?;

    let debrief = sqlx::query_as::<_, AiDebrief>("SELECT * FROM ai_debriefs WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(debrief)
}
