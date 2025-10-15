# API 接口文档

本文档描述果蔬智造平台的完整 REST API 接口。

## 基础信息

- **后端基础 URL**: `http://localhost:8080/api/v1`
- **AI 服务 URL**: `http://localhost:8002`
- **内容类型**: `application/json`
- **上传文件**: `multipart/form-data`
- **时间格式**: ISO 8601 (UTC)
- **字符编码**: UTF-8

## 认证说明

当前版本接口无需认证，部署在受控网络环境中即可直接访问。生产环境建议配置反向代理或防火墙规则。

---

## 后端 API (Rust Axum)

### 健康检查

#### GET /health

检查后端服务运行状态。

**请求示例**:
```bash
curl http://localhost:8080/health
```

**响应示例**:
```json
{
  "status": "ok"
}
```

---

## 核心功能接口

### 1. 图像上传与分析

#### POST /api/v1/uploads

上传图片文件，触发 AI 分析并返回检测结果。

**请求格式**: `multipart/form-data`

**参数**:
| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `file` | File | 是 | 图片文件（支持 JPG/PNG） |

**请求示例**:
```bash
curl -X POST http://localhost:8080/api/v1/uploads \
  -F "file=@fruit.jpg"
```

**成功响应** (200 OK):
```json
{
  "upload_id": "0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36",
  "filename": "fruit.jpg",
  "status": "completed",
  "created_at": "2025-10-14T08:30:00Z",
  "total_fruits": 3,
  "grade_a": 2,
  "grade_b": 1,
  "grade_c": 0,
  "avg_confidence": 0.91,
  "detections": [
    {
      "class_name": "Apple Red 1",
      "confidence": 0.95,
      "bbox": [10.5, 20.3, 150.2, 180.7],
      "quality_grade": "A"
    },
    {
      "class_name": "Apple Red 1",
      "confidence": 0.88,
      "bbox": [160.0, 25.0, 280.5, 190.2],
      "quality_grade": "A"
    },
    {
      "class_name": "Banana",
      "confidence": 0.90,
      "bbox": [20.0, 200.0, 300.0, 350.0],
      "quality_grade": "B"
    }
  ]
}
```

**错误响应**:
- `400 Bad Request`: 文件格式不支持或文件损坏
- `413 Payload Too Large`: 文件大小超过限制（默认 10MB）
- `500 Internal Server Error`: AI 服务异常

---

### 2. 统一记录查询 (NEW)

#### GET /api/v1/records

查询所有记录（融合上传记录和实时检测），支持分页和筛选。

**查询参数**:
| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `page` | int | 否 | 1 | 页码（从 1 开始） |
| `limit` | int | 否 | 20 | 每页记录数（1-100） |
| `source` | string | 否 | all | 数据来源筛选 |
| `status` | string | 否 | - | 状态筛选 |

**source 可选值**:
- `all`: 所有来源
- `upload`: 图像上传记录
- `webcam`: 摄像头记录（包括 live_camera, camera_1, camera_2）
- `mobile`: 移动应用记录

**status 可选值**:
- `completed`: 已完成
- `processing`: 处理中
- `failed`: 失败
- `pending`: 待处理

**请求示例**:
```bash
# 查询所有记录
curl "http://localhost:8080/api/v1/records?page=1&limit=20"

# 筛选上传且已完成的记录
curl "http://localhost:8080/api/v1/records?source=upload&status=completed"

# 查询摄像头记录
curl "http://localhost:8080/api/v1/records?source=webcam&limit=50"
```

**成功响应** (200 OK):
```json
{
  "data": [
    {
      "id": "0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36",
      "filename": "fruit_image.jpg",
      "source": "upload",
      "record_type": "upload",
      "status": "completed",
      "thumbnail_url": "/api/v1/records/0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36/image",
      "detections": [
        {
          "class_name": "Apple Red 1",
          "confidence": 0.95,
          "bbox": [10.5, 20.3, 150.2, 180.7],
          "quality_grade": "A"
        }
      ],
      "statistics": {
        "total_detections": 1,
        "grade_a": 1,
        "grade_b": 0,
        "grade_c": 0,
        "avg_confidence": 0.95,
        "unique_classes": 1
      },
      "created_at": "2025-10-14T08:30:00Z"
    },
    {
      "id": "1bc9d7d8-42e5-4c55-b794-7f2b6e8e2d47",
      "filename": "detection_1bc9d7d8.jpg",
      "source": "live_camera",
      "record_type": "detection",
      "status": "completed",
      "thumbnail_url": "/api/v1/records/1bc9d7d8-42e5-4c55-b794-7f2b6e8e2d47/image",
      "detections": [
        {
          "class_name": "Banana",
          "confidence": 0.88,
          "bbox": [0.0, 0.0, 0.0, 0.0],
          "quality_grade": "B"
        }
      ],
      "statistics": {
        "total_detections": 1,
        "grade_a": 0,
        "grade_b": 1,
        "grade_c": 0,
        "avg_confidence": 0.88,
        "unique_classes": 1
      },
      "created_at": "2025-10-14T07:15:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 82,
    "pages": 5
  },
  "summary": {
    "total_records": 82,
    "upload_records": 50,
    "detection_records": 32,
    "completed": 75,
    "processing": 5,
    "failed": 2
  }
}
```

