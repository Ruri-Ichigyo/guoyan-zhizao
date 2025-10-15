#!/bin/bash

# Database Migration Application Script
# Usage: ./apply-chat-migrations.sh

set -e

# Load environment variables
if [ -f "$(dirname "$0")/../.env" ]; then
    export $(cat "$(dirname "$0")/../.env" | grep -v '^#' | xargs)
fi

# Database connection defaults
DB_HOST=${POSTGRES_HOST:-localhost}
DB_PORT=${POSTGRES_PORT:-5432}
DB_USER=${POSTGRES_USER:-gyz}
DB_NAME=${POSTGRES_DB:-gyz}
MIGRATIONS_DIR="$(dirname "$0")/migrations"

echo "🔍 Checking database connection..."
if ! psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1" > /dev/null 2>&1; then
    echo "❌ Cannot connect to database"
    echo "   Host: $DB_HOST:$DB_PORT"
    echo "   Database: $DB_NAME"
    echo "   User: $DB_USER"
    exit 1
fi

echo "✅ Database connection successful"
echo ""

echo "📋 Applying migrations..."
echo ""

# Apply Migration 001
echo "⏳ Migration 001: Creating chat_sessions table..."
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$MIGRATIONS_DIR/001_create_chat_sessions.sql" > /dev/null 2>&1; then
    echo "✅ Migration 001 applied successfully"
else
    echo "❌ Migration 001 failed"
    exit 1
fi

# Apply Migration 002
echo "⏳ Migration 002: Creating chat_messages table..."
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$MIGRATIONS_DIR/002_create_chat_messages.sql" > /dev/null 2>&1; then
    echo "✅ Migration 002 applied successfully"
else
    echo "❌ Migration 002 failed"
    exit 1
fi

# Apply Migration 003
echo "⏳ Migration 003: Creating MCP readonly user..."
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$MIGRATIONS_DIR/003_create_mcp_readonly_user.sql" > /dev/null 2>&1; then
    echo "✅ Migration 003 applied successfully"
else
    echo "❌ Migration 003 failed"
    exit 1
fi

echo ""
echo "🎉 All migrations applied successfully!"
echo ""

# Verification
echo "🔍 Verifying tables..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "\\dt chat_*"

echo ""
echo "🔍 Verifying MCP readonly user..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "\\du mcp_readonly"

echo ""
echo "✨ Database setup complete!"
echo ""
echo "📝 Next steps:"
echo "   1. Update mcp-config.json with the MCP readonly credentials"
echo "   2. Start the MCP server: npx @modelcontextprotocol/server-postgres"
echo "   3. Test MCP connection: curl http://localhost:3000/health"
