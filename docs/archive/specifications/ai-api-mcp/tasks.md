# Implementation Tasks: AI Assistant Chat Module

**Plan Reference**: `.specify/specs/ai-api-mcp/plan.md`
**Generated**: 2025-10-14
**Status**: Not Started

## Overview

This document breaks down the AI Assistant Chat Module implementation into discrete, executable tasks organized by user story. The feature enables users to interact with an intelligent AI assistant through a floating chat widget that can query detection data using natural language.

### User Stories (Priority Order)

1. **US1**: User initiates chat conversation (P1 - MVP)
2. **US2**: AI assistant processes user query (P1 - MVP)
3. **US3**: User queries detection results via natural language (P1 - MVP)
4. **US4**: User reviews conversation history (P2)
5. **US5**: System manages conversation sessions (P2)
6. **US6**: Administrator configures LLM API settings (P3)

### Implementation Strategy

- **MVP Scope**: US1 + US2 + US3 (core chat functionality with database querying)
- **Incremental Delivery**: Each user story is independently testable
- **Testing Approach**: Unit tests (≥85% coverage) + Integration tests + E2E tests per constitutional requirements

---

## Phase 1: Setup & Infrastructure

### T001: Configure Project Environment [P]
**Story**: Foundation
**Description**: Set up environment variables and configuration files for chat module
**Files**: `backend/.env`, `frontend/.env`, `mcp-config.json`
**Dependencies**: None
**Acceptance Criteria**:
- [X] Backend `.env` includes: LLM_API_URL, LLM_API_KEY, LLM_MODEL, MCP_SERVER_URL
- [X] Frontend `.env` includes: VITE_CHAT_POLLING_INTERVAL
- [X] MCP configuration file created with database connection string
- [X] All sensitive keys documented in `.env.example`

**Effort**: 1h
**Status**: ✅ COMPLETED

### T002: Install Backend Dependencies [P]
**Story**: Foundation
**Description**: Add Rust crates for LLM client, JSON-RPC, and enhanced HTTP support
**Files**: `backend/Cargo.toml`
**Dependencies**: None
**Acceptance Criteria**:
- [X] `reqwest` with json feature added for LLM HTTP client
- [X] `uuid` with serde feature for session/message IDs
- [X] `serde_json` for JSON-RPC and JSONB handling
- [X] `cargo build` completes without errors

**Effort**: 0.5h
**Status**: ✅ COMPLETED

### T003: Install Frontend Dependencies [P]
**Story**: Foundation
**Description**: Add Pinia state management for chat widget
**Files**: `frontend/package.json`
**Dependencies**: None
**Acceptance Criteria**:
- [X] `pinia` package installed
- [X] `npm install` completes successfully
- [X] No version conflicts with existing dependencies

**Effort**: 0.5h
**Status**: ✅ COMPLETED

### T004: Install MCP Server [P]
**Story**: Foundation
**Description**: Install Model Context Protocol server for database querying
**Files**: System-wide or Docker container
**Dependencies**: None
**Acceptance Criteria**:
- [X] `@modelcontextprotocol/server-postgres` installation documented
- [X] MCP server setup guide created (docs/MCP_SERVER.md)
- [X] Configuration file ready for deployment

**Effort**: 1h
**Status**: ✅ COMPLETED

---

## Phase 2: Database Foundation

### T005: Create Database Migrations
**Story**: Foundation (Blocking)
**Description**: Create SQL migration scripts for chat tables
**Files**: `backend/migrations/001_create_chat_sessions.sql`, `backend/migrations/002_create_chat_messages.sql`, `backend/migrations/003_create_mcp_readonly_user.sql`
**Dependencies**: None
**Acceptance Criteria**:
- [X] Migration 001 creates `chat_sessions` table with indexes
- [X] Migration 002 creates `chat_messages` table with GIN index on metadata
- [X] Migration 003 creates `mcp_readonly` role with SELECT-only permissions
- [X] Migrations include rollback scripts
- [X] All migrations tested in local database

**Effort**: 2h

### T006: Apply Database Migrations
**Story**: Foundation (Blocking)
**Description**: Execute migrations to create chat schema in PostgreSQL
**Files**: Database
**Dependencies**: T005
**Acceptance Criteria**:
- [ ] `chat_sessions` table exists with correct schema
- [ ] `chat_messages` table exists with correct schema
- [ ] `mcp_readonly` user exists with correct permissions
- [ ] Foreign key constraints enforced
- [ ] Indexes created for performance

**Effort**: 0.5h

---

