-- Rollback Migration 001: Drop chat_sessions table
-- Created: 2025-10-14

-- Drop trigger first
DROP TRIGGER IF EXISTS trigger_update_chat_session_timestamp ON chat_sessions;
DROP FUNCTION IF EXISTS update_chat_session_timestamp();

-- Drop indexes
DROP INDEX IF EXISTS idx_chat_sessions_updated_at;
DROP INDEX IF EXISTS idx_chat_sessions_status;
DROP INDEX IF EXISTS idx_chat_sessions_user_id;

-- Drop table
DROP TABLE IF EXISTS chat_sessions CASCADE;
