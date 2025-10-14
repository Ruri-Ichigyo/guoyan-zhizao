#!/bin/bash

# 果蔬智造数据库验证脚本
# 用于验证数据库结构和数据完整性

set -e

echo "========================================="
echo "果蔬智造数据库验证脚本"
echo "========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查函数
check_passed() {
    echo -e "${GREEN}✓${NC} $1"
}

check_failed() {
    echo -e "${RED}✗${NC} $1"
}

check_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. 检查PostgreSQL容器状态
echo "1. 检查PostgreSQL容器状态..."
if docker compose ps postgres | grep -q "healthy"; then
    check_passed "PostgreSQL容器运行正常"
else
    check_failed "PostgreSQL容器未运行或不健康"
    exit 1
fi
echo ""

# 2. 检查数据库表
echo "2. 检查数据库表结构..."
TABLES=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;")
EXPECTED_TABLES=("chat_messages" "chat_sessions" "detections" "results" "uploads" "users")

for table in "${EXPECTED_TABLES[@]}"; do
    if echo "$TABLES" | grep -q "$table"; then
        check_passed "表 $table 存在"
    else
        check_failed "表 $table 不存在"
    fi
done
echo ""

# 3. 检查MCP只读用户
echo "3. 检查MCP只读用户..."
if docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT rolname FROM pg_roles WHERE rolname = 'mcp_readonly';" | grep -q "mcp_readonly"; then
    check_passed "MCP只读用户存在"

    # 检查权限
    PRIVILEGES=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT table_name FROM information_schema.table_privileges WHERE grantee = 'mcp_readonly' AND privilege_type = 'SELECT';")

    for table in "${EXPECTED_TABLES[@]}"; do
        if echo "$PRIVILEGES" | grep -q "$table"; then
            check_passed "MCP用户对表 $table 有SELECT权限"
        else
            check_failed "MCP用户对表 $table 无SELECT权限"
        fi
    done
else
    check_failed "MCP只读用户不存在"
fi
echo ""

# 4. 检查数据完整性
echo "4. 检查数据完整性..."

# 检查用户数据
USER_COUNT=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM users;")
if [ "$USER_COUNT" -gt 0 ]; then
    check_passed "用户表有 $USER_COUNT 条记录"
else
    check_warning "用户表没有数据"
fi

# 检查上传记录
UPLOAD_COUNT=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM uploads;")
if [ "$UPLOAD_COUNT" -gt 0 ]; then
    check_passed "上传表有 $UPLOAD_COUNT 条记录"
else
    check_warning "上传表没有数据"
fi

# 检查检测结果
RESULT_COUNT=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM results;")
if [ "$RESULT_COUNT" -gt 0 ]; then
    check_passed "结果表有 $RESULT_COUNT 条记录"
else
    check_warning "结果表没有数据"
fi

# 检查实时检测
DETECTION_COUNT=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM detections;")
if [ "$DETECTION_COUNT" -gt 0 ]; then
    check_passed "检测表有 $DETECTION_COUNT 条记录"
else
    check_warning "检测表没有数据"
fi

# 检查聊天会话
SESSION_COUNT=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM chat_sessions;")
if [ "$SESSION_COUNT" -gt 0 ]; then
    check_passed "聊天会话表有 $SESSION_COUNT 条记录"
else
    check_warning "聊天会话表没有数据"
fi

# 检查聊天消息
MESSAGE_COUNT=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM chat_messages;")
if [ "$MESSAGE_COUNT" -gt 0 ]; then
    check_passed "聊天消息表有 $MESSAGE_COUNT 条记录"
else
    check_warning "聊天消息表没有数据"
fi
echo ""

# 5. 检查外键约束
echo "5. 检查外键约束..."
FK_COUNT=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM information_schema.table_constraints WHERE constraint_type = 'FOREIGN KEY';")
if [ "$FK_COUNT" -gt 0 ]; then
    check_passed "数据库有 $FK_COUNT 个外键约束"
else
    check_warning "数据库没有外键约束"
fi
echo ""

# 6. 测试MCP用户查询
echo "6. 测试MCP用户查询能力..."
if docker compose exec -T postgres psql -U mcp_readonly -d gyz -t -c "SELECT COUNT(*) FROM users;" >/dev/null 2>&1; then
    check_passed "MCP用户可以查询users表"
else
    check_failed "MCP用户无法查询users表"
fi

if docker compose exec -T postgres psql -U mcp_readonly -d gyz -t -c "SELECT u.username, COUNT(up.id) FROM users u LEFT JOIN uploads up ON u.id = up.user_id GROUP BY u.username;" >/dev/null 2>&1; then
    check_passed "MCP用户可以执行联表查询"
else
    check_failed "MCP用户无法执行联表查询"
fi

# 测试MCP用户是否可以写入（应该失败）
if docker compose exec -T postgres psql -U mcp_readonly -d gyz -t -c "INSERT INTO users (username, email, password_hash, role) VALUES ('test', 'test@test.com', 'hash', 'user');" 2>&1 | grep -q "permission denied"; then
    check_passed "MCP用户正确地无法写入数据"
else
    check_warning "MCP用户可能有写入权限（安全风险）"
fi
echo ""

# 7. 数据库统计摘要
echo "========================================="
echo "数据库统计摘要"
echo "========================================="
docker compose exec -T postgres psql -U gyz -d gyz -c "
SELECT
    '用户数' as 类别, COUNT(*)::text as 数量 FROM users
UNION ALL
SELECT '上传记录', COUNT(*)::text FROM uploads
UNION ALL
SELECT '检测结果', COUNT(*)::text FROM results
UNION ALL
SELECT '实时检测', COUNT(*)::text FROM detections
UNION ALL
SELECT '聊天会话', COUNT(*)::text FROM chat_sessions
UNION ALL
SELECT '聊天消息', COUNT(*)::text FROM chat_messages;
"

echo ""
echo "========================================="
echo "验证完成！"
echo "========================================="
