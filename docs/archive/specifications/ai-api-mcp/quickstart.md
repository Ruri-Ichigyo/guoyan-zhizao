# Quickstart Guide: AI Assistant Chat Module

**Feature**: AI Assistant Chat Module
**Audience**: Developers implementing or integrating with the chat feature
**Estimated Setup Time**: 30-45 minutes

---

## Overview

This guide walks you through setting up and testing the AI Assistant Chat Module locally. By the end, you'll have a working chat interface that can query your fruit/vegetable detection database using natural language.

### What You'll Build

```
User asks: "上周有多少个A级水果?"
    ↓
Frontend sends message to backend
    ↓
Backend sends to LLM (SiliconFlow/Hunyuan-MT-7B)
    ↓
LLM uses MCP to query database
    ↓
Backend returns: "根据数据库查询,上周您识别了42个A级水果。"
```

---

## Prerequisites

Before starting, ensure you have:

- ✅ **Node.js 20+** (for MCP server)
- ✅ **Rust 1.75+** with cargo
- ✅ **PostgreSQL 14+** running locally
- ✅ **Redis 7+** running locally
- ✅ **SiliconFlow API Key** (provided: `sk-qqkhvtehvscrwenhzixflzzmmdgzmzaufwurejxgnfbrxcjo`)
- ✅ **Existing user authentication** (JWT tokens working)

### Verify Prerequisites

```bash
# Check versions
node --version    # Should be v20.x or higher
cargo --version   # Should be 1.75 or higher
psql --version    # Should be 14.x or higher
redis-cli ping    # Should return PONG

# Check services
psql -h localhost -U gyz -d gyz -c "SELECT 1;"  # Test DB connection
redis-cli ping                                   # Test Redis
```

---

## Step 1: Database Setup (5 minutes)

### 1.1 Create Read-Only User for MCP

```bash
psql -h localhost -U gyz -d gyz
```

```sql
-- Create MCP read-only role
CREATE ROLE mcp_readonly WITH LOGIN PASSWORD 'mcp_secure_pass_2025';

-- Grant permissions
GRANT CONNECT ON DATABASE gyz TO mcp_readonly;
GRANT USAGE ON SCHEMA public TO mcp_readonly;
GRANT SELECT ON uploads, results TO mcp_readonly;

-- Verify permissions
\du mcp_readonly
```

### 1.2 Run Chat Migrations

```bash
cd backend

# Apply migration 001: chat_sessions table
psql -h localhost -U gyz -d gyz -f migrations/001_create_chat_sessions.sql

# Apply migration 002: chat_messages table
psql -h localhost -U gyz -d gyz -f migrations/002_create_chat_messages.sql

# Verify tables created
psql -h localhost -U gyz -d gyz -c "\dt chat_*"
```

Expected output:
```
             List of relations
 Schema |      Name      | Type  | Owner
--------+----------------+-------+-------
 public | chat_messages  | table | gyz
 public | chat_sessions  | table | gyz
```

### 1.3 Insert Test Data (Optional)

```sql
-- Create test session
INSERT INTO chat_sessions (id, user_id, title, status) VALUES
('11111111-1111-1111-1111-111111111111', '<your-user-id>', '测试会话', 'active');

-- Create test messages
INSERT INTO chat_messages (session_id, role, content) VALUES
('11111111-1111-1111-1111-111111111111', 'user', '你好'),
('11111111-1111-1111-1111-111111111111', 'assistant', '您好！我可以帮您查询识别数据。');
```

---

## Step 2: MCP Server Setup (10 minutes)

### 2.1 Install MCP Server

```bash
# Install globally
npm install -g @modelcontextprotocol/server-postgres

# Verify installation
npx @modelcontextprotocol/server-postgres --version
```

### 2.2 Create MCP Configuration

Create `mcp-config.json`:

```json
{
  "name": "guoyan-zhizao-mcp",
  "version": "1.0.0",
  "database": {
    "connectionString": "postgresql://mcp_readonly:mcp_secure_pass_2025@localhost:5432/gyz",
    "allowedTables": ["uploads", "results"],
    "maxRowsPerQuery": 1000,
    "queryTimeoutMs": 5000
  },
  "server": {
    "host": "0.0.0.0",
    "port": 3000
  }
}
```

