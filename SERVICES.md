# ConHub Services Architecture

## 5 Core Services

### 1. Frontend (Next.js)
- **Port**: 3000
- **Technology**: Next.js 14, TypeScript, Tailwind CSS
- **Location**: `./frontend/`
- **Dockerfile**: ✅ `./frontend/Dockerfile`
- **Purpose**: User interface and dashboard

### 2. Backend (Rust)
- **Port**: 3001
- **Technology**: Rust, Actix-web
- **Location**: `./backend/`
- **Dockerfile**: ✅ `./backend/Dockerfile`
- **Purpose**: API endpoints, authentication, data connectors

### 3. Lexor (Rust)
- **Port**: 3002
- **Technology**: Rust, Tantivy search engine
- **Location**: `./lexor/`
- **Dockerfile**: ✅ `./lexor/Dockerfile`
- **Purpose**: Code indexing, semantic search, cross-references

### 4. Doc Search (Python)
- **Port**: 8001
- **Technology**: Python, FastAPI, Haystack
- **Location**: `./doc-search/`
- **Dockerfile**: ✅ `./doc-search/Dockerfile`
- **Purpose**: Document processing, vector search, semantic search

### 5. Langchain Service (TypeScript)
- **Port**: 8002
- **Technology**: TypeScript, Express, Langchain
- **Location**: `./langchain-service/`
- **Dockerfile**: ✅ `./langchain-service/Dockerfile`
- **Purpose**: AI agent orchestration, advanced language processing

## Startup Commands

```bash
# Start all services
npm start

# Stop all services
npm stop

# Check service status
npm run status

# Individual service commands
npm run dev:frontend    # Port 3000
npm run dev:backend     # Port 3001
npm run dev:lexor       # Port 3002
npm run dev:doc-search  # Port 8001
npm run dev:langchain   # Port 8002
```

## Docker Deployment

Each service can be deployed independently using their respective Dockerfiles:

```bash
# Build all services
docker-compose build

# Run all services
docker-compose up

# Individual service builds
docker build -t conhub-frontend ./frontend
docker build -t conhub-backend ./backend
docker build -t conhub-lexor ./lexor
docker build -t conhub-doc-search ./doc-search
docker build -t conhub-langchain ./langchain-service
```

## Service Dependencies

- **Frontend** → Backend, Langchain Service
- **Backend** → Lexor, Doc Search
- **Lexor** → Independent (code indexing)
- **Doc Search** → Independent (document processing)
- **Langchain Service** → Doc Search (for enhanced AI capabilities)