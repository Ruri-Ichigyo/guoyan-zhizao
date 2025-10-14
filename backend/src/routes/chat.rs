use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    cache::{
        append_to_conversation_cache, cache_active_session, cache_conversation_messages,
        get_cached_conversation_messages, invalidate_conversation_cache,
    },
    errors::AppError,
    models::{
        ChatHistoryResponse, ChatMessage, ChatSession, CreateSessionRequest,
        CreateSessionResponse, LLMMessage, ListSessionsResponse, SendMessageRequest,
        SendMessageResponse,
    },
    services::LLMClient,
    AppState,
};

// ============================================================================
// Session Management (T014-T016)
// ============================================================================

/// Create a new chat session
/// POST /api/v1/chat/sessions
pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, AppError> {
    // TODO: Extract user_id from JWT token (for now using a mock UUID)
    let user_id = Uuid::new_v4(); // This should come from authentication middleware

    let session_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Insert session into database
    sqlx::query(
        r#"
        INSERT INTO chat_sessions (id, user_id, title, status, created_at, updated_at)
        VALUES ($1, $2, $3, 'active', $4, $4)
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .bind(req.title)
    .bind(now)
    .execute(&state.pool)
    .await?;

    // Cache as active session
    cache_active_session(&state.redis, user_id, session_id).await?;

    Ok(Json(CreateSessionResponse {
        session_id,
        created_at: now,
    }))
}

/// List user's chat sessions
/// GET /api/v1/chat/sessions?status=active&limit=10&offset=0
#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    #[serde(default)]
    status: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    10
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<Json<ListSessionsResponse>, AppError> {
    // TODO: Extract user_id from JWT token
    let user_id = Uuid::new_v4();

    let mut sql = "SELECT * FROM chat_sessions WHERE user_id = $1".to_string();

    if let Some(status) = &query.status {
        sql.push_str(&format!(" AND status = '{}'", status));
    }

    sql.push_str(" ORDER BY updated_at DESC LIMIT $2 OFFSET $3");

    let sessions = sqlx::query_as::<_, ChatSession>(&sql)
        .bind(user_id)
        .bind(query.limit)
        .bind(query.offset)
        .fetch_all(&state.pool)
        .await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_sessions WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(ListSessionsResponse { sessions, total }))
}

/// Delete a chat session
/// DELETE /api/v1/chat/sessions/{session_id}
pub async fn delete_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // TODO: Verify user owns this session

    // Delete from database (CASCADE will delete messages)
    sqlx::query("DELETE FROM chat_sessions WHERE id = $1")
        .bind(session_id)
        .execute(&state.pool)
        .await?;

    // Invalidate cache
    invalidate_conversation_cache(&state.redis, session_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Message Handling with LLM Integration (T017-T018)
// ============================================================================

/// Send a message and get AI response
/// POST /api/v1/chat/message
pub async fn send_message(
    State(state): State<AppState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, AppError> {
    // TODO: Extract user_id from JWT token
    // For now, use a fixed demo user ID
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("Valid UUID");

    // Verify session exists, or create if it doesn't
    let session: ChatSession = match sqlx::query_as("SELECT * FROM chat_sessions WHERE id = $1")
        .bind(req.session_id)
        .fetch_one(&state.pool)
        .await
    {
        Ok(session) => session,
        Err(sqlx::Error::RowNotFound) => {
            // Session doesn't exist, create it
            let now = chrono::Utc::now();
            sqlx::query(
                r#"
                INSERT INTO chat_sessions (id, user_id, title, status, created_at, updated_at)
                VALUES ($1, $2, '新对话', 'active', $3, $3)
                "#,
            )
            .bind(req.session_id)
            .bind(user_id)
            .bind(now)
            .execute(&state.pool)
            .await?;

            // Return the newly created session
            ChatSession {
                id: req.session_id,
                user_id,
                title: Some("新对话".to_string()),
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            }
        }
        Err(e) => return Err(e.into()),
    };

    // Validate message content length
    if req.content.len() > 10000 {
        return Err(AppError::Validation(
            "Message content exceeds 10000 characters".to_string(),
        ));
    }

    // Save user message
    let user_message_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO chat_messages (id, session_id, role, content, created_at)
        VALUES ($1, $2, 'user', $3, $4)
        "#,
    )
    .bind(user_message_id)
    .bind(req.session_id)
    .bind(&req.content)
    .bind(now)
    .execute(&state.pool)
    .await?;

    let user_message = ChatMessage {
        id: user_message_id,
        session_id: req.session_id,
        role: "user".to_string(),
        content: req.content.clone(),
        metadata: None,
        created_at: now,
    };

    // Update cache
    append_to_conversation_cache(&state.redis, req.session_id, &user_message).await?;

    // Get conversation history from cache (last 20 messages)
    let history = get_cached_conversation_messages(&state.redis, req.session_id)
        .await?
        .unwrap_or_default();

    // Build LLM request with conversation context
    let llm_messages: Vec<LLMMessage> = history
        .iter()
        .map(|msg| LLMMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        })
        .collect();

    // Add database query tool for intelligent data access
    let tools = vec![LLMClient::build_database_query_tool()];

    // Send to LLM with database query capability
    let llm_response = state
        .llm_client
        .send_message(llm_messages.clone(), Some(tools))
        .await
        .map_err(|e| AppError::Unexpected(format!("LLM error: {}", e)))?;

    // Check if LLM requested database query
    let tool_calls = state.llm_client.extract_tool_calls(&llm_response);
    let mut assistant_content = String::new();
    let tool_call_count = tool_calls.len();

    if !tool_calls.is_empty() {
        // Process database query tool calls
        for (tool_name, input) in tool_calls {
            if tool_name == "query_database" {
                let query = input["query"]
                    .as_str()
                    .ok_or_else(|| AppError::Validation("Invalid query in tool call".to_string()))?;

                // Execute query through direct database access
                let mcp_result = state
                    .mcp_client
                    .query(query, vec![], user_id)
                    .await
                    .map_err(|e| AppError::Unexpected(format!("Database query error: {}", e)))?;

                // Format results for LLM
                let results_json = serde_json::to_string_pretty(&mcp_result.rows)
                    .map_err(|e| AppError::Unexpected(format!("JSON serialization error: {}", e)))?;

                // Send results back to LLM for synthesis
                let mut follow_up_messages = llm_messages.clone();
                follow_up_messages.push(LLMMessage {
                    role: "assistant".to_string(),
                    content: format!("我将查询数据库: {}", query),
                });
                follow_up_messages.push(LLMMessage {
                    role: "user".to_string(),
                    content: format!("查询结果:\n{}", results_json),
                });

                let synthesis_response = state
                    .llm_client
                    .send_message(follow_up_messages, None)
                    .await
                    .map_err(|e| AppError::Unexpected(format!("LLM synthesis error: {}", e)))?;

                assistant_content = state
                    .llm_client
                    .extract_text_from_response(&synthesis_response)
                    .map_err(|e| AppError::Unexpected(format!("LLM parse error: {}", e)))?;
            }
        }
    } else {
        // Simple text response without database query
        assistant_content = state
            .llm_client
            .extract_text_from_response(&llm_response)
            .map_err(|e| AppError::Unexpected(format!("LLM parse error: {}", e)))?;
    }

    // Save assistant response
    let assistant_message_id = Uuid::new_v4();
    let response_time = chrono::Utc::now();

    let metadata = serde_json::json!({
        "model": llm_response.model,
        "usage": llm_response.usage,
        "tool_calls": tool_call_count,
    });

    sqlx::query(
        r#"
        INSERT INTO chat_messages (id, session_id, role, content, metadata, created_at)
        VALUES ($1, $2, 'assistant', $3, $4, $5)
        "#,
    )
    .bind(assistant_message_id)
    .bind(req.session_id)
    .bind(&assistant_content)
    .bind(&metadata)
    .bind(response_time)
    .execute(&state.pool)
    .await?;

    let assistant_message = ChatMessage {
        id: assistant_message_id,
        session_id: req.session_id,
        role: "assistant".to_string(),
        content: assistant_content.clone(),
        metadata: Some(metadata.clone()),
        created_at: response_time,
    };

    // Update cache
    append_to_conversation_cache(&state.redis, req.session_id, &assistant_message).await?;

    // Update session updated_at
    sqlx::query("UPDATE chat_sessions SET updated_at = $1 WHERE id = $2")
        .bind(response_time)
        .bind(req.session_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(SendMessageResponse {
        message_id: assistant_message_id,
        session_id: req.session_id,
        role: "assistant".to_string(),
        content: assistant_content,
        metadata: Some(metadata),
        created_at: response_time,
    }))
}

