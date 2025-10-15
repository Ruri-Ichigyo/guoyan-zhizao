# 果蔬智造核心平台

轻量化保留核心功能的果蔬识别与质量分级系统。平台聚焦于"图像上传 → AI 分析 → 质量分级 → 结果查询"流程，采用 Rust 后端 + FastAPI AI 服务，Redis 作为热点缓存，PostgreSQL 负责持久化，前端使用 Vue 3 + Element Plus 提供现代化操作界面。

## ✨ 核心功能

- 🍎 **智能识别**: 基于 DenseNet121 的果蔬分类，支持 60+ 品类
- 📊 **质量分级**: A/B/C 三级质量评估系统（颜色、表面、新鲜度分析）
- 📸 **多源接入**: 支持图像上传、实时摄像头、移动应用等多种数据来源
- 📈 **统计分析**: 实时统计、数据可视化、历史记录查询
- 🔄 **实时刷新**: 自动数据同步，30秒轮询 + 标签页切换刷新
- 🗂️ **统一记录**: 融合上传记录与实时检测，提供统一查询接口
- 🤖 **AI 助手**: 智能聊天对话，支持自然语言查询数据库和识别统计
  - 会话管理：创建、列表、删除聊天会话
  - 实时对话：与 LLM 交互，支持工具调用
  - 数据库查询：通过 MCP 协议直接查询 PostgreSQL
  - 智能分析：基于历史数据提供统计和见解

## 🏗️ 架构速览

| 组件 | 技术栈 | 端口 | 说明 |
| --- | --- | --- | --- |
| 前端 | Vue 3 + Element Plus + TypeScript | 3000 | 响应式界面、数据可视化、操作控制台、AI 聊天助手 |
| 后端 | Rust (Axum + SQLx) | 8080 | RESTful API、业务编排、数据持久化、聊天服务、MCP 集成 |
| AI 服务 | Python (FastAPI + PyTorch) | 8002 | 图像分类、质量评估、模型推理 |
| 缓存 | Redis 7 | 6379 | 结果缓存、会话管理、聊天历史、性能优化 |
| 数据库 | PostgreSQL 15 | 5432 | 数据持久化、关系管理、事务支持 |

**注意**: MCP (Model Context Protocol) 服务已集成到后端，通过 SQLx 直接访问数据库，无需独立部署。

## 🚀 快速上手

### 1. 环境准备

**系统要求**:
- Docker Desktop / Docker Engine 20.10+
- Docker Compose v2+
- 8GB+ 可用内存
- 10GB+ 磁盘空间

**一键启动**:

```bash
# 克隆项目
git clone <repository-url>
cd guoyan-zhizao

# 配置环境变量（可选，使用默认配置即可快速启动）
cp .env.example .env

# 启动所有服务
./scripts/start.sh

# 或使用 Docker Compose
docker compose up -d
```

### 2. 服务访问

启动完成后，可访问以下服务：

| 服务 | 地址 | 说明 |
| --- | --- | --- |
| 🌐 **前端界面** | http://localhost:3000 | Vue 3 应用主界面 |
| 🔌 **后端 API** | http://localhost:8080/api/v1 | RESTful API 接口 |
| 🤖 **AI 服务** | http://localhost:8002 | 推理服务（内部调用） |
| 💾 **PostgreSQL** | localhost:5432 | 数据库（gyz/nU1/zzGa） |
| 🗃️ **Redis** | localhost:6379 | 缓存服务（密码: 6RQgSuJH） |

### 3. 快速验证

```bash
# 检查所有服务状态
docker compose ps

# 健康检查
curl http://localhost:8080/health     # 后端
curl http://localhost:8002/health     # AI 服务
curl http://localhost:3000/health     # 前端

# 查看统计信息（包含测试数据）
curl http://localhost:8080/api/v1/statistics

# 查看记录列表
curl http://localhost:8080/api/v1/records?page=1&limit=10

# 测试图片上传
curl -X POST http://localhost:8080/api/v1/uploads \
  -F "file=@path/to/fruit.jpg"
```

