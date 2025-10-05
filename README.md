# ConHub

Supercharge Your Development with AI - Unify repositories, docs, and URLs with AI agents for enhanced development workflows.

## Overview

ConHub is a comprehensive AI-powered platform that connects multiple knowledge sources (repositories, documents, URLs) with AI agents through the Model Context Protocol (MCP) for enhanced development context. It provides semantic search, code indexing, document processing, AI-powered Q&A, and seamless AI assistant integration across all your connected data sources.

## Architecture

ConHub consists of 4 integrated services working together to deliver a complete AI development platform:

-   **Frontend** (Next.js, Port 3000) - Modern user interface and dashboard.
-   **Backend** (Rust, Port 3001) - High-performance API, authentication, data connectors, and MCP server.
-   **Lexor** (Rust, Port 3002) - Lightning-fast code indexing and semantic search.
-   **AI Service** (Python, Port 8001) - Unified AI agents, vector search, and document processing.

## Key Features

### 🔗 Data Source Integration

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

## Quick Start

### Prerequisites

-   **Node.js** (v18 or higher)
-   **Rust** and **Cargo**
-   **Python** (3.9+)
-   **Git**

### Installation & Setup

1.  **Clone the repository:**
    ```bash
    git clone <your-repo-url>
    cd ConHub
    ```

2.  **Install dependencies:**
    ```bash
    # Install all JavaScript/TypeScript dependencies
    npm install
    
    # Install Python dependencies
    pip install -r requirements.txt
    ```

3.  **Configure environment:**
    ```bash
    cp .env.example .env
    # Edit .env with your API keys and configuration
    ```

4.  **Build services:**
    ```bash
    # Build the Rust backend and lexor services
    cargo build --release
    ```

5.  **Run the application:**
    ```bash
    # Start all services
    npm start
    ```
    Your application will be available at `http://localhost:3000`.

## Indexing Pipeline Architecture

ConHub features a sophisticated **3-phase indexing pipeline** that automatically processes and indexes all connected data sources for optimal AI agent performance.

1.  **Data Source Connection**: Securely connect to repositories (GitHub, GitLab, BitBucket) and document sources (Google Drive, Dropbox, etc.).
2.  **Automatic Indexing**: Once a source is connected, the backend automatically triggers the appropriate indexing service.
    -   **Lexor Service**: Indexes code files, extracts symbols, and builds a searchable code graph.
    -   **AI Service**: Processes documents (READMEs, PDFs, etc.) and web content for semantic search.
3.  **Context Aggregation**: A unified API endpoint (`/api/agents/query`) fetches context from both Lexor and the AI Service, providing a complete picture for AI agents.

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

### Connect Document Sources
```bash
# Upload a local file
curl -X POST http://localhost:8001/sources/local-files -F "files=@your-document.txt"

# Connect Dropbox
curl -X POST http://localhost:8001/sources/dropbox -F "access_token=your_token" -F "folder_path=/Documents"
```

### Index a URL
```bash
curl -X POST http://localhost:8001/index/urls \
  -F 'urls=["https://httpbin.org/html"]' \
  -F 'config={"crawl_depth": 2}'
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
- **Key Pages**: `/dashboard`, `/agents`, `/amazon-q`, `/cursor`, `/cline`

### Backend (Port 3001)
- **Technology**: Rust, Actix-web
- **Features**: API endpoints, authentication, AI agent management
- **Key Endpoints**: `/health`, `/api/agents/*`

### Lexor (Port 3002)
- **Technology**: Rust, Tantivy search engine
- **Features**: Code indexing, semantic search, cross-references

### AI Service (Port 8001)
- **Technology**: Python, FastAPI, Haystack
- **Features**: Document processing, vector search, AI integrations

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

### Essential Commands
```bash
npm start                    # Start all services
npm run stop                 # Stop all services
npm run status              # Check service status
npm run test:services       # Test service health
npm run test:ai-agents      # Test AI agent integration
```

### Development Mode
```bash
npm run dev:frontend        # Frontend development (port 3000)
npm run dev:backend         # Backend development (port 3001)
npm run dev:lexor          # Lexor development (port 3002)
npm run dev:ai             # AI service development (port 8001)
```

### Build Commands
```bash
npm run build              # Build all services
npm run build:frontend     # Build frontend only
npm run build:backend      # Build backend only
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

3. **Python Dependencies**
   ```bash
   pip install -r requirements.txt --force-reinstall
   ```

4. **Service Health Checks**
   ```bash
   curl http://localhost:3001/health   # Backend
   curl http://localhost:3002/health   # Lexor
   curl http://localhost:8001/health   # AI Service
   ```

### Logs and Debugging
- **Backend logs**: `logs/conhub.log`
- **AI Service logs**: `ai-service/logs/`
- **Frontend logs**: Browser console + Next.js console

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │    Backend      │    │     Lexor       │
│   (Next.js)     │◄──►│    (Rust)       │◄──►│   (Rust)        │
│   Port 3000     │    │   Port 3001     │    │   Port 3002     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   AI Service    │
                    │   (Python)      │
                    │   Port 8001     │
                    └─────────────────┘
                             │
                    ┌─────────────────┐
                    │   AI Agents     │
                    │ • Amazon Q      │
                    │ • GitHub Copilot│
                    │ • Cline         │
                    │ • Cursor IDE    │
                    └─────────────────┘
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details.
