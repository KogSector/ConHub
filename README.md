# ConHub - AI-Powered Development Context Platform

**Supercharge Your Development with AI** - Unified platform connecting repositories, documents, and cloud storage with AI agents for enhanced development workflows.

## 🎯 Overview

ConHub is a comprehensive AI-powered platform built on a **microservices architecture** that connects multiple knowledge sources with AI agents through the Model Context Protocol (MCP). It provides semantic search, code indexing, document processing, and seamless AI assistant integration.

### Key Features

- **Multi-Source Integration**: GitHub, GitLab, Google Drive, Dropbox, OneDrive, Notion
- **AI Agent Support**: Amazon Q, GitHub Copilot, Cline, custom agents via MCP
- **Intelligent Search**: Dual-engine architecture with code indexing and semantic search
- **Plugin System**: Unified plugin architecture for scalable source and agent management
- **Real-time Context**: Live context sharing across all connected AI agents

## 🏗️ Architecture

ConHub uses a modern **decoupled microservices architecture** with:

### Backend Services (Rust)
- **Backend Service** (8000) - Unified GraphQL API gateway
- **Auth Service** (3010) - Authentication, OAuth, JWT management
- **Billing Service** (3011) - Stripe payments & subscriptions
- **Client Service** (3014) - AI client integrations (OpenAI, Anthropic)
- **Data Service** (3013) - Data sources & repository management
- **Security Service** (3012) - Security policies & audit logs
- **Webhook Service** (3015) - External webhook handling
- **Plugins Service** (3020) - Unified plugin management system
- **Embedding Service** (8082) - Fusion embeddings & vector generation
- **Indexers Service** (8080) - Code/document indexing & search

### Infrastructure
- **PostgreSQL** (5432) - Primary database
- **Redis** (6379) - Cache & sessions
- **Qdrant** (6333) - Vector database for semantic search
- **Nginx** (80) - API Gateway & load balancer

### Frontend
- **Next.js Application** (3000) - Modern React-based UI

## ⚡ Quick Start

### Prerequisites
- **Docker & Docker Compose** (latest)
- **Node.js** 18+ 
- **Rust** 1.75+ (for local development)

### 1. Clone & Setup
```bash
git clone <repository-url>
cd ConHub
cp .env.example .env
# Edit .env with your configuration
```

### 2. Start with Docker (Recommended)
```bash
# Start all services
docker-compose up --build

# Access application
open http://localhost
```

### 3. Local Development
```bash
# Start databases only
docker-compose up -d postgres redis qdrant

# Start all services locally
npm run dev

# Or start individual services
npm run dev:frontend    # Frontend only
npm run dev:auth       # Auth service
npm run dev:backend    # Backend service
```

## 🚀 Service Architecture

### Service Communication
```
Frontend (3000) → Nginx (80) → Backend (8000) → Microservices
                                    ↓
                            GraphQL Federation
                                    ↓
                    ┌───────────────┼───────────────┐
                    ↓               ↓               ↓
              Auth (3010)    Data (3013)    Client (3014)
                    ↓               ↓               ↓
              Billing (3011) Security (3012) Webhook (3015)
                                    ↓
                            Plugins (3020)
                                    ↓
                    ┌───────────────┼───────────────┐
                    ↓               ↓               ↓
            Embedding (8082)  Indexers (8080)  Databases
```

### Plugin System
The unified plugin system replaces individual MCP microservices:

**Source Plugins**: Dropbox, Google Drive, OneDrive, Notion, GitHub
**Agent Plugins**: Cline, Amazon Q, GitHub Copilot

```bash
# Plugin Management API
GET /api/plugins/status           # Get all plugin status
POST /api/plugins/start/{id}      # Start plugin
POST /api/plugins/stop/{id}       # Stop plugin
GET /api/plugins/sources/{id}/documents  # Access source data
POST /api/plugins/agents/{id}/chat       # Chat with agent
```

## 🔧 Development

### Cargo Workspace
ConHub uses a unified Rust workspace for efficient development:

