# Feature Specification: AI Assistant Chat Module

**Version**: 0.1.0
**Author**: System Architect
**Date**: 2025-10-14
**Status**: Draft

## Overview

### Purpose
Enable users to interact with an intelligent AI assistant that can answer questions about fruit/vegetable detection results, query historical data, and provide insights about recognition patterns through natural language conversations.

### Scope
**In Scope**:
- Floating chat bubble widget in Vue 3 frontend with collapsible interface
- Backend REST API endpoints for chat message handling and conversation management
- Integration with Large Language Model (LLM) API for intelligent response generation
- Model Context Protocol (MCP) server for database querying capabilities
- Conversation history persistence and retrieval
- User authentication integration with existing JWT system
- Support for querying detection results, statistics, and historical upload data

**Out of Scope**:
- Voice input/output capabilities
- Multi-language support (Chinese only for v0.1.0)
- Advanced analytics or business intelligence dashboards
- Integration with external knowledge bases beyond the platform database
- Real-time collaborative chat features
- Custom model fine-tuning or training

### Success Criteria
- [ ] Users can initiate conversations from any page within 2 clicks
- [ ] 90% of database queries receive accurate responses within 3 seconds
- [ ] Chat interface remains accessible and functional on mobile viewports (≥375px width)
- [ ] System maintains conversation context across at least 10 message exchanges
- [ ] User satisfaction rating ≥4.0/5.0 for AI response quality
- [ ] Zero credential leaks or unauthorized database access through MCP layer

## Constitutional Compliance

This specification MUST align with the project constitution (`.specify/memory/constitution.md`).

### Principle 1: Code Quality Standards
- [ ] Rust components pass `cargo clippy` and `cargo fmt` (chat API routes, MCP integration)
- [ ] Python code follows PEP 8 with full type annotations (MCP server implementation)
- [ ] Vue components use Composition API (`<script setup>`) for chat widget

### Principle 2: Testing Standards
- [ ] Unit test plan achieving ≥85% coverage for chat message handling and MCP query translation
- [ ] Integration tests for LLM API communication and database MCP interactions
- [ ] E2E test scenarios for complete conversation flows and query execution

### Principle 3: User Experience Consistency
- [ ] Chat widget uses Toast notifications for error states (network failures, LLM timeouts)
- [ ] Responsive design adapts to all viewport sizes with consistent spacing
- [ ] Loading states and typing indicators provide clear feedback during AI processing

### Principle 4: Performance Requirements
- [ ] Chat API response time P95 ≤2000ms (includes LLM call overhead)
- [ ] Conversation history cached in Redis with >95% hit rate for recent messages
- [ ] Chat widget bundle size impact ≤150KB gzipped to maintain FCP <3s budget

## Requirements

### Functional Requirements

**FR1**: User Initiates Chat Conversation
- **Actor**: Authenticated user
- **Precondition**: User is logged in and viewing any page in the application
- **Steps**:
  1. User clicks the floating chat bubble in bottom-right corner
  2. Chat window expands to show conversation interface
  3. System displays welcome message and suggested queries
  4. User types message and presses Enter or clicks Send button
- **Postcondition**: Message is sent to backend, conversation starts, chat history begins recording

**FR2**: AI Assistant Processes User Query
- **Actor**: System (triggered by user message)
- **Precondition**: User message received by backend
- **Steps**:
  1. Backend authenticates request using existing JWT token
  2. System retrieves conversation history from Redis/PostgreSQL
  3. Backend forwards message with context to LLM API
  4. LLM analyzes query and determines if database lookup is needed
  5. If database query required, system invokes MCP server with structured query
  6. MCP server executes query against PostgreSQL (uploads/results tables)
  7. LLM synthesizes response using query results and context
  8. Backend saves message exchange to conversation history
- **Postcondition**: AI response returned to frontend, displayed in chat interface

**FR3**: User Queries Detection Results via Natural Language
- **Actor**: Authenticated user
- **Precondition**: Chat conversation active, user has upload history
- **Steps**:
  1. User asks question like "上周有多少个A级水果?" (How many Grade A fruits last week?)
  2. LLM identifies query intent and required data points
  3. System translates to SQL via MCP: `SELECT COUNT(*) FROM results WHERE grade_a > 0 AND created_at > NOW() - INTERVAL '7 days'`
  4. MCP server executes query with read-only permissions
  5. LLM formats results into conversational response
- **Postcondition**: User receives natural language answer with relevant statistics

