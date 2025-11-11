-- Migration: Create connected_accounts table for storing external data source connections

CREATE TABLE IF NOT EXISTS connected_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    connector_type VARCHAR(50) NOT NULL,
    account_name VARCHAR(255) NOT NULL,
    account_identifier VARCHAR(255) NOT NULL,
    credentials JSONB NOT NULL DEFAULT '{}',
    status JSONB NOT NULL DEFAULT '{"status": "connected"}',
    last_sync_at TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_connected_accounts_user_id ON connected_accounts(user_id);
CREATE INDEX idx_connected_accounts_connector_type ON connected_accounts(connector_type);
CREATE INDEX idx_connected_accounts_status ON connected_accounts((status->>'status'));

-- Create updated_at trigger
CREATE OR REPLACE FUNCTION update_connected_accounts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER connected_accounts_updated_at
    BEFORE UPDATE ON connected_accounts
    FOR EACH ROW
    EXECUTE FUNCTION update_connected_accounts_updated_at();

-- Create documents table for storing document metadata from all sources
CREATE TABLE IF NOT EXISTS source_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL,
    connector_type VARCHAR(50) NOT NULL,
    external_id VARCHAR(500) NOT NULL,
    name VARCHAR(500) NOT NULL,
    path TEXT,
    content_type VARCHAR(50),
    mime_type VARCHAR(100),
    size BIGINT,
    url TEXT,
    parent_id VARCHAR(500),
    is_folder BOOLEAN DEFAULT FALSE,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    indexed_at TIMESTAMPTZ,
    
    CONSTRAINT fk_source FOREIGN KEY (source_id) REFERENCES connected_accounts(id) ON DELETE CASCADE,
    UNIQUE(source_id, external_id)
);

CREATE INDEX idx_source_documents_source_id ON source_documents(source_id);
CREATE INDEX idx_source_documents_connector_type ON source_documents(connector_type);
CREATE INDEX idx_source_documents_external_id ON source_documents(external_id);
CREATE INDEX idx_source_documents_name ON source_documents(name);
CREATE INDEX idx_source_documents_is_folder ON source_documents(is_folder);

-- Create updated_at trigger for source_documents
CREATE TRIGGER source_documents_updated_at
    BEFORE UPDATE ON source_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_connected_accounts_updated_at();

-- Create embeddings queue table
CREATE TABLE IF NOT EXISTS embedding_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    retry_count INT NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMPTZ,
    
    CONSTRAINT fk_document FOREIGN KEY (document_id) REFERENCES source_documents(id) ON DELETE CASCADE
);

CREATE INDEX idx_embedding_queue_status ON embedding_queue(status);
CREATE INDEX idx_embedding_queue_document_id ON embedding_queue(document_id);
CREATE INDEX idx_embedding_queue_created_at ON embedding_queue(created_at);
