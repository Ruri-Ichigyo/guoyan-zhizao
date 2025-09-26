# 生产环境部署指南

## 概览

本指南详细介绍如何将果蔬智能分拣系统部署到生产环境，包括单机部署、集群部署、监控配置和扩容策略。

## 系统要求

### 最低配置

| 组件 | CPU | 内存 | 存储 | GPU |
|------|-----|------|------|-----|
| Kong 网关 | 2 核 | 4GB | 20GB | - |
| Rust 服务 | 4 核 | 8GB | 50GB | - |
| AI 推理 | 8 核 | 16GB | 100GB | RTX 3080 或以上 |
| 前端 | 2 核 | 2GB | 10GB | - |
| MinIO | 4 核 | 8GB | 500GB | - |
| 数据库 | 4 核 | 8GB | 200GB | - |

### 推荐配置

| 组件 | CPU | 内存 | 存储 | GPU |
|------|-----|------|------|-----|
| Kong 网关 | 4 核 | 8GB | 50GB | - |
| Rust 服务 | 8 核 | 16GB | 100GB | - |
| AI 推理 | 16 核 | 32GB | 500GB | RTX 4090 或 A100 |
| 前端 | 4 核 | 4GB | 20GB | - |
| MinIO | 8 核 | 16GB | 2TB | - |
| 数据库 | 8 核 | 16GB | 500GB SSD | - |

## Docker Compose 部署

### 1. 环境准备

```bash
# 克隆项目
git clone <repository-url>
cd guoyan-zhizao

# 创建环境配置文件
cp .env.example .env
```

### 2. 配置环境变量

编辑 `.env` 文件：

```bash
# 数据库配置
POSTGRES_DB=fruit_sort
POSTGRES_USER=postgres
POSTGRES_PASSWORD=your-secure-password
DATABASE_URL=postgresql://postgres:your-secure-password@postgres:5432/fruit_sort

# Redis 配置
REDIS_URL=redis://redis:6379

# MinIO 配置
MINIO_ROOT_USER=admin
MINIO_ROOT_PASSWORD=your-minio-password
MINIO_ENDPOINT=http://minio:9000

# Kafka 配置
KAFKA_BOOTSTRAP_SERVERS=kafka:9092

# JWT 配置
JWT_SECRET=your-256-bit-secret-key-change-in-production

# AI 服务配置
MODEL_PATH=/app/model/yolov8n-fruit.pt
INFERENCE_BATCH_SIZE=4
CONFIDENCE_THRESHOLD=0.5

# 域名配置
DOMAIN=your-domain.com
```

### 3. 创建 docker-compose.yml

