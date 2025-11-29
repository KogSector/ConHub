-- Migration: GitHub App Integration for Code, Issues, and PRs
-- This migration adds tables for GitHub App installations and per-repo sync configurations

-- ============================================================================
-- GitHub App Installations
-- ============================================================================

-- Store GitHub App installations per tenant
CREATE TABLE IF NOT EXISTS github_app_installations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    installation_id BIGINT NOT NULL UNIQUE,
    account_login VARCHAR(255) NOT NULL,
    account_type VARCHAR(50) NOT NULL DEFAULT 'Organization', -- Organization or User
    account_id BIGINT,
    app_id BIGINT NOT NULL,
    permissions JSONB DEFAULT '{}',
    events JSONB DEFAULT '[]',
    target_type VARCHAR(50), -- Organization, User, etc.
    repository_selection VARCHAR(50) DEFAULT 'all', -- all or selected
    suspended_at TIMESTAMPTZ,
    suspended_by VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'active', -- active, suspended, removed
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_github_app_installations_tenant FOREIGN KEY (tenant_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_github_app_installations_tenant_id ON github_app_installations(tenant_id);
CREATE INDEX idx_github_app_installations_installation_id ON github_app_installations(installation_id);
CREATE INDEX idx_github_app_installations_account_login ON github_app_installations(account_login);
CREATE INDEX idx_github_app_installations_status ON github_app_installations(status);

-- ============================================================================
-- GitHub Repository Configurations
-- ============================================================================

-- Per-repository sync configuration
CREATE TABLE IF NOT EXISTS github_repo_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    installation_id UUID NOT NULL,
    github_repo_id BIGINT NOT NULL,
    full_name VARCHAR(255) NOT NULL, -- owner/repo format
    name VARCHAR(255) NOT NULL,
    owner VARCHAR(255) NOT NULL,
    default_branch VARCHAR(255) NOT NULL DEFAULT 'main',
    private BOOLEAN NOT NULL DEFAULT false,
    description TEXT,
    html_url VARCHAR(512),
    clone_url VARCHAR(512),
    
    -- Sync configuration flags
    sync_code BOOLEAN NOT NULL DEFAULT true,
    sync_issues BOOLEAN NOT NULL DEFAULT false,
    sync_prs BOOLEAN NOT NULL DEFAULT false,
    sync_wiki BOOLEAN NOT NULL DEFAULT false,
    sync_discussions BOOLEAN NOT NULL DEFAULT false,
    
    -- Code sync settings
    code_branches JSONB DEFAULT '[]', -- List of branches to sync, empty = default only
    code_exclude_paths JSONB DEFAULT '["node_modules", "dist", "build", ".git", "vendor", "__pycache__"]',
    code_include_extensions JSONB DEFAULT '[]', -- Empty = all text files
    code_max_file_size_mb INTEGER DEFAULT 5,
    
    -- Issues sync settings
    issues_include_closed BOOLEAN DEFAULT false,
    issues_since_days INTEGER DEFAULT 90, -- Only sync issues from last N days
    issues_labels_filter JSONB DEFAULT '[]', -- Empty = all labels
    
    -- PRs sync settings
    prs_include_closed BOOLEAN DEFAULT false,
    prs_include_merged BOOLEAN DEFAULT true,
    prs_since_days INTEGER DEFAULT 90,
    prs_include_diffs BOOLEAN DEFAULT false,
    
    -- Sync status tracking
    last_code_sync_at TIMESTAMPTZ,
    last_code_sync_commit VARCHAR(40),
    last_issues_sync_at TIMESTAMPTZ,
    last_issues_cursor VARCHAR(255),
    last_prs_sync_at TIMESTAMPTZ,
    last_prs_cursor VARCHAR(255),
    
    -- Metadata
    languages JSONB DEFAULT '[]',
    topics JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_github_repo_configs_tenant FOREIGN KEY (tenant_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_github_repo_configs_installation FOREIGN KEY (installation_id) REFERENCES github_app_installations(id) ON DELETE CASCADE,
    UNIQUE(installation_id, github_repo_id)
);

CREATE INDEX idx_github_repo_configs_tenant_id ON github_repo_configs(tenant_id);
CREATE INDEX idx_github_repo_configs_installation_id ON github_repo_configs(installation_id);
CREATE INDEX idx_github_repo_configs_full_name ON github_repo_configs(full_name);
CREATE INDEX idx_github_repo_configs_is_active ON github_repo_configs(is_active);
CREATE INDEX idx_github_repo_configs_sync_flags ON github_repo_configs(sync_code, sync_issues, sync_prs);

-- ============================================================================
-- GitHub Sync Jobs (extends sync_jobs for GitHub-specific tracking)
-- ============================================================================

CREATE TABLE IF NOT EXISTS github_sync_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    installation_id UUID NOT NULL,
    repo_config_id UUID NOT NULL,
    job_type VARCHAR(50) NOT NULL, -- code, issues, prs, full
    
    -- Job status
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, running, completed, failed, cancelled
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    
    -- Progress tracking
    items_total INTEGER DEFAULT 0,
    items_processed INTEGER DEFAULT 0,
    items_failed INTEGER DEFAULT 0,
    chunks_created INTEGER DEFAULT 0,
    
    -- For code sync
    target_branch VARCHAR(255),
    target_commit VARCHAR(40),
    base_commit VARCHAR(40), -- For incremental sync
    
    -- For issues/PRs sync
    cursor_position VARCHAR(255),
    
    -- Metadata
    config JSONB DEFAULT '{}',
    metrics JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_github_sync_jobs_tenant FOREIGN KEY (tenant_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_github_sync_jobs_installation FOREIGN KEY (installation_id) REFERENCES github_app_installations(id) ON DELETE CASCADE,
    CONSTRAINT fk_github_sync_jobs_repo FOREIGN KEY (repo_config_id) REFERENCES github_repo_configs(id) ON DELETE CASCADE
);

CREATE INDEX idx_github_sync_jobs_tenant_id ON github_sync_jobs(tenant_id);
CREATE INDEX idx_github_sync_jobs_repo_config_id ON github_sync_jobs(repo_config_id);
CREATE INDEX idx_github_sync_jobs_status ON github_sync_jobs(status);
CREATE INDEX idx_github_sync_jobs_job_type ON github_sync_jobs(job_type);
CREATE INDEX idx_github_sync_jobs_created_at ON github_sync_jobs(created_at DESC);

-- ============================================================================
-- GitHub Documents (code files, issues, PRs)
-- ============================================================================

CREATE TABLE IF NOT EXISTS github_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    repo_config_id UUID NOT NULL,
    
    -- Document identification
    doc_type VARCHAR(50) NOT NULL, -- code_file, issue, pr, pr_diff, wiki_page
    external_id VARCHAR(255) NOT NULL, -- SHA for code, number for issues/PRs
    
    -- Common fields
    title VARCHAR(1024),
    content TEXT,
    content_hash VARCHAR(64), -- For deduplication
    url VARCHAR(512),
    
    -- Code-specific fields
    file_path VARCHAR(1024),
    file_language VARCHAR(100),
    branch VARCHAR(255),
    commit_sha VARCHAR(40),
    
    -- Issue/PR-specific fields
    number INTEGER,
    state VARCHAR(50), -- open, closed, merged
    author VARCHAR(255),
    assignees JSONB DEFAULT '[]',
    labels JSONB DEFAULT '[]',
    milestone VARCHAR(255),
    
    -- PR-specific fields
    base_branch VARCHAR(255),
    head_branch VARCHAR(255),
    merged_at TIMESTAMPTZ,
    merged_by VARCHAR(255),
    
    -- Timestamps from GitHub
    github_created_at TIMESTAMPTZ,
    github_updated_at TIMESTAMPTZ,
    github_closed_at TIMESTAMPTZ,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Indexing status
    is_indexed BOOLEAN DEFAULT false,
    indexed_at TIMESTAMPTZ,
    chunk_count INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_github_documents_tenant FOREIGN KEY (tenant_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_github_documents_repo FOREIGN KEY (repo_config_id) REFERENCES github_repo_configs(id) ON DELETE CASCADE,
    UNIQUE(repo_config_id, doc_type, external_id)
);

CREATE INDEX idx_github_documents_tenant_id ON github_documents(tenant_id);
CREATE INDEX idx_github_documents_repo_config_id ON github_documents(repo_config_id);
CREATE INDEX idx_github_documents_doc_type ON github_documents(doc_type);
CREATE INDEX idx_github_documents_external_id ON github_documents(external_id);
CREATE INDEX idx_github_documents_file_path ON github_documents(file_path) WHERE doc_type = 'code_file';
CREATE INDEX idx_github_documents_number ON github_documents(number) WHERE doc_type IN ('issue', 'pr');
CREATE INDEX idx_github_documents_is_indexed ON github_documents(is_indexed);
CREATE INDEX idx_github_documents_content_hash ON github_documents(content_hash);

-- ============================================================================
-- GitHub Issue/PR Comments (for conversation chunking)
-- ============================================================================

CREATE TABLE IF NOT EXISTS github_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    document_id UUID NOT NULL, -- References github_documents
    
    comment_id BIGINT NOT NULL,
    comment_type VARCHAR(50) NOT NULL, -- issue_comment, pr_comment, pr_review, pr_review_comment
    author VARCHAR(255),
    body TEXT,
    
    -- For PR review comments
    diff_hunk TEXT,
    path VARCHAR(1024),
    position INTEGER,
    original_position INTEGER,
    commit_id VARCHAR(40),
    
    -- For PR reviews
    review_state VARCHAR(50), -- approved, changes_requested, commented, dismissed
    
    github_created_at TIMESTAMPTZ,
    github_updated_at TIMESTAMPTZ,
    
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_github_comments_tenant FOREIGN KEY (tenant_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_github_comments_document FOREIGN KEY (document_id) REFERENCES github_documents(id) ON DELETE CASCADE,
    UNIQUE(document_id, comment_id)
);

CREATE INDEX idx_github_comments_document_id ON github_comments(document_id);
CREATE INDEX idx_github_comments_comment_type ON github_comments(comment_type);
CREATE INDEX idx_github_comments_author ON github_comments(author);

-- ============================================================================
-- OAuth State for GitHub App Installation Flow
-- ============================================================================

CREATE TABLE IF NOT EXISTS github_oauth_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    state_token VARCHAR(255) NOT NULL UNIQUE,
    redirect_uri VARCHAR(512),
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_github_oauth_states_tenant FOREIGN KEY (tenant_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_github_oauth_states_state_token ON github_oauth_states(state_token);
CREATE INDEX idx_github_oauth_states_expires_at ON github_oauth_states(expires_at);

-- ============================================================================
-- Triggers for updated_at
-- ============================================================================

CREATE OR REPLACE FUNCTION update_github_tables_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER github_app_installations_updated_at
    BEFORE UPDATE ON github_app_installations
    FOR EACH ROW
    EXECUTE FUNCTION update_github_tables_updated_at();

CREATE TRIGGER github_repo_configs_updated_at
    BEFORE UPDATE ON github_repo_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_github_tables_updated_at();

CREATE TRIGGER github_sync_jobs_updated_at
    BEFORE UPDATE ON github_sync_jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_github_tables_updated_at();

CREATE TRIGGER github_documents_updated_at
    BEFORE UPDATE ON github_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_github_tables_updated_at();

CREATE TRIGGER github_comments_updated_at
    BEFORE UPDATE ON github_comments
    FOR EACH ROW
    EXECUTE FUNCTION update_github_tables_updated_at();

-- ============================================================================
-- Add SourceKind values for GitHub content types
-- ============================================================================

-- Update feature toggles to include GitHub App
INSERT INTO feature_toggles (feature_name, is_enabled, description) VALUES
('GitHubAppConnector', true, 'Enable GitHub App-based repository connector'),
('GitHubIssuesSync', true, 'Enable GitHub Issues synchronization'),
('GitHubPRsSync', true, 'Enable GitHub Pull Requests synchronization')
ON CONFLICT (feature_name) DO NOTHING;
