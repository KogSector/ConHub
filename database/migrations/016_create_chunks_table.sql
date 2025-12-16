-- Migration: Create new chunks table keyed by chunk_id
-- This is the single source of truth for chunk text content.
-- Vector DB stores embeddings + metadata; graph stores relationships + evidence pointers.

-- Create the chunks table (primary key = chunk_id)
CREATE TABLE IF NOT EXISTS chunks (
    chunk_id UUID PRIMARY KEY,
    
    -- Link to source document/item
    tenant_id UUID NOT NULL,
    source_item_id UUID NOT NULL,  -- References the original file/doc/thread
    source_id UUID NOT NULL,       -- Connected account / integration ID
    
    -- Chunk position within the source item
    chunk_index INT NOT NULL,
    
    -- Content (single source of truth for text)
    content TEXT NOT NULL,
    content_hash TEXT NOT NULL,    -- For change detection (e.g., SHA256)
    
    -- Chunk semantics
    source_kind VARCHAR(50) NOT NULL,  -- code_repo, document, chat, wiki, ticketing, email, web
    block_type VARCHAR(50),            -- code, text, table, comment, heading, etc.
    language VARCHAR(50),              -- Programming language for code chunks
    
    -- Rich metadata (provenance, filters, etc.)
    metadata JSONB NOT NULL DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure we don't duplicate chunks for the same source item + index
    UNIQUE(source_item_id, chunk_index)
);

-- Indexes for common access patterns
CREATE INDEX idx_chunks_tenant_id ON chunks(tenant_id);
CREATE INDEX idx_chunks_source_item_id ON chunks(source_item_id);
CREATE INDEX idx_chunks_source_id ON chunks(source_id);
CREATE INDEX idx_chunks_source_kind ON chunks(source_kind);
CREATE INDEX idx_chunks_content_hash ON chunks(content_hash);
CREATE INDEX idx_chunks_updated_at ON chunks(updated_at);
CREATE INDEX idx_chunks_metadata ON chunks USING gin(metadata);

-- Full-text search on chunk content (optional but useful)
CREATE INDEX idx_chunks_content_fts ON chunks USING gin(to_tsvector('english', content));

-- Trigger for auto-updating updated_at
CREATE OR REPLACE FUNCTION update_chunks_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER chunks_updated_at_trigger
    BEFORE UPDATE ON chunks
    FOR EACH ROW
    EXECUTE FUNCTION update_chunks_updated_at();

-- Comments
COMMENT ON TABLE chunks IS 'Single source of truth for chunk text content. Vector DB stores embeddings; graph stores relationship pointers.';
COMMENT ON COLUMN chunks.chunk_id IS 'Stable unique ID for this chunk, used as key in vector DB and graph evidence tables.';
COMMENT ON COLUMN chunks.content_hash IS 'Hash of content for change detection during incremental sync.';
COMMENT ON COLUMN chunks.source_kind IS 'Type of source: code_repo, document, chat, wiki, ticketing, email, web.';
COMMENT ON COLUMN chunks.block_type IS 'Semantic type of chunk content: code, text, table, comment, heading, etc.';
