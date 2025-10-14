# Technical Research: AI Assistant Chat Module

**Feature**: AI Assistant Chat Module (ai-api-mcp)
**Research Date**: 2025-10-14
**Status**: Complete

## Overview

This document consolidates technical research findings to guide implementation decisions for the AI Assistant Chat Module. All critical unknowns from the specification have been researched and resolved.

---

## Research Topic 1: SiliconFlow API Integration with Hunyuan-MT-7B

### Decision
Integrate SiliconFlow API using **Anthropic-compatible message format** with the Tencent Hunyuan-MT-7B model.

### Rationale
- **Chinese Language Optimization**: Hunyuan-MT-7B is specifically designed for Chinese natural language understanding, making it ideal for our user base
- **Cost-Effectiveness**: SiliconFlow offers competitive pricing compared to OpenAI/Anthropic while maintaining quality for Chinese text
- **Message Format Compatibility**: Uses Anthropic's `/v1/messages` endpoint format, simplifying integration
- **Token Capacity**: 8192 max tokens provides sufficient context window for conversation history + database query results
- **Production Ready**: SiliconFlow is a stable Chinese AI infrastructure provider with good uptime

### API Specifications
```rust
// Endpoint: https://api.siliconflow.cn/v1/messages
// Method: POST
// Headers:
//   Authorization: Bearer {API_KEY}
//   Content-Type: application/json

// Request Format (Anthropic-compatible):
{
  "model": "tencent/Hunyuan-MT-7B",
  "messages": [
    {"role": "user", "content": "用户消息"},
    {"role": "assistant", "content": "AI响应"},
    {"role": "user", "content": "新消息"}
  ],
  "max_tokens": 8192,
  "temperature": 0.7,  // Optional: 0.0-1.0 for response creativity
  "top_p": 0.9         // Optional: nucleus sampling parameter
}

// Response Format:
{
  "id": "msg_xxx",
  "type": "message",
  "role": "assistant",
  "content": [
    {"type": "text", "text": "AI assistant's response"}
  ],
  "model": "tencent/Hunyuan-MT-7B",
  "stop_reason": "end_turn",
  "usage": {
    "input_tokens": 125,
    "output_tokens": 256
  }
}
```

### Implementation Notes
- Use `reqwest` crate in Rust for HTTP client with connection pooling
- Implement exponential backoff retry (3 attempts, 1s/2s/4s delays)
- Set timeout to 10 seconds for API calls
- Parse response using `serde_json` with strongly-typed structs
- Handle rate limiting (429 status) with backoff strategy

### Alternatives Considered
1. **OpenAI GPT-4**: Higher quality but 3x cost, weaker Chinese language performance
2. **Anthropic Claude**: Excellent reasoning but requires separate API account, higher cost
3. **Local Ollama**: Zero API cost but requires GPU infrastructure (16GB+ VRAM), lower quality
4. **Zhipu GLM-4**: Strong Chinese support but less established infrastructure

---

## Research Topic 2: Model Context Protocol (MCP) Implementation

### Decision
Use **`@modelcontextprotocol/server-postgres`** (Node.js package) for MCP server implementation, deployed as standalone service.

### Rationale
- **Official Implementation**: Maintained by Anthropic as reference implementation
- **PostgreSQL Native Support**: Built specifically for PostgreSQL databases
- **Security Features**: Built-in query parameterization, table whitelisting, read-only mode
- **JSON-RPC Protocol**: Standard MCP protocol over stdio or HTTP
- **TypeScript**: Type-safe query generation and validation

### Architecture
```
┌─────────────────┐      HTTP       ┌──────────────────┐
│ Rust Backend    │ ──────────────> │  MCP Server      │
│ (LLM Client)    │                 │  (Node.js)       │
└─────────────────┘                 └──────────────────┘
                                             │
                                             │ SQL Queries
                                             ▼
                                    ┌──────────────────┐
                                    │  PostgreSQL DB   │
                                    │  (Read-Only)     │
                                    └──────────────────┘
```

