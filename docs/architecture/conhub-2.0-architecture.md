# ConHub 2.0 Architecture – Updated Knowledge Layer Design

**Status**: Active  
**Date**: November 2025  
**Purpose**: Define the updated ConHub architecture with separate chunker, vector_rag, graph_rag, and decision_engine services.

---

## 1. Executive Summary

ConHub 2.0 represents an evolution of the knowledge layer architecture with clearer separation of concerns:

- **Chunker** is now a standalone microservice (previously embedded in `data/`)
- **Vector RAG** (`embedding/` → `vector_rag/`) explicitly focuses on dense embedding retrieval
- **Graph RAG** (`graph/` → `graph_rag/`) explicitly focuses on knowledge graph retrieval
- **Decision Engine** (new) orchestrates retrieval strategies (vector, graph, hybrid, agentic)
- **Redis caching layer** for performance optimization
- **Optimized data pipeline** for large-scale ingestion (codebases, docs, chats)

---

## 2. Updated Microservices

### 2.1 Core Services

#### `auth/` – Authentication & Authorization
- **Unchanged from 1.0**
- Owns: users, identities, sessions, JWTs, OAuth flows, RBAC
- Does NOT own: connectors, documents, billing, embeddings

#### `data/` – Connectors & Document Ingestion
- **Updated responsibilities:**
  - Connector management (GitHub, Bitbucket, Google Drive, Slack, local files, web scraping)
  - OAuth flow handling and token management
  - Raw document fetching and normalization
  - Document model persistence
  - Sync job/run orchestration
  - **Removed**: chunking logic (moved to `chunker/`)
- **New behaviors:**
  - Streaming support for large repositories
  - Batch processing with pagination
  - Redis caching for connector responses
  - Rate limit handling and retry logic

#### `chunker/` – Document Chunking Service (NEW)
- **Standalone microservice** for all chunking strategies
- **Responsibilities:**
  - AST-based code chunking (per function, class, module)
  - Markdown/doc chunking (by heading hierarchy)
  - HTML chunking (semantic sections, article extraction)
  - Chat threading (conversation windows, message context)
  - Token-aware chunking (respecting LLM context limits)
- **Features:**
  - Stateless, horizontally scalable
  - Multiple concurrent chunking profiles
  - Configurable overlap and max token limits
  - Language-specific parsers (Rust, Python, TypeScript, Go, Java, etc.)
- **API**: REST/gRPC endpoints for chunking requests

#### `vector_rag/` – Vector Embeddings & Semantic Search (RENAMED from `embedding/`)
- **Renamed for clarity**: explicitly "Vector RAG"
- **Responsibilities:**
  - Dense text embedding generation (multiple models/profiles)
  - Vector database management (Qdrant collections, indexes)
  - Semantic similarity search
  - Hybrid search (semantic + keyword)
  - Re-ranking and fusion strategies
- **Optimizations:**
  - Batch embedding generation
  - Embedding caching in Redis
  - Async indexing pipeline
  - Multiple embedding profiles (default, long-context, code-specific)

#### `graph_rag/` – Knowledge Graph & Graph-Based Retrieval (RENAMED from `graph/`)
- **Renamed for clarity**: explicitly "Graph RAG"
- **Responsibilities:**
  - Knowledge graph construction (nodes: Users, Repos, Files, Issues, Messages, Chunks)
  - Edge relationship management (AUTHORED, MENTIONS, BELONGS_TO, REPLIES_TO, etc.)
  - Graph-based retrieval (subgraph extraction, neighborhood expansion, path finding)
  - Optional graph embeddings (node2vec, GraphSAGE)
  - Entity resolution and deduplication
- **Features:**
  - Multi-hop traversal with filtering
  - Centrality and importance scoring
  - Temporal graph support (versioned relationships)

#### `decision_engine/` – Retrieval Strategy Orchestrator (NEW)
- **New central orchestrator** for retrieval decisions
- **Responsibilities:**
  - Decide retrieval strategy: `vector | graph | hybrid | auto`
  - Orchestrate calls to `vector_rag/` and `graph_rag/`
  - Implement hybrid strategies (vector seeds → graph expansion)
  - Context ranking and fusion
  - Query analysis and routing
