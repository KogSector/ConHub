# ConHub RAG System Architecture

## Overview

ConHub implements a comprehensive three-mode RAG (Retrieval-Augmented Generation) system designed to handle massive multi-source knowledge bases including code repositories, documents, chats, tickets, and more.

## Three RAG Modes

### Mode A: Vector RAG (Fast, Generic)
**Use Cases:**
- "Explain this file/function/class"
- "Where is X implemented?"
- "Summarize Slack conversation/doc/PR"

**Architecture:**
- Embedding models (Qwen for code, OpenAI for general text)
- Vector DB (Qdrant) with per-tenant collections
- Metadata filtering (repo, connector, time, authors, tags)

**Flow:**
```
Query → Embedding → Vector Search → Top-K Chunks → Answer
```

### Mode B: Graph + Vector RAG (Structure-Aware)
**Use Cases:**
- Ownership & dependency questions ("Who owns this service?")
- Multi-hop questions ("How did this bug evolve from JIRA → PRs → releases?")
- Context clustered around repos/people/tasks

**Architecture:**
- Knowledge Graph (Neo4j) with entities and relationships
- Vector DB for semantic search
- Hybrid retrieval combining graph structure + vector similarity

**Entities:**
- Person, Repository, File, CodeEntity, Commit, PR, Issue
- Document, Conversation, Message, Project, Concept

**Relationships:**
- BELONGS_TO (File → Repository, Message → Conversation)
- AUTHORED_BY (File/Commit/Document → Person)
- MENTIONS (Message/Document → Concept/Person/Issue)
- REFERS_TO (PR → Issue, Issue → Commit)
- DOCUMENTS (Document → CodeEntity/Repository)

**Flow:**
```
Query → Graph Search (entities) → Expand Neighbors → Vector Search (related chunks) → Fused Answer
```

### Mode C: Agentic RAG (Planner + Tools)
**Use Cases:**
- Cross-system, multi-step tasks
- Iterative refinement and decision-making
- Complex investigations ("Given this incident, trace related logs, PRs, and Slack discussions")

**Architecture:**
- LLM Orchestrator (GPT-4 Turbo)
- Tool-based retrieval system
- Multi-step planning and execution

**Tools:**
- `vector_search` - Semantic search over chunks
- `graph_get_entity` - Fetch entity by ID
- `graph_neighbors` - Expand entity relationships
- `graph_paths` - Find paths between entities
- `metadata_query` - SQL/HTTP queries for raw data

**Flow:**
```
Query → Classify → Select Strategy → Execute Tools → Refine → Answer
```

## Data Ingestion Pipeline

### Phase 1: Connector Sync
**Connectors:**
- **Code:** GitHub, GitLab, Bitbucket, Local Git
- **Docs:** Google Drive, Docs, Sheets, M365 (Word, Excel, PPT), Notion, Confluence, Figma
- **Chat:** Slack, Teams (future), Gmail, Outlook
- **Tickets:** JIRA, GitHub Issues/PRs
- **Web:** URL Scraper, Dropbox

**Output:** Normalized `DocumentForEmbedding` with:
- `id`, `source_id`, `connector_type`
- `content`, `content_type` (e.g., `text/code:typescript`)
- `metadata` (repo, path, authors, timestamps, etc.)

### Phase 2: Chunking
**Strategies by Source Type:**

**Code:**
- Function/method-level splitting (language-aware)
- Class-level and file-level embeddings
- Preserve: `repo`, `path`, `language`, `symbol_name`, `commit`, `authors`

**Documents:**
- Heading-based segmentation (H1/H2/H3 boundaries)
- Token-length breaking (512-1024 tokens)
- Preserve: `doc_id`, `title`, `heading_path`, `url`, `owners`, `tags`

**Chat/Issues/Tickets:**
- Windowed chunks (10-30 messages or 512-1024 tokens)
- Thread-aware segmentation
- Preserve: `channel`, `participants`, `thread_id`, `issue_key`, `author_ids`

**Service:** Chunker service (port 8083)
**Output:** `Chunk` objects with `chunk_id`, `content`, `metadata`, `block_type`, `language`

### Phase 3: Embedding
**Models:**
- **Code:** Qwen text-embedding-v3
- **General Text:** OpenAI text-embedding-3
- **Dimension:** 1536

**Service:** Embedding service (port 8082)
**Storage:** Qdrant collections per tenant and source type
- `tenant_X_code`
- `tenant_X_docs`
- `tenant_X_chats`

### Phase 4: Graph Ingestion
**Entity Extraction:**
- Chunker sends chunks to graph service
- Graph service extracts entities from chunks
- Entity resolution and fusion

