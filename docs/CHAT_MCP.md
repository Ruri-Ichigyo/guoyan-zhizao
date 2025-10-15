# AI 聊天助手与 MCP 集成文档

## 概览

果蔬智造平台集成了智能聊天助手功能，通过 LLM（大语言模型）和 MCP（Model Context Protocol）协议，用户可以使用自然语言查询数据库、获取统计信息、分析识别结果。

### 核心特性

- 💬 **自然语言交互**：支持中英文对话，理解用户意图
- 🔧 **工具调用能力**：自动调用数据库查询工具获取实时数据
- 📊 **智能分析**：基于历史数据提供统计分析和见解
- 🔄 **实时响应**：支持长轮询和流式响应
- 💾 **会话管理**：多会话支持，保持上下文连续性
- 🔒 **安全可控**：SQL 查询限制、表级别权限控制

---

## 系统架构

### 组件关系

```
┌─────────────┐
│   前端界面   │ (Vue 3 + ChatWidget)
└──────┬──────┘
       │ REST API
       ↓
┌─────────────────────────────────────┐
│         Rust 后端 (Axum)             │
│  ┌──────────────┐  ┌──────────────┐ │
│  │ Chat Router  │  │ LLM Client   │ │
│  └──────┬───────┘  └──────┬───────┘ │
│         │                 │         │
│         │   ┌─────────────┘         │
│         │   │                       │
│  ┌──────↓───↓───────┐              │
│  │   MCP Client     │              │
│  │  (SQLx Direct)   │              │
│  └──────────┬────────┘              │
└─────────────┼─────────────────────-─┘
              │
         ┌────┴────┐
         ↓         ↓
    PostgreSQL   Redis
  (数据持久化) (会话缓存)
```

### 数据流

1. **用户发送消息** → 前端 ChatWidget
2. **API 请求** → `POST /api/v1/chat/message`
3. **LLM 处理** → LLM Client 调用外部 API
4. **工具决策** → LLM 决定是否需要查询数据库
5. **MCP 查询** → MCP Client 使用 SQLx 执行 SQL
6. **结果整合** → LLM 基于查询结果生成回复
7. **持久化** → 存储到 PostgreSQL
8. **缓存** → 更新 Redis 会话缓存
9. **返回前端** → 显示助手回复

---

## MCP 协议实现

### 什么是 MCP？

Model Context Protocol（模型上下文协议）是一个标准化的接口，允许 LLM 安全地访问外部数据源和工具。在本项目中，MCP 用于：

- 执行数据库查询
- 限制查询范围和权限
- 格式化查询结果供 LLM 理解

### 实现方式

本项目采用**内置集成**方式，直接在 Rust 后端实现 MCP 协议，而非独立部署 MCP 服务器。

**优势**：
- ✅ 无需额外部署和维护独立服务
- ✅ 复用现有数据库连接池，性能更好
- ✅ 统一的错误处理和日志记录
- ✅ 更简单的权限控制和安全策略

**代码位置**：`backend/src/services/mcp_client.rs`

### 支持的工具

#### 1. database_query

执行只读 SQL 查询，返回结构化数据。

**参数**：
```json
{
  "query": "SELECT COUNT(*) as total FROM uploads WHERE status = 'completed'"
}
```

**返回示例**：
```json
{
  "success": true,
  "rows": [
    {"total": 82}
  ],
  "row_count": 1
}
```

**安全限制**：
- 仅允许查询白名单表：`uploads`, `results`, `detections`, `chat_sessions`, `chat_messages`, `users`
- 禁止 `INSERT`, `UPDATE`, `DELETE`, `DROP`, `ALTER` 等修改操作
- 最大返回行数：1000
- 查询超时：5 秒

---

## LLM 集成

### 支持的 LLM 提供商

目前支持以下 LLM API：

1. **SiliconFlow**（默认）
   - API: `https://api.siliconflow.cn/v1/messages`
   - 模型: `Qwen/Qwen3-Coder-30B-A3B-Instruct`, `tencent/Hunyuan-MT-7B` 等

2. **Anthropic Claude**（可配置）
   - API: `https://api.anthropic.com/v1/messages`
   - 模型: `claude-3-5-sonnet-20241022` 等

3. **其他兼容 Claude API 的服务**
   - 需要实现相同的消息格式和工具调用协议

### 配置 LLM

在 `.env` 或 `docker-compose.yml` 中配置：

```bash
# LLM API 配置
LLM_API_URL=https://api.siliconflow.cn/v1/messages
LLM_API_KEY=your-api-key-here
LLM_MODEL=Qwen/Qwen3-Coder-30B-A3B-Instruct
LLM_MAX_TOKENS=12800
```

### 工具调用流程

1. **用户消息**：
   ```json
   {
     "content": "今天上传了多少张图片？"
   }
   ```

2. **LLM 分析**：识别需要查询数据库

