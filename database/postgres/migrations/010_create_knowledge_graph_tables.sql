-- Knowledge Graph Metadata Tables
-- This migration creates tables to track graph entities and relationships in PostgreSQL
-- while the actual graph structure is stored in Neo4j

-- Entity metadata table (tracks entities in the graph)
CREATE TABLE IF NOT EXISTS graph_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL,
    source VARCHAR(50) NOT NULL,
    source_id VARCHAR(500) NOT NULL,
    canonical_id UUID,
    name VARCHAR(1000) NOT NULL,
    content TEXT,
    properties JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    embedding_vector_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(source, source_id)
);

-- Canonical entities table (resolved entities across sources)
CREATE TABLE IF NOT EXISTS canonical_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL,
    canonical_name VARCHAR(1000) NOT NULL,
    properties JSONB DEFAULT '{}',
    confidence_score REAL NOT NULL DEFAULT 1.0,
    source_entity_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Relationship metadata table
CREATE TABLE IF NOT EXISTS graph_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    relationship_type VARCHAR(100) NOT NULL,
    from_entity_id UUID NOT NULL,
    to_entity_id UUID NOT NULL,
    source VARCHAR(50) NOT NULL,
    properties JSONB DEFAULT '{}',
    confidence_score REAL NOT NULL DEFAULT 1.0,
    timestamp TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    FOREIGN KEY (from_entity_id) REFERENCES graph_entities(id) ON DELETE CASCADE,
    FOREIGN KEY (to_entity_id) REFERENCES graph_entities(id) ON DELETE CASCADE
);

-- Entity resolution tracking
CREATE TABLE IF NOT EXISTS entity_resolutions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity1_id UUID NOT NULL,
    entity2_id UUID NOT NULL,
    canonical_id UUID,
    confidence_score REAL NOT NULL,
    matching_features JSONB DEFAULT '[]',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    reviewed_by UUID,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (entity1_id) REFERENCES graph_entities(id) ON DELETE CASCADE,
    FOREIGN KEY (entity2_id) REFERENCES graph_entities(id) ON DELETE CASCADE,
    FOREIGN KEY (canonical_id) REFERENCES canonical_entities(id) ON DELETE SET NULL,
    UNIQUE(entity1_id, entity2_id)
);

-- Graph sync jobs (for incremental updates)
CREATE TABLE IF NOT EXISTS graph_sync_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source VARCHAR(50) NOT NULL,
    job_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    entities_processed INTEGER NOT NULL DEFAULT 0,
    relationships_processed INTEGER NOT NULL DEFAULT 0,
    errors JSONB DEFAULT '[]',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Graph statistics (cached for performance)
