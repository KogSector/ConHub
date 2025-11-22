# Knowledge Graph Architecture (Integrated into Data Service)

## Overview

The Knowledge Graph functionality implements GraphRAG (Graph Retrieval-Augmented Generation) capabilities for ConHub. It builds and maintains a unified knowledge graph across all connected data sources. This functionality is integrated into the `data` microservice rather than being a standalone service. including repositories, documents, conversations, and more.

## Core Principles

### 1. Multi-Source Integration
- **GitHub**: Repositories, commits, files, issues, PRs
- **Slack**: Messages, threads, channels, users
- **Notion**: Pages, databases, blocks
- **Documents**: PDFs, Markdown, Google Docs
- **Local Files**: Any local document source

### 2. Entity Resolution
Automatically identifies when entities from different sources refer to the same real-world thing.

Example:
```
GitHub: "alice_dev" (username)
Slack: "Alice Chen" (display name)
Notion: "Alice C." (author)
→ Resolves to Canonical Person: "Alice Chen"
```

### 3. Knowledge Fusion
Merges entities and relationships from multiple sources into a unified graph.

## Architecture Components

### 1. Storage Layer

#### Neo4j (Graph Database)
- **Purpose**: Store the actual graph structure (entities and relationships)
- **Port**: 7687 (Bolt), 7474 (HTTP)
- **Features**:
  - Fast graph traversal
  - Pattern matching with Cypher
  - APOC procedures for advanced operations
  - Graph Data Science algorithms

#### PostgreSQL (Metadata Store)
- **Purpose**: Track entity metadata, sync jobs, statistics
- **Tables**:
  - `graph_entities`: Entity metadata
  - `canonical_entities`: Resolved entities
  - `graph_relationships`: Relationship metadata
  - `entity_resolutions`: Resolution tracking
  - `graph_sync_jobs`: Incremental update jobs

#### Qdrant (Vector Store)
- **Purpose**: Store embeddings for semantic search
- **Integration**: Each entity has an embedding for semantic similarity

### 2. Service Layer

#### Knowledge Graph Service
- **Port**: 3017
- **Language**: Rust
- **Responsibilities**:
  - Entity extraction from multiple sources
  - Entity resolution and deduplication
  - Knowledge fusion
  - Graph queries (REST and GraphQL)
  - Incremental updates via webhooks

### 3. Processing Pipeline

```
┌─────────────────────────────────────────────────────┐
│           Data Sources (Multi-Source)                │
├──────────┬──────────┬──────────┬────────────────────┤
│ GitHub   │ Slack    │ Notion   │ Documents/PDFs    │
└──────────┴──────────┴──────────┴────────────────────┘
           │           │           │
           ▼           ▼           ▼
┌─────────────────────────────────────────────────────┐
│         Entity Extraction Layer                      │
│  (Source-specific extractors + NER + Embeddings)    │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│        Entity Resolution & Fusion                    │
│  (Match entities across sources, deduplicate)       │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────┐
│         Unified Knowledge Graph                       │
│  ┌────────────────────────────────────────────┐      │
│  │ Neo4j: Entities + Relationships (graph)    │      │
│  └────────────────────────────────────────────┘      │
│  ┌────────────────────────────────────────────┐      │
│  │ Qdrant: Embeddings (vector + semantic)     │      │
│  └────────────────────────────────────────────┘      │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│       Query Interface / API                          │
│  (REST, GraphQL, Cypher queries)                    │
└─────────────────────────────────────────────────────┘
```

## Entity Types

### Universal Entity Types
```rust
- Person: Developers, authors, collaborators
- CodeEntity: Files, functions, classes, modules
- Document: PDFs, Notion pages, markdown files
- Conversation: Slack/Discord threads, chat messages
- Project: Repositories, workspaces
- Concept: Technical topics, features, bugs
- Timestamp: Events, commits, messages
- Organization: Companies, teams, departments
```

## Relationship Types