### Implementation Approach
```bash
# 1. Install MCP server
npm install -g @modelcontextprotocol/server-postgres

# 2. Configuration (JSON)
{
  "mcpServers": {
    "postgres": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres", "postgresql://readonly_user:password@localhost:5432/gyz"],
      "env": {
        "ALLOWED_TABLES": "uploads,results",
        "MAX_ROWS": "1000"
      }
    }
  }
}

# 3. Run as standalone service
# Docker container with health checks
```

### MCP Tools Provided
The MCP server exposes these tools to the LLM:
1. **`query`**: Execute read-only SQL queries
2. **`list_tables`**: Get available tables (filtered by whitelist)
3. **`describe_table`**: Get schema for a specific table
4. **`list_columns`**: Get column details with types

### Security Measures
- **Read-Only Credentials**: MCP connects with PostgreSQL role having SELECT-only permissions
- **Table Whitelist**: Only `uploads` and `results` tables accessible
- **Query Validation**: Block `DELETE`, `UPDATE`, `INSERT`, `DROP`, `ALTER` statements
- **Row Limit**: Maximum 1000 rows per query to prevent resource exhaustion
- **User Context Injection**: Backend adds `WHERE user_id = $current_user` to all queries automatically

### Integration Flow
```
1. User asks: "上周有多少个A级水果?"
2. Backend sends to LLM with system prompt including MCP tools
3. LLM generates tool call: query("SELECT COUNT(*) FROM results WHERE grade_a > 0 AND created_at > NOW() - INTERVAL '7 days' AND user_id = $1", [user_id])
4. Backend forwards query to MCP server
5. MCP server executes with read-only credentials
6. Results returned: {"count": 42}
7. LLM synthesizes: "上周您识别了42个A级水果"
8. Backend caches response and returns to frontend
```

### Alternatives Considered
1. **Custom Docker MCP** (@data_wise/database-mcp): Less mature, uncertain PostgreSQL support
2. **Direct SQL Generation in Backend**: Security risk (SQL injection), no LLM-guided query construction
3. **GraphQL Layer**: Overkill for read-only queries, adds complexity

---

## Research Topic 3: Conversation Context Management

### Decision
Use **sliding window with summarization** for conversation context, storing last 10 exchanges (20 messages) in full detail.

### Rationale
- **Token Budget**: Hunyuan-MT-7B has 8192 token limit; ~4000 for context, ~4000 for response
- **Conversation Quality**: 10 exchanges sufficient for most support queries
- **Cost Control**: Reduces tokens sent per request, lowering API costs
- **Performance**: Smaller context = faster LLM response times

### Context Window Strategy
```
┌────────────────────────────────────────┐
│ System Prompt (400 tokens)             │
│ - Role definition                      │
│ - MCP tools description                │
│ - Database schema summary              │
│ - Output format instructions           │
├────────────────────────────────────────┤
│ Conversation Summary (0-800 tokens)    │
│ - Generated after 10+ exchanges        │
│ - Key facts and decisions from history │
│ - User preferences noted               │
├────────────────────────────────────────┤
│ Recent Messages (2000-2800 tokens)     │
│ - Last 10 user-assistant exchanges     │
│ - Full message content preserved       │
├────────────────────────────────────────┤
│ Current User Message (200-800 tokens)  │
│ - Latest query from user               │
└────────────────────────────────────────┘
Total: ~4000 tokens for context
```

### Summarization Trigger
After 10 exchanges (20 messages):
1. Backend sends summarization request to LLM
2. Prompt: "Summarize the key points from this conversation in 100 words"
3. Store summary, discard oldest 5 exchanges
4. Next request includes: summary + last 5 exchanges + new message

### Redis Caching Strategy
```
Key: chat:session:{session_id}:messages
Type: List (LPUSH for new messages, LRANGE for retrieval)
TTL: 86400 seconds (24 hours)
Value: JSON array of Message objects

Key: chat:session:{session_id}:summary
Type: String
TTL: 86400 seconds
Value: Text summary of older conversation

Key: chat:user:{user_id}:active_session
Type: String
TTL: 1800 seconds (30 minutes)
Value: session_id UUID
```

