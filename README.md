# 果蔬智造核心平台

轻量化保留核心功能的果蔬识别系统。平台聚焦于“图像上传 → AI 分析 → 结果查询”流程，采用 Rust 后端 + FastAPI AI 服务，Redis 作为热点缓存，PostgreSQL 负责持久化，前端使用 Vue 3 提供操作界面。

## 架构速览

| 组件 | 技术栈 | 说明 |
| --- | --- | --- |
| 前端 | Vue 3 + Element Plus | 图像上传、结果列表、检测详情 |
| 后端 | Rust (Axum) | 统一接入层，存储上传文件、触发 AI 推理、暴露 REST API |
| AI 服务 | Python (FastAPI + PyTorch) | 处理图像分类并返回检测结果 |
| 缓存 | Redis | 缓存检测结果，提升查询性能 |
| 数据库 | PostgreSQL | 存储上传记录、检测结果统计 |

## 快速上手

### 1. 环境准备

- Docker Desktop / Docker Engine 20.10+
- Docker Compose v2

```bash
# 如果没有 .env 文件，从示例创建
cp .env.example .env

# 使用启动脚本一键部署
./scripts/start.sh
```

脚本会完成镜像构建并启动以下服务：

- 前端：http://localhost:3000
- 后端 API：http://localhost:8080/api/v1
- AI 服务健康检查：http://localhost:8002/health
- PostgreSQL：`postgres://gyz:gyz123@localhost:5432/gyz`
- Redis：`redis://localhost:6379`

查看运行状态：

```bash
docker compose ps
docker compose logs -f backend
```

停止服务：

```bash
docker compose down
```

### 2. 本地开发

使用 `./scripts/dev-start.sh` 可快速检查依赖并按需启动基础设施。主要服务开发流程：

#### 后端 (Rust Axum)
```bash
cd backend
export DATABASE_URL=postgres://gyz:gyz123@localhost:5432/gyz
export REDIS_URL=redis://localhost:6379
export AI_SERVICE_URL=http://localhost:8002
# 可选：覆盖默认的 JWT 配置
export JWT_SECRET=local-dev-secret
export JWT_TTL_SECONDS=3600
cargo run
```

#### AI 推理服务 (FastAPI)
```bash
cd ai
python -m venv .venv && source .venv/bin/activate  # 可选
pip install -r requirements.txt
uvicorn main:app --reload --host 0.0.0.0 --port 8002
```

#### 前端 (Vue 3)
```bash
cd frontend
npm install
npm run dev
# 开发代理默认指向 http://localhost:8080
```

### 3. 测试部署

部署完成后，可以测试各个服务：

```bash
# 检查所有服务状态
docker compose ps

# 测试健康检查端点
curl http://localhost:8002/health  # AI 服务
curl http://localhost:8080/health  # 后端服务
curl http://localhost:3000/health  # 前端服务

# 测试上传图片并识别
curl -X POST http://localhost:8080/api/v1/uploads \
  -F "file=@path/to/your/fruit-image.jpg"

# 查看统计信息
curl http://localhost:8080/api/v1/statistics

# 查看结果列表
curl http://localhost:8080/api/v1/results
```

### 4. 认证与默认账号

后端启用了基于 JWT 的认证流程。服务启动时会在数据库中确保存在一个默认管理员账号，便于首次登录：

- 默认用户名：`admin`
- 默认密码：`admin123456`

可通过以下环境变量覆盖默认值：

- `ADMIN_USERNAME`
- `ADMIN_PASSWORD`
- `ADMIN_ROLE`（默认为 `admin`）
- `JWT_SECRET`（默认 `development-secret-change-me`，生产环境务必修改）
- `JWT_TTL_SECONDS`（默认 3600 秒）

**获取访问令牌示例**

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123456"}'
```

登录成功后，响应会包含 `token` 和 `expires_at` 字段，后续请求需在 `Authorization` 头中携带 `Bearer <token>`。前端应用会在登录页中请求该接口并在本地保存会话信息。

## API 概要

**后端 API (http://localhost:8080)**

- `POST /api/v1/uploads`：上传图片并返回检测结果（主要功能）
- `GET /api/v1/results`：分页查询历史记录
- `GET /api/v1/results/{id}`：获取单条检测详情
- `GET /api/v1/statistics`：统计上传与识别概况
- `GET /health`：后端服务健康检查

**AI 服务 API (http://localhost:8002)**

- `POST /infer`：图像推理接口（由后端调用）
- `GET /health`：AI 服务健康检查
- `GET /models`：列出所有已注册模型
- `GET /metrics`：获取服务指标

详细说明见 [docs/API.md](docs/API.md)。

## 项目结构

```
.
├── ai/            # FastAPI + PyTorch 推理服务
├── backend/       # Rust 后端，负责数据存储与业务编排
├── frontend/      # Vue 3 前端界面
├── docs/          # 项目文档与部署、算法说明
├── scripts/       # 辅助脚本（启动、环境验证等）
├── docker-compose.yml
└── .env.example   # 环境变量示例
```

## 文档索引
- [快速启动](docs/QUICKSTART.md)：部署与开发模式操作步骤
- [项目指南](docs/PROJECT_GUIDE.md)：架构、模块与配置说明
- [部署手册](docs/DEPLOY.md)：生产发布与运维建议
- [API 参考](docs/API.md)：REST 接口定义与示例
- [算法说明](docs/ALGORITHM.md)：模型训练与推理细节
- [搭建核对](docs/SETUP_REPORT.md)：仓库结构与必备文件检查
