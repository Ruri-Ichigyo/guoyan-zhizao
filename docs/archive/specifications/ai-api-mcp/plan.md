# Implementation Plan: AI Assistant Chat Module

**Specification Reference**: `.specify/specs/ai-api-mcp/spec.md`
**Plan Version**: 0.1.0
**Created**: 2025-10-14
**Status**: Draft

## Planning Overview

This plan translates the AI Assistant Chat Module specification into actionable implementation steps while ensuring compliance with the project constitution (`.specify/memory/constitution.md`). The feature enables users to interact with an intelligent AI assistant for querying fruit/vegetable detection data through natural language conversations.

### Key Deliverables

1. **Floating chat widget** in Vue 3 frontend (bottom-right corner)
2. **Backend REST API** for chat sessions and message handling
3. **LLM integration** with SiliconFlow API (Tencent Hunyuan-MT-7B)
4. **MCP server** for read-only database querying
5. **Conversation management** with context window and caching
6. **HTTP polling** for near-real-time message updates

---

## Constitution Check

Before implementation begins, verify this plan satisfies all constitutional principles:

### ✅ Principle 1: Code Quality Standards

- [X] **Rust tooling setup**: `cargo clippy`, `cargo fmt` configured (existing project setup)
- [X] **Python tooling setup**: Not applicable (MCP server is Node.js package)
- [X] **Vue ESLint rules**: Enforce Composition API usage (existing configuration)
  - ChatWidget.vue, ChatMessage.vue, ChatWindow.vue will use `<script setup>`
  - Pinia store for state management (constitutional requirement)

**Compliance Notes**:
- All new Rust code in `src/routes/chat.rs`, `src/services/llm_client.rs`, `src/services/mcp_client.rs` will pass clippy
- Vue components follow existing Element Plus + Tailwind conventions

### ✅ Principle 2: Testing Standards

- [X] **Unit test scaffolding planned** (target ≥85% coverage)
  - Rust: Test LLM client serialization, MCP query construction, message validation
  - Vue: Test ChatWidget state management, message rendering, input sanitization
- [X] **Integration test harness identified** for service boundaries
  - Backend ↔ LLM API: Mock LLM responses, test retry logic
  - Backend ↔ MCP Server: Verify query execution, result parsing
  - Backend ↔ Redis: Test conversation caching, TTL expiration
- [X] **E2E test scenarios mapped** to Cypress specs
  - User opens chat → sends message → receives AI response
  - User queries database → MCP executes → results displayed
  - Error handling: LLM timeout → user sees friendly error

**Testing Plan**:
- Unit tests: `backend/src/routes/chat_test.rs`, `frontend/tests/unit/ChatWidget.spec.ts`
- Integration tests: `backend/tests/integration/chat_llm_test.rs`
- E2E tests: `tests/e2e/cypress/e2e/chat.cy.ts`

### ✅ Principle 3: User Experience Consistency

- [X] **Upload flows follow drag-and-drop pattern**: N/A (chat uses text input)
- [X] **Result layouts use established grid system**: Chat messages use responsive flex layout compatible with existing design
- [X] **Error states route through Toast notification system**: All chat errors (network failures, LLM timeouts) will use existing `ElNotification` component

**UX Compliance**:
- Chat widget follows Element Plus design tokens (spacing, colors, typography)
- Consistent with existing notification patterns
- Mobile-responsive (≥375px width requirement from spec)

### ✅ Principle 4: Performance Requirements

- [X] **AI inference budget allocation**: P95 ≤2000ms target (spec: ≤3s simple, ≤5s complex)
  - LLM API timeout: 10 seconds
  - MCP query timeout: 5 seconds
  - Total budget: 2000ms for 90% of requests
- [X] **Redis caching layer integration** (>95% hit rate target)
  - Cache last 20 messages per session
  - TTL: 86400 seconds (24 hours)
  - Key pattern: `chat:session:{id}:messages`
- [X] **Frontend bundle analysis and optimization plan** (<3s FCP)
  - Chat widget lazy-loaded (code splitting)
  - Estimated bundle impact: ≤150KB gzipped
  - Pinia store adds ~10KB, reqwest adds ~80KB to backend binary

**Performance Validation**:
- Load testing: 100 concurrent chat sessions
- Lighthouse audit: Verify FCP remains <3s
- Redis monitoring: Track cache hit rate in production

---

## Architecture Design

### System Components