**FR4**: User Reviews Conversation History
- **Actor**: Authenticated user
- **Precondition**: User has previous chat interactions
- **Steps**:
  1. User opens chat widget
  2. System loads last 20 messages from Redis cache or PostgreSQL
  3. Messages displayed in chronological order with timestamps
  4. User can scroll to view older messages (lazy load)
- **Postcondition**: Complete conversation context visible to user

**FR5**: System Manages Conversation Sessions
- **Actor**: System
- **Precondition**: User actively chatting or idle
- **Steps**:
  1. System creates conversation session on first message
  2. Session ID stored in user's browser (localStorage/sessionStorage)
  3. Each message associates with session ID
  4. After 30 minutes of inactivity, session marked as inactive
  5. User can manually start new conversation via "New Chat" button
- **Postcondition**: Conversation context properly segmented, avoids confusion across sessions

**FR6**: Administrator Configures LLM API Settings
- **Actor**: System administrator
- **Precondition**: Access to backend configuration files
- **Steps**:
  1. Administrator sets environment variables: `LLM_API_URL`, `LLM_API_KEY`, `LLM_MODEL_NAME`, `LLM_MAX_TOKENS`
  2. Backend validates configuration on startup
  3. System performs health check to LLM endpoint
- **Postcondition**: AI assistant operational with configured LLM provider

### Non-Functional Requirements

**NFR1**: Response Latency
- **Metric**: P95 latency from user message send to AI response display
- **Target**: ≤3 seconds for simple queries, ≤5 seconds for complex database lookups

**NFR2**: Conversation Context Window
- **Metric**: Number of message exchanges retained in context
- **Target**: Minimum 10 exchanges (20 messages) with automatic summarization for older context

**NFR3**: Security - Data Access Control
- **Metric**: Authorization checks per database query
- **Target**: 100% of MCP queries enforce user-specific data filtering (no cross-user data leakage)

**NFR4**: Availability
- **Metric**: Chat service uptime
- **Target**: 99.5% uptime excluding planned maintenance windows

**NFR5**: Scalability
- **Metric**: Concurrent chat sessions supported
- **Target**: Support 100 concurrent users with <5% degradation in response time

**NFR6**: Data Retention
- **Metric**: Conversation history storage duration
- **Target**: Retain last 90 days in PostgreSQL, last 24 hours in Redis cache

## Architecture Impact

### Component Changes

**Backend (Rust/Axum)**:
- Add new module `src/routes/chat.rs` for chat endpoints (`POST /api/v1/chat/message`, `GET /api/v1/chat/history/{session_id}`, `POST /api/v1/chat/sessions`)
- Add `src/services/llm_client.rs` for HTTP client to LLM API (handle retries, timeouts, streaming responses)
- Add `src/services/mcp_client.rs` for communication with MCP server
- Extend `src/models.rs` with `ChatMessage`, `ChatSession`, `LLMRequest`, `LLMResponse`, `MCPQuery` structs
- Update `src/db.rs` to create tables: `chat_sessions`, `chat_messages`
- Add conversation caching in `src/cache.rs` using Redis key pattern `chat:session:{id}:messages`

**AI Service (FastAPI)** - No changes required for inference service

**MCP Server (New Component - Python/FastAPI or Node.js)**:
- Create standalone MCP server (following Model Context Protocol specification)
- Implement read-only query interface for PostgreSQL (whitelist tables: `uploads`, `results`)
- Query translation layer: natural language → SQL with parameterization
- Security layer: user-scoped filtering, query validation, injection prevention
- Two implementation options:
  1. Docker-based: `docker build -t mcp/redis -f src/redis/Dockerfile .` (custom implementation)
  2. NPM package: `npm install -g @data_wise/database-mcp` (if compatible with PostgreSQL)

**Frontend (Vue 3)**:
- New component `src/components/ChatWidget.vue` (floating bubble + expandable window)
- New component `src/components/ChatMessage.vue` (message display with user/AI differentiation)
- New composable `src/composables/useChat.ts` (WebSocket or HTTP polling for real-time updates)
- Add API methods in `src/api/chat.ts` (`sendMessage`, `getHistory`, `createSession`)
- Update `src/App.vue` to include global `<ChatWidget>` component
- Add Tailwind/Element Plus styling for chat UI (dark mode support optional)

**Database (PostgreSQL)**:
- New table `chat_sessions`:
  ```sql
  CREATE TABLE chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    title TEXT,
    status TEXT DEFAULT 'active', -- active/inactive/archived
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
  );
  ```