// ============================================================================
// History and Polling (T021-T022)
// ============================================================================

/// Get conversation history
/// GET /api/v1/chat/history/{session_id}?limit=20&offset=0
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_history_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_history_limit() -> i64 {
    20
}

pub async fn get_history(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<ChatHistoryResponse>, AppError> {
    // TODO: Verify user owns this session

    // Get session
    let session: ChatSession = sqlx::query_as("SELECT * FROM chat_sessions WHERE id = $1")
        .bind(session_id)
        .fetch_one(&state.pool)
        .await?;

    // Try cache first
    let cached_messages = get_cached_conversation_messages(&state.redis, session_id).await?;

    let messages = if let Some(cached) = cached_messages {
        cached
    } else {
        // Fallback to database
        let msgs: Vec<ChatMessage> = sqlx::query_as(
            r#"
            SELECT * FROM chat_messages
            WHERE session_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(session_id)
        .bind(query.limit)
        .bind(query.offset)
        .fetch_all(&state.pool)
        .await?;

        // Update cache
        if query.offset == 0 {
            cache_conversation_messages(&state.redis, session_id, &msgs).await?;
        }

        msgs
    };

    Ok(Json(ChatHistoryResponse { messages, session }))
}

/// Poll for new messages (HTTP polling implementation)
/// GET /api/v1/chat/messages/new?session_id={id}&after={msg_id}
#[derive(Debug, Deserialize)]
pub struct PollQuery {
    session_id: Uuid,
    #[serde(default)]
    after: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct PollResponse {
    messages: Vec<ChatMessage>,
    has_more: bool,
}

pub async fn poll_new_messages(
    State(state): State<AppState>,
    Query(query): Query<PollQuery>,
) -> Result<Json<PollResponse>, AppError> {
    // TODO: Verify user owns this session

    let messages = if let Some(after_id) = query.after {
        // Get messages after specific ID
        sqlx::query_as::<_, ChatMessage>(
            r#"
            SELECT * FROM chat_messages
            WHERE session_id = $1 AND created_at > (
                SELECT created_at FROM chat_messages WHERE id = $2
            )
            ORDER BY created_at ASC
            "#,
        )
        .bind(query.session_id)
        .bind(after_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        // Get latest messages
        sqlx::query_as::<_, ChatMessage>(
            r#"
            SELECT * FROM chat_messages
            WHERE session_id = $1
            ORDER BY created_at DESC
            LIMIT 20
            "#,
        )
        .bind(query.session_id)
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(PollResponse {
        has_more: messages.len() >= 20,
        messages,
    }))
}
