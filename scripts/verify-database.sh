#!/bin/bash

# 数据库配置验证脚本
# 用于检查 Redis 和 PostgreSQL 是否正确配置并可以连接

set -e

echo "================================"
echo "  数据库配置验证脚本"
echo "================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 加载 .env 文件
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
    echo -e "${GREEN}✓${NC} 已加载 .env 配置文件"
else
    echo -e "${RED}✗${NC} 未找到 .env 文件"
    exit 1
fi

echo ""
echo "=== 1. 检查 Docker 容器状态 ==="
echo ""

# 检查容器是否运行
POSTGRES_RUNNING=$(docker ps --filter "name=gyz-postgres" --filter "status=running" -q)
REDIS_RUNNING=$(docker ps --filter "name=gyz-redis" --filter "status=running" -q)
BACKEND_RUNNING=$(docker ps --filter "name=gyz-backend" --filter "status=running" -q)

if [ -z "$POSTGRES_RUNNING" ]; then
    echo -e "${RED}✗${NC} PostgreSQL 容器未运行"
    exit 1
else
    echo -e "${GREEN}✓${NC} PostgreSQL 容器正在运行"
fi

if [ -z "$REDIS_RUNNING" ]; then
    echo -e "${RED}✗${NC} Redis 容器未运行"
    exit 1
else
    echo -e "${GREEN}✓${NC} Redis 容器正在运行"
fi

if [ -z "$BACKEND_RUNNING" ]; then
    echo -e "${YELLOW}!${NC} Backend 容器未运行（可能正在启动）"
else
    echo -e "${GREEN}✓${NC} Backend 容器正在运行"
fi

echo ""
echo "=== 2. 验证端口映射 ==="
echo ""

# 检查端口是否被监听
if command -v netstat &> /dev/null; then
    if netstat -tuln | grep -q ":5432 "; then
        echo -e "${GREEN}✓${NC} PostgreSQL 端口 5432 已开放"
    else
        echo -e "${RED}✗${NC} PostgreSQL 端口 5432 未开放"
    fi

    if netstat -tuln | grep -q ":6379 "; then
        echo -e "${GREEN}✓${NC} Redis 端口 6379 已开放"
    else
        echo -e "${RED}✗${NC} Redis 端口 6379 未开放"
    fi

    if netstat -tuln | grep -q ":8080 "; then
        echo -e "${GREEN}✓${NC} Backend 端口 8080 已开放"
    else
        echo -e "${YELLOW}!${NC} Backend 端口 8080 未开放（可能正在启动）"
    fi
else
    echo -e "${YELLOW}!${NC} netstat 未安装，跳过端口检查"
fi

echo ""
echo "=== 3. 验证 PostgreSQL 连接 ==="
echo ""

# 检查 PostgreSQL 是否可以连接
if docker exec gyz-postgres pg_isready -U ${POSTGRES_USER} -d ${POSTGRES_DB} &> /dev/null; then
    echo -e "${GREEN}✓${NC} PostgreSQL 数据库可以连接"
else
    echo -e "${RED}✗${NC} PostgreSQL 数据库连接失败"
    exit 1
fi

