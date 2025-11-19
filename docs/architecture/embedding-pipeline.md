# ConHub Multi-Model Embedding Pipeline Architecture

## Overview

ConHub implements a sophisticated multi-model embedding pipeline that routes documents to specialized embedding models based on their source type, fuses embeddings intelligently, and stores them with rich relational metadata in Qdrant for optimal AI agent retrieval.

## Architecture Components

### 1. Data Service (Port 3013)
**Role:** Connector orchestration, document ingestion, and embedding coordination

**Key Responsibilities:**
- Manages OAuth flows for all connectors (GitHub, Jira, Confluence, Figma, etc.)
- Orchestrates document synchronization from connected sources
- Chunks documents intelligently based on content type
- Sends documents to embedding service via batch protocol
- Triggers unified indexer for structural code analysis

**Environment Variables:**
```env
EMBEDDING_SERVICE_URL=http://localhost:8082
UNIFIED_INDEXER_URL=http://localhost:8080
GITHUB_CLIENT_ID=<your-github-oauth-app-client-id>
GITHUB_CLIENT_SECRET=<your-github-oauth-app-secret>
GITHUB_REDIRECT_URL=http://localhost:3000/auth/github/callback
```

### 2. Embedding Service (Port 8082)
**Role:** Multi-model embedding generation and vector storage

**Key Responsibilities:**
- Routes documents to appropriate embedding models based on `connector_type`
- Fuses multiple model embeddings using configurable strategies
- Enriches metadata with embedding profile and versioning info
- Stores vectors in Qdrant with comprehensive payload

**Environment Variables:**
```env
EMBEDDING_FUSION_CONFIG_PATH=config/fusion_config.json
QWEN_API_KEY=<your-qwen-api-key>
OPENAI_API_KEY=<your-openai-api-key>
QDRANT_URL=https://your-qdrant-cluster-url
QDRANT_API_KEY=<your-qdrant-api-key>
QDRANT_COLLECTION=conhub_embeddings
```

### 3. Qdrant Vector Database
**Role:** Scalable vector storage with metadata filtering

**Key Responsibilities:**
- Stores fused embeddings with cosine similarity indexing
- Maintains rich payload for each vector point
- Enables hybrid search (vector + metadata filters)
- Supports multi-tenant isolation via payload filtering

## End-to-End GitHub Flow

### Phase 1: Connection & OAuth
```
User → Frontend → Data Service
  ↓
Data Service calls GitHubConnector.authenticate()
  ↓
Returns OAuth URL with GITHUB_CLIENT_ID
  ↓
User authorizes → GitHub redirects to GITHUB_REDIRECT_URL
  ↓
Frontend sends code to Data Service
  ↓
GitHubConnector.complete_oauth() exchanges code for access_token
  ↓
Stores credentials in connected_accounts table
```

### Phase 2: Initial Sync
```
Data Service (on successful connection)
  ↓
Calls ConnectorManager.sync(account_id, force_full_sync=true)
  ↓
GitHubConnector walks repository tree
  ↓
For each file:
  - Fetches content via GitHub API
  - Chunks content (1000 chars, 200 overlap)
  - Creates DocumentForEmbedding with:
    * id, source_id, connector_type="github"
    * external_id (file SHA)
    * name, path, content
    * metadata: {url, size, branch, repository}
    * chunks: [{chunk_number, content, offsets, metadata}]
  ↓
Returns Vec<DocumentForEmbedding>
```

