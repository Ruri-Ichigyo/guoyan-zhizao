-- Migration 002: Create chat_messages table
-- Created: 2025-10-14
-- Description: Stores individual messages within chat sessions

CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL CHECK (char_length(content) <= 10000),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_chat_messages_session_id ON chat_messages(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_chat_messages_created_at ON chat_messages(created_at DESC);

-- Comments for documentation
COMMENT ON TABLE chat_messages IS 'Individual messages within chat conversations';
COMMENT ON COLUMN chat_messages.id IS 'Unique message identifier';
COMMENT ON COLUMN chat_messages.session_id IS 'Foreign key to chat_sessions table';
COMMENT ON COLUMN chat_messages.role IS 'Message sender: user (human), assistant (AI), system (automated)';
COMMENT ON COLUMN chat_messages.content IS 'Message text content (max 10000 characters)';
COMMENT ON COLUMN chat_messages.metadata IS 'Additional data: MCP query info, LLM model, tokens, timing';
COMMENT ON COLUMN chat_messages.created_at IS 'When the message was created';
