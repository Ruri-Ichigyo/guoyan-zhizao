use axum::{
    extract::{Multipart, State},
    Json,
};
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

use crate::{
    cache, db,
    errors::AppError,
    models::{DetectionRecord, UploadResponsePayload},
    AppState,
};

const MAX_UPLOAD_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponsePayload>, AppError> {
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| AppError::Unexpected(err.to_string()))?
    {
        if field.name() == Some("file") {
            file_name = field
                .file_name()
                .map(|name| name.to_string())
                .or_else(|| Some("upload.jpg".to_string()));
            content_type = field.content_type().map(|ct| ct.to_string());

            let data = field
                .bytes()
                .await
                .map_err(|err| AppError::Unexpected(err.to_string()))?
                .to_vec();

            if data.len() > MAX_UPLOAD_SIZE {
                return Err(AppError::Validation("上传文件不能超过10MB".to_string()));
            }
            file_bytes = Some(data);
            break;
        }
    }

    let file_bytes =
        file_bytes.ok_or_else(|| AppError::Validation("未找到上传文件".to_string()))?;
    let filename = file_name.unwrap_or_else(|| "upload.jpg".to_string());
    let upload_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO uploads (id, filename, content_type, data, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $6)",
    )
    .bind(upload_id)
    .bind(&filename)
    .bind(content_type.as_deref())
    .bind(&file_bytes)
    .bind("processing")
    .bind(now)
    .execute(&state.pool)
    .await?;

    let ai_result = match state
        .ai_client
        .infer(
            upload_id,
            &filename,
            content_type.as_deref(),
            file_bytes.clone(),
        )
        .await
    {
        Ok(result) => result,
        Err(err) => {
            error!("AI inference failed: {}", err);
            db::mark_upload_failed(&state.pool, upload_id).await.ok();
            return Err(err);
        }
    };

    let detections = ai_result.results;
    let total_fruits = ai_result.total_fruits as i32;
    let mut grade_a = 0;
    let mut grade_b = 0;
    let mut grade_c = 0;

    let mut confidence_sum = 0.0_f32;
    for DetectionRecord {
        quality_grade,
        confidence,
        ..
    } in &detections
    {
        match quality_grade.as_str() {
            "A" | "a" => grade_a += 1,
            "B" | "b" => grade_b += 1,
            "C" | "c" => grade_c += 1,
            _ => {}
        }
        confidence_sum += confidence;
    }

    let avg_confidence = if detections.is_empty() {
        0.0
    } else {
        confidence_sum / detections.len() as f32
    };

    let completed_at = Utc::now();
    let detections_json =
        serde_json::to_value(&detections).map_err(|err| AppError::Unexpected(err.to_string()))?;

    let mut transaction = state.pool.begin().await?;

    sqlx::query("UPDATE uploads SET status = $1, updated_at = $2 WHERE id = $3")
        .bind("completed")
        .bind(completed_at)
        .bind(upload_id)
        .execute(&mut *transaction)
        .await?;

    sqlx::query(
        "INSERT INTO results (upload_id, status, detections, total_fruits, grade_a, grade_b, grade_c, avg_confidence, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (upload_id) DO UPDATE
         SET status = EXCLUDED.status,
             detections = EXCLUDED.detections,
             total_fruits = EXCLUDED.total_fruits,
             grade_a = EXCLUDED.grade_a,
             grade_b = EXCLUDED.grade_b,
             grade_c = EXCLUDED.grade_c,
             avg_confidence = EXCLUDED.avg_confidence,
             created_at = EXCLUDED.created_at",
    )
    .bind(upload_id)
    .bind(&ai_result.status)
    .bind(detections_json)
    .bind(total_fruits)
    .bind(grade_a)
    .bind(grade_b)
    .bind(grade_c)
    .bind(avg_confidence)
    .bind(completed_at)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    let payload = UploadResponsePayload {
        upload_id,
        filename: filename.clone(),
        status: "completed".to_string(),
        created_at: now,
        total_fruits,
        grade_a,
        grade_b,
        grade_c,
        avg_confidence,
        detections: detections.clone(),
    };

    let cache_key = format!("result:{upload_id}");
    let cache_value =
        serde_json::to_string(&payload).map_err(|err| AppError::Unexpected(err.to_string()))?;
    cache::cache_result(&state.redis, &cache_key, &cache_value, state.cache_ttl)
        .await
        .map_err(|err| AppError::Unexpected(err.to_string()))?;

    Ok(Json(payload))
}