### 2.3 Start MCP Server

```bash
# Terminal 1: Start MCP server
export DATABASE_URL=postgresql://mcp_readonly:mcp_secure_pass_2025@localhost:5432/gyz
npx -y @modelcontextprotocol/server-postgres

# Should output:
# MCP Server listening on http://0.0.0.0:3000
# Connected to database: gyz
```

### 2.4 Test MCP Server

```bash
# Terminal 2: Test health check
curl http://localhost:3000/health

# Expected response:
# {"status":"healthy","database":"connected","uptime_seconds":5}

# Test list_tables
curl -X POST http://localhost:3000/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"list_tables","params":{}}'

# Expected: {"jsonrpc":"2.0","id":"1","result":{"tables":[{"name":"uploads",...},{"name":"results",...}]}}

# Test query
curl -X POST http://localhost:3000/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"2","method":"query","params":{"sql":"SELECT COUNT(*) FROM results","parameters":[]}}'

# Expected: {"jsonrpc":"2.0","id":"2","result":{"columns":["count"],"rows":[[42]],...}}
```

✅ **Checkpoint**: MCP server is running and can query the database

---

## Step 3: Backend Integration (15 minutes)

### 3.1 Update Environment Variables

Edit `backend/.env`:

```bash
# Existing variables...
DATABASE_URL=postgresql://gyz:gyz123@localhost:5432/gyz
REDIS_URL=redis://localhost:6379

# Add new chat-related variables
LLM_API_URL=https://api.siliconflow.cn/v1/messages
LLM_API_KEY=sk-qqkhvtehvscrwenhzixflzzmmdgzmzaufwurejxgnfbrxcjo
LLM_MODEL=tencent/Hunyuan-MT-7B
LLM_MAX_TOKENS=8192
LLM_TIMEOUT_SECONDS=10

MCP_SERVER_URL=http://localhost:3000

CHAT_CONTEXT_WINDOW=10
CHAT_POLLING_INTERVAL_MS=2000
```

### 3.2 Add Rust Dependencies

Edit `backend/Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "uuid", "time"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Add for chat feature
reqwest = { version = "0.11", features = ["json"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
```

### 3.3 Build Backend

```bash
cd backend

# Build (will download dependencies)
cargo build

# Expected: Compiling completes without errors
```

### 3.4 Run Backend

```bash
# Terminal 3: Start backend
cargo run

# Expected output:
# INFO Connecting to database...
# INFO Redis connection established
# INFO LLM client initialized: https://api.siliconflow.cn/v1/messages
# INFO MCP client initialized: http://localhost:3000
# INFO Server listening on http://0.0.0.0:8080
```

### 3.5 Test Chat API

```bash
# Get your JWT token (assuming you're logged in)
export JWT_TOKEN="your-jwt-token-here"

# Create new chat session
curl -X POST http://localhost:8080/api/v1/chat/sessions \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"测试会话"}'

# Save the session_id from response
export SESSION_ID="<session-id-from-response>"

# Send test message
curl -X POST http://localhost:8080/api/v1/chat/message \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"session_id\":\"$SESSION_ID\",\"message\":\"你好\"}"

# Expected response:
# {
#   "message_id": "...",
#   "role": "assistant",
#   "content": "您好！我可以帮您查询果蔬识别数据。请告诉我您想了解什么？",
#   "timestamp": "2025-10-14T10:30:00Z",
#   "metadata": {
#     "mcp_query_executed": false,
#     "input_tokens": 50,
#     "output_tokens": 80
#   }
# }
```

✅ **Checkpoint**: Backend is running and can communicate with LLM

---

## Step 4: Frontend Integration (10 minutes)

### 4.1 Install Frontend Dependencies

```bash
cd frontend

# Install new dependencies
npm install pinia

# Existing dependencies should already be installed
npm install
```

### 4.2 Update Environment Variables