- New table `chat_messages`:
  ```sql
  CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id),
    role TEXT NOT NULL, -- 'user' or 'assistant'
    content TEXT NOT NULL,
    metadata JSONB, -- store MCP query details, LLM model used, etc.
    created_at TIMESTAMPTZ DEFAULT NOW()
  );
  CREATE INDEX idx_chat_messages_session ON chat_messages(session_id, created_at);
  ```

**Cache (Redis)**:
- New key pattern `chat:session:{session_id}:messages` → list of serialized message objects (last 20 messages)
- TTL: 86400 seconds (24 hours)
- New key pattern `chat:user:{user_id}:active_session` → current active session ID
- TTL: 1800 seconds (30 minutes)

### API Changes

```
POST /api/v1/chat/sessions
Request:
{
  "title": "string (optional)"
}
Response:
{
  "session_id": "uuid",
  "created_at": "timestamp"
}
Status Codes: 201 Created, 401 Unauthorized
```

```
POST /api/v1/chat/message
Request:
{
  "session_id": "uuid",
  "message": "string"
}
Response:
{
  "message_id": "uuid",
  "role": "assistant",
  "content": "string",
  "timestamp": "timestamp",
  "metadata": {
    "mcp_query_executed": "boolean",
    "query_results_count": "integer"
  }
}
Status Codes: 200 OK, 400 Bad Request, 401 Unauthorized, 429 Too Many Requests, 503 Service Unavailable
```

```
GET /api/v1/chat/history/{session_id}?limit=20&offset=0
Request: Query parameters for pagination
Response:
{
  "session_id": "uuid",
  "messages": [
    {
      "id": "uuid",
      "role": "user|assistant",
      "content": "string",
      "created_at": "timestamp"
    }
  ],
  "total": "integer",
  "has_more": "boolean"
}
Status Codes: 200 OK, 401 Unauthorized, 404 Not Found
```

```
GET /api/v1/chat/sessions?status=active&limit=10
Request: Filter by status
Response:
{
  "sessions": [
    {
      "id": "uuid",
      "title": "string",
      "status": "string",
      "last_message_at": "timestamp",
      "message_count": "integer"
    }
  ]
}
Status Codes: 200 OK, 401 Unauthorized
```

### Data Model

```
ChatSession
- id: UUID (primary key)
- user_id: UUID (foreign key → users.id)
- title: TEXT (nullable, auto-generated from first message)
- status: TEXT (active/inactive/archived)
- created_at: TIMESTAMPTZ
- updated_at: TIMESTAMPTZ
```

```
ChatMessage
- id: UUID (primary key)
- session_id: UUID (foreign key → chat_sessions.id)
- role: TEXT (enum: 'user', 'assistant')
- content: TEXT
- metadata: JSONB (stores MCP query info, model version, etc.)
- created_at: TIMESTAMPTZ
```

```
MCPQuery (in-memory/logged)
- query_text: TEXT (natural language query)
- translated_sql: TEXT (generated SQL query)
- parameters: JSON (bound parameters for safety)
- result_count: INTEGER
- execution_time_ms: INTEGER
```

## Implementation Guidance

### Code Quality Checkpoints
- Pre-commit hooks enforce linting (Rust: clippy, Python: black/ruff for MCP server, Vue: ESLint)
- CI pipeline blocks merge on quality gate failures
- Rust async/await patterns must use tokio runtime consistently
- Python type hints required for all MCP server functions

### Testing Strategy

**Unit Tests**:
- Rust: Test LLM client request/response serialization, MCP query construction, message validation
- Python: Test MCP query translation logic, SQL injection prevention, parameterization
- Vue: Test ChatWidget component state management, message rendering, input sanitization

**Integration Tests**:
- Backend ↔ LLM API: Mock LLM responses, test timeout/retry logic
- Backend ↔ MCP Server: Verify query execution, result parsing, error propagation
- Backend ↔ Redis: Test conversation caching, TTL expiration
- Backend ↔ PostgreSQL: Test chat_messages/chat_sessions CRUD operations

**E2E Tests (Cypress)**:
- User opens chat widget → sends message → receives AI response
- User queries detection results → MCP executes database query → results displayed
- User switches between conversation sessions → history loads correctly
- Error handling: LLM API timeout → user sees friendly error message

