# 果蔬智造核心平台项目文档

## 1. 项目概览

### 1.1 背景与目标
- 面向果蔬分拣场景，提供从图像上传、AI 推理到结果查询的端到端解决方案。
- 强调轻量部署与易维护，核心链路仅依赖前端、Rust 后端、FastAPI 推理服务和基础数据服务。
- 保持接口与模块的可扩展性，便于替换模型、接入对象存储或引入消息队列等能力。

### 1.2 核心能力
- 多端上传果蔬图像，后端持久化原始数据并触发即时推理。
- DenseNet121 分类模型结合质量评分模块，返回等级统计与置信度。
- Redis 热点缓存 + PostgreSQL 持久化，支持查询、统计与回溯。
- Vue 3 前端提供上传、实时反馈、历史记录与统计看板。

## 2. 系统架构

### 2.1 组件角色
| 模块 | 技术栈 | 关键职责 | 代码位置 |
| --- | --- | --- | --- |
| 前端 UI | Vue 3 + Element Plus + Vite | 上传任务、实时展示检测结果、历史查询与统计 | `frontend/` |
| 接入后端 | Rust (Axum) | 校验与落盘上传文件、调用 AI 服务、写入数据库、提供 REST API | `backend/src/` |
| AI 推理服务 | FastAPI + PyTorch | 图像预处理、DenseNet121 推理、质量评估 | `ai/main.py` |
| 缓存层 | Redis | 缓存单条检测结果 JSON，减少数据库压力 | 外部服务 |
| 数据库 | PostgreSQL | 存储上传 binary、推理结果与统计字段 | 外部服务 |

### 2.2 服务拓扑
```
[Vue 前端] --REST--> [Axum 后端] --HTTP--> [FastAPI AI]
      |                      |                |
      |                      |                +--> 模型权重 / 质量评估
      |                      +--> Redis 缓存 ←+
      +--> 静态资源         |
                             +--> PostgreSQL (uploads/results)
```

### 2.3 核心流程
1. 用户在前端 (`frontend/src/components/UploadPanel.vue`) 选择图片触发 `POST /api/v1/uploads`。
2. 后端 (`backend/src/routes/uploads.rs`) 校验 `multipart`，写入 `uploads` 表，并向 AI 服务发送推理请求。
3. FastAPI (`ai/main.py`) 完成预处理、DenseNet121 推理与质量分级，返回结构化 JSON。
4. 后端计算等级统计，写入 `results` 表，并将完整响应缓存到 Redis（键格式 `result:<upload_id>`）。
5. 前端通过 `GET /api/v1/results`、`GET /api/v1/results/{id}`、`GET /api/v1/statistics` 展示历史记录与统计。

## 3. 模块详解

### 3.1 Rust 后端 (`backend/`)
- 入口：`src/main.rs`，初始化配置 `AppConfig`、数据库连接池、Redis 客户端与 AI Client。
- 配置加载：`src/config.rs` 从环境变量读取数据库、Redis、AI 服务地址、缓存 TTL。
- 数据访问：`src/db.rs` 在启动时自动建表，并提供失败状态回写逻辑。
- 业务路由：
  - `routes/uploads.rs`：处理文件上传、调用 AI、写入结果、缓存响应。
  - `routes/results.rs`：支持分页过滤、详情查询（优先读取 Redis）。
  - `routes/statistics.rs`：聚合上传状态与平均置信度。
  - `routes/health.rs`：返回健康检查。
- 错误与模型定义：`src/errors.rs` 统一错误类型，`src/models.rs` 描述数据库行、API 响应与缓存结构。
- 对外 API 列表参见 `docs/API.md`。

### 3.2 AI 推理服务 (`ai/`)
- 核心类 `FruitDetectionService`（`main.py`），在启动时加载 `model/labels.json` 与 `fruit_densenet_weights.pth`（若缺失则退回 ImageNet 预训练头）。
- 图像预处理：`utils/preprocess.py`（调整尺寸、归一化），推理后使用 `torch.topk` 选取 Top-K 结果。
- 质量评估：`utils/postprocess.py` 根据颜色均匀度、纹理、面积及置信度综合打分，转换为 `A/B/C`。
- FastAPI 端点：
  - `GET /health`：健康检查。
  - `POST /infer`：接受 `upload_id` + 文件字节，返回推理结果（`InferenceResponse`）。
- 更详细的算法与训练流程见 `docs/ALGORITHM.md`，训练脚本位于 `ai/train.py`。

### 3.3 前端应用 (`frontend/`)
- 技术栈：Vue 3 + TypeScript + Vite + Element Plus + Tailwind。
- 入口文件 `src/main.ts`，路由定义 `src/router/index.ts`，`src/stores/` 目录预留给全局状态管理。
- 主要页面：
  - `HomeView.vue`：项目概览与快速入口。
  - `UploadPanel.vue`（嵌入上传页）：完成文件选择、上传反馈、最近结果展示。
  - `ResultsView.vue`：分页列表、筛选、详情抽屉，展示统计数据与历史记录。
- API 封装：`src/api/upload.ts`、`src/api/result.ts`，通过 `VITE_API_BASE_URL` 与后端交互。
- 构建与开发指令见 `frontend/package.json`。

## 4. 数据与存储

### 4.1 PostgreSQL 表结构（`backend/src/db.rs`）
- `uploads`
  - `id UUID PRIMARY KEY`
  - `filename TEXT`
  - `content_type TEXT`
  - `data BYTEA` —— 直接存储原图二进制
  - `status TEXT` (`processing/completed/failed`)
  - `created_at`, `updated_at`（默认 `NOW()`）
