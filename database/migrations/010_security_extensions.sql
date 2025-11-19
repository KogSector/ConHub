-- Security Extensions Migration
-- Adds dedicated tables for enhanced security features
-- Run this migration when ready to enable encrypted secrets and advanced security events

-- Dedicated security_events table (separate from audit log for different use cases)
CREATE TABLE IF NOT EXISTS security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    ip_address INET,
    user_agent TEXT,
    details JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Encrypted secrets for storing sensitive data (API tokens, OAuth refresh tokens, etc.)
CREATE TABLE IF NOT EXISTS encrypted_secrets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_name TEXT NOT NULL,
    encrypted_value BYTEA NOT NULL,
    encryption_version TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, key_name)
);

-- Optional: Dedicated api_tokens table (alternative to api_keys)
-- Uncomment if you want to migrate from api_keys to api_tokens
/*
CREATE TABLE IF NOT EXISTS api_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMP WITH TIME ZONE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON api_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_api_tokens_token_hash ON api_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_api_tokens_is_active ON api_tokens(is_active);
*/

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_security_events_user_id ON security_events(user_id);
CREATE INDEX IF NOT EXISTS idx_security_events_event_type ON security_events(event_type);
CREATE INDEX IF NOT EXISTS idx_security_events_severity ON security_events(severity);
CREATE INDEX IF NOT EXISTS idx_security_events_created_at ON security_events(created_at);

CREATE INDEX IF NOT EXISTS idx_encrypted_secrets_user_id ON encrypted_secrets(user_id);
CREATE INDEX IF NOT EXISTS idx_encrypted_secrets_key_name ON encrypted_secrets(key_name);

-- Trigger for encrypted_secrets updated_at
CREATE TRIGGER update_encrypted_secrets_updated_at
    BEFORE UPDATE ON encrypted_secrets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE security_events IS 'Dedicated security events table for application-level security monitoring';
COMMENT ON TABLE encrypted_secrets IS 'Encrypted storage for sensitive user data like API tokens and OAuth credentials';
COMMENT ON COLUMN encrypted_secrets.encryption_version IS 'Version identifier for encryption algorithm used';
COMMENT ON COLUMN encrypted_secrets.metadata IS 'Optional metadata about the secret (non-sensitive)';