### Universal Relationships
```rust
// Authorship and ownership
- AUTHORED_BY: (Document/Code → Person)
- OWNED_BY: (Project → Person/Organization)

// Discussion and communication
- DISCUSSED_IN: (Concept/Code → Conversation)
- MENTIONED_IN: (Person/Code → Message)

// Documentation
- DOCUMENTED_IN: (Code/Feature → Document)
- DESCRIBES: (Document → Code/Concept)

// Code relationships
- IMPLEMENTS: (Code → Concept)
- DEPENDS_ON: (Code → Code)
- IMPORTS: (File → File)
- CALLS: (Function → Function)

// Project relationships
- BELONGS_TO: (Code/Document → Project)
- CONTAINS: (Project → File)

// Semantic relationships
- RELATED_TO: (Any → Any, by semantic similarity)

// Entity resolution
- RESOLVES_TO: (Source Entity → Canonical Entity)
```

## Example Graph Structure

```cypher
// Alice worked on authentication across multiple sources

// GitHub
(alice_github:Person {source: "GitHub", username: "alice_dev"})
  -[:RESOLVES_TO]-> (alice_canonical:CanonicalPerson {name: "Alice Chen"})
(alice_github) -[:AUTHORED]-> (login_file:File {path: "src/auth/login.py"})
(login_file) -[:IMPLEMENTS]-> (oauth_concept:Concept {name: "OAuth2"})

// Slack
(alice_slack:Person {source: "Slack", name: "Alice Chen"})
  -[:RESOLVES_TO]-> (alice_canonical)
(alice_slack) -[:SENT]-> (slack_msg:Message {text: "We need OAuth2"})
(slack_msg) -[:MENTIONS]-> (oauth_concept)

// Notion
(alice_notion:Person {source: "Notion", name: "Alice C."})
  -[:RESOLVES_TO]-> (alice_canonical)
(alice_notion) -[:AUTHORED]-> (auth_doc:Document {title: "Auth Spec v2"})
(auth_doc) -[:DOCUMENTS]-> (login_file)

// Unified view: Alice's work on OAuth2 across all sources
MATCH (alice_canonical)-[:RESOLVES_TO*]-(sources)-[r]->(artifacts)
WHERE artifacts.topic CONTAINS "OAuth" OR artifacts.name CONTAINS "auth"
RETURN alice_canonical, sources, artifacts, r
ORDER BY artifacts.timestamp
```

## API Endpoints

### REST API

#### Query Endpoints
```
POST /api/graph/query
POST /api/graph/cross-source-query
GET  /api/graph/entity/:id
GET  /api/graph/statistics
GET  /api/graph/paths/:from/:to
```

#### Ingestion Endpoints
```
POST /api/graph/ingest
POST /api/graph/update
POST /api/graph/batch-ingest
```

#### Webhook Endpoints
```
POST /api/graph/webhook/github
POST /api/graph/webhook/slack
POST /api/graph/webhook/notion
```

### GraphQL API

Access GraphQL Playground at: `http://localhost:3017/api/graph/graphql/playground`

Example queries:
```graphql
# Find entity by ID
query {
  entity(id: "uuid-here") {
    id
    name
    entityType
    source
    content
  }
}

# Find cross-source activities for a person
query {
  crossSourceActivities(canonicalName: "Alice Chen") {
    id
    name
    entityType
    source
    createdAt
  }
}

# Get graph statistics
query {
  statistics {
    totalEntities
    totalRelationships
    canonicalEntities
    unresolvedEntities
  }
}
```

## Cross-Source Query Example

**Question**: "Who worked on authentication and what did they discuss?"

**Query**:
```json
{
  "topic": "authentication",
  "sources": ["GitHub", "Slack", "Notion"],
  "entity_types": ["Person", "CodeEntity", "Document"],
  "date_range": {
    "start": "2024-01-01T00:00:00Z",
    "end": "2024-12-31T23:59:59Z"
  }
}
```

**Response**:
```json
{
  "canonical_entities": [
    {
      "id": "uuid-1",
      "canonical_name": "Alice Chen",
      "entity_type": "Person",
      "source_entities": ["github:alice_dev", "slack:U12345", "notion:alice_c"]
    }
  ],
  "activities": [
    {
      "entity_name": "Slack message",
      "source": "Slack",
      "timestamp": "2024-03-15T10:00:00Z",
      "content": "We need OAuth2 implementation"
    },
    {
      "entity_name": "Auth Spec v2",
      "source": "Notion",
      "timestamp": "2024-03-18T14:30:00Z",
      "content": "OAuth2 Architecture Design"
    },
    {
      "entity_name": "login.py",
      "source": "GitHub",
      "timestamp": "2024-03-20T09:15:00Z",
      "content": "Implement OAuth2 authentication"
    }
  ],
  "timeline": [...]
}
```

