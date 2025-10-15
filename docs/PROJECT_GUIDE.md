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

### 2.1 架构图

为了更直观地理解系统各组件的交互关系，我们提供了详细的系统架构图。该图展示了从用户请求到数据处理、再到结果返回的完整流程。

#### 完整系统架构（v1.1.0）

![系统架构图 v1.1.0](diagrams/output/system-architecture.png)

**主要组件**：
- **用户界面层**: Web 浏览器（Vue 3）、移动应用、实时摄像头
- **前端服务**: Nginx 托管、上传组件、结果展示、聊天组件、统计看板
- **后端 API 服务**: Axum HTTP 服务器、路由层、服务层（AI/LLM/MCP Client）、缓存层
- **AI 推理服务**: FastAPI 服务器、模型注册表、DenseNet121、质量评估
- **外部 LLM 服务**: SiliconFlow API、Anthropic Claude API
- **数据存储**: 
  - **PostgreSQL**: 持久化存储上传记录、检测结果、聊天会话和消息
  - **Redis**: 缓存检测结果、活跃会话、聊天历史

#### 核心流程图

**1. 图像上传与识别流程**

![上传流程图](diagrams/output/upload-flow.png)

展示从用户上传图片到 AI 识别完成的完整时序：
- 图像上传和验证
- AI 推理处理
- 结果持久化和缓存
- 响应返回
- 后续查询

**2. AI 聊天助手交互流程**

![聊天流程图](diagrams/output/chat-flow.png)

展示聊天功能的完整交互流程：
- 会话初始化
- 用户消息发送
- LLM 处理与工具决策
- MCP 数据库查询
- 响应生成和持久化
- 消息轮询

**3. 系统数据流图**

![数据流图](diagrams/output/data-flow.png)

展示数据在各组件间的流动和转换：
- 数据源和输入层
- 数据传输和处理层
- 数据存储和输出层
- 数据展示层

**图表源文件**: 所有架构图和流程图均使用 PlantUML 生成，源文件位于 `docs/diagrams/*.puml`。详细说明和生成方法请参阅 [docs/diagrams/README.md](diagrams/README.md)。

### 2.2 组件角色
| 模块 | 技术栈 | 关键职责 | 代码位置 |
| --- | --- | --- | --- |
| 前端 UI | Vue 3 + Element Plus + Vite | 上传任务、实时展示检测结果、历史查询与统计、AI 聊天界面 | `frontend/` |
| 接入后端 | Rust (Axum) | 处理上传、调用 AI 服务、写入数据库、提供 REST API、聊天服务、MCP 集成 | `backend/src/` |
| AI 推理服务 | FastAPI + PyTorch | 图像预处理、DenseNet121 推理、质量评估 | `ai/app/` |
| 聊天服务 | Rust (内置) | LLM 集成、MCP 协议实现、工具调用、会话管理 | `backend/src/services/` |
| 缓存层 | Redis | 缓存检测结果、聊天历史、会话数据，减少数据库压力 | 外部服务 |
| 数据库 | PostgreSQL | 存储上传、推理结果、聊天会话、用户信息 | 外部服务 |

### 2.3 服务拓扑
```
[Vue 前端] --REST--> [Axum 后端] --HTTP--> [FastAPI AI]
      |                      |                |
      |                      |                +--> 模型权重 / 质量评估
      |                      |
      |                      +--> LLM API (SiliconFlow/Claude)
      |                      |
      |                      +--> Redis 缓存 (结果 + 聊天历史)
      |                      |
      +--> 静态资源          +--> PostgreSQL (uploads/results/chat)
                             |         ↑
                             +--> MCP 查询
```

### 2.4 核心流程

#### 图像识别流程
1. 用户在上传组件 (`frontend/src/components/UploadPanel.vue`) 选择图片触发 `POST /api/v1/uploads`。
2. 后端 (`backend/src/routes/uploads.rs`) 校验 `multipart`，写入 `uploads` 表，并向 AI 服务发送推理请求。
3. FastAPI (`ai/app/api.py`) 完成预处理、DenseNet121 推理与质量分级，返回结构化 JSON。
4. 后端计算等级统计，写入 `results` 表，并将完整响应缓存到 Redis（键格式 `result:<upload_id>`）。
5. 前端通过 `GET /api/v1/records`、`GET /api/v1/records/{id}` 展示历史记录与统计。

#### AI 聊天流程
1. 用户在聊天组件 (`frontend/src/components/ChatWidget.vue`) 创建会话或发送消息。
2. 后端 (`backend/src/routes/chat.rs`) 接收消息，调用 LLM 服务 (`services/llm_client.rs`)。
3. LLM 决定是否需要工具调用（如数据库查询），通过 MCP 客户端 (`services/mcp_client.rs`) 执行。
4. MCP 客户端使用 SQLx 直接查询 PostgreSQL，返回结构化数据。
5. LLM 基于工具输出生成最终回复，后端存储消息到 `chat_messages` 表并缓存到 Redis。
6. 前端通过轮询 (`GET /api/v1/chat/messages/new`) 或长轮询获取新消息。