3. **工具调用请求**：
   ```json
   {
     "id": "tool_1",
     "name": "database_query",
     "input": {
       "query": "SELECT COUNT(*) as count FROM uploads WHERE DATE(created_at) = CURRENT_DATE"
     }
   }
   ```

4. **MCP 执行**：返回查询结果
   ```json
   {
     "success": true,
     "rows": [{"count": 15}]
   }
   ```

5. **LLM 生成回复**：
   ```
   根据数据库查询，今天总共上传了 15 张图片。
   ```

---

## 数据库表结构

### chat_sessions（聊天会话）

```sql
CREATE TABLE chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT,
    status TEXT NOT NULL DEFAULT 'active' 
        CHECK (status IN ('active', 'inactive', 'archived')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**索引**：
- `idx_chat_sessions_user_id`：按用户查询
- `idx_chat_sessions_status`：按状态过滤
- `idx_chat_sessions_updated_at`：按更新时间排序

### chat_messages（聊天消息）

```sql
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL CHECK (char_length(content) <= 10000),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**metadata 字段示例**：
```json
{
  "tool_calls": [
    {
      "tool": "database_query",
      "input": {"query": "..."},
      "output": {"rows": [...], "row_count": 5},
      "success": true,
      "duration_ms": 45
    }
  ],
  "model": "Qwen/Qwen3-Coder-30B-A3B-Instruct",
  "tokens": {"input": 120, "output": 85}
}
```

---

## API 接口详解

### 创建会话

```http
POST /api/v1/chat/sessions
Content-Type: application/json

{
  "title": "数据查询助手"
}
```

**响应**：
```json
{
  "session_id": "uuid",
  "title": "数据查询助手",
  "status": "active",
  "created_at": "2025-10-15T10:00:00Z"
}
```

### 发送消息

```http
POST /api/v1/chat/message
Content-Type: application/json

{
  "session_id": "uuid",
  "content": "最近一小时检测了多少个苹果？"
}
```

**响应**：
```json
{
  "message_id": "uuid",
  "role": "assistant",
  "content": "根据数据库查询，最近一小时共检测到 25 个苹果，其中 A 级 15 个，B 级 8 个，C 级 2 个。",
  "tool_calls": [
    {
      "tool": "database_query",
      "input": {...},
      "output": {...},
      "success": true
    }
  ],
  "created_at": "2025-10-15T10:01:00Z"
}
```

### 获取历史

```http
GET /api/v1/chat/history/{session_id}?limit=20
```

**响应**：
```json
{
  "session_id": "uuid",
  "messages": [
    {
      "id": "uuid",
      "role": "user",
      "content": "...",
      "created_at": "..."
    },
    {
      "id": "uuid",
      "role": "assistant",
      "content": "...",
      "created_at": "..."
    }
  ],
  "total": 10
}
```

### 轮询新消息

```http
GET /api/v1/chat/messages/new?session_id={uuid}&since=2025-10-15T10:00:00Z
```

**响应**：
```json
{
  "messages": [
    {
      "id": "uuid",
      "role": "assistant",
      "content": "...",
      "created_at": "..."
    }
  ],
  "has_more": false
}
```

---

## Redis 缓存策略

### 活跃会话缓存

- **键**：`active_session:<user_id>`
- **值**：当前活跃会话的 UUID
- **TTL**：1 小时
- **用途**：快速获取用户当前会话，避免重复数据库查询

### 会话历史缓存

- **键**：`conversation:<session_id>`
- **值**：List 类型，存储最近 20 条消息
- **TTL**：30 分钟
- **用途**：提供给 LLM 的上下文历史，加速对话响应

**缓存项结构**：
```json
{
  "role": "user|assistant",
  "content": "消息内容",
  "timestamp": "ISO 8601"
}
```

**更新策略**：
- 发送消息时追加到列表
- 保持最近 20 条（LTRIM）
- 自动刷新 TTL

---

## 使用示例

### 示例 1：查询上传统计

**用户**：今天上传了多少张图片？

**系统执行**：
1. LLM 识别需要查询数据库
2. 生成 SQL：`SELECT COUNT(*) FROM uploads WHERE DATE(created_at) = CURRENT_DATE`
3. MCP 执行查询
4. LLM 整合结果

**助手**：今天总共上传了 15 张图片。

### 示例 2：质量分析

**用户**：最近一周 A 级水果的比例是多少？

**系统执行**：
```sql
SELECT 
  SUM(grade_a) as a_grade,
  SUM(total_fruits) as total
FROM results
WHERE created_at > NOW() - INTERVAL '7 days'
```

**助手**：最近一周共检测 1,245 个水果，其中 A 级 856 个，占比约 68.7%。

### 示例 3：类别统计

**用户**：哪种水果检测最多？

**系统执行**：
```sql
SELECT 
  jsonb_array_elements(detections)->>'class_name' as class_name,
  COUNT(*) as count
FROM results
GROUP BY class_name
ORDER BY count DESC
LIMIT 5
```

