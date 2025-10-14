# 端口与服务配置文档

本文档详细说明果蔬智造平台所有服务的端口配置、访问方式和网络架构。

## 🔌 端口总览

| 服务名称 | 容器名称 | 主机端口 | 容器端口 | 协议 | 访问方式 | 说明 |
|---------|---------|---------|---------|------|---------|------|
| **前端** | gyz-frontend | 3000 | 80 | HTTP | 浏览器/HTTP客户端 | Vue 3 应用（Nginx 托管） |
| **后端 API** | gyz-backend | 8080 | 8080 | HTTP | HTTP客户端/前端 | Rust Axum RESTful API |
| **AI 服务** | gyz-ai | 8002 | 8002 | HTTP | 内部调用（后端） | FastAPI 推理服务 |
| **PostgreSQL** | gyz-postgres | 5432 | 5432 | TCP | psql/JDBC | 主数据库 |
| **Redis** | gyz-redis | 6379 | 6379 | TCP | redis-cli | 缓存服务 |

---

## 📦 服务详细说明

### 1. 前端服务 (Frontend)

**容器名称**: `gyz-frontend`
**镜像**: 基于 Node 18 + Nginx Alpine
**端口映射**: `3000:80`

#### 访问方式

- **浏览器访问**: http://localhost:3000
- **健康检查**: http://localhost:3000/health
- **API 代理**: 前端自动将 `/api/v1/*` 代理到后端 `http://backend:8080`

#### 配置文件

- **Vite 配置**: `frontend/vite.config.ts`
  ```typescript
  server: {
    port: 3000,
    proxy: {
      '/api/v1': {
        target: 'http://localhost:8080',
        changeOrigin: true
      }
    }
  }
  ```

- **Nginx 配置** (生产环境):
  - 配置文件: `frontend/nginx.conf`
  - 静态文件目录: `/usr/share/nginx/html`
  - 日志目录: `/var/log/nginx` (映射到宿主机 `./logs/frontend`)

#### 环境变量

```bash
NODE_ENV=production
VITE_API_BASE_URL=/api/v1
```

#### 网络连接

- **入站**: 用户浏览器 → 端口 3000
- **出站**: Nginx → 后端容器 (backend:8080)

---

### 2. 后端 API 服务 (Backend)

**容器名称**: `gyz-backend`
**镜像**: 基于 Rust (多阶段构建)
**端口映射**: `8080:8080`

#### 访问方式

- **API 基础 URL**: http://localhost:8080/api/v1
- **健康检查**: http://localhost:8080/health
- **直接访问**: 任何 HTTP 客户端（curl, Postman, 前端）

#### 主要端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/api/v1/uploads` | POST | 上传图片 |
| `/api/v1/records` | GET | 查询记录 |
| `/api/v1/records/:id` | GET/DELETE | 记录详情/删除 |
| `/api/v1/records/:id/image` | GET | 获取图片 |
| `/api/v1/statistics` | GET | 统计数据 |

#### 环境变量

```bash
RUST_LOG=info
DATABASE_URL=postgres://gyz:nU1%2FzzGa@postgres:5432/gyz
REDIS_URL=redis://:6RQgSuJH@redis:6379
AI_SERVICE_URL=http://ai:8002
BIND_ADDRESS=0.0.0.0:8080
CACHE_TTL_SECONDS=900
```

**重要提示**:
- `DATABASE_URL` 中的特殊字符需要 URL 编码（`/` → `%2F`）
- 容器内网络使用服务名称（`postgres`, `redis`, `ai`）而非 localhost

#### 网络连接

- **入站**: 前端容器、外部 HTTP 客户端 → 端口 8080
- **出站**:
  - PostgreSQL 容器 (postgres:5432)
  - Redis 容器 (redis:6379)
  - AI 服务容器 (ai:8002)

#### 日志位置

- **容器内**: `/app/logs`
- **宿主机映射**: `./logs/backend`

---

### 3. AI 推理服务 (AI Service)

**容器名称**: `gyz-ai`
**镜像**: 基于 Python 3.12 + PyTorch
**端口映射**: `8002:8002`

#### 访问方式