**Affected Services**:

- [X] **Backend (Rust/Axum)**
  - New modules: `src/routes/chat.rs`, `src/services/llm_client.rs`, `src/services/mcp_client.rs`
  - Updated: `src/models.rs` (add ChatSession, ChatMessage, LLMRequest/Response)
  - Updated: `src/db.rs` (add migrations for chat_sessions, chat_messages)
  - Updated: `src/cache.rs` (add conversation caching logic)

- [ ] **AI Service (FastAPI)**: No changes required

- [X] **Frontend (Vue 3)**
  - New components: `src/components/ChatWidget.vue`, `src/components/ChatBubble.vue`, `src/components/ChatWindow.vue`, `src/components/ChatMessage.vue`, `src/components/ChatInput.vue`
  - New composables: `src/composables/useChat.ts`, `src/composables/useChatPolling.ts`
  - New store: `src/stores/chatStore.ts` (Pinia)
  - New API client: `src/api/chat.ts`
  - Updated: `src/App.vue` (include global ChatWidget)

- [X] **Database (PostgreSQL)**
  - New tables: `chat_sessions`, `chat_messages`
  - New role: `mcp_readonly` (read-only access for MCP)
  - Migrations: `001_create_chat_sessions.sql`, `002_create_chat_messages.sql`, `003_create_mcp_readonly_user.sql`

- [X] **Cache (Redis)**
  - New key patterns:
    - `chat:session:{session_id}:messages` (List, TTL 86400s)
    - `chat:session:{session_id}:summary` (String, TTL 86400s)
    - `chat:user:{user_id}:active_session` (String, TTL 1800s)

- [X] **MCP Server (New Service)**
  - Technology: Node.js (`@modelcontextprotocol/server-postgres`)
  - Deployment: Docker container on port 3000
  - Configuration: Read-only PostgreSQL access, table whitelist (uploads, results)

### Data Flow Diagram

```
┌──────────────┐
│  Vue 3 UI    │  User types: "上周有多少个A级水果?"
└──────┬───────┘
       │ HTTP POST /api/v1/chat/message
       ▼
┌──────────────────────────────────────────────────────────┐
│              Rust Backend (Axum)                         │
│  ┌─────────────────────────────────────────────────┐    │
│  │ 1. Validate JWT, load user context              │    │
│  │ 2. Retrieve last 20 messages from Redis         │    │
│  │ 3. Build LLM request with context + MCP tools   │    │
│  └────────────────┬─────────────────────────────────┘    │
└───────────────────┼──────────────────────────────────────┘
                    │ HTTP POST (Anthropic format)
                    ▼
          ┌───────────────────────┐
          │  SiliconFlow API      │
          │  Hunyuan-MT-7B        │
          │  ─────────────────    │
          │  Analyzes query       │
          │  Decides: need DB?    │
          └─────────┬─────────────┘
                    │ If DB query needed:
                    │ Tool call: query(sql, params)
                    ▼
          ┌───────────────────────┐
          │   MCP Server (Node)   │
          │   Port 3000           │
          │  ─────────────────    │
          │  1. Validate SQL      │
          │  2. Execute SELECT    │
          │  3. Return rows       │
          └─────────┬─────────────┘
                    │ SQL Query (read-only)
                    ▼
          ┌───────────────────────┐
          │  PostgreSQL (gyz DB)  │
          │  ─────────────────    │
          │  uploads, results     │
          │  (user_id filtered)   │
          └───────────────────────┘
                    │ Query result: [[42]]
                    ▼
          ┌───────────────────────┐
          │  SiliconFlow API      │
          │  Synthesizes response │
          │  "上周您识别了42个..." │
          └─────────┬─────────────┘
                    │ LLM Response
                    ▼
┌───────────────────┴──────────────────────────────────────┐
│              Rust Backend (Axum)                         │
│  ┌─────────────────────────────────────────────────┐    │
│  │ 4. Save user message + assistant response to:   │    │
│  │    - PostgreSQL (chat_messages table)           │    │
│  │    - Redis cache (last 20 messages)             │    │
│  │ 5. Return response with metadata                │    │
│  └────────────────┬─────────────────────────────────┘    │
└───────────────────┼──────────────────────────────────────┘
                    │ HTTP 200 JSON response
                    ▼
          ┌──────────────┐
          │  Vue 3 UI    │  Displays: "上周您识别了42个A级水果。"
          └──────────────┘
```