## 3. 模块详解

### 3.1 Rust 后端 (`backend/`)
- **入口**：`src/main.rs`，初始化配置 `AppConfig`、数据库连接池、Redis 客户端、AI Client、LLM Client 和 MCP Client。
- **配置加载**：`src/config.rs` 从环境变量读取数据库、Redis、AI 服务、LLM API 地址和密钥。
- **数据访问**：`src/db.rs` 在启动时自动建表（`uploads`、`results`、`detections`、`chat_sessions`、`chat_messages`、`users`）。
- **业务路由**：
  - `routes/uploads.rs`：处理文件上传、调用 AI、写入结果、缓存响应。
  - `routes/records.rs`：统一记录 API，支持上传和实时检测记录的统一查询。
  - `routes/results.rs`：遗留端点，向后兼容。
  - `routes/detections.rs`：实时检测记录的创建和查询。
  - `routes/statistics.rs`：聚合上传状态、质量分级、类别分布统计。
  - `routes/chat.rs`：聊天会话管理、消息发送、历史查询、新消息轮询。
  - `routes/health.rs`：返回健康检查。
- **服务层**：
  - `services/llm_client.rs`：LLM API 客户端，支持工具调用和流式响应。
  - `services/mcp_client.rs`：MCP 协议实现，直接使用 SQLx 查询数据库。
  - `ai_client.rs`：AI 推理服务 HTTP 客户端。
- **缓存层**：`src/cache.rs` 提供 Redis 缓存工具函数，支持结果缓存、会话缓存、聊天历史缓存。
- **错误与模型定义**：`src/errors.rs` 统一错误类型，`src/models.rs` 描述数据库行、API 响应与缓存结构。
- **对外 API 列表**参见 `docs/API.md`。

### 3.2 AI 推理服务 (`ai/`)
- **核心类**：`FruitDetectionService`（`app/service.py`），在启动时加载模型注册表。
- **模型管理**：
  - `app/model_registry.py`：管理多模型支持，支持动态加载。
  - `app/model_metadata.py`：模型元数据管理（版本、类别数、设备信息）。
  - `app/model_cache.py`：模型权重缓存，避免重复加载。
- **图像处理**：
  - `app/utils/preprocess.py`：图像预处理（调整尺寸、归一化）。
  - `app/utils/postprocess.py`：质量评估（颜色均匀度、纹理、面积、新鲜度）。
- **FastAPI 端点**（`app/api.py`）：
  - `GET /health`：健康检查。
  - `GET /models`：列出所有可用模型。
  - `GET /models/{model_name}`：获取特定模型信息。
  - `POST /infer`：执行推理，接受 `upload_id`、`file` 和可选的 `model_name`。
- **配置与日志**：
  - `app/config.py`：从环境变量读取配置。
  - `app/logging_config.py`：结构化日志配置。
- **更详细的算法与训练流程**见 `docs/ALGORITHM.md`。

### 3.3 前端应用 (`frontend/`)
- **技术栈**：Vue 3 + TypeScript + Vite + Element Plus + Tailwind CSS。
- **入口文件**：`src/main.ts`，路由定义 `src/router/index.ts`。
- **主要页面**：
  - `views/HomeView.vue`：项目概览与快速入口。
  - `views/UploadView.vue`：文件上传、进度跟踪、结果展示。
  - `views/ResultsView.vue`：分页列表、筛选、详情抽屉，展示统计数据与历史记录。
  - `views/DetectionsView.vue`：实时检测记录查看。
- **核心组件**：
  - `components/UploadPanel.vue`：文件选择、拖拽上传、进度条。
  - `components/ChatWidget.vue`：AI 聊天浮窗入口。
  - `components/ChatWindow.vue`：聊天界面主窗口。
  - `components/ChatBubble.vue`：聊天按钮（最小化状态）。
  - `components/ChatMessage.vue`：消息渲染（支持 Markdown）。
- **状态管理**：
  - `stores/chatStore.ts`：Pinia store，管理聊天会话、消息、UI 状态。
  - `stores/recordStore.ts`：管理记录列表和过滤状态。
- **API 封装**：
  - `src/api/upload.ts`：上传相关 API。
  - `src/api/result.ts`：结果查询 API。
  - `src/api/chat.ts`：聊天相关 API（会话、消息）。
- **构建与开发**指令见 `frontend/package.json`。

## 4. 数据与存储

### 4.1 PostgreSQL 表结构（`init-db/00-schema.sql`）
- **users**（用户表）
  - `id UUID PRIMARY KEY`
  - `username VARCHAR(50) UNIQUE`
  - `email VARCHAR(255) UNIQUE`
  - `password_hash TEXT`
  - `role VARCHAR(20)` (`admin/user/operator`)
  - `status VARCHAR(20)` (`active/inactive/suspended`)
  