- **API 基础 URL**: http://localhost:8002
- **健康检查**: http://localhost:8002/health
- **推理接口**: http://localhost:8002/infer (POST)
- **模型列表**: http://localhost:8002/models
- **服务指标**: http://localhost:8002/metrics

⚠️ **注意**: 生产环境中，AI 服务通常仅供后端内部调用，不对外暴露。

#### 主要端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/infer` | POST | 图像推理（multipart/form-data） |
| `/models` | GET | 列出所有模型 |
| `/models/:name` | GET | 获取模型详情 |
| `/metrics` | GET | 服务运行指标 |

#### 环境变量

```bash
PYTHONUNBUFFERED=1
API_HOST=0.0.0.0
API_PORT=8002
MODEL_PATH=model/fruit_densenet_weights.pth
USE_GPU=True
TOP_K=3
```

#### 网络连接

- **入站**: 后端容器 (backend:8080) → 端口 8002
- **出站**: 无（仅被动响应）

#### 挂载卷

```yaml
volumes:
  - ./ai/model:/app/model          # 模型权重
  - ./logs/ai:/app/logs             # 日志文件
```

#### 性能配置

- **GPU 支持**: 自动检测 CUDA 可用性，不可用时降级到 CPU
- **模型缓存**: 使用 LRU 缓存，最多缓存 10 个模型
- **推理超时**: 建议 30 秒（在后端 AI 客户端配置）

---

### 4. PostgreSQL 数据库

**容器名称**: `gyz-postgres`
**镜像**: `postgres:15-alpine`
**端口映射**: `5432:5432`

#### 访问方式

**命令行客户端**:
```bash
# Docker 内部访问
docker exec -it gyz-postgres psql -U gyz -d gyz

# 宿主机访问
psql -h localhost -p 5432 -U gyz -d gyz

# JDBC URL
jdbc:postgresql://localhost:5432/gyz?user=gyz&password=nU1/zzGa
```

#### 数据库配置

| 配置项 | 值 | 说明 |
|-------|---|------|
| 数据库名 | `gyz` | 主数据库 |
| 用户名 | `gyz` | 数据库用户 |
| 密码 | `nU1/zzGa` | **生产环境务必修改** |
| 端口 | 5432 | 标准 PostgreSQL 端口 |
| 字符集 | UTF-8 | 默认字符编码 |

#### 环境变量

```bash
POSTGRES_DB=gyz
POSTGRES_USER=gyz
POSTGRES_PASSWORD=nU1/zzGa
```

#### 数据持久化

```yaml
volumes:
  - postgres_data:/var/lib/postgresql/data    # 数据文件
  - ./logs/postgres:/var/log/postgresql       # 日志文件
  - ./init-db:/docker-entrypoint-initdb.d     # 初始化脚本
```

#### 初始化脚本

容器首次启动时自动执行：
1. `01-init-tables.sql` - 创建表结构和索引
2. `02-seed-test-data.sql` - 插入测试数据（80+ 条记录）

#### 健康检查

```bash
# 容器健康检查命令
pg_isready -U gyz -d gyz

# 手动检查
docker exec gyz-postgres pg_isready -U gyz
```

#### 主要数据表

| 表名 | 说明 | 主要字段 |
|------|------|---------|
| `uploads` | 上传记录 | id, filename, data, status, created_at |
| `results` | 检测结果 | upload_id, detections, total_fruits, grade_a/b/c |
| `detections` | 实时检测 | id, source, labels, confidence_scores, snapshot |

#### 网络连接

- **入站**: 后端容器 (backend) → 端口 5432
- **出站**: 无

---

### 5. Redis 缓存服务

**容器名称**: `gyz-redis`
**镜像**: `redis:7-alpine`
**端口映射**: `6379:6379`

#### 访问方式

**命令行客户端**:
```bash
# Docker 内部访问
docker exec -it gyz-redis redis-cli -a 6RQgSuJH

# 宿主机访问
redis-cli -h localhost -p 6379 -a 6RQgSuJH

# 连接 URL
redis://:6RQgSuJH@localhost:6379
```

#### Redis 配置

