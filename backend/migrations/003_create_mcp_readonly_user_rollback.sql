-- Rollback Migration 003: Drop MCP read-only user
-- Created: 2025-10-14

-- Revoke all privileges
REVOKE ALL ON ALL TABLES IN SCHEMA public FROM mcp_readonly;
REVOKE USAGE ON SCHEMA public FROM mcp_readonly;
REVOKE CONNECT ON DATABASE gyz FROM mcp_readonly;

-- Drop role
DROP ROLE IF EXISTS mcp_readonly;
