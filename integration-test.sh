#!/bin/bash

# 果蔬智造集成测试脚本
# 测试数据库、后端服务和MCP集成

set -e

echo "========================================="
echo "果蔬智造集成测试"
echo "========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试计数
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 测试函数
test_passed() {
    echo -e "${GREEN}✓ PASS${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

test_failed() {
    echo -e "${RED}✗ FAIL${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

test_info() {
    echo -e "${BLUE}ℹ INFO${NC} $1"
}

# 1. 数据库集成测试
echo "========================================="
echo "1. 数据库集成测试"
echo "========================================="
echo ""

# 1.1 测试基本查询
test_info "测试基本SELECT查询..."
if docker compose exec -T postgres psql -U gyz -d gyz -c "SELECT * FROM users LIMIT 1;" >/dev/null 2>&1; then
    test_passed "基本SELECT查询成功"
else
    test_failed "基本SELECT查询失败"
fi

# 1.2 测试JOIN查询
test_info "测试JOIN查询..."
if docker compose exec -T postgres psql -U gyz -d gyz -c "SELECT u.username, COUNT(up.id) FROM users u LEFT JOIN uploads up ON u.id = up.user_id GROUP BY u.username;" >/dev/null 2>&1; then
    test_passed "JOIN查询成功"
else
    test_failed "JOIN查询失败"
fi

# 1.3 测试聚合查询
test_info "测试聚合查询..."
TOTAL_USERS=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM users;")
if [ "$TOTAL_USERS" -gt 0 ]; then
    test_passed "聚合查询成功 (用户数: $TOTAL_USERS)"
else
    test_failed "聚合查询失败"
fi

# 1.4 测试外键完整性
test_info "测试外键完整性..."
if docker compose exec -T postgres psql -U gyz -d gyz -c "SELECT cs.id, cs.user_id, u.username FROM chat_sessions cs JOIN users u ON cs.user_id = u.id LIMIT 1;" >/dev/null 2>&1; then
    test_passed "外键完整性验证通过"
else
    test_failed "外键完整性验证失败"
fi
echo ""

# 2. MCP用户权限测试
echo "========================================="
echo "2. MCP用户权限测试"
echo "========================================="
echo ""

# 2.1 测试读权限
test_info "测试MCP用户读权限..."
TABLES=("users" "uploads" "results" "detections" "chat_sessions" "chat_messages")
for table in "${TABLES[@]}"; do
    if docker compose exec -T postgres psql -U mcp_readonly -d gyz -c "SELECT COUNT(*) FROM $table;" >/dev/null 2>&1; then
        test_passed "MCP可读取表: $table"
    else
        test_failed "MCP无法读取表: $table"
    fi
done

# 2.2 测试写权限（应该被拒绝）
test_info "测试MCP用户写权限限制..."
if docker compose exec -T postgres psql -U mcp_readonly -d gyz -c "DELETE FROM users WHERE id = '00000000-0000-0000-0000-000000000000';" 2>&1 | grep -q "permission denied"; then
    test_passed "MCP用户DELETE操作被正确拒绝"
else
    test_failed "MCP用户可能有不当的写权限"
fi

if docker compose exec -T postgres psql -U mcp_readonly -d gyz -c "UPDATE users SET username = 'hacked' WHERE 1=1;" 2>&1 | grep -q "permission denied"; then
    test_passed "MCP用户UPDATE操作被正确拒绝"
else
    test_failed "MCP用户可能有不当的写权限"
fi
echo ""

# 3. 数据关系测试
echo "========================================="
echo "3. 数据关系完整性测试"
echo "========================================="
echo ""

# 3.1 测试用户与上传的关系
test_info "测试用户与上传记录的关系..."
ORPHAN_UPLOADS=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM uploads WHERE user_id IS NOT NULL AND user_id NOT IN (SELECT id FROM users);")
if [ "$ORPHAN_UPLOADS" -eq 0 ]; then
    test_passed "所有上传记录都有有效的用户关联"
else
    test_failed "发现 $ORPHAN_UPLOADS 条孤立的上传记录"
fi

# 3.2 测试上传与结果的关系
test_info "测试上传与检测结果的关系..."
ORPHAN_RESULTS=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM results WHERE upload_id NOT IN (SELECT id FROM uploads);")
if [ "$ORPHAN_RESULTS" -eq 0 ]; then
    test_passed "所有检测结果都有有效的上传记录关联"
else
    test_failed "发现 $ORPHAN_RESULTS 条孤立的检测结果"
fi

# 3.3 测试聊天会话与用户的关系
test_info "测试聊天会话与用户的关系..."
ORPHAN_SESSIONS=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM chat_sessions WHERE user_id NOT IN (SELECT id FROM users);")
if [ "$ORPHAN_SESSIONS" -eq 0 ]; then
    test_passed "所有聊天会话都有有效的用户关联"
else
    test_failed "发现 $ORPHAN_SESSIONS 条孤立的聊天会话"
fi

# 3.4 测试聊天消息与会话的关系
test_info "测试聊天消息与会话的关系..."
ORPHAN_MESSAGES=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM chat_messages WHERE session_id NOT IN (SELECT id FROM chat_sessions);")
if [ "$ORPHAN_MESSAGES" -eq 0 ]; then
    test_passed "所有聊天消息都有有效的会话关联"
else
    test_failed "发现 $ORPHAN_MESSAGES 条孤立的聊天消息"
fi
echo ""

# 4. 复杂查询测试
echo "========================================="
echo "4. 复杂查询测试"
echo "========================================="
echo ""

# 4.1 测试用户统计查询
test_info "测试用户统计查询..."
if docker compose exec -T postgres psql -U mcp_readonly -d gyz -c "
    SELECT
        u.username,
        COUNT(DISTINCT up.id) as upload_count,
        COUNT(DISTINCT cs.id) as session_count,
        COUNT(DISTINCT d.id) as detection_count
    FROM users u
    LEFT JOIN uploads up ON u.id = up.user_id
    LEFT JOIN chat_sessions cs ON u.id = cs.user_id
    LEFT JOIN detections d ON u.id = d.user_id
    GROUP BY u.id, u.username;
" >/dev/null 2>&1; then
    test_passed "用户统计查询成功"
else
    test_failed "用户统计查询失败"
fi

# 4.2 测试检测结果聚合查询
test_info "测试检测结果聚合查询..."
if docker compose exec -T postgres psql -U mcp_readonly -d gyz -c "
    SELECT
        DATE_TRUNC('day', created_at) as date,
        SUM(total_fruits) as total_fruits,
        SUM(grade_a) as grade_a,
        SUM(grade_b) as grade_b,
        SUM(grade_c) as grade_c,
        AVG(avg_confidence) as avg_confidence
    FROM results
    GROUP BY DATE_TRUNC('day', created_at)
    ORDER BY date DESC
    LIMIT 7;
" >/dev/null 2>&1; then
    test_passed "检测结果聚合查询成功"
else
    test_failed "检测结果聚合查询失败"
fi

# 4.3 测试聊天历史查询
test_info "测试聊天历史查询..."
if docker compose exec -T postgres psql -U mcp_readonly -d gyz -c "
    SELECT
        cs.id,
        cs.title,
        u.username,
        COUNT(cm.id) as message_count,
        MAX(cm.created_at) as last_message_at
    FROM chat_sessions cs
    JOIN users u ON cs.user_id = u.id
    LEFT JOIN chat_messages cm ON cs.id = cm.session_id
    GROUP BY cs.id, cs.title, u.username
    ORDER BY last_message_at DESC;
" >/dev/null 2>&1; then
    test_passed "聊天历史查询成功"
else
    test_failed "聊天历史查询失败"
fi
echo ""

# 5. 性能测试
echo "========================================="
echo "5. 性能测试"
echo "========================================="
echo ""

# 5.1 测试索引存在性
test_info "检查关键索引..."
INDEX_COUNT=$(docker compose exec -T postgres psql -U gyz -d gyz -t -c "SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public';")
if [ "$INDEX_COUNT" -gt 10 ]; then
    test_passed "数据库有 $INDEX_COUNT 个索引"
else
    test_failed "索引数量不足: $INDEX_COUNT"
fi

# 5.2 测试查询性能（简单）
test_info "测试查询性能..."
START_TIME=$(date +%s%N)
docker compose exec -T postgres psql -U mcp_readonly -d gyz -c "SELECT * FROM uploads ORDER BY created_at DESC LIMIT 100;" >/dev/null 2>&1
END_TIME=$(date +%s%N)
DURATION=$(( (END_TIME - START_TIME) / 1000000 ))
if [ "$DURATION" -lt 1000 ]; then
    test_passed "查询性能良好 (${DURATION}ms)"
else
    test_failed "查询性能较慢 (${DURATION}ms)"
fi
echo ""

# 测试总结
echo "========================================="
echo "测试总结"
echo "========================================="
echo -e "总测试数: ${BLUE}$TOTAL_TESTS${NC}"
echo -e "通过: ${GREEN}$PASSED_TESTS${NC}"
echo -e "失败: ${RED}$FAILED_TESTS${NC}"
echo ""

if [ "$FAILED_TESTS" -eq 0 ]; then
    echo -e "${GREEN}所有测试通过！✓${NC}"
    exit 0
else
    echo -e "${RED}部分测试失败！✗${NC}"
    exit 1
fi