- **Strategies:**
  - **Vector**: pure semantic search for Q&A, summarization
  - **Graph**: relationship queries, "who/what/how connected"
  - **Hybrid**: vector recall + graph expansion for context enrichment
  - **Auto**: LLM-based or rule-based strategy selection
- **API**: Single `/context/query` endpoint with strategy parameter

#### `agentic/` – Agentic Orchestration & Tool Calling
- **Responsibilities:**
  - Multi-step planning and execution
  - Decide when to use cached knowledge vs. live tool calls
  - Orchestrate complex workflows (triage, review, investigate)
  - Call `decision_engine/` for context
  - Call live APIs (GitHub, Slack, Drive) when needed
- **Use cases:**
  - Freshness requirements ("latest commits in last hour")
  - Actions (comment on PR, send Slack message, create issue)
  - Permission-sensitive queries (live ACL checks)
  - Missing data (fetch details not in index)

#### `billing/` – Subscription & Usage Tracking
- **Unchanged from 1.0**
- Owns: plans, subscriptions, invoices, payments, usage tracking

#### `security/` – Audit & Compliance
- **Unchanged from 1.0**
- Owns: audit events, security policies, threat detection, rate limiting

#### `webhook/` – External Webhooks
- **Unchanged from 1.0**
- Owns: webhook endpoints for GitHub, Stripe, etc.

#### `mcp/` – MCP Protocol Adapter
- **Updated:**
  - Thin adapter to `decision_engine/` (not directly to vector/graph services)
  - Exposes unified context tools to AI agents
  - Provides connector tools for live API calls

#### `backend/` – HTTP/GraphQL BFF
- **Updated:**
  - BFF layer over `decision_engine/`
  - Exposes `/context/query` REST/GraphQL API
  - Aggregates dashboard data from all services

#### `frontend/` – Next.js UI
- **Optimizations (new):**
  - Lazy loading and code splitting
  - Virtual scrolling for large lists
  - Optimistic updates with SWR/React Query
  - Redis-backed server-side caching

#### `indexers/` – Background Jobs
- **Updated:**
  - Orchestrates full ingestion pipeline:
    1. `data/` for raw fetch
    2. `chunker/` for chunking
    3. `vector_rag/` + `graph_rag/` for indexing
  - Incremental sync detection
  - Job queues (Redis-backed)

---

## 3. New Infrastructure Components

### 3.1 Redis Caching Layer
- **Purpose**: Performance optimization across services
- **Use cases:**
  - Connector API response caching (GitHub rate limit mitigation)
  - Embedding cache (avoid re-embedding identical content)
  - Query result caching (decision_engine responses)
  - Session and rate limit tracking
- **Deployment**: Shared Redis instance or per-service as needed

### 3.2 Message Queue (Optional, Future)
- **For async pipelines:**
  - Document ingestion events
  - Chunk processing queues
  - Index update events
- **Tech options**: Redis Streams, RabbitMQ, or Kafka

---

## 4. Data Flow

### 4.1 Ingestion Pipeline (Offline)

```
User configures connector (UI)
        ↓
    data/ fetches raw items (GitHub API, Drive API, etc.)
        ↓
    data/ normalizes → Document model
        ↓
    data/ calls chunker/ → DocumentChunks
        ↓
    data/ persists documents + chunks
        ↓
    indexers/ triggers:
        ├─→ vector_rag/ for embedding + vector index
        └─→ graph_rag/ for graph construction
```

### 4.2 Query Pipeline (Online)

```
AI Agent / Client
        ↓
   mcp/ or backend/
        ↓
   decision_engine/
        ├─→ vector_rag/search (if strategy = vector)
        ├─→ graph_rag/search (if strategy = graph)
        └─→ both + fusion (if strategy = hybrid)
        ↓
   Ranked ContextBlocks
        ↓
   Return to agent/client
```

### 4.3 Agentic Flow (Selective)

```
AI Agent objective
        ↓
    agentic/ planner
        ├─→ decision_engine/ for context
        ├─→ live GitHub API (if fresh data needed)
        ├─→ live Slack API (if action needed)
        └─→ Combine results
```

---

## 5. API Contracts

### 5.1 `chunker/` API

**`POST /chunker/chunk`**

