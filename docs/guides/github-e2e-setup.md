# GitHub End-to-End Setup Guide

This guide walks you through setting up the complete GitHub integration in ConHub, from OAuth configuration to embedding generation and vector storage.

## Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- PostgreSQL 15+ (or Neon DB account)
- Qdrant Cloud account (or local Docker instance)
- GitHub account

## Step 1: Create GitHub OAuth App

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click "New OAuth App"
3. Fill in the details:
   - **Application name:** ConHub (or your preferred name)
   - **Homepage URL:** `http://localhost:3000`
   - **Authorization callback URL:** `http://localhost:3000/auth/github/callback`
4. Click "Register application"
5. Copy the **Client ID**
6. Click "Generate a new client secret" and copy the **Client Secret**

## Step 2: Set Up Qdrant

### Option A: Qdrant Cloud (Recommended for Production)

1. Sign up at [Qdrant Cloud](https://cloud.qdrant.io/)
2. Create a new cluster
3. Copy the **Cluster URL** (e.g., `https://xyz.eu-central-1-0.aws.cloud.qdrant.io`)
4. Copy the **API Key** from cluster settings

### Option B: Local Docker

```bash
docker run -p 6333:6333 -p 6334:6334 \
  -v $(pwd)/qdrant_storage:/qdrant/storage:z \
  qdrant/qdrant
```

Use `http://localhost:6333` as your Qdrant URL (no API key needed).

## Step 3: Configure Environment Variables

### Data Service (`data/.env`)

```env
# Data Service
DATA_SERVICE_PORT=3013
HOST=0.0.0.0
RUST_LOG=info

# Database (use your Neon DB or local Postgres)
DATABASE_URL_NEON=postgresql://user:password@host/database?sslmode=require

# Redis (optional, for caching)
REDIS_URL=redis://localhost:6379

# JWT (for authentication)
JWT_PUBLIC_KEY_PATH=./keys/public_key.pem
JWT_ISSUER=conhub
JWT_AUDIENCE=conhub-api

# Qdrant
QDRANT_URL=https://your-cluster-url
QDRANT_API_KEY=your-api-key
QDRANT_COLLECTION=conhub_embeddings
EMBEDDING_DIMENSION=1536

# GitHub OAuth (from Step 1)
GITHUB_CLIENT_ID=your_github_client_id
GITHUB_CLIENT_SECRET=your_github_client_secret
GITHUB_REDIRECT_URL=http://localhost:3000/auth/github/callback

# Embedding Service Integration
EMBEDDING_SERVICE_URL=http://localhost:8082

# Unified Indexer Integration (optional, for code graph)
UNIFIED_INDEXER_URL=http://localhost:8080
```

### Embedding Service (`embedding/.env`)

```env
# Embedding Service
EMBEDDING_SERVICE_PORT=8082
EMBEDDING_HOST=0.0.0.0
RUST_LOG=info
ENV_MODE=local

# Qdrant (same as data service)
QDRANT_URL=https://your-cluster-url
QDRANT_API_KEY=your-api-key
QDRANT_COLLECTION=conhub_embeddings
EMBEDDING_DIMENSION=1536

# Embedding Configuration
MAX_TEXT_LENGTH=8192
MAX_BATCH_SIZE=32
EMBEDDING_CACHE_SIZE=10000
EMBEDDING_MAX_CONCURRENCY=10
EMBEDDING_BATCH_SIZE=100

# Fusion Config
EMBEDDING_FUSION_CONFIG_PATH=config/fusion_config.json

# AI Model API Keys
# Get Qwen API key from: https://dashscope.console.aliyun.com/
QWEN_API_KEY=your_qwen_api_key

# Get OpenAI API key from: https://platform.openai.com/api-keys
OPENAI_API_KEY=your_openai_api_key
```

### Frontend (`frontend/.env`)

```env
NEXT_PUBLIC_AUTH_SERVICE_URL=http://localhost:3010
NEXT_PUBLIC_DATA_SERVICE_URL=http://localhost:3013
NEXT_PUBLIC_BILLING_SERVICE_URL=http://localhost:3011
NODE_ENV=development
ENV_MODE=local
```

## Step 4: Enable Heavy Feature Toggle

Edit `feature-toggles.json` in the root directory:

```json
{
  "Heavy": true,
  "Auth": true,
  "Billing": true,
  "GitHub": true,
  "GitLab": true,
  "Bitbucket": true,
  "GoogleDrive": true,
  "Dropbox": true,
  "Slack": true,
  "UrlScraper": true
}
```

**Important:** `Heavy: true` is required to enable embedding generation.

## Step 5: Run Database Migrations

```bash
cd database
sqlx migrate run --database-url "your_database_url"
```

This creates all necessary tables including:
- `connected_accounts`
- `source_documents`
- `document_chunks`
- `sync_jobs`
- `sync_runs`

## Step 6: Start Services

### Terminal 1: Embedding Service

```bash
cd embedding
cargo run --release
```

Expected output:
```
Starting embedding service on 0.0.0.0:8082
Initializing multi-model fusion embedding service...
Loading fusion config from: config/fusion_config.json
✓ Initialized qwen client for model code_primary
✓ Initialized openai client for model general_text
✓ Fusion embedding service initialized with 2 models
Starting HTTP server...
```

### Terminal 2: Data Service

```bash
cd data
cargo run --release
```

Expected output:
```
🚀 [Data Service] Starting on port 3013
🔐 [Data Service] Authentication middleware initialized
📊 [Data Service] Database connection established
📊 [Data Service] Embedding client initialized: http://localhost:8082
🔌 [Data Service] Connector Manager initialized
```

### Terminal 3: Frontend

```bash
cd frontend
npm install
npm run dev
```

Expected output:
```
ready - started server on 0.0.0.0:3000, url: http://localhost:3000
```

## Step 7: Connect GitHub Repository

1. Open browser to `http://localhost:3000`
2. Sign in (or create account)
3. Navigate to **Data Sources** → **Connect New Source**
4. Click **GitHub**
5. Click **Authorize** (redirects to GitHub)
6. Authorize the ConHub app
7. You'll be redirected back to ConHub

**What happens behind the scenes:**
- OAuth flow completes, access token stored
- Initial full sync starts automatically
- All repository files are fetched
- Files are chunked (1000 chars, 200 overlap)
- Chunks sent to embedding service
- Embeddings generated using Qwen model (for code)
- Vectors stored in Qdrant with metadata
- Indexer triggered for code graph (if running)

## Step 8: Verify Embeddings

### Check Qdrant

```bash
curl -X POST 'http://localhost:6333/collections/conhub_embeddings/points/scroll' \
  -H 'Content-Type: application/json' \
  -d '{
    "limit": 10,
    "with_payload": true,
    "with_vector": false
  }'
```

Or use Qdrant Cloud dashboard to browse the collection.

### Check Database

```sql
SELECT 
  sd.name,
  sd.connector_type,
  COUNT(dc.id) as chunk_count
FROM source_documents sd
LEFT JOIN document_chunks dc ON dc.document_id = sd.id
WHERE sd.connector_type = 'github'
GROUP BY sd.id, sd.name, sd.connector_type;
```

## Step 9: Test Semantic Search

### Via API

```bash
curl -X POST 'http://localhost:3013/api/search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer YOUR_JWT_TOKEN' \
  -d '{
    "query": "authentication middleware implementation",
    "limit": 5,
    "filters": {
      "connector_type": "github"
    }
  }'
```

### Via Frontend

1. Go to **Search** page
2. Enter query: "authentication middleware implementation"
3. Filter by **Source: GitHub**
4. View results with highlighted code snippets

## Troubleshooting

### Issue: "Failed to initialize fusion embedding service"

**Cause:** Missing or invalid API keys

**Solution:**
1. Check `embedding/.env` has valid `QWEN_API_KEY` and `OPENAI_API_KEY`
2. Test API keys:
   ```bash
   # Test Qwen
   curl -X POST 'https://dashscope.aliyuncs.com/api/v1/services/embeddings/text-embedding/text-embedding' \
     -H "Authorization: Bearer $QWEN_API_KEY" \
     -H 'Content-Type: application/json' \
     -d '{"model":"text-embedding-v3","input":{"texts":["test"]}}'
   
   # Test OpenAI
   curl -X POST 'https://api.openai.com/v1/embeddings' \
     -H "Authorization: Bearer $OPENAI_API_KEY" \
     -H 'Content-Type: application/json' \
     -d '{"model":"text-embedding-3-small","input":"test"}'
   ```

### Issue: "Qdrant connection failed"

**Cause:** Incorrect Qdrant URL or API key

**Solution:**
1. Verify `QDRANT_URL` is correct (include `https://` for cloud)
2. Test connection:
   ```bash
   curl -X GET "$QDRANT_URL/collections" \
     -H "api-key: $QDRANT_API_KEY"
   ```
3. For local Docker, ensure container is running:
   ```bash
   docker ps | grep qdrant
   ```

### Issue: "GitHub OAuth failed"

**Cause:** Mismatched redirect URL or invalid credentials

**Solution:**
1. Verify `GITHUB_REDIRECT_URL` in `data/.env` matches OAuth app config
2. Check `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET` are correct
3. Ensure OAuth app is not suspended
4. Check browser console for redirect errors

### Issue: "No documents synced"

**Cause:** Repository is empty or access token lacks permissions

**Solution:**
1. Verify repository has files
2. Check GitHub token scopes include `repo` (for private repos) or `public_repo`
3. View sync job status:
   ```bash
   curl -X GET 'http://localhost:3013/api/ingestion/jobs' \
     -H 'Authorization: Bearer YOUR_JWT_TOKEN'
   ```

### Issue: "Embeddings not stored in Qdrant"

**Cause:** `store_in_vector_db` flag not set or Qdrant write failed

**Solution:**
1. Check embedding service logs for Qdrant errors
2. Verify collection exists:
   ```bash
   curl -X GET "$QDRANT_URL/collections/conhub_embeddings" \
     -H "api-key: $QDRANT_API_KEY"
   ```
3. If collection doesn't exist, it will be created automatically on first upsert

## Performance Tuning

### For Large Repositories (1000+ files)

1. **Increase batch size:**
   ```env
   # embedding/.env
   EMBEDDING_BATCH_SIZE=200
   MAX_BATCH_SIZE=64
   ```

2. **Enable parallel processing:**
   ```env
   EMBEDDING_MAX_CONCURRENCY=20
   ```

3. **Adjust chunk size:**
   ```rust
   // In GitHubConnector::chunk_content
   const CHUNK_SIZE: usize = 1500;  // Larger chunks
   const CHUNK_OVERLAP: usize = 300; // More overlap
   ```

### For Rate Limiting

1. **Add delays between API calls:**
   ```rust
   // In GitHubConnector::sync_repository_branch
   tokio::time::sleep(Duration::from_millis(100)).await;
   ```

2. **Implement exponential backoff:**
   ```rust
   let mut retries = 0;
   loop {
       match api_call().await {
           Ok(result) => break result,
           Err(e) if retries < 3 => {
               retries += 1;
               tokio::time::sleep(Duration::from_secs(2_u64.pow(retries))).await;
           }
           Err(e) => return Err(e),
       }
   }
   ```

## Next Steps

1. **Add More Connectors:**
   - Follow same pattern for Jira, Confluence, Figma
   - Update `fusion_config.json` with appropriate routing rules

2. **Implement Incremental Sync:**
   - Use GitHub webhooks for real-time updates
   - Only re-embed changed files

3. **Enable Code Graph:**
   - Implement unified indexer HTTP API
   - Combine vector search with graph traversal

4. **Add Monitoring:**
   - Set up Prometheus metrics
   - Create Grafana dashboards
   - Configure alerts for failures

## Support

- **Documentation:** `/docs/architecture/embedding-pipeline.md`
- **API Reference:** `/docs/api/`
- **GitHub Issues:** [ConHub Issues](https://github.com/your-org/conhub/issues)