- **uploads**（上传表）
  - `id UUID PRIMARY KEY`
  - `user_id UUID` → `users(id)`
  - `filename TEXT`
  - `content_type TEXT`
  - `data BYTEA` —— 直接存储原图二进制
  - `status TEXT` (`pending/processing/completed/failed`)
  - `created_at`, `updated_at`
  
- **results**（检测结果表）
  - `upload_id UUID PRIMARY KEY` → `uploads(id)`
  - `status TEXT`
  - `detections JSONB`（检测明细数组）
  - `total_fruits INTEGER`
  - `grade_a/grade_b/grade_c INTEGER`
  - `avg_confidence REAL`
  - `created_at TIMESTAMPTZ`
  
- **detections**（实时检测表）
  - `id UUID PRIMARY KEY`
  - `user_id UUID` → `users(id)`
  - `snapshot BYTEA`（快照图像）
  - `labels TEXT[]`
  - `confidence_scores REAL[]`
  - `detected_classes TEXT[]`
  - `source TEXT` (默认 `live_camera`)
  - `created_at TIMESTAMPTZ`
  
- **chat_sessions**（聊天会话表）
  - `id UUID PRIMARY KEY`
  - `user_id UUID` → `users(id)`
  - `title TEXT`
  - `status TEXT` (`active/inactive/archived`)
  - `created_at`, `updated_at`
  
- **chat_messages**（聊天消息表）
  - `id UUID PRIMARY KEY`
  - `session_id UUID` → `chat_sessions(id)`
  - `role TEXT` (`user/assistant/system`)
  - `content TEXT`
  - `metadata JSONB`（工具调用等元数据）
  - `created_at TIMESTAMPTZ`
### 4.2 Redis 缓存策略
- **检测结果缓存**：
  - 键：`result:<upload_id>`
  - 值：序列化后的 `UploadResponsePayload`
  - TTL：默认 900 秒，可通过 `CACHE_TTL_SECONDS` 覆盖
  
- **聊天会话缓存**：
  - 键：`active_session:<user_id>`
  - 值：当前活跃会话 ID
  - TTL：1 小时
  
- **聊天历史缓存**：
  - 键：`conversation:<session_id>`
  - 值：最近 20 条消息的列表
  - TTL：30 分钟
  - 操作：追加新消息时自动更新

## 5. 配置与环境变量

### 5.1 顶层 `.env`（参考 `.env.example`）
- **数据库配置**：
  - `DATABASE_URL` / `POSTGRES_*`：PostgreSQL 连接信息
- **缓存配置**：
  - `REDIS_URL`、`REDIS_PASSWORD`：Redis 连接信息
  - `CACHE_TTL_SECONDS`：缓存过期时间（默认 900 秒）
- **后端配置**：
  - `BIND_ADDRESS`：后端监听地址（默认 `0.0.0.0:8080`）
  - `AI_SERVICE_URL`：AI 服务地址（默认 `http://ai:8002`）
  - `RUST_LOG`：日志级别（`debug/info/warn/error`）
- **LLM 配置**：
  - `LLM_API_URL`：LLM API 端点（默认 SiliconFlow）
  - `LLM_API_KEY`：LLM API 密钥（**必需**）
  - `LLM_MODEL`：使用的模型名称（默认 `Qwen/Qwen3-Coder-30B-A3B-Instruct`）
  - `LLM_MAX_TOKENS`：最大令牌数（默认 12800）
- **前端配置**：
  - `VITE_API_BASE_URL`：API 基础 URL（默认 `/api/v1`）
  - `VITE_APP_NAME`：应用名称
- **MCP 配置**（内置，无需独立配置）：
  - 允许查询的表：`uploads`, `results`, `detections`, `chat_sessions`, `chat_messages`, `users`
  - 最大行数限制：1000
  - 查询超时：5 秒

### 5.2 组件特定配置
- **后端**：
  - `DATABASE_URL`、`REDIS_URL`、`AI_SERVICE_URL`
  - `LLM_API_URL`、`LLM_API_KEY`、`LLM_MODEL`
  - `CACHE_TTL_SECONDS`、`BIND_ADDRESS`、`RUST_LOG`
- **AI 服务**：
  - 默认为 CPU 推理，可通过 `CUDA_VISIBLE_DEVICES` 控制 GPU
  - 模型文件位于 `ai/model/`
  - 支持多模型动态加载
- **前端**：
  - 在 `.env` 或 `.env.local` 中设置 `VITE_API_BASE_URL` 指向后端 API
  - 构建时注入环境变量

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



## 9. 相关文档与资源
- `README.md`：快速上手与项目简介。
- `docs/API.md`：REST API 详细约定。
- `docs/ALGORITHM.md`：模型与算法说明。
- `docs/DEPLOY.md`：生产部署指南。
- `backend/README.md`：后端服务补充说明。
- `frontend/front.md`：前端目录结构速览。