Request:
```json
{
  "document_id": "uuid",
  "content": "string",
  "mime_type": "text/x-rust",
  "content_type": "code|doc|chat|web",
  "options": {
    "max_tokens": 512,
    "overlap_tokens": 64,
    "language": "rust"
  }
}
```

Response:
```json
{
  "document_id": "uuid",
  "chunks": [
    {
      "position": 0,
      "offset_start": 0,
      "offset_end": 1200,
      "content": "fn main() { ... }",
      "heading_path": ["main.rs", "main"],
      "metadata": {
        "function_name": "main",
        "start_line": 1,
        "end_line": 15
      }
    }
  ]
}
```

### 5.2 `vector_rag/` API

**`POST /vector_rag/search`**

Request:
```json
{
  "profile": "default",
  "query": "How to implement authentication?",
  "filters": {
    "source_type": ["github"],
    "tenant_id": "uuid"
  },
  "top_k": 20
}
```

Response:
```json
{
  "results": [
    {
      "chunk_id": "uuid",
      "document_id": "uuid",
      "score": 0.89,
      "content": "...",
      "metadata": { ... }
    }
  ]
}
```

### 5.3 `graph_rag/` API

**`POST /graph_rag/expand`**

Request:
```json
{
  "seed_nodes": ["file_uuid_1", "issue_uuid_2"],
  "max_hops": 2,
  "edge_types": ["MENTIONS", "AUTHORED_BY"],
  "max_nodes": 50
}
```

Response:
```json
{
  "nodes": [ ... ],
  "edges": [ ... ],
  "paths": [ ... ]
}
```

### 5.4 `decision_engine/` API

**`POST /context/query`**

Request:
```json
{
  "tenant_id": "uuid",
  "user_id": "uuid",
  "query": "Explain the authentication flow",
  "filters": {
    "source_types": ["github"],
    "repos": ["org/repo"]
  },
  "strategy": "vector|graph|hybrid|auto",
  "top_k": 20
}
```

Response:
```json
{
  "strategy_used": "hybrid",
  "blocks": [
    {
      "chunk_id": "uuid",
      "document_id": "uuid",
      "content": "...",
      "score": 0.91,
      "provenance": {
        "vector": { "similarity": 0.93 },
        "graph": { "path": ["auth.rs", "login_handler", "verify_token"] }
      }
    }
  ]
}
```

---

## 6. Technology Stack

### Languages & Frameworks
- **Backend**: Rust (Axum, Tokio, SQLx)
- **Frontend**: Next.js 14 (React, TypeScript, TailwindCSS)
- **Chunking**: tree-sitter (AST parsing), comrak (Markdown), html5ever (HTML)

### Databases & Storage
- **Primary DB**: NeonDB (PostgreSQL)
- **Vector DB**: Qdrant
- **Graph DB**: Neo4j or custom graph layer on PostgreSQL
- **Cache**: Redis
- **Object Storage**: S3-compatible (for large files)

### Deployment
- **Container**: Docker
- **Orchestration**: Azure Container Apps / Kubernetes
- **Gateway**: Nginx
- **Monitoring**: Prometheus + Grafana

---

## 7. Chunking Strategies (Advanced)

### 7.1 Code Chunking (AST-based)
- Use `tree-sitter` for parsing
- Chunk boundaries:
  - Functions/methods
  - Classes/structs/enums
  - Modules/namespaces
- Include:
  - Function signatures
  - Docstrings/comments
  - Import context (top-level imports)
- Language support: Rust, Python, TypeScript, JavaScript, Go, Java, C++

### 7.2 Document Chunking (Semantic)
- Markdown: by heading hierarchy (`#`, `##`, `###`)
- HTML: by semantic tags (`<article>`, `<section>`, `<h1>`-`<h6>`)
- PDF: by page or paragraph (future)
- Preserve:
  - Heading path breadcrumbs
  - Links and references

### 7.3 Chat Chunking (Threading)
- Slack/Discord/Teams: by thread + context window
- Preserve:
  - Thread parent
  - Participant list
  - Reactions and mentions
- Chunk strategies:
  - Fixed message windows (e.g., 10 messages)
  - Semantic thread boundaries (topic shifts)

### 7.4 Token-Oriented Chunking
- Respect LLM context limits (e.g., 512, 1024, 2048 tokens)
- Configurable overlap (e.g., 64 tokens)
- Token counting: `tiktoken` or model-specific tokenizer
- Avoid mid-sentence splits (use sentence boundaries)

