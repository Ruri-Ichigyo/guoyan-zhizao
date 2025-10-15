-- Rollback Migration 002: Drop chat_messages table
-- Created: 2025-10-14

-- Drop indexes
DROP INDEX IF EXISTS idx_chat_messages_metadata;
DROP INDEX IF EXISTS idx_chat_messages_created_at;
DROP INDEX IF EXISTS idx_chat_messages_session_id;

-- Drop table
DROP TABLE IF EXISTS chat_messages CASCADE;