## Entity Resolution

### Matching Strategies

1. **Exact Match** (Confidence: 0.9)
   - Email addresses
   - Unique identifiers

2. **Fuzzy Match** (Confidence: 0.5-0.8)
   - Name similarity (Levenshtein distance)
   - Username patterns

3. **Attribute Overlap** (Confidence: 0.3-0.7)
   - Multiple shared properties
   - Email domain + similar name

4. **Graph-Based** (Confidence: 0.4-0.6)
   - Shared connections
   - Same repositories/channels

### Resolution Process

```rust
// 1. Extract features
let person1_features = {
    "github_username": "alice_dev",
    "email": "alice@company.com",
    "name": "Alice Chen"
};

let person2_features = {
    "slack_user_id": "U12345",
    "email": "alice@company.com",
    "name": "Alice Chen"
};

// 2. Compute matching score
let score = compute_matching_score(person1, person2);
// Email match (0.9) + Name match (1.0) → High confidence

// 3. Create canonical entity
if score > 0.7 {
    create_canonical_entity(person1, person2);
    create_relationship(person1, RESOLVES_TO, canonical);
    create_relationship(person2, RESOLVES_TO, canonical);
}
```

## Incremental Updates

### Webhook Integration

When new data arrives (e.g., GitHub push), the system:

1. **Extract** entities and relationships
2. **Check** if entities exist
3. **Update** or create entities
4. **Re-run** entity resolution
5. **Update** graph statistics

Example:
```rust
// GitHub webhook: new commit pushed
handle_webhook_event(
    source: EntitySource::GitHub,
    event_type: "push",
    payload: {
        "commits": [{
            "sha": "abc123",
            "message": "Implement OAuth2",
            "author": {
                "name": "Alice",
                "email": "alice@company.com"
            }
        }]
    }
);

// Process:
// 1. Extract commit entity
// 2. Extract person entity (author)
// 3. Create AUTHORED_BY relationship
// 4. Run entity resolution on author
// 5. Update embeddings
```

## Performance Optimization

### Indexes
- Entity type and source indexes
- Full-text search on names and content
- JSONB indexes on properties
- Timestamp indexes for temporal queries

### Caching
- Graph statistics cached in PostgreSQL
- Frequently accessed entities cached in Redis
- Embedding cache in Qdrant

### Batch Processing
- Batch insert for large ingestions
- Parallel entity extraction
- Async relationship creation

## Security

### Access Control
- JWT authentication for API access
- Role-based permissions for sensitive data
- Audit logging for all operations

### Data Privacy
- PII detection and masking
- Encryption at rest (Neo4j, PostgreSQL)
- Encryption in transit (TLS)

## Monitoring

### Metrics
- Entity count by type and source
- Relationship count by type
- Resolution accuracy
- Query performance
- Sync job success rate

### Alerts
- Failed sync jobs
- Resolution conflicts
- Graph size thresholds
- Query timeout warnings

## Deployment

### Local Development
```bash
# Start all services including Neo4j
docker-compose up -d

# Access Neo4j browser
open http://localhost:7474

# Access GraphQL playground
open http://localhost:3017/api/graph/graphql/playground
```

### Production Considerations
- Neo4j cluster for high availability
- Read replicas for query performance
- Regular backups (Neo4j + PostgreSQL)
- Monitoring and alerting
- Rate limiting on API endpoints

## Future Enhancements

1. **Advanced Entity Extraction**
   - NER with fine-tuned models
   - Code AST analysis
   - Image and diagram understanding

2. **Graph Algorithms**
   - Community detection
   - Centrality measures
   - Path finding optimizations

3. **ML-Powered Resolution**
   - Learning from user feedback
   - Confidence score calibration
   - Automated conflict resolution

4. **Real-Time Updates**
   - WebSocket subscriptions for graph changes
   - Live query results
   - Collaborative editing

## References

- [Neo4j Documentation](https://neo4j.com/docs/)
- [Graph Data Science](https://neo4j.com/docs/graph-data-science/)
- [Entity Resolution Best Practices](https://neo4j.com/docs/graph-data-science/current/algorithms/node-similarity/)
- [GraphRAG Patterns](https://arxiv.org/abs/2404.16130)