- `results`
  - `upload_id UUID PRIMARY KEY`（外键关联 `uploads.id`）
  - `status TEXT`
  - `detections JSONB`（缓存检测明细）
  - `total_fruits INTEGER`
  - `grade_a/grade_b/grade_c INTEGER`
  - `avg_confidence REAL`
  - `created_at TIMESTAMPTZ`

> 若未来要迁移到对象存储，可将二进制改为外部链接，并保留元数据列。

### 4.2 Redis 缓存策略
- 键：`result:<upload_id>`。
- 值：序列化后的 `UploadResponsePayload`（见 `backend/src/models.rs`）。
- TTL：默认 900 秒，可通过 `CACHE_TTL_SECONDS` 覆盖。

## 5. 配置与环境变量

### 5.1 顶层 `.env`（参考 `.env.example`）
- `DATABASE_URL` / `POSTGRES_*`：PostgreSQL 连接信息。
- `REDIS_URL`、`CACHE_TTL_SECONDS`：缓存配置。
- `BIND_ADDRESS`、`AI_SERVICE_URL`、`RUST_LOG`：后端服务配置。
- `VITE_API_BASE_URL`、`VITE_APP_NAME` 等：前端构建时读取。
- `DEV_*` 变量为 `scripts/dev-start.sh` 输出的本地开发建议值。

### 5.2 组件特定配置
- 后端：
  - `DATABASE_URL`、`REDIS_URL`、`AI_SERVICE_URL`、`CACHE_TTL_SECONDS`、`BIND_ADDRESS`。
- AI 服务：
  - 默认为 CPU 推理，可通过 `CUDA_VISIBLE_DEVICES` 控制 GPU；模型文件位于 `ai/model/`。
- 前端：
  - 在 `.env` 或 `.env.local` 中设置 `VITE_API_BASE_URL` 指向后端 API。

## 6. 启动与开发流程

### 6.1 Docker 部署
- 执行 `./scripts/start.sh` 会逐个构建并启动 `postgres`、`redis`、`ai`、`backend`、`frontend` 服务，并等待健康检查通过。
- 部署细节、资源建议与反向代理配置见 `docs/DEPLOY.md`。

### 6.2 本地开发
- 运行辅助脚本 `./scripts/dev-start.sh` 检查依赖并可选启动 Docker 基础设施。
- 后端：
  ```bash
  cd backend
  export DATABASE_URL=postgres://gyz:gyz123@localhost:5432/gyz
  export REDIS_URL=redis://localhost:6379
  export AI_SERVICE_URL=http://localhost:8002
  cargo run
  ```
- AI 服务：
  ```bash
  cd ai
  python -m venv .venv && source .venv/bin/activate  # 可选
  pip install -r requirements.txt
  uvicorn main:app --reload --host 0.0.0.0 --port 8002
  ```
- 前端：
  ```bash
  cd frontend
  npm install
  npm run dev
  ```

### 6.3 数据与脚本
- 开发阶段默认使用本地 PostgreSQL/Redis，首次启动后端即会建表。
- `scripts/verify-setup.sh`、`scripts/start.sh`、`scripts/dev-start.sh` 可在根目录查看具体逻辑。

## 7. 测试与质量保障

- Rust 后端：`cargo test`（核心逻辑位于 `backend`，可扩展针对 `routes` 的集成测试）。
- Python AI 服务：`pytest`（`tests/unit/test_ai_service.py` 等）验证预处理、质量评估、异常处理。
- 前端：`npm run test` / `npx vitest`（`tests/unit/test_frontend.spec.ts` 覆盖 `UploadPanel` 与 `ResultsView` 的核心流程）。
- 端到端与负载：
  - `tests/integration/upload_service_test.rs`：示例性路由测试，使用内建路由模拟接口返回。
  - `tests/e2e/cypress/`：简单的导航与数据渲染校验，可按需扩展更完整的浏览器场景。
  - `tests/load/locustfile.py`：针对上传、列表与统计的基础压测脚本，可作为性能基线。
- 日常建议：在合入前至少执行后端与 AI 服务的单元测试，并确认前端构建通过；若改动接口，需同步更新 `docs/API.md`。

## 8. 日志与监控
- 后端使用 `tracing` 输出结构化日志，级别由 `RUST_LOG` 控制，可接入 Loki / ELK。
- AI 服务使用 Python logging，默认 INFO，可通过环境变量提升级别。
- 数据层建议开启 PostgreSQL `pg_stat_statements` 与 Redis `INFO` 监控。
- 可扩展 Prometheus 导出器（如 `redis_exporter`、`postgres_exporter`）与自定义指标（推理耗时、缓存命中率）。

## 9. 扩展路线
1. 将上传文件存储迁移至对象存储（MinIO/S3），数据库保留元数据与引用。
2. 引入消息队列（Kafka/RabbitMQ）异步解耦上传与推理，提升吞吐。
3. 增强前端权限控制与多租户支持。
4. 丰富测试矩阵：完善 Locust/Cypress 场景，覆盖失败路径与性能基线。
5. 增加模型版本管理，支持多模型或 A/B 测试。

## 10. 相关文档与资源
- `README.md`：快速上手与项目简介。
- `docs/API.md`：REST API 详细约定。
- `docs/ALGORITHM.md`：模型与算法说明。
- `docs/DEPLOY.md`：生产部署指南。
- `backend/README.md`：后端服务补充说明。
- `frontend/front.md`：前端目录结构速览。

> 建议在提交代码调整核心流程时，同时更新本文件与相关子文档，保持架构视图与实现一致。
