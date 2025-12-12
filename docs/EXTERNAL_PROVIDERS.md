# External Provider Configuration

This document explains how to configure external AI and database providers for ConHub's knowledge layer.

## Overview

ConHub uses a **vendor-agnostic** architecture for external services:

| Service | Purpose | Supported Providers |
|---------|---------|---------------------|
| **Embeddings** | Generate vector embeddings for semantic search | Jina AI, OpenAI, Cohere, custom HTTP API |
| **Reranking** | Re-rank search results for better relevance | Jina AI, Cohere, custom HTTP API |
| **Vector Store** | Store and search embeddings | Zilliz Cloud |
| **Knowledge Graph** | Store relationships and entities | Neo4j AuraDB, local Neo4j |

---

## 1. Embeddings API Configuration

ConHub uses an HTTP-based embedding client that works with any OpenAI-compatible embedding API.

### Environment Variables

```bash
# Required: Embedding API endpoint
EMBEDDINGS_API_URL=https://api.jina.ai/v1/embeddings

# Required: API key for authentication
EXTERNAL_SEARCH_API_KEY=your_api_key_here

# Required: Model name
EMBEDDINGS_MODEL=jina-embeddings-v3

# Required: Embedding dimension (must match your model)
EMBEDDING_DIMENSION=1024
```

### Supported Providers

#### Jina AI (Recommended)
```bash
EMBEDDINGS_API_URL=https://api.jina.ai/v1/embeddings
EMBEDDINGS_MODEL=jina-embeddings-v3
EMBEDDING_DIMENSION=1024
```

#### OpenAI
```bash
EMBEDDINGS_API_URL=https://api.openai.com/v1/embeddings
EMBEDDINGS_MODEL=text-embedding-3-small
EMBEDDING_DIMENSION=1536
```

#### Cohere
```bash
EMBEDDINGS_API_URL=https://api.cohere.ai/v1/embed
EMBEDDINGS_MODEL=embed-english-v3.0
EMBEDDING_DIMENSION=1024
```

---

## 2. Reranking API Configuration

Reranking improves search result relevance by scoring query-document pairs.

### Environment Variables

```bash
# Required: Rerank API endpoint
RERANK_API_URL=https://api.jina.ai/v1/rerank

# Required: API key (typically same as embeddings)
EXTERNAL_SEARCH_API_KEY=your_api_key_here

# Optional: Model name
RERANK_MODEL=jina-reranker-v2-base-multilingual
```

### Supported Providers

#### Jina AI
```bash
RERANK_API_URL=https://api.jina.ai/v1/rerank
RERANK_MODEL=jina-reranker-v2-base-multilingual
```

#### Cohere
```bash
RERANK_API_URL=https://api.cohere.ai/v1/rerank
RERANK_MODEL=rerank-english-v3.0
```

---

## 3. Zilliz Cloud (Vector Store)

Zilliz Cloud is used as the vector database for storing and searching embeddings.

### Environment Variables

```bash
# Required: Zilliz Cloud endpoint
ZILLIZ_PUBLIC_ENDPOINT=https://in03-xxxxx.serverless.aws-eu-central-1.cloud.zilliz.com

# Required: API key
ZILLIZ_API_KEY=your_zilliz_api_key_here

# Required: Collection name
ZILLIZ_COLLECTION=conhub_embeddings

# Required: Must match EMBEDDING_DIMENSION
EMBEDDING_DIMENSION=1024
```

### Getting Zilliz Credentials