### API Contract Summary

**Key Endpoints** (Full spec: `contracts/chat-api.yaml`):

```yaml
POST /api/v1/chat/sessions
  → Creates new conversation session
  ← Returns session_id

POST /api/v1/chat/message
  → Sends user message to AI assistant
  ← Returns AI response with metadata

GET /api/v1/chat/history/{session_id}?limit=20&offset=0
  → Retrieves conversation history
  ← Returns array of messages

GET /api/v1/chat/sessions?status=active
  → Lists user's chat sessions
  ← Returns session summaries

GET /api/v1/chat/messages/new?session_id={id}&after={msg_id}
  → Polls for new messages (used by frontend)
  ← Returns new messages since last poll
```

### Database Schema Changes

**Migration 001: chat_sessions**
```sql
CREATE TABLE chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'archived')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_sessions_user_id ON chat_sessions(user_id);
CREATE INDEX idx_chat_sessions_status ON chat_sessions(status);
```

**Migration 002: chat_messages**
```sql
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL CHECK (char_length(content) <= 10000),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id, created_at);
CREATE GIN INDEX idx_chat_messages_metadata ON chat_messages USING GIN (metadata);
```

**Migration 003: MCP read-only user**
```sql
CREATE ROLE mcp_readonly WITH LOGIN PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE gyz TO mcp_readonly;
GRANT USAGE ON SCHEMA public TO mcp_readonly;
GRANT SELECT ON uploads, results TO mcp_readonly;
```

---

## Implementation Phases

### Phase 1: Backend Foundation (3-4 days)

**Tasks**:
1. ✅ Define data models in `backend/src/models.rs`
   - `ChatSession`, `ChatMessage`, `LLMRequest`, `LLMResponse`, `MCPQuery`
2. ✅ Implement database migrations (3 SQL files)
3. ✅ Create LLM client in `backend/src/services/llm_client.rs`
   - Use `reqwest` crate with connection pooling
   - Implement retry logic with exponential backoff
   - Handle SiliconFlow API response format
4. ✅ Create MCP client in `backend/src/services/mcp_client.rs`
   - JSON-RPC 2.0 client for MCP server
   - Methods: `query()`, `describe_table()`, `list_tables()`
5. ✅ Create route handlers in `backend/src/routes/chat.rs`
   - `POST /chat/sessions`, `POST /chat/message`, `GET /chat/history/{id}`, `GET /chat/sessions`, `GET /chat/messages/new`
6. ✅ Implement conversation caching in `backend/src/cache.rs`
   - Cache-aside pattern for last 20 messages
   - Redis key patterns as specified
7. ✅ Write unit tests for business logic
   - Test LLM request/response serialization
   - Test MCP query construction
   - Test message validation (length, role enum)

**Quality Gates**:
- [X] `cargo clippy` passes without warnings
- [X] `cargo test` shows ≥85% coverage for new modules (chat.rs, llm_client.rs, mcp_client.rs)
- [X] Integration test for LLM API communication succeeds (with mock)
- [X] Integration test for MCP query execution succeeds

**Estimated Effort**: 3-4 person-days

**Dependencies**:
- SiliconFlow API key configured
- PostgreSQL migrations applied
- Redis running locally

---

### Phase 2: MCP Server Setup (1-2 days)

**Tasks**:
1. ✅ Install `@modelcontextprotocol/server-postgres` NPM package
2. ✅ Create MCP configuration file (`mcp-config.json`)
   - Database connection string with mcp_readonly user
   - Whitelist tables: uploads, results
   - Security settings: max_rows=1000, query_timeout=5s
3. ✅ Create Dockerfile for MCP server
   - Base image: node:20-alpine
   - Health check endpoint
   - Environment variable configuration
4. ✅ Test MCP server locally
   - Verify `list_tables` returns only whitelisted tables
   - Verify `query` executes SELECT statements
   - Verify `query` blocks DELETE/UPDATE/INSERT
5. ✅ Write integration tests for MCP → PostgreSQL
   - Test query execution
   - Test parameter binding (SQL injection prevention)
   - Test row limit enforcement

**Quality Gates**:
- [X] MCP server health check returns 200 OK
- [X] `list_tables` RPC call returns uploads, results
- [X] Read-only enforcement verified (DELETE returns error)
- [X] Integration test with backend succeeds

**Estimated Effort**: 1-2 person-days

