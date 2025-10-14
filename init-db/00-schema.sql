-- =============================================================================
-- 果蔬智造数据库Schema
-- 创建时间: 2025-10-14
-- 说明: 统一的数据库表结构定义
-- =============================================================================

-- =============================================================================
-- 1. 用户与权限模块
-- =============================================================================

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name VARCHAR(100),
    role VARCHAR(20) NOT NULL DEFAULT 'user' CHECK (role IN ('admin', 'user', 'operator')),
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'suspended')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);

-- 注释
COMMENT ON TABLE users IS '系统用户表';
COMMENT ON COLUMN users.id IS '用户唯一标识';
COMMENT ON COLUMN users.username IS '用户名（唯一）';
COMMENT ON COLUMN users.email IS '邮箱（唯一）';
COMMENT ON COLUMN users.role IS '用户角色: admin(管理员), user(普通用户), operator(操作员)';
COMMENT ON COLUMN users.status IS '用户状态: active(活跃), inactive(未激活), suspended(已停用)';

-- =============================================================================
-- 2. 图片上传与检测模块
-- =============================================================================

-- 上传表
CREATE TABLE IF NOT EXISTS uploads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    filename TEXT NOT NULL,
    content_type TEXT,
    data BYTEA NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_uploads_user_id ON uploads(user_id);
CREATE INDEX IF NOT EXISTS idx_uploads_status ON uploads(status);
CREATE INDEX IF NOT EXISTS idx_uploads_created_at ON uploads(created_at DESC);

-- 注释
COMMENT ON TABLE uploads IS '用户上传的图片记录';
COMMENT ON COLUMN uploads.id IS '上传记录唯一标识';
COMMENT ON COLUMN uploads.user_id IS '上传用户ID';
COMMENT ON COLUMN uploads.filename IS '文件名';
COMMENT ON COLUMN uploads.status IS '处理状态';

-- 检测结果表
CREATE TABLE IF NOT EXISTS results (
    upload_id UUID PRIMARY KEY REFERENCES uploads(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    detections JSONB NOT NULL DEFAULT '[]'::JSONB,
    total_fruits INTEGER NOT NULL DEFAULT 0,
    grade_a INTEGER NOT NULL DEFAULT 0,
    grade_b INTEGER NOT NULL DEFAULT 0,
    grade_c INTEGER NOT NULL DEFAULT 0,
    avg_confidence REAL NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_results_status ON results(status);
CREATE INDEX IF NOT EXISTS idx_results_created_at ON results(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_results_detections ON results USING GIN(detections);

-- 注释
COMMENT ON TABLE results IS 'AI检测结果表';
COMMENT ON COLUMN results.upload_id IS '关联的上传记录ID';
COMMENT ON COLUMN results.detections IS '检测结果JSON数组';
COMMENT ON COLUMN results.total_fruits IS '检测到的水果总数';

-- =============================================================================
-- 3. 实时检测模块
-- =============================================================================

-- 实时检测表
CREATE TABLE IF NOT EXISTS detections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    snapshot BYTEA NOT NULL,
    labels TEXT[] NOT NULL,
    confidence_scores REAL[] NOT NULL,
    detected_classes TEXT[] NOT NULL,
    source TEXT DEFAULT 'live_camera',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_detections_user_id ON detections(user_id);
CREATE INDEX IF NOT EXISTS idx_detections_source ON detections(source);
CREATE INDEX IF NOT EXISTS idx_detections_created_at ON detections(created_at DESC);

-- 注释
COMMENT ON TABLE detections IS '实时摄像头检测记录';
COMMENT ON COLUMN detections.id IS '检测记录唯一标识';
COMMENT ON COLUMN detections.user_id IS '检测用户ID';
COMMENT ON COLUMN detections.source IS '检测来源（摄像头标识）';

-- =============================================================================
-- 4. AI聊天模块
-- =============================================================================

-- 聊天会话表
CREATE TABLE IF NOT EXISTS chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'archived')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_chat_sessions_user_id ON chat_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_chat_sessions_status ON chat_sessions(status);
CREATE INDEX IF NOT EXISTS idx_chat_sessions_updated_at ON chat_sessions(updated_at DESC);

-- 注释
COMMENT ON TABLE chat_sessions IS 'AI聊天会话表';
COMMENT ON COLUMN chat_sessions.id IS '会话唯一标识';
COMMENT ON COLUMN chat_sessions.user_id IS '用户ID';
COMMENT ON COLUMN chat_sessions.title IS '会话标题';
COMMENT ON COLUMN chat_sessions.status IS '会话状态: active(活跃), inactive(闲置), archived(已归档)';

-- 聊天消息表
CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL CHECK (char_length(content) <= 10000),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_chat_messages_session_id ON chat_messages(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_chat_messages_created_at ON chat_messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_messages_metadata ON chat_messages USING GIN(metadata);

-- 注释
COMMENT ON TABLE chat_messages IS 'AI聊天消息表';
COMMENT ON COLUMN chat_messages.id IS '消息唯一标识';
COMMENT ON COLUMN chat_messages.session_id IS '所属会话ID';
COMMENT ON COLUMN chat_messages.role IS '消息角色: user(用户), assistant(AI助手), system(系统)';
COMMENT ON COLUMN chat_messages.metadata IS '元数据（MCP查询信息、token使用等）';

-- =============================================================================
-- 5. 触发器
-- =============================================================================

-- 用户表更新时间触发器
CREATE OR REPLACE FUNCTION update_users_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_users_timestamp
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_users_timestamp();

-- 上传表更新时间触发器
CREATE OR REPLACE FUNCTION update_uploads_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_uploads_timestamp
BEFORE UPDATE ON uploads
FOR EACH ROW
EXECUTE FUNCTION update_uploads_timestamp();

-- 聊天会话更新时间触发器
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

-- =============================================================================
-- 6. MCP只读用户
-- =============================================================================

-- 创建只读角色
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'mcp_readonly') THEN
        CREATE ROLE mcp_readonly WITH LOGIN PASSWORD 'mcp_secure_pass_2025';
    END IF;
END
$$;

-- 授予基础权限
GRANT CONNECT ON DATABASE gyz TO mcp_readonly;
GRANT USAGE ON SCHEMA public TO mcp_readonly;

-- 授予所有相关表的SELECT权限
GRANT SELECT ON users TO mcp_readonly;
GRANT SELECT ON uploads TO mcp_readonly;
GRANT SELECT ON results TO mcp_readonly;
GRANT SELECT ON detections TO mcp_readonly;
GRANT SELECT ON chat_sessions TO mcp_readonly;
GRANT SELECT ON chat_messages TO mcp_readonly;

-- 确保没有写权限
REVOKE INSERT, UPDATE, DELETE, TRUNCATE ON ALL TABLES IN SCHEMA public FROM mcp_readonly;
REVOKE CREATE ON SCHEMA public FROM mcp_readonly;
REVOKE ALL ON ALL SEQUENCES IN SCHEMA public FROM mcp_readonly;
REVOKE ALL ON ALL FUNCTIONS IN SCHEMA public FROM mcp_readonly;

-- 注释
COMMENT ON ROLE mcp_readonly IS 'MCP服务器只读角色，用于AI助手查询数据库';