```yaml
version: '3.8'

services:
  # 数据库
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: ${POSTGRES_DB}
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER} -d ${POSTGRES_DB}"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # Redis 缓存
  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 3s
      retries: 5
    restart: unless-stopped

  # Kafka 消息队列
  zookeeper:
    image: confluentinc/cp-zookeeper:7.4.0
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181
      ZOOKEEPER_TICK_TIME: 2000
    volumes:
      - zookeeper_data:/var/lib/zookeeper/data
    restart: unless-stopped

  kafka:
    image: confluentinc/cp-kafka:7.4.0
    depends_on:
      - zookeeper
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: PLAINTEXT:PLAINTEXT
      KAFKA_INTER_BROKER_LISTENER_NAME: PLAINTEXT
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_AUTO_CREATE_TOPICS_ENABLE: true
    volumes:
      - kafka_data:/var/lib/kafka/data
    ports:
      - "9092:9092"
    healthcheck:
      test: ["CMD", "kafka-broker-api-versions", "--bootstrap-server", "localhost:9092"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

  # MinIO 对象存储
  minio:
    build: ./minio
    environment:
      MINIO_ROOT_USER: ${MINIO_ROOT_USER}
      MINIO_ROOT_PASSWORD: ${MINIO_ROOT_PASSWORD}
    volumes:
      - minio_data:/data
    ports:
      - "9000:9000"
      - "9001:9001"
    command: minio server /data --console-address ":9001"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 30s
      timeout: 20s
      retries: 3
    restart: unless-stopped

  # Kong API 网关
  kong:
    build: ./gateway
    depends_on:
      - postgres
    environment:
      KONG_DATABASE: postgres
      KONG_PG_HOST: postgres
      KONG_PG_USER: ${POSTGRES_USER}
      KONG_PG_PASSWORD: ${POSTGRES_PASSWORD}
      KONG_PROXY_ACCESS_LOG: /dev/stdout
      KONG_ADMIN_ACCESS_LOG: /dev/stdout
      KONG_PROXY_ERROR_LOG: /dev/stderr
      KONG_ADMIN_ERROR_LOG: /dev/stderr
      KONG_ADMIN_LISTEN: 0.0.0.0:8001
    ports:
      - "8000:8000"
      - "8001:8001"
      - "8443:8443"
      - "8444:8444"
    healthcheck:
      test: ["CMD", "kong", "health"]
      interval: 10s
      timeout: 10s
      retries: 5
    restart: unless-stopped

  # 文件上传服务
  upload-svc:
    build:
      context: ./backend
      dockerfile: upload-svc/Dockerfile
    depends_on:
      - postgres
      - kafka
      - minio
    environment:
      DATABASE_URL: ${DATABASE_URL}
      KAFKA_BOOTSTRAP_SERVERS: ${KAFKA_BOOTSTRAP_SERVERS}
      MINIO_ENDPOINT: ${MINIO_ENDPOINT}
      MINIO_ACCESS_KEY: ${MINIO_ROOT_USER}
      MINIO_SECRET_KEY: ${MINIO_ROOT_PASSWORD}
    volumes:
      - upload_temp:/tmp/uploads
    ports:
      - "3001:3001"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 2G
        reservations:
          memory: 1G

  # 查询服务
  query-svc:
    build:
      context: ./backend
      dockerfile: query-svc/Dockerfile
    depends_on:
      - postgres
      - redis
    environment:
      DATABASE_URL: ${DATABASE_URL}
      REDIS_URL: ${REDIS_URL}
    ports:
      - "3002:3002"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3002/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 2G
        reservations:
          memory: 1G

  # 分拣控制服务
  sort-svc:
    build:
      context: ./backend
      dockerfile: sort-svc/Dockerfile
    depends_on:
      - postgres
    environment:
      DATABASE_URL: ${DATABASE_URL}
    ports:
      - "3003:3003"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3003/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

  # AI 推理服务
  ai-inference:
    build: ./ai
    depends_on:
      - kafka
      - minio
    environment:
      KAFKA_BOOTSTRAP_SERVERS: ${KAFKA_BOOTSTRAP_SERVERS}
      MINIO_ENDPOINT: ${MINIO_ENDPOINT}
      MINIO_ACCESS_KEY: ${MINIO_ROOT_USER}
      MINIO_SECRET_KEY: ${MINIO_ROOT_PASSWORD}
      MODEL_PATH: ${MODEL_PATH}
      INFERENCE_BATCH_SIZE: ${INFERENCE_BATCH_SIZE}
      CONFIDENCE_THRESHOLD: ${CONFIDENCE_THRESHOLD}
      CUDA_VISIBLE_DEVICES: "0"
    volumes:
      - ./ai/model:/app/model
      - ai_cache:/app/cache
    ports:
      - "3004:3004"
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
    restart: unless-stopped

  # 前端服务
  frontend:
    build: ./frontend
    depends_on:
      - kong
    environment:
      VITE_API_BASE_URL: http://${DOMAIN}:8000
      VITE_WS_URL: ws://${DOMAIN}:8000
    ports:
      - "3000:80"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:80/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  zookeeper_data:
  kafka_data:
  minio_data:
  upload_temp:
  ai_cache:

networks:
  default:
    driver: bridge
```