## Phase 3: MCP Server Setup

### T007: Configure MCP Server
**Story**: Foundation (Blocking)
**Description**: Create MCP configuration for read-only database access
**Files**: `mcp-config.json`, `docker-compose.yml` (optional)
**Dependencies**: T006
**Acceptance Criteria**:
- [ ] Configuration specifies `mcp_readonly` credentials
- [ ] Table whitelist includes only `uploads` and `results`
- [ ] Max rows per query set to 1000
- [ ] Query timeout set to 5 seconds

**Effort**: 1h

### T008: Deploy MCP Server
**Story**: Foundation (Blocking)
**Description**: Start MCP server as standalone service or Docker container
**Files**: Docker container or system service
**Dependencies**: T007
**Acceptance Criteria**:
- [ ] MCP server running on port 3000
- [ ] Health check endpoint returns 200 OK
- [ ] `list_tables` RPC call returns uploads and results tables
- [ ] Server logs to stdout/file for debugging

**Effort**: 1h

### T009: Test MCP Security
**Story**: Foundation (Blocking)
**Description**: Verify MCP server enforces read-only access and table whitelist
**Files**: Test scripts or manual curl commands
**Dependencies**: T008
**Acceptance Criteria**:
- [ ] `DELETE FROM results` query returns error
- [ ] `SELECT * FROM users` query returns error (table not whitelisted)
- [ ] `SELECT * FROM results LIMIT 1` query succeeds
- [ ] Query with >1000 rows returns error or truncates
- [ ] SQL injection attempts fail safely

**Effort**: 1h

---

## Phase 4: Backend Core Models & Services

### T010: Define Chat Data Models
**Story**: US1, US2 (Shared)
**Description**: Create Rust structs for chat sessions, messages, and LLM requests/responses
**Files**: `backend/src/models.rs`
**Dependencies**: T002
**Acceptance Criteria**:
- [ ] `ChatSession` struct with id, user_id, title, status, timestamps
- [ ] `ChatMessage` struct with id, session_id, role, content, metadata
- [ ] `LLMRequest` and `LLMResponse` structs matching SiliconFlow API format
- [ ] `MCPQuery` and `MCPResult` structs for JSON-RPC communication
- [ ] All structs derive `Serialize`, `Deserialize`, `Debug`
- [ ] Unit tests for serialization/deserialization

**Effort**: 2h

### T011: Implement LLM Client Service [P]
**Story**: US2 (Core)
**Description**: Create HTTP client for SiliconFlow API with retry logic
**Files**: `backend/src/services/llm_client.rs`
**Dependencies**: T002, T010
**Acceptance Criteria**:
- [ ] `LLMClient` struct with `new()` and `send_message()` methods
- [ ] HTTP client uses connection pooling (10 idle connections)
- [ ] Request timeout set to 10 seconds
- [ ] Exponential backoff retry (3 attempts: 1s, 2s, 4s delays)
- [ ] Handles 401, 429, 503 errors appropriately
- [ ] Unit tests with mocked HTTP responses

**Effort**: 3h

### T012: Implement MCP Client Service [P]
**Story**: US3 (Core)
**Description**: Create JSON-RPC 2.0 client for MCP server communication
**Files**: `backend/src/services/mcp_client.rs`
**Dependencies**: T002, T008, T010
**Acceptance Criteria**:
- [ ] `MCPClient` struct with `query()`, `describe_table()`, `list_tables()` methods
- [ ] JSON-RPC 2.0 request/response handling
- [ ] Timeout set to 7 seconds (5s query + 2s overhead)
- [ ] Error mapping for MCP error codes (-32001 to -32005)
- [ ] Unit tests with mocked MCP responses

**Effort**: 3h

### T013: Implement Conversation Cache Service [P]
**Story**: US2, US4 (Shared)
**Description**: Create Redis caching layer for conversation history
**Files**: `backend/src/cache.rs`
**Dependencies**: T010
**Acceptance Criteria**:
- [ ] `save_message()` function adds message to Redis list
- [ ] `get_recent_messages()` retrieves last 20 messages with cache-aside pattern
- [ ] `save_summary()` stores conversation summary
- [ ] Key patterns: `chat:session:{id}:messages`, `chat:session:{id}:summary`, `chat:user:{id}:active_session`
- [ ] TTL enforcement: 86400s for messages/summary, 1800s for active session
- [ ] Unit tests verify caching logic

**Effort**: 2h

---

## Phase 5: Backend API Routes (US1 - User Initiates Chat)