---

## 8. Caching Strategy

### 8.1 Connector Response Cache
- **Key**: `connector:{type}:{external_id}:{endpoint}`
- **TTL**: 5-60 minutes (configurable per connector)
- **Purpose**: Reduce GitHub/Slack API calls, avoid rate limits

### 8.2 Embedding Cache
- **Key**: `embedding:{profile}:{content_hash}`
- **TTL**: Indefinite (invalidate on model change)
- **Purpose**: Avoid re-embedding identical content

### 8.3 Query Result Cache
- **Key**: `context_query:{query_hash}:{filters_hash}`
- **TTL**: 1-5 minutes
- **Purpose**: Serve repeated queries fast

### 8.4 Session & Rate Limit Tracking
- **Key**: `rate_limit:{user_id}:{endpoint}`
- **TTL**: 1 minute - 1 hour
- **Purpose**: Enforce API quotas

---

## 9. Performance Optimizations

### 9.1 Data Service
- Streaming APIs for large repos (avoid loading entire tree in memory)
- Parallel fetching (concurrent file downloads)
- Incremental sync (only fetch changed files since last sync)
- Batch processing (chunk multiple documents in one call)

### 9.2 Chunker Service
- Stateless design (horizontal scaling)
- Pool of language parsers (reuse tree-sitter instances)
- Streaming chunking (yield chunks as parsed, don't wait for full doc)

### 9.3 Vector RAG
- Batch embedding generation (process multiple chunks in one model call)
- Async indexing (non-blocking upserts)
- Index sharding (by tenant or source type)

### 9.4 Graph RAG
- Lazy graph loading (don't load entire graph into memory)
- Indexed lookups (on external_id, type, tenant_id)
- Materialized path queries (for faster traversals)

### 9.5 Frontend
- Code splitting and lazy loading (per route)
- Virtual scrolling (react-window) for large lists
- Debounced search inputs
- Optimistic updates with SWR/React Query
- Server-side caching (Next.js ISR/SSR with Redis)

---

## 10. Migration Path from 1.0 to 2.0

### Step 1: Rename Services
1. `embedding/` → `vector_rag/`
2. `graph/` → `graph_rag/`
3. Update all import paths and service discovery

### Step 2: Extract Chunker
1. Create new `chunker/` crate
2. Move chunking logic from `data/src/services/chunking.rs` to `chunker/src/`
3. Expose REST API in `chunker/`
4. Update `data/` to call `chunker/` via HTTP

### Step 3: Create Decision Engine
1. New `decision_engine/` crate
2. Implement `/context/query` endpoint
3. Wire to `vector_rag/` and `graph_rag/`
4. Start with vector-only, then add graph and hybrid

### Step 4: Add Redis
1. Deploy Redis instance
2. Add caching middleware to `data/`, `vector_rag/`, `decision_engine/`
3. Implement cache-aside pattern

### Step 5: Update MCP & Backend
1. Point `mcp/` tools to `decision_engine/` instead of direct vector/graph calls
2. Add `/context/query` to `backend/` GraphQL schema
3. Update frontend to use new endpoint

### Step 6: Optimize & Test
1. Add performance monitoring
2. Load testing with large repos
3. Frontend optimizations (lazy loading, virtualization)
4. End-to-end integration tests

---

## 11. Success Metrics

- **Ingestion throughput**: Documents/sec, chunks/sec
- **Query latency**: p50, p95, p99 for context queries
- **Cache hit rates**: Connector cache, embedding cache, query cache
- **Accuracy**: Retrieval quality (vector recall, graph precision)
- **Scalability**: Max repo size, max concurrent users

---

## 12. Next Steps

1. ✅ Update architecture docs (this file)
2. [ ] Update `.windsurf/rules/` (manual, reference this doc)
3. [ ] Implement `chunker/` service
4. [ ] Rename `embedding/` → `vector_rag/`
5. [ ] Rename `graph/` → `graph_rag/`
6. [ ] Implement `decision_engine/`
7. [ ] Add Redis caching layer
8. [ ] Optimize data service for large codebases
9. [ ] Frontend optimizations
10. [ ] End-to-end testing

---

**Document maintained by**: ConHub Architecture Team  
**Last updated**: November 2025
