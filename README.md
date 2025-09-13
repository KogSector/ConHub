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
- ✅ **LangChain Service** on port 3002
- ✅ **Haystack Service** on port 8001
- ✅ **Auto-reload** on file changes
- ✅ **Cross-platform** (Windows, Mac, Linux)

**Individual Services** (if needed):
```bash
npm run dev:frontend    # Frontend only
npm run dev:backend     # Backend only
npm run dev:lexor       # Lexor service only
npm run dev:langchain   # LangChain service only
npm run dev:haystack    # Haystack service only
```

## Services

### Frontend (Port 3000)
**Next.js 14 + React 18 + TypeScript**
- Modern React application with App Router
- Tailwind CSS + shadcn/ui components
- Auth0 authentication
- Real-time dashboard and search interface

**Features:**
- Multi-source data management
- Semantic search interface
- AI chat and Q&A
- Repository and document browser
- User authentication and settings

### Backend (Port 3001)
**Rust + Actix Web**
- High-performance API server
- Authentication and authorization
- CORS-enabled REST API
- Integration with all services

**API Endpoints:**
- Authentication and user management
- Service orchestration
- Health checks and monitoring

### Lexor Service (Port 3002)
**Rust + Actix Web + Tantivy**
- High-performance code indexing
- Full-text search across codebases
- Symbol cross-referencing
- Git history analysis
- Multi-language support

**Features:**
- Fast source code indexing
- Advanced search capabilities
- Symbol definitions and references
- Git integration and history tracking
- Tree-sitter syntax highlighting

**API Endpoints:**
- `GET /health` - Health check
- `POST /api/search` - Search code
- `GET /api/projects` - List projects
- `POST /api/projects` - Add project
- `POST /api/projects/{id}/index` - Index project

### LangChain Service (Port 3002)
**Node.js + TypeScript + LangChain**
- Multi-source data indexing
- Vector embeddings and semantic search
- AI-powered Q&A
- Real-time data synchronization

**Supported Data Sources:**
- GitHub repositories (public/private)
- Google Drive documents
- Notion pages and databases
- Web pages and URLs
- Local file uploads

**API Endpoints:**
- `GET /api/data-sources` - List connected sources
- `POST /api/data-sources/connect` - Connect new source
- `POST /api/indexing/repository` - Index GitHub repo
- `POST /api/search/universal` - Universal search
- `POST /api/search/code` - Code-specific search

### Haystack Service (Port 8001)
**Python + FastAPI + Haystack**
- Document processing and indexing
- Semantic search with local models
- Question answering system
- File upload and processing

**Features:**
- PDF, Word, text file processing
- Local embedding models (no API keys needed)
- In-memory or Elasticsearch storage
- Question answering with context
- Batch document processing

**API Endpoints:**
- `POST /documents` - Index documents
- `POST /documents/upload` - Upload files
- `POST /search` - Semantic search
- `POST /ask` - Question answering
- `GET /stats` - Document statistics

## Configuration

### Environment Variables

All services use the single `.env` file at the root:

```bash
# Service URLs
NEXT_PUBLIC_API_URL=http://localhost:3001
LANGCHAIN_SERVICE_URL=http://localhost:3002
HAYSTACK_SERVICE_URL=http://localhost:8001

# Auth0 Configuration
AUTH0_SECRET=your-secret-here
AUTH0_CLIENT_ID=your-client-id
AUTH0_CLIENT_SECRET=your-client-secret

# AI Configuration
OPENAI_API_KEY=your-openai-key
QDRANT_URL=http://localhost:6333

# Data Source APIs
GITHUB_ACCESS_TOKEN=your-github-token
GOOGLE_DRIVE_CLIENT_ID=your-google-drive-id
NOTION_API_KEY=your-notion-key
```

### Auth0 Setup

1. Create an Auth0 account at https://auth0.com
2. Create a new application (Single Page Application)
3. Configure your application settings:
   - Allowed Callback URLs: `http://localhost:3000`
   - Allowed Logout URLs: `http://localhost:3000`
   - Allowed Web Origins: `http://localhost:3000`
4. Update `.env` with your Auth0 credentials

### Vector Database Setup

**Option 1: Qdrant (Recommended)**
```bash
docker run -p 6333:6333 qdrant/qdrant
```

**Option 2: Pinecone**
- Sign up at pinecone.io
- Create an index
- Add API key to `.env`

