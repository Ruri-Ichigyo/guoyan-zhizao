use std::collections::HashMap;

use axum::{extract::State, Json};

use crate::{errors::AppError, models::StatisticsResponse, AppState};

#[derive(sqlx::FromRow)]
struct UploadStatsRow {
    total: i64,
    completed: Option<i64>,
    failed: Option<i64>,
    processing: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct ResultStatsRow {
    total_fruits: Option<i64>,
    avg_confidence: Option<f64>,
}

pub async fn statistics(State(state): State<AppState>) -> Result<Json<StatisticsResponse>, AppError> {
    let upload_stats = sqlx::query_as::<_, UploadStatsRow>(
        "SELECT 
            COUNT(*)::bigint AS total,
            COUNT(*) FILTER (WHERE status = 'completed')::bigint AS completed,
            COUNT(*) FILTER (WHERE status = 'failed')::bigint AS failed,
            COUNT(*) FILTER (WHERE status = 'processing')::bigint AS processing
         FROM uploads",
    )
    .fetch_one(&state.pool)
    .await?;

    let result_stats = sqlx::query_as::<_, ResultStatsRow>(
        "SELECT 
            COALESCE(SUM(total_fruits), 0)::bigint AS total_fruits,
            AVG(avg_confidence)::double precision AS avg_confidence
         FROM results",
    )
    .fetch_one(&state.pool)
    .await?;

    let mut totals = HashMap::new();
    totals.insert("total".to_string(), upload_stats.total);
    totals.insert("completed".to_string(), upload_stats.completed.unwrap_or(0));
    totals.insert("failed".to_string(), upload_stats.failed.unwrap_or(0));
    totals.insert(
        "processing".to_string(),
        upload_stats.processing.unwrap_or(0),
    );
    totals.insert(
        "total_fruits".to_string(),
        result_stats.total_fruits.unwrap_or(0),
    );

    let mut averages = HashMap::new();
    averages.insert(
        "avg_confidence".to_string(),
        result_stats.avg_confidence.unwrap_or_default() as f32,
    );

    Ok(Json(StatisticsResponse { totals, averages }))
}