### 4. 测试数据

系统启动时自动加载测试数据：
- **50+ 上传记录**: 包含各种状态（completed/processing/failed）
- **30+ 实时检测**: 来自摄像头和移动应用的记录
- **60+ 水果品类**: Apple、Banana、Orange、Grape 等
- **完整统计**: 质量分级、置信度、时间分布

### 5. 本地开发模式

#### 后端开发 (Rust + Axum)

```bash
cd backend

# 设置环境变量
export DATABASE_URL="postgres://gyz:nU1%2FzzGa@localhost:5432/gyz"
export REDIS_URL="redis://:6RQgSuJH@localhost:6379"
export AI_SERVICE_URL="http://localhost:8002"
export BIND_ADDRESS="0.0.0.0:8080"
export RUST_LOG=info

# 开发运行
cargo run

# 构建发布版本
cargo build --release

# 运行测试
cargo test
```

#### AI 服务开发 (Python + FastAPI)

```bash
cd ai

# 使用 uv (推荐)
uv sync
uv run uvicorn main:app --reload --host 0.0.0.0 --port 8002

# 或使用传统 pip
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
uvicorn main:app --reload --host 0.0.0.0 --port 8002

# 运行测试
uv run pytest
```

#### 前端开发 (Vue 3 + TypeScript)

```bash
cd frontend

# 安装依赖
npm install

# 开发服务器（自动代理到 http://localhost:8080）
npm run dev

# 类型检查
npm run type-check

# 构建生产版本
npm run build

# 预览构建结果
npm run preview
```

## 📡 API 概要

### 后端 API (http://localhost:8080/api/v1)

#### 核心接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `POST` | `/uploads` | 上传图片并触发 AI 分析 |
| `GET` | `/records` | 统一记录查询（分页、筛选） |
| `GET` | `/records/:id` | 获取单条记录详情 |
| `GET` | `/records/:id/image` | 获取记录图片/快照 |
| `DELETE` | `/records/:id` | 删除指定记录 |
| `GET` | `/statistics` | 获取统计概览 |
| `POST` | `/chat/sessions` | 创建聊天会话 |
| `POST` | `/chat/message` | 发送聊天消息（支持 AI 和 MCP 查询） |
| `GET` | `/chat/history/:id` | 获取聊天历史 |
| `GET` | `/health` | 健康检查 |

#### 兼容接口（向后兼容）

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/results` | 查询上传结果列表 |
| `GET` | `/results/:id` | 获取单条上传结果 |
| `GET` | `/detections` | 查询实时检测列表 |
| `POST` | `/detections` | 创建实时检测记录 |
| `GET` | `/detections/:id` | 获取单条检测记录 |
| `GET` | `/detections/:id/snapshot` | 获取检测快照 |

**查询参数示例**:

```bash
# 统一记录查询
GET /api/v1/records?page=1&limit=20&source=upload&status=completed

