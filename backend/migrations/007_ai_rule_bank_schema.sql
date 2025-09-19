-- AI Rule Bank & Memory Management Schema
-- This creates the database structure for the advanced AI agent rule bank and memory system

-- Extension for UUID generation and text similarity
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- AI Rules table for storing instructions and behavioral rules
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
    
    -- Indexes for performance
    CONSTRAINT ai_rules_priority_check CHECK (priority >= -100 AND priority <= 100)
);

-- AI Agent Profiles for different types of agents
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

-- AI Memory Bank for storing context and learning data
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

-- Rule Application History for tracking usage
CREATE TABLE ai_rule_applications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rule_id UUID NOT NULL REFERENCES ai_rules(id) ON DELETE CASCADE,
    agent_type VARCHAR(100) NOT NULL,
    context_hash VARCHAR(64), -- Hash of the context for privacy
    applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    success BOOLEAN,
    feedback_rating INTEGER CHECK (feedback_rating >= 1 AND feedback_rating <= 5),
    notes TEXT
);

-- Performance indexes for fast queries
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

-- Triggers for automatic timestamp updates
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_ai_rules_updated_at
    BEFORE UPDATE ON ai_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ai_agent_profiles_updated_at
    BEFORE UPDATE ON ai_agent_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to increment memory access count
