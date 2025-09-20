# ConHub

Supercharge Your Development with AI - Unify repositories, docs, and URLs with AI agents for enhanced development workflows.

## Overview

ConHub is a comprehensive AI-powered platform that connects multiple knowledge sources (repositories, documents, URLs) with AI agents through the Model Context Protocol (MCP) for enhanced development context. It provides semantic search, code indexing, document processing, AI-powered Q&A, and seamless GitHub Copilot integration across all your connected data sources.

## Architecture

ConHub consists of 5 integrated services working together to deliver a complete AI development platform:

- **Frontend** (Next.js) - Modern user interface and dashboard
- **Backend** (Rust) - High-performance API, authentication, and MCP server
- **Lexor** (Rust) - Lightning-fast code indexing and semantic search
- **LangChain Service** (Node.js) - AI-powered data source integration and processing
- **Haystack Service** (Python) - Advanced document processing and Q&A capabilities

## Features

### 🔗 Data Source Integration
- **GitHub** - Repositories, issues, pull requests, README files
- **BitBucket** - Repositories, issues, pull requests
- **Google Drive** - Documents, spreadsheets, presentations
- **Notion** - Pages, databases, subpages
- **URLs** - Web crawling with content extraction
- **Local Files** - Upload and index documents

### 🤖 AI Agent Integration
- **GitHub Copilot** - Enhanced code assistance with full MCP integration
- **Amazon Q** - AWS-focused development help
- **OpenAI GPT Models** - Chat and code completion
- **Anthropic Claude** - Advanced reasoning and analysis
- **Custom Agents** - Extensible agent framework with MCP support

### 🔍 Advanced Search & Indexing
- **Semantic Search** - Natural language queries across all sources
- **Code Search** - Symbol-aware code search with Lexor
- **Document Q&A** - Ask questions about your documents
- **Real-time Sync** - Keep data up-to-date automatically
- **MCP Protocol** - Standardized context sharing between AI systems

### 📊 Comprehensive Analytics
- **Indexing Progress** - Track document processing status
- **Search Analytics** - Monitor query performance
- **Agent Usage** - AI interaction metrics
- **MCP Monitoring** - Context usage and performance tracking

### 🔒 Enterprise Security
- **Multi-factor Authentication** - Secure access control
- **Permission Management** - Fine-grained resource access
- **Rate Limiting** - Configurable API limits
- **Audit Logging** - Complete operation tracking
- **TLS Encryption** - Secure communication channels

## Quick Start

### Prerequisites
- **Node.js** (v18 or higher)
- **Rust** and **Cargo**
- **Python** (3.9+)
- **Git**

### Installation & Setup

1. **Clone the repository:**
   ```bash
   git clone <your-repo-url>
   cd ConHub
   ```

2. **Install dependencies:**
   ```bash
   # Install all JavaScript/TypeScript dependencies
   npm install
   
   # Install Python dependencies
   pip install -r requirements.txt
   ```

3. **Configure environment:**
   ```bash
   cp .env.example .env
   # Edit .env with your API keys and configuration
   ```

### Development

**🚀 Start ConHub (One Command)**

```bash
npm start
```

This automatically starts all services:
- ✅ **Frontend** on port 3000
- ✅ **Backend API** on port 3001
- ✅ **Lexor Service** on port 3002
- ✅ **LangChain Service** on port 3003
- ✅ **Haystack Service** on port 8001
- ✅ **Auto-reload** on file changes
- ✅ **Cross-platform** (Windows, Mac, Linux)

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
curl -X POST http://localhost:3003/api/data-sources/connect \
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
curl -X POST http://localhost:3003/api/ai-agents/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "How does authentication work in this codebase?",
    "includeContext": true
  }'
```

### Search Content
```bash
curl -X POST http://localhost:3003/api/search/universal \
  -H "Content-Type: application/json" \
  -d '{
    "query": "user authentication",
    "limit": 10
  }'