# source 可选值: all, upload, webcam, mobile
# status 可选值: completed, processing, failed
```

### AI 服务 API (http://localhost:8002)

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `POST` | `/infer` | 图像推理（multipart/form-data） |
| `GET` | `/health` | 健康检查 |
| `GET` | `/models` | 列出所有模型 |
| `GET` | `/models/:name` | 获取模型详情 |
| `GET` | `/metrics` | 获取服务指标 |

详细 API 文档: [docs/API.md](docs/API.md)

## 📁 项目结构

```
guoyan-zhizao/
├── frontend/                    # Vue 3 + TypeScript 前端
│   ├── src/
│   │   ├── api/                # API 接口封装
│   │   │   ├── http.ts        # Axios 实例配置
│   │   │   ├── records.ts     # 统一记录 API
│   │   │   └── chat.ts        # 聊天 API
│   │   ├── components/         # 组件
│   │   │   ├── ChatWidget.vue # AI 聊天助手组件
│   │   │   └── ...            # 其他组件
│   │   ├── views/              # 页面组件
│   │   │   ├── HomeView.vue   # 首页（上传+统计）
│   │   │   └── RecordsView.vue # 记录查看（表格/卡片视图）
│   │   ├── router/             # 路由配置
│   │   ├── stores/             # Pinia 状态管理
│   │   └── App.vue             # 根组件
│   ├── public/                 # 静态资源
│   ├── package.json            # 前端依赖
│   ├── vite.config.ts          # Vite 配置
│   └── Dockerfile              # 前端容器化配置
│
├── backend/                     # Rust + Axum 后端
│   ├── src/
│   │   ├── routes/             # API 路由
│   │   │   ├── records.rs     # 统一记录接口
│   │   │   ├── uploads.rs     # 图片上传
│   │   │   ├── results.rs     # 结果查询
│   │   │   ├── detections.rs  # 实时检测
│   │   │   ├── statistics.rs  # 统计数据
│   │   │   ├── chat.rs        # 聊天服务（NEW）
│   │   │   └── health.rs      # 健康检查
│   │   ├── services/           # 服务层
│   │   │   ├── llm_client.rs  # LLM 客户端（NEW）
│   │   │   └── mcp_client.rs  # MCP 客户端（NEW）
│   │   ├── models.rs           # 数据模型
│   │   ├── db.rs               # 数据库连接
│   │   ├── cache.rs            # Redis 缓存
│   │   ├── ai_client.rs        # AI 服务客户端
│   │   ├── errors.rs           # 错误处理
│   │   ├── config.rs           # 配置管理
│   │   └── main.rs             # 应用入口
│   ├── migrations/             # 数据库迁移脚本
│   ├── Cargo.toml              # Rust 依赖
│   └── Dockerfile              # 后端容器化配置
│
├── ai/                          # Python + FastAPI AI 服务
│   ├── app/                    # 应用代码
│   │   ├── api.py             # FastAPI 路由
│   │   ├── models.py          # DenseNet121 模型
│   │   ├── service.py         # 推理服务
│   │   ├── schemas.py         # 数据模型
│   │   ├── config.py          # 配置管理
│   │   ├── model_cache.py     # 模型缓存
│   │   └── utils/             # 工具函数
│   │       ├── preprocess.py  # 图像预处理
│   │       └── postprocess.py # 质量评估
│   ├── model/                  # 模型权重
│   │   ├── fruit_densenet_weights.pth
│   │   └── labels.json
│   ├── tests/                  # 单元测试
│   ├── pyproject.toml          # Python 项目配置
│   ├── requirements.txt        # Python 依赖
│   └── Dockerfile              # AI 服务容器化
│
├── init-db/                     # 数据库初始化脚本
│   ├── 00-schema.sql           # 表结构
│   └── 02-seed-test-data.sql   # 测试数据
│
├── docs/                        # 项目文档
│   ├── README.md               # 文档中心索引
│   ├── API.md                  # API 接口文档
│   ├── PROJECT_GUIDE.md        # 项目开发指南
│   ├── DEPLOY.md               # 部署手册
│   ├── ALGORITHM.md            # 算法说明
│   ├── PORTS.md                # 端口文档
│   └── archive/                # 归档文档
│       └── specifications/     # 项目规范文档归档
│
├── scripts/                     # 辅助脚本
│   ├── start.sh                # 一键启动脚本
│   ├── verify-database.sh      # 数据库验证脚本
│   └── verify-setup.sh         # 环境验证脚本
│
├── logs/                        # 日志目录（自动创建）
│   ├── backend/
│   ├── frontend/
│   ├── ai/
│   ├── postgres/
│   └── redis/
│
├── docker-compose.yml           # Docker Compose 配置
├── mcp-config.json             # MCP 服务器配置
├── .env.example                 # 环境变量模板
└── README.md                    # 项目说明（本文件）
```

## 📚 文档索引

- **[文档中心](docs/README.md)** - 完整的文档导航和索引
- **[API 参考](docs/API.md)** - 完整的 REST API 接口定义与示例
- **[项目指南](docs/PROJECT_GUIDE.md)** - 架构设计、模块说明、技术选型
- **[部署手册](docs/DEPLOY.md)** - 生产环境部署、运维建议、性能优化
- **[算法说明](docs/ALGORITHM.md)** - DenseNet121 模型、质量评估、训练细节
- **[端口文档](docs/PORTS.md)** - 所有服务端口说明与访问方式
- **[规范文档归档](docs/archive/specifications/)** - 项目开发过程的详细规格说明和设计文档

## 🔧 常用命令

### Docker Compose 管理

```bash
# 启动所有服务
docker compose up -d

