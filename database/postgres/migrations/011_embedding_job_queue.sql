-- Embedding Update Job Queue Migration
-- This migration creates tables for async embedding updates and tracking

-- ============================================================================
-- EMBEDDING UPDATE QUEUE
-- Tracks pending embedding generation/regeneration jobs
-- ============================================================================

CREATE TABLE IF NOT EXISTS embedding_update_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Entity identification
    entity_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,  -- FUNCTION, CLASS, DOCUMENT, CHUNK, etc.
    node_type VARCHAR(50) NOT NULL,     -- 'entity' or 'chunk'
    
    -- Source identification
    source VARCHAR(50) NOT NULL,        -- github, dropbox, notion, etc.
    source_id VARCHAR(500) NOT NULL,    -- unique identifier in source system
    
    -- Job status
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- pending, processing, completed, failed
    reason_for_update VARCHAR(100) NOT NULL,        -- source_changed, model_changed, manual_request, initial
    
    -- Timing
    source_modified_timestamp TIMESTAMPTZ,
    processing_started_at TIMESTAMPTZ,
    processing_completed_at TIMESTAMPTZ,
    
    -- Retry handling
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    error_message TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- EMBEDDING METADATA
-- Tracks embedding generation history for audit and model migration
-- ============================================================================

CREATE TABLE IF NOT EXISTS embedding_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Entity identification
    entity_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    node_type VARCHAR(50) NOT NULL,     -- 'entity' or 'chunk'
    
    -- Embedding details
    model_used VARCHAR(255) NOT NULL,           -- e.g., "sentence-transformers-384"
    provider_used VARCHAR(100) NOT NULL,        -- e.g., "openai", "cohere", "voyage"
    vector_dimension INTEGER NOT NULL,          -- 384, 768, 1024, etc.
    
    -- Text that was embedded (for debugging/reprocessing)
    text_hash VARCHAR(64),                      -- MD5 hash of embedded text
    text_length INTEGER,                        -- Length of text that was embedded
    
    -- Generation timing
    generation_duration_ms INTEGER,             -- How long embedding took
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- For tracking model migrations
    previous_metadata_id UUID REFERENCES embedding_metadata(id),
    is_current BOOLEAN NOT NULL DEFAULT TRUE
);

-- ============================================================================
-- EMBEDDING CONFIGURATION
-- Stores configurable weights for confidence boosting
-- ============================================================================

CREATE TABLE IF NOT EXISTS embedding_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default configuration values
INSERT INTO embedding_config (config_key, config_value, description) VALUES
    ('vector_dimension', '384', 'Dimension of embedding vectors'),
    ('default_model', '"sentence-transformers-all-MiniLM-L6-v2"', 'Default embedding model'),
    ('default_provider', '"local"', 'Default embedding provider'),
    ('explicit_mention_boost', '0.15', 'Confidence boost when document mentions entity name'),
    ('temporal_proximity_boost', '0.10', 'Max confidence boost for temporal proximity'),
    ('author_overlap_boost', '0.10', 'Confidence boost when authors match'),
    ('min_similarity_threshold', '0.5', 'Minimum similarity score for cross-source links'),
    ('stale_embedding_days', '30', 'Days after which embeddings are considered stale')
ON CONFLICT (config_key) DO NOTHING;

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Queue indexes for efficient polling
CREATE INDEX IF NOT EXISTS idx_embedding_queue_status 
    ON embedding_update_queue(status);