```

## Environment Variables

```bash
# Service URLs
NEXT_PUBLIC_API_URL=http://localhost:3001
LANGCHAIN_SERVICE_URL=http://localhost:3003
HAYSTACK_SERVICE_URL=http://localhost:8001

# AI Configuration
OPENAI_API_KEY=your_openai_key
GITHUB_ACCESS_TOKEN=your_github_token

# Vector Database
QDRANT_URL=http://localhost:6333
PINECONE_API_KEY=your_pinecone_key

# Authentication Configuration
JWT_SECRET=your_jwt_secret_key
DATABASE_URL=sqlite:./conhub.db
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
- **Service orchestration**

### Lexor Service (Port 3002)
- **Rust + Tantivy** for code indexing
- **Tree-sitter** syntax analysis
- **Git integration** and history tracking

### LangChain Service (Port 3003)
- **Node.js + TypeScript**
- **Multi-source data connectors**
- **AI agent integration**
- **Vector embeddings** and semantic search

### Haystack Service (Port 8001)
- **Python + FastAPI**
- **Document processing** and Q&A
- **Local embedding models**

## Supported File Types

### Code Files
- JavaScript/TypeScript (`.js`, `.ts`, `.jsx`, `.tsx`)
- Python (`.py`)
- Rust (`.rs`)
- Java (`.java`)
- C/C++ (`.c`, `.cpp`, `.h`)
- Go (`.go`)
- And many more...

### Documents
- Markdown (`.md`)
- Text files (`.txt`)
- PDFs (`.pdf`)
- Word documents (`.docx`)
- Google Docs, Sheets, Slides
- Notion pages and databases

### Web Content
- HTML pages
- JSON APIs
- RSS feeds
- Documentation sites

## Development Tips

1. **Start with GitHub/BitBucket** - Easiest to set up and test
2. **Use local models** - No API keys needed for basic functionality
3. **Monitor logs** - Each service provides detailed logging
4. **Check health endpoints** - Verify services are running properly
5. **Use the unified .env** - All configuration in one place

## Troubleshooting

### Common Issues

1. **Port conflicts**: Check if ports 3000-3003, 8001 are available
2. **Missing dependencies**: Run `npm install` and `pip install -r requirements.txt`
3. **API rate limits**: Use personal access tokens for higher limits
4. **Memory issues**: Reduce batch sizes or use smaller models