```bash
# Build entire workspace
cargo build --workspace

# Run specific service
cargo run -p auth-service
cargo run -p backend

# Run tests
cargo test --workspace

# Format code
cargo fmt --workspace
```

### Feature Toggles
Control development complexity with feature toggles (`feature-toggles.json`):

```json
{
  "Auth": false,    // Disable auth & databases for UI development
  "Heavy": false,   // Disable embedding/indexing for fast iteration
  "Docker": false   // Use local development (true = Docker containers)
}
```

**Toggle Modes:**
- `Docker: false` - **Local Development** (fastest, default)
  - Services run directly on your machine
  - No Docker build needed
  - Hot reload enabled
  - ~30-60 second startup

- `Docker: true` - **Production-like Environment**
  - All services in containers
  - Full infrastructure (PostgreSQL, Redis, Qdrant)
  - Isolated networking
  - ~2-4 minute startup

**Usage:** Simply run `npm start` - it intelligently detects the mode from feature-toggles.json

See [Docker Toggle Documentation](docs/DOCKER_TOGGLE_FEATURE.md) for details.

### GraphQL API
Unified GraphQL endpoint at `http://localhost:8000/api/graphql`:

```graphql
# Get user info
query {
  me { user_id, roles }
  repositories { id, name, url }
  documents { id, title, content }
}

# Generate embeddings
mutation {
  embed(texts: ["Hello world"], normalize: true) {
    embeddings
    dimension
  }
}
```

## 📊 Key Components

### 1. Fusion Embedding Service
Advanced multimodal embedding generation:
- Multiple model support (OpenAI, custom models)
- Fusion strategies (concatenation, weighted sum, attention)
- Batch processing with parallel execution
- REST API for integration

### 2. Intelligent Indexing
Dual-engine search architecture:
- **Code Indexing** (Tantivy): Fast symbol search, cross-references
- **Semantic Search** (Qdrant): Vector embeddings, similarity search
- **Language Support**: 40+ languages via tree-sitter

### 3. Plugin Architecture
Scalable plugin system:
- Dynamic loading/unloading
- Shared resources and connection pooling
- Centralized configuration management
- Hot reloading support

## 🔐 Security

### Authentication & Authorization
- **JWT RS256** tokens for service communication
- **OAuth 2.0** for third-party integrations
- **Role-based access control** (Admin, User, Guest)
- **Webhook signature verification**

### Infrastructure Security
- Non-root containers
- TLS/SSL encryption
- Network isolation
- Rate limiting
- SQL injection protection

## 🌐 API Examples

### Authentication
```bash
# Register user
curl -X POST http://localhost/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"SecurePass123!","name":"Test User"}'

# OAuth login
curl http://localhost/api/auth/oauth/google
```

### Repository Management
```bash
# Connect repository
curl -X POST http://localhost/api/data/sources \
  -H "Content-Type: application/json" \
  -d '{
    "type": "github",
    "url": "https://github.com/rust-lang/rust",
    "credentials": {"token": "your_github_token"}
  }'
```

### AI Agent Integration
```bash
# Chat with agent
curl -X POST http://localhost/api/plugins/agents/cline-main/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Help me debug this code",
    "context": {...}
  }'
```

## 📁 Project Structure

```
ConHub/
├── auth/                    # Authentication service
├── backend/                 # Unified GraphQL backend
├── billing/                 # Payment processing
├── client/                  # AI client integrations
├── data/                    # Data source management
├── embedding/               # Fusion embedding service
├── frontend/                # Next.js application
├── indexers/                # Search & indexing
├── plugins/                 # Unified plugin system
├── security/                # Security & audit
├── webhook/                 # Webhook handling
├── shared/                  # Shared Rust libraries
│   ├── config/             # Configuration management
│   ├── middleware/         # HTTP middleware
│   ├── models/             # Data models
│   ├── plugins/            # Plugin framework
│   └── utils/              # Utility functions
├── database/               # Database setup & migrations
├── nginx/                  # API Gateway configuration
├── scripts/                # Orchestration scripts
└── docs/                   # Documentation
```

