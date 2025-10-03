# ConHub

Supercharge Your Development with AI - Unify repositories, docs, and URLs with AI agents for enhanced development workflows.

## Overview

ConHub is a comprehensive AI-powered platform that connects multiple knowledge sources (repositories, documents, URLs) with AI agents through the Model Context Protocol (MCP) for enhanced development context. It provides semantic search, code indexing, document processing, AI-powered Q&A, and seamless AI assistant integration across all your connected data sources.

## Architecture

ConHub consists of 4 integrated services working together to deliver a complete AI development platform:

-   **Frontend** (Next.js) - Modern user interface and dashboard
-   **Backend** (Rust) - High-performance API, authentication, data connectors, and MCP server
-   **Lexor** (Rust) - Lightning-fast code indexing and semantic search
-   **AI Service** (Python) - Unified AI agents, vector search, and document processing

## Features

### 🔗 Data Source Integration

-   **Version Control Systems**: Connect repositories from GitHub, BitBucket, and more. Includes issues, pull requests, and README files.
-   **Cloud Drives**: Integrate with Google Drive for documents, spreadsheets, and presentations.
-   **Collaboration Tools**: Link Notion pages and databases.
-   **Web Content**: Crawl public URLs and documentation sites.
-   **Local Files**: Upload and index local documents.

### 🤖 AI Agent Integration

-   **Multi-Agent Support**: Connect with various AI assistants like GitHub Copilot, Amazon Q, OpenAI GPT models, and Anthropic Claude.
-   **Extensible Framework**: Add custom agents with MCP support for a unified experience.
-   **Unified Context**: Provide AI agents with complete, real-time context across all connected repositories, documents, and URLs.
-   **Smart Routing**: Intelligently route queries to the most appropriate agent based on context.

### 🔍 Advanced Search & Indexing

-   **Semantic Search**: Use natural language queries across all sources.
-   **Code Search**: Perform symbol-aware code searches with Lexor.
-   **Document Q&A**: Ask questions and get answers from your documents.
-   **Real-time Sync**: Keep all data sources up-to-date automatically.

### 📊 Comprehensive Analytics & Monitoring

-   **Indexing Progress**: Track document processing and sync status.
-   **Search Analytics**: Monitor query performance and relevance.
-   **Agent Usage**: View metrics on AI agent interactions and token consumption.
-   **MCP Monitoring**: Observe context usage and performance.

### 🔒 Enterprise Security

-   **Secure Authentication**: Robust authentication for users and services.
-   **Permission Management**: Fine-grained access control for data sources and agents.
-   **Rate Limiting**: Configurable API limits to protect resources.
-   **Audit Logging**: Track all major operations for security and compliance.

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

## Environment Variables

```bash
# Service URLs
NEXT_PUBLIC_API_URL=http://localhost:3001
AI_SERVICE_URL=http://localhost:8001

# AI Configuration
OPENAI_API_KEY=your_openai_key
GITHUB_ACCESS_TOKEN=your_github_token

# Vector Database
QDRANT_URL=http://localhost:6333

# Authentication Configuration
JWT_SECRET=your_jwt_secret_key
DATABASE_URL=sqlite:./conhub.db
```

## Troubleshooting

### Common Issues

1.  **Port conflicts**: Check if ports 3000, 3001, 3002, and 8001 are available.
2.  **Missing dependencies**: Run `npm install`, `pip install -r requirements.txt`, and ensure Rust is correctly installed.
3.  **API rate limits**: Use personal access tokens for external services like GitHub to ensure higher rate limits.

### Service Health Checks

```bash
curl http://localhost:3000          # Frontend
curl http://localhost:3001/health   # Backend
curl http://localhost:3002/health   # Lexor
curl http://localhost:8001/health   # AI Service
```

## Data Source Configuration

### GitHub
```json
{
  "type": "github",
  "credentials": {
    "accessToken": "your_github_token"
  },
  "config": {
    "repositories": ["owner/repo1", "owner/repo2"],
    "includeReadme": true,
    "includeCode": true,
    "fileExtensions": [".js", ".ts", ".py", ".md"]
  }
}
```

