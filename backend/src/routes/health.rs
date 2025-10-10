use axum::{extract::State, Json};
use serde_json::json;

use crate::{cache, errors::AppError, AppState};

pub async fn health(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;

    let mut conn = cache::get_connection(&state.redis).await?;
    redis::cmd("PING")
        .query_async::<_, String>(&mut conn)
        .await?;
    Ok(Json(json!({ "status": "ok" })))
}