**助手**：检测最多的水果依次是：
1. Apple Red 1 - 320 次
2. Banana - 285 次
3. Orange - 201 次
4. Grape Blue - 178 次
5. Strawberry - 156 次

---

## 安全与限制

### SQL 注入防护

- ✅ 使用 SQLx 预编译语句
- ✅ 参数化查询
- ✅ 禁止动态拼接 SQL
- ✅ 关键字黑名单检查

### 权限控制

- ✅ 表级白名单：仅允许访问指定表
- ✅ 操作限制：仅支持 SELECT 查询
- ✅ 行数限制：最多返回 1000 行
- ✅ 查询超时：5 秒自动中断

### 性能优化

- ✅ Redis 缓存热点查询
- ✅ 数据库连接池复用
- ✅ 异步非阻塞处理
- ✅ 查询结果流式返回

### 成本控制

- ✅ LLM 令牌限制：单次最多 12,800 tokens
- ✅ 消息长度限制：10,000 字符
- ✅ 会话超时：30 分钟无活动自动归档
- ✅ 历史消息限制：仅缓存最近 20 条

---

## 故障排查

### 常见问题

#### 1. 聊天无响应

**检查项**：
- ✓ `LLM_API_KEY` 是否正确配置
- ✓ LLM API 服务是否可访问
- ✓ 网络连接是否正常
- ✓ 后端日志是否有错误

**解决方法**：
```bash
# 查看后端日志
docker logs gyz-backend

# 测试 LLM API
curl -X POST https://api.siliconflow.cn/v1/messages \
  -H "Authorization: Bearer $LLM_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{...}'
```

#### 2. 数据库查询失败

**检查项**：
- ✓ SQL 语法是否正确
- ✓ 表名是否在白名单中
- ✓ 是否使用了禁止的关键字
- ✓ 查询是否超时

**解决方法**：
```bash
# 查看 MCP 日志
docker logs gyz-backend | grep "MCP"

# 手动测试 SQL
docker exec -it gyz-postgres psql -U gyz -d gyz -c "YOUR_SQL"
```

#### 3. 会话丢失

**检查项**：
- ✓ Redis 是否正常运行
- ✓ 缓存是否过期
- ✓ Session ID 是否有效

**解决方法**：
```bash
# 检查 Redis
docker exec -it gyz-redis redis-cli -a 6RQgSuJH ping

# 查看会话缓存
docker exec -it gyz-redis redis-cli -a 6RQgSuJH KEYS "active_session:*"
```

---

## 性能指标

### 响应时间

- **无工具调用**：< 1 秒
- **单次数据库查询**：< 2 秒
- **多次工具调用**：< 5 秒

### 缓存命中率

- **会话缓存**：> 80%
- **历史消息**：> 90%

### 并发能力

- **最大并发会话**：100+
- **单会话 QPS**：10+
- **数据库连接池**：20 连接

---

## 开发指南

### 添加新工具

1. 在 `backend/src/services/mcp_client.rs` 中定义工具：

```rust
pub async fn execute_tool(&self, tool_name: &str, input: Value) -> Result<Value> {
    match tool_name {
        "database_query" => self.query_database(input).await,
        "your_new_tool" => self.your_new_tool_impl(input).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}
```

2. 在 `backend/src/services/llm_client.rs` 中注册工具：

```rust
"tools": [
    {
        "name": "your_new_tool",
        "description": "工具描述",
        "input_schema": {
            "type": "object",
            "properties": {...}
        }
    }
]
```

### 自定义提示词

修改 `backend/src/routes/chat.rs` 中的系统提示：

```rust
let system_prompt = r#"
你是一个果蔬识别系统的智能助手...
你可以使用以下工具：
1. database_query - 查询数据库
2. your_new_tool - 你的新工具
"#;
```

---

## 最佳实践

### 用户体验

1. **提供示例问题**：在界面显示常见查询示例
2. **显示思考过程**：展示 LLM 的工具调用过程
3. **快速响应**：使用缓存和流式响应
4. **错误友好**：提供清晰的错误提示和建议

### 性能优化

1. **预热缓存**：启动时加载常用查询
2. **批量处理**：合并多个查询请求
3. **异步处理**：使用 tokio 异步运行时
4. **连接池**：复用数据库连接

### 安全建议

1. **最小权限**：MCP 仅授予只读权限
2. **审计日志**：记录所有工具调用
3. **输入验证**：检查用户输入长度和内容
4. **速率限制**：防止滥用 API

---

## 未来规划

- [ ] 支持更多工具（文件上传、图像分析等）
- [ ] 多模态对话（图片+文本）
- [ ] 语音输入/输出
- [ ] 知识库集成（RAG）
- [ ] 多语言支持优化
- [ ] 自定义工具市场

---

**最后更新**：2025-10-15  
**维护团队**：果蔬智造团队