# 查看服务状态
docker compose ps

# 查看实时日志
docker compose logs -f

# 查看特定服务日志
docker compose logs -f backend
docker compose logs -f ai

# 重启服务
docker compose restart backend

# 停止服务
docker compose down

# 停止并删除所有数据（重新初始化）
docker compose down -v

# 重新构建镜像
docker compose build --no-cache
```

### 数据库管理

```bash
# 连接到 PostgreSQL
docker exec -it gyz-postgres psql -U gyz -d gyz

# 查看表结构
docker exec -it gyz-postgres psql -U gyz -d gyz -c "\dt"

# 查询数据
docker exec -it gyz-postgres psql -U gyz -d gyz -c "SELECT COUNT(*) FROM uploads;"

# 执行 SQL 脚本
docker exec -i gyz-postgres psql -U gyz -d gyz < init-db/01-init-tables.sql
```

### Redis 管理

```bash
# 连接到 Redis
docker exec -it gyz-redis redis-cli -a 6RQgSuJH

# 查看所有键
docker exec -it gyz-redis redis-cli -a 6RQgSuJH keys "*"

# 清空缓存
docker exec -it gyz-redis redis-cli -a 6RQgSuJH FLUSHALL

# 查看缓存统计
docker exec -it gyz-redis redis-cli -a 6RQgSuJH INFO stats
```

## 🐛 故障排查

### 服务无法启动

```bash
# 检查容器状态
docker compose ps -a

# 查看详细日志
docker compose logs [service-name]

# 检查端口占用
netstat -tuln | grep -E "3000|8080|8002|5432|6379"

# 清理并重新启动
docker compose down -v
docker compose up -d
```

### 数据库连接失败

```bash
# 验证数据库连接
docker exec gyz-backend env | grep DATABASE_URL

# 测试数据库连接
docker exec -it gyz-postgres pg_isready -U gyz

# 检查数据库日志
docker compose logs postgres
```

### AI 服务推理失败

```bash
# 检查模型文件
ls -lh ai/model/

# 查看 AI 服务日志
docker compose logs ai

# 手动测试推理接口
curl -X POST http://localhost:8002/infer \
  -F "upload_id=test-001" \
  -F "file=@test-image.jpg"
```

### 前端无法访问后端

```bash
# 检查代理配置
cat frontend/vite.config.ts

# 验证后端健康
curl http://localhost:8080/health

# 检查前端构建日志
docker compose logs frontend
```

## 🚢 生产部署建议

1. **环境变量**: 务必修改 `.env` 中的默认密码和密钥
2. **数据备份**: 定期备份 PostgreSQL 数据和 Redis 持久化文件
3. **日志管理**: 配置日志轮转，防止磁盘占满
4. **监控告警**: 部署 Prometheus + Grafana 监控服务状态
5. **反向代理**: 使用 Nginx/Caddy 作为前端代理，添加 SSL 证书
6. **资源限制**: 在 `docker-compose.yml` 中配置内存和 CPU 限制
7. **健康检查**: 利用 Docker 健康检查自动重启异常服务

详见 [docs/DEPLOY.md](docs/DEPLOY.md)

## 🤝 贡献指南

欢迎贡献代码、报告问题或提出建议！

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

详见项目许可证文件。

## 📞 支持与反馈

如遇问题或有建议，请通过以下方式联系：
- 提交 Issue
- 查看文档 [docs/](docs/)
- 邮件联系项目维护者

---

**最后更新**: 2025-10-14
**版本**: v1.0.0
**作者**: 果蔬智造团队
**状态**: 生产就绪
