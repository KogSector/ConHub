# RAG Pipeline Design

## Overview

This document outlines the Retrieval-Augmented Generation (RAG) pipeline architecture for ConHub, detailing the flow from text input through embedding generation, indexing, storage, and retrieval.

## Pipeline Architecture

```
Text Input → Embedding Service → Indexing Service → Vector Storage → Retrieval → Generation
     ↓              ↓                   ↓               ↓            ↓           ↓
  Document      Fusion Embedding    Chunking &      Qdrant DB    Similarity   LLM Response
  Processing    (Multi-modal)       Metadata        Storage      Search       Generation
```

## 1. Text Processing & Chunking

### Input Sources
- **Documents**: PDF, DOCX, TXT files
- **Code**: Repository files, documentation
- **Web Content**: Scraped pages, articles
- **Multimodal**: Images, audio, video content

### Chunking Strategy
```rust
pub struct ChunkingConfig {
    pub max_chunk_size: usize,      // Default: 512 tokens
    pub overlap_size: usize,        // Default: 50 tokens
    pub min_chunk_size: usize,      // Default: 100 tokens
    pub preserve_structure: bool,   // Respect paragraphs/sections
}
```

### Metadata Extraction
- Document title, author, creation date
- File path, content type
- Section headers, page numbers
- Code language, function names

## 2. Embedding Generation

### Fusion Embedding Service (Port 8082)

#### Supported Modalities
- **Text**: BGE, Qwen3, E5 models
- **Image**: Vision transformers
- **Audio**: Speech-to-text + text embedding
- **Code**: Code-specific embeddings
- **Multimodal**: Cross-modal fusion

#### Fusion Strategies
```rust
pub enum FusionStrategy {
    Concatenation,      // Simple concat
    WeightedSum,        // Learned weights
    AttentionWeighted,  // Attention mechanism
    Average,            // Mean pooling
    Max,               // Max pooling
}
```

#### API Endpoints
- `POST /embed` - Generate embeddings
- `POST /rerank` - Rerank documents
- `GET /health` - Service health check

## 3. Indexing Service (Port 8081)

### Core Responsibilities
- Coordinate embedding generation
- Manage chunking and metadata
- Handle batch processing
- Store embeddings in vector database

### Service Integration
```rust
pub struct IndexingPipeline {
    embedding_service: EmbeddingService,
    chunking_service: ChunkingService,
    vector_store: VectorStore,
    metadata_store: MetadataStore,
}
```

### Processing Flow
1. **Document Ingestion**
   - Validate file format and size
   - Extract text content
   - Parse metadata

2. **Chunking**
   - Split into semantic chunks
   - Maintain context overlap
   - Preserve document structure

3. **Embedding Generation**
   - Send chunks to embedding service
   - Handle batch processing (max 32 texts)
   - Implement retry logic with exponential backoff

4. **Storage**
   - Store embeddings in Qdrant
   - Index metadata in PostgreSQL
   - Maintain document-chunk relationships

## 4. Vector Storage (Qdrant)

### Collection Schema
```rust
pub struct DocumentChunk {
    pub id: String,
    pub embedding: Vec<f32>,
    pub metadata: ChunkMetadata,
}

pub struct ChunkMetadata {
    pub document_id: String,
    pub chunk_index: usize,
    pub content: String,
    pub source_file: String,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
}
```

### Indexing Configuration
- **Vector Dimension**: 768 (BGE-base)
- **Distance Metric**: Cosine similarity
- **Index Type**: HNSW (Hierarchical Navigable Small World)
- **Quantization**: Scalar quantization for efficiency

## 5. Retrieval Process

### Query Processing
1. **Query Embedding**
   - Generate embedding for user query
   - Apply same normalization as documents

2. **Similarity Search**
   - Search Qdrant for similar vectors
   - Apply filters (document type, date range)
   - Return top-k results (default: 10)

