# API 文档

## 概览

果蔬智能分拣系统采用微服务架构，提供完整的 RESTful API 和 WebSocket 接口，支持文件上传、AI 推理、结果查询和实时通信。

### 服务架构

- **上传服务 (Upload Service)**: 负责文件上传和预签名 URL 生成
- **查询服务 (Query Service)**: 处理检测结果查询和 WebSocket 实时推送
- **分拣服务 (Sort Service)**: 控制分拣设备和管理分拣任务
- **API 网关 (Kong)**: 统一入口和路由代理

> **注意**: 目前部分功能仍在开发中，如统计数据、数据导出、分片上传等。已实现的功能包括基本的文件上传、结果查询和分拣控制。

## 基础信息

果蔬智能分拣系统采用微服务架构，各服务运行在不同端口：

- **上传服务**: `http://localhost:3001` (文件上传和预签名 URL)
- **查询服务**: `http://localhost:3002` (结果查询和 WebSocket)
- **分拣服务**: `http://localhost:3003` (分拣控制和状态)
- **网关代理**: `http://localhost:8000` (Kong API 网关，统一入口)
- **WebSocket URL**: `ws://localhost:3002/ws/live`
- **认证方式**: JWT Token
- **Content-Type**: `application/json` (除文件上传外)

## 认证

所有 API 请求都需要在 Header 中包含 JWT Token：

```http
Authorization: Bearer <your-jwt-token>
```

### 获取 Token

```bash
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "your-username", "password": "your-password"}'
```

## 文件上传服务 (Upload Service)

### 1. 直接上传文件

**POST** `/api/upload`

上传单个或多个图片文件进行 AI 分析。

```bash
curl -X POST http://localhost:3001/api/upload \
  -H "Authorization: Bearer <token>" \
  -F "file=@/path/to/fruit.jpg"
```

**响应:**

```json
{
  "upload_id": "upload_12345",
  "status": "uploaded",
  "presigned_url": null
}
```

### 2. 获取预签名 URL

**POST** `/api/presign`

获取 MinIO 预签名 URL，用于直接上传到对象存储。

```bash
curl -X POST http://localhost:3001/api/presign \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "filename": "fruit.jpg",
    "content_type": "image/jpeg"
  }'
```

**响应:**

```json
{
  "upload_id": "upload_12345",
  "presigned_url": "https://minio:9000/fruit-images/uploads/12345/fruit.jpg?X-Amz-Algorithm=...",
  "status": "presigned"
}
```

````

## 查询服务 (Query Service)

### 1. 获取分析结果

**GET** `/api/result/{job_id}`

获取特定任务的分析结果。

```bash
curl -X GET http://localhost:3002/api/result/upload_12345 \
  -H "Authorization: Bearer <token>"
````

**响应:**

```json
{
  "job_id": "upload_12345",
  "status": "completed",
  "results": [
    {
      "class_name": "apple",
      "confidence": 0.95,
      "bbox": [10.0, 20.0, 100.0, 150.0],
      "quality_grade": "A"
    },
    {
      "class_name": "orange",
      "confidence": 0.89,
      "bbox": [200.0, 50.0, 300.0, 200.0],
      "quality_grade": "B"
    }
  ],
  "created_at": "2023-12-01T10:30:00Z"
}
```

### 2. 获取结果列表

**GET** `/api/results`

获取分页的结果列表。

**查询参数:**

- `page`: 页码 (默认: 1)
- `limit`: 每页数量 (默认: 10)
- `status`: 过滤状态 (`pending`, `processing`, `completed`, `failed`)
- `from_date`: 开始日期 (ISO 8601)
- `to_date`: 结束日期 (ISO 8601)

```bash
curl -X GET "http://localhost:3002/api/results?page=1&limit=20&status=completed" \
  -H "Authorization: Bearer <token>"
```

**响应:**

```json
{
  "data": [
    {
      "job_id": "test-job-1",
      "status": "completed",
      "total_fruits": 15,
      "grade_a": 8,
      "grade_b": 5,
      "grade_c": 2,
      "created_at": "2023-12-01T10:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 1,
    "pages": 1
  }
}
```

## 分拣控制服务 (Sort Service)

### 1. 发送分拣命令

**POST** `/api/sort/command`

控制分拣设备。

```bash
curl -X POST http://localhost:3003/api/sort/command \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "job_id": "sort_12345",
    "action": "start",
    "target_grade": "A",
    "conveyor_speed": 75
  }'
```

**可用命令:**

- `start`: 开始分拣
- `stop`: 停止分拣
- `pause`: 暂停分拣
- `resume`: 恢复分拣
- `emergency_stop`: 紧急停止

**响应:**

```json
{
  "success": true,
  "message": "Sorting started successfully",
  "status": {
    "job_id": "sort_12345",
    "status": "running",
    "conveyor_speed": 75,
    "sorted_count": 0,
    "error_count": 0,
    "uptime": 0
  }
}
```

### 2. 获取分拣状态

**GET** `/api/sort/status`

获取当前分拣设备状态。

```bash
curl -X GET http://localhost:3003/api/sort/status \
  -H "Authorization: Bearer <token>"
