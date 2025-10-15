use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    cache,
    errors::AppError,
    models::DetectionRecord,
    AppState,
};

// ===== 请求/响应类型 =====

#[derive(Debug, Deserialize)]
pub struct RecordsQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub source: Option<String>,  // "all", "upload", "webcam", "rtsp", "webrtc"
    pub status: Option<String>,  // "all", "completed", "processing", "failed"
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRecord {
    pub id: Uuid,
    pub filename: String,
    pub source: String,
    pub record_type: String,  // "upload" or "detection"
    pub status: String,
    pub thumbnail_url: String,
    pub detections: Vec<DetectionRecord>,
    pub statistics: RecordStatistics,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordStatistics {
    pub total_detections: i32,  // 总检测数（重命名以保持一致性）
    pub grade_a: i32,           // A级数量（基于质量，非置信度）
    pub grade_b: i32,           // B级数量（基于质量，非置信度）
    pub grade_c: i32,           // C级数量（基于质量，非置信度）
    pub avg_confidence: f32,    // 平均识别置信度
    pub unique_classes: i32,    // 唯一类别数
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedRecordsResponse {
    pub data: Vec<UnifiedRecord>,
    pub pagination: PaginationInfo,
    pub summary: RecordsSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub pages: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordsSummary {
    pub total_records: i64,
    pub upload_records: i64,
    pub detection_records: i64,
    pub completed: i64,
    pub processing: i64,
    pub failed: i64,
}

// ===== API 处理函数 =====

/// 列出所有识别记录（统一查询）
pub async fn list_records(
    State(state): State<AppState>,
    Query(query): Query<RecordsQuery>,
) -> Result<Json<UnifiedRecordsResponse>, AppError> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    // 构建缓存键
    let cache_key = format!(
        "records:list:{}:{}:{}:{}",
        page,
        limit,
        query.source.as_deref().unwrap_or("all"),
        query.status.as_deref().unwrap_or("all")
    );

    // 尝试从缓存获取
    if let Some(cached) = cache::get_cached_result(&state.redis, &cache_key).await
        .map_err(|e| AppError::Unexpected(format!("Cache error: {}", e)))?
    {
        if let Ok(response) = serde_json::from_str::<UnifiedRecordsResponse>(&cached) {
            return Ok(Json(response));
        }
    }

    // 从数据库查询
    let mut all_records = Vec::new();
    let source_filter = query.source.as_deref().unwrap_or("all");

    // 1. 查询上传记录
    if source_filter == "all" || source_filter == "upload" {
        let upload_records = fetch_upload_records(&state, &query, offset, limit).await?;
        all_records.extend(upload_records);
    }

    // 2. 查询实时检测记录
    // 前端source映射：
    // - "webcam" -> 数据库的 live_camera, camera_1, camera_2
    // - "mobile" -> 数据库的 mobile_app
    // - "all" -> 所有检测记录
    if source_filter == "all"
        || source_filter == "webcam"
        || source_filter == "mobile"
    {
        let detection_records = fetch_detection_records(&state, &query, offset, limit).await?;
        all_records.extend(detection_records);
    }

    // 3. 按时间排序
    all_records.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // 4. 分页处理
    let total = all_records.len() as i64;
    let start = offset.min(total) as usize;
    let end = (offset + limit).min(total) as usize;
    let paginated_data = all_records[start..end].to_vec();

    // 5. 获取汇总统计
    let summary = fetch_summary(&state).await?;

    let pages = if total == 0 { 1 } else { (total + limit - 1) / limit };

    let response = UnifiedRecordsResponse {
        data: paginated_data,
        pagination: PaginationInfo {
            page,
            limit,
            total,
            pages,
        },
        summary,
    };

    // 缓存结果 (5分钟)
    if let Ok(json) = serde_json::to_string(&response) {
        let _ = cache::cache_result(&state.redis, &cache_key, &json, 300).await;
    }

    Ok(Json(response))
}

/// 获取单条记录详情
pub async fn get_record(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedRecord>, AppError> {
    let cache_key = format!("records:detail:{}", id);

    // 尝试从缓存获取
    if let Some(cached) = cache::get_cached_result(&state.redis, &cache_key).await
        .map_err(|e| AppError::Unexpected(format!("Cache error: {}", e)))?
    {
        if let Ok(record) = serde_json::from_str::<UnifiedRecord>(&cached) {
            return Ok(Json(record));
        }
    }

    // 先尝试从 uploads 表查找
    if let Some(record) = fetch_upload_by_id(&state, id).await? {
        // 缓存结果 (15分钟)
        if let Ok(json) = serde_json::to_string(&record) {
            let _ = cache::cache_result(&state.redis, &cache_key, &json, 900).await;
        }
        return Ok(Json(record));
    }

    // 再从 detections 表查找
    if let Some(record) = fetch_detection_by_id(&state, id).await? {
        // 缓存结果 (15分钟)
        if let Ok(json) = serde_json::to_string(&record) {
            let _ = cache::cache_result(&state.redis, &cache_key, &json, 900).await;
        }
        return Ok(Json(record));
    }

    Err(AppError::NotFound(format!("Record {} not found", id)))
}

/// 获取记录的图片数据
pub async fn get_record_image(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Vec<u8>), AppError> {
    let cache_key = format!("records:image:{}", id);

    // 尝试从缓存获取 (1小时)
    if let Some(cached) = cache::get_cached_result(&state.redis, &cache_key).await
        .map_err(|e| AppError::Unexpected(format!("Cache error: {}", e)))?
    {
        if let Ok(data) = serde_json::from_str::<Vec<u8>>(&cached) {
            return Ok((StatusCode::OK, data));
        }
    }

    // 先从 uploads 表查找
    let image_data: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT data FROM uploads WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(data) = image_data {
        // 缓存图片数据 (1小时)
        if let Ok(json) = serde_json::to_string(&data) {
            let _ = cache::cache_result(&state.redis, &cache_key, &json, 3600).await;
        }
        return Ok((StatusCode::OK, data));
    }

    // 再从 detections 表查找
    let snapshot_data: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT snapshot FROM detections WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(data) = snapshot_data {
        // 缓存快照数据 (1小时)
        if let Ok(json) = serde_json::to_string(&data) {
            let _ = cache::cache_result(&state.redis, &cache_key, &json, 3600).await;
        }
        return Ok((StatusCode::OK, data));
    }

    Err(AppError::NotFound(format!("Image for record {} not found", id)))
}

/// 删除记录
pub async fn delete_record(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // 先尝试从 uploads 删除
    let rows_affected = sqlx::query("DELETE FROM uploads WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows_affected > 0 {
        // 清除相关缓存
        let _ = invalidate_record_cache(&state.redis, id).await;
        return Ok(StatusCode::NO_CONTENT);
    }

    // 再尝试从 detections 删除
    let rows_affected = sqlx::query("DELETE FROM detections WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows_affected > 0 {
        let _ = invalidate_record_cache(&state.redis, id).await;
        return Ok(StatusCode::NO_CONTENT);
    }

    Err(AppError::NotFound(format!("Record {} not found", id)))
}

// ===== 辅助函数 =====

async fn fetch_upload_records(
    state: &AppState,
    query: &RecordsQuery,
    offset: i64,
    limit: i64,
) -> Result<Vec<UnifiedRecord>, AppError> {
    let mut sql = String::from(
        "SELECT u.id, u.filename, u.status, u.created_at, \
         r.detections, r.total_fruits, r.grade_a, r.grade_b, r.grade_c, r.avg_confidence \
         FROM uploads u \
         LEFT JOIN results r ON u.id = r.upload_id"
    );

    let mut conditions = Vec::new();

    if let Some(status) = &query.status {
        if status != "all" {
            conditions.push(format!("u.status = '{}'", status));
        }
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY u.created_at DESC");

    let rows: Vec<(Uuid, String, String, DateTime<Utc>, Option<serde_json::Value>, Option<i32>, Option<i32>, Option<i32>, Option<i32>, Option<f32>)> =
        sqlx::query_as(&sql)
            .fetch_all(&state.pool)
            .await?;

    let records = rows.into_iter().map(|row| {
        let mut detections: Vec<DetectionRecord> = if let Some(det_value) = row.4 {
            serde_json::from_value(det_value).unwrap_or_default()
        } else {
            Vec::new()
        };

        // 只保留置信度最高的检测结果
        if !detections.is_empty() {
            detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
            detections.truncate(1);  // 只保留第一个（最高置信度）
        }

        // 最高置信度（如果有检测结果）
        let max_confidence = detections.first().map(|d| d.confidence).unwrap_or(0.0);

        // 质量等级（基于最高置信度结果的quality_grade）
        let quality_grade = detections.first().map(|d| d.quality_grade.as_str()).unwrap_or("");
        let (grade_a, grade_b, grade_c) = match quality_grade {
            "A" | "a" => (1, 0, 0),
            "B" | "b" => (0, 1, 0),
            "C" | "c" => (0, 0, 1),
            _ => (0, 0, 0),
        };

        UnifiedRecord {
            id: row.0,
            filename: row.1,
            source: "upload".to_string(),
            record_type: "upload".to_string(),
            status: row.2,
            thumbnail_url: format!("/api/v1/records/{}/image", row.0),
            detections,  // 只包含最高置信度的一个结果
            statistics: RecordStatistics {
                total_detections: 1,              // 只显示最高置信度的结果，所以总数为1
                grade_a,                          // 基于最高置信度结果的质量等级
                grade_b,
                grade_c,
                avg_confidence: max_confidence,   // 使用最高置信度，而不是平均值
                unique_classes: 1,                // 只有一个类别
            },
            created_at: row.3,
        }
    }).collect();

    Ok(records)
}

async fn fetch_detection_records(
    state: &AppState,
    query: &RecordsQuery,
    offset: i64,
    limit: i64,
) -> Result<Vec<UnifiedRecord>, AppError> {
    let mut sql = String::from(
        "SELECT id, labels, confidence_scores, detected_classes, created_at, source \
         FROM detections"
    );

    let mut conditions = Vec::new();

    // Source过滤映射
    if let Some(source) = &query.source {
        match source.as_str() {
            "all" | "upload" => {
                // "all" 和 "upload" 不过滤detections表
            }
            "webcam" => {
                // 前端的 "webcam" 映射到数据库的摄像头相关来源
                conditions.push(
                    "source IN ('live_camera', 'camera_1', 'camera_2')".to_string()
                );
            }
            "mobile" => {
                // 前端的 "mobile" 映射到数据库的 mobile_app
                conditions.push("source = 'mobile_app'".to_string());
            }
            _ => {
                // 其他直接匹配数据库source值
                conditions.push(format!("source = '{}'", source));
            }
        }
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY created_at DESC");

    let rows: Vec<(Uuid, Vec<String>, Vec<f32>, Vec<String>, DateTime<Utc>, String)> =
        sqlx::query_as(&sql)
            .fetch_all(&state.pool)
            .await?;

    let records = rows.into_iter().map(|row| {
        // 将实时检测数据转换为统一格式
        let mut detections: Vec<DetectionRecord> = row.1.iter().zip(row.2.iter()).zip(row.3.iter())
            .map(|((label, confidence), _class)| {
                DetectionRecord {
                    class_name: label.clone(),
                    confidence: *confidence,
                    quality_grade: classify_grade_by_confidence(*confidence),
                    bbox: vec![0.0, 0.0, 0.0, 0.0],
                }
            })
            .collect();

        // 只保留置信度最高的检测结果
        if !detections.is_empty() {
            detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
            detections.truncate(1);  // 只保留第一个（最高置信度）
        }

        // 最高置信度
        let max_confidence = detections.first().map(|d| d.confidence).unwrap_or(0.0);

        // 质量等级（基于最高置信度结果）
        let quality_grade = detections.first().map(|d| d.quality_grade.as_str()).unwrap_or("");
        let (grade_a, grade_b, grade_c) = match quality_grade {
            "A" | "a" => (1, 0, 0),
            "B" | "b" => (0, 1, 0),
            "C" | "c" => (0, 0, 1),
            _ => (0, 0, 0),
        };

        UnifiedRecord {
            id: row.0,
            filename: format!("detection_{}.jpg", row.0.to_string().split('-').next().unwrap_or("unknown")),
            source: row.5,
            record_type: "detection".to_string(),
            status: "completed".to_string(),
            thumbnail_url: format!("/api/v1/records/{}/image", row.0),
            detections,  // 只包含最高置信度的一个结果
            statistics: RecordStatistics {
                total_detections: 1,              // 只有一个结果
                grade_a,                          // 基于最高置信度的分级
                grade_b,
                grade_c,
                avg_confidence: max_confidence,   // 最高置信度
                unique_classes: 1,                // 只有一个类别
            },
            created_at: row.4,
        }
    }).collect();

    Ok(records)
}

async fn fetch_upload_by_id(state: &AppState, id: Uuid) -> Result<Option<UnifiedRecord>, AppError> {
    let row: Option<(Uuid, String, String, DateTime<Utc>, Option<serde_json::Value>, Option<i32>, Option<i32>, Option<i32>, Option<i32>, Option<f32>)> =
        sqlx::query_as(
            "SELECT u.id, u.filename, u.status, u.created_at, \
             r.detections, r.total_fruits, r.grade_a, r.grade_b, r.grade_c, r.avg_confidence \
             FROM uploads u \
             LEFT JOIN results r ON u.id = r.upload_id \
             WHERE u.id = $1"
        )
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;

    Ok(row.map(|row| {
        let mut detections: Vec<DetectionRecord> = if let Some(det_value) = row.4 {
            serde_json::from_value(det_value).unwrap_or_default()
        } else {
            Vec::new()
        };

        // 只保留置信度最高的检测结果
        if !detections.is_empty() {
            detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
            detections.truncate(1);
        }

        // 最高置信度
        let max_confidence = detections.first().map(|d| d.confidence).unwrap_or(0.0);

        // 质量等级
        let quality_grade = detections.first().map(|d| d.quality_grade.as_str()).unwrap_or("");
        let (grade_a, grade_b, grade_c) = match quality_grade {
            "A" | "a" => (1, 0, 0),
            "B" | "b" => (0, 1, 0),
            "C" | "c" => (0, 0, 1),
            _ => (0, 0, 0),
        };

        UnifiedRecord {
            id: row.0,
            filename: row.1,
            source: "upload".to_string(),
            record_type: "upload".to_string(),
            status: row.2,
            thumbnail_url: format!("/api/v1/records/{}/image", row.0),
            detections,
            statistics: RecordStatistics {
                total_detections: 1,
                grade_a,
                grade_b,
                grade_c,
                avg_confidence: max_confidence,
                unique_classes: 1,
            },
            created_at: row.3,
        }
    }))
}

async fn fetch_detection_by_id(state: &AppState, id: Uuid) -> Result<Option<UnifiedRecord>, AppError> {
    let row: Option<(Uuid, Vec<String>, Vec<f32>, Vec<String>, DateTime<Utc>, String)> =
        sqlx::query_as(
            "SELECT id, labels, confidence_scores, detected_classes, created_at, source \
             FROM detections \
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;

    Ok(row.map(|row| {
        let mut detections: Vec<DetectionRecord> = row.1.iter().zip(row.2.iter()).zip(row.3.iter())
            .map(|((label, confidence), _class)| {
                DetectionRecord {
                    class_name: label.clone(),
                    confidence: *confidence,
                    quality_grade: classify_grade_by_confidence(*confidence),
                    bbox: vec![0.0, 0.0, 0.0, 0.0],
                }
            })
            .collect();

        // 只保留置信度最高的检测结果
        if !detections.is_empty() {
            detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
            detections.truncate(1);
        }

        // 最高置信度
        let max_confidence = detections.first().map(|d| d.confidence).unwrap_or(0.0);

        // 质量等级
        let quality_grade = detections.first().map(|d| d.quality_grade.as_str()).unwrap_or("");
        let (grade_a, grade_b, grade_c) = match quality_grade {
            "A" | "a" => (1, 0, 0),
            "B" | "b" => (0, 1, 0),
            "C" | "c" => (0, 0, 1),
            _ => (0, 0, 0),
        };

        UnifiedRecord {
            id: row.0,
            filename: format!("detection_{}.jpg", row.0.to_string().split('-').next().unwrap_or("unknown")),
            source: row.5,
            record_type: "detection".to_string(),
            status: "completed".to_string(),
            thumbnail_url: format!("/api/v1/records/{}/image", row.0),
            detections,
            statistics: RecordStatistics {
                total_detections: 1,
                grade_a,
                grade_b,
                grade_c,
                avg_confidence: max_confidence,
                unique_classes: 1,
            },
            created_at: row.4,
        }
    }))
}

async fn fetch_summary(state: &AppState) -> Result<RecordsSummary, AppError> {
    // 统计上传记录
    let upload_stats: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT \
         COUNT(*) as total, \
         COUNT(*) FILTER (WHERE status = 'completed') as completed, \
         COUNT(*) FILTER (WHERE status = 'processing') as processing, \
         COUNT(*) FILTER (WHERE status = 'failed') as failed \
         FROM uploads"
    )
    .fetch_one(&state.pool)
    .await?;

    // 统计检测记录
    let detection_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM detections")
        .fetch_one(&state.pool)
        .await?;

    Ok(RecordsSummary {
        total_records: upload_stats.0 + detection_count,
        upload_records: upload_stats.0,
        detection_records: detection_count,
        completed: upload_stats.1 + detection_count, // 检测记录都视为completed
        processing: upload_stats.2,
        failed: upload_stats.3,
    })
}

async fn invalidate_record_cache(client: &redis::Client, id: Uuid) -> Result<(), AppError> {
    // 清除特定记录的所有缓存
    let patterns = vec![
        format!("records:detail:{}", id),
        format!("records:image:{}", id),
        "records:list:*".to_string(),
    ];

    for pattern in patterns {
        let _ = cache::get_cached_result(client, &pattern).await;
    }

    Ok(())
}

/// 根据识别置信度分级（仅用于实时检测，因为没有质量评估）
fn classify_grade_by_confidence(confidence: f32) -> String {
    if confidence >= 0.90 {
        "A".to_string()
    } else if confidence >= 0.75 {
        "B".to_string()
    } else {
        "C".to_string()
    }
}

/// 统计各等级数量（基于 quality_grade 字段）
fn count_grades(detections: &[DetectionRecord]) -> (i32, i32, i32) {
    let mut grade_a = 0;
    let mut grade_b = 0;
    let mut grade_c = 0;

    for det in detections {
        match det.quality_grade.as_str() {
            "A" | "a" => grade_a += 1,
            "B" | "b" => grade_b += 1,
            "C" | "c" => grade_c += 1,
            _ => {}
        }
    }

    (grade_a, grade_b, grade_c)
}
