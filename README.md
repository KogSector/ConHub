# ConHub

Unify your repositories, docs, and URLs with AI for better development workflows.

## Overview

ConHub is a comprehensive platform that connects multiple knowledge sources (repositories, documents, URLs) with AI agents for enhanced development context. It provides semantic search, code indexing, document processing, and AI-powered Q&A across all your connected data sources.

## Architecture

ConHub consists of 5 integrated services:

- **Frontend** (Next.js) - User interface and dashboard
- **Backend** (Rust) - Core API and authentication
- **Lexor** (Rust) - High-performance code indexing and search
- **LangChain Service** (Node.js) - AI-powered data source integration
- **Haystack Service** (Python) - Document processing and Q&A

## Features

### 🔗 Data Source Integration
- **GitHub** - Repositories, issues, pull requests, README files
- **BitBucket** - Repositories, issues, pull requests
- **Google Drive** - Documents, spreadsheets, presentations
- **Notion** - Pages, databases, subpages
- **URLs** - Web crawling with content extraction
- **Local Files** - Upload and index documents

### 🤖 AI Agent Integration
- **GitHub Copilot** - Code assistance and suggestions
- **Amazon Q** - AWS-focused development help
- **Custom Agents** - Extensible agent framework

### 🔍 Advanced Search & Indexing
- **Semantic Search** - Natural language queries across all sources
- **Code Search** - Symbol-aware code search with Lexor
- **Document Q&A** - Ask questions about your documents
- **Real-time Sync** - Keep data up-to-date automatically

### 📊 Comprehensive Analytics
- **Indexing Progress** - Track document processing status
- **Search Analytics** - Monitor query performance
- **Agent Usage** - AI interaction metrics

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

# Auth0 Configuration
AUTH0_SECRET=your_auth0_secret
AUTH0_CLIENT_ID=your_client_id
AUTH0_CLIENT_SECRET=your_client_secret
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