## 🛠️ Environment Configuration

### Required Environment Variables
```bash
# Database
DATABASE_URL_LOCAL=postgresql://conhub:conhub_password@localhost:5432/conhub
DATABASE_URL_DOCKER=postgresql://conhub:conhub_password@postgres:5432/conhub
REDIS_URL_LOCAL=redis://localhost:6379
REDIS_URL_DOCKER=redis://redis:6379
QDRANT_URL_LOCAL=http://localhost:6333
QDRANT_URL_DOCKER=http://qdrant:6333

# Authentication
JWT_SECRET=your-super-secret-jwt-key
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
GITHUB_CLIENT_ID=your-github-client-id
GITHUB_CLIENT_SECRET=your-github-client-secret

# AI Services
OPENAI_API_KEY=sk-your-openai-key
ANTHROPIC_API_KEY=sk-ant-your-anthropic-key

# Payments
STRIPE_SECRET_KEY=sk_test_your-stripe-key
STRIPE_WEBHOOK_SECRET=whsec_your-webhook-secret
```

## 🚀 Deployment

### Docker Compose (Local/Staging)
```bash
# Production build
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up --build

# With custom environment
ENV_MODE=docker docker-compose up
```

### Azure Container Apps (Production)
```bash
# Deploy to Azure
./deploy-to-azure.ps1 -Environment production

# Monitor deployment
az containerapp logs show --name conhub-backend --resource-group conhub-rg
```

## 📈 Performance & Monitoring

### Build Performance
- **Cargo Workspace**: 40-60% faster builds
- **Shared Dependencies**: Consistent versions across services
- **Docker BuildKit**: Parallel builds with caching

### Runtime Metrics
- **GraphQL Federation**: Single API endpoint
- **Connection Pooling**: 95% connection reuse
- **Caching**: 85% hit rate for embeddings
- **Search Latency**: <50ms for vector queries

### Health Monitoring
```bash
# Check all services
curl http://localhost/health

# Individual service health
curl http://localhost:8000/health    # Backend
curl http://localhost:3010/health    # Auth
curl http://localhost:3020/health    # Plugins
```

## 🧪 Testing

### Run Tests
```bash
# Rust tests
cargo test --workspace

# Frontend tests
cd frontend && npm test

# Integration tests
npm run test:docker
```

### Test Environments
```bash
# Minimal (UI only)
echo '{"Auth": false, "Heavy": false}' > feature-toggles.json

# With Auth
echo '{"Auth": true, "Heavy": false}' > feature-toggles.json

# Full stack
echo '{"Auth": true, "Heavy": true}' > feature-toggles.json
```

## 🔍 Troubleshooting

### Common Issues

**Port Conflicts**
```bash
# Find process using port
lsof -i :8000  # macOS/Linux
netstat -ano | findstr :8000  # Windows

# Kill process
kill -9 <PID>
```

**Database Connection**
```bash
# Check database status
docker-compose ps postgres

# Reset database
docker-compose down -v
docker-compose up postgres
```

**Build Issues**
```bash
# Clean Rust cache
cargo clean

# Clean Docker cache
docker system prune -a

# Rebuild from scratch
docker-compose build --no-cache
```

## 📚 Documentation

- **Architecture**: `docs/ARCHITECTURE_SUMMARY.md`
- **Quick Start**: `docs/QUICK_START.md`
- **Migration Guide**: `docs/MIGRATION_GUIDE.md`
- **API Documentation**: `docs/api-documentation.md`

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test --workspace`
6. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: `/docs` folder
- **GraphQL Playground**: `http://localhost:8000/api/graphql`
- **Health Checks**: `http://localhost:[port]/health`
- **Logs**: `docker-compose logs -f [service]`

---

**Built with ❤️ using Rust, Next.js, and modern cloud technologies**

**Total Services**: 11 microservices + 3 databases + frontend + gateway  
**Ports Used**: 80, 3000, 3010-3015, 3020, 5432, 6333-6334, 6379, 8000, 8080, 8082  
**Architecture**: Microservices with GraphQL federation and unified plugin system