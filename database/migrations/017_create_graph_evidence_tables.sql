-- Migration: Create graph evidence tables for dual-truth architecture
-- Graph stores relationships + evidence pointers (chunk_ids); never stores chunk text.
-- Every entity and relationship must have evidence linking it to source chunks.

-- Entity evidence: links entities to the chunks that justify their existence
CREATE TABLE IF NOT EXISTS entity_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID NOT NULL,  -- References entities.id (from migration 010/011)
    chunk_id UUID NOT NULL REFERENCES chunks(chunk_id) ON DELETE CASCADE,
    
    -- Evidence quality
    confidence REAL NOT NULL DEFAULT 1.0,  -- 0.0 to 1.0
    extraction_method VARCHAR(50),          -- ast, regex, llm, manual, etc.
    
    -- Optional: span within the chunk where the entity appears
    span_start INT,
    span_end INT,
    
    -- Additional context
    attributes JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure we don't duplicate the same evidence
    UNIQUE(entity_id, chunk_id)
);

CREATE INDEX idx_entity_evidence_entity_id ON entity_evidence(entity_id);
CREATE INDEX idx_entity_evidence_chunk_id ON entity_evidence(chunk_id);
CREATE INDEX idx_entity_evidence_confidence ON entity_evidence(confidence);

-- Relationship evidence: links relationships to the chunks that justify them
CREATE TABLE IF NOT EXISTS relationship_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    relationship_id UUID NOT NULL,  -- References relationships.id or entity_relations.id
    chunk_id UUID NOT NULL REFERENCES chunks(chunk_id) ON DELETE CASCADE,
    
    -- Evidence quality
    confidence REAL NOT NULL DEFAULT 1.0,  -- 0.0 to 1.0
    extraction_method VARCHAR(50),          -- ast, regex, llm, co_occurrence, manual, etc.
    
    -- Optional: span within the chunk where the relationship is evidenced
    span_start INT,
    span_end INT,
    
    -- Additional context
    attributes JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure we don't duplicate the same evidence
    UNIQUE(relationship_id, chunk_id)
);

CREATE INDEX idx_relationship_evidence_rel_id ON relationship_evidence(relationship_id);
CREATE INDEX idx_relationship_evidence_chunk_id ON relationship_evidence(chunk_id);
CREATE INDEX idx_relationship_evidence_confidence ON relationship_evidence(confidence);

-- Helper view: entities with their evidence chunks
CREATE OR REPLACE VIEW entities_with_evidence AS
SELECT 
    e.id AS entity_id,
    e.entity_type,
    e.canonical_name,
    e.normalized_name,
    e.service_name,
    e.language,
    COUNT(DISTINCT ee.chunk_id) AS evidence_count,
    ARRAY_AGG(DISTINCT ee.chunk_id) AS chunk_ids,
    AVG(ee.confidence) AS avg_confidence
FROM entities e
LEFT JOIN entity_evidence ee ON e.id = ee.entity_id
GROUP BY e.id, e.entity_type, e.canonical_name, e.normalized_name, e.service_name, e.language;

-- Helper view: relationships with their evidence chunks
CREATE OR REPLACE VIEW relationships_with_evidence AS
SELECT 
    er.id AS relationship_id,
    er.source_entity_id,
    er.target_entity_id,
    er.relation_type,
    se.canonical_name AS source_name,
    te.canonical_name AS target_name,
    COUNT(DISTINCT re.chunk_id) AS evidence_count,
    ARRAY_AGG(DISTINCT re.chunk_id) AS chunk_ids,
    AVG(re.confidence) AS avg_confidence
FROM entity_relations er
JOIN entities se ON er.source_entity_id = se.id
JOIN entities te ON er.target_entity_id = te.id
LEFT JOIN relationship_evidence re ON er.id = re.relationship_id
GROUP BY er.id, er.source_entity_id, er.target_entity_id, er.relation_type, se.canonical_name, te.canonical_name;

-- Comments
COMMENT ON TABLE entity_evidence IS 'Links graph entities to the chunks that justify their existence. Every entity should have evidence.';
COMMENT ON TABLE relationship_evidence IS 'Links graph relationships to the chunks that justify them. Every relationship should have evidence.';
COMMENT ON COLUMN entity_evidence.chunk_id IS 'References chunks.chunk_id - the source of truth for chunk text.';
COMMENT ON COLUMN relationship_evidence.chunk_id IS 'References chunks.chunk_id - the source of truth for chunk text.';
