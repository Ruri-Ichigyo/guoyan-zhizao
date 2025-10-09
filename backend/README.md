# 后端服务概述

Rust 编写的单体服务，负责：

1. 接收前端上传的图像，保存原始二进制数据。
2. 调用 AI 推理服务获取分类结果，写入 PostgreSQL。
3. 将检测结果缓存到 Redis 以加速重复查询。
4. 提供 RESTful API 给前端使用。

## 技术栈

- Axum + Tokio：异步 HTTP 服务器
- SQLx：与 PostgreSQL 交互
- Redis：热点结果缓存
- Reqwest：调用 AI FastAPI 服务
- Tracing：结构化日志

## 目录结构

```
src/
├── ai_client.rs   # 调用 AI 推理服务
├── cache.rs       # Redis 客户端封装
├── config.rs      # 环境变量加载
├── db.rs          # 数据库连接与表初始化
├── errors.rs      # 统一错误处理
├── main.rs        # 程序入口与路由
├── models.rs      # 数据模型与序列化
└── routes/        # 上传、查询、统计、健康检查接口
```

## 环境变量

- `DATABASE_URL`：PostgreSQL 连接串
- `REDIS_URL`：Redis 连接串
- `AI_SERVICE_URL`：AI 服务地址，例如 `http://localhost:8002`
- `BIND_ADDRESS`：服务监听地址，默认 `0.0.0.0:8080`
- `CACHE_TTL_SECONDS`：缓存有效期（秒），默认 900

## 运行

```bash
export DATABASE_URL=postgres://gyz:gyz123@localhost:5432/gyz
export REDIS_URL=redis://localhost:6379
export AI_SERVICE_URL=http://localhost:8002
cargo run
```

## 数据表

启动时若不存在会自动创建：
- `uploads`：保存原始文件、状态、时间戳
- `results`：保存 AI 检测输出、统计信息

## API

- `POST /api/v1/uploads`
- `GET /api/v1/results`
- `GET /api/v1/results/{id}`
- `GET /api/v1/statistics`
- `GET /health`

详细说明见根目录 [docs/API.md](../docs/API.md)。
