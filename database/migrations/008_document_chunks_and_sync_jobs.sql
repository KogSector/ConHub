-- Migration: Create document chunks and sync jobs tables for ingestion engine

-- Create document_chunks table for storing chunked document content
CREATE TABLE IF NOT EXISTS document_chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL,
    chunk_number INTEGER NOT NULL,
    content TEXT NOT NULL,
    start_offset INTEGER NOT NULL,
    end_offset INTEGER NOT NULL,
    metadata JSONB DEFAULT '{}',
    embedding_vector VECTOR(1536), -- OpenAI embedding dimension
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_document_chunks_document FOREIGN KEY (document_id) REFERENCES source_documents(id) ON DELETE CASCADE,
    UNIQUE(document_id, chunk_number)
);

CREATE INDEX idx_document_chunks_document_id ON document_chunks(document_id);
CREATE INDEX idx_document_chunks_chunk_number ON document_chunks(chunk_number);
CREATE INDEX idx_document_chunks_embedding ON document_chunks USING ivfflat (embedding_vector vector_cosine_ops);

-- Create sync_jobs table for scheduling and tracking sync operations
CREATE TABLE IF NOT EXISTS sync_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    account_id UUID NOT NULL,
    connector_type VARCHAR(50) NOT NULL,
    job_type VARCHAR(50) NOT NULL DEFAULT 'full_sync', -- full_sync, incremental_sync, webhook_sync
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, running, completed, failed, cancelled
    priority INTEGER NOT NULL DEFAULT 5, -- 1 (highest) to 10 (lowest)
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    config JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_sync_jobs_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_sync_jobs_account FOREIGN KEY (account_id) REFERENCES connected_accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_sync_jobs_user_id ON sync_jobs(user_id);
CREATE INDEX idx_sync_jobs_account_id ON sync_jobs(account_id);
CREATE INDEX idx_sync_jobs_status ON sync_jobs(status);
CREATE INDEX idx_sync_jobs_scheduled_at ON sync_jobs(scheduled_at);
CREATE INDEX idx_sync_jobs_priority ON sync_jobs(priority);

-- Create sync_runs table for detailed sync execution tracking
CREATE TABLE IF NOT EXISTS sync_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL,
    run_number INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'running',
    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMPTZ,
    documents_discovered INTEGER DEFAULT 0,
    documents_processed INTEGER DEFAULT 0,
    documents_failed INTEGER DEFAULT 0,
    documents_skipped INTEGER DEFAULT 0,
    bytes_processed BIGINT DEFAULT 0,
    error_details JSONB DEFAULT '{}',
    performance_metrics JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_sync_runs_job FOREIGN KEY (job_id) REFERENCES sync_jobs(id) ON DELETE CASCADE,
    UNIQUE(job_id, run_number)
);

CREATE INDEX idx_sync_runs_job_id ON sync_runs(job_id);
CREATE INDEX idx_sync_runs_status ON sync_runs(status);
CREATE INDEX idx_sync_runs_started_at ON sync_runs(started_at);

-- Create connector_configs table for storing connector-specific configurations
CREATE TABLE IF NOT EXISTS connector_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    connector_type VARCHAR(50) NOT NULL,
    config_name VARCHAR(255) NOT NULL,
    config_data JSONB NOT NULL DEFAULT '{}',
    is_default BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_connector_configs_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, connector_type, config_name)
);

CREATE INDEX idx_connector_configs_user_id ON connector_configs(user_id);
CREATE INDEX idx_connector_configs_connector_type ON connector_configs(connector_type);
CREATE INDEX idx_connector_configs_is_default ON connector_configs(is_default);

-- Create updated_at triggers
CREATE OR REPLACE FUNCTION update_document_chunks_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER document_chunks_updated_at
    BEFORE UPDATE ON document_chunks
    FOR EACH ROW
    EXECUTE FUNCTION update_document_chunks_updated_at();

CREATE TRIGGER sync_jobs_updated_at
    BEFORE UPDATE ON sync_jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_connected_accounts_updated_at();

CREATE TRIGGER connector_configs_updated_at
    BEFORE UPDATE ON connector_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_connected_accounts_updated_at();

-- Add indexes for better query performance
CREATE INDEX idx_source_documents_connector_type_indexed ON source_documents(connector_type, indexed_at);
CREATE INDEX idx_embedding_queue_status_created ON embedding_queue(status, created_at);

-- Add feature toggle support in database
CREATE TABLE IF NOT EXISTS feature_toggles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feature_name VARCHAR(100) NOT NULL UNIQUE,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    description TEXT,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default feature toggles
INSERT INTO feature_toggles (feature_name, is_enabled, description) VALUES
('Auth', true, 'Enable authentication and authorization'),
('Heavy', false, 'Enable heavy operations like embedding and indexing'),
('Docker', false, 'Use Docker for builds and deployments'),
('Redis', true, 'Enable Redis for caching and sessions'),
('Billing', false, 'Enable billing and subscription features'),
('GitHubConnector', true, 'Enable GitHub repository connector'),
('GoogleDriveConnector', true, 'Enable Google Drive connector'),
('DropboxConnector', true, 'Enable Dropbox connector'),
('SlackConnector', true, 'Enable Slack connector'),
('UrlConnector', true, 'Enable URL crawler connector'),
('BitbucketConnector', true, 'Enable Bitbucket repository connector'),
('NotionConnector', false, 'Enable Notion connector (coming soon)')
ON CONFLICT (feature_name) DO NOTHING;

CREATE TRIGGER feature_toggles_updated_at
    BEFORE UPDATE ON feature_toggles
    FOR EACH ROW
    EXECUTE FUNCTION update_connected_accounts_updated_at();