### Phase 3: Batch Embedding
```
Data Service
  ↓
EmbeddingClient.embed_documents(documents)
  ↓
POST /batch/embed to EMBEDDING_SERVICE_URL
  ↓
Embedding Service receives BatchEmbedRequest
  ↓
For each document:
  ↓
  batch_embed_handler extracts connector_type="github"
  ↓
  FusionEmbeddingService.generate_embeddings(texts, "github")
  ↓
  Looks up routing rule for "github" in fusion_config.json
  ↓
  Routes to "code_primary" model (Qwen text-embedding-v3)
  ↓
  Generates embeddings via QwenEmbeddingClient
  ↓
  Applies fusion strategy (weighted_average for single model)
  ↓
  Creates EmbeddedChunk with:
    * document_id, chunk_number, content, embedding
    * metadata:
      - connector_type, source_id, external_id, path
      - chunk_number, start_offset, end_offset
      - embedding_profile="github"
      - embedding_strategy="fusion"
      - normalize_embeddings=true
  ↓
If store_in_vector_db=true:
  ↓
  VectorStoreService.ensure_collection("conhub_embeddings", 1536)
  ↓
  For each chunk:
    - id = "{document_id}-{chunk_number}"
    - vector = embedding
    - payload = metadata + {content: chunk_text}
  ↓
  VectorStoreService.upsert(collection, points)
  ↓
  Qdrant stores vectors with IVFFlat index
```

### Phase 4: Indexer Trigger (GitHub-specific)
```
Data Service (after embedding)
  ↓
Extracts unique repository URLs from document metadata
  ↓
For each repo:
  ↓
  POST {repository_url, branch} to UNIFIED_INDEXER_URL/api/index/repository
  ↓
  Unified Indexer (when implemented):
    - Clones repository
    - Runs tree-sitter parsing
    - Extracts symbols, definitions, references
    - Builds code graph
    - Stores in separate index
```

## Multi-Model Fusion Configuration

### Fusion Config Structure (`config/fusion_config.json`)

```json
{
  "models": [
    {
      "name": "code_primary",
      "client": "qwen",
      "model": "text-embedding-v3",
      "dimension": 1536,
      "strengths": ["code", "github", "gitlab", "bitbucket"]
    },
    {
      "name": "general_text",
      "client": "openai",
      "model": "text-embedding-3-small",
      "dimension": 1536,
      "strengths": ["docs", "jira", "confluence", "slack"]
    },
    {
      "name": "design_context",
      "client": "openai",
      "model": "text-embedding-3-large",
      "dimension": 3072,
      "strengths": ["figma", "confluence"]
    }
  ],
  "routing": [
    {
      "source": "github",
      "models": ["code_primary"],
      "weights": [1.0],
      "fusion_strategy": "weighted_average"
    },
    {
      "source": "confluence",
      "models": ["general_text", "design_context"],
      "weights": [0.7, 0.3],
      "fusion_strategy": "weighted_average"
    }
  ],
  "fallback_model": "general_text"
}
```

### Fusion Strategies

1. **Weighted Average** (default)
   - Combines embeddings using configured weights
   - Normalizes by total weight
   - Handles dimension mismatches via interpolation
   - Best for: Most use cases, maintains dimension

2. **Concatenate**
   - Concatenates all model embeddings
   - Output dimension = sum of all model dimensions
   - Best for: Maximum information retention
   - Trade-off: Larger vectors, higher storage/compute

3. **Max Pooling**
   - Takes element-wise maximum across embeddings
   - Maintains single model dimension
   - Best for: Capturing strongest signals
   - Trade-off: May lose nuanced information

## Relational Metadata Architecture

### Core Principle
**Relations live in metadata, not in vector space.**

### Qdrant Payload Structure

Every vector point in Qdrant has this payload:

```json
{
  "content": "actual chunk text",
  
  // Core relational IDs
  "connector_type": "github",
  "source_id": "uuid-of-connected-account",
  "external_id": "file-sha-or-issue-key",
  "path": "src/main.rs",
  
  // Chunk positioning
  "chunk_number": 0,
  "start_offset": 0,
  "end_offset": 1000,
  
  // Connector-specific metadata
  "url": "https://github.com/org/repo/blob/main/src/main.rs",
  "size": 5432,
  "branch": "main",
  "repository": "org/repo",
  
  // Embedding metadata
  "embedding_profile": "github",
  "embedding_strategy": "fusion",
  "normalize_embeddings": true,
  
  // Optional chunk-level metadata
  "chunk_metadata": {
    "file_path": "src/main.rs",
    "length": 1000
  }
}
```