#### GET /api/v1/records/:id

获取单条记录的详细信息。

**路径参数**:
| 参数 | 类型 | 说明 |
| --- | --- | --- |
| `id` | UUID | 记录 ID |

**请求示例**:
```bash
curl http://localhost:8080/api/v1/records/0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36
```

**成功响应** (200 OK):
```json
{
  "id": "0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36",
  "filename": "fruit_image.jpg",
  "source": "upload",
  "record_type": "upload",
  "status": "completed",
  "thumbnail_url": "/api/v1/records/0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36/image",
  "detections": [
    {
      "class_name": "Apple Red 1",
      "confidence": 0.95,
      "bbox": [10.5, 20.3, 150.2, 180.7],
      "quality_grade": "A"
    }
  ],
  "statistics": {
    "total_detections": 1,
    "grade_a": 1,
    "grade_b": 0,
    "grade_c": 0,
    "avg_confidence": 0.95,
    "unique_classes": 1
  },
  "created_at": "2025-10-14T08:30:00Z"
}
```

**错误响应**:
- `404 Not Found`: 记录不存在

#### GET /api/v1/records/:id/image

获取记录对应的图片或快照数据。

**路径参数**:
| 参数 | 类型 | 说明 |
| --- | --- | --- |
| `id` | UUID | 记录 ID |

**请求示例**:
```bash
curl http://localhost:8080/api/v1/records/0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36/image \
  --output image.jpg
```

**成功响应** (200 OK):
- 返回二进制图片数据
- Content-Type: `image/jpeg` 或 `image/png`

**错误响应**:
- `404 Not Found`: 图片不存在

#### DELETE /api/v1/records/:id

删除指定记录（包括关联的图片和检测数据）。

**路径参数**:
| 参数 | 类型 | 说明 |
| --- | --- | --- |
| `id` | UUID | 记录 ID |

**请求示例**:
```bash
curl -X DELETE http://localhost:8080/api/v1/records/0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36
```

**成功响应** (204 No Content):
- 无响应体

**错误响应**:
- `404 Not Found`: 记录不存在

---

### 3. 统计信息

#### GET /api/v1/statistics

获取系统整体统计数据。

**请求示例**:
```bash
curl http://localhost:8080/api/v1/statistics
```

**成功响应** (200 OK):
```json
{
  "totals": {
    "total": 82,
    "completed": 75,
    "processing": 5,
    "failed": 2,
    "total_fruits": 320
  },
  "averages": {
    "avg_confidence": 0.87
  }
}
```

---

## 兼容接口（向后兼容）

### 上传结果查询

#### GET /api/v1/results

查询上传记录列表（仅上传图片的结果）。

**查询参数**:
| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `page` | int | 否 | 1 | 页码 |
| `limit` | int | 否 | 10 | 每页记录数 |
| `status` | string | 否 | - | 状态筛选 |

**请求示例**:
```bash
curl "http://localhost:8080/api/v1/results?page=1&limit=10&status=completed"
```

**成功响应** (200 OK):
```json
{
  "data": [
    {
      "upload_id": "0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36",
      "filename": "fruit.jpg",
      "status": "completed",
      "created_at": "2025-10-14T08:30:00Z",
      "total_fruits": 3,
      "grade_a": 2,
      "grade_b": 1,
      "grade_c": 0,
      "avg_confidence": 0.91
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 10,
    "total": 50,
    "pages": 5
  }
}
```

#### GET /api/v1/results/:upload_id

获取单条上传结果详情。

**成功响应** (200 OK):
```json
{
  "upload_id": "0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36",
  "filename": "fruit.jpg",
  "status": "completed",
  "created_at": "2025-10-14T08:30:00Z",
  "total_fruits": 3,
  "grade_a": 2,
  "grade_b": 1,
  "grade_c": 0,
  "avg_confidence": 0.91,
  "detections": [
    {
      "class_name": "Apple Red 1",
      "confidence": 0.95,
      "bbox": [10.5, 20.3, 150.2, 180.7],
      "quality_grade": "A"
    }
  ]
}
```

### 实时检测接口

#### POST /api/v1/detections

创建实时检测记录（用于摄像头或移动应用）。