# 检查表是否存在
TABLES=$(docker exec gyz-postgres psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_name IN ('uploads', 'results', 'detections');" 2>/dev/null | xargs)

if [ "$TABLES" = "3" ]; then
    echo -e "${GREEN}✓${NC} 数据库表已创建（uploads, results, detections）"
else
    echo -e "${YELLOW}!${NC} 数据库表未完全创建（可能正在初始化）"
fi

# 检查测试数据
UPLOAD_COUNT=$(docker exec gyz-postgres psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -t -c "SELECT COUNT(*) FROM uploads;" 2>/dev/null | xargs || echo "0")
RESULT_COUNT=$(docker exec gyz-postgres psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -t -c "SELECT COUNT(*) FROM results;" 2>/dev/null | xargs || echo "0")
DETECTION_COUNT=$(docker exec gyz-postgres psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -t -c "SELECT COUNT(*) FROM detections;" 2>/dev/null | xargs || echo "0")

echo ""
echo "数据统计:"
echo "  - 上传记录: ${UPLOAD_COUNT} 条"
echo "  - 检测结果: ${RESULT_COUNT} 条"
echo "  - 实时检测: ${DETECTION_COUNT} 条"

if [ "$UPLOAD_COUNT" -ge "50" ]; then
    echo -e "${GREEN}✓${NC} 测试数据已成功导入"
else
    echo -e "${YELLOW}!${NC} 测试数据未完全导入（预期至少50条上传记录）"
fi

echo ""
echo "=== 4. 验证 Redis 连接 ==="
echo ""

# 检查 Redis 是否可以连接
if docker exec gyz-redis redis-cli -a ${REDIS_PASSWORD} ping 2>/dev/null | grep -q "PONG"; then
    echo -e "${GREEN}✓${NC} Redis 服务可以连接"
    echo -e "${GREEN}✓${NC} Redis 密码认证成功"
else
    echo -e "${RED}✗${NC} Redis 连接或认证失败"
    exit 1
fi

# 测试 Redis 基本操作
docker exec gyz-redis redis-cli -a ${REDIS_PASSWORD} SET test_key "test_value" &> /dev/null
TEST_VALUE=$(docker exec gyz-redis redis-cli -a ${REDIS_PASSWORD} GET test_key 2>/dev/null)
docker exec gyz-redis redis-cli -a ${REDIS_PASSWORD} DEL test_key &> /dev/null

if [ "$TEST_VALUE" = "test_value" ]; then
    echo -e "${GREEN}✓${NC} Redis 读写操作正常"
else
    echo -e "${RED}✗${NC} Redis 读写操作失败"
fi

echo ""
echo "=== 5. 验证配置一致性 ==="
echo ""

# 检查 backend 的环境变量
BACKEND_DB_URL=$(docker exec gyz-backend env 2>/dev/null | grep DATABASE_URL | cut -d'=' -f2 || echo "")
BACKEND_REDIS_URL=$(docker exec gyz-backend env 2>/dev/null | grep REDIS_URL | cut -d'=' -f2 || echo "")

EXPECTED_DB_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/${POSTGRES_DB}"
EXPECTED_REDIS_URL="redis://:${REDIS_PASSWORD}@redis:6379"

if [ "$BACKEND_DB_URL" = "$EXPECTED_DB_URL" ]; then
    echo -e "${GREEN}✓${NC} Backend DATABASE_URL 配置正确"
else
    echo -e "${YELLOW}!${NC} Backend DATABASE_URL 配置:"
    echo "    实际: $BACKEND_DB_URL"
    echo "    预期: $EXPECTED_DB_URL"
fi

if [ "$BACKEND_REDIS_URL" = "$EXPECTED_REDIS_URL" ]; then
    echo -e "${GREEN}✓${NC} Backend REDIS_URL 配置正确"
else
    echo -e "${YELLOW}!${NC} Backend REDIS_URL 配置:"
    echo "    实际: $BACKEND_REDIS_URL"
    echo "    预期: $EXPECTED_REDIS_URL"
fi

echo ""
echo "=== 6. API 接口测试 ==="
echo ""

# 等待 backend 启动
if [ -n "$BACKEND_RUNNING" ]; then
    sleep 2

    # 测试健康检查接口
    if curl -s http://localhost:8080/api/v1/health &> /dev/null; then
        echo -e "${GREEN}✓${NC} Backend 健康检查接口响应正常"

        # 测试结果列表接口
        RESULTS=$(curl -s http://localhost:8080/api/v1/results?page=1&limit=1)
        if echo "$RESULTS" | grep -q "data"; then
            echo -e "${GREEN}✓${NC} Results API 响应正常"
        else
            echo -e "${YELLOW}!${NC} Results API 响应异常"
        fi

        # 测试统计接口
        STATS=$(curl -s http://localhost:8080/api/v1/statistics)
        if echo "$STATS" | grep -q "totals"; then
            echo -e "${GREEN}✓${NC} Statistics API 响应正常"
        else
            echo -e "${YELLOW}!${NC} Statistics API 响应异常"
        fi
    else
        echo -e "${YELLOW}!${NC} Backend API 暂时无法访问（可能正在启动）"
    fi
else
    echo -e "${YELLOW}!${NC} Backend 未运行，跳过 API 测试"
fi

echo ""
echo "=== 7. 数据示例查询 ==="
echo ""

echo "最近的检测结果:"
docker exec gyz-postgres psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -c "
SELECT
    u.filename,
    r.status,
    r.total_fruits,
    r.grade_a,
    r.grade_b,
    r.grade_c,
    ROUND(r.avg_confidence::numeric, 2) as avg_conf,
    TO_CHAR(r.created_at, 'YYYY-MM-DD HH24:MI') as created
FROM uploads u
JOIN results r ON u.id = r.upload_id
ORDER BY r.created_at DESC
LIMIT 5;
" 2>/dev/null || echo "查询失败"

echo ""
echo "状态统计:"
docker exec gyz-postgres psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -c "
SELECT
    status,
    COUNT(*) as count
FROM uploads
GROUP BY status
ORDER BY count DESC;
" 2>/dev/null || echo "查询失败"

echo ""
echo "================================"
echo "  验证完成！"
echo "================================"
echo ""
echo "访问地址:"
echo "  - 前端: http://localhost:3000"
echo "  - API:  http://localhost:8080/api/v1"
echo ""
echo "数据库连接信息:"
echo "  - PostgreSQL: localhost:5432 (用户: ${POSTGRES_USER}, 密码: ${POSTGRES_PASSWORD})"
echo "  - Redis:      localhost:6379 (密码: ${REDIS_PASSWORD})"
echo ""