### Postgres Schema (Canonical Relations)

```sql
-- Source documents table
CREATE TABLE source_documents (
    id UUID PRIMARY KEY,
    source_id UUID NOT NULL,  -- connected_accounts.id
    connector_type VARCHAR(50),
    external_id VARCHAR(255),
    name VARCHAR(255),
    path TEXT,
    content_type VARCHAR(50),
    size BIGINT,
    metadata JSONB,
    indexed_at TIMESTAMPTZ,
    FOREIGN KEY (source_id) REFERENCES connected_accounts(id)
);

-- Document chunks table
CREATE TABLE document_chunks (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL,
    chunk_number INTEGER,
    content TEXT,
    start_offset INTEGER,
    end_offset INTEGER,
    metadata JSONB,
    embedding_vector VECTOR(1536),
    FOREIGN KEY (document_id) REFERENCES source_documents(id) ON DELETE CASCADE,
    UNIQUE(document_id, chunk_number)
);
```

### Cross-Source Relations

Relations between different sources (e.g., GitHub PR ↔ Jira issue) are built by:

1. **Explicit Links in Metadata:**
   ```json
   {
     "connector_type": "github",
     "external_id": "pr-123",
     "linked_jira_issues": ["PROJ-456", "PROJ-789"]
   }
   ```

2. **Semantic Search:**
   - Query Qdrant with issue description vector
   - Filter by `connector_type="github"`
   - Find related code via cosine similarity

3. **Graph Layer (Future):**
   - Build knowledge graph from payload metadata
   - Store in Neo4j or similar
   - Enable graph traversal queries

## Scalability & Optimization

### 1. Embedding Generation

**Parallel Processing:**
```rust
// Process chunks in batches of 16
const CHUNK_BATCH_SIZE: usize = 16;

for chunk_batch in chunks.chunks(CHUNK_BATCH_SIZE) {
    let texts: Vec<String> = chunk_batch.iter()
        .map(|c| c.content.clone())
        .collect();
    
    let embeddings = service.generate_embeddings(&texts, source_type).await?;
    // Process embeddings...
}
```

**Caching:**
- `EmbeddingCache` with LRU eviction (10,000 entries)
- Cache key: `hash(text + model + source_type)`
- Reduces redundant API calls for duplicate content

### 2. Vector Storage

**Qdrant Optimization:**
- IVFFlat index for approximate nearest neighbor search
- Payload indexing on frequently filtered fields:
  - `connector_type`
  - `source_id`
  - `embedding_profile`
- Batch upsert (up to 100 points per request)

**Collection Strategy:**
- Single collection: `conhub_embeddings`
- Multi-tenancy via payload filtering
- Dimension: 1536 (matches most models)
- Distance: Cosine similarity

### 3. Data Structures & Algorithms

**Chunking Algorithm:**
```rust
// Optimized sliding window with overlap
const CHUNK_SIZE: usize = 1000;
const CHUNK_OVERLAP: usize = 200;

fn chunk_content(content: &str) -> Vec<DocumentChunk> {
    let bytes = content.as_bytes();
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut chunk_number = 0;
    
    while start < bytes.len() {
        let end = (start + CHUNK_SIZE).min(bytes.len());
        
        // Find word boundary
        let chunk_end = if end < bytes.len() {
            bytes[start..end]
                .iter()
                .rposition(|&b| b == b' ' || b == b'\n')
                .map(|pos| start + pos)
                .unwrap_or(end)
        } else {
            end
        };
        
        let chunk_text = String::from_utf8_lossy(&bytes[start..chunk_end]);
        chunks.push(DocumentChunk {
            chunk_number,
            content: chunk_text.to_string(),
            start_offset: start,
            end_offset: chunk_end,
            metadata: None,
        });
        
        chunk_number += 1;
        start = chunk_end.saturating_sub(CHUNK_OVERLAP);
    }
    
    chunks
}
```