**请求体**:
```json
{
  "source": "live_camera",
  "labels": ["Apple Red 1", "Banana"],
  "confidence_scores": [0.95, 0.88],
  "detected_classes": ["apple", "banana"],
  "snapshot": "base64_encoded_image_data..."
}
```

**成功响应** (201 Created):
```json
{
  "id": "1bc9d7d8-42e5-4c55-b794-7f2b6e8e2d47",
  "source": "live_camera",
  "created_at": "2025-10-14T07:15:00Z"
}
```

#### GET /api/v1/detections

查询实时检测记录列表。

**查询参数**:
- `page`: 页码
- `limit`: 每页记录数
- `source`: 来源筛选（live_camera, mobile_app 等）

#### GET /api/v1/detections/:id

获取单条检测记录详情。

#### GET /api/v1/detections/:id/snapshot

获取检测快照图片。

---

## AI 服务 API (Python FastAPI)

### 健康检查

#### GET /health

检查 AI 服务运行状态。

**请求示例**:
```bash
curl http://localhost:8002/health
```

**响应示例**:
```json
{
  "status": "ok"
}
```

### 图像推理

#### POST /infer

执行图像分类和质量评估。

**请求格式**: `multipart/form-data`

**参数**:
| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `upload_id` | string | 是 | 上传任务 ID |
| `file` | File | 是 | 图片文件 |
| `model_name` | string | 否 | 模型名称（默认: densenet121-fruit-v1） |

**请求示例**:
```bash
curl -X POST http://localhost:8002/infer \
  -F "upload_id=test-001" \
  -F "file=@fruit.jpg" \
  -F "model_name=densenet121-fruit-v1"
```

**成功响应** (200 OK):
```json
{
  "upload_id": "test-001",
  "status": "completed",
  "total_fruits": 1,
  "timestamp": "2025-10-14T08:30:00Z",
  "results": [
    {
      "class_name": "Apple Red 1",
      "confidence": 0.95,
      "bbox": [0.0, 0.0, 224.0, 224.0],
      "quality_grade": "A"
    }
  ],
  "metadata": {
    "model": "densenet121-fruit-v1",
    "model_version": "v1.0.0",
    "device": "cuda:0",
    "top_k": "3"
  }
}
```

### 模型管理

#### GET /models

列出所有可用模型。

**成功响应** (200 OK):
```json
{
  "models": [
    {
      "name": "densenet121-fruit-v1",
      "version": "v1.0.0",
      "classes": 60,
      "input_size": [224, 224],
      "device": "cuda:0"
    }
  ]
}
```

#### GET /models/:model_name

获取特定模型详情。

#### GET /metrics

获取服务运行指标。

**成功响应** (200 OK):
```json
{
  "inference_count": 1523,
  "avg_inference_time_ms": 45.2,
  "cache_hit_rate": 0.78,
  "uptime_seconds": 3600
}
```

---

## AI 聊天接口

### 1. 会话管理

#### POST /api/v1/chat/sessions

创建新的聊天会话。

**请求体**:
```json
{
  "title": "关于果蔬识别的问题"
}
```

**成功响应** (200 OK):
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "关于果蔬识别的问题",
  "status": "active",
  "created_at": "2025-10-14T10:00:00Z"
}
```

#### GET /api/v1/chat/sessions

获取用户的聊天会话列表。

**查询参数**:
| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | string | 否 | 过滤状态：`active`/`inactive`/`archived` |
| `limit` | number | 否 | 返回数量（默认 20） |
| `offset` | number | 否 | 偏移量（默认 0） |

**请求示例**:
```bash
curl "http://localhost:8080/api/v1/chat/sessions?status=active&limit=10"
```

**成功响应** (200 OK):
```json
{
  "sessions": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "关于果蔬识别的问题",
      "status": "active",
      "created_at": "2025-10-14T10:00:00Z",
      "updated_at": "2025-10-14T10:15:00Z",
      "message_count": 5
    }
  ],
  "total": 1
}
```

#### DELETE /api/v1/chat/sessions/:id

删除指定的聊天会话（包括所有消息）。

**路径参数**:
| 参数 | 类型 | 说明 |
| --- | --- | --- |
| `id` | UUID | 会话 ID |

**请求示例**:
```bash
curl -X DELETE http://localhost:8080/api/v1/chat/sessions/550e8400-e29b-41d4-a716-446655440000
```

**成功响应** (204 No Content):
- 无响应体

### 2. 消息交互

#### POST /api/v1/chat/message

向 AI 助手发送消息并获取响应。

**请求体**:
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "今天总共上传了多少张图片？"
}
```

