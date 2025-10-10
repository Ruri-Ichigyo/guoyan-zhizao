# API 文档

本文档描述精简后的核心接口。所有 HTTP 接口均由 Rust 后端服务提供，默认基础 URL：`http://localhost:8080/api/v1`。

## 认证

系统内置基于 JWT 的登录与会话校验机制。默认管理员账号由环境变量 `ADMIN_USERNAME` / `ADMIN_PASSWORD` 控制（参考 `.env.example`）。登录成功后返回的 `token` 需在有效期内通过 `Authorization: Bearer <token>` 头参与后续请求。

### 登录

**POST** `/auth/login`

请求体：

```json
{
  "username": "admin",
  "password": "admin123456"
}
```

成功响应：

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2024-06-01T12:30:00Z",
  "user": {
    "user_id": "2d8a8d41-1b0b-4d61-9d4c-bdd1c7d9c5c3",
    "username": "admin",
    "role": "admin"
  }
}
```

### 获取当前用户信息

**GET** `/auth/me`

头部：`Authorization: Bearer <token>`

```json
{
  "user_id": "2d8a8d41-1b0b-4d61-9d4c-bdd1c7d9c5c3",
  "username": "admin",
  "role": "admin"
}
```

## 通用说明

- Accept / Content-Type：`application/json`
- 上传图片使用 `multipart/form-data`
- 响应示例中的时间为 ISO 8601 字符串
- 除健康检查外的业务接口均需携带 `Authorization: Bearer <token>` 请求头

## 健康检查

**GET** `/health`

用于探测服务存活状态。

```json
{
  "status": "ok"
}
```

## 上传图像并触发分析

**POST** `/uploads`

头部：`Authorization: Bearer <token>`

表单字段：
- `file`：图像文件

成功响应返回上传结果及 AI 推理信息。

```json
{
  "upload_id": "0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36",
  "filename": "fruit.jpg",
  "status": "completed",
  "created_at": "2024-06-01T12:00:00Z",
  "total_fruits": 3,
  "grade_a": 2,
  "grade_b": 1,
  "grade_c": 0,
  "avg_confidence": 0.91,
  "detections": [
    {
      "class_name": "apple",
      "confidence": 0.95,
      "bbox": [0, 0, 640, 480],
      "quality_grade": "A"
    }
  ]
}
```

## 查询历史记录

### 列表查询

**GET** `/results`

头部：`Authorization: Bearer <token>`

查询参数：
- `page` (可选，默认 `1`)
- `limit` (可选，默认 `10`，最大 `100`)
- `status` (可选，`processing` / `completed` / `failed`)

```json
{
  "data": [
    {
      "upload_id": "0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36",
      "filename": "fruit.jpg",
      "status": "completed",
      "created_at": "2024-06-01T12:00:00Z",
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
    "total": 1,
    "pages": 1
  }
}
```

### 获取单条结果

**GET** `/results/{upload_id}`

头部：`Authorization: Bearer <token>`

```json
{
  "upload_id": "0ac8c6c7-31d4-4b44-a683-6e1a5d7d1c36",
  "filename": "fruit.jpg",
  "status": "completed",
  "created_at": "2024-06-01T12:00:00Z",
  "total_fruits": 3,
  "grade_a": 2,
  "grade_b": 1,
  "grade_c": 0,
  "avg_confidence": 0.91,
  "detections": [
    {
      "class_name": "apple",
      "confidence": 0.95,
      "bbox": [0, 0, 640, 480],
      "quality_grade": "A"
    }
  ]
}
```

## 统计信息

**GET** `/statistics`

头部：`Authorization: Bearer <token>`

返回当前系统的聚合数据。

```json
{
  "totals": {
    "total": 42,
    "completed": 39,
    "processing": 2,
    "failed": 1,
    "total_fruits": 120
  },
  "averages": {
    "avg_confidence": 0.89
  }
}
```

## 错误示例

```json
{
  "message": "resource not found"
}
```

- 上传校验错误返回 `400`
- 未找到记录返回 `404`
- 内部错误返回 `500`