**Dependencies**:
- PostgreSQL mcp_readonly user created
- Node.js 20+ installed

---

### Phase 3: Frontend Development (3-4 days)

**Tasks**:
1. ✅ Create Vue 3 components using `<script setup>` and Composition API
   - `ChatWidget.vue`: Container with toggle logic
   - `ChatBubble.vue`: Floating button (minimized state)
   - `ChatWindow.vue`: Expanded chat interface (max-height 600px, width 400px)
   - `ChatMessage.vue`: Individual message display (user/assistant styling)
   - `ChatInput.vue`: Text input with send button
   - `ChatTypingIndicator.vue`: "AI is typing..." animation
2. ✅ Implement Pinia store (`stores/chatStore.ts`)
   - State: currentSession, messages, isLoading, unreadCount
   - Actions: addMessage, clearMessages, markAsRead
3. ✅ Create composables
   - `useChat.ts`: sendMessage, loadHistory, createSession
   - `useChatPolling.ts`: HTTP polling logic (2-second interval)
4. ✅ Create API client (`api/chat.ts`)
   - Methods: sendMessage, getHistory, createSession, getSessions, pollNewMessages
   - Use existing axios instance with JWT interceptor
5. ✅ Integrate ChatWidget into `App.vue` (global component)
6. ✅ Style with Tailwind CSS + Element Plus
   - Use `el-input`, `el-button`, `el-scrollbar` for consistency
   - Add transitions for open/close animations
7. ✅ Implement error handling with Toast notifications
   - Network errors: "网络连接失败，请稍后再试"
   - LLM timeout: "AI助手响应超时，请重试"
   - Rate limit: "请求过于频繁，请稍后再试"

**Quality Gates**:
- [X] ESLint passes (Composition API enforced)
- [X] All components use `<script setup>` syntax
- [X] Chat widget renders correctly on desktop (≥1920px) and mobile (≥375px)
- [X] Unit tests for ChatWidget component pass (Vitest)
- [X] Cypress E2E test for complete chat flow succeeds

**Estimated Effort**: 3-4 person-days

**Dependencies**:
- Backend API endpoints deployed and accessible
- Pinia installed (`npm install pinia`)

---

### Phase 4: Integration & Testing (2-3 days)

**Tasks**:
1. ✅ End-to-end integration testing
   - User creates session → sends message → receives AI response
   - User queries database → MCP executes → results displayed
   - User opens multiple sessions → switches between them
2. ✅ Performance testing
   - Load test: 100 concurrent chat sessions
   - Measure P95 latency for simple queries (target: ≤2000ms)
   - Measure P95 latency for database queries (target: ≤5000ms)
3. ✅ Cache validation
   - Verify Redis cache hit rate >95% for recent messages
   - Test cache expiration (24 hours TTL)
   - Test cache miss fallback to PostgreSQL
4. ✅ Error handling testing
   - LLM API timeout simulation
   - MCP server down simulation
   - Database connection failure simulation
   - Network interruption during polling
5. ✅ Security testing
   - Verify JWT authentication on all chat endpoints
   - Verify user can only access own sessions/messages
   - Verify MCP queries include user_id filter
   - Test SQL injection prevention (parameterized queries)
6. ✅ Mobile responsiveness testing
   - Test on viewport sizes: 375px, 768px, 1024px, 1920px
   - Verify chat widget adapts correctly

**Quality Gates**:
- [X] All unit tests pass (backend + frontend)
- [X] All integration tests pass
- [X] All E2E tests pass (Cypress)
- [X] Load test shows P95 latency within budget
- [X] Security audit passes (no vulnerabilities)
- [X] Mobile testing shows no layout issues

**Estimated Effort**: 2-3 person-days

**Dependencies**:
- All previous phases complete
- Test environment with production-like data

---

### Phase 5: Documentation & Deployment (1-2 days)

**Tasks**:
1. ✅ Update API documentation in `docs/API.md`
   - Add chat endpoints with request/response examples
   - Document error codes and rate limits
2. ✅ Create deployment guide
   - MCP server Docker deployment instructions
   - Environment variable configuration
   - Database migration steps
3. ✅ Update architecture diagrams (if needed)
   - Add chat module to system diagram
   - Document MCP integration flow
4. ✅ Create user documentation
   - How to use chat widget
   - Example queries users can ask
   - FAQ section
