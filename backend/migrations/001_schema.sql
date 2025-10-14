



CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS pg_trgm;


DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'userrole') THEN
        CREATE TYPE userrole AS ENUM ('admin', 'user', 'moderator');
    END IF;
END$$;


DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'subscription_tier') THEN
        CREATE TYPE subscription_tier AS ENUM ('free', 'personal', 'team', 'enterprise');
    END IF;
END$$;


DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'social_platform') THEN
        CREATE TYPE social_platform AS ENUM ('slack', 'notion', 'google_drive', 'gmail', 'dropbox', 'linkedin');
    END IF;
END$$;


CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    avatar_url VARCHAR(512),
    organization VARCHAR(255),
    role userrole NOT NULL DEFAULT 'user',
    subscription_tier subscription_tier NOT NULL DEFAULT 'free',
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE
);


CREATE TABLE social_connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,
    platform_user_id VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_expires_at TIMESTAMP WITH TIME ZONE,
    scope TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_sync_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, platform, platform_user_id)
);


CREATE TABLE social_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    connection_id UUID NOT NULL REFERENCES social_connections(id) ON DELETE CASCADE,
    token_type VARCHAR(50) NOT NULL, 
    token_value TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    scope TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);


CREATE TABLE social_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    connection_id UUID NOT NULL REFERENCES social_connections(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,
    data_type VARCHAR(100) NOT NULL, 
    external_id VARCHAR(255) NOT NULL,
    title TEXT,
    content TEXT,
    url VARCHAR(512),
    metadata JSONB NOT NULL DEFAULT '{}',
    synced_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(connection_id, external_id)
);


CREATE TABLE api_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMP WITH TIME ZONE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);


CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    url VARCHAR(512) NOT NULL,
    events JSONB NOT NULL DEFAULT '[]',
    secret_hash VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_triggered_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);


CREATE TABLE ai_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    content TEXT NOT NULL,
    rule_type JSONB NOT NULL DEFAULT '{"type": "Custom", "Custom": "general"}',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    version INTEGER DEFAULT 1,
    priority INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    tags TEXT[] DEFAULT '{}',
    
    
    CONSTRAINT ai_rules_priority_check CHECK (priority >= -100 AND priority <= 100)
);


CREATE TABLE ai_agent_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    agent_type VARCHAR(100) NOT NULL UNIQUE,
    rule_ids UUID[] DEFAULT '{}',
    capabilities TEXT[] DEFAULT '{}',
    default_settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true
);


CREATE TABLE ai_memory_bank (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    memory_type JSONB NOT NULL DEFAULT '{"type": "Context"}',
    tags TEXT[] DEFAULT '{}',
    context VARCHAR(255) NOT NULL,
    access_count BIGINT DEFAULT 0,
    last_accessed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true
);


CREATE TABLE ai_rule_applications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rule_id UUID NOT NULL REFERENCES ai_rules(id) ON DELETE CASCADE,
    agent_type VARCHAR(100) NOT NULL,
    context_hash VARCHAR(64), 
    applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    success BOOLEAN,
    feedback_rating INTEGER CHECK (feedback_rating >= 1 AND feedback_rating <= 5),
    notes TEXT
);


CREATE INDEX idx_social_connections_user_id ON social_connections(user_id);
CREATE INDEX idx_social_connections_platform ON social_connections(platform);
CREATE INDEX idx_social_tokens_connection_id ON social_tokens(connection_id);
CREATE INDEX idx_social_data_connection_id ON social_data(connection_id);
CREATE INDEX idx_social_data_platform ON social_data(platform);
CREATE INDEX idx_social_data_type ON social_data(data_type);
CREATE INDEX idx_social_data_synced_at ON social_data(synced_at);

CREATE INDEX idx_ai_rules_active ON ai_rules(is_active) WHERE is_active = true;
CREATE INDEX idx_ai_rules_priority ON ai_rules(priority DESC) WHERE is_active = true;
CREATE INDEX idx_ai_rules_type ON ai_rules USING GIN(rule_type);
CREATE INDEX idx_ai_rules_metadata ON ai_rules USING GIN(metadata);
CREATE INDEX idx_ai_rules_tags ON ai_rules USING GIN(tags);
CREATE INDEX idx_ai_rules_content_search ON ai_rules USING GIN(to_tsvector('english', title || ' ' || description || ' ' || content));

CREATE INDEX idx_ai_agent_profiles_type ON ai_agent_profiles(agent_type) WHERE is_active = true;
CREATE INDEX idx_ai_agent_profiles_rule_ids ON ai_agent_profiles USING GIN(rule_ids);

CREATE INDEX idx_ai_memory_context ON ai_memory_bank(context) WHERE is_active = true;
CREATE INDEX idx_ai_memory_type ON ai_memory_bank USING GIN(memory_type);
CREATE INDEX idx_ai_memory_tags ON ai_memory_bank USING GIN(tags);
CREATE INDEX idx_ai_memory_access ON ai_memory_bank(access_count DESC, last_accessed DESC) WHERE is_active = true;
CREATE INDEX idx_ai_memory_content_search ON ai_memory_bank USING GIN(to_tsvector('english', title || ' ' || content));

CREATE INDEX idx_ai_rule_applications_rule ON ai_rule_applications(rule_id, applied_at DESC);
CREATE INDEX idx_ai_rule_applications_agent ON ai_rule_applications(agent_type, applied_at DESC);


CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_social_connections_updated_at BEFORE UPDATE ON social_connections
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_social_tokens_updated_at BEFORE UPDATE ON social_tokens
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_ai_rules_updated_at
    BEFORE UPDATE ON ai_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