| 配置项 | 值 | 说明 |
|-------|---|------|
| 密码 | `6RQgSuJH` | **生产环境务必修改** |
| 端口 | 6379 | 标准 Redis 端口 |
| 持久化 | RDB | 每 900 秒保存一次（有修改时） |
| 日志级别 | warning | 生产环境配置 |

#### 启动命令

```bash
redis-server \
  --requirepass 6RQgSuJH \
  --save 900 1 \
  --loglevel warning \
  --logfile /var/log/redis/redis.log
```

#### 环境变量

```bash
REDIS_PASSWORD=6RQgSuJH
```

#### 数据持久化

```yaml
volumes:
  - redis_data:/data                    # RDB 持久化文件
  - ./logs/redis:/var/log/redis         # 日志文件
```

#### 缓存策略

| 缓存键前缀 | TTL | 说明 |
|-----------|-----|------|
| `records:list:*` | 5 分钟 | 记录列表查询结果 |
| `records:detail:*` | 15 分钟 | 单条记录详情 |
| `records:image:*` | 1 小时 | 图片/快照数据 |
| `statistics:*` | 5 分钟 | 统计数据 |

#### 常用命令

```bash
# 查看所有键
redis-cli -a 6RQgSuJH keys "*"

# 查看缓存统计
redis-cli -a 6RQgSuJH INFO stats

# 清空所有缓存
redis-cli -a 6RQgSuJH FLUSHALL

# 监控实时命令
redis-cli -a 6RQgSuJH MONITOR
```

#### 健康检查

```bash
# 容器健康检查命令
redis-cli -a 6RQgSuJH ping

# 预期响应: PONG
```

#### 网络连接

- **入站**: 后端容器 (backend) → 端口 6379
- **出站**: 无

---

## 🌐 网络架构

### Docker 网络拓扑

```
┌─────────────────────────────────────────────────────────────┐
│                      宿主机 (Host)                            │
│                                                               │
│  浏览器 ──→ :3000  :8080  :8002  :5432  :6379                │
│              │       │       │       │       │                │
└──────────────┼───────┼───────┼───────┼───────┼───────────────┘
               │       │       │       │       │
               ↓       ↓       ↓       ↓       ↓
┌─────────────────────────────────────────────────────────────┐
│            Docker 网络 (guoyan-zhizao_default)               │
│                                                               │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐               │
│  │ frontend │───→│ backend  │───→│   ai     │               │
│  │  :80     │    │  :8080   │    │  :8002   │               │
│  └──────────┘    └────┬─────┘    └──────────┘               │
│                       │                                       │
│                       ├─────→┌──────────┐                    │
│                       │      │ postgres │                    │
│                       │      │  :5432   │                    │
│                       │      └──────────┘                    │
│                       │                                       │
│                       └─────→┌──────────┐                    │
│                              │  redis   │                    │
│                              │  :6379   │                    │
│                              └──────────┘                    │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 服务依赖关系

```
frontend
  ↓
backend
  ├─→ postgres (必需)
  ├─→ redis (必需)
  └─→ ai (必需)
```

**启动顺序**:
1. PostgreSQL、Redis (并行启动，健康检查)
2. AI 服务 (健康检查)
3. 后端服务 (等待所有依赖就绪)
4. 前端服务 (等待后端启动)

---

## 🔒 安全配置建议

### 生产环境端口策略

#### 对外暴露
- ✅ **端口 3000** (前端) - 通过反向代理 (Nginx/Caddy) 暴露，配置 SSL
- ✅ **端口 8080** (后端API) - 可选暴露，建议仅通过前端代理访问

#### 仅内部访问
- ❌ **端口 8002** (AI 服务) - 仅供后端调用，禁止外网访问
- ❌ **端口 5432** (PostgreSQL) - 仅供后端访问，使用防火墙限制
- ❌ **端口 6379** (Redis) - 仅供后端访问，使用防火墙限制

### 防火墙规则示例 (iptables)

```bash
# 允许前端端口
iptables -A INPUT -p tcp --dport 3000 -j ACCEPT

# 允许后端 API（可选）
iptables -A INPUT -p tcp --dport 8080 -j ACCEPT

