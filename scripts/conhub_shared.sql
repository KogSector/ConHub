-- ConHub Shared Database Initial Dump
-- Generated for team synchronization

-- Create tables (basic structure)
CREATE TABLE IF NOT EXISTS users (
    user_id VARCHAR(255) PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS repositories (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    url VARCHAR(500) NOT NULL,
    owner_id VARCHAR(255) REFERENCES users(user_id),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS documents (
    id VARCHAR(255) PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    source_type VARCHAR(100),
    owner_id VARCHAR(255) REFERENCES users(user_id),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Sample data for development
INSERT INTO users (user_id, email, name, created_at) VALUES
('user-1', 'alice@conhub.dev', 'Alice Developer', NOW()),
('user-2', 'bob@conhub.dev', 'Bob Engineer', NOW()),
('user-3', 'charlie@conhub.dev', 'Charlie Designer', NOW())
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO repositories (id, name, url, owner_id, created_at) VALUES
('repo-1', 'conhub-frontend', 'https://github.com/team/conhub-frontend', 'user-1', NOW()),
('repo-2', 'conhub-backend', 'https://github.com/team/conhub-backend', 'user-2', NOW()),
('repo-3', 'shared-components', 'https://github.com/team/shared-components', 'user-1', NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO documents (id, title, content, source_type, owner_id, created_at) VALUES
('doc-1', 'API Documentation', 'ConHub API endpoints and usage', 'notion', 'user-1', NOW()),
('doc-2', 'Architecture Guide', 'System architecture overview', 'confluence', 'user-2', NOW()),
('doc-3', 'Development Setup', 'Local development environment setup', 'google_drive', 'user-3', NOW())
ON CONFLICT (id) DO NOTHING;