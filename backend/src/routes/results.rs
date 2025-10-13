use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{
    cache,
    errors::AppError,
    models::{
        DetailedRow, PaginatedResults, PaginationPayload, ResultSummary, SummaryRow,
        UploadResponsePayload,
    },
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>,
}

pub async fn list_results(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<PaginatedResults>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut list_builder = QueryBuilder::new(
        "SELECT u.id, u.filename, u.status, u.created_at, \
         r.total_fruits, r.grade_a, r.grade_b, r.grade_c, r.avg_confidence \
         FROM uploads u \
         LEFT JOIN results r ON u.id = r.upload_id",
    );

    if let Some(status) = &query.status {
        list_builder.push(" WHERE u.status = ");
        list_builder.push_bind(status);
    }

    list_builder.push(" ORDER BY u.created_at DESC LIMIT ");
    list_builder.push_bind(limit);
    list_builder.push(" OFFSET ");
    list_builder.push_bind(offset);

    let summaries: Vec<SummaryRow> = list_builder.build_query_as().fetch_all(&state.pool).await?;

    let mut count_builder = QueryBuilder::new("SELECT COUNT(*) as total FROM uploads u");
    if let Some(status) = &query.status {
        count_builder.push(" WHERE u.status = ");
        count_builder.push_bind(status);
    }

    let total: (i64,) = count_builder
        .build_query_as()
        .fetch_one(&state.pool)
        .await?;

    let data = summaries
        .into_iter()
        .map(|row| ResultSummary {
            upload_id: row.id,
            filename: row.filename,
            status: row.status,
            created_at: row.created_at,
            total_fruits: row.total_fruits,
            grade_a: row.grade_a,
            grade_b: row.grade_b,
            grade_c: row.grade_c,
            avg_confidence: row.avg_confidence,
        })
        .collect::<Vec<_>>();

    let pages = if total.0 == 0 {
        1
    } else {
        ((total.0 as f64) / (limit as f64)).ceil() as i64
    };

    let payload = PaginatedResults {
        data,
        pagination: PaginationPayload {
            page,
            limit,
            total: total.0,
            pages,
        },
    };

    Ok(Json(payload))
}

pub async fn get_result(
    State(state): State<AppState>,
    Path(result_id): Path<Uuid>,
) -> Result<Json<UploadResponsePayload>, AppError> {
    let cache_key = format!("result:{result_id}");
    if let Some(cached) = cache::get_cached_result(&state.redis, &cache_key)
        .await
        .map_err(|err| AppError::Unexpected(err.to_string()))?
    {
        let payload: UploadResponsePayload =
            serde_json::from_str(&cached).map_err(|err| AppError::Unexpected(err.to_string()))?;
        return Ok(Json(payload));
    }

    let row: Option<DetailedRow> = sqlx::query_as(
        "SELECT u.id, u.filename, u.status, u.created_at, u.updated_at, \
                r.status as result_status, r.detections, r.total_fruits, \
                r.grade_a, r.grade_b, r.grade_c, r.avg_confidence \
         FROM uploads u \
         LEFT JOIN results r ON u.id = r.upload_id \
         WHERE u.id = $1",
    )
    .bind(result_id)
    .fetch_optional(&state.pool)
    .await?;

    let Some(row) = row else {
        return Err(AppError::NotFound(format!("Result {} not found", result_id)));
    };

    let detections_value = row
        .detections
        .unwrap_or_else(|| serde_json::Value::Array(Vec::new()));
    let detections: Vec<crate::models::DetectionRecord> = serde_json::from_value(detections_value)
        .map_err(|err| AppError::Unexpected(err.to_string()))?;

    let payload = UploadResponsePayload {
        upload_id: row.id,
        filename: row.filename,
        status: row.result_status.unwrap_or(row.status),
        created_at: row.created_at,
        total_fruits: row.total_fruits.unwrap_or_default(),
        grade_a: row.grade_a.unwrap_or_default(),
        grade_b: row.grade_b.unwrap_or_default(),
        grade_c: row.grade_c.unwrap_or_default(),
        avg_confidence: row.avg_confidence.unwrap_or_default(),
        detections,
    };

    let cache_value =
        serde_json::to_string(&payload).map_err(|err| AppError::Unexpected(err.to_string()))?;
    cache::cache_result(&state.redis, &cache_key, &cache_value, state.cache_ttl)
        .await
        .map_err(|err| AppError::Unexpected(err.to_string()))?;

    Ok(Json(payload))
}