### T014: Implement Create Session Endpoint
**Story**: US1
**Description**: POST /api/v1/chat/sessions endpoint to create new conversation
**Files**: `backend/src/routes/chat.rs`
**Dependencies**: T006, T010
**Acceptance Criteria**:
- [ ] Endpoint validates JWT token and extracts user_id
- [ ] Creates row in `chat_sessions` table with 'active' status
- [ ] Generates UUID for session_id
- [ ] Auto-generates title from request or sets NULL
- [ ] Returns JSON with session_id and created_at
- [ ] Returns 201 Created on success, 401 Unauthorized if invalid token
- [ ] Unit test with mock database

**Effort**: 2h

### T015: Implement List Sessions Endpoint [P]
**Story**: US1, US5
**Description**: GET /api/v1/chat/sessions endpoint with pagination and filtering
**Files**: `backend/src/routes/chat.rs`
**Dependencies**: T006, T010
**Acceptance Criteria**:
- [ ] Validates JWT and filters by user_id
- [ ] Supports query params: status, limit (default 10), offset (default 0)
- [ ] Returns array of session summaries with message_count and last_message_at
- [ ] Includes `total` count and pagination info
- [ ] Returns 200 OK with data, 401 Unauthorized if invalid token
- [ ] Unit test with mock database

**Effort**: 2h

### T016: Implement Get Session Details Endpoint [P]
**Story**: US1
**Description**: GET /api/v1/chat/sessions/{session_id} endpoint for session details
**Files**: `backend/src/routes/chat.rs`
**Dependencies**: T006, T010
**Acceptance Criteria**:
- [ ] Validates JWT and verifies session belongs to user
- [ ] Returns session details with message_count
- [ ] Returns 200 OK with data, 404 Not Found if session doesn't exist
- [ ] Returns 401 if session belongs to different user
- [ ] Unit test with mock database

**Effort**: 1.5h

---

## Phase 6: Backend API Routes (US2 - AI Processes Query)

### T017: Implement Send Message Endpoint
**Story**: US2
**Description**: POST /api/v1/chat/message endpoint with LLM integration
**Files**: `backend/src/routes/chat.rs`
**Dependencies**: T006, T010, T011, T013
**Acceptance Criteria**:
- [ ] Validates JWT, session_id, and message content (1-10000 chars)
- [ ] Saves user message to `chat_messages` table
- [ ] Retrieves last 20 messages from Redis/PostgreSQL for context
- [ ] Builds LLM request with conversation history
- [ ] Calls `llm_client.send_message()` with context
- [ ] Saves assistant response to `chat_messages` with metadata (tokens, timing)
- [ ] Caches both messages in Redis
- [ ] Returns JSON with message_id, content, timestamp, metadata
- [ ] Returns 200 OK on success, 400 Bad Request for invalid input
- [ ] Returns 503 Service Unavailable if LLM API fails
- [ ] Rate limiting: 20 requests/minute per user
- [ ] Unit test with mocked LLM client

**Effort**: 4h

### T018: Implement Context Window Management
**Story**: US2
**Description**: Add logic to summarize conversation after 10 exchanges
**Files**: `backend/src/routes/chat.rs`, `backend/src/services/llm_client.rs`
**Dependencies**: T011, T013, T017
**Acceptance Criteria**:
- [ ] After 20 messages (10 exchanges), trigger summarization
- [ ] Send summarization request to LLM: "Summarize key points in 100 words"
- [ ] Store summary in Redis with key `chat:session:{id}:summary`
- [ ] Subsequent requests include summary + last 10 exchanges
- [ ] Unit test verifies summarization trigger

**Effort**: 2h

---

## Phase 7: Backend API Routes (US3 - Natural Language Database Queries)

### T019: Integrate MCP into Message Handling
**Story**: US3
**Description**: Add MCP query execution to send_message endpoint
**Files**: `backend/src/routes/chat.rs`
**Dependencies**: T012, T017
**Acceptance Criteria**:
- [ ] LLM system prompt includes MCP tool descriptions
- [ ] LLM system prompt includes database schema summary (uploads, results)
- [ ] Backend parses LLM tool calls for MCP query requests
- [ ] Calls `mcp_client.query()` with SQL and parameters
- [ ] Injects `user_id` filter into all SQL queries automatically
- [ ] LLM receives query results and synthesizes natural language response
- [ ] Metadata includes: mcp_query_executed, mcp_query_text, query_results_count
- [ ] Returns 200 OK with query results in response
- [ ] Integration test with real MCP server and test database

**Effort**: 4h