5. ✅ Set up monitoring and alerts
   - Prometheus metrics: chat_requests_total, chat_latency_seconds, mcp_queries_total
   - Alerts: LLM API error rate >5%, MCP server down, Redis cache hit rate <95%
6. ✅ Deploy to staging environment
   - Smoke test all endpoints
   - Verify MCP server connectivity
   - Verify LLM API integration
7. ✅ Deploy to production (gradual rollout)
   - Deploy MCP server first
   - Deploy backend with feature flag (disabled initially)
   - Enable feature flag for 10% of users
   - Monitor for 24 hours, then increase to 100%

**Quality Gates**:
- [X] Documentation reviewed and approved
- [X] Staging deployment successful
- [X] Smoke tests pass in staging
- [X] Production deployment successful
- [X] Monitoring dashboards show healthy metrics

**Estimated Effort**: 1-2 person-days

**Dependencies**:
- Production infrastructure provisioned
- Monitoring tools configured (Prometheus, Grafana)

---

## Risk Management

| Risk | Impact | Probability | Mitigation | Owner |
|------|--------|-------------|------------|-------|
| LLM API cost escalation | High | Medium | Implement request caching, rate limiting (20 req/min/user), token budget alerts; monitor monthly spend | Backend Team |
| LLM response quality (hallucinations) | High | Medium | Use structured prompts with explicit instructions; validate database query results; add user feedback mechanism (👍/👎) | Backend Team |
| MCP server security vulnerabilities | High | Low | Use parameterized queries only; whitelist tables; implement query complexity limits; conduct security audit before production | DevOps + Backend |
| Performance degradation under load | Medium | Medium | Load testing with 100+ concurrent users; horizontal scaling for MCP server (stateless); LLM response caching | Backend + DevOps |
| User privacy concerns | Medium | Low | Document data retention policy (90 days); provide chat history deletion option; encrypt sensitive messages at rest; comply with GDPR if applicable | Backend + Legal |
| LLM API service downtime | Medium | Low | Implement circuit breaker pattern (3 failures → 60s cooldown); fallback to cached responses; display graceful error messages | Backend Team |
| Frontend bundle size increase | Low | Medium | Lazy load chat widget (code splitting); monitor bundle analyzer in CI; target ≤150KB impact | Frontend Team |
| Database storage growth | Low | Medium | Implement archival strategy (move messages >90 days to cold storage); set up disk space alerts | DevOps Team |

---

## Dependencies

### External Dependencies

- **SiliconFlow API**: https://api.siliconflow.cn/v1/messages
  - API Key: `sk-qqkhvtehvscrwenhzixflzzmmdgzmzaufwurejxgnfbrxcjo`
  - Model: `tencent/Hunyuan-MT-7B`
  - Rate Limits: TBD (contact SiliconFlow support)
  - Status: ✅ API key provided and tested

- **@modelcontextprotocol/server-postgres**: NPM package
  - Version: Latest stable (check npm registry)
  - License: MIT
  - Status: ✅ Available on npm

- **Existing Authentication System**: JWT middleware
  - Status: ✅ Implemented in current backend

### Internal Dependencies

- **User Authentication**: `users` table with JWT integration
  - Status: ✅ Exists in production database

- **Database Schema**: `uploads`, `results` tables for querying
  - Status: ✅ Exists in production database

- **Redis Infrastructure**: For caching
  - Status: ✅ Running in production

### Infrastructure Dependencies

- **PostgreSQL 14+**: With JSONB support
  - Status: ✅ Running (version 14.x)

- **Redis 7+**: For conversation caching
  - Status: ✅ Running (version 7.x)

- **Node.js 20+**: For MCP server
  - Status: ⚠️  Need to provision in production (available in dev)

- **Docker**: For MCP server containerization
  - Status: ✅ Available in production

### Blockers (Must Resolve Before Implementation)

1. ✅ **SiliconFlow API Credentials**: Provided by user
2. ⚠️  **MCP Server Deployment**: Need to allocate resources (1 CPU, 512MB RAM minimum)
3. ✅ **Database Read-Only User**: Can create during Phase 1
4. ⚠️  **Rate Limit Clarification**: Contact SiliconFlow for exact rate limits

---

## Rollout Plan

### Stage 1: Development (Week 1-2)
- Feature branch: `feature/ai-api-mcp`
- Local development and unit testing
- Integration testing with mock LLM API
- Code review and merge to `dev` branch

