//! # Routes Module
//!
//! 查询服务的路由处理模块，定义了检测结果查询相关的API接口。
//! 包含获取单个检测结果和分页查询检测结果列表的功能。

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::SharedState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultResponse {
    /// 任务的唯一标识符
    pub job_id: String,
    /// 任务的当前状态（如：completed, processing, failed）
    pub status: String,
    /// 检测到的物体结果列表
    pub results: Vec<DetectionResult>,
    /// 任务创建时间（ISO 8601格式）
    pub created_at: String,
}

/// 单个检测结果
///
/// 表示在图像中检测到的一个果蔬物体的详细信息
#[derive(Debug, Serialize, Deserialize)]
pub struct DetectionResult {
    /// 物体类别名称（如：apple, orange, banana等）
    pub class_name: String,
    /// 检测置信度（0.0-1.0范围）
    pub confidence: f32,
    /// 边界框坐标 [x1, y1, x2, y2]，表示物体在图像中的位置
    pub bbox: Vec<f32>,
    /// 质量等级（A/B/C）
    pub quality_grade: String,
}

/// 查询参数结构
///
/// 用于分页查询和筛选检测结果列表
/// 查询参数结构
///
/// 用于分页查询和筛选检测结果列表
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    /// 页码（从1开始）
    pub page: Option<u32>,
    /// 每页数量限制
    pub limit: Option<u32>,
    /// 筛选的状态（如：completed, processing）
    #[allow(dead_code)]
    pub status: Option<String>,
}

/// 获取指定任务的检测结果
///
/// 根据任务ID获取对应的检测结果详细信息。
/// 目前返回模拟数据，未来将从数据库中查询。
///
/// # Arguments
///
/// * `job_id` - 任务的唯一标识符
/// * `state` - 应用程序状态
///
/// # Returns
///
/// 返回包含检测结果的JSON响应，或者错误状态码和错误信息
pub async fn get_result(
    Path(job_id): Path<String>,
    State(_state): State<SharedState>,
) -> Result<Json<ResultResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Getting result for job: {}", job_id);

    // TODO: Query from database
    // For now, return mock data
    let mock_result = ResultResponse {
        job_id: job_id.clone(),
        status: "completed".to_string(),
        results: vec![
            DetectionResult {
                class_name: "apple".to_string(),
                confidence: 0.95,
                bbox: vec![10.0, 20.0, 100.0, 150.0],
                quality_grade: "A".to_string(),
            },
            DetectionResult {
                class_name: "orange".to_string(),
                confidence: 0.89,
                bbox: vec![200.0, 50.0, 300.0, 200.0],
                quality_grade: "B".to_string(),
            },
        ],
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(mock_result))
}

/// 获取检测结果列表
///
/// 分页查询检测结果列表，支持按状态筛选。
/// 目前返回模拟数据，未来将从数据库中查询。
///
/// # Arguments
///
/// * `params` - 查询参数，包含分页信息和筛选条件
/// * `state` - 应用程序状态
///
/// # Returns
///
/// 返回包含检测结果列表和分页信息的JSON响应
pub async fn list_results(
    Query(params): Query<QueryParams>,
    State(_state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);

    info!("Listing results: page={}, limit={}", page, limit);

    // TODO: Query from database with pagination
    // For now, return mock data
    let mock_results = serde_json::json!({
        "data": [
            {
                "job_id": "test-job-1",
                "status": "completed",
                "total_fruits": 15,
                "grade_a": 8,
                "grade_b": 5,
                "grade_c": 2,
                "created_at": chrono::Utc::now().to_rfc3339()
            }
        ],
        "pagination": {
            "page": page,
            "limit": limit,
            "total": 1,
            "pages": 1
        }
    });

    Ok(Json(mock_results))
}
