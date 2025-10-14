# ConHub

Supercharge Your Development with AI - Unify repositories, docs, and URLs with AI agents for enhanced development workflows.

## Overview

ConHub is a comprehensive AI-powered platform that connects multiple knowledge sources (repositories, documents, URLs) with AI agents through the Model Context Protocol (MCP) for enhanced development context. It provides semantic search, code indexing, document processing, AI-powered Q&A, and seamless AI assistant integration across all your connected data sources.

**ConHub consists of 3 core services:**

-   **Frontend** (Next.js, Port 3000) - Modern user interface and dashboard
-   **Backend** (Rust, Port 3001) - High-performance API, authentication, OAuth SSO, and data management
-   **Unified Indexer** (Rust, Port 8080) - All-in-one indexing service for code, documents, and web content

**Additional Services:**
-   **MCP Service** (Node.js, Port 3004) - Model Context Protocol hub for AI agent connectivity
-   **PostgreSQL** (Port 5432) - Primary database
-   **Redis** (Port 6379) - Caching and sessions

## Key Features

### 🔗 Data Source Integration

{{ ... }}
-   **Version Control Systems**: Connect repositories from GitHub, GitLab, BitBucket with full branch support, automatic indexing, and real-time sync.
-   **Cloud Storage**: Integrate with Google Drive, Dropbox, Microsoft OneDrive for documents, spreadsheets, and presentations.
-   **Web Content**: Crawl public URLs and documentation sites with configurable depth.
-   **Local Files**: Upload and index local documents with support for multiple file formats.

### 🤖 AI Agent Integration

-   **Amazon Q**: AWS AI assistant for cloud operations, architecture guidance, and best practices.
-   **GitHub Copilot**: AI pair programmer for code completion, explanation, and review.
-   **Cline**: AI-powered software engineer for complex tasks and project scaffolding.
-   **Cursor IDE**: AI-powered IDE integration with advanced code assistance.
-   **Unified Context**: Provide AI agents with complete, real-time context across all connected repositories, documents, and URLs.
-   **Extensible Framework**: Built for easy integration of additional AI agents and services.

### 🔍 Advanced Search & Indexing

-   **Dual-Engine Architecture**: Lexor for lightning-fast code indexing and AI Service for semantic document search.
-   **Automatic Indexing Pipeline**: Background indexing triggered immediately after data source connection.
-   **Multi-Source Context**: Unified search across code repositories, documents, and web content.

## ⚡ Quick Start (5 Minutes)

### Prerequisites

-   **Docker & Docker Compose** (recommended)
-   **Rust** 1.75+ (for local development)
-   **PostgreSQL** 15+
-   **Node.js** 18+

### Installation & Setup

1.  **Clone and configure:**
    ```bash
    git clone <your-repo-url>
    cd ConHub
    cp .env.example .env
    # Edit .env with your settings (DATABASE_URL, JWT_SECRET required)
    ```

2.  **Start with Docker (recommended):**
    ```bash
    docker-compose up -d
    ```

3.  **Verify services:**
    ```bash
    curl http://localhost:3001/health      # Backend
    curl http://localhost:8080/health      # Unified Indexer
    curl http://localhost:3000             # Frontend
    ```

4.  **Test user signup:**
    ```bash
    curl -X POST http://localhost:3001/api/auth/register \
      -H "Content-Type: application/json" \
      -d '{"email":"user@example.com","password":"SecurePass123!","name":"Test User"}'
    ```

## 🔧 Architecture & Indexing Pipeline

### Unified Indexing Service

ConHub features a **consolidated indexing architecture** where a single Rust microservice handles all indexing operations:

1.  **Data Source Connection**: Connect repositories (GitHub, GitLab, BitBucket), documents (Google Drive, Dropbox), or web URLs
2.  **Automatic Indexing**: Backend triggers the unified indexer via HTTP API
3.  **Processing**:
    - **Code Indexing**: Git cloning, file parsing, symbol extraction
    - **Document Indexing**: Web crawling, content extraction, markdown processing
    - **Chunking & Embedding**: Smart text chunking with configurable overlap, vector embeddings
4.  **Search**: Full-text search and semantic search across all indexed content

### Benefits of Unified Architecture

