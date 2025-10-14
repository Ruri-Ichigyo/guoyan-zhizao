# Data Model: AI Assistant Chat Module

**Feature**: AI Assistant Chat Module (ai-api-mcp)
**Version**: 0.1.0
**Last Updated**: 2025-10-14

## Overview

This document defines the data entities, relationships, and validation rules for the AI Assistant Chat Module. All entities are stored in PostgreSQL with appropriate indexes for performance.

---

## Entity Diagram

```
┌─────────────────┐
│     users       │
│  (existing)     │
└────────┬────────┘
         │ 1
         │
         │ N
┌────────▼────────────┐
│  chat_sessions      │
│  ─────────────────  │
│  id (PK)            │
│  user_id (FK)       │
│  title              │
│  status             │
│  created_at         │
│  updated_at         │
└────────┬────────────┘
         │ 1
         │
         │ N
┌────────▼────────────┐
│  chat_messages      │
│  ─────────────────  │
│  id (PK)            │
│  session_id (FK)    │
│  role               │
│  content            │
│  metadata (JSONB)   │
│  created_at         │
└─────────────────────┘
```

---

## Entities

### 1. ChatSession

**Purpose**: Represents a conversation session between user and AI assistant.

**Table**: `chat_sessions`

**Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PRIMARY KEY, DEFAULT gen_random_uuid() | Unique session identifier |
| user_id | UUID | NOT NULL, REFERENCES users(id) ON DELETE CASCADE | Owner of the session |
| title | TEXT | NULLABLE | Auto-generated from first user message (first 50 chars) |
| status | TEXT | NOT NULL, DEFAULT 'active' | Session state: 'active', 'inactive', 'archived' |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Session creation timestamp |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Last message timestamp |

**Indexes**:
```sql
CREATE INDEX idx_chat_sessions_user_id ON chat_sessions(user_id);
CREATE INDEX idx_chat_sessions_status ON chat_sessions(status);
CREATE INDEX idx_chat_sessions_updated_at ON chat_sessions(updated_at DESC);
```

**Validation Rules**:
- `status` must be one of: 'active', 'inactive', 'archived'
- `title` length ≤ 200 characters
- `user_id` must reference existing user
- `updated_at` must be ≥ `created_at`

**State Transitions**:
```
active ──[30 min idle]──> inactive
active ──[user action]──> archived
inactive ──[new message]──> active
archived ──[cannot change]──> archived (final state)
```

**Business Rules**:
1. User can have multiple sessions (no limit)
2. Only one session per user can be 'active' at a time
3. Sessions auto-transition to 'inactive' after 30 minutes of no activity
4. Title is auto-generated from first user message on session creation
5. Deleting a user cascades to delete all their sessions and messages

---

### 2. ChatMessage

**Purpose**: Stores individual messages within a conversation session.

**Table**: `chat_messages`

**Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PRIMARY KEY, DEFAULT gen_random_uuid() | Unique message identifier |
| session_id | UUID | NOT NULL, REFERENCES chat_sessions(id) ON DELETE CASCADE | Parent session |
| role | TEXT | NOT NULL, CHECK (role IN ('user', 'assistant', 'system')) | Message sender |
| content | TEXT | NOT NULL | Message text content |
| metadata | JSONB | NULLABLE | Additional data (MCP queries, tokens, etc.) |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Message timestamp |

**Indexes**:
```sql
CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id, created_at);
CREATE INDEX idx_chat_messages_created_at ON chat_messages(created_at DESC);
CREATE GIN INDEX idx_chat_messages_metadata ON chat_messages USING GIN (metadata);
```

**Validation Rules**:
- `role` must be one of: 'user', 'assistant', 'system'
- `content` length ≤ 10000 characters (approximately 8000 tokens)
- `metadata` must be valid JSON if present
- Messages within a session must have monotonically increasing `created_at`

**Metadata Schema** (JSONB):
```json
{
  "mcp_query_executed": boolean,
  "mcp_query_text": string (optional),
  "mcp_result_count": integer (optional),
  "llm_model": string,
  "input_tokens": integer,
  "output_tokens": integer,
  "response_time_ms": integer,
  "error": string (optional)
}
```

**Business Rules**:
1. First message in session must have `role = 'user'`
2. Messages alternate between 'user' and 'assistant' roles (system messages can appear anywhere)
3. Cannot modify message content after creation (immutable audit trail)
4. Deleting a session cascades to delete all messages
5. Metadata is populated by backend, not user-editable

