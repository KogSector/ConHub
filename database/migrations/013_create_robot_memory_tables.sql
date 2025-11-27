-- Migration: Create robot memory and sensor stream tables for Apache integration
-- This supports the robot "brain" architecture with episodic and semantic memory

-- ============================================================================
-- ROBOTS TABLE
-- ============================================================================
-- Represents a registered robot or autonomous agent that sends sensor data
CREATE TABLE IF NOT EXISTS robots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    
    -- Robot identification
    name VARCHAR(255) NOT NULL,
    robot_type VARCHAR(100) NOT NULL DEFAULT 'generic', -- generic, mobile, arm, drone, humanoid
    description TEXT,
    
    -- Capabilities (stored as JSONB array)
    capabilities JSONB DEFAULT '[]', -- ["navigation", "manipulation", "vision", "speech"]
    
    -- Connection status
    status VARCHAR(50) NOT NULL DEFAULT 'registered', -- registered, connected, disconnected, error
    last_heartbeat TIMESTAMPTZ,
    
    -- Kafka connection info (assigned on registration)
    kafka_topics JSONB DEFAULT '{}', -- {"raw_events": "robot.uuid.raw_events", ...}
    kafka_credentials JSONB DEFAULT '{}', -- encrypted credentials
    
    -- HTTP ingestion endpoints (alternative to Kafka)
    http_endpoints JSONB DEFAULT '{}', -- {"events_url": "/api/ingestion/robots/uuid/events", ...}
    
    -- Configuration
    config JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_robots_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_robots_tenant_id ON robots(tenant_id);
CREATE INDEX idx_robots_user_id ON robots(user_id);
CREATE INDEX idx_robots_status ON robots(status);
CREATE INDEX idx_robots_robot_type ON robots(robot_type);
CREATE UNIQUE INDEX idx_robots_tenant_name ON robots(tenant_id, name);

-- ============================================================================
-- ROBOT STREAMS TABLE
-- ============================================================================
-- Represents a sensor stream from a robot (camera, lidar, IMU, etc.)
CREATE TABLE IF NOT EXISTS robot_streams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    robot_id UUID NOT NULL,
    
    -- Stream identification
    name VARCHAR(255) NOT NULL,
    stream_type VARCHAR(100) NOT NULL, -- camera_rgb, camera_depth, lidar_2d, lidar_3d, imu, encoder, audio, custom
    description TEXT,
    
    -- Schema for stream events (for Flink/validation)
    event_schema JSONB DEFAULT '{}', -- {"fields": [{"name": "timestamp", "type": "timestamp"}, ...]}
    
    -- Kafka topic for this stream
    kafka_topic VARCHAR(500),
    
    -- Stream status
    status VARCHAR(50) NOT NULL DEFAULT 'active', -- active, paused, error
    
    -- Sampling/processing config
    sample_rate_hz FLOAT,
    buffer_size INTEGER,
    config JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_robot_streams_robot FOREIGN KEY (robot_id) REFERENCES robots(id) ON DELETE CASCADE,
    UNIQUE(robot_id, name)
);

CREATE INDEX idx_robot_streams_robot_id ON robot_streams(robot_id);
CREATE INDEX idx_robot_streams_stream_type ON robot_streams(stream_type);
CREATE INDEX idx_robot_streams_status ON robot_streams(status);