### PostgreSQL Storage
- **Full History**: All messages stored permanently in `chat_messages` table
- **Retrieval**: Backend loads from Redis first, falls back to PostgreSQL
- **Pagination**: Load 20 messages per page when user scrolls up in UI

### Alternatives Considered
1. **Fixed Context Window (No Summarization)**: Simpler but fails after 10 exchanges
2. **Vector Database (Semantic Search)**: Overkill for chat history, adds infrastructure complexity
3. **No Context Limits**: Risks hitting token limits, unpredictable API costs

---

## Research Topic 4: Rust HTTP Client for LLM Integration

### Decision
Use **`reqwest`** crate with **async/await** and **connection pooling** for LLM API calls.

### Rationale
- **Industry Standard**: Most popular Rust HTTP client (40M+ downloads/month)
- **Async Support**: Works seamlessly with Axum's Tokio runtime
- **Connection Pooling**: Reuses TCP connections, reduces latency
- **Timeout Control**: Built-in request/response timeouts
- **TLS Support**: Native HTTPS handling

### Implementation Example
```rust
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct LLMRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct LLMResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: String,
}

pub struct LLMClient {
    client: Client,
    api_url: String,
    api_key: String,
}

impl LLMClient {
    pub fn new(api_url: String, api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, api_url, api_key }
    }

    pub async fn send_message(&self, messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error>> {
        let request = LLMRequest {
            model: "tencent/Hunyuan-MT-7B".to_string(),
            messages,
            max_tokens: 8192,
        };

        let response = self.client
            .post(&self.api_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let llm_response: LLMResponse = response.json().await?;
            Ok(llm_response.content[0].text.clone())
        } else {
            Err(format!("LLM API error: {}", response.status()).into())
        }
    }
}
```

### Error Handling Strategy
```rust
pub enum LLMError {
    NetworkError(String),
    RateLimitExceeded,
    InvalidResponse(String),
    Timeout,
    AuthenticationFailed,
}

// Retry logic with exponential backoff
async fn call_with_retry(&self, messages: Vec<Message>) -> Result<String, LLMError> {
    for attempt in 1..=3 {
        match self.send_message(messages.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) if should_retry(&e) => {
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Err(LLMError::Timeout)
}
```

### Performance Optimizations
- **Connection Pooling**: 10 idle connections per host
- **Request Timeout**: 10 seconds (LLM should respond within 5s typically)
- **Keep-Alive**: Enabled by default in reqwest
- **Compression**: Enable gzip/br encoding for request/response

### Alternatives Considered
1. **`hyper`**: Lower-level, more complex, no benefit for simple HTTP calls
2. **`ureq`**: Synchronous blocking client, incompatible with async Axum
3. **`surf`**: Less mature, smaller ecosystem

---

## Research Topic 5: Vue 3 Chat Widget Architecture

### Decision
Use **Composition API with `<script setup>`** and **Pinia** for state management.

### Rationale
- **Constitutional Compliance**: Principle 1 requires Composition API
- **Type Safety**: Better TypeScript support with `<script setup>`
- **State Management**: Pinia is Vue 3's recommended store (replaces Vuex)
- **Reactivity**: Composition API provides fine-grained reactivity control
- **Code Reuse**: Composables enable logic sharing across components

### Component Structure
```
frontend/src/
├── components/
│   ├── ChatWidget.vue           # Main floating widget container
│   ├── ChatBubble.vue           # Minimized floating button
│   ├── ChatWindow.vue           # Expanded chat interface
│   ├── ChatMessage.vue          # Individual message component
│   ├── ChatInput.vue            # Message input with send button
│   └── ChatTypingIndicator.vue  # "AI is typing..." animation
├── composables/
│   ├── useChat.ts               # Chat state and API calls
│   ├── useChatScroll.ts         # Auto-scroll to bottom logic
│   └── useChatPolling.ts        # HTTP polling for new messages
├── stores/
│   └── chatStore.ts             # Pinia store for global chat state
└── api/
    └── chat.ts                  # API client methods
```