---

## Derived Entities (Redis Cache)

### 3. CachedConversation

**Purpose**: Cache recent messages for fast retrieval without hitting PostgreSQL.

**Redis Key**: `chat:session:{session_id}:messages`

**Type**: List (LPUSH for new, LRANGE for retrieval)

**TTL**: 86400 seconds (24 hours)

**Value Structure**:
```json
[
  {
    "id": "uuid",
    "session_id": "uuid",
    "role": "user",
    "content": "消息内容",
    "created_at": "2025-10-14T10:30:00Z"
  },
  {
    "id": "uuid",
    "session_id": "uuid",
    "role": "assistant",
    "content": "AI回复",
    "metadata": {
      "mcp_query_executed": true,
      "input_tokens": 150,
      "output_tokens": 200
    },
    "created_at": "2025-10-14T10:30:05Z"
  }
]
```

**Business Rules**:
- Store only last 20 messages per session in cache
- On cache miss, load from PostgreSQL and repopulate cache
- Expire after 24 hours of inactivity

---

### 4. ConversationSummary

**Purpose**: Store summarized context for long conversations (>10 exchanges).

**Redis Key**: `chat:session:{session_id}:summary`

**Type**: String

**TTL**: 86400 seconds (24 hours)

**Value Structure**:
```json
{
  "summary_text": "用户询问了上周A级水果数量，AI回答42个。用户接着询问B级水果...",
  "generated_at": "2025-10-14T10:45:00Z",
  "covers_messages_up_to": "uuid",
  "message_count": 20
}
```

**Business Rules**:
- Generated after 10 exchanges (20 messages)
- Summarizes oldest messages, keeps recent 10 exchanges in full
- Used in LLM context to stay within token budget

---

### 5. ActiveSession

**Purpose**: Track user's currently active chat session.

**Redis Key**: `chat:user:{user_id}:active_session`

**Type**: String

**TTL**: 1800 seconds (30 minutes)

**Value**: `session_id` (UUID string)

**Business Rules**:
- Only one active session per user
- TTL refreshed on each user activity
- On expiry, session transitions to 'inactive' status in PostgreSQL

---

## Relationships

### One-to-Many: User → ChatSessions
- **Cardinality**: 1:N
- **Cascade**: DELETE user → DELETE all sessions
- **Foreign Key**: `chat_sessions.user_id → users.id`

### One-to-Many: ChatSession → ChatMessages
- **Cardinality**: 1:N
- **Cascade**: DELETE session → DELETE all messages
- **Foreign Key**: `chat_messages.session_id → chat_sessions.id`

---

## Database Migrations

### Migration 001: Create chat_sessions table

```sql
-- File: backend/migrations/001_create_chat_sessions.sql

CREATE TABLE IF NOT EXISTS chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'archived')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_sessions_user_id ON chat_sessions(user_id);
CREATE INDEX idx_chat_sessions_status ON chat_sessions(status);
CREATE INDEX idx_chat_sessions_updated_at ON chat_sessions(updated_at DESC);

-- Trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_chat_session_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_chat_session_timestamp
BEFORE UPDATE ON chat_sessions
FOR EACH ROW
EXECUTE FUNCTION update_chat_session_timestamp();
```

### Migration 002: Create chat_messages table

```sql
-- File: backend/migrations/002_create_chat_messages.sql

CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL CHECK (char_length(content) <= 10000),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id, created_at);
CREATE INDEX idx_chat_messages_created_at ON chat_messages(created_at DESC);
CREATE GIN INDEX idx_chat_messages_metadata ON chat_messages USING GIN (metadata);
```

### Migration 003: Add read-only user for MCP

```sql
-- File: backend/migrations/003_create_mcp_readonly_user.sql

-- Create read-only role for MCP server
CREATE ROLE mcp_readonly WITH LOGIN PASSWORD 'mcp_secure_password_here';

-- Grant CONNECT privilege
GRANT CONNECT ON DATABASE gyz TO mcp_readonly;

-- Grant USAGE on schema
GRANT USAGE ON SCHEMA public TO mcp_readonly;

-- Grant SELECT on specific tables only
GRANT SELECT ON uploads TO mcp_readonly;
GRANT SELECT ON results TO mcp_readonly;

-- Ensure no write permissions
REVOKE INSERT, UPDATE, DELETE, TRUNCATE ON ALL TABLES IN SCHEMA public FROM mcp_readonly;
```

