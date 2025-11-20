# Chunker Service

The chunker microservice is responsible for intelligent chunking of source content in the ConHub Graph RAG pipeline.

## Role in Pipeline

```
data/ → chunker/ → (embedding/ + graph/)
```

1. **data/** fetches content from connectors (GitHub, Drive, Slack, etc.) and sends per-file/doc/thread items to chunker
2. **chunker/** applies type-aware chunking strategies and fans out chunks to:
   - **embedding/** for vectorization
   - **graph/** for entity/relationship extraction

## Features

- **Type-aware chunking strategies**:
  - **Code**: Function/class detection + sliding windows with overlap
  - **Documents**: Paragraph-based + sliding windows
  - **Chat**: Message window chunking with context overlap
  
- **Optimized for huge repositories**:
  - Streaming per-file processing
  - Stable chunk IDs for idempotency
  - Async fan-out to downstream services

- **Production-ready**:
  - Job-based processing with progress tracking
  - Concurrent job limits
  - Error handling and retry logic

## API Endpoints

### `POST /chunk/jobs`
Start a chunking job with a batch of source items.

**Request:**
```json
{
  "source_id": "uuid",
  "source_kind": "code_repo",
  "items": [
    {
      "id": "uuid",
      "source_id": "uuid",
      "source_kind": "code_repo",
      "content_type": "text/code:rust",
      "content": "fn main() { ... }",
      "metadata": {
        "repo": "org/repo",
        "path": "src/main.rs",
        "branch": "main"
      }
    }
  ]
}
```

**Response:**
```json
{
  "job_id": "uuid",
  "accepted": 1
}
```

### `GET /chunk/jobs/{job_id}`
Get status of a chunking job.

**Response:**
```json
{
  "job_id": "uuid",
  "status": "running",
  "items_total": 100,
  "items_processed": 45,
  "chunks_emitted": 523,
  "error_message": null
}
```

## Configuration

See `.env.example` for environment variables.

## Running

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/chunker
```

## Dependencies

- Embedding service at `EMBEDDING_SERVICE_URL`
- Graph service at `GRAPH_SERVICE_URL`