# 拒绝数据库直接访问
iptables -A INPUT -p tcp --dport 5432 -j DROP
iptables -A INPUT -p tcp --dport 6379 -j DROP
iptables -A INPUT -p tcp --dport 8002 -j DROP
```

### Docker 网络隔离

生产环境建议配置多个独立网络：

```yaml
networks:
  frontend-net:
    # 前端 ↔ 后端
  backend-net:
    # 后端 ↔ AI、数据库
  db-net:
    internal: true  # 仅内部访问
    # PostgreSQL、Redis
```

---

## 🔍 端口诊断与故障排查

### 检查端口占用

```bash
# Linux/macOS
netstat -tuln | grep -E "3000|8080|8002|5432|6379"
lsof -i :3000

# 查看 Docker 端口映射
docker compose ps
docker port gyz-backend
```

### 测试端口连通性

```bash
# 检查端口是否监听
nc -zv localhost 3000
nc -zv localhost 8080
nc -zv localhost 8002
nc -zv localhost 5432
nc -zv localhost 6379

# 测试 HTTP 服务
curl -I http://localhost:3000
curl -I http://localhost:8080/health
curl -I http://localhost:8002/health

# 测试数据库连接
pg_isready -h localhost -p 5432 -U gyz

# 测试 Redis 连接
redis-cli -h localhost -p 6379 -a 6RQgSuJH ping
```

### 容器内部网络测试

```bash
# 从后端容器测试其他服务
docker exec gyz-backend sh -c "nc -zv postgres 5432"
docker exec gyz-backend sh -c "nc -zv redis 6379"
docker exec gyz-backend sh -c "nc -zv ai 8002"

# 从后端容器访问 AI 服务
docker exec gyz-backend curl http://ai:8002/health
```

### 日志查看

```bash
# 查看特定服务日志
docker compose logs -f frontend
docker compose logs -f backend
docker compose logs -f ai
docker compose logs -f postgres
docker compose logs -f redis

# 查看宿主机日志文件
tail -f logs/backend/*.log
tail -f logs/ai/*.log
tail -f logs/postgres/*.log
```

---

## 📋 环境变量参考

### .env 文件示例

```bash
# PostgreSQL
POSTGRES_DB=gyz
POSTGRES_USER=gyz
POSTGRES_PASSWORD=nU1/zzGa

# Redis
REDIS_PASSWORD=6RQgSuJH

# Backend
RUST_LOG=info
DATABASE_URL=postgres://gyz:nU1%2FzzGa@postgres:5432/gyz
REDIS_URL=redis://:6RQgSuJH@redis:6379
AI_SERVICE_URL=http://ai:8002
BIND_ADDRESS=0.0.0.0:8080
CACHE_TTL_SECONDS=900

# AI Service
PYTHONUNBUFFERED=1
API_HOST=0.0.0.0
API_PORT=8002
USE_GPU=True
TOP_K=3

# Frontend (构建时)
NODE_ENV=production
VITE_API_BASE_URL=/api/v1
```

---

## 🚀 性能优化建议

### 端口和连接池配置

#### PostgreSQL 连接池
```rust
// backend/src/db.rs
PoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(10))
```

#### Redis 连接池
```rust
// backend/src/cache.rs
redis::Client::open("redis://:password@redis:6379")?
    .get_multiplexed_tokio_connection_manager()
```

#### HTTP 客户端超时
```rust
// backend/src/ai_client.rs
reqwest::Client::builder()
    .timeout(Duration::from_secs(30))  // AI 推理超时
    .build()?
```

### Nginx 反向代理配置

```nginx
upstream backend {
    server backend:8080 max_fails=3 fail_timeout=30s;
    keepalive 32;
}

server {
    listen 80;

    location /api/v1/ {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_buffering off;
    }
}
```

---

## 📞 技术支持

如遇端口配置问题或网络连接异常，请参考：
- [QUICKSTART.md](QUICKSTART.md) - 快速启动与验证
- [API.md](API.md) - API 接口详细说明
- [故障排查章节](#-端口诊断与故障排查)

---

**最后更新**: 2025-10-14
**文档版本**: v1.0
**维护**: 果蔬智造团队