CREATE TABLE IF NOT EXISTS graph_statistics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    total_entities INTEGER NOT NULL DEFAULT 0,
    total_relationships INTEGER NOT NULL DEFAULT 0,
    total_canonical_entities INTEGER NOT NULL DEFAULT 0,
    entities_by_type JSONB DEFAULT '{}',
    entities_by_source JSONB DEFAULT '{}',
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert initial statistics row
INSERT INTO graph_statistics (total_entities, total_relationships, total_canonical_entities)
VALUES (0, 0, 0)
ON CONFLICT DO NOTHING;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_graph_entities_type ON graph_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_graph_entities_source ON graph_entities(source);
CREATE INDEX IF NOT EXISTS idx_graph_entities_canonical ON graph_entities(canonical_id);
CREATE INDEX IF NOT EXISTS idx_graph_entities_name ON graph_entities USING gin(to_tsvector('english', name));
CREATE INDEX IF NOT EXISTS idx_graph_entities_content ON graph_entities USING gin(to_tsvector('english', content));
CREATE INDEX IF NOT EXISTS idx_graph_entities_properties ON graph_entities USING gin(properties);
CREATE INDEX IF NOT EXISTS idx_graph_entities_created ON graph_entities(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_canonical_entities_type ON canonical_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_canonical_entities_name ON canonical_entities(canonical_name);

CREATE INDEX IF NOT EXISTS idx_graph_relationships_type ON graph_relationships(relationship_type);
CREATE INDEX IF NOT EXISTS idx_graph_relationships_from ON graph_relationships(from_entity_id);
CREATE INDEX IF NOT EXISTS idx_graph_relationships_to ON graph_relationships(to_entity_id);
CREATE INDEX IF NOT EXISTS idx_graph_relationships_source ON graph_relationships(source);
CREATE INDEX IF NOT EXISTS idx_graph_relationships_timestamp ON graph_relationships(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_entity_resolutions_entity1 ON entity_resolutions(entity1_id);
CREATE INDEX IF NOT EXISTS idx_entity_resolutions_entity2 ON entity_resolutions(entity2_id);
CREATE INDEX IF NOT EXISTS idx_entity_resolutions_canonical ON entity_resolutions(canonical_id);
CREATE INDEX IF NOT EXISTS idx_entity_resolutions_status ON entity_resolutions(status);

CREATE INDEX IF NOT EXISTS idx_graph_sync_jobs_source ON graph_sync_jobs(source);
CREATE INDEX IF NOT EXISTS idx_graph_sync_jobs_status ON graph_sync_jobs(status);
CREATE INDEX IF NOT EXISTS idx_graph_sync_jobs_created ON graph_sync_jobs(created_at DESC);

-- Triggers for updated_at
CREATE OR REPLACE FUNCTION update_graph_entity_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_graph_entity_timestamp
    BEFORE UPDATE ON graph_entities
    FOR EACH ROW
    EXECUTE FUNCTION update_graph_entity_timestamp();

CREATE TRIGGER trigger_update_canonical_entity_timestamp
    BEFORE UPDATE ON canonical_entities
    FOR EACH ROW
    EXECUTE FUNCTION update_graph_entity_timestamp();

-- Function to update graph statistics
CREATE OR REPLACE FUNCTION update_graph_statistics()
RETURNS void AS $$
BEGIN
    UPDATE graph_statistics
    SET
        total_entities = (SELECT COUNT(*) FROM graph_entities),
        total_relationships = (SELECT COUNT(*) FROM graph_relationships),
        total_canonical_entities = (SELECT COUNT(*) FROM canonical_entities),
        entities_by_type = (
            SELECT jsonb_object_agg(entity_type, count)
            FROM (
                SELECT entity_type, COUNT(*) as count
                FROM graph_entities
                GROUP BY entity_type
            ) as type_counts
        ),
        entities_by_source = (
            SELECT jsonb_object_agg(source, count)
            FROM (
                SELECT source, COUNT(*) as count
                FROM graph_entities
                GROUP BY source
            ) as source_counts
        ),
        last_updated_at = NOW()
    WHERE id = (SELECT id FROM graph_statistics LIMIT 1);
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE graph_entities IS 'Metadata for entities in the knowledge graph (actual graph stored in Neo4j)';
COMMENT ON TABLE canonical_entities IS 'Resolved entities that represent the same real-world thing across sources';
COMMENT ON TABLE graph_relationships IS 'Metadata for relationships between entities';
COMMENT ON TABLE entity_resolutions IS 'Tracks entity resolution decisions and confidence scores';
COMMENT ON TABLE graph_sync_jobs IS 'Tracks incremental sync jobs from various sources';
COMMENT ON TABLE graph_statistics IS 'Cached statistics about the knowledge graph';

COMMENT ON COLUMN graph_entities.source IS 'Source system (GitHub, Slack, Notion, etc.)';
COMMENT ON COLUMN graph_entities.source_id IS 'ID from the source system';
COMMENT ON COLUMN graph_entities.canonical_id IS 'Links to canonical entity if resolved';
COMMENT ON COLUMN graph_entities.embedding_vector_id IS 'Reference to embedding in Qdrant';
COMMENT ON COLUMN canonical_entities.source_entity_count IS 'Number of source entities that resolve to this canonical entity';
COMMENT ON COLUMN entity_resolutions.status IS 'Status: pending, approved, rejected';