### BitBucket
```json
{
  "type": "bitbucket",
  "credentials": {
    "username": "your_username",
    "appPassword": "your_app_password"
  },
  "config": {
    "repositories": ["workspace/repo1", "workspace/repo2"],
    "includeReadme": true,
    "includeCode": true
  }
}
```

### Google Drive
```json
{
  "type": "google-drive",
  "credentials": {
    "clientId": "your_client_id",
    "clientSecret": "your_client_secret",
    "refreshToken": "your_refresh_token"
  },
  "config": {
    "folderIds": ["folder_id_1", "folder_id_2"],
    "includeShared": false,
    "fileTypes": [
      "application/vnd.google-apps.document",
      "application/vnd.google-apps.presentation"
    ]
  }
}
```

### Notion
```json
{
  "type": "notion",
  "credentials": {
    "apiKey": "your_notion_api_key"
  },
  "config": {
    "databaseIds": ["database_id_1"],
    "pageIds": ["page_id_1"],
    "includeSubpages": true
  }
}
```

### URLs
```json
{
  "type": "url",
  "config": {
    "urls": ["https://docs.example.com", "https://blog.example.com"],
    "crawlDepth": 2,
    "allowedDomains": ["example.com", "docs.example.com"]
  }
}
```

## API Examples

### Connect Data Source
```bash
curl -X POST http://localhost:3001/api/data-sources/connect \
  -H "Content-Type: application/json" \
  -d '{
    "type": "github",
    "credentials": {"accessToken": "your_token"},
    "config": {
      "name": "My GitHub Repos",
      "repositories": ["owner/repo"],
      "includeReadme": true,
      "includeCode": true
    }
  }'
```

### Query AI Agent
```bash
curl -X POST http://localhost:3001/api/ai-agents/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "How does authentication work in this codebase?",
    "includeContext": true
  }'
```

### Search Content
```bash
curl -X POST http://localhost:3001/api/search/universal \
  -H "Content-Type: application/json" \
  -d '{
    "query": "user authentication",
    "limit": 10
  }'
```

## Service Architecture

### Frontend (Port 3000)
- **Next.js 14** with App Router
- **React 18** with TypeScript
- **Tailwind CSS** + shadcn/ui components
- **Real-time dashboard** and search interface

### Backend (Port 3001)
- **Rust + Actix Web** for high performance
- **Authentication** and authorization
- **Native data connectors** (GitHub, Bitbucket, Google Drive, Notion, URLs)
- **Service orchestration** and MCP server

### Lexor Service (Port 3002)
- **Rust + Tantivy** for code indexing
- **Tree-sitter** syntax analysis
- **Git integration** and history tracking

### AI Service (Port 8001)
- **Python + FastAPI**
- **Unified AI agent management**
- **Vector embeddings** and semantic search
- **Document processing** and Q&A
- **Local embedding models**

## Model Context Protocol (MCP) Implementation

ConHub features a comprehensive **Model Context Protocol (MCP)** implementation that provides a standardized, scalable, and secure way to connect AI agents with various data sources and contextual information.

### MCP Key Features

- **Security & Authentication**: Multiple authentication methods, fine-grained access control, and audit logging.
- **Scalability & Performance**: Connection pooling, asynchronous operations, and resource caching.
- **Context Management**: Structured context types, automatic resource discovery, and relevance scoring.
- **Tool Integration**: Support for built-in and custom tools with schema validation.

---

## 🚀 GitHub and AI Agent Integration

ConHub features comprehensive GitHub integration, providing seamless access to repositories, organizations, and AI-powered development tools. This allows AI agents to have rich, contextual information from your source code.

### ✨ Features

#### GitHub Integration
- **Multi-Authentication Support**: Personal Access Tokens, GitHub App, and OAuth.
- **Repository Management**: Browse, search, and analyze repositories.
- **Organization Access**: Manage organization repositories and members.
- **Real-time Analytics**: Commits, issues, pull requests, and activity tracking.
- **Content Access**: Browse repository files and directories.