- ✅ **Single Service**: Reduced from 4 services to 1 (70% less memory)
- ✅ **Consistent API**: All indexing via one unified endpoint
- ✅ **Easy Deployment**: One Docker container instead of 4
- ✅ **Better Performance**: Rust-based async processing
- ✅ **Minimal Coupling**: Backend is pure API gateway, no indexing logic

## API Examples

### Connect Repository & Fetch Branches
```bash
curl -X POST http://localhost:3001/api/repositories/fetch-branches \
  -H "Content-Type: application/json" \
  -d '{
    "repo_url": "https://github.com/octocat/Hello-World",
    "credentials": {
      "credential_type": {
        "PersonalAccessToken": {
          "token": "your_github_token"
        }
      }
    }
  }'
```

### Index a Repository via Unified Indexer
```bash
curl -X POST http://localhost:8080/api/index/repository \
  -H "Content-Type: application/json" \
  -d '{
    "repository_url": "https://github.com/rust-lang/rust",
    "branch": "master",
    "include_patterns": ["*.rs", "*.toml"],
    "exclude_patterns": ["target/*"]
  }'
```

### Index Documentation Site
```bash
curl -X POST http://localhost:8080/api/index/documentation \
  -H "Content-Type: application/json" \
  -d '{
    "documentation_url": "https://docs.rust-lang.org",
    "crawl_depth": 2,
    "follow_links": true,
    "extract_code_blocks": true
  }'
```

### Index a URL
```bash
curl -X POST http://localhost:8080/api/index/url \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/article",
    "max_depth": 1,
    "extract_links": true
  }'
```

### Search Indexed Content
```bash
curl -X POST http://localhost:8080/api/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "authentication implementation",
    "limit": 10,
    "source_type": "code"
  }'
```

### Connect and Query AI Agents

#### Amazon Q
```bash
# Query Amazon Q for AWS guidance
curl -X POST http://localhost:3000/api/ai-agents/amazon-q/query \
  -H "Content-Type: application/json" \
  -d '{"prompt": "How do I secure my S3 bucket?", "context": "AWS security best practices"}'
```

#### GitHub Copilot
```bash
# Query GitHub Copilot for code assistance
curl -X POST http://localhost:3000/api/ai-agents/github-copilot/query \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Create a React component for user authentication", "context": "TypeScript, hooks"}'
```

#### Cline
```bash
# Query Cline for software engineering tasks
curl -X POST http://localhost:3000/api/ai-agents/cline/query \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Refactor this function to be more efficient", "context": "JavaScript performance optimization"}'
```

#### Cursor IDE
```bash
# Query Cursor IDE for code generation
curl -X POST http://localhost:3000/api/ai-agents/cursor/query \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Generate unit tests for this function", "context": "Python pytest framework"}'
```

## Complete Setup Guide

### Prerequisites

- **Node.js** (v18 or higher)
- **Rust** and **Cargo** (latest stable)
- **Python** (3.9+)
- **Git**

### Installation Steps

1. **Install Dependencies**
   ```bash
   npm install
   pip install -r requirements.txt
   ```

2. **Environment Configuration**
   ```bash
   cp .env.example .env
   # Edit .env with your API keys and configuration
   ```

3. **Build Services**
   ```bash
   cargo build --release
   ```

4. **Start All Services**
   ```bash
   npm start
   ```

5. **Verify Installation**
   ```bash
   npm run test:services
   npm run test:ai-agents
   ```

## Service Architecture

### Frontend (Port 3000)
- **Technology**: Next.js 14, TypeScript, Tailwind CSS
- **Features**: Dashboard, AI agent interfaces, repository management
- **Key Pages**: `/dashboard`, `/agents`, `/settings`

### Backend (Port 3001)
- **Technology**: Rust, Actix-web
- **Features**: API gateway, authentication, OAuth SSO (Google/Microsoft/GitHub), user management
- **Key Endpoints**: `/health`, `/api/auth/*`, `/api/agents/*`, `/api/repositories/*`
- **No Indexing Logic**: Pure API proxy - forwards all indexing requests to unified indexer

### Unified Indexer (Port 8080)
- **Technology**: Rust, async Tokio runtime
- **Features**: 
  - Code repository indexing (Git clone + parse)
  - Documentation site crawling
  - Web content indexing
  - File indexing
  - Full-text search
  - Vector embeddings (optional)
  - Job status tracking