### Example Implementation (ChatWidget.vue)
```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useChatStore } from '@/stores/chatStore'
import { useChat } from '@/composables/useChat'
import ChatBubble from './ChatBubble.vue'
import ChatWindow from './ChatWindow.vue'

const chatStore = useChatStore()
const { sendMessage, loadHistory } = useChat()

const isExpanded = ref(false)
const isMinimized = computed(() => !isExpanded.value)

const toggleChat = () => {
  isExpanded.value = !isExpanded.value
  if (isExpanded.value) {
    loadHistory()
  }
}

const handleSendMessage = async (content: string) => {
  await sendMessage(content)
}
</script>

<template>
  <div class="chat-widget-container">
    <Transition name="fade">
      <ChatWindow
        v-if="isExpanded"
        :messages="chatStore.messages"
        :loading="chatStore.isLoading"
        @send="handleSendMessage"
        @close="toggleChat"
      />
    </Transition>
    <ChatBubble
      v-if="isMinimized"
      :unread-count="chatStore.unreadCount"
      @click="toggleChat"
    />
  </div>
</template>

<style scoped>
.chat-widget-container {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 9999;
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>
```

### State Management (Pinia Store)
```typescript
// stores/chatStore.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ChatMessage, ChatSession } from '@/types/chat'

export const useChatStore = defineStore('chat', () => {
  const currentSession = ref<ChatSession | null>(null)
  const messages = ref<ChatMessage[]>([])
  const isLoading = ref(false)
  const unreadCount = ref(0)

  const addMessage = (message: ChatMessage) => {
    messages.value.push(message)
    if (message.role === 'assistant') {
      unreadCount.value++
    }
  }

  const clearMessages = () => {
    messages.value = []
  }

  const markAsRead = () => {
    unreadCount.value = 0
  }

  return {
    currentSession,
    messages,
    isLoading,
    unreadCount,
    addMessage,
    clearMessages,
    markAsRead,
  }
})
```

### Styling Approach
- **Tailwind CSS**: Use existing utility classes for layout and spacing
- **Element Plus**: Use `el-input`, `el-button`, `el-scrollbar` for consistency
- **Custom CSS**: Minimal custom styles for animations and transitions
- **Dark Mode**: Optional - use Tailwind's `dark:` variant if enabled

### Alternatives Considered
1. **Options API**: Deprecated approach, violates constitutional principles
2. **Vuex**: Being phased out in favor of Pinia
3. **WebSocket**: More complex, deferred to v0.2.0

---

## Research Topic 6: HTTP Polling vs WebSocket for Real-Time Chat

### Decision
Implement **HTTP polling (2-second interval)** for v0.1.0, with WebSocket migration planned for v0.2.0.

### Rationale
- **Simplicity**: HTTP polling requires no additional infrastructure
- **Adequate Latency**: 2-second delay acceptable for chat use case (not instant messaging)
- **Axum Compatibility**: No need for WebSocket support in current backend
- **Debugging**: Easier to trace and debug HTTP requests/responses
- **Gradual Enhancement**: Can migrate to WebSocket without breaking changes

### Polling Implementation
```typescript
// composables/useChatPolling.ts
import { ref, onMounted, onUnmounted } from 'vue'
import { useChatStore } from '@/stores/chatStore'
import { chatApi } from '@/api/chat'

export function useChatPolling(sessionId: string) {
  const chatStore = useChatStore()
  const pollingInterval = ref<number | null>(null)
  const lastMessageId = ref<string | null>(null)

  const startPolling = () => {
    pollingInterval.value = window.setInterval(async () => {
      try {
        const newMessages = await chatApi.getNewMessages(sessionId, lastMessageId.value)

        if (newMessages.length > 0) {
          newMessages.forEach(msg => chatStore.addMessage(msg))
          lastMessageId.value = newMessages[newMessages.length - 1].id
        }
      } catch (error) {
        console.error('Polling error:', error)
      }
    }, 2000) // Poll every 2 seconds
  }

  const stopPolling = () => {
    if (pollingInterval.value) {
      clearInterval(pollingInterval.value)
      pollingInterval.value = null
    }
  }

  onMounted(() => {
    startPolling()
  })

  onUnmounted(() => {
    stopPolling()
  })

  return { startPolling, stopPolling }
}
```