**Code Documents:**
- Entities: Repository, File, Function, Class, Person (authors)
- Relationships: File BELONGS_TO Repository, File AUTHORED_BY Person

**Chat Documents:**
- Entities: Conversation, Message, Person (participants)
- Relationships: Message BELONGS_TO Conversation, Message AUTHORED_BY Person

**Generic Documents:**
- Entities: Document, Person (owners)
- Relationships: Document AUTHORED_BY Person

**Service:** Graph Service (graph/ microservice, port 8006)
**Storage:** Neo4j (bolt://localhost:7687) + Postgres (entities table)

## Microservice Architecture

### data/ (Port 3013)
**Responsibilities:**
- Connector management and OAuth flows
- Document normalization and sync jobs
- Triggering chunker service for GraphRAG pipeline

**Key Services:**
- `ConnectorManager` - Manages all data source connectors
- `IngestionService` - Orchestrates sync jobs and pipelines
- `GraphRagIngestionService` - Sends docs to chunker service

**APIs:**
- `POST /api/connectors/connect` - Connect data source
- `POST /api/ingestion/sync` - Start sync job
- `GET /api/ingestion/jobs` - List sync jobs
- `GET /api/ingestion/jobs/{id}` - Get job status

### graph/ (Port 8006)
**Responsibilities:**
- Graph entity extraction and Neo4j ingestion
- Entity resolution and fusion
- Graph query APIs
- Knowledge graph operations

**Key Services:**
- `ChunkProcessor` - Extracts entities from chunks
- `GraphService` - Entity CRUD and resolution
- `Neo4jClient` - Neo4j database operations
- `EntityResolver` - Cross-source entity resolution
- `FusionEngine` - Entity fusion and deduplication

**APIs:**
- `POST /graph/chunks` - Ingest chunks (from chunker)
- `GET /api/graph/entities/{id}` - Get entity
- `POST /api/graph/entities` - Create entity
- `GET /api/graph/entities/{id}/neighbors` - Get neighbors
- `GET /api/graph/paths` - Find paths between entities
- `GET /api/graph/statistics` - Graph statistics
- `POST /api/graph/query` - Unified graph query
- `POST /api/graph/cross_source` - Cross-source query
- `POST /api/graph/semantic_search` - Semantic graph search

### embedding/ (Port 8082)
**Responsibilities:**
- Embedding generation (Qwen, OpenAI)
- Vector database management (Qdrant)
- Vector search APIs
- Hybrid retrieval (vector + metadata filters)

**APIs:**
- `POST /vector/search` - Semantic search with filters
- `POST /vector/search_by_ids` - Search by chunk/entity IDs
- `POST /batch/embed` - Batch embedding generation

### agentic/ (Port 3005)
**Responsibilities:**
- Agentic RAG orchestration
- Multi-step query execution
- Tool-based retrieval planning
- LLM-based reasoning (ready for integration)

**Services:**
- `AgenticOrchestrator` - Plans and executes multi-step queries
- Tool execution engine
- Query classification and planning

**Tools:**
- `vector_search` - Calls embedding/ for semantic search
- `graph_get_entity` - Fetches entities from graph/
- `graph_neighbors` - Expands entity relationships
- `graph_paths` - Finds paths between entities
- `metadata_query` - Queries raw data

**APIs:**
- `POST /api/agentic/query` - Execute agentic multi-step query

**Features:**
- Query classification (graph-first, vector-first, hybrid)
- Multi-step tool execution with context accumulation
- Step-by-step reasoning and result tracking
- Answer generation with source attribution

### mcp/ (Port 3004)
**Responsibilities:**
- MCP protocol implementation (stdio)
- Tool and resource definitions for AI agents
- Proxy layer to ConHub services

**Protocol:**
- Implements Model Context Protocol over stdio
- Provides tools that agents can invoke
- Returns structured responses

**Tools (proxies to services):**
- `vector_search` → embedding/
- `graph_query` → graph/
- `data_fetch` → data/
- `agentic_query` → agentic/

**APIs:**
- `GET /health` - Health check only
- MCP protocol on stdin/stdout

### backend/ (Port 3010)
**Responsibilities:**
- Public HTTP/GraphQL APIs
- BFF layer for frontend
- RAG query endpoint aggregation
- Hybrid RAG orchestration

**Services:**
- `RagService` - Orchestrates vector, hybrid, and agentic queries
- Query classification
- Result fusion and reranking

**APIs:**
- `POST /api/rag/query` - Unified RAG query (auto mode selection)
- `POST /api/rag/vector` - Pure vector search
- `POST /api/rag/hybrid` - Hybrid vector + graph search
- `POST /api/rag/agentic` - Agentic multi-step retrieval (proxies to MCP)

### frontend/ (Port 3000)
**Responsibilities:**
- UI for connecting sources
- Graph visualization
- Chat interface with ConHub
- Document/code browsing

## Configuration

### Environment Variables

**data/.env:**
```bash
# Database
DATABASE_URL_NEON=postgresql://...
REDIS_URL=rediss://...

# Services
EMBEDDING_SERVICE_URL=http://localhost:8082
CHUNKER_SERVICE_URL=http://localhost:8083
GRAPH_SERVICE_URL=http://localhost:8006

# LLM for Agentic RAG
LLM_PROVIDER=openai
LLM_API_KEY=your_key
LLM_MODEL=gpt-4-turbo-preview
```

**graph/.env:**
```bash
# Database
DATABASE_URL_NEON=postgresql://...

# Neo4j
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=your_password

# Services
EMBEDDING_SERVICE_URL=http://localhost:8082
DATA_SERVICE_URL=http://localhost:3013

# Server
GRAPH_PORT=8006
```

**embedding/.env:**
```bash
# Vector DB
QDRANT_URL=https://...
QDRANT_API_KEY=...
EMBEDDING_DIMENSION=1536

# Embedding Models
QWEN_API_KEY=your_key
QWEN_EMBEDDING_MODEL=text-embedding-v3
OPENAI_API_KEY=your_key

# Services
DATA_SERVICE_URL=http://localhost:3013
```

**agentic/.env:**
```bash
# Server
AGENTIC_PORT=3005
AGENTIC_HOST=0.0.0.0

# Service URLs
EMBEDDING_SERVICE_URL=http://localhost:8082
GRAPH_SERVICE_URL=http://localhost:8006
DATA_SERVICE_URL=http://localhost:3013

# LLM Provider (ready for integration)
LLM_PROVIDER=openai
LLM_API_KEY=your_openai_api_key_here
LLM_MODEL=gpt-4-turbo-preview
LLM_TEMPERATURE=0.1
LLM_MAX_TOKENS=4096
```

**mcp/.env:**
```bash
# Server
MCP_PORT=3004

# Service URLs (for MCP tools to proxy requests)
DATA_SERVICE_URL=http://localhost:3013
EMBEDDING_SERVICE_URL=http://localhost:8082
GRAPH_SERVICE_URL=http://localhost:8006
AGENTIC_SERVICE_URL=http://localhost:3005
```

**backend/.env:**
```bash
# Services
EMBEDDING_SERVICE_URL=http://localhost:8082
GRAPH_SERVICE_URL=http://localhost:8006
AGENTIC_SERVICE_URL=http://localhost:3005

# Server
BACKEND_PORT=3010
```

## Tenant Isolation & Privacy

### Per-Tenant Data Segregation
- **Postgres:** All tables have `user_id` or `tenant_id` columns
- **Qdrant:** Separate collections per tenant (`tenant_X_code`, `tenant_X_docs`)
- **Neo4j:** All entities have `tenant_id` property, filtered in all queries

### Authorization
- Auth0 JWT tokens with `sub` (user ID) claim
- Middleware validates tokens on all API calls
- Graph queries filter by `tenant_id` from token claims
- Vector searches scoped to user's collections

### Data Privacy
- OAuth tokens encrypted at rest
- Sensitive metadata redacted based on rule system
- Audit logging for all data access
- GDPR-compliant data deletion

## Agentic RAG Strategies

### Strategy 1: Graph-First (Ownership/Structure Questions)
```
1. graph_search / graph_neighbors → Find key entities (repos, files, people)
2. Use entity IDs to fetch connected entities (AUTHORED_BY, BELONGS_TO)
3. vector_search_by_entity → Get semantically relevant chunks for those entities
4. Combine and rank → Generate answer
```

**Example:** "Who owns the authentication service?"
- Graph: Find Repository entity for "authentication service"
- Expand: Get File entities BELONGS_TO that repo
- Expand: Get Person entities via AUTHORED_BY relationships
- Vector: Fetch recent commits/PRs for context
- Answer: "Alice (60% commits), Bob (30% commits)"

### Strategy 2: Vector-First (Content Questions)
```
1. vector_search → Get top-k chunks by semantic similarity
2. Extract entity IDs from chunk metadata
3. graph_neighbors → Pull related files, issues, people
4. Use graph info to rerank and cluster
5. Generate answer with rich context
```

**Example:** "How does the payment processing work?"
- Vector: Find chunks about "payment processing"
- Graph: Get related Files, Commits, PRs, Issues
- Cluster: Group by repository/module
- Answer: With code snippets + design docs + recent changes

### Strategy 3: Hybrid Multi-Step (Complex Investigations)
```
1. Classify query → Determine scope and entities
2. Initial retrieval → Graph or vector based on classification
3. Analyze results → LLM decides what's missing
4. Refinement → Additional tool calls with filters
5. Iterate until confidence threshold or max steps
6. Generate comprehensive answer
```

**Example:** "Trace the bug from JIRA ticket ABC-123 to production"
- Graph: Find Issue entity ABC-123
- Expand: Get linked PR entities (REFERS_TO)
- Expand: Get Commit entities from PRs
- Vector: Search for related error logs/monitoring data
- Graph: Find deployment/release entities
- Timeline: Order by timestamps
- Answer: Full trace with links to each artifact

## Performance & Scalability

### Indexing Performance
- **Chunking:** ~1000 docs/min per worker
- **Embedding:** ~100 chunks/sec (batched)
- **Graph Ingestion:** ~500 entities/sec (batched)

### Query Performance
- **Vector Search:** <100ms (p95) for top-50 results
- **Graph Queries:** <200ms (p95) for 3-hop traversal
- **Agentic RAG:** 2-10 seconds (depends on steps)

### Scalability
- **Horizontal:** Each service independently scalable
- **Vertical:** Qdrant and Neo4j support clustering
- **Caching:** Redis for frequently accessed entities/chunks

## Future Enhancements

### Phase 2 (Q2 2025)
- Advanced code analysis (tree-sitter, LSP integration)
- Multi-modal embeddings (images, diagrams, Figma)
- Real-time sync via webhooks
- Collaborative filtering for personalized results

### Phase 3 (Q3 2025)
- Federated search across multiple tenants (with permissions)
- Knowledge graph reasoning (inference, entity resolution)
- Proactive insights and recommendations
- Integration with CI/CD for code quality checks

## Monitoring & Observability

### Metrics
- Ingestion throughput (docs/min, chunks/min)
- Query latency (p50, p95, p99)
- Cache hit rates
- Graph size (entities, relationships)
- Vector DB collection sizes

### Logging
- Structured logs (JSON) with trace IDs
- Per-request logging for debugging
- Audit logs for data access

### Alerting
- Ingestion failures
- Query timeouts
- Service health checks
- Database connection issues

## Security

### Authentication
- Auth0 for human users
- Service-to-service JWT tokens
- API key rotation

### Authorization
- RBAC for data access
- Tenant-scoped queries
- Rule system for sensitive data

### Data Protection
- Encryption at rest (database, vector DB)
- Encryption in transit (TLS)
- Secrets management (environment variables, not hardcoded)

## Getting Started

### Prerequisites
```bash
# Install Neo4j
docker run -d -p 7687:7687 -p 7474:7474 \
  -e NEO4J_AUTH=neo4j/your_password \
  neo4j:latest

# Install Qdrant
docker run -d -p 6333:6333 qdrant/qdrant

# Set up NeonDB (already configured)
# Set up Redis (already configured)
```

### Start Services
```bash
# Terminal 1: Data Service
cd data
cargo run

# Terminal 2: Embedding Service
cd embedding
cargo run

# Terminal 3: MCP Service
cd mcp
cargo run

# Terminal 4: Backend
cd backend
cargo run

# Terminal 5: Frontend
cd frontend
npm run dev
```

### Connect Your First Data Source
```bash
# Via API
curl -X POST http://localhost:3013/api/connectors/connect \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "connector_type": "github",
    "account_name": "My GitHub",
    "credentials": {
      "access_token": "ghp_..."
    }
  }'

# Via UI
# Navigate to http://localhost:3000/dashboard
# Click "Connect Data Source" → Select GitHub → Authorize
```

### Query Your Knowledge
```bash
# Vector RAG
curl -X POST http://localhost:3010/api/rag/query \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"query": "How does authentication work?", "mode": "vector"}'

# Graph + Vector RAG
curl -X POST http://localhost:3010/api/rag/query \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"query": "Who owns the payment service?", "mode": "hybrid"}'

# Agentic RAG
curl -X POST http://localhost:3010/api/rag/query \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"query": "Trace bug ABC-123 from JIRA to production", "mode": "agentic"}'
```

## Support

For questions or issues:
- Documentation: `/docs`
- GitHub Issues: [ConHub Issues](https://github.com/your-org/conhub/issues)
- Slack: #conhub-support