### T020: Implement User Context Injection
**Story**: US3
**Description**: Ensure all MCP queries filter by authenticated user's data
**Files**: `backend/src/services/mcp_client.rs`
**Dependencies**: T012, T019
**Acceptance Criteria**:
- [ ] All SQL queries include `WHERE user_id = $user_id` clause
- [ ] User ID extracted from JWT token
- [ ] SQL parsing validates user_id parameter present
- [ ] Prevents cross-user data leakage
- [ ] Unit test verifies user_id injection

**Effort**: 2h

---

## Phase 8: Backend API Routes (US4 - Conversation History)

### T021: Implement Get History Endpoint
**Story**: US4
**Description**: GET /api/v1/chat/history/{session_id} endpoint with pagination
**Files**: `backend/src/routes/chat.rs`
**Dependencies**: T006, T010, T013
**Acceptance Criteria**:
- [ ] Validates JWT and session ownership
- [ ] Supports query params: limit (default 20), offset (default 0)
- [ ] Tries Redis cache first, falls back to PostgreSQL
- [ ] Returns array of messages with id, role, content, created_at
- [ ] Includes `total` message count and `has_more` boolean
- [ ] Returns 200 OK with data, 404 Not Found if session doesn't exist
- [ ] Unit test with mock cache and database

**Effort**: 2.5h

### T022: Implement Polling Endpoint [P]
**Story**: US4
**Description**: GET /api/v1/chat/messages/new endpoint for HTTP polling
**Files**: `backend/src/routes/chat.rs`
**Dependencies**: T006, T010
**Acceptance Criteria**:
- [ ] Validates JWT, session_id query param, optional after (message_id) param
- [ ] Returns messages created after specified message_id
- [ ] Returns empty array if no new messages (not an error)
- [ ] Returns 200 OK with array of messages
- [ ] Optimized query using `WHERE id > $after` index scan
- [ ] Unit test with mock database

**Effort**: 1.5h

---

## Phase 9: Frontend Core Components (US1 - Chat Widget UI)

### T023: Create Pinia Chat Store
**Story**: US1
**Description**: Implement global state management for chat module
**Files**: `frontend/src/stores/chatStore.ts`
**Dependencies**: T003
**Acceptance Criteria**:
- [ ] Store exports `useChatStore()` function
- [ ] State: `currentSession`, `messages`, `isLoading`, `unreadCount`
- [ ] Actions: `addMessage()`, `clearMessages()`, `markAsRead()`
- [ ] Uses Composition API (ref, computed)
- [ ] TypeScript types for all state and actions
- [ ] Unit test with Vitest

**Effort**: 2h

### T024: Create ChatBubble Component [P]
**Story**: US1
**Description**: Floating chat bubble button (minimized state)
**Files**: `frontend/src/components/ChatBubble.vue`
**Dependencies**: T003
**Acceptance Criteria**:
- [ ] Uses `<script setup>` syntax (constitutional requirement)
- [ ] Displays unread message count badge if >0
- [ ] Positioned fixed bottom-right (20px from edges)
- [ ] Click emits 'click' event to parent
- [ ] Styled with Tailwind + Element Plus colors
- [ ] Responsive: visible on mobile (≥375px) and desktop
- [ ] Unit test with Vitest

**Effort**: 2h

### T025: Create ChatWindow Component [P]
**Story**: US1
**Description**: Expanded chat interface container
**Files**: `frontend/src/components/ChatWindow.vue`
**Dependencies**: T003, T023
**Acceptance Criteria**:
- [ ] Uses `<script setup>` syntax
- [ ] Fixed position bottom-right, width 400px, max-height 600px
- [ ] Header with title and close button
- [ ] Scrollable message area using `el-scrollbar`
- [ ] Input area with send button
- [ ] Emits 'send' event with message content
- [ ] Emits 'close' event when close button clicked
- [ ] Styled consistently with Element Plus design tokens
- [ ] Unit test with Vitest

**Effort**: 3h

### T026: Create ChatMessage Component [P]
**Story**: US1
**Description**: Individual message display (user vs assistant styling)
**Files**: `frontend/src/components/ChatMessage.vue`
**Dependencies**: T003
**Acceptance Criteria**:
- [ ] Uses `<script setup>` syntax
- [ ] Props: `message` (with role, content, created_at)
- [ ] User messages: right-aligned, blue background
- [ ] Assistant messages: left-aligned, gray background
- [ ] Displays timestamp in relative format ("2 minutes ago")
- [ ] Markdown rendering for assistant responses (optional v0.2.0)
- [ ] Unit test with Vitest