### Service Health Checks
```bash
curl http://localhost:3000          # Frontend
curl http://localhost:3001/health   # Backend
curl http://localhost:3002/health   # Lexor
curl http://localhost:3003/health   # LangChain
curl http://localhost:8001/health   # Haystack
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details

---

## Model Context Protocol (MCP) Implementation

ConHub features a comprehensive **Model Context Protocol (MCP)** implementation that provides a standardized, scalable, and secure way to connect AI agents with various data sources and contextual information.

### MCP Architecture Overview

#### Core Components

1. **MCP Models** (`backend/src/models/mcp.rs`)
   - Complete type definitions for MCP protocol
   - Resource, Context, Tool, and Server models
   - JSON-RPC message structures
   - Comprehensive error handling

2. **MCP Server** (`backend/src/services/mcp_server.rs`)
   - Full-featured MCP server implementation
   - Context providers for repositories, documents, URLs
   - Built-in tools for search, analysis, and context creation
   - Security and authentication support

3. **MCP Client** (`backend/src/services/mcp_client.rs`)
   - Robust client for external MCP servers
   - Connection pooling and retry logic
   - Multiple authentication methods
   - Health monitoring and error recovery

### MCP Key Features

#### 🔒 Security & Authentication
- **Multiple Auth Methods**: API keys, Bearer tokens, OAuth2, Certificate-based
- **Resource Access Control**: Fine-grained permissions per resource
- **Rate Limiting**: Configurable limits per client
- **TLS Encryption**: Secure communication channels
- **Audit Logging**: Complete operation tracking

#### 🚀 Scalability & Performance
- **Connection Pooling**: Efficient client connection management
- **Async Operations**: Non-blocking I/O throughout
- **Resource Caching**: Intelligent caching for frequently accessed resources
- **Health Monitoring**: Automatic health checks and failover
- **Retry Logic**: Robust error recovery mechanisms

#### 🎯 Context Management
- **Structured Contexts**: Well-defined context types (Repository, Document, URL, etc.)
- **Resource Discovery**: Automatic resource indexing and discovery
- **Relevance Scoring**: Context ranking based on relevance
- **Metadata Support**: Rich annotations and metadata
- **Context Expiration**: Time-based context invalidation

#### 🛠️ Tool Integration
- **Built-in Tools**: Search, analysis, and context creation tools
- **Custom Tools**: Support for custom tool implementations
- **Tool Chaining**: Ability to chain multiple tools
- **Schema Validation**: Input/output schema enforcement
- **Error Handling**: Comprehensive tool error management

### MCP API Endpoints

#### Server Management
```http
POST /api/mcp/server/initialize    # Initialize MCP server
GET /api/mcp/server/status        # Check server status
POST /api/mcp/server/stop         # Stop MCP server
```

#### External Server Connections
```http
POST /api/mcp/external/connect                    # Connect to external MCP server
GET /api/mcp/external/connections                 # List active connections
DELETE /api/mcp/external/{connection_id}/disconnect # Disconnect external server
```

#### Context Operations
```http
POST /api/mcp/contexts           # Create new context
GET /api/mcp/contexts/{context_id} # Retrieve context
```

#### Resource Management
```http
GET /api/mcp/resources          # List available resources
POST /api/mcp/resources/read    # Read resource content
```

#### Tool Execution
```http
POST /api/mcp/tools/call        # Execute tool
GET /api/mcp/tools/list         # List available tools
```

### MCP Usage Examples

#### 1. Initialize MCP Server
```bash
curl -X POST http://localhost:3001/api/mcp/server/initialize \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ConHub MCP Server",
    "description": "Production MCP server for ConHub",
    "enable_auth": true,
    "rate_limit": 1000
  }'
```

#### 2. Connect to External MCP Server
```bash
curl -X POST http://localhost:3001/api/mcp/external/connect \
  -H "Content-Type: application/json" \
  -d '{
    "name": "External AI Service",
    "endpoint": "https://ai-service.example.com/mcp",
    "auth_method": "api_key",
    "credentials": {
      "api_key": "your-api-key-here"
    }
  }'
```

#### 3. Create Context for AI Agent
```bash
curl -X POST http://localhost:3001/api/mcp/contexts \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Development Context",
    "context_type": "repository",
    "resources": ["repo_1", "doc_api_guide"],
    "metadata": {
      "project": "ConHub",
      "scope": "backend_development"
    }
  }'
```

#### 4. Call Search Tool
```bash
curl -X POST http://localhost:3001/api/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "tool_name": "search",
    "arguments": {
      "query": "authentication implementation",
      "sources": ["repositories", "documents"]
    }
  }'
```

---

## 🚀 GitHub + Copilot Integration

ConHub now features comprehensive GitHub integration with full Copilot management capabilities, providing seamless access to repositories, organizations, and AI-powered development tools.

### ✨ Features

#### GitHub Integration
- **Multi-Authentication Support**: Personal Access Tokens, GitHub App, and OAuth
- **Repository Management**: Browse, search, and analyze repositories
- **Organization Access**: Manage organization repositories and members
- **Real-time Analytics**: Commits, issues, pull requests, and activity tracking
- **Content Access**: Browse repository files and directories

#### GitHub Copilot Management
- **Seat Management**: Add/remove users from Copilot access
- **Usage Analytics**: Track Copilot usage across organizations
- **Repository Control**: Enable/disable Copilot for specific repositories
- **Billing Insights**: Monitor Copilot costs and seat utilization
- **Activity Tracking**: View user activity and editor usage

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

#### GitHub Copilot API (`/api/copilot`)
```
GET    /billing/:org                            # Copilot billing info
GET    /seats/:org                              # Copilot seat assignments
POST   /seats/:org/add                          # Add Copilot seats
DELETE /seats/:org/remove                       # Remove Copilot seats
GET    /usage/:org                              # Copilot usage metrics
GET    /usage/enterprise/:enterprise            # Enterprise usage metrics
GET    /user/info                               # User Copilot info
GET    /repos/:org                              # Copilot enabled repositories
POST   /repos/:org/enable                       # Enable Copilot for repos
DELETE /repos/:org/disable                      # Disable Copilot for repos
```

### 🔐 Authentication Methods

#### 1. Personal Access Token (PAT)
```typescript
// Required scopes: repo, read:org, admin:org (for Copilot)
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