#### AI Agent Management
- **Usage Analytics**: Track AI agent usage and interactions across organizations.
- **Repository Control**: Configure which repositories are accessible to AI agents.
- **Activity Tracking**: View user activity and interactions with AI agents.

### 🔧 API Endpoints

#### GitHub Core API (`/api/github`)
```
GET    /user                                    # Current user info
GET    /repositories                            # User repositories
GET    /organizations                           # User organizations
GET    /organizations/:org                      # Organization details
GET    /organizations/:org/repositories         # Organization repositories
GET    /repositories/:owner/:repo/contents      # Repository content
GET    /repositories/:owner/:repo/commits       # Repository commits
GET    /repositories/:owner/:repo/issues        # Repository issues
GET    /repositories/:owner/:repo/pulls         # Repository pull requests
GET    /repositories/:owner/:repo/analytics     # Repository analytics
GET    /search/repositories                     # Search repositories
GET    /health                                  # GitHub integration health
```

### 🔐 Authentication Methods

#### 1. Personal Access Token (PAT)
```typescript
// Required scopes: repo, read:org
const headers = {
  'Authorization': `Bearer ${personalAccessToken}`
};
```

#### 2. GitHub App Authentication
```typescript
// Environment variables required:
// GITHUB_APP_ID, GITHUB_APP_PRIVATE_KEY
const appAuth = getGitHubAppAuth();
const octokit = await appAuth.getInstallationOctokit(installationId);
```

#### 3. OAuth Flow
```typescript
// Environment variables required:
// GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET
const oauthApp = new OAuthApp({
  clientId: process.env.GITHUB_CLIENT_ID,
  clientSecret: process.env.GITHUB_CLIENT_SECRET
});
```

### 🛠 Setup Instructions

#### 1. Environment Configuration
```bash
# GitHub App Configuration
GITHUB_APP_ID=your_app_id
GITHUB_APP_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----\n..."

# OAuth Configuration (optional)
GITHUB_CLIENT_ID=your_client_id
GITHUB_CLIENT_SECRET=your_client_secret

# Webhook Configuration (optional)
GITHUB_WEBHOOK_SECRET=your_webhook_secret
```

#### 2. GitHub App Setup
1. Create a GitHub App in your organization settings.
2. Generate a private key and save it securely.
3. Install the app in your organization.
4. Configure necessary permissions (e.g., read/write for contents, issues, pull requests; read for organization data).

### 🔄 MCP Integration

The GitHub integration is fully MCP-compatible, exposing tools and resources through the Model Context Protocol:

#### Available Tools
- `github.get_user`: Get authenticated user information.
- `github.get_repositories`: Retrieve user/organization repositories.
- `github.search_repositories`: Search GitHub repositories.
- `github.get_repository_content`: Access repository files.
- `auth.github_app`: GitHub App authentication.
- `auth.oauth`: OAuth authentication flow.

#### Resource Access
- `github://repositories`: Repository data access.
- `github://organizations`: Organization data access.
- `github://ai-agents`: AI agent management and analytics.

### 📈 Performance & Scalability

- **Connection Pooling**: Efficient HTTP connection management.
- **Rate Limiting**: Intelligent GitHub API rate limit handling.
- **Caching**: Strategic caching of frequently accessed data.
- **Pagination**: Efficient handling of large data sets.

### 🔒 Security Features

- **Token Encryption**: Secure storage of authentication tokens.
- **Permission Validation**: Strict permission checking for API calls.
- **Audit Logging**: Complete audit trail of all operations.
- **Secure Communication**: HTTPS-only API communication.
- **Input Validation**: Comprehensive input sanitization.

## Feature Toggles

ConHub uses feature toggles in `feature-toggles.json` to enable or disable functionality during development. This allows for flexible testing and deployment strategies.

## Scripts & Automation

The project includes a comprehensive set of scripts for managing services across different operating systems (Windows, Linux, macOS). Use `npm start`, `npm run stop`, and `npm run check:services` to manage the application stack.

## Logging & Monitoring System

ConHub has a detailed logging and monitoring system. Configure log levels, output formats, and performance monitoring through environment variables in the `.env` file. Logs are stored in the `logs/` directory.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details.