### 4. 启动服务

```bash
# 创建并启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看服务日志
docker-compose logs -f ai-inference

# 停止所有服务
docker-compose down

# 停止服务并删除数据卷（谨慎使用）
docker-compose down -v
```

## Kubernetes 部署

### 1. 创建命名空间

```bash
kubectl apply -f k8s/namespace.yaml
```

### 2. 创建配置和密钥

```bash
# 应用配置
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secrets.yaml
```

### 3. 部署基础服务

```bash
# 数据库和消息队列
kubectl apply -f k8s/deployments/postgres.yaml
kubectl apply -f k8s/deployments/redis.yaml
kubectl apply -f k8s/deployments/kafka.yaml
kubectl apply -f k8s/deployments/minio.yaml
```

### 4. 部署应用服务

```bash
# 后端服务
kubectl apply -f k8s/deployments/upload-svc.yaml
kubectl apply -f k8s/deployments/query-svc.yaml
kubectl apply -f k8s/deployments/sort-svc.yaml

# AI 推理服务
kubectl apply -f k8s/deployments/ai-inference.yaml

# 前端和网关
kubectl apply -f k8s/deployments/kong.yaml
kubectl apply -f k8s/deployments/frontend.yaml
```

### 5. 配置 Ingress

```bash
kubectl apply -f k8s/ingress.yaml
```

## 监控和日志

### 1. Prometheus 监控

```yaml
# prometheus.yml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
      - job_name: 'fruit-sort-services'
        static_configs:
          - targets:
            - 'upload-svc:3001'
            - 'query-svc:3002'
            - 'sort-svc:3003'
            - 'ai-inference:3004'
        metrics_path: '/metrics'
        scrape_interval: 30s
```

### 2. Grafana 仪表板

```json
{
  "dashboard": {
    "title": "果蔬分拣系统监控",
    "panels": [
      {
        "title": "请求 QPS",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])"
          }
        ]
      },
      {
        "title": "响应时间",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "AI 推理性能",
        "targets": [
          {
            "expr": "rate(inference_total[5m])"
          }
        ]
      }
    ]
  }
}
```

### 3. ELK 日志收集

```yaml
# filebeat.yml
filebeat.inputs:
- type: container
  paths:
    - '/var/lib/docker/containers/*/*.log'
  processors:
  - add_docker_metadata:
      host: "unix:///var/run/docker.sock"

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "fruit-sort-logs-%{+yyyy.MM.dd}"
```

## SSL/TLS 配置

### 1. Let's Encrypt 自动证书

```bash
# 安装 certbot
sudo apt-get install certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d your-domain.com

# 自动续期
sudo crontab -e
0 12 * * * /usr/bin/certbot renew --quiet
```

### 2. Nginx 配置

```nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    location /api/ {
        proxy_pass http://localhost:8000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /ws/ {
        proxy_pass http://localhost:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}
```

## 备份和恢复

### 1. 数据库备份

```bash
# 创建备份脚本
cat > backup.sh << 'EOF'
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup"

# PostgreSQL 备份
docker exec postgres pg_dump -U $POSTGRES_USER $POSTGRES_DB > "$BACKUP_DIR/postgres_$DATE.sql"

# MinIO 数据备份
docker exec minio mc mirror --overwrite /data "$BACKUP_DIR/minio_$DATE"

# 压缩备份
tar -czf "$BACKUP_DIR/backup_$DATE.tar.gz" "$BACKUP_DIR/postgres_$DATE.sql" "$BACKUP_DIR/minio_$DATE"

# 清理旧备份（保留 30 天）
find $BACKUP_DIR -name "backup_*.tar.gz" -mtime +30 -delete
EOF

chmod +x backup.sh

# 添加到 crontab
echo "0 2 * * * /path/to/backup.sh" | crontab -
```

### 2. 恢复数据

