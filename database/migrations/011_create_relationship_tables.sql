-- Migration: Create relationship graph tables for cross-source entity linking

-- Create entities table for storing canonical entities (functions, APIs, files, etc.)
CREATE TABLE IF NOT EXISTS entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL, -- code_symbol, api_endpoint, file, ticket, feature, pr, etc.
    canonical_name TEXT NOT NULL,
    normalized_name TEXT NOT NULL, -- Lowercase, normalized version for matching
    
    -- Scope/context information
    service_name VARCHAR(255), -- Which microservice/project this belongs to
    repository_id UUID, -- Reference to connected repository if applicable
    language VARCHAR(50), -- Programming language for code symbols
    
    -- Additional metadata
    metadata JSONB DEFAULT '{}',
    
    -- Tracking
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    occurrence_count INTEGER DEFAULT 1,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure uniqueness per scope
    UNIQUE(entity_type, normalized_name, service_name, language)
);

CREATE INDEX idx_entities_type ON entities(entity_type);
CREATE INDEX idx_entities_canonical_name ON entities(canonical_name);
CREATE INDEX idx_entities_normalized_name ON entities(normalized_name);
CREATE INDEX idx_entities_service ON entities(service_name);
CREATE INDEX idx_entities_language ON entities(language);
CREATE INDEX idx_entities_last_seen ON entities(last_seen_at);

-- Create chunk_entities table for linking chunks to entities
CREATE TABLE IF NOT EXISTS chunk_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chunk_id UUID NOT NULL, -- References document_chunks.id
    entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    
    -- Relationship type
    relation_type VARCHAR(50) NOT NULL, -- implements, describes, discusses, mentions, calls, imports, etc.
    
    -- Confidence and context
    confidence FLOAT NOT NULL DEFAULT 1.0, -- 0.0 to 1.0
    extraction_method VARCHAR(50), -- regex, ast_parser, similarity, manual, etc.
    context_snippet TEXT, -- Small snippet showing the mention
    
    -- Position in chunk
    start_position INTEGER,
    end_position INTEGER,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure we don't duplicate the same relation
    UNIQUE(chunk_id, entity_id, relation_type)
);

CREATE INDEX idx_chunk_entities_chunk ON chunk_entities(chunk_id);
CREATE INDEX idx_chunk_entities_entity ON chunk_entities(entity_id);
CREATE INDEX idx_chunk_entities_relation ON chunk_entities(relation_type);
CREATE INDEX idx_chunk_entities_confidence ON chunk_entities(confidence);

-- Create chunk_relations table for direct chunk-to-chunk relationships
CREATE TABLE IF NOT EXISTS chunk_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_chunk_id UUID NOT NULL, -- References document_chunks.id
    target_chunk_id UUID NOT NULL, -- References document_chunks.id
    
    -- Relationship type
    relation_type VARCHAR(50) NOT NULL, -- semantically_related, similar_topic, prerequisite, follows, etc.
    
    -- Similarity score
    score FLOAT NOT NULL DEFAULT 0.0, -- 0.0 to 1.0
    
    -- How was this relation discovered
    discovery_method VARCHAR(50), -- embedding_similarity, entity_overlap, temporal, manual, etc.
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure uniqueness and prevent self-relations
    UNIQUE(source_chunk_id, target_chunk_id, relation_type),
    CHECK (source_chunk_id != target_chunk_id)
);

CREATE INDEX idx_chunk_relations_source ON chunk_relations(source_chunk_id);
CREATE INDEX idx_chunk_relations_target ON chunk_relations(target_chunk_id);
CREATE INDEX idx_chunk_relations_type ON chunk_relations(relation_type);
CREATE INDEX idx_chunk_relations_score ON chunk_relations(score);
CREATE INDEX idx_chunk_relations_method ON chunk_relations(discovery_method);

-- Create entity_aliases table for alternative names/references to entities
CREATE TABLE IF NOT EXISTS entity_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    alias TEXT NOT NULL,
    alias_type VARCHAR(50), -- abbreviation, nickname, old_name, full_qualified, etc.
    confidence FLOAT DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(entity_id, alias)
);

CREATE INDEX idx_entity_aliases_entity ON entity_aliases(entity_id);
CREATE INDEX idx_entity_aliases_alias ON entity_aliases(alias);

-- Create entity_relations table for entity-to-entity relationships
CREATE TABLE IF NOT EXISTS entity_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    target_entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    
    relation_type VARCHAR(50) NOT NULL, -- calls, imports, extends, implements, depends_on, related_to, etc.
    
    confidence FLOAT DEFAULT 1.0,
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(source_entity_id, target_entity_id, relation_type),
    CHECK (source_entity_id != target_entity_id)
);