CREATE OR REPLACE FUNCTION increment_memory_access(memory_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE ai_memory_bank 
    SET access_count = access_count + 1, 
        last_accessed = NOW()
    WHERE id = memory_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get relevant memories with similarity scoring
CREATE OR REPLACE FUNCTION get_similar_memories(
    context_text TEXT,
    limit_count INTEGER DEFAULT 10,
    similarity_threshold FLOAT DEFAULT 0.3
)
RETURNS TABLE (
    id UUID,
    title VARCHAR(255),
    content TEXT,
    memory_type JSONB,
    tags TEXT[],
    context VARCHAR(255),
    access_count BIGINT,
    similarity_score FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        m.id,
        m.title,
        m.content,
        m.memory_type,
        m.tags,
        m.context,
        m.access_count,
        similarity(m.context, context_text) as similarity_score
    FROM ai_memory_bank m
    WHERE m.is_active = true 
    AND similarity(m.context, context_text) > similarity_threshold
    ORDER BY similarity_score DESC, m.access_count DESC
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql;

-- Insert default AI agent profiles
INSERT INTO ai_agent_profiles (agent_type, name, description, capabilities, default_settings) VALUES
('code-reviewer', 'Code Review Agent', 'Specialized AI agent for code review and quality analysis', 
 ARRAY['code-analysis', 'security-review', 'performance-optimization', 'best-practices'],
 '{"strictness_level": "Moderate", "explain_reasoning": true, "max_response_length": 2000, "communication_style": "constructive"}'),

('security-auditor', 'Security Audit Agent', 'AI agent focused on security vulnerabilities and compliance',
 ARRAY['vulnerability-detection', 'security-patterns', 'compliance-checking', 'threat-modeling'],
 '{"strictness_level": "Strict", "explain_reasoning": true, "max_response_length": 1500, "communication_style": "detailed"}'),

('documentation-writer', 'Documentation Agent', 'AI agent specialized in creating and improving documentation',
 ARRAY['technical-writing', 'api-documentation', 'user-guides', 'code-comments'],
 '{"strictness_level": "Lenient", "explain_reasoning": false, "max_response_length": 3000, "communication_style": "clear"}'),

('performance-optimizer', 'Performance Optimization Agent', 'AI agent focused on performance analysis and optimization',
 ARRAY['performance-analysis', 'bottleneck-detection', 'optimization-recommendations', 'benchmarking'],
 '{"strictness_level": "Moderate", "explain_reasoning": true, "max_response_length": 2500, "communication_style": "analytical"}'),

('accessibility-checker', 'Accessibility Compliance Agent', 'AI agent specialized in accessibility standards and compliance',
 ARRAY['wcag-compliance', 'screen-reader-compatibility', 'keyboard-navigation', 'color-contrast'],
 '{"strictness_level": "Strict", "explain_reasoning": true, "max_response_length": 2000, "communication_style": "inclusive"}');

-- Insert sample AI rules for different contexts
INSERT INTO ai_rules (title, description, content, rule_type, metadata, priority, tags) VALUES
('Secure Coding Practices', 'Core security guidelines for all code',
 'Always validate input data, use parameterized queries to prevent SQL injection, implement proper authentication and authorization, sanitize output data, use HTTPS for all communications, keep dependencies updated.',
 '{"type": "SecurityGuideline"}',
 '{"languages": ["rust", "javascript", "python"], "categories": ["security", "best-practices"], "complexity_level": "Intermediate", "applicable_contexts": ["code-review", "development"]}',
 90,
 ARRAY['security', 'input-validation', 'sql-injection', 'authentication']),

('Performance Best Practices', 'Guidelines for writing performant code',
 'Use efficient algorithms and data structures, implement caching where appropriate, minimize database queries, use lazy loading for expensive operations, profile code to identify bottlenecks, optimize critical paths.',
 '{"type": "CodingStandard"}',
 '{"languages": ["rust", "javascript", "python"], "categories": ["performance", "optimization"], "complexity_level": "Advanced", "applicable_contexts": ["performance-review", "optimization"]}',
 80,
 ARRAY['performance', 'caching', 'optimization', 'algorithms']),

('Code Documentation Standards', 'Requirements for code documentation',
 'Write clear and concise comments explaining the why not the what, document all public APIs, include usage examples, maintain up-to-date README files, use consistent documentation format.',
 '{"type": "CodingStandard"}',
 '{"languages": ["rust", "javascript", "python"], "categories": ["documentation", "maintainability"], "complexity_level": "Beginner", "applicable_contexts": ["documentation", "code-review"]}',
 70,
 ARRAY['documentation', 'comments', 'api-docs', 'readme']),

('Accessibility Requirements', 'Web accessibility compliance guidelines',
 'Ensure proper semantic HTML, provide alt text for images, maintain good color contrast, support keyboard navigation, use ARIA labels appropriately, test with screen readers.',
 '{"type": "Custom", "Custom": "accessibility"}',
 '{"languages": ["html", "css", "javascript"], "categories": ["accessibility", "compliance"], "complexity_level": "Intermediate", "applicable_contexts": ["frontend-review", "accessibility-audit"]}',
 85,
 ARRAY['accessibility', 'wcag', 'semantic-html', 'aria']),

('Error Handling Patterns', 'Best practices for error handling',
 'Use Result types for recoverable errors, implement proper error propagation, provide meaningful error messages, log errors appropriately, handle edge cases gracefully.',
 '{"type": "CodingStandard"}',
 '{"languages": ["rust"], "categories": ["error-handling", "reliability"], "complexity_level": "Intermediate", "applicable_contexts": ["code-review", "development"]}',
 75,
 ARRAY['error-handling', 'result-type', 'logging', 'reliability']);

-- Insert sample memory bank entries
INSERT INTO ai_memory_bank (title, content, memory_type, tags, context) VALUES
('SQL Injection Prevention Example', 
 'Example of secure parameterized query: sqlx::query("SELECT * FROM users WHERE id = $1").bind(user_id).fetch_one(&pool).await',
 '{"type": "BestPracticeExample"}',
 ARRAY['security', 'sql', 'rust', 'sqlx'],
 'sql-injection-prevention'),

('Performance Optimization Case Study',
 'Reduced API response time from 2.5s to 150ms by implementing Redis caching for frequently accessed data and using connection pooling.',
 '{"type": "Solution"}',
 ARRAY['performance', 'redis', 'caching', 'api'],
 'performance-optimization'),

('Common Authentication Anti-pattern',
 'Avoid storing passwords in plain text or using weak hashing algorithms like MD5 or SHA1. Always use bcrypt, scrypt, or Argon2.',
 '{"type": "AntiPattern"}',
 ARRAY['security', 'authentication', 'passwords', 'hashing'],
 'authentication-security'),

('Accessibility Implementation Pattern',
 'Implementation of accessible dropdown menu with proper ARIA attributes and keyboard navigation support.',
 '{"type": "Pattern"}',
 ARRAY['accessibility', 'aria', 'dropdown', 'keyboard-navigation'],
 'accessibility-implementation'),

('Error Handling Best Practice',
 'Use custom error types with context: #[derive(Debug, thiserror::Error)] enum MyError { #[error("Database error: {0}")] Database(#[from] sqlx::Error) }',
 '{"type": "BestPracticeExample"}',
 ARRAY['error-handling', 'rust', 'thiserror', 'custom-errors'],
 'error-handling-patterns');

-- Create materialized view for fast rule lookup by agent type
CREATE MATERIALIZED VIEW ai_agent_rules AS
SELECT 
    ap.agent_type,
    r.id as rule_id,
    r.title,
    r.content,
    r.rule_type,
    r.metadata,
    r.priority,
    r.tags
FROM ai_agent_profiles ap
CROSS JOIN LATERAL unnest(ap.rule_ids) as rule_id
JOIN ai_rules r ON r.id = rule_id
WHERE ap.is_active = true AND r.is_active = true;

CREATE UNIQUE INDEX idx_ai_agent_rules_agent_rule ON ai_agent_rules(agent_type, rule_id);
CREATE INDEX idx_ai_agent_rules_agent_priority ON ai_agent_rules(agent_type, priority DESC);

-- Function to refresh the materialized view
CREATE OR REPLACE FUNCTION refresh_ai_agent_rules()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW ai_agent_rules;
END;
$$ LANGUAGE plpgsql;

-- Trigger to refresh materialized view when agent profiles or rules change
CREATE OR REPLACE FUNCTION trigger_refresh_agent_rules()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM refresh_ai_agent_rules();
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER refresh_agent_rules_on_profile_change
    AFTER INSERT OR UPDATE OR DELETE ON ai_agent_profiles
    FOR EACH STATEMENT
    EXECUTE FUNCTION trigger_refresh_agent_rules();

CREATE TRIGGER refresh_agent_rules_on_rules_change
    AFTER INSERT OR UPDATE OR DELETE ON ai_rules
    FOR EACH STATEMENT
    EXECUTE FUNCTION trigger_refresh_agent_rules();