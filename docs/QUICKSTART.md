# 快速启动指南

## 修复内容总结

✅ **已修复 Redis 密码配置不一致问题**
✅ **已修复 PostgreSQL 密码配置问题**
✅ **已创建数据库初始化脚本（80条测试数据）**
✅ **已验证端口映射配置**

## 快速开始

### 1. 启动服务（全新初始化）

```bash
# 如果之前运行过，先清理旧数据
docker compose down -v

# 启动所有服务
docker compose up -d

# 查看启动日志
docker compose logs -f
```

### 2. 验证配置

运行自动验证脚本：

```bash
./verify-database.sh
```

该脚本会检查：
- ✓ Docker 容器状态
- ✓ 端口映射
- ✓ PostgreSQL 连接和数据
- ✓ Redis 连接和认证
- ✓ 配置一致性
- ✓ API 接口响应
- ✓ 显示数据示例

### 3. 手动验证（可选）

#### 检查 PostgreSQL
```bash
docker exec -it gyz-postgres psql -U gyz -d gyz -c "SELECT COUNT(*) FROM uploads;"
# 预期输出: 50

docker exec -it gyz-postgres psql -U gyz -d gyz -c "SELECT status, COUNT(*) FROM uploads GROUP BY status;"
```

#### 检查 Redis
```bash
docker exec -it gyz-redis redis-cli -a 6RQgSuJH ping
# 预期输出: PONG
```

#### 测试 API
```bash
# 健康检查
curl http://localhost:8080/api/v1/health

# 获取结果列表
curl http://localhost:8080/api/v1/results?page=1&limit=10

# 获取统计数据
curl http://localhost:8080/api/v1/statistics
```

### 4. 访问应用

- 前端界面: http://localhost:3000
- API 文档: http://localhost:8080/api/v1

## 测试数据说明

### 上传记录（50条）
- **文件名**: test_image_1.jpg ~ test_image_50.jpg
- **状态分布**:
  - 大部分: completed（已完成）
  - 少数: processing（处理中）
  - 少数: failed（失败）
- **时间跨度**: 过去30天内随机分布

### 检测结果
- **水果类型**: 苹果、香蕉、橙子、葡萄、西瓜、草莓、樱桃、梨、桃子、芒果
- **等级分布**: A/B/C 随机分布
- **置信度**: 0.70 - 0.99
- **每张图片**: 5-20个检测结果

### 实时检测记录（30条）
- **数据源**: live_camera, camera_1, camera_2, mobile_app
- **时间跨度**: 过去7天内随机分布
- **每条记录**: 3-10个检测物体

## 配置文件位置

- **Docker Compose**: [docker-compose.yml](docker-compose.yml)
- **环境变量**: [.env](.env)
- **初始化脚本**: [init-db/](init-db/)
  - `01-init-tables.sql` - 创建表和索引
  - `02-seed-test-data.sql` - 插入测试数据
- **详细文档**: [DATABASE-SETUP.md](DATABASE-SETUP.md)

## 服务端口

| 服务 | 主机端口 | 容器端口 |
|------|---------|---------|
| PostgreSQL | 5432 | 5432 |
| Redis | 6379 | 6379 |
| Backend API | 8080 | 8080 |
| Frontend | 3000 | 80 |
| AI Service | 8002 | 8002 |

## 常用命令

```bash
# 查看所有服务状态
docker compose ps

# 查看特定服务日志
docker compose logs postgres
docker compose logs redis
docker compose logs backend

# 重启服务
docker compose restart backend

# 停止所有服务
docker compose down

# 停止并删除数据（重新初始化）
docker compose down -v
```

## 故障排查

### 容器启动失败
```bash
# 查看详细日志
docker compose logs [service-name]

# 检查容器状态
docker compose ps -a
```

### 无法连接数据库
```bash
# 检查网络
docker network ls
docker network inspect guoyan-zhizao_default

# 检查环境变量
docker exec gyz-backend env | grep -E 'DATABASE_URL|REDIS_URL'
```

### 初始化脚本未执行
```bash
# 完全重新初始化
docker compose down -v
docker volume prune -f
docker compose up -d
```

## 下一步

1. ✅ 所有配置已修复并验证
2. ✅ 测试数据已就绪
3. 📝 可以开始前端开发和测试
4. 📝 可以根据需要调整测试数据

更多详细信息请参考 [DATABASE-SETUP.md](DATABASE-SETUP.md)