**Effort**: 2h

### T027: Create ChatInput Component [P]
**Story**: US1
**Description**: Text input with send button
**Files**: `frontend/src/components/ChatInput.vue`
**Dependencies**: T003
**Acceptance Criteria**:
- [ ] Uses `<script setup>` syntax
- [ ] `el-input` with type="textarea", autosize
- [ ] Send button disabled if input empty or loading
- [ ] Enter key sends message (Shift+Enter for newline)
- [ ] Input clears after send
- [ ] Emits 'send' event with message content
- [ ] Input sanitization (trim whitespace, max 10000 chars)
- [ ] Unit test with Vitest

**Effort**: 1.5h

### T028: Create ChatTypingIndicator Component [P]
**Story**: US2
**Description**: "AI is typing..." loading animation
**Files**: `frontend/src/components/ChatTypingIndicator.vue`
**Dependencies**: T003
**Acceptance Criteria**:
- [ ] Uses `<script setup>` syntax
- [ ] Animated ellipsis (three bouncing dots)
- [ ] Styled consistently with ChatMessage
- [ ] Visible only when `isLoading` prop is true
- [ ] Smooth fade-in/out transition

**Effort**: 1h

### T029: Create ChatWidget Container
**Story**: US1
**Description**: Main component integrating ChatBubble and ChatWindow
**Files**: `frontend/src/components/ChatWidget.vue`
**Dependencies**: T024, T025, T026, T027, T028
**Acceptance Criteria**:
- [ ] Uses `<script setup>` syntax
- [ ] State: `isExpanded` (boolean)
- [ ] Toggles between ChatBubble (minimized) and ChatWindow (expanded)
- [ ] Transition animation between states (fade)
- [ ] Global component (single instance in App.vue)
- [ ] Unit test with Vitest

**Effort**: 2h

---

## Phase 10: Frontend API Integration (US2 - Send Message)

### T030: Create Chat API Client
**Story**: US1, US2
**Description**: HTTP client methods for chat endpoints
**Files**: `frontend/src/api/chat.ts`
**Dependencies**: None
**Acceptance Criteria**:
- [ ] `createSession()` - POST /api/v1/chat/sessions
- [ ] `getSessions()` - GET /api/v1/chat/sessions with query params
- [ ] `getSession()` - GET /api/v1/chat/sessions/{id}
- [ ] `sendMessage()` - POST /api/v1/chat/message
- [ ] `getHistory()` - GET /api/v1/chat/history/{session_id}
- [ ] `pollNewMessages()` - GET /api/v1/chat/messages/new
- [ ] Uses existing axios instance with JWT interceptor
- [ ] TypeScript types for all requests/responses
- [ ] Error handling with try/catch

**Effort**: 2h

### T031: Create useChat Composable
**Story**: US2
**Description**: Composable for chat state management and API calls
**Files**: `frontend/src/composables/useChat.ts`
**Dependencies**: T023, T030
**Acceptance Criteria**:
- [ ] Exports `useChat()` function
- [ ] Methods: `sendMessage()`, `loadHistory()`, `createSession()`
- [ ] Integrates with Pinia chatStore
- [ ] Handles loading states (sets `isLoading` in store)
- [ ] Error handling with Toast notifications
- [ ] TypeScript types for all methods

**Effort**: 2.5h

### T032: Integrate ChatWidget with Backend [P]
**Story**: US2
**Description**: Connect frontend components to backend APIs
**Files**: `frontend/src/components/ChatWidget.vue`
**Dependencies**: T029, T031
**Acceptance Criteria**:
- [ ] On first open, creates new session via `createSession()`
- [ ] On send button click, calls `sendMessage()` from useChat
- [ ] Displays user message immediately (optimistic UI)
- [ ] Displays assistant response when API returns
- [ ] Shows ChatTypingIndicator while waiting for response
- [ ] Error handling: displays Toast notification on failure
- [ ] Integration test with mocked API

**Effort**: 2h

---

## Phase 11: Frontend Polling (US4 - Real-Time Updates)

### T033: Create useChatPolling Composable
**Story**: US4
**Description**: HTTP polling logic for new messages
**Files**: `frontend/src/composables/useChatPolling.ts`
**Dependencies**: T030
**Acceptance Criteria**:
- [ ] Polls `/api/v1/chat/messages/new` every 2 seconds
- [ ] Starts polling when chat window opened
- [ ] Stops polling when chat window closed
- [ ] Tracks last message ID to query only new messages
- [ ] Updates Pinia store with new messages
- [ ] Handles errors silently (logs to console, no Toast)
- [ ] Uses `onMounted` and `onUnmounted` lifecycle hooks
- [ ] Unit test with Vitest and fake timers