```

**响应:**

```json
{
  "job_id": "current-job",
  "status": "idle",
  "conveyor_speed": 0,
  "sorted_count": 0,
  "error_count": 0,
  "uptime": 0
}
```

### 3. 更新分拣配置

**PUT** `/api/sort/config`

更新分拣设备配置。

```bash
curl -X PUT http://localhost:3003/api/sort/config \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "confidence_threshold": 0.8,
    "quality_standards": {
      "A": {"min_confidence": 0.9, "max_defects": 0},
      "B": {"min_confidence": 0.7, "max_defects": 2},
      "C": {"min_confidence": 0.5, "max_defects": 5}
    },
    "sorting_mode": "quality_first",
    "max_speed": 100
  }'
```

## WebSocket 接口

### 实时数据流

连接到 WebSocket 获取实时更新。

```javascript
const ws = new WebSocket("ws://localhost:3002/ws/live");

ws.onmessage = function (event) {
  const data = JSON.parse(event.data);
  console.log("Real-time update:", data);
};

// 发送心跳
setInterval(() => {
  ws.send("ping");
}, 30000);
```

### WebSocket 消息类型

#### 1. 连接确认

```json
{
  "type": "connected",
  "message": "Connected to live detection feed",
  "timestamp": "2023-12-01T10:30:00Z"
}
```

#### 2. 检测结果更新

```json
{
  "type": "detection_result",
  "data": {
    "job_id": "upload_12345",
    "status": "completed",
    "results": [...],
    "timestamp": "2023-12-01T10:30:00Z"
  }
}
```

#### 3. 分拣状态更新

```json
{
  "type": "sort_status",
  "data": {
    "status": "running",
    "conveyor_speed": 75,
    "sorted_count": 324,
    "timestamp": "2023-12-01T10:30:00Z"
  }
}
```

#### 4. 系统警报

```json
{
  "type": "alert",
  "level": "warning",
  "message": "传送带速度异常",
  "timestamp": "2023-12-01T10:30:00Z"
}
```

#### 5. 心跳响应

```json
{
  "type": "pong",
  "timestamp": "2023-12-01T10:30:00Z"
}
```

## 错误处理

### HTTP 状态码

- `200 OK`: 请求成功
- `201 Created`: 资源创建成功
- `400 Bad Request`: 请求参数错误
- `401 Unauthorized`: 认证失败
- `403 Forbidden`: 权限不足
- `404 Not Found`: 资源不存在
- `429 Too Many Requests`: 请求频率超限
- `500 Internal Server Error`: 服务器内部错误

### 错误响应格式

```json
{
  "error": {
    "code": "INVALID_FILE_FORMAT",
    "message": "不支持的文件格式",
    "details": "只支持 jpg, png, gif 格式的图片文件",
    "timestamp": "2023-12-01T10:30:00Z"
  }
}
```

### 常见错误码

- `INVALID_TOKEN`: JWT Token 无效或过期
- `INVALID_FILE_FORMAT`: 不支持的文件格式
- `FILE_SIZE_EXCEEDED`: 文件大小超过限制
- `PROCESSING_FAILED`: AI 推理处理失败
- `DEVICE_OFFLINE`: 分拣设备离线
- `RATE_LIMIT_EXCEEDED`: 请求频率超限

## SDK 和工具

### Python SDK

```python
from guoyan_client import FruitSortClient

client = FruitSortClient(
    upload_url='http://localhost:3001',
    query_url='http://localhost:3002',
    sort_url='http://localhost:3003',
    token='your-jwt-token'
)

# 上传文件
result = client.upload_file('path/to/fruit.jpg')
print(f"Upload ID: {result['upload_id']}")

# 获取结果
analysis = client.get_result(result['upload_id'])
print(f"Detected fruits: {len(analysis['results'])}")
```

### JavaScript SDK

```javascript
import { FruitSortClient } from "@guoyan/client";

const client = new FruitSortClient({
  uploadURL: "http://localhost:3001",
  queryURL: "http://localhost:3002",
  sortURL: "http://localhost:3003",
  token: "your-jwt-token",
});

// 上传文件
const result = await client.uploadFile(fileInput.files[0]);
console.log(`Upload ID: ${result.upload_id}`);

// WebSocket 连接
const ws = client.connectWebSocket("ws://localhost:3002/ws/live");
ws.on("detection_result", (data) => {
  console.log("New result:", data);
});
```

## 性能和限制

### API 限制

- **上传文件大小**: 最大 50MB
- **请求频率**: 100 请求/分钟 (可配置)
- **并发连接**: WebSocket 最大 1000 个连接
- **批量处理**: 单次最多 10 个文件

### 性能指标

- **平均响应时间**: < 200ms (不含 AI 推理)
- **AI 推理时间**: 1-3 秒 (取决于图片大小和复杂度)
- **吞吐量**: 500 张图片/小时 (单 GPU)

## 监控和日志

### 健康检查

各服务都提供健康检查端点：

```bash
# 上传服务
curl -X GET http://localhost:3001/health

# 查询服务
curl -X GET http://localhost:3002/health

# 分拣服务
curl -X GET http://localhost:3003/health

# 通过网关访问
curl -X GET http://localhost:8000/health
```

**响应:**

```json
{
  "status": "ok"
}
```

### 指标端点

目前各服务为独立部署，指标收集需要通过单独配置实现。可以通过 Prometheus 等工具收集各服务的运行指标。

### 日志格式

```json
{
  "timestamp": "2023-12-01T10:30:00Z",
  "level": "INFO",
  "service": "upload-svc",
  "message": "File uploaded successfully",
  "metadata": {
    "upload_id": "upload_12345",
    "file_size": 1048576,
    "processing_time": 2.3
  }
}
```
