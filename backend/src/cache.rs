use redis::{aio::ConnectionManager, AsyncCommands, Client};

pub fn init_client(redis_url: &str) -> anyhow::Result<Client> {
    let client = Client::open(redis_url)?;
    Ok(client)
}

pub async fn get_connection(client: &Client) -> anyhow::Result<ConnectionManager> {
    let connection = client.get_connection_manager().await?;
    Ok(connection)
}

pub async fn cache_result(
    client: &Client,
    key: &str,
    value: &str,
    ttl_seconds: u64,
) -> anyhow::Result<()> {
    let mut conn = get_connection(client).await?;
    conn.set_ex::<_, _, ()>(key, value, ttl_seconds).await?;
    Ok(())
}

pub async fn get_cached_result(client: &Client, key: &str) -> anyhow::Result<Option<String>> {
    let mut conn = get_connection(client).await?;
    let cached: Option<String> = conn.get(key).await?;
    Ok(cached)
}

// ============================================================================
// Chat Conversation Caching (T013)
// ============================================================================

use crate::models::ChatMessage;
use uuid::Uuid;

/// Cache conversation messages (last N messages)
/// Key pattern: chat:session:{session_id}:messages
pub async fn cache_conversation_messages(
    client: &Client,
    session_id: Uuid,
    messages: &[ChatMessage],
) -> anyhow::Result<()> {
    let mut conn = get_connection(client).await?;
    let key = format!("chat:session:{}:messages", session_id);

    // Serialize messages to JSON
    let json_value = serde_json::to_string(messages)?;

    // Store with 24-hour TTL
    conn.set_ex::<_, _, ()>(&key, json_value, 86400).await?;
    Ok(())
}

/// Get cached conversation messages
pub async fn get_cached_conversation_messages(
    client: &Client,
    session_id: Uuid,
) -> anyhow::Result<Option<Vec<ChatMessage>>> {
    let mut conn = get_connection(client).await?;
    let key = format!("chat:session:{}:messages", session_id);

    let cached: Option<String> = conn.get(&key).await?;

    match cached {
        Some(json) => {
            let messages: Vec<ChatMessage> = serde_json::from_str(&json)?;
            Ok(Some(messages))
        }
        None => Ok(None),
    }
}

/// Append a new message to cached conversation
/// Maintains only the last 20 messages
pub async fn append_to_conversation_cache(
    client: &Client,
    session_id: Uuid,
    message: &ChatMessage,
) -> anyhow::Result<()> {
    let mut messages = get_cached_conversation_messages(client, session_id)
        .await?
        .unwrap_or_default();

    // Add new message
    messages.push(message.clone());

    // Keep only last 20 messages
    if messages.len() > 20 {
        let skip_count = messages.len() - 20;
        messages = messages.into_iter().skip(skip_count).collect();
    }

    // Update cache
    cache_conversation_messages(client, session_id, &messages).await
}

/// Cache conversation summary/title
/// Key pattern: chat:session:{session_id}:summary
pub async fn cache_conversation_summary(
    client: &Client,
    session_id: Uuid,
    summary: &str,
) -> anyhow::Result<()> {
    let mut conn = get_connection(client).await?;
    let key = format!("chat:session:{}:summary", session_id);

    // Store with 24-hour TTL
    conn.set_ex::<_, _, ()>(&key, summary, 86400).await?;
    Ok(())
}

/// Get cached conversation summary
pub async fn get_cached_conversation_summary(
    client: &Client,
    session_id: Uuid,
) -> anyhow::Result<Option<String>> {
    let mut conn = get_connection(client).await?;
    let key = format!("chat:session:{}:summary", session_id);

    let cached: Option<String> = conn.get(&key).await?;
    Ok(cached)
}

/// Cache user's active session ID
/// Key pattern: chat:user:{user_id}:active_session
pub async fn cache_active_session(
    client: &Client,
    user_id: Uuid,
    session_id: Uuid,
) -> anyhow::Result<()> {
    let mut conn = get_connection(client).await?;
    let key = format!("chat:user:{}:active_session", user_id);

    // Store with 30-minute TTL (auto-expire if inactive)
    conn.set_ex::<_, _, ()>(&key, session_id.to_string(), 1800).await?;
    Ok(())
}

/// Get user's active session ID from cache
pub async fn get_active_session(
    client: &Client,
    user_id: Uuid,
) -> anyhow::Result<Option<Uuid>> {
    let mut conn = get_connection(client).await?;
    let key = format!("chat:user:{}:active_session", user_id);

    let cached: Option<String> = conn.get(&key).await?;

    match cached {
        Some(id_str) => {
            let session_id = Uuid::parse_str(&id_str)?;
            Ok(Some(session_id))
        }
        None => Ok(None),
    }
}

/// Invalidate conversation cache (called when session is deleted)
pub async fn invalidate_conversation_cache(
    client: &Client,
    session_id: Uuid,
) -> anyhow::Result<()> {
    let mut conn = get_connection(client).await?;

    let keys = vec![
        format!("chat:session:{}:messages", session_id),
        format!("chat:session:{}:summary", session_id),
    ];

    for key in keys {
        let _: () = conn.del(&key).await?;
    }

    Ok(())
}

#[cfg(test)]
mod chat_cache_tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_cache_and_retrieve_messages() {
        // This test requires a running Redis instance
        // Skip in CI if Redis is not available
        if std::env::var("REDIS_URL").is_err() {
            return;
        }

        let redis_url = std::env::var("REDIS_URL").unwrap();
        let client = init_client(&redis_url).unwrap();
        let session_id = Uuid::new_v4();

        let messages = vec![
            ChatMessage {
                id: Uuid::new_v4(),
                session_id,
                role: "user".to_string(),
                content: "Hello".to_string(),
                metadata: None,
                created_at: Utc::now(),
            },
            ChatMessage {
                id: Uuid::new_v4(),
                session_id,
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
                metadata: None,
                created_at: Utc::now(),
            },
        ];

        cache_conversation_messages(&client, session_id, &messages)
            .await
            .unwrap();

        let cached = get_cached_conversation_messages(&client, session_id)
            .await
            .unwrap();

        assert!(cached.is_some());
        let cached_messages = cached.unwrap();
        assert_eq!(cached_messages.len(), 2);
        assert_eq!(cached_messages[0].content, "Hello");
        assert_eq!(cached_messages[1].content, "Hi there!");
    }
}