Edit `frontend/.env.local`:

```bash
VITE_API_BASE_URL=http://localhost:8080/api/v1
VITE_APP_NAME=果蔬智造核心平台
VITE_CHAT_POLLING_INTERVAL=2000
```

### 4.3 Start Frontend Dev Server

```bash
# Terminal 4: Start frontend
npm run dev

# Expected output:
# VITE v5.x.x ready in xxx ms
# ➜ Local:   http://localhost:5173/
# ➜ Network: use --host to expose
```

### 4.4 Test in Browser

1. Open `http://localhost:5173` in your browser
2. Log in with your test account
3. Look for the **floating chat bubble** in the bottom-right corner
4. Click the bubble to open the chat window
5. Type a message: "你好"
6. Verify you receive an AI response

✅ **Checkpoint**: Chat widget is visible and functional

---

## Step 5: Test Database Query Flow (5 minutes)

### 5.1 Test Natural Language Query

In the chat widget, try these queries:

**Query 1: Simple Statistics**
```
你好，我想了解识别情况
```

**Expected**: AI introduces itself and offers to help

**Query 2: Database Query**
```
上周有多少个A级水果?
```

**Expected**: AI responds with actual count from database
```
根据数据库查询,上周您识别了42个A级水果。
```

**Query 3: Complex Query**
```
最近10次识别中,平均置信度是多少?
```

**Expected**: AI queries database and provides statistical answer

### 5.2 Verify MCP Execution

Check backend logs for MCP query execution:

```bash
# In Terminal 3 (backend logs), you should see:
INFO MCP query executed: SELECT COUNT(*) FROM results WHERE grade_a > 0 AND created_at > NOW() - INTERVAL '7 days'
INFO Query returned 1 rows in 45ms
```

Check MCP server logs:

```bash
# In Terminal 1 (MCP server logs):
INFO Executing query: SELECT COUNT(*) FROM results...
INFO Query completed in 45ms, returned 1 rows
```

✅ **Checkpoint**: End-to-end flow working (Frontend → Backend → LLM → MCP → Database)

---

## Step 6: Test Additional Features (5 minutes)

### 6.1 Test Conversation History

```bash
# Get conversation history
curl -X GET "http://localhost:8080/api/v1/chat/history/$SESSION_ID?limit=20" \
  -H "Authorization: Bearer $JWT_TOKEN"

# Expected: Array of messages from your conversation
```

### 6.2 Test Session Listing

```bash
# List all sessions
curl -X GET "http://localhost:8080/api/v1/chat/sessions?status=active" \
  -H "Authorization: Bearer $JWT_TOKEN"

# Expected: Array of your chat sessions
```

### 6.3 Test HTTP Polling

1. Keep chat window open
2. In another browser tab, send a message via API:
   ```bash
   curl -X POST http://localhost:8080/api/v1/chat/message \
     -H "Authorization: Bearer $JWT_TOKEN" \
     -H "Content-Type: application/json" \
     -d "{\"session_id\":\"$SESSION_ID\",\"message\":\"测试轮询\"}"
   ```
3. Within 2 seconds, the message should appear in the chat window (via polling)

✅ **Checkpoint**: All features working correctly

---

## Common Issues & Troubleshooting

### Issue 1: MCP Server Connection Failed

**Symptom**: Backend logs show "MCP client error: connection refused"

**Solution**:
```bash
# Verify MCP server is running
curl http://localhost:3000/health

# If not running, restart MCP server
npx -y @modelcontextprotocol/server-postgres
```

### Issue 2: LLM API Error 401 Unauthorized

**Symptom**: Backend logs show "LLM API error: 401"

**Solution**:
```bash
# Verify API key is correct in backend/.env
echo $LLM_API_KEY

# Test API key directly
curl -X POST https://api.siliconflow.cn/v1/messages \
  -H "Authorization: Bearer sk-qqkhvtehvscrwenhzixflzzmmdgzmzaufwurejxgnfbrxcjo" \
  -H "Content-Type: application/json" \
  -d '{"model":"tencent/Hunyuan-MT-7B","messages":[{"role":"user","content":"你好"}],"max_tokens":100}'
```