### 💼 Use Cases

#### For Developers
- **Repository Discovery**: Find and explore repositories across organizations
- **Code Analysis**: Access repository content and commit history
- **Issue Tracking**: Monitor issues and pull requests
- **Copilot Access**: Manage personal Copilot subscription

#### For Team Leads
- **Seat Management**: Add/remove team members from Copilot
- **Usage Monitoring**: Track team Copilot usage and productivity
- **Repository Oversight**: Enable Copilot for specific projects
- **Cost Management**: Monitor Copilot billing and optimize seats

#### For Organizations
- **Enterprise Analytics**: Comprehensive usage metrics across the organization
- **Policy Enforcement**: Control Copilot access by repository or team
- **Budget Tracking**: Monitor and forecast Copilot costs
- **Compliance**: Audit Copilot usage and access patterns

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
1. Create a GitHub App in your organization settings
2. Generate a private key and save it securely
3. Install the app in your organization
4. Configure permissions:
   - Repository permissions: Read/Write access to contents, issues, pull requests
   - Organization permissions: Read access to organization data
   - Account permissions: Read access to user data and Copilot billing

#### 3. Frontend Integration
```tsx
import GitHubCopilotDashboard from '@/components/github/copilot-dashboard';

export default function GitHubPage() {
  return <GitHubCopilotDashboard />;
}
```

### 📊 Frontend Features

#### Interactive Dashboard
- **Repository Browser**: Visual repository listing with search and filters
- **Organization Manager**: Organization selection and member management
- **Copilot Console**: Real-time usage tracking and seat management
- **Analytics View**: Charts and metrics for usage patterns

#### Smart Components
- **Enhanced Repository Dialog**: Multi-authentication connection flow
- **Copilot Seat Manager**: Drag-and-drop user assignment
- **Usage Analytics**: Interactive charts and reports
- **Permission Manager**: Fine-grained access control

### 🔄 MCP Integration

The GitHub + Copilot integration is fully MCP-compatible, exposing tools and resources through the Model Context Protocol:

#### Available Tools
- `github.get_user`: Get authenticated user information
- `github.get_repositories`: Retrieve user/organization repositories
- `github.search_repositories`: Search GitHub repositories
- `github.get_repository_content`: Access repository files
- `copilot.get_seats`: Manage Copilot seat assignments
- `copilot.get_usage_metrics`: Track usage analytics
- `auth.github_app`: GitHub App authentication
- `auth.oauth`: OAuth authentication flow

#### Resource Access
- `github://repositories`: Repository data access
- `github://organizations`: Organization data access
- `github://copilot`: Copilot management and analytics

### 📈 Performance & Scalability

#### Optimizations
- **Connection Pooling**: Efficient HTTP connection management
- **Rate Limiting**: Intelligent GitHub API rate limit handling
- **Caching**: Strategic caching of frequently accessed data
- **Pagination**: Efficient handling of large data sets

#### Monitoring
- **Health Checks**: Real-time service health monitoring
- **Error Tracking**: Comprehensive error logging and alerting
- **Performance Metrics**: API response time and throughput tracking
- **Usage Analytics**: User interaction and feature adoption metrics

### 🔒 Security Features

