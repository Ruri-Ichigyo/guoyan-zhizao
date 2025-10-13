# 生产环境部署指南

## 环境要求

- **Docker**: 20.10.0 或更高版本
- **Docker Compose**: 2.0.0 或更高版本（Plugin 或独立版本均可）

### 安装 Docker Compose

**Ubuntu/Debian (推荐使用 Plugin):**
```bash
sudo apt-get update
sudo apt-get install docker-compose-plugin
```

**CentOS/RHEL:**
```bash
sudo yum install docker-compose-plugin
```

**独立版本 docker-compose:**
```bash
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

验证安装：
```bash
# Plugin 版本
docker compose version

# 或独立版本
docker-compose --version
```

> 💡 启动脚本会自动检测并使用可用的 Docker Compose 命令。

## 快速启动

```bash
# 1. 克隆项目
git clone <repository-url>
cd guoyan-zhizao

# 2. 配置环境变量
cp .env.example .env
# 编辑 .env 文件，设置数据库密码等

# 3. 赋予脚本执行权限
chmod +x scripts/*.sh

# 4. 启动所有服务
./scripts/start.sh

# 5. 停止服务
./scripts/stop.sh
```

启动成功后访问：
- 前端界面: http://localhost:3000
- 后端 API: http://localhost:8080
- API 文档: http://localhost:8080/api/docs
- AI 服务: http://localhost:8000

## 组件与资源建议

| 组件 | CPU | 内存 | 磁盘 | 备注 |
| --- | --- | --- | --- | --- |
| Rust 后端 | 4 核 | 4 GB | 20 GB | 负责接收上传、调用 AI、写入数据库 |
| AI 推理 (FastAPI + PyTorch) | 8 核 | 16 GB | 50 GB | 可选 GPU；如无 GPU 建议调低并发 |
| PostgreSQL | 4 核 | 8 GB | 100 GB SSD | 存储上传文件与检测结果 |
| Redis | 2 核 | 2 GB | 10 GB | 缓存热点数据 |
| 前端 | 1 核 | 1 GB | 5 GB | 可与反向代理共用节点 |



## Docker Compose 部署

### 1. 准备环境

```bash
git clone <repository-url>
cd guoyan-zhizao
cp .env.example .env
# 根据需要调整数据库、缓存、端口等配置
# ⚠️ 生产环境务必修改以下配置：
#   - POSTGRES_PASSWORD: 数据库密码
```

### 2. 启动服务

**使用启动脚本（推荐）：**

```bash
chmod +x scripts/start.sh scripts/stop.sh
./scripts/start.sh
```

**或手动启动：**

```bash
docker compose build
docker compose up -d
```

该脚本会：
1. 构建/更新镜像
2. 依序启动 PostgreSQL、Redis、AI、Backend、Frontend
3. 等待服务通过健康检查

完成后可使用以下命令查看状态：

```bash
docker compose ps
docker compose logs -f backend
```

首次启动后可直接访问 `http://<host>:3000` 进行上传与查询，无需登录认证。

### 3. 常用命令

```bash
# 查看服务状态
docker compose ps

# 查看所有日志
docker compose logs -f

# 查看特定服务日志
docker compose logs -f backend
docker compose logs -f ai

# 重启服务
docker compose restart backend

# 停止所有服务
./scripts/stop.sh
# 或
docker compose down

# 完全清理（包括数据卷）
docker compose down -v

# 进入容器调试
docker compose exec backend sh
docker compose exec postgres psql -U postgres -d guoyan_zhizao
```

### 4. 备份与数据迁移

- **PostgreSQL**：使用 `pg_dump` / `pg_restore` 或云厂商托管服务。
- **Redis**：生产环境建议使用持久化与主从复制。
- **上传文件**：当前方案将文件二进制存储在数据库，如需对象存储可自行拓展。

### 5. 反向代理与 HTTPS

可将前端产物部署至 Nginx / Caddy，并设置 API 反向代理到后端 8080 端口。示例 Nginx 配置片段：

```nginx
location /api/ {
    proxy_pass http://backend:8080/api/;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
}
```

使用 certbot/lego 等工具自动签发证书，将 HTTPS 终结在反向代理层。

## 故障排查

### 服务启动失败

1. 查看具体服务日志：
   ```bash
   docker compose logs backend
   docker compose logs ai
   ```

2. 检查端口占用：
   ```bash
   lsof -i :3000  # 前端
   lsof -i :8080  # 后端
   lsof -i :8000  # AI
   lsof -i :5432  # PostgreSQL
   lsof -i :6379  # Redis
   ```

3. 检查磁盘空间：
   ```bash
   df -h
   docker system df
   ```

### 数据库连接失败

```bash
# 检查 PostgreSQL 是否就绪
docker compose exec postgres pg_isready -U postgres

# 手动连接测试
docker compose exec postgres psql -U postgres -d guoyan_zhizao
```

### AI 服务性能问题

- 如无 GPU，在 `docker-compose.yml` 中调整 AI 服务的并发设置
- 考虑增加 AI 服务的内存限制
- 监控 CPU/内存使用率

## 监控与日志

- 后端默认输出 JSON 日志，可接入 Loki / Elastic Stack。
- PostgreSQL、Redis 建议使用云服务或现成的监控模板（Prometheus exporters）。
- AI 服务可增加请求计数与模型推理耗时指标。

## 安全加固建议

1. 为 Docker 网络配置专用子网，限制外部访问数据库与 Redis。
2. 修改 `.env` 中默认密码等敏感配置，并使用秘密管理服务托管。
3. 前端与后端之间启用 HTTPS，后端可添加鉴权逻辑。
4. 定期更新镜像，关注依赖的安全公告。