**Effort**: 2h

### T034: Integrate Polling into ChatWidget [P]
**Story**: US4
**Description**: Add polling to ChatWidget for near-real-time updates
**Files**: `frontend/src/components/ChatWidget.vue`
**Dependencies**: T029, T033
**Acceptance Criteria**:
- [ ] Calls `useChatPolling()` composable
- [ ] Starts polling when `isExpanded` becomes true
- [ ] Stops polling when `isExpanded` becomes false
- [ ] New messages append to chat window automatically
- [ ] Scroll to bottom when new message arrives
- [ ] Integration test verifies polling behavior

**Effort**: 1h

---

## Phase 12: App Integration (US1)

### T035: Add ChatWidget to App.vue
**Story**: US1
**Description**: Integrate global ChatWidget component
**Files**: `frontend/src/App.vue`
**Dependencies**: T029
**Acceptance Criteria**:
- [ ] Import `ChatWidget` component
- [ ] Add `<ChatWidget />` to template (outside main router-view)
- [ ] Component visible on all pages
- [ ] z-index set high (9999) to appear above all content
- [ ] No layout shift or interference with existing UI

**Effort**: 0.5h

### T036: Configure Pinia in Main App
**Story**: US1
**Description**: Initialize Pinia in Vue app
**Files**: `frontend/src/main.ts`
**Dependencies**: T003, T023
**Acceptance Criteria**:
- [ ] Import `createPinia` from 'pinia'
- [ ] Create Pinia instance: `const pinia = createPinia()`
- [ ] Register with app: `app.use(pinia)`
- [ ] Pinia devtools available in browser

**Effort**: 0.5h

---

## Phase 13: Testing (Constitutional Requirement)

### T037: Write Backend Unit Tests
**Story**: All
**Description**: Unit tests for backend services achieving ≥85% coverage
**Files**: `backend/src/routes/chat_test.rs`, `backend/src/services/llm_client_test.rs`, `backend/src/services/mcp_client_test.rs`
**Dependencies**: T010, T011, T012, T013, T014-T022
**Acceptance Criteria**:
- [ ] Test LLM client: request serialization, retry logic, error handling
- [ ] Test MCP client: JSON-RPC formatting, query execution, user_id injection
- [ ] Test chat routes: session creation, message handling, history retrieval
- [ ] Mocked dependencies (database, HTTP clients)
- [ ] Code coverage report shows ≥85% for chat module
- [ ] All tests pass with `cargo test`

**Effort**: 8h

### T038: Write Frontend Unit Tests [P]
**Story**: All
**Description**: Unit tests for Vue components and composables achieving ≥85% coverage
**Files**: `frontend/tests/unit/ChatWidget.spec.ts`, `frontend/tests/unit/useChat.spec.ts`, `frontend/tests/unit/useChatPolling.spec.ts`
**Dependencies**: T023-T034
**Acceptance Criteria**:
- [ ] Test ChatWidget: toggle expand/collapse, send message, display messages
- [ ] Test useChat composable: sendMessage, loadHistory, error handling
- [ ] Test useChatPolling: start/stop polling, message updates
- [ ] Mocked API calls with Vitest
- [ ] Code coverage report shows ≥85% for chat module
- [ ] All tests pass with `npm run test`

**Effort**: 6h

### T039: Write Integration Tests
**Story**: US2, US3
**Description**: Integration tests for backend ↔ LLM and backend ↔ MCP
**Files**: `backend/tests/integration/chat_llm_test.rs`, `backend/tests/integration/chat_mcp_test.rs`
**Dependencies**: T011, T012, T017, T019
**Acceptance Criteria**:
- [ ] Test backend sends valid requests to LLM API (use mocked server)
- [ ] Test backend handles LLM errors (timeout, 503, rate limit)
- [ ] Test backend sends valid JSON-RPC requests to MCP server
- [ ] Test MCP query execution with test database
- [ ] Test user_id filtering prevents data leakage
- [ ] All integration tests pass in CI

**Effort**: 6h