CREATE INDEX idx_entity_relations_source ON entity_relations(source_entity_id);
CREATE INDEX idx_entity_relations_target ON entity_relations(target_entity_id);
CREATE INDEX idx_entity_relations_type ON entity_relations(relation_type);

-- Create relationship_extraction_jobs table for tracking extraction tasks
CREATE TABLE IF NOT EXISTS relationship_extraction_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(50) NOT NULL, -- entity_extraction, similarity_linking, full_rebuild, etc.
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, running, completed, failed
    
    -- Scope
    document_ids UUID[], -- Specific documents to process, NULL for all
    chunk_ids UUID[], -- Specific chunks to process
    
    -- Progress tracking
    total_items INTEGER DEFAULT 0,
    processed_items INTEGER DEFAULT 0,
    failed_items INTEGER DEFAULT 0,
    
    -- Timing
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_ms BIGINT,
    
    -- Results
    entities_created INTEGER DEFAULT 0,
    entities_updated INTEGER DEFAULT 0,
    relations_created INTEGER DEFAULT 0,
    
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_relationship_jobs_status ON relationship_extraction_jobs(status);
CREATE INDEX idx_relationship_jobs_type ON relationship_extraction_jobs(job_type);
CREATE INDEX idx_relationship_jobs_created ON relationship_extraction_jobs(created_at);

-- Create triggers for updated_at
CREATE TRIGGER entities_updated_at
    BEFORE UPDATE ON entities
    FOR EACH ROW
    EXECUTE FUNCTION update_connected_accounts_updated_at();

CREATE TRIGGER entity_relations_updated_at
    BEFORE UPDATE ON entity_relations
    FOR EACH ROW
    EXECUTE FUNCTION update_connected_accounts_updated_at();

CREATE TRIGGER relationship_jobs_updated_at
    BEFORE UPDATE ON relationship_extraction_jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_connected_accounts_updated_at();

-- Create helper views for common queries

-- View: Chunks with their entities
CREATE OR REPLACE VIEW chunk_entities_view AS
SELECT 
    dc.id AS chunk_id,
    dc.document_id,
    dc.content AS chunk_content,
    sd.name AS document_name,
    sd.connector_type,
    e.id AS entity_id,
    e.entity_type,
    e.canonical_name AS entity_name,
    ce.relation_type,
    ce.confidence,
    ce.context_snippet
FROM document_chunks dc
JOIN source_documents sd ON dc.document_id = sd.id
LEFT JOIN chunk_entities ce ON dc.id = ce.chunk_id
LEFT JOIN entities e ON ce.entity_id = e.id;

-- View: Entities with their chunk count
CREATE OR REPLACE VIEW entity_popularity AS
SELECT 
    e.id,
    e.entity_type,
    e.canonical_name,
    e.service_name,
    e.language,
    COUNT(DISTINCT ce.chunk_id) AS chunk_count,
    COUNT(DISTINCT dc.document_id) AS document_count,
    ARRAY_AGG(DISTINCT sd.connector_type) AS source_types,
    e.last_seen_at
FROM entities e
LEFT JOIN chunk_entities ce ON e.id = ce.entity_id
LEFT JOIN document_chunks dc ON ce.chunk_id = dc.id
LEFT JOIN source_documents sd ON dc.document_id = sd.id
GROUP BY e.id, e.entity_type, e.canonical_name, e.service_name, e.language, e.last_seen_at;

-- View: Related chunks through entities
CREATE OR REPLACE VIEW related_chunks_via_entities AS
SELECT 
    ce1.chunk_id AS source_chunk_id,
    ce2.chunk_id AS target_chunk_id,
    e.entity_type,
    e.canonical_name AS shared_entity,
    ce1.relation_type AS source_relation,
    ce2.relation_type AS target_relation,
    (ce1.confidence + ce2.confidence) / 2.0 AS avg_confidence
FROM chunk_entities ce1
JOIN chunk_entities ce2 ON ce1.entity_id = ce2.entity_id
JOIN entities e ON ce1.entity_id = e.id
WHERE ce1.chunk_id < ce2.chunk_id; -- Avoid duplicates

-- Comment the schema
COMMENT ON TABLE entities IS 'Canonical entities (functions, APIs, files, etc.) extracted from all data sources';
COMMENT ON TABLE chunk_entities IS 'Links between document chunks and the entities they reference';
COMMENT ON TABLE chunk_relations IS 'Direct semantic relationships between chunks';
COMMENT ON TABLE entity_aliases IS 'Alternative names and references for entities';
COMMENT ON TABLE entity_relations IS 'Relationships between entities (e.g., function A calls function B)';
COMMENT ON TABLE relationship_extraction_jobs IS 'Background jobs for building the relationship graph';
