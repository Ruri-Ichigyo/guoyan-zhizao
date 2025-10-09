# 生产环境部署指南

## 组件与资源建议

| 组件 | CPU | 内存 | 磁盘 | 备注 |
| --- | --- | --- | --- | --- |
| Rust 后端 | 4 核 | 4 GB | 20 GB | 负责接收上传、调用 AI、写入数据库 |
| AI 推理 (FastAPI + PyTorch) | 8 核 | 16 GB | 50 GB | 可选 GPU；如无 GPU 建议调低并发 |
| PostgreSQL | 4 核 | 8 GB | 100 GB SSD | 存储上传文件与检测结果 |
| Redis | 2 核 | 2 GB | 10 GB | 缓存热点数据 |
| 前端 | 1 核 | 1 GB | 5 GB | 可与反向代理共用节点 |

> 根据业务规模可水平扩展后端与 AI 服务，Redis/PostgreSQL 建议部署高可用集群。

## Docker Compose 部署

### 1. 准备环境

```bash
git clone <repository-url>
cd guoyan-zhizao
cp .env.example .env
# 根据需要调整数据库、缓存、端口等配置
```

### 2. 启动服务

```bash
./scripts/start.sh
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

### 3. 备份与数据迁移

- **PostgreSQL**：使用 `pg_dump` / `pg_restore` 或云厂商托管服务。
- **Redis**：生产环境建议使用持久化与主从复制。
- **上传文件**：当前方案将文件二进制存储在数据库，如需对象存储可自行拓展。

### 4. 反向代理与 HTTPS

可将前端产物部署至 Nginx / Caddy，并设置 API 反向代理到后端 8080 端口。示例 Nginx 配置片段：

```nginx
location /api/ {
    proxy_pass http://backend:8080/api/;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
}
```

使用 certbot/lego 等工具自动签发证书，将 HTTPS 终结在反向代理层。

## 监控与日志

- 后端默认输出 JSON 日志，可接入 Loki / Elastic Stack。
- PostgreSQL、Redis 建议使用云服务或现成的监控模板（Prometheus exporters）。
- AI 服务可增加请求计数与模型推理耗时指标。

## 安全加固建议

1. 为 Docker 网络配置专用子网，限制外部访问数据库与 Redis。
2. 修改 `.env` 中默认密码，并使用秘密管理服务托管敏感信息。
3. 前端与后端之间启用 HTTPS，后端可添加鉴权逻辑。
4. 定期更新镜像，关注依赖的安全公告。

## 扩展方向

- 对接对象存储以替代数据库二进制字段。
- 引入消息队列缓冲高峰上传任务。
- 增加多模型推理或分布式调度。

如需更复杂的部署（Kubernetes、CI/CD 等），可在此基础上自行编排。
