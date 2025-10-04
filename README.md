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

-   **GitHub Copilot & Amazon Q**: Seamless integration with major AI assistants for enhanced code assistance with full repository context.
-   **Extensible Framework**: Built for future integration with OpenAI GPT models, Anthropic Claude, and agentic IDEs like Cursor.
-   **Unified Context**: Provide AI agents with complete, real-time context across all connected repositories, documents, and URLs.

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

### Connect AI Agents
```bash
# Connect GitHub Copilot
curl -X POST http://localhost:8001/ai/agents/connect \
  -F "agent_type=github_copilot" \
  -F 'credentials={"github_token": "your_github_token"}'

# Connect Amazon Q
curl -X POST http://localhost:8001/ai/agents/connect \
  -F "agent_type=amazon_q" \
  -F 'credentials={"access_key_id": "your_key", "secret_access_key": "your_secret", "region": "us-east-1"}'
```

### Query via AI Agent
```bash
curl -X POST http://localhost:3001/api/agents/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "How does authentication work?",
    "agent_type": "github_copilot",
    "include_code": true,
    "include_documents": true,
    "max_results": 10
  }'
```

## Troubleshooting

### Common Issues

1.  **Port Conflicts**: Check if ports 3000, 3001, 3002, and 8001 are available. Use `npm run stop` to terminate all services.
2.  **Python Dependencies**: Ensure all packages are installed with `pip install -r requirements.txt`.
3.  **Rust Compilation**: If you encounter build issues, run `cargo clean` and then `cargo build --release`.

### Service Health Checks
```bash
npm run status
# Or check services individually
curl http://localhost:3001/health   # Backend
curl http://localhost:3002/health   # Lexor
curl http://localhost:8001/health   # AI Service
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details.
