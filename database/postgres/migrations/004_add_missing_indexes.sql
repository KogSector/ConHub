-- Migration 004: Add Missing Database Indexes
-- Purpose: Add indexes to improve query performance on frequently accessed columns

-- Index on users.last_login_at for tracking and filtering by last login time
CREATE INDEX IF NOT EXISTS idx_users_last_login_at ON users(last_login_at);

-- Index on users.is_active for filtering active/inactive users
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active) WHERE is_active = TRUE;

-- Index on api_tokens.user_id for foreign key lookups
CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON api_tokens(user_id);

-- Index on webhooks.user_id for foreign key lookups
CREATE INDEX IF NOT EXISTS idx_webhooks_user_id ON webhooks(user_id);

-- Index on social_connections.token_expires_at for finding expiring/expired tokens
CREATE INDEX IF NOT EXISTS idx_social_connections_token_expires_at ON social_connections(token_expires_at);