1. Sign up at [Zilliz Cloud](https://cloud.zilliz.com)
2. Create a new cluster (Serverless recommended for development)
3. Go to **Cluster Details** → **Connect**
4. Copy the **Public Endpoint** and **API Key**

### Collection Schema

ConHub automatically creates the collection with this schema:

| Field | Type | Description |
|-------|------|-------------|
| `id` | VARCHAR(64) | Primary key (chunk ID) |
| `tenant_id` | VARCHAR(64) | Tenant isolation |
| `document_id` | VARCHAR(64) | Parent document |
| `source_type` | VARCHAR(32) | github, slack, gdrive, etc. |
| `embedding` | FLOAT_VECTOR(1024) | Vector embedding |
| `text` | VARCHAR(65535) | Chunk text content |
| `metadata` | JSON | Additional metadata |

---

## 4. Neo4j (Knowledge Graph)

Neo4j stores entities and relationships for graph-based retrieval.

### Environment Variables

```bash
# Required: Neo4j connection URI
# For AuraDB: neo4j+s://xxxxx.databases.neo4j.io
# For local: bolt://localhost:7687
NEO4J_URI=neo4j+s://xxxxx.databases.neo4j.io

# Required: Database username
NEO4J_USER=neo4j

# Required: Database password
NEO4J_PASSWORD=your_password_here
```

### Getting Neo4j AuraDB Credentials

1. Sign up at [Neo4j Aura](https://console.neo4j.io)
2. Create a new AuraDB Free instance
3. Save the generated password (shown only once)
4. Copy the connection URI from the instance details

### Graph Schema

ConHub creates these node and relationship types:

**Node Labels:**
- `Repository` - Code repositories
- `File` - Source files
- `Function` - Functions/methods
- `Class` - Classes/structs
- `Person` - Contributors/users
- `Channel` - Slack/chat channels
- `Document` - Documents/wikis
- `Issue` - Tickets/issues
- `PullRequest` - PRs/MRs

**Relationship Types:**
- `CONTAINS` - Parent-child relationships
- `MENTIONS` - References between entities
- `AUTHORED_BY` - Authorship
- `DEPENDS_ON` - Dependencies
- `RELATED_TO` - General relationships

---

## 5. Service Configuration by Microservice

### data/.env
```bash
# Database
DATABASE_URL_NEON=postgresql://...

# External Embedding API
EMBEDDINGS_API_URL=https://api.jina.ai/v1/embeddings
EXTERNAL_SEARCH_API_KEY=your_jina_api_key
EMBEDDINGS_MODEL=jina-embeddings-v3
EMBEDDING_DIMENSION=1024

# Zilliz Cloud
ZILLIZ_PUBLIC_ENDPOINT=https://in03-xxxxx.cloud.zilliz.com
ZILLIZ_API_KEY=your_zilliz_key
ZILLIZ_COLLECTION=conhub_embeddings

# Downstream services
CHUNKER_SERVICE_URL=http://localhost:3017
GRAPH_RAG_SERVICE_URL=http://localhost:8006
```

### vector_rag/.env
```bash
# External Embedding API
EMBEDDINGS_API_URL=https://api.jina.ai/v1/embeddings
EXTERNAL_SEARCH_API_KEY=your_jina_api_key
EMBEDDINGS_MODEL=jina-embeddings-v3
EMBEDDING_DIMENSION=1024

# Reranking API
RERANK_API_URL=https://api.jina.ai/v1/rerank
RERANK_MODEL=jina-reranker-v2-base-multilingual

# Zilliz Cloud
ZILLIZ_PUBLIC_ENDPOINT=https://in03-xxxxx.cloud.zilliz.com
ZILLIZ_API_KEY=your_zilliz_key
ZILLIZ_COLLECTION=conhub_embeddings
```

### graph_rag/.env
```bash
# Database
DATABASE_URL_NEON=postgresql://...

# Neo4j AuraDB
NEO4J_URI=neo4j+s://xxxxx.databases.neo4j.io
NEO4J_USER=neo4j
NEO4J_PASSWORD=your_password

# Downstream services
VECTOR_RAG_SERVICE_URL=http://localhost:8082
CHUNKER_SERVICE_URL=http://localhost:3017
```

### decision_engine/.env
```bash
# Upstream services
VECTOR_RAG_URL=http://localhost:8082
GRAPH_RAG_URL=http://localhost:8006

# Redis (optional, for caching)
REDIS_URL=redis://localhost:6379
```

---

## 6. Source Retrieval Modes

ConHub supports configuring how each data source is indexed:

| Mode | Vector Store | Knowledge Graph | Best For |
|------|--------------|-----------------|----------|
| `vector_only` | ✅ | ❌ | Simple semantic search, docs |
| `graph_only` | ❌ | ✅ | Relationship-heavy data |
| `hybrid` | ✅ | ✅ | Complex content, code repos |

The decision engine automatically selects the appropriate retrieval strategy based on:
1. Source's configured retrieval mode
2. Query type analysis (fact lookup vs. relationship query)
3. Available indexes

---

## 7. Testing Your Configuration

### Test Embeddings API
```bash
curl -X POST "$EMBEDDINGS_API_URL" \
  -H "Authorization: Bearer $EXTERNAL_SEARCH_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"input": ["test"], "model": "jina-embeddings-v3"}'
```

### Test Zilliz Connection
```bash
curl -X POST "$ZILLIZ_PUBLIC_ENDPOINT/v2/vectordb/collections/list" \
  -H "Authorization: Bearer $ZILLIZ_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{}'
```

### Test Neo4j Connection
```bash
# Start the graph_rag service and check health
curl http://localhost:8006/health
```

---

## 8. Troubleshooting

### Embeddings API Errors

| Error | Cause | Solution |
|-------|-------|----------|
| 401 Unauthorized | Invalid API key | Check `EXTERNAL_SEARCH_API_KEY` |
| 400 Bad Request | Invalid model | Verify `EMBEDDINGS_MODEL` is correct |
| Dimension mismatch | Wrong dimension | Update `EMBEDDING_DIMENSION` |

### Zilliz Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Connection refused | Wrong endpoint | Check `ZILLIZ_PUBLIC_ENDPOINT` |
| Authentication failed | Invalid key | Regenerate API key in Zilliz console |
| Collection not found | First run | Collection is auto-created on first insert |

### Neo4j Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Connection failed | Wrong URI format | Use `neo4j+s://` for AuraDB |
| Authentication error | Wrong credentials | Reset password in Aura console |
| SSL error | Missing encryption | Ensure URI starts with `neo4j+s://` |

---

## 9. Security Best Practices

1. **Never commit API keys** - Use `.env` files and add to `.gitignore`
2. **Use environment-specific keys** - Separate keys for dev/staging/prod
3. **Rotate keys regularly** - Especially after team member changes
4. **Restrict API key permissions** - Use read-only keys where possible
5. **Monitor usage** - Set up alerts for unusual API consumption
