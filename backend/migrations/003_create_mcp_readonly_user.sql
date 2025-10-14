-- Migration 003: Create MCP read-only user
-- Created: 2025-10-14
-- Description: Creates a restricted database role for MCP server with SELECT-only permissions

-- Create read-only role for MCP server
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'mcp_readonly') THEN
        CREATE ROLE mcp_readonly WITH LOGIN PASSWORD 'mcp_secure_pass_2025';
    END IF;
END
$$;

-- Grant CONNECT privilege
GRANT CONNECT ON DATABASE gyz TO mcp_readonly;

-- Grant USAGE on schema
GRANT USAGE ON SCHEMA public TO mcp_readonly;

-- Grant SELECT on specific tables only (whitelist approach)
GRANT SELECT ON uploads TO mcp_readonly;
GRANT SELECT ON results TO mcp_readonly;

-- Ensure no write permissions
REVOKE INSERT, UPDATE, DELETE, TRUNCATE ON ALL TABLES IN SCHEMA public FROM mcp_readonly;

-- Revoke dangerous permissions
REVOKE CREATE ON SCHEMA public FROM mcp_readonly;
REVOKE ALL ON ALL SEQUENCES IN SCHEMA public FROM mcp_readonly;
REVOKE ALL ON ALL FUNCTIONS IN SCHEMA public FROM mcp_readonly;

-- Comments for documentation
COMMENT ON ROLE mcp_readonly IS 'Read-only role for MCP server to query uploads and results tables';