```bash
# PostgreSQL 恢复
docker exec -i postgres psql -U $POSTGRES_USER $POSTGRES_DB < backup_file.sql

# MinIO 数据恢复
docker exec minio mc mirror backup_dir/ /data
```

## 扩容策略

### 1. 水平扩容

```yaml
# HPA 配置
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ai-inference-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ai-inference
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### 2. 集群扩容

```bash
# 添加新节点
kubectl apply -f - << 'EOF'
apiVersion: v1
kind: Node
metadata:
  name: worker-node-2
  labels:
    node-role: worker
    gpu: "true"
spec:
  capacity:
    nvidia.com/gpu: "2"
EOF

# 分配 GPU 节点标签
kubectl label nodes worker-node-2 accelerator=nvidia-tesla-v100
```

## 性能调优

### 1. Rust 服务优化

```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
```

### 2. AI 推理优化

```python
# 批处理优化
BATCH_SIZE = 8
MODEL_PRECISION = "fp16"  # 或 "int8"
ENABLE_TENSORRT = True
```

### 3. 数据库优化

```sql
-- PostgreSQL 配置优化
ALTER SYSTEM SET shared_buffers = '4GB';
ALTER SYSTEM SET effective_cache_size = '12GB';
ALTER SYSTEM SET maintenance_work_mem = '1GB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
```

## 安全配置

### 1. 网络安全

```yaml
# NetworkPolicy
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: fruit-sort-network-policy
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: kong
    ports:
    - protocol: TCP
      port: 8000
```

### 2. RBAC 配置

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  namespace: fruit-sort
  name: fruit-sort-operator
rules:
- apiGroups: [""]
  resources: ["pods", "services"]
  verbs: ["get", "list", "watch"]
```

### 3. 密钥管理

```bash
# 使用 Kubernetes Secrets
kubectl create secret generic database-secret \
  --from-literal=username=postgres \
  --from-literal=password=your-secure-password

# 使用 Vault（可选）
vault kv put secret/fruit-sort/database \
  username=postgres \
  password=your-secure-password
```

## 故障排除

### 1. 常见问题

#### AI 推理服务 GPU 内存不足
```bash
# 检查 GPU 使用情况
nvidia-smi

# 调整批处理大小
export INFERENCE_BATCH_SIZE=2
```

#### 数据库连接池耗尽
```sql
-- 检查连接数
SELECT count(*) FROM pg_stat_activity;

-- 增加连接池大小
ALTER SYSTEM SET max_connections = 200;
```

#### Kafka 消息堆积
```bash
# 检查消费者组状态
kafka-consumer-groups --bootstrap-server kafka:9092 --describe --group ai-inference-group

# 增加分区数
kafka-topics --bootstrap-server kafka:9092 --alter --topic image-events --partitions 6
```

### 2. 日志分析

```bash
# 查看服务日志
kubectl logs -f deployment/ai-inference

# 查看系统资源使用
kubectl top nodes
kubectl top pods

# 查看事件
kubectl get events --sort-by=.metadata.creationTimestamp
```

### 3. 性能分析

```bash
# 使用 hey 进行压力测试
hey -n 1000 -c 50 http://your-domain.com/api/upload

# 分析响应时间分布
curl -w "@curl-format.txt" -o /dev/null http://your-domain.com/api/result/test
```

## 更新和维护

### 1. 滚动更新

```bash
# 更新镜像
kubectl set image deployment/ai-inference ai-inference=your-registry/ai-inference:v2.0

# 查看更新状态
kubectl rollout status deployment/ai-inference

# 回滚更新
kubectl rollout undo deployment/ai-inference
```

### 2. 数据库迁移

```bash
# 运行迁移
docker exec backend-upload-svc sqlx migrate run

# 检查迁移状态
docker exec backend-upload-svc sqlx migrate info
```

通过以上配置和流程，您可以将果蔬智能分拣系统成功部署到生产环境并保证其高可用性、可扩展性和安全性。