#### Authentication Security
- **Token Encryption**: Secure storage of authentication tokens
- **Permission Validation**: Strict permission checking for API calls
- **Audit Logging**: Complete audit trail of all operations
- **Rate Limiting**: Protection against API abuse

#### Data Protection
- **Secure Communication**: HTTPS-only API communication
- **Input Validation**: Comprehensive input sanitization
- **Error Handling**: Secure error messages without data leakage
- **Access Control**: Role-based access to sensitive operations

### Copilot Integration Features

- **Structured Context Access**: Rich, metadata-enhanced context instead of raw file content
- **Real-time Updates**: Live updates when repository or document changes occur
- **Tool Execution**: Execute ConHub tools for enhanced code analysis and generation
- **Scoped Access**: Workspace and permission-based context filtering
- **Session Management**: Secure, token-based authentication with configurable timeouts

### Copilot API Endpoints

#### Authentication & Session Management

**Initialize Session:**
```http
POST /api/copilot/session
Content-Type: application/json

{
  "user_id": "string",
  "workspace_id": "optional_string", 
  "auth_token": "github_token"
}
```

**Get Capabilities:**
```http
GET /api/copilot/capabilities
```

#### Context Access

**Request Context:**
```http
POST /api/copilot/context
Content-Type: application/json

{
  "session_id": "uuid",
  "context_type": "repository|document|url|data_source",
  "query": "optional search query",
  "workspace_path": "optional/path/filter",
  "file_patterns": ["*.rs", "*.ts"],
  "include_dependencies": true
}
```

#### Tool Execution

**Call Tool:**
```http
POST /api/copilot/tools/call
Content-Type: application/json

{
  "session_id": "uuid",
  "tool_name": "search|analyze|context_create|resource_read",
  "arguments": {
    "query": "function definition",
    "scope": "current_file"
  },
  "context_id": "optional_uuid"
}
```

### Copilot Security Features

- **GitHub Token Validation**: Secure authentication with GitHub tokens
- **Session-based Authentication**: Configurable session timeouts and limits
- **Permission-based Access**: Fine-grained resource access control
- **Rate Limiting**: Configurable limits for requests and context size

### Usage Examples

#### 1. Initialize Copilot Session
```bash
curl -X POST http://localhost:3001/api/copilot/session \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "github_user_123",
    "workspace_id": "conhub_workspace",
    "auth_token": "ghp_xxxxxxxxxxxxxxxxxxxx"
  }'
```

#### 2. Request Repository Context
```bash
curl -X POST http://localhost:3001/api/copilot/context \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "session_uuid",
    "context_type": "repository",
    "query": "authentication functions",
    "file_patterns": ["*.rs"],
    "include_dependencies": true
  }'
```

---

## Feature Toggles

ConHub uses feature toggles to enable/disable functionality during development, allowing for flexible deployment and testing strategies.

### Configuration

Feature toggles are configured in `feature-toggles.json` at the project root:

```json
{
  "Login": false
}
```

### Available Toggles

#### Login Toggle
- **Type**: Boolean
- **Default**: `false`
- **Description**: Controls authentication system

**When `Login` is `true`:**
- Full Auth0 authentication is enabled
- Users must sign in to access protected routes
- Real user data is used throughout the app

**When `Login` is `false`:**
- Authentication is bypassed
- Mock user data is provided
- All routes are accessible without authentication
- AuthGuard components allow access automatically

### Usage in Code

#### Checking Feature Status
```typescript
import { isLoginEnabled, isFeatureEnabled } from '@/lib/feature-toggles'

// Check specific features
const loginEnabled = isLoginEnabled()
const customFeature = isFeatureEnabled('CustomFeature')
```

#### Protecting Routes
```typescript
import { AuthGuard } from '@/components/auth/AuthGuard'

export default function ProtectedPage() {
  return (
    <AuthGuard>
      <div>This content is protected</div>
    </AuthGuard>
  )
}
```

#### Custom Fallback
```typescript
<AuthGuard fallback={<div>Please sign in</div>}>
  <ProtectedContent />
</AuthGuard>
```

