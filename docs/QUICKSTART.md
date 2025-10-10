# 快速启动指南

## 依赖要求

### Docker 部署（推荐）
- Docker Desktop / Docker Engine 20.10+
- Docker Compose v2

### 本地开发
- Rust 1.75+ 与 Cargo
- Python 3.10+
- Node.js 18+
- Docker（用于运行 PostgreSQL 与 Redis）

---

## 方式一：Docker 一键启动

```bash
cp .env.example .env
./scripts/start.sh
```

脚本会依次：
1. 构建 `postgres`、`redis`、`ai`、`backend`、`frontend` 镜像
2. 启动所有服务并等待健康检查通过

启动完成后：
- 前端：http://localhost:3000
- 后端 API：http://localhost:8080/api/v1
- AI 服务：http://localhost:8002/health
- PostgreSQL：`postgres://gyz:gyz123@localhost:5432/gyz`
- Redis：`redis://localhost:6379`
- 默认管理员：`admin` / `admin123456`

常用命令：
```bash
docker compose ps
docker compose logs -f backend
docker compose down
```

---

## 方式二：本地开发模式

### 1. 启动基础设施

```bash
./scripts/dev-start.sh  # 或手动执行 docker compose up -d postgres redis
```

### 2. 后端服务（Rust）
```bash
cd backend
export DATABASE_URL=postgres://gyz:gyz123@localhost:5432/gyz
export REDIS_URL=redis://localhost:6379
export AI_SERVICE_URL=http://localhost:8002
export JWT_SECRET=local-dev-secret           # 可选：覆盖 JWT 密钥
export JWT_TTL_SECONDS=3600                  # 可选：令牌有效期（秒）
export ADMIN_USERNAME=admin                  # 可选：覆盖默认管理员账号
export ADMIN_PASSWORD=admin123456            # 可选：覆盖默认管理员密码
cargo run
```

### 3. AI 服务（FastAPI）
```bash
cd ai
python -m venv .venv && source .venv/bin/activate  # 可选
pip install -r requirements.txt
uvicorn main:app --reload --host 0.0.0.0 --port 8002
```

### 4. 前端（Vue 3）
```bash
cd frontend
npm install
npm run dev
# 开发代理自动转发 /api/v1 到 http://localhost:8080
```

---

## 获取访问令牌

后端业务接口要求携带 `Authorization: Bearer <token>` 请求头。首次启动可使用默认管理员账号登录，或在 `.env` 中通过 `ADMIN_USERNAME` / `ADMIN_PASSWORD` 自定义后重启服务。

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123456"}'
```

返回结果包含 `token` 与 `expires_at` 字段，前端应用会自动保存并附带到后续请求。

---

## 数据初始化

首次启动会自动创建以下表结构：
- `uploads`：存储原始文件及状态
- `results`：存储检测详情和统计指标
- `users`：存储登录账户的加盐密码与角色信息

如需清空数据，可执行：
```bash
docker compose down -v  # 删除容器与卷
```

---

## 常见问题

- **构建失败（Rust 依赖下载）**：确保网络可访问 crates.io；必要时配置国内镜像。
- **AI 服务加载失败**：确认 `ai/model` 下存在权重文件与 `labels.json`。
- **前端无法请求**：检查 `VITE_API_BASE_URL` 是否与后端地址一致。

更多细节请参见 [README](../README.md) 与 [API 文档](API.md)。