### Issue 3: Database Permission Denied

**Symptom**: MCP server logs show "permission denied for table uploads"

**Solution**:
```sql
-- Reconnect to database and grant permissions
psql -h localhost -U gyz -d gyz

GRANT SELECT ON uploads, results TO mcp_readonly;
```

### Issue 4: Chat Bubble Not Appearing

**Symptom**: No floating chat bubble visible in frontend

**Solution**:
```bash
# Check browser console for errors
# Open DevTools (F12) → Console tab

# Verify ChatWidget component is imported in App.vue
grep -r "ChatWidget" frontend/src/App.vue

# Rebuild frontend
cd frontend && npm run dev
```

### Issue 5: Polling Not Working

**Symptom**: New messages don't appear automatically

**Solution**:
```javascript
// Check browser console for polling errors
// Verify VITE_CHAT_POLLING_INTERVAL is set

// Test polling endpoint manually
fetch('http://localhost:8080/api/v1/chat/messages/new?session_id=<SESSION_ID>&after=<LAST_MESSAGE_ID>', {
  headers: { 'Authorization': 'Bearer <JWT_TOKEN>' }
})
.then(r => r.json())
.then(console.log)
```

---

## Next Steps

Now that your chat module is working, consider:

1. **Performance Testing**: Test with 100+ concurrent users
2. **Error Handling**: Test LLM API failures, database timeouts
3. **UI Polish**: Add loading animations, typing indicators
4. **Mobile Testing**: Verify responsiveness on mobile devices
5. **Security Audit**: Review MCP permissions, rate limiting

---

## Integration Scenarios

### Scenario 1: Adding Chat to Existing Page

```vue
<!-- frontend/src/views/ResultsView.vue -->
<script setup lang="ts">
import { useChatStore } from '@/stores/chatStore'

const chatStore = useChatStore()

// Pre-fill chat with context
const askAboutResults = () => {
  chatStore.sendMessage('显示最近的识别结果')
}
</script>

<template>
  <div>
    <h1>识别结果</h1>
    <button @click="askAboutResults">询问AI助手</button>
    <!-- Results table... -->
  </div>
</template>
```

### Scenario 2: Custom MCP Query in Backend

```rust
// backend/src/routes/chat.rs
use crate::services::mcp_client::MCPClient;

pub async fn send_message(
    // ...
) -> Result<Json<ChatMessageResponse>, AppError> {
    // Custom query logic
    let mcp_result = mcp_client.query(
        "SELECT upload_id, grade_a FROM results WHERE user_id = $1 ORDER BY created_at DESC LIMIT 5",
        vec![serde_json::json!(user_id)]
    ).await?;

    // Format for LLM
    let context = format!("数据库查询结果: {:?}", mcp_result.rows);

    // Send to LLM with context
    let llm_response = llm_client.send_message(messages, Some(context)).await?;

    // ...
}
```

### Scenario 3: Adding New MCP Tool

```typescript
// mcp-server/custom-tools.ts
export const customTools = {
  get_statistics: async (timeRange: string) => {
    const query = `
      SELECT
        COUNT(*) as total,
        SUM(grade_a) as a_count,
        SUM(grade_b) as b_count,
        SUM(grade_c) as c_count
      FROM results
      WHERE created_at > NOW() - INTERVAL '${timeRange}'
    `;
    return await db.query(query);
  }
};
```

---

## Additional Resources

- **OpenAPI Spec**: `.specify/specs/ai-api-mcp/contracts/chat-api.yaml`
- **Data Model**: `.specify/specs/ai-api-mcp/data-model.md`
- **Research Document**: `.specify/specs/ai-api-mcp/research.md`
- **SiliconFlow Docs**: https://docs.siliconflow.cn/
- **MCP Specification**: https://modelcontextprotocol.io/
- **Project README**: `../../../README.md`

---

**Congratulations!** 🎉 You've successfully set up the AI Assistant Chat Module. Your users can now query detection data using natural language!