**Fusion Dimension Normalization:**
```rust
// Linear interpolation for dimension mismatch
fn normalize_dimension(embedding: &[f32], target_dim: usize) -> Vec<f32> {
    let source_dim = embedding.len();
    let ratio = source_dim as f32 / target_dim as f32;
    
    (0..target_dim)
        .map(|i| {
            let source_idx = (i as f32 * ratio) as usize;
            embedding.get(source_idx).copied().unwrap_or(0.0)
        })
        .collect()
}
```

### 4. Batch Protocol Optimization

**Request Batching:**
- Max 100 documents per batch
- Max 8192 chars per chunk
- Automatic chunking if exceeded

**Response Streaming (Future):**
- Server-Sent Events for real-time progress
- Partial results for long-running batches

## Security & Access Control

### 1. Credential Storage
- OAuth tokens encrypted in `connected_accounts.credentials` (JSONB)
- Encryption key from `JWT_SECRET` environment variable
- Tokens refreshed automatically before expiration

### 2. Multi-Tenancy
- All documents tagged with `source_id` (connected account)
- Connected accounts linked to `user_id`
- Qdrant queries filtered by `source_id` to enforce isolation

### 3. API Authentication
- Data service: JWT-based auth via `conhub-middleware`
- Embedding service: Internal-only (no public exposure)
- Qdrant: API key authentication

## Monitoring & Observability

### Metrics to Track

1. **Embedding Performance:**
   - Embeddings generated per second
   - Average latency per model
   - Cache hit rate
   - Fusion strategy distribution

2. **Vector Storage:**
   - Qdrant collection size
   - Query latency (p50, p95, p99)
   - Upsert throughput
   - Index rebuild frequency

3. **Connector Health:**
   - Sync success rate per connector
   - Documents processed per sync
   - API rate limit usage
   - OAuth token refresh failures

### Logging

```rust
// Structured logging with tracing
log::info!(
    "📦 Batch processing complete: {} successful, {} failed in {}ms",
    successful,
    failed,
    duration
);

log::debug!(
    "Generating embeddings with model: {} for source: {}",
    model_name,
    source_type
);
```

## Future Enhancements

### 1. Adaptive Routing
- ML model to learn optimal model selection per document
- A/B testing different fusion strategies
- Automatic fallback on model failure

### 2. Incremental Embeddings
- Delta updates for modified documents
- Chunk-level change detection
- Efficient re-embedding of changed sections

### 3. Multi-Vector Support
- Store multiple embeddings per chunk (one per model)
- Query-time fusion based on query type
- Qdrant named vectors feature

### 4. Knowledge Graph Integration
- Extract entities and relationships from payload
- Build graph in Neo4j
- Hybrid retrieval: vector + graph traversal

## Troubleshooting

### Common Issues

1. **"No embedding clients could be initialized"**
   - Check API keys in `.env`
   - Verify `Heavy` feature toggle is enabled
   - Ensure `fusion_config.json` is accessible

2. **"Qdrant connection failed"**
   - Verify `QDRANT_URL` and `QDRANT_API_KEY`
   - Check network connectivity
   - Ensure collection exists or service can create it

3. **"GitHub OAuth failed"**
   - Verify `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET`
   - Check redirect URL matches OAuth app config
   - Ensure user granted required scopes (`repo`, `read:user`)

4. **"Dimension mismatch"**
   - Check `fusion_config.json` model dimensions
   - Ensure Qdrant collection dimension matches
   - Verify fusion strategy handles dimension correctly

## References

- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [OpenAI Embeddings Guide](https://platform.openai.com/docs/guides/embeddings)
- [GitHub OAuth Apps](https://docs.github.com/en/developers/apps/building-oauth-apps)
- [Qwen Embedding API](https://help.aliyun.com/zh/dashscope/developer-reference/text-embedding-api-details)
