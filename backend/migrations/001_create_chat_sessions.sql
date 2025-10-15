-- Migration 001: Create chat_sessions table
-- Created: 2025-10-14
-- Description: Stores chat conversation sessions for AI assistant feature

CREATE TABLE IF NOT EXISTS chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    title TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'archived')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_chat_sessions_user_id ON chat_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_chat_sessions_status ON chat_sessions(status);
CREATE INDEX IF NOT EXISTS idx_chat_sessions_updated_at ON chat_sessions(updated_at DESC);

-- Trigger to auto-update updated_at timestamp
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

-- Comments for documentation
COMMENT ON TABLE chat_sessions IS 'Chat conversation sessions between users and AI assistant';
COMMENT ON COLUMN chat_sessions.id IS 'Unique session identifier';
COMMENT ON COLUMN chat_sessions.user_id IS 'ID of the user who owns this session';
COMMENT ON COLUMN chat_sessions.title IS 'Session title (auto-generated from first message or user-provided)';
COMMENT ON COLUMN chat_sessions.status IS 'Session state: active (in use), inactive (idle >30min), archived (user deleted)';
COMMENT ON COLUMN chat_sessions.created_at IS 'When the session was created';
COMMENT ON COLUMN chat_sessions.updated_at IS 'Last message timestamp';