CREATE INDEX IF NOT EXISTS idx_embedding_queue_status_created 
    ON embedding_update_queue(status, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_embedding_queue_entity 
    ON embedding_update_queue(entity_id);
CREATE INDEX IF NOT EXISTS idx_embedding_queue_entity_type 
    ON embedding_update_queue(entity_type);
CREATE INDEX IF NOT EXISTS idx_embedding_queue_processing 
    ON embedding_update_queue(status, processing_started_at) 
    WHERE status = 'processing';

-- Metadata indexes
CREATE INDEX IF NOT EXISTS idx_embedding_metadata_entity 
    ON embedding_metadata(entity_id);
CREATE INDEX IF NOT EXISTS idx_embedding_metadata_current 
    ON embedding_metadata(entity_id, is_current) 
    WHERE is_current = TRUE;
CREATE INDEX IF NOT EXISTS idx_embedding_metadata_generated 
    ON embedding_metadata(generated_at DESC);
CREATE INDEX IF NOT EXISTS idx_embedding_metadata_model 
    ON embedding_metadata(model_used);

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_embedding_queue_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_embedding_queue_timestamp
    BEFORE UPDATE ON embedding_update_queue
    FOR EACH ROW
    EXECUTE FUNCTION update_embedding_queue_timestamp();

CREATE TRIGGER trigger_update_embedding_config_timestamp
    BEFORE UPDATE ON embedding_config
    FOR EACH ROW
    EXECUTE FUNCTION update_embedding_queue_timestamp();

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Enqueue entity for embedding update
CREATE OR REPLACE FUNCTION enqueue_embedding_update(
    p_entity_id UUID,
    p_entity_type VARCHAR(100),
    p_node_type VARCHAR(50),
    p_source VARCHAR(50),
    p_source_id VARCHAR(500),
    p_reason VARCHAR(100)
)
RETURNS UUID AS $$
DECLARE
    v_job_id UUID;
BEGIN
    -- Upsert: if pending job exists, just update timestamp
    INSERT INTO embedding_update_queue (
        entity_id, entity_type, node_type, source, source_id, 
        reason_for_update, source_modified_timestamp
    ) VALUES (
        p_entity_id, p_entity_type, p_node_type, p_source, p_source_id,
        p_reason, NOW()
    )
    ON CONFLICT DO NOTHING
    RETURNING id INTO v_job_id;
    
    RETURN v_job_id;
END;
$$ LANGUAGE plpgsql;

-- Get next batch of jobs to process
CREATE OR REPLACE FUNCTION get_pending_embedding_jobs(p_batch_size INTEGER DEFAULT 100)
RETURNS SETOF embedding_update_queue AS $$
BEGIN
    RETURN QUERY
    UPDATE embedding_update_queue
    SET status = 'processing',
        processing_started_at = NOW()
    WHERE id IN (
        SELECT id FROM embedding_update_queue
        WHERE status = 'pending'
        ORDER BY created_at ASC
        LIMIT p_batch_size
        FOR UPDATE SKIP LOCKED
    )
    RETURNING *;
END;
$$ LANGUAGE plpgsql;

-- Mark job as completed
CREATE OR REPLACE FUNCTION complete_embedding_job(
    p_job_id UUID,
    p_success BOOLEAN,
    p_error_message TEXT DEFAULT NULL
)
RETURNS VOID AS $$
BEGIN
    IF p_success THEN
        UPDATE embedding_update_queue
        SET status = 'completed',
            processing_completed_at = NOW()
        WHERE id = p_job_id;
    ELSE
        UPDATE embedding_update_queue
        SET status = CASE 
                WHEN retry_count >= max_retries THEN 'failed'
                ELSE 'pending'
            END,
            retry_count = retry_count + 1,
            error_message = p_error_message,
            processing_completed_at = NOW()
        WHERE id = p_job_id;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Get queue statistics
CREATE OR REPLACE FUNCTION get_embedding_queue_stats()
RETURNS TABLE (
    status VARCHAR(50),
    count BIGINT,
    oldest_created_at TIMESTAMPTZ,
    newest_created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        e.status,
        COUNT(*) as count,
        MIN(e.created_at) as oldest_created_at,
        MAX(e.created_at) as newest_created_at
    FROM embedding_update_queue e
    GROUP BY e.status;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE embedding_update_queue IS 'Queue for async embedding generation/regeneration jobs';
COMMENT ON TABLE embedding_metadata IS 'Audit trail of embedding generations for model migration tracking';
COMMENT ON TABLE embedding_config IS 'Configurable parameters for embedding and cross-source linking';

COMMENT ON COLUMN embedding_update_queue.status IS 'Job status: pending, processing, completed, failed';
COMMENT ON COLUMN embedding_update_queue.reason_for_update IS 'Why embedding needs update: source_changed, model_changed, manual_request, initial';
COMMENT ON COLUMN embedding_metadata.is_current IS 'TRUE for the most recent embedding, FALSE for historical records';