**成功响应** (200 OK):
```json
{
  "message_id": "660e8400-e29b-41d4-a716-446655440001",
  "role": "assistant",
  "content": "根据数据库查询，今天总共上传了 82 张图片。",
  "tool_calls": [
    {
      "tool": "database_query",
      "input": "SELECT COUNT(*) FROM uploads WHERE DATE(created_at) = CURRENT_DATE",
      "output": "82"
    }
  ],
  "created_at": "2025-10-14T10:01:00Z"
}
```

#### GET /api/v1/chat/history/:session_id

获取指定会话的历史消息。

**路径参数**:
| 参数 | 类型 | 说明 |
| --- | --- | --- |
| `session_id` | UUID | 会话 ID |

**查询参数**:
| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `limit` | number | 否 | 返回数量（默认 50） |
| `before` | string | 否 | 获取此时间之前的消息（ISO 8601） |

**请求示例**:
```bash
curl "http://localhost:8080/api/v1/chat/history/550e8400-e29b-41d4-a716-446655440000?limit=20"
```

**成功响应** (200 OK):
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "messages": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "role": "user",
      "content": "今天总共上传了多少张图片？",
      "created_at": "2025-10-14T10:00:30Z"
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440002",
      "role": "assistant",
      "content": "根据数据库查询，今天总共上传了 82 张图片。",
      "metadata": {
        "tool_calls": [
          {
            "tool": "database_query",
            "success": true
          }
        ]
      },
      "created_at": "2025-10-14T10:01:00Z"
    }
  ],
  "total": 2
}
```

#### GET /api/v1/chat/messages/new

轮询获取会话的新消息（长轮询）。

**查询参数**:
| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `session_id` | UUID | 是 | 会话 ID |
| `since` | string | 是 | 获取此时间之后的消息（ISO 8601） |

**请求示例**:
```bash
curl "http://localhost:8080/api/v1/chat/messages/new?session_id=550e8400-e29b-41d4-a716-446655440000&since=2025-10-14T10:00:00Z"
```

**成功响应** (200 OK):
```json
{
  "messages": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440003",
      "role": "assistant",
      "content": "检测已完成",
      "created_at": "2025-10-14T10:02:00Z"
    }
  ],
  "has_more": false
}
```

---

## 数据模型定义

### UnifiedRecord (统一记录)

```typescript
{
  id: string                    // UUID
  filename: string              // 文件名
  source: string                // 数据来源
  record_type: string           // 'upload' | 'detection'
  status: string                // 'completed' | 'processing' | 'failed'
  thumbnail_url: string         // 缩略图 URL
  detections: DetectionRecord[] // 检测结果数组
  statistics: RecordStatistics  // 统计数据
  created_at: string            // ISO 8601 时间戳
}
```

### DetectionRecord (检测结果)

```typescript
{
  class_name: string      // 类别名称
  confidence: number      // 置信度 (0-1)
  bbox: number[]          // 边界框 [x1, y1, x2, y2]
  quality_grade: string   // 质量等级 'A' | 'B' | 'C'
}
```

### RecordStatistics (记录统计)

```typescript
{
  total_detections: number    // 总检测数
  grade_a: number             // A 级数量
  grade_b: number             // B 级数量
  grade_c: number             // C 级数量
  avg_confidence: number      // 平均置信度
  unique_classes: number      // 唯一类别数
}
```

### RecordsSummary (汇总统计)

```typescript
{
  total_records: number       // 总记录数
  upload_records: number      // 上传记录数
  detection_records: number   // 检测记录数
  completed: number           // 已完成数
  processing: number          // 处理中数
  failed: number              // 失败数
}
```

---

## 错误响应格式

所有错误响应遵循统一格式：

```json
{
  "message": "error description"
}
```

### HTTP 状态码

| 状态码 | 说明 |
| --- | --- |
| 200 | 请求成功 |
| 201 | 创建成功 |
| 204 | 删除成功（无响应体） |
| 400 | 请求参数错误 |
| 404 | 资源不存在 |
| 413 | 请求体过大 |
| 500 | 服务器内部错误 |
| 503 | 服务暂时不可用 |

---

## 速率限制

当前版本无速率限制。生产环境建议配置反向代理实现限流。

## 缓存策略

- **记录列表**: 缓存 5 分钟
- **记录详情**: 缓存 15 分钟
- **图片数据**: 缓存 1 小时
- **统计信息**: 缓存 5 分钟

缓存使用 Redis 实现，删除操作会自动清除相关缓存。

---

**最后更新**: 2025-10-15  
**API 版本**: v1.1.0  
**文档维护**: 果蔬智造团队  

## 版本历史

- **v1.1.0** (2025-10-15): 新增 AI 聊天接口、统一记录 API、实时检测端点
- **v1.0.0** (2025-10-14): 初始版本，包含上传、结果查询、统计接口