### Backend Support for Polling
```rust
// GET /api/v1/chat/messages/new?session_id={id}&after={message_id}
// Returns only messages created after the specified message_id
// If no new messages, returns empty array with 200 OK

pub async fn get_new_messages(
    Query(params): Query<NewMessagesQuery>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<ChatMessage>>, AppError> {
    let messages = sqlx::query_as!(
        ChatMessage,
        r#"
        SELECT id, session_id, role, content, created_at
        FROM chat_messages
        WHERE session_id = $1 AND id > $2
        ORDER BY created_at ASC
        "#,
        params.session_id,
        params.after.unwrap_or_default()
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(messages))
}
```

### Optimization: Smart Polling
- **Active Window**: Poll every 2 seconds when chat widget is open
- **Background/Minimized**: Stop polling to save bandwidth
- **Exponential Backoff**: If no new messages for 5 polls, increase interval to 5 seconds
- **Resume on User Action**: Reset to 2-second interval when user sends message

### WebSocket Migration Path (v0.2.0)
```
1. Add Axum WebSocket support:
   - tokio-tungstenite crate
   - Upgrade HTTP connection to WS

2. Frontend change:
   - Replace useChatPolling with useWebSocket composable
   - Connect on widget open, disconnect on close

3. Message format: JSON over WebSocket
   { "type": "message", "data": { ... } }

4. Backward compatibility:
   - Keep HTTP endpoints for clients without WS support
   - Feature detection in frontend
```

### Alternatives Considered
1. **WebSocket from Start**: More complex, unnecessary for MVP
2. **Long Polling**: No benefit over standard polling for 2-second intervals
3. **Server-Sent Events (SSE)**: One-way only, need separate channel for user messages

---

## Implementation Recommendations

### Priority Order
1. **Highest**: LLM client integration (Topic 1, 4) - Core functionality
2. **High**: MCP server setup (Topic 2) - Enables database queries
3. **Medium**: Vue chat widget (Topic 5) - User interface
4. **Medium**: Context management (Topic 3) - Conversation quality
5. **Low**: HTTP polling (Topic 6) - Can use simple reload initially

### Configuration Management
Store all settings in `.env`:
```bash
# LLM Configuration
LLM_API_URL=https://api.siliconflow.cn/v1/messages
LLM_API_KEY=sk-qqkhvtehvscrwenhzixflzzmmdgzmzaufwurejxgnfbrxcjo
LLM_MODEL=tencent/Hunyuan-MT-7B
LLM_MAX_TOKENS=8192
LLM_TIMEOUT_SECONDS=10

# MCP Configuration
MCP_SERVER_URL=http://localhost:3000
MCP_ALLOWED_TABLES=uploads,results
MCP_MAX_ROWS=1000

# Chat Configuration
CHAT_CONTEXT_WINDOW=10
CHAT_SUMMARIZE_AFTER=10
CHAT_POLLING_INTERVAL_MS=2000
```

### Testing Strategy
- **Unit Tests**: Mock LLM responses, test context window logic
- **Integration Tests**: Use test LLM API key, verify MCP queries execute
- **E2E Tests**: Cypress test for complete conversation flow
- **Load Tests**: 100 concurrent chat sessions, measure response times

---

## Open Questions Resolved

All research topics have been completed. No blocking questions remain for implementation.

**Next Step**: Proceed to Phase 1 (Design & Contracts) to create data-model.md and API contracts.