## Available Scripts

- `npm start` - Start all services
- `npm run dev` - Same as start (alias)
- `npm run build` - Build all services for production
- `npm run lint` - Lint all JavaScript/TypeScript code
- `npm run test` - Run tests for all services
- `npm run dev:frontend` - Frontend only
- `npm run dev:backend` - Backend only
- `npm run dev:lexor` - Lexor service only
- `npm run dev:langchain` - LangChain service only
- `npm run dev:haystack` - Haystack service only

## Project Structure

```
ConHub/
├── 📦 package.json        # Master JS/TS dependencies
├── 🦀 Cargo.toml          # Master Rust project
├── 🐍 requirements.txt    # Master Python dependencies
├── 🔐 .env                # Master environment config
├── ⚙️  Configuration files # All other master configs
├── frontend/              # Next.js application
├── backend/               # Rust API server
├── lexor/                 # Rust code indexing service
├── langchain-service/     # Node.js AI service
└── haystack-service/      # Python document service
```

## Tech Stack

- **Frontend**: Next.js 14, React 18, TypeScript, Tailwind CSS, shadcn/ui
- **Backend**: Rust, Actix Web
- **Lexor**: Rust, Actix Web, Tantivy, Tree-sitter
- **LangChain**: Node.js, TypeScript, LangChain, OpenAI
- **Haystack**: Python, FastAPI, Haystack, Transformers
- **Authentication**: Auth0
- **Vector Databases**: Qdrant, Pinecone
- **Storage**: In-memory, Elasticsearch

## Features

- ✅ **Multi-source connectivity** - Git repos, docs, URLs
- ✅ **AI agent integration** - LangChain + Haystack
- ✅ **RAG architecture** - Retrieval-Augmented Generation
- ✅ **Code indexing** - Fast search across codebases
- ✅ **Document processing** - PDFs, Word docs, text files
- ✅ **Semantic search** - Natural language queries
- ✅ **Question answering** - Direct answers from your data
- ✅ **Real-time sync** - Keep data up-to-date
- ✅ **Secure authentication** - Auth0 integration
- ✅ **Local models** - Run offline without API keys
- ✅ **Cross-platform** - Windows, Mac, Linux

## API Examples

### Connect GitHub Repository
```bash
curl -X POST http://localhost:3002/api/data-sources/connect \
  -H "Content-Type: application/json" \
  -d '{
    "type": "github",
    "credentials": {"accessToken": "your_token"},
    "config": {"repositories": ["owner/repo"]}
  }'
```

### Search Code
```bash
curl -X POST http://localhost:3002/api/search/code \
  -H "Content-Type: application/json" \
  -d '{"query": "authentication function", "limit": 10}'
```

### Upload Document
```bash
curl -X POST http://localhost:8001/documents/upload \
  -F "file=@document.pdf" \
  -F "metadata={\"source\": \"manual\"}"
```

### Ask Question
```bash
curl -X POST http://localhost:8001/ask \
  -H "Content-Type: application/json" \
  -d '{"query": "How does authentication work?", "top_k": 3}'
```

## Production Deployment

### Docker
Each service includes a Dockerfile for containerization.

### Build for Production
```bash
npm run build
cargo build --release
```

### Environment
- Set `NODE_ENV=production`
- Use production databases (Elasticsearch, PostgreSQL)
- Configure reverse proxy (nginx)
- Set up SSL certificates
- Monitor with logging and metrics

## Development Tips

1. **Start with local models** - No API keys needed for development
2. **Use in-memory storage** - Faster for development and testing
3. **Enable hot reload** - All services support auto-reload
4. **Check logs** - Each service provides detailed logging
5. **Use the unified .env** - All configuration in one place

## Troubleshooting

### Common Issues

1. **Port conflicts**: Check if ports 3000-3002, 8001 are available
2. **Missing dependencies**: Run `npm install` and `pip install -r requirements.txt`
3. **Auth0 errors**: Verify Auth0 configuration in `.env`
4. **Model downloads**: Ensure internet connection for first-time model downloads
5. **Memory issues**: Reduce batch sizes or use smaller models

### Service Health Checks
- Frontend: http://localhost:3000
- Backend: http://localhost:3001/health
- Lexor: http://localhost:3002/health
- LangChain: http://localhost:3002/health
- Haystack: http://localhost:8001/health

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details