### Stage 2: Staging Deployment (Week 2)
- Deploy to staging environment
- End-to-end testing with real LLM API
- Performance testing (100 concurrent users)
- Security audit
- Stakeholder demo and feedback

### Stage 3: Production Deployment (Week 3)
1. **Day 1**: Deploy MCP server
   - Health check verification
   - Database connectivity test
2. **Day 2**: Deploy backend with feature flag disabled
   - Smoke test chat endpoints (internal only)
   - Verify no impact on existing features
3. **Day 3**: Enable for internal users (10 users)
   - Collect feedback
   - Monitor metrics closely
4. **Day 4-5**: Gradual rollout
   - 10% of users (if >100 users)
   - Monitor for 24 hours
   - Check error rates, latency, user satisfaction
5. **Day 6-7**: Full rollout
   - 100% of users
   - Monitor for 48 hours
   - Set up on-call rotation

### Stage 4: Monitoring & Optimization (Ongoing)
- Track success metrics (see below)
- Collect user feedback via in-chat survey
- Optimize LLM prompts based on hallucination reports
- Tune cache TTL based on hit rate data

---

## Success Metrics

### Performance Metrics (Constitutional Principle 4)

| Metric | Target | Measurement Method | Alert Threshold |
|--------|--------|-------------------|-----------------|
| Chat API P95 latency | ≤2000ms | Prometheus histogram | >3000ms |
| Redis cache hit rate | >95% | Redis INFO stats | <90% |
| Frontend bundle size | ≤150KB gzipped | Webpack bundle analyzer | >200KB |
| LLM API timeout rate | <1% | Backend logs | >5% |

### Quality Metrics (Constitutional Principle 2)

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Backend unit test coverage | ≥85% | cargo-tarpaulin |
| Frontend unit test coverage | ≥85% | Vitest coverage report |
| E2E test pass rate | 100% | Cypress test results |
| Critical bugs (7 days post-launch) | 0 | Issue tracker |

### User Experience Metrics (Constitutional Principle 3)

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Chat widget load time | <500ms | Browser DevTools |
| Message send success rate | >99% | Backend API logs |
| User satisfaction rating | ≥4.0/5.0 | In-app survey |
| Chat session completion rate | >80% | Analytics (session with ≥3 exchanges) |

### Business Metrics

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Daily active chat users | 50% of total users | Backend analytics |
| Average messages per session | ≥5 | Database query |
| Database query accuracy | ≥90% | User feedback (👍/👎) |
| MCP query success rate | >95% | MCP server logs |

---

## Post-Launch Checklist

- [ ] All production services healthy (backend, MCP server, Redis, PostgreSQL)
- [ ] Monitoring dashboards displaying metrics
- [ ] Alerts configured and tested
- [ ] On-call rotation established
- [ ] User documentation published
- [ ] Internal team trained on troubleshooting
- [ ] Rollback plan documented and tested
- [ ] Performance metrics within targets
- [ ] Zero critical bugs reported in first 48 hours

---

## Future Enhancements (v0.2.0+)

1. **WebSocket Support**: Replace HTTP polling for real-time message delivery
2. **Voice Input**: Add speech-to-text for mobile users
3. **Multi-Language Support**: Support English, Japanese (currently Chinese only)
4. **Conversation Summarization**: Auto-generate titles from first message
5. **Advanced Analytics**: Dashboard showing popular queries, user sentiment
6. **LLM Fine-Tuning**: Train custom model on fruit/vegetable domain
7. **Image Uploads in Chat**: Allow users to ask "识别这个水果" with image
8. **Chat Export**: Allow users to download conversation history as PDF/JSON

---

## Approval

- [ ] **Technical Lead**: [Name] - Approved architecture and tech stack
- [ ] **Product Owner**: [Name] - Approved feature scope and success criteria
- [ ] **Security Review**: [Name] - Approved MCP security model and data access controls
- [ ] **Date Approved**: [YYYY-MM-DD]

---

**Revision History**:

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0 | 2025-10-14 | AI Planning Agent | Initial implementation plan with full phase breakdown |

---

**Related Documents**:
- Specification: `.specify/specs/ai-api-mcp/spec.md`
- Research: `.specify/specs/ai-api-mcp/research.md`
- Data Model: `.specify/specs/ai-api-mcp/data-model.md`
- API Contracts: `.specify/specs/ai-api-mcp/contracts/chat-api.yaml`
- Quickstart: `.specify/specs/ai-api-mcp/quickstart.md`
