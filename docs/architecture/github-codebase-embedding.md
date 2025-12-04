# GitHub Codebase Embedding Pipeline

This document describes the end-to-end flow for fetching GitHub repositories and embedding them into ConHub's vector and graph RAG systems.

## Architecture Overview

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Frontend   │────▶│   Data Svc   │────▶│  Auth Svc    │
│ (React/Next) │     │  (Port 3013) │     │ (Port 3010)  │
└──────────────┘     └──────┬───────┘     └──────────────┘
                           │ Token Resolution
                           │ (Internal API)
                           ▼
                    ┌──────────────┐
                    │   GitHub     │
                    │     API      │
                    └──────┬───────┘
                           │ Fetch Code
                           ▼
              ┌────────────────────────────┐
              │   DocumentForEmbedding     │
              │   (Chunked Code Files)     │
              └──────────┬─────────────────┘
                         │
           ┌─────────────┴─────────────┐
           │                           │
           ▼                           ▼
    ┌──────────────┐           ┌──────────────┐
    │  Embedding   │           │   Chunker    │
    │   Service    │           │   Service    │
    │ (Port 8082)  │           │ (Port 3017)  │
    └──────┬───────┘           └──────┬───────┘
           │                          │
           ▼                          ▼
    ┌──────────────┐           ┌──────────────┐
    │   Zilliz     │           │  Graph RAG   │
    │ Vector Store │           │   (Neo4j)    │
    └──────────────┘           └──────────────┘
```

## Components

### 1. Auth Service (Internal Endpoints)

**File:** `auth/src/handlers/auth.rs`

New internal endpoints for service-to-service token resolution:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/internal/oauth/{provider}/token` | GET | Get OAuth token for a user |
| `/internal/oauth/{provider}/status` | GET | Check connection status |

Query params: `?user_id={uuid}`

These endpoints are not protected by auth middleware and should be network-isolated in production.

### 2. Data Service (GitHub Sync)

**File:** `data/src/main.rs`

New secure sync endpoint:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/github/sync` | POST | Sync repository (secure, uses JWT) |
| `/api/github/sync-repository` | POST | Sync repository (legacy, needs token) |

**Secure Sync Request:**
```json
{
  "repo_url": "https://github.com/owner/repo",
  "branch": "main",
  "include_languages": ["rust", "typescript"],  // optional
  "exclude_paths": ["node_modules", "dist"],     // optional
  "max_file_size_mb": 5                          // optional
}
```

**Response:**
```json
{
  "success": true,
  "documents_processed": 150,
  "embeddings_created": 150,
  "sync_duration_ms": 5000,
  "graph_job_id": "uuid",
  "error_message": null
}
```

### 3. Auth Client (`data/src/services/auth_client.rs`)

A new client for internal service-to-service calls:

```rust
let auth_client = AuthClient::from_env();

// Get OAuth token
let token = auth_client.get_oauth_token(user_id, "github").await?;

// Check connection status
let status = auth_client.check_oauth_status(user_id, "github").await?;
```

### 4. Embedding Client (`data/src/services/embedding_client.rs`)

Sends documents to embedding service:

```rust
let embedding_client = EmbeddingClient::new(url, enabled);
embedding_client.embed_documents(documents).await?;
```

### 5. Graph RAG Ingestion (`data/src/services/graph_rag_ingestion.rs`)

Sends documents to chunker service for graph construction:

```rust
let graph_service = GraphRagIngestionService::new(chunker_url);
let job_id = graph_service.ingest_documents(source_id, SourceKind::CodeRepo, documents).await?;
```

## Data Flow

1. **User triggers sync** via frontend or API
2. **Data service** extracts `user_id` from JWT claims
3. **Auth client** calls internal auth endpoint to get GitHub token
4. **GitHubConnector** fetches repository files and creates `DocumentForEmbedding` objects
5. **Documents are sent** in parallel to:
   - Embedding service → Zilliz vector store
   - Graph RAG ingestion → Chunker → Neo4j graph

## Environment Variables

### Data Service (`data/.env`)

```env
# Auth service for token resolution
AUTH_SERVICE_URL=http://localhost:3010

# Embedding service
EMBEDDING_SERVICE_URL=http://localhost:8082
EMBEDDING_ENABLED=true

# Chunker service for graph RAG
CHUNKER_SERVICE_URL=http://localhost:3017
GRAPH_RAG_ENABLED=true
```

## Frontend Component

**File:** `frontend/components/sources/connectors/github/GitHubRepoSync.tsx`

A React component that:
- Shows connected GitHub account status
- Lists available repositories
- Allows branch selection
- Provides advanced filtering (languages, exclude paths, file size)
- Shows sync progress and results

## Security Considerations

1. **Token never exposed to frontend** - The frontend calls the data service, which internally calls auth service
2. **Internal endpoints are network-isolated** - `/internal/*` endpoints have no auth middleware but should be protected at network level
3. **JWT validation** - All user-facing endpoints require valid JWT tokens
4. **Rate limiting** - Sync operations should be rate-limited per user

## Observability

- All operations log with emojis for easy scanning
- Sync duration tracked and returned in response
- Graph job ID returned for async tracking
- Errors gracefully handled with detailed messages

## Future Improvements

1. **Incremental sync** - Only re-index changed files
2. **Webhook-triggered sync** - Auto-sync on push events
3. **Progress streaming** - Real-time sync progress via SSE/WebSocket
4. **File-level filtering** - Select specific files/folders to index
5. **Sync scheduling** - Automatic periodic syncs