3. **Reranking** (Optional)
   - Use reranking model for better relevance
   - Cross-encoder for query-document pairs
   - Final ranking based on semantic similarity

### Search Parameters
```rust
pub struct SearchConfig {
    pub top_k: usize,           // Number of results
    pub score_threshold: f32,   // Minimum similarity score
    pub filters: Vec<Filter>,   // Metadata filters
    pub rerank: bool,          // Enable reranking
}
```

## 6. Error Handling & Resilience

### Retry Logic
- **Exponential Backoff**: 500ms → 1s → 2s → 4s
- **Max Retries**: 3 attempts
- **Circuit Breaker**: Fail fast after consecutive failures
- **Graceful Degradation**: Fallback to cached results

### Monitoring
- Embedding service health checks
- Processing latency metrics
- Error rate tracking
- Queue depth monitoring

## 7. Performance Optimization

### Batch Processing
- Group similar documents for embedding
- Parallel processing of independent chunks
- Streaming for large documents

### Caching
- Embedding cache for repeated content
- Query result caching
- Metadata caching in Redis

### Resource Management
- Connection pooling for databases
- Memory-efficient chunking
- Async processing pipelines

## 8. Configuration

### Environment Variables
```bash
# Embedding Service
EMBEDDING_SERVICE_URL=http://localhost:8082
EMBEDDING_MODEL=bge-base-en-v1.5
FUSION_STRATEGY=attention_weighted

# Vector Database
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=your_api_key
COLLECTION_NAME=conhub_embeddings

# Processing
MAX_CHUNK_SIZE=512
CHUNK_OVERLAP=50
BATCH_SIZE=32
MAX_CONCURRENT_REQUESTS=10
```

### Production Settings
```rust
pub struct ProductionConfig {
    pub embedding_cache_size: usize,    // 10000 entries
    pub connection_pool_size: u32,      // 20 connections
    pub request_timeout: Duration,      // 30 seconds
    pub max_file_size: usize,          // 50MB
    pub concurrent_limit: usize,        // 100 requests
}
```

## 9. Testing Strategy

### Unit Tests
- Individual component testing
- Mock external services
- Error condition handling

### Integration Tests
- End-to-end pipeline testing
- Service communication validation
- Performance benchmarking

### Load Testing
- Concurrent request handling
- Large document processing
- Memory usage profiling

## 10. Deployment

### Docker Compose
```yaml
services:
  embedding-service:
    build: ./embedding
    ports:
      - "8082:8082"
    environment:
      - RUST_LOG=info
      
  indexing-service:
    build: ./indexers
    ports:
      - "8081:8081"
    environment:
      - EMBEDDING_SERVICE_URL=http://embedding-service:8082
      
  qdrant:
    image: qdrant/qdrant
    ports:
      - "6333:6333"
    volumes:
      - qdrant_data:/qdrant/storage
```

### Health Checks
- Service availability monitoring
- Database connectivity checks
- Model loading verification
- Performance metrics collection

## 11. Future Enhancements

### Advanced Features
- **Hybrid Search**: Combine vector and keyword search
- **Multi-language Support**: Language-specific embeddings
- **Real-time Updates**: Incremental indexing
- **Federated Search**: Multiple vector stores

### Model Improvements
- **Fine-tuned Models**: Domain-specific embeddings
- **Larger Context**: Support for longer documents
- **Multimodal Fusion**: Better cross-modal understanding
- **Adaptive Chunking**: Dynamic chunk size optimization

## 12. Security Considerations

### Data Protection
- Encryption at rest and in transit
- Access control and authentication
- Audit logging for sensitive operations
- Data retention policies

### API Security
- Rate limiting and throttling
- Input validation and sanitization
- CORS configuration
- API key management

This RAG pipeline design provides a robust, scalable foundation for ConHub's retrieval-augmented generation capabilities, ensuring high-quality embeddings, efficient storage, and fast retrieval for enhanced user experiences.