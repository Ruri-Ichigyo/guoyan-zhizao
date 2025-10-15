# Database Migrations

This directory contains SQL migration scripts for the chat module.

## Migrations

1. **001_create_chat_sessions.sql** - Creates the `chat_sessions` table with indexes and triggers
2. **002_create_chat_messages.sql** - Creates the `chat_messages` table with GIN index for JSONB
3. **003_create_mcp_readonly_user.sql** - Creates the read-only database role for MCP server

## Applying Migrations

### Manual Execution
```bash
# Apply migrations in order
psql -h localhost -U gyz -d gyz -f 001_create_chat_sessions.sql
psql -h localhost -U gyz -d gyz -f 002_create_chat_messages.sql
psql -h localhost -U gyz -d gyz -f 003_create_mcp_readonly_user.sql
```

### Verification
```sql
-- Check tables exist
\dt chat_*

-- Check indexes
\di chat_*

-- Check role
\du mcp_readonly

-- Verify permissions
\dp uploads
\dp results
```

## Rolling Back

If you need to rollback:
```bash
psql -h localhost -U gyz -d gyz -f 003_create_mcp_readonly_user_rollback.sql
psql -h localhost -U gyz -d gyz -f 002_create_chat_messages_rollback.sql
psql -h localhost -U gyz -d gyz -f 001_create_chat_sessions_rollback.sql
```

## Notes

- Migrations must be applied in order (001 → 002 → 003)
- Rollbacks must be executed in reverse order (003 → 002 → 001)
- Migration 003 sets the MCP readonly user password to `mcp_secure_pass_2025` - change this in production!
- The MCP user only has SELECT permissions on `uploads` and `results` tables
