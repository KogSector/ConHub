-- Knowledge Graph Tables for GraphRAG Implementation
-- This migration creates the unified knowledge layer with entity resolution

-- Entities table: Stores all entities from all sources
CREATE TABLE IF NOT EXISTS entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL,
    source VARCHAR(50) NOT NULL,
    source_id VARCHAR(255) NOT NULL,
    name TEXT NOT NULL,
    canonical_id UUID,
    properties JSONB DEFAULT '{}',
    embedding VECTOR(768), -- For semantic search
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(source, source_id)
);

-- Canonical entities: Merged view of entities from multiple sources
CREATE TABLE IF NOT EXISTS canonical_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL,
    canonical_name TEXT NOT NULL,
    merged_properties JSONB DEFAULT '{}',
    source_entities JSONB DEFAULT '[]',
    confidence_score REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Relationships table: Connections between entities
CREATE TABLE IF NOT EXISTS relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    to_entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL,
    properties JSONB DEFAULT '{}',
    weight REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Entity resolution matches: Track which entities match
CREATE TABLE IF NOT EXISTS entity_resolution_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity1_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    entity2_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    confidence_score REAL NOT NULL,
    matching_features JSONB DEFAULT '[]',
    matching_strategy VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(entity1_id, entity2_id)
);

-- Semantic relationships: Vector-based similarity
CREATE TABLE IF NOT EXISTS semantic_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    to_entity_id UUID NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    similarity_score REAL NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(from_entity_id, to_entity_id)
);

-- Entity extraction metadata
CREATE TABLE IF NOT EXISTS entity_extraction_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source VARCHAR(50) NOT NULL,
    source_reference VARCHAR(255),
    status VARCHAR(50) NOT NULL,
    entities_extracted INT DEFAULT 0,
    relationships_extracted INT DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    error_message TEXT
);

-- Indexes for performance
CREATE INDEX idx_entities_type ON entities(entity_type);
CREATE INDEX idx_entities_source ON entities(source);
CREATE INDEX idx_entities_canonical ON entities(canonical_id);
CREATE INDEX idx_entities_name ON entities USING gin(to_tsvector('english', name));
CREATE INDEX idx_entities_properties ON entities USING gin(properties);

CREATE INDEX idx_canonical_entities_type ON canonical_entities(entity_type);
CREATE INDEX idx_canonical_entities_name ON canonical_entities USING gin(to_tsvector('english', canonical_name));

CREATE INDEX idx_relationships_from ON relationships(from_entity_id);
CREATE INDEX idx_relationships_to ON relationships(to_entity_id);
CREATE INDEX idx_relationships_type ON relationships(relationship_type);
CREATE INDEX idx_relationships_from_to ON relationships(from_entity_id, to_entity_id);

CREATE INDEX idx_resolution_matches_entity1 ON entity_resolution_matches(entity1_id);
CREATE INDEX idx_resolution_matches_entity2 ON entity_resolution_matches(entity2_id);
CREATE INDEX idx_resolution_matches_score ON entity_resolution_matches(confidence_score);

CREATE INDEX idx_semantic_relationships_from ON semantic_relationships(from_entity_id);
CREATE INDEX idx_semantic_relationships_to ON semantic_relationships(to_entity_id);
CREATE INDEX idx_semantic_relationships_score ON semantic_relationships(similarity_score);

-- Add foreign key for canonical entities
ALTER TABLE entities ADD CONSTRAINT fk_entities_canonical 
    FOREIGN KEY (canonical_id) REFERENCES canonical_entities(id) ON DELETE SET NULL;

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for auto-updating timestamps
CREATE TRIGGER update_entities_updated_at BEFORE UPDATE ON entities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_canonical_entities_updated_at BEFORE UPDATE ON canonical_entities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_relationships_updated_at BEFORE UPDATE ON relationships
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