### T040: Write E2E Tests
**Story**: US1, US2, US3
**Description**: Cypress tests for complete chat flow
**Files**: `tests/e2e/cypress/e2e/chat.cy.ts`
**Dependencies**: T035, T036
**Acceptance Criteria**:
- [ ] Test: User clicks chat bubble → window opens → welcome message displayed
- [ ] Test: User types message → sends → receives AI response
- [ ] Test: User asks database question → MCP executes → results displayed
- [ ] Test: Error handling - LLM timeout shows Toast notification
- [ ] Test: Chat widget responsive on mobile viewport (375px)
- [ ] All E2E tests pass with `npx cypress run`

**Effort**: 8h

---

## Phase 14: Documentation & Deployment

### T041: Update API Documentation [P]
**Story**: All
**Description**: Add chat endpoints to docs/API.md
**Files**: `docs/API.md`
**Dependencies**: T014-T022
**Acceptance Criteria**:
- [ ] Document all 6 chat endpoints with request/response examples
- [ ] Include error codes and rate limits
- [ ] Add authentication requirements (JWT)
- [ ] Update table of contents

**Effort**: 2h

### T042: Create User Documentation [P]
**Story**: US1
**Description**: Document how to use chat widget for end users
**Files**: `docs/USER_GUIDE_CHAT.md`
**Dependencies**: T035
**Acceptance Criteria**:
- [ ] Explain how to open chat widget
- [ ] Provide example queries users can ask
- [ ] Include FAQ section
- [ ] Screenshots of chat interface

**Effort**: 2h

### T043: Set Up Monitoring
**Story**: All
**Description**: Add Prometheus metrics for chat module
**Files**: `backend/src/metrics.rs`, Prometheus configuration
**Dependencies**: T017, T019
**Acceptance Criteria**:
- [ ] Metric: `chat_requests_total` (counter by endpoint)
- [ ] Metric: `chat_latency_seconds` (histogram by endpoint)
- [ ] Metric: `mcp_queries_total` (counter)
- [ ] Metric: `llm_api_errors_total` (counter by error type)
- [ ] Metrics exposed at `/metrics` endpoint
- [ ] Grafana dashboard created (optional)

**Effort**: 4h

### T044: Deploy to Staging
**Story**: All
**Description**: Deploy chat module to staging environment
**Files**: Deployment scripts, Kubernetes manifests
**Dependencies**: T001-T043
**Acceptance Criteria**:
- [ ] MCP server deployed and healthy
- [ ] Backend deployed with chat routes enabled
- [ ] Frontend deployed with chat widget visible
- [ ] Smoke tests pass in staging
- [ ] No errors in logs

**Effort**: 3h

### T045: Production Deployment
**Story**: All
**Description**: Gradual rollout to production (10% → 100%)
**Files**: Feature flags, deployment scripts
**Dependencies**: T044
**Acceptance Criteria**:
- [ ] Feature flag implemented for chat module
- [ ] Deployed to production with flag disabled
- [ ] Enabled for internal users (10 users)
- [ ] Monitored for 24 hours with no critical issues
- [ ] Gradually rolled out: 10% → 50% → 100%
- [ ] Post-launch monitoring shows metrics within targets

**Effort**: 4h

---

## Dependency Graph

```
Phase 1: Setup (All parallel)
T001, T002, T003, T004

Phase 2: Database (Sequential)
T005 → T006

Phase 3: MCP (Sequential)
T007 → T008 → T009

Phase 4: Backend Services (Parallel after T002, T010)
T010 → T011 [P], T012 [P], T013 [P]

Phase 5: US1 Routes (After T006, T010)
T014, T015 [P], T016 [P]

Phase 6: US2 Routes (After T011, T013, T014)
T017 → T018

Phase 7: US3 Integration (After T012, T017)
T019 → T020

Phase 8: US4 Routes (After T013)
T021, T022 [P]

Phase 9: Frontend Components (Parallel after T003, T023)
T023 → T024 [P], T025 [P], T026 [P], T027 [P], T028 [P] → T029

Phase 10: API Integration (After T023, T029, T030)
T030 → T031 → T032

Phase 11: Polling (After T030, T029)
T033 → T034

Phase 12: App Integration (After T029, T023)
T035, T036

Phase 13: Testing (After implementation phases)
T037, T038 [P], T039, T040

Phase 14: Deployment (After testing)
T041 [P], T042 [P], T043 → T044 → T045
```

---

## Parallel Execution Opportunities

### Setup Phase (4 tasks in parallel)
- T001: Configure environment variables
- T002: Install backend dependencies
- T003: Install frontend dependencies
- T004: Install MCP server

### Backend Services (3 tasks in parallel)
- T011: Implement LLM client
- T012: Implement MCP client
- T013: Implement conversation cache