### Performance Considerations
- Implement LLM response streaming to reduce perceived latency
- Redis caching for last 20 messages per session to avoid repeated DB queries
- Lazy load older messages (pagination) in chat history
- Rate limiting on chat endpoints: 20 requests/minute per user
- Connection pooling for LLM API client (reuse HTTP connections)
- Database query optimization: index on chat_messages(session_id, created_at)
- Frontend: Virtual scrolling for long conversation histories

### Security Considerations
- MCP server operates with read-only database credentials
- Whitelist allowed tables for MCP queries (uploads, results only)
- Enforce row-level security: filter by user_id automatically
- Sanitize LLM-generated SQL before execution (parameterized queries only)
- Validate session ownership before returning chat history
- Rate limiting to prevent abuse/DOS of LLM API
- Do not log sensitive user message content in plain text
- Implement content filtering for inappropriate user inputs

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| LLM API cost escalation | High | Medium | Implement request caching, rate limiting, token budget alerts; monitor monthly spend |
| LLM response quality (hallucinations) | High | Medium | Use structured prompts with explicit instructions; validate database query results; add user feedback mechanism |
| MCP server security vulnerabilities (SQL injection) | High | Low | Use parameterized queries only; whitelist tables; implement query complexity limits; security audit before production |
| Performance degradation under load | Medium | Medium | Load testing with 100+ concurrent users; horizontal scaling for MCP server; LLM response caching |
| User privacy concerns (chat history storage) | Medium | Low | Document data retention policy; provide chat history deletion option; encrypt sensitive messages at rest |
| LLM API service downtime | Medium | Low | Implement circuit breaker pattern; fallback to cached responses; display graceful error messages |

## Dependencies

**External**:
- LLM API service: SiliconFlow API (https://api.siliconflow.cn/v1/messages) with Tencent Hunyuan-MT-7B model
  - Excellent Chinese language support (designed for Chinese market)
  - Cost-effective pricing structure
  - Max tokens: 8192 per request
- Model Context Protocol specification (for MCP server implementation)
- Existing authentication/authorization system (JWT middleware)

**Internal**:
- Existing user authentication (`users` table with JWT integration)
- Existing database schema (`uploads`, `results` tables for querying)
- Existing Redis infrastructure for caching

**Blockers**:
- SiliconFlow API credentials (API key: sk-qqkhvtehvscrwenhzixflzzmmdgzmzaufwurejxgnfbrxcjo) and endpoint configuration
- Decision on MCP implementation approach (Docker custom build vs @data_wise/database-mcp NPM package)
- Database read-only user credentials provisioning for MCP server

## Assumptions

1. **LLM Provider**: Using SiliconFlow API with Tencent Hunyuan-MT-7B model, which provides Anthropic-compatible message format
2. **Message Format**: Text-only messages (no image/file attachments in v0.1.0)
3. **Concurrent Sessions**: Users maintain one active session at a time (can create multiple but switch between them)
4. **Database Access**: MCP server has read-only access to production database replica (preferred) or main database with restricted permissions
5. **Language**: Primary language is Chinese (Simplified), LLM prompts and responses in Chinese - ideal for Hunyuan-MT-7B's optimization
6. **Browser Support**: Modern browsers with ES2020+ support (Chrome 90+, Firefox 88+, Safari 14+)

## Implementation Decisions

Based on analysis and stakeholder input, the following technical decisions have been made:

1. **LLM Provider**: SiliconFlow API with Tencent Hunyuan-MT-7B model
   - Rationale: Optimized for Chinese language understanding, cost-effective, 8192 token context window
   - API endpoint: https://api.siliconflow.cn/v1/messages
   - Compatible with Anthropic message format for easier integration

2. **MCP Implementation**: Start with @data_wise/database-mcp NPM package
   - Rationale: Rapid prototyping, proven stability, lower maintenance burden
   - Fallback plan: Migrate to custom Docker implementation if feature limitations discovered
   - Validation: Conduct technical spike (2-3 days) to verify PostgreSQL compatibility

3. **Real-time Communication**: HTTP polling for v0.1.0
   - Rationale: Simpler infrastructure, adequate latency for chat use case (1-2s acceptable)
   - Future enhancement: Evaluate WebSocket migration in v0.2.0 based on user feedback
   - Polling interval: 2 seconds when chat window open, pause when minimized

## Approval

- [ ] Technical Lead
- [ ] Product Owner
- [ ] Security Review (required - MCP database access and LLM integration)

---

**Revision History**:
| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0 | 2025-10-14 | System Architect | Initial draft with full requirements and architecture design |