-- ============================================================================
-- ROBOT EPISODES TABLE
-- ============================================================================
-- Represents a bounded sequence of observations + actions + outcomes
-- This is the core of episodic memory
CREATE TABLE IF NOT EXISTS robot_episodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    robot_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    
    -- Episode identification
    episode_number BIGINT NOT NULL,
    episode_type VARCHAR(100) DEFAULT 'general', -- navigation, manipulation, interaction, observation
    
    -- Time bounds
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_ms BIGINT,
    
    -- Location context
    location_id VARCHAR(255),
    location_name VARCHAR(255),
    location_coordinates JSONB, -- {"x": 0.0, "y": 0.0, "z": 0.0, "frame": "map"}
    
    -- Natural language summary (for LLM context)
    summary TEXT,
    detailed_description TEXT,
    
    -- Structured data
    observations JSONB DEFAULT '[]', -- array of observation events
    actions JSONB DEFAULT '[]', -- array of actions taken
    outcomes JSONB DEFAULT '{}', -- result of the episode
    
    -- Entities involved
    objects_seen JSONB DEFAULT '[]', -- ["box", "person", "door"]
    people_involved JSONB DEFAULT '[]', -- ["person_id_1", ...]
    tasks_related JSONB DEFAULT '[]', -- ["task_id_1", ...]
    
    -- Quality and confidence
    confidence_score FLOAT,
    quality_score FLOAT,
    
    -- Indexing status
    indexed_at TIMESTAMPTZ,
    embedding_id VARCHAR(255), -- reference to vector store
    graph_node_id VARCHAR(255), -- reference to graph store
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_robot_episodes_robot FOREIGN KEY (robot_id) REFERENCES robots(id) ON DELETE CASCADE,
    UNIQUE(robot_id, episode_number)
);

CREATE INDEX idx_robot_episodes_robot_id ON robot_episodes(robot_id);
CREATE INDEX idx_robot_episodes_tenant_id ON robot_episodes(tenant_id);
CREATE INDEX idx_robot_episodes_started_at ON robot_episodes(started_at);
CREATE INDEX idx_robot_episodes_episode_type ON robot_episodes(episode_type);
CREATE INDEX idx_robot_episodes_location_id ON robot_episodes(location_id);
CREATE INDEX idx_robot_episodes_indexed_at ON robot_episodes(indexed_at);

-- ============================================================================
-- ROBOT SEMANTIC FACTS TABLE
-- ============================================================================
-- Stable facts derived from many episodes (long-term semantic memory)
CREATE TABLE IF NOT EXISTS robot_semantic_facts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    robot_id UUID, -- NULL if global fact
    tenant_id UUID NOT NULL,
    
    -- Fact identification
    fact_type VARCHAR(100) NOT NULL, -- object_location, person_preference, route_efficiency, affordance
    subject VARCHAR(255) NOT NULL, -- what the fact is about
    predicate VARCHAR(255) NOT NULL, -- relationship or property
    object_value TEXT, -- the value or target
    
    -- Natural language representation
    fact_text TEXT NOT NULL, -- "The red box is usually on shelf A3"
    
    -- Confidence and provenance
    confidence_score FLOAT NOT NULL DEFAULT 0.5,
    evidence_count INTEGER DEFAULT 1, -- how many episodes support this
    first_observed_at TIMESTAMPTZ NOT NULL,
    last_observed_at TIMESTAMPTZ NOT NULL,
    
    -- Source episodes
    source_episode_ids JSONB DEFAULT '[]',
    
    -- Indexing
    indexed_at TIMESTAMPTZ,
    embedding_id VARCHAR(255),
    graph_node_id VARCHAR(255),
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_robot_semantic_facts_robot FOREIGN KEY (robot_id) REFERENCES robots(id) ON DELETE CASCADE
);

CREATE INDEX idx_robot_semantic_facts_robot_id ON robot_semantic_facts(robot_id);
CREATE INDEX idx_robot_semantic_facts_tenant_id ON robot_semantic_facts(tenant_id);
CREATE INDEX idx_robot_semantic_facts_fact_type ON robot_semantic_facts(fact_type);
CREATE INDEX idx_robot_semantic_facts_subject ON robot_semantic_facts(subject);
CREATE INDEX idx_robot_semantic_facts_confidence ON robot_semantic_facts(confidence_score);

-- ============================================================================
-- ROBOT EVENTS LOG TABLE
-- ============================================================================
-- Passive context storage - everything logged, available on demand
CREATE TABLE IF NOT EXISTS robot_events_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    robot_id UUID NOT NULL,
    stream_id UUID,
    tenant_id UUID NOT NULL,
    
    -- Event identification
    event_type VARCHAR(100) NOT NULL,
    event_source VARCHAR(255) NOT NULL, -- sensor name or system
    
    -- Timing
    event_timestamp TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Payload
    payload JSONB NOT NULL,
    
    -- Processing status
    processed BOOLEAN DEFAULT FALSE,
    episode_id UUID, -- if assigned to an episode
    
    -- Partitioning hint (for time-series queries)
    partition_key VARCHAR(50), -- e.g., "2025-11-28"
    
    CONSTRAINT fk_robot_events_log_robot FOREIGN KEY (robot_id) REFERENCES robots(id) ON DELETE CASCADE,
    CONSTRAINT fk_robot_events_log_stream FOREIGN KEY (stream_id) REFERENCES robot_streams(id) ON DELETE SET NULL
);