---

## Data Access Patterns

### Pattern 1: Create New Session
```rust
// When user sends first message
let session = sqlx::query_as!(
    ChatSession,
    r#"
    INSERT INTO chat_sessions (user_id, title, status)
    VALUES ($1, $2, 'active')
    RETURNING id, user_id, title, status, created_at, updated_at
    "#,
    user_id,
    title
)
.fetch_one(&pool)
.await?;
```

### Pattern 2: Load Recent Messages (Cache-Aside)
```rust
// Try Redis first
let cached_messages = redis_client.lrange(
    format!("chat:session:{}:messages", session_id),
    0,
    19  // Last 20 messages
).await;

if cached_messages.is_empty() {
    // Cache miss - load from PostgreSQL
    let messages = sqlx::query_as!(
        ChatMessage,
        "SELECT * FROM chat_messages WHERE session_id = $1 ORDER BY created_at DESC LIMIT 20",
        session_id
    )
    .fetch_all(&pool)
    .await?;

    // Repopulate cache
    for msg in &messages {
        redis_client.lpush(
            format!("chat:session:{}:messages", session_id),
            serde_json::to_string(msg)?
        ).await?;
    }
    redis_client.expire(format!("chat:session:{}:messages", session_id), 86400).await?;
}
```

### Pattern 3: Save New Message
```rust
// Save to PostgreSQL
let message = sqlx::query_as!(
    ChatMessage,
    r#"
    INSERT INTO chat_messages (session_id, role, content, metadata)
    VALUES ($1, $2, $3, $4)
    RETURNING id, session_id, role, content, metadata, created_at
    "#,
    session_id,
    role,
    content,
    metadata
)
.fetch_one(&pool)
.await?;

// Update cache
redis_client.lpush(
    format!("chat:session:{}:messages", session_id),
    serde_json::to_string(&message)?
).await?;

// Update session timestamp
sqlx::query!(
    "UPDATE chat_sessions SET updated_at = NOW() WHERE id = $1",
    session_id
)
.execute(&pool)
.await?;
```

---

## Performance Considerations

### Indexes
- `idx_chat_messages_session_id` enables fast message retrieval by session
- `idx_chat_sessions_user_id` enables fast session listing for user
- GIN index on `metadata` enables fast JSONB queries (e.g., find messages with MCP queries)

### Partitioning (Future Enhancement)
For high volume, consider partitioning `chat_messages` by `created_at`:
```sql
-- Partition by month
CREATE TABLE chat_messages_2025_10 PARTITION OF chat_messages
FOR VALUES FROM ('2025-10-01') TO ('2025-11-01');
```

### Archival Strategy
- Messages older than 90 days: move to cold storage table
- Keep only metadata in main table (link to archive)
- Implement in Phase 2 if storage becomes concern

---

## Security Considerations

1. **Row-Level Security**: Users can only access their own sessions/messages
2. **MCP Read-Only Access**: Separate database role with SELECT-only permissions
3. **Input Validation**: Check message content length, role values before insert
4. **Audit Trail**: Messages are immutable (no UPDATE allowed)
5. **PII Protection**: No user email/phone stored in chat tables

---

## Testing Data

### Sample Session
```sql
-- Create test user (if not exists)
INSERT INTO users (id, email, password_hash) VALUES
('00000000-0000-0000-0000-000000000001', 'test@example.com', 'hash');

-- Create test session
INSERT INTO chat_sessions (id, user_id, title, status) VALUES
('11111111-1111-1111-1111-111111111111', '00000000-0000-0000-0000-000000000001', '询问水果识别情况', 'active');

-- Create test messages
INSERT INTO chat_messages (session_id, role, content, metadata) VALUES
('11111111-1111-1111-1111-111111111111', 'user', '上周有多少个A级水果?', NULL),
('11111111-1111-1111-1111-111111111111', 'assistant', '上周您识别了42个A级水果。', '{"mcp_query_executed": true, "mcp_result_count": 1, "input_tokens": 150, "output_tokens": 200}');
```

---

**Next Steps**: Create API contracts in `/contracts/` directory with OpenAPI specifications.
