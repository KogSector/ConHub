# ConHub Haystack Service

A Python-based document indexing and search service using Haystack for advanced Q&A and semantic search capabilities.

## Features

- **Document Indexing**: Index PDFs, Word docs, text files, and more
- **Semantic Search**: Find relevant documents using natural language queries
- **Question Answering**: Get direct answers from your document collection
- **File Upload**: Upload and index files directly via API
- **Multiple Storage**: Support for in-memory and Elasticsearch document stores
- **Local Models**: Run completely offline with local embeddings and reader models

## Quick Start

### Prerequisites

- Python 3.9+
- pip

### Installation

1. **Create virtual environment:**
   ```bash
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   ```

2. **Install dependencies:**
   ```bash
   pip install -r requirements.txt
   ```

3. **Set up environment:**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. **Run the service:**
   ```bash
   uvicorn app.main:app --host 0.0.0.0 --port 8001 --reload
   ```

The service will start on `http://localhost:8001`

## API Endpoints

### Health & Status
- `GET /health` - Health check
- `GET /stats` - Document statistics

### Document Management
- `POST /documents` - Index documents
- `POST /documents/upload` - Upload and index files

### Search & Q&A
- `POST /search` - Semantic search
- `POST /ask` - Question answering

## Example Usage

### Index Documents
```bash
curl -X POST http://localhost:8001/documents \\
  -H "Content-Type: application/json" \\
  -d '{
    "documents": [
      {
        "content": "Your document content here...",
        "meta": {
          "filename": "example.txt",
          "source": "manual"
        }
      }
    ]
  }'
```

### Upload File
```bash
curl -X POST http://localhost:8001/documents/upload \\
  -F "file=@document.pdf" \\
  -F "metadata={\"source\": \"uploaded\", \"category\": \"manual\"}"
```

### Search Documents
```bash
curl -X POST http://localhost:8001/search \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "How to configure authentication?",
    "top_k": 5
  }'
```

### Ask Question
```bash
curl -X POST http://localhost:8001/ask \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "What is the main purpose of this system?",
    "top_k": 3
  }'
```

## Configuration

Key environment variables in `.env`:

```bash
# Basic Configuration
ENVIRONMENT=development
PORT=8001

# Document Store
USE_IN_MEMORY_STORE=true  # or false for Elasticsearch

# Models (all free and local)
EMBEDDING_MODEL=sentence-transformers/all-MiniLM-L6-v2
READER_MODEL=deepset/roberta-base-squad2

# External Services
LANGCHAIN_SERVICE_URL=http://localhost:3001
```

## Storage Options

### In-Memory Store (Default)
- **Pros**: No setup required, fast for development
- **Cons**: Data lost on restart, limited scalability
- **Use for**: Development, testing, small datasets

### Elasticsearch Store
- **Pros**: Persistent, scalable, production-ready
- **Cons**: Requires Elasticsearch setup
- **Use for**: Production, large datasets

To use Elasticsearch:
```bash
# Run Elasticsearch with Docker
docker run -d -p 9200:9200 -p 9300:9300 -e "discovery.type=single-node" elasticsearch:7.17.0

# Update .env
USE_IN_MEMORY_STORE=false
ELASTICSEARCH_HOST=localhost
ELASTICSEARCH_PORT=9200
```

## Models

### Embedding Models
- **Default**: `sentence-transformers/all-MiniLM-L6-v2`
  - Fast, good quality, runs locally
  - No API key required
- **Alternative**: OpenAI embeddings (requires API key)

### Reader Models (for Q&A)
- **Default**: `deepset/roberta-base-squad2`
  - Good performance, runs locally
  - No API key required

## Integration with ConHub

This service integrates with the ConHub ecosystem:

1. **LangChain Service** calls Haystack for document-heavy operations
2. **Frontend** can display Haystack search results
3. **Rust Backend** can proxy requests to Haystack

### Integration Flow
```
User uploads document → Frontend → LangChain Service → Haystack Service
User asks question → Frontend → LangChain Service → Haystack Service → Answer
```

## Docker Deployment

### Build and run with Docker:
```bash
docker build -t conhub-haystack .
docker run -p 8001:8001 conhub-haystack
```

### With Docker Compose:
```yaml
version: '3.8'
services:
  haystack:
    build: .
    ports:
      - "8001:8001"
    environment:
      - ENVIRONMENT=production
      - USE_IN_MEMORY_STORE=true
    volumes:
      - ./logs:/app/logs
```

## Development

### Project Structure
```
app/
├── main.py              # FastAPI application
├── core/
│   ├── config.py        # Configuration management
│   └── haystack_manager.py  # Haystack components
└── models/
    └── schemas.py       # Pydantic models
```

### Adding New Document Loaders
1. Install required dependencies
2. Update `haystack_manager.py`
3. Add new endpoint in `main.py`

### Testing
```bash
# Install test dependencies
pip install pytest pytest-asyncio

# Run tests
pytest
```

## Performance Tips

1. **Use local models** for development (no API costs)
2. **Elasticsearch** for production workloads
3. **Batch indexing** for large document sets
4. **Filters** to narrow search scope
5. **Appropriate chunk sizes** for your content type

## Troubleshooting

### Common Issues

1. **Model download fails**:
   - Ensure internet connection
   - Check available disk space
   - Try smaller models first

2. **Out of memory**:
   - Reduce batch sizes
   - Use smaller embedding models
   - Increase system RAM

3. **Elasticsearch connection**:
   - Verify Elasticsearch is running
   - Check connection settings
   - Ensure index permissions

### Logs
Check logs in:
- Console output (development)
- `/app/logs/` (Docker)
- Application logs for detailed error info

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## License

MIT License - see LICENSE file for details
