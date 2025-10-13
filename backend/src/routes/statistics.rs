use std::collections::HashMap;

use axum::{extract::State, Json};
use serde::Serialize;

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
    grade_a_total: Option<i64>,
    grade_b_total: Option<i64>,
    grade_c_total: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct CategoryStatsRow {
    class_name: String,
    count: i64,
}

#[derive(Debug, Serialize)]
pub struct EnhancedStatisticsResponse {
    pub totals: HashMap<String, i64>,
    pub averages: HashMap<String, f32>,
    pub quality_distribution: HashMap<String, i64>,
    pub category_distribution: Vec<CategoryData>,
}

#[derive(Debug, Serialize)]
pub struct CategoryData {
    pub name: String,
    pub value: i64,
}

pub async fn statistics(State(state): State<AppState>) -> Result<Json<EnhancedStatisticsResponse>, AppError> {
    // Get upload stats
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

    // Get result stats with quality grades
    let result_stats = sqlx::query_as::<_, ResultStatsRow>(
        "SELECT
            COALESCE(SUM(total_fruits), 0)::bigint AS total_fruits,
            AVG(avg_confidence)::double precision AS avg_confidence,
            COALESCE(SUM(grade_a), 0)::bigint AS grade_a_total,
            COALESCE(SUM(grade_b), 0)::bigint AS grade_b_total,
            COALESCE(SUM(grade_c), 0)::bigint AS grade_c_total
         FROM results",
    )
    .fetch_one(&state.pool)
    .await?;

    // Get category distribution from detections JSON
    let category_stats = sqlx::query_as::<_, CategoryStatsRow>(
        "SELECT
            detection->>'class_name' as class_name,
            COUNT(*)::bigint as count
         FROM results,
              jsonb_array_elements(detections) as detection
         GROUP BY detection->>'class_name'
         ORDER BY count DESC
         LIMIT 10",
    )
    .fetch_all(&state.pool)
    .await?;

    // Build totals map
    let mut totals = HashMap::new();
    totals.insert("total".to_string(), upload_stats.total);
    totals.insert("completed".to_string(), upload_stats.completed.unwrap_or(0));
    totals.insert("failed".to_string(), upload_stats.failed.unwrap_or(0));
    totals.insert("processing".to_string(), upload_stats.processing.unwrap_or(0));
    totals.insert("total_fruits".to_string(), result_stats.total_fruits.unwrap_or(0));

    // Build averages map
    let mut averages = HashMap::new();
    averages.insert(
        "avg_confidence".to_string(),
        result_stats.avg_confidence.unwrap_or_default() as f32,
    );

    // Build quality distribution
    let mut quality_distribution = HashMap::new();
    quality_distribution.insert("grade_a".to_string(), result_stats.grade_a_total.unwrap_or(0));
    quality_distribution.insert("grade_b".to_string(), result_stats.grade_b_total.unwrap_or(0));
    quality_distribution.insert("grade_c".to_string(), result_stats.grade_c_total.unwrap_or(0));

    // Build category distribution
    let category_distribution: Vec<CategoryData> = category_stats
        .into_iter()
        .map(|row| CategoryData {
            name: row.class_name,
            value: row.count,
        })
        .collect();

    Ok(Json(EnhancedStatisticsResponse {
        totals,
        averages,
        quality_distribution,
        category_distribution,
    }))
}