### Development Workflow

1. **Working on non-auth features**: Set `"Login": false`
2. **Testing authentication**: Set `"Login": true`
3. **Production**: Always set `"Login": true`

### Mock Data

When login is disabled, the following mock user is provided:
```json
{
  "name": "Development User",
  "email": "dev@conhub.local",
  "picture": undefined
}
```

---

## Scripts & Automation

ConHub includes a comprehensive set of platform-specific scripts for managing services across different operating systems.

### Core Scripts

#### Platform Detection
- `check-platform.js` - Detects OS and runs appropriate platform-specific script

#### Windows Scripts (.ps1)
- `start.ps1` - Starts all ConHub services with port conflict detection
- `stop.ps1` - Stops all ConHub services 
- `check.ps1` - Health checks for all services with detailed endpoint testing
- `status.ps1` - Quick status check showing which services are running
- `test-settings.ps1` - Comprehensive API endpoint testing for settings

#### Linux/macOS Scripts (.sh)
- `start.sh` - Starts all ConHub services on Unix-like systems
- `stop.sh` - Stops all ConHub services on Unix-like systems  
- `check.sh` - Health checks for all services on Unix-like systems

#### Helper Scripts
- `monitor-services.js` - Background service to monitor startup and display URLs

### Script Usage

#### Start Services
```bash
npm start              # Cross-platform (uses check-platform.js)
npm run start:windows  # Windows specific
npm run start:linux    # Linux/macOS specific
```

#### Check Service Status
```bash
npm run status         # Quick status check
npm run check:services # Comprehensive health check
```

#### Stop Services
```bash
npm run stop:windows   # Windows
npm run stop:linux     # Linux/macOS
```

#### Test APIs
```bash
npm run test:settings  # Test settings API endpoints
```

### Service Ports
- **Frontend** (Next.js): 3000
- **Backend** (Rust): 3001  
- **Lexor** (Rust): 3002
- **LangChain Service**: 3003
- **Haystack Service**: 8001

---

## Database Setup & Management

ConHub includes automation scripts for database setup and management across different platforms.

### Database Scripts

#### Setup Database
Sets up the PostgreSQL database and applies the schema:

**Windows (PowerShell):**
```powershell
.\scripts\setup-database.ps1
```

**Linux/macOS (Bash):**
```bash
./scripts/setup-database.sh
```

#### Test Database Connection
Tests the PostgreSQL connection and verifies the database setup:

**Windows (PowerShell):**
```powershell
.\scripts\test-database.ps1
```

**Linux/macOS (Bash):**
```bash
./scripts/test-database.sh
```

### Database Setup Workflow

1. **First Time Setup:**
   ```bash
   # Windows
   .\scripts\setup-database.ps1
   
   # Linux/macOS
   ./scripts/setup-database.sh
   ```

2. **Test Connection:**
   ```bash
   # Windows
   .\scripts\test-database.ps1
   
   # Linux/macOS
   ./scripts/test-database.sh
   ```

3. **Update .env file** with your PostgreSQL password when prompted

4. **Start Services:**
   ```bash
   # Windows
   .\scripts\start.ps1
   
   # Linux/macOS
   ./scripts/start.sh
   ```

### Database Prerequisites
- PostgreSQL 17+ installed and running
- Node.js 18+ for frontend services
- Rust 1.70+ for backend services
- Git for version control

### Additional Testing Scripts

#### Test Settings
Tests the settings configuration:

**Windows (PowerShell):**
```powershell
.\scripts\test-settings.ps1
```

#### Test URLs
Tests URL connectivity:

**Windows (PowerShell):**
```powershell
.\scripts\test-urls.ps1
```

#### Check Platform
Checks platform compatibility:

**Node.js:**
```bash
node scripts/check-platform.js
```

### Script Notes

- All scripts should be run from the project root directory
- Ensure PostgreSQL is running before executing database scripts
- Scripts will prompt for credentials when needed
- Log files are created in the project root for debugging