-- Partition by time for efficient queries (if using TimescaleDB or similar)
CREATE INDEX idx_robot_events_log_robot_timestamp ON robot_events_log(robot_id, event_timestamp DESC);
CREATE INDEX idx_robot_events_log_tenant_timestamp ON robot_events_log(tenant_id, event_timestamp DESC);
CREATE INDEX idx_robot_events_log_event_type ON robot_events_log(event_type);
CREATE INDEX idx_robot_events_log_processed ON robot_events_log(processed) WHERE NOT processed;

-- ============================================================================
-- ROBOT MEMORY SEARCH CACHE TABLE
-- ============================================================================
-- Cache for frequently accessed memory queries
CREATE TABLE IF NOT EXISTS robot_memory_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    robot_id UUID NOT NULL,
    
    -- Query fingerprint
    query_hash VARCHAR(64) NOT NULL,
    query_text TEXT NOT NULL,
    filters JSONB DEFAULT '{}',
    
    -- Cached result
    result JSONB NOT NULL,
    result_count INTEGER,
    
    -- Cache management
    hits INTEGER DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_robot_memory_cache_robot FOREIGN KEY (robot_id) REFERENCES robots(id) ON DELETE CASCADE,
    UNIQUE(robot_id, query_hash)
);

CREATE INDEX idx_robot_memory_cache_expires ON robot_memory_cache(expires_at);

-- ============================================================================
-- TRIGGERS
-- ============================================================================
CREATE OR REPLACE FUNCTION update_robots_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER robots_updated_at
    BEFORE UPDATE ON robots
    FOR EACH ROW
    EXECUTE FUNCTION update_robots_updated_at();

CREATE TRIGGER robot_streams_updated_at
    BEFORE UPDATE ON robot_streams
    FOR EACH ROW
    EXECUTE FUNCTION update_robots_updated_at();

CREATE TRIGGER robot_episodes_updated_at
    BEFORE UPDATE ON robot_episodes
    FOR EACH ROW
    EXECUTE FUNCTION update_robots_updated_at();

CREATE TRIGGER robot_semantic_facts_updated_at
    BEFORE UPDATE ON robot_semantic_facts
    FOR EACH ROW
    EXECUTE FUNCTION update_robots_updated_at();

-- ============================================================================
-- FEATURE TOGGLES FOR ROBOT MEMORY
-- ============================================================================
INSERT INTO feature_toggles (feature_name, is_enabled, description, config) VALUES
('RobotConnector', true, 'Enable robot and sensor data ingestion', '{"kafka_enabled": true, "http_enabled": true}'),
('RobotMemory', true, 'Enable robot episodic and semantic memory', '{"episodic": true, "semantic": true}'),
('ApacheKafka', true, 'Enable Apache Kafka integration for streaming', '{"bootstrap_servers": "localhost:9092"}'),
('ApacheFlink', false, 'Enable Apache Flink for stream processing', '{"job_manager": "localhost:8081"}'),
('ApacheSpark', false, 'Enable Apache Spark for batch analytics', '{}')
ON CONFLICT (feature_name) DO NOTHING;

-- ============================================================================
-- COMMENTS
-- ============================================================================
COMMENT ON TABLE robots IS 'Registered robots and autonomous agents that send sensor data to ConHub';
COMMENT ON TABLE robot_streams IS 'Sensor streams from robots (cameras, lidar, IMU, etc.)';
COMMENT ON TABLE robot_episodes IS 'Episodic memory: bounded sequences of observations, actions, and outcomes';
COMMENT ON TABLE robot_semantic_facts IS 'Semantic memory: stable facts derived from many episodes';
COMMENT ON TABLE robot_events_log IS 'Passive context storage: all events logged for on-demand retrieval';
COMMENT ON TABLE robot_memory_cache IS 'Cache for frequently accessed robot memory queries';