### US1 Routes (2 tasks in parallel)
- T015: List sessions endpoint
- T016: Get session details endpoint

### US4 Routes (2 tasks in parallel)
- T021: Get history endpoint
- T022: Polling endpoint

### Frontend Components (5 tasks in parallel)
- T024: ChatBubble component
- T025: ChatWindow component
- T026: ChatMessage component
- T027: ChatInput component
- T028: ChatTypingIndicator component

### Testing (2 tasks in parallel)
- T037: Backend unit tests
- T038: Frontend unit tests

### Documentation (2 tasks in parallel)
- T041: Update API documentation
- T042: Create user documentation

**Total Parallel Opportunities**: 19 tasks can be executed in parallel

---

## MVP Scope (User Stories 1-3)

**Minimum Viable Product** includes:
- US1: User initiates chat conversation (T014-T016, T023-T029, T035-T036)
- US2: AI assistant processes query (T011, T017-T018, T030-T032)
- US3: Natural language database queries (T012, T019-T020)
- Foundation: T001-T013 (Setup, Database, MCP)
- Testing: T037-T040 (Constitutional requirement)

**MVP Task Count**: ~35 tasks
**Estimated Effort**: 70-80 person-hours (2 weeks for 1 developer, 1 week for 2 developers)

---

## Progress Tracking

| Task ID | Story | Status | Assignee | Started | Completed | Notes |
|---------|-------|--------|----------|---------|-----------|-------|
| T001 | Foundation | Not Started | - | - | - | - |
| T002 | Foundation | Not Started | - | - | - | - |
| T003 | Foundation | Not Started | - | - | - | - |
| ... | ... | ... | ... | ... | ... | ... |

---

## Definition of Done

A task is considered complete when:
1. ✅ All acceptance criteria checked
2. ✅ Code passes `cargo clippy` (backend) or ESLint (frontend)
3. ✅ Unit tests written and passing (if applicable)
4. ✅ Code reviewed and approved
5. ✅ Changes merged to feature branch
6. ✅ Documentation updated (if applicable)
7. ✅ Constitutional principles verified

---

## Risk Mitigation Tasks

### High Priority Risks

**Risk: LLM API Cost Escalation**
- Mitigation Task: Implement request logging and cost tracking (add to T043)
- Mitigation Task: Add rate limiting per user (included in T017)

**Risk: MCP Security Vulnerabilities**
- Mitigation Task: Security audit of MCP queries (T009)
- Mitigation Task: SQL injection prevention tests (T039)

**Risk: Performance Degradation**
- Mitigation Task: Load testing with 100 concurrent users (add after T040)
- Mitigation Task: Monitor P95 latency metrics (T043)

---

## Summary

**Total Tasks**: 45
**Estimated Total Effort**: 110-120 person-hours (~3 weeks for 1 developer)

### Breakdown by Phase:
- **Phase 1 (Setup)**: 4 tasks, 3h
- **Phase 2 (Database)**: 2 tasks, 2.5h
- **Phase 3 (MCP)**: 3 tasks, 3h
- **Phase 4 (Services)**: 4 tasks, 10h
- **Phase 5 (US1 Routes)**: 3 tasks, 5.5h
- **Phase 6 (US2 Routes)**: 2 tasks, 6h
- **Phase 7 (US3 Integration)**: 2 tasks, 6h
- **Phase 8 (US4 Routes)**: 2 tasks, 4h
- **Phase 9 (Frontend UI)**: 7 tasks, 15.5h
- **Phase 10 (API Integration)**: 3 tasks, 6.5h
- **Phase 11 (Polling)**: 2 tasks, 3h
- **Phase 12 (App Integration)**: 2 tasks, 1h
- **Phase 13 (Testing)**: 4 tasks, 28h
- **Phase 14 (Deployment)**: 5 tasks, 15h

### By User Story:
- **US1 (Initiate Chat)**: 13 tasks, ~25h
- **US2 (Process Query)**: 8 tasks, ~20h
- **US3 (Database Queries)**: 4 tasks, ~12h
- **US4 (History)**: 4 tasks, ~7h
- **US5 (Session Management)**: 1 task, ~2h (T015 shared)
- **US6 (Admin Config)**: 0 tasks (handled via environment variables in T001)
- **Foundation**: 13 tasks, ~18.5h
- **Testing**: 4 tasks, ~28h
- **Deployment**: 5 tasks, ~15h

---

**Last Updated**: 2025-10-14
**Next Steps**: Begin with Phase 1 (Setup) tasks T001-T004 in parallel, then proceed sequentially through database and MCP setup.