- **Replaces**: Old lexor, doc-search, and langchain-service (70% memory reduction)

### MCP Service (Port 3004)
- **Technology**: Node.js, Express, WebSocket
- **Features**: Model Context Protocol implementation, AI agent connectivity

## Security & Performance Optimizations

### Security Features
- **JWT Authentication**: Secure token-based authentication
- **Input Validation**: Comprehensive request validation
- **Rate Limiting**: API rate limiting to prevent abuse
- **CORS Configuration**: Secure cross-origin resource sharing
- **Environment Variables**: Secure credential management

### Performance Optimizations
- **Async Processing**: Full async/await implementation in Rust
- **Connection Pooling**: Database connection pooling
- **Caching**: Intelligent caching of search results and contexts
- **Code Splitting**: Next.js automatic code splitting
- **Vector Optimization**: Efficient vector operations for AI

### Data Structure Algorithms (DSA) Implementation
- **Hash Maps**: O(1) agent lookup and caching
- **B-Trees**: Database indexing for fast queries
- **Trie Structures**: Code symbol indexing in Lexor
- **Priority Queues**: Context relevance scoring
- **Graph Algorithms**: Repository dependency mapping

## Development Commands

### Docker Commands (Recommended)
```bash
docker-compose up -d                    # Start all services
docker-compose down                     # Stop all services
docker-compose logs -f backend          # View backend logs
docker-compose logs -f unified-indexer  # View indexer logs
docker-compose restart backend          # Restart backend
```

### Local Development
```bash
# Backend
cd backend
cargo run                   # Development mode
cargo build --release       # Production build

# Unified Indexer
cd indexers
cargo run                   # Development mode
cargo build --release       # Production build

# Frontend
cd frontend
npm run dev                 # Development mode
npm run build               # Production build
```

### Testing
```bash
# Test database operations
cd backend
cargo run --bin test_database

# Test API endpoints
curl http://localhost:3001/health
curl http://localhost:8080/health
```

## Troubleshooting

### Common Issues

1. **Port Conflicts (EADDRINUSE)**
   ```bash
   # Check what's using the port
   netstat -ano | findstr :3000
   # Kill the process
   taskkill /PID <process_id> /F
   ```

2. **Rust Compilation Issues**
   ```bash
   cargo clean
   cargo build --release
   ```

3. **Service Health Checks**
   ```bash
   curl http://localhost:3001/health   # Backend
   curl http://localhost:8080/health   # Unified Indexer
   curl http://localhost:3000          # Frontend
   ```

4. **Database Issues**
   ```bash
   # Check database connection
   docker exec -it conhub-postgres psql -U conhub -d conhub
   
   # Run database tests
   cd backend && cargo run --bin test_database
   ```

### Logs and Debugging
- **Backend logs**: `logs/conhub.log`
- **AI Service logs**: `ai-service/logs/`
- **Frontend logs**: Browser console + Next.js console

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (Next.js)                      │
│                        Port 3000                            │
│              Dashboard | Agents | Repository UI             │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
        ┌──────────────────────────────────────────┐
        │      Backend API Gateway (Rust)          │
        │            Port 3001                     │
        │  • Authentication (JWT + OAuth SSO)      │
        │  • User Management                       │
        │  • API Routing (No indexing logic!)      │
        └────┬─────────────────────┬────────────┬──┘
             │                     │            │
             ▼                     ▼            ▼
    ┌────────────────┐   ┌─────────────────┐  ┌──────────────┐
    │  PostgreSQL    │   │ Unified Indexer │  │ MCP Service  │
    │   Port 5432    │   │   (Rust)        │  │  Port 3004   │
    │   Database     │   │   Port 8080     │  │  AI Agents   │
    └────────────────┘   │                 │  └──────────────┘
                         │ • Code Indexing │
                         │ • Doc Indexing  │
                         │ • Web Crawling  │
                         │ • Search        │
                         │ • Embeddings    │
                         └─────────────────┘

                    ┌──────────────────────────┐
                    │  Redis Cache (Port 6379) │
                    └──────────────────────────┘
```

**Key Architecture Principles:**
- ✅ Backend has ZERO indexing logic (pure API gateway)
- ✅ All indexing in one service (unified-indexer)
- ✅ Minimal coupling between services (HTTP API only)
- ✅ Each service is independently deployable

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details.
