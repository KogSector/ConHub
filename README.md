# ConHub - Microservices Architecture

Supercharge Your Development with AI - Unified platform connecting repositories, docs, and URLs with AI agents for enhanced development workflows.

## 🎯 Overview

ConHub is a comprehensive AI-powered platform built on a **microservices architecture** that connects multiple knowledge sources with AI agents through the Model Context Protocol (MCP). It provides semantic search, code indexing, document processing, and seamless AI assistant integration.

### Architecture

ConHub uses a modern **decoupled microservices architecture**:

**6 Backend Microservices** (Rust):
- **Auth Service** (3010) - Authentication, OAuth, sessions
- **Billing Service** (3011) - Stripe payments & subscriptions
- **AI Service** (3012) - AI agents & LLM operations
- **Data Service** (3013) - Data sources & integrations
- **Security Service** (3014) - Security & rulesets
- **Webhook Service** (3015) - External webhooks

**Infrastructure**:
- **PostgreSQL** (5432) - Main database
- **Redis** (6379) - Cache & sessions
- **Qdrant** (6333) - Vector database

**Other Services**:
- **Frontend** (3000) - Next.js application
- **Unified Indexer** (8080) - Code/doc indexing
- **MCP Service** (3004) - Model Context Protocol hub
- **MCP Servers** (3005-3007) - Google Drive, Dropbox, Filesystem
- **Nginx** (80) - API Gateway

## ⚡ Quick Start

### Prerequisites
- Node.js >= 18.0.0
- Rust (latest stable)
- Docker and Docker Compose
- Cargo (comes with Rust)

### Local Development Setup

1. **Start Databases**
   ```bash
   cd database
   docker-compose up -d
   cd ..
   ```

2. **Configure Environment**
   ```bash
   cp .env.example .env
   # Edit .env with your API keys and configuration
   # ENV_MODE should be set to 'local'
   ```

3. **Install Dependencies**
   ```bash
   npm install
   ```

4. **Start All Services**
   ```bash
   npm start
   ```

   This starts 12 services:
   - Frontend (Next.js) on port 3000
   - 6 Rust microservices (ports 3010-3015)
   - Unified Indexer on port 8080
   - MCP Service on port 3004
   - 3 MCP Servers (ports 3005-3007)

5. **Verify Setup**
   ```bash
   npm run test:local
   ```

6. **Access Application**
   - Frontend: http://localhost:3000
   - API Services: http://localhost:3010-3015

### Docker Deployment

1. **Create Network**
   ```bash
   docker network create conhub-network
   ```

2. **Start Databases**
   ```bash
   cd database
   docker-compose up -d
   cd ..
   ```

3. **Build and Start Services**
   ```bash
   docker-compose up --build
   ```

4. **Verify Setup**
   ```bash
   npm run test:docker
   ```

5. **Access Application**
   - Frontend: http://localhost (via Nginx on port 80)

### Development Workflows

**Run Individual Services (for debugging):**
```bash
# Individual Rust services
npm run dev:auth
npm run dev:billing
npm run dev:ai
npm run dev:data
npm run dev:security
npm run dev:webhook
npm run dev:indexer

# Frontend only
npm run dev:frontend

# Individual MCP services
npm run dev:mcp-service
npm run dev:mcp-gdrive
npm run dev:mcp-fs
npm run dev:mcp-dropbox
```

**Build for Production:**
```bash
npm run build        # Builds both frontend and backend
npm run build:frontend   # Next.js production build
npm run build:backend    # Cargo release build
```

**Database Management:**
```bash
npm run db:start     # Start databases
npm run db:test      # Test database connection
npm run db:clear     # Clear database
```

**Stop Services:**
```bash
# Local development
Ctrl+C in the terminal running npm start

# Docker
docker-compose down

# Force stop (kill all processes)
npm run force-stop
```

### Troubleshooting

**Port Conflicts:**
```bash
npm run cleanup      # Kills processes on required ports
```

**Database Connection Issues:**
- Verify databases are running: `docker ps | grep conhub`
- Check ENV_MODE matches your setup (local vs docker)
- Verify .env has correct DATABASE_URL_LOCAL or DATABASE_URL_DOCKER

**Rust Build Failures:**
```bash
cargo clean
cargo build          # Rebuild from scratch
```

**MCP Service Issues:**
```bash
cd mcp/service && npm install
cd ../servers/google-drive && npm install
cd ../filesystem && npm install
cd ../dropbox && npm install
```

**Docker Build Failures:**
- Frontend: Verify `output: 'standalone'` in next.config.js
- Services: Check ENV_MODE=docker in docker-compose.yml
- Network: Ensure conhub-network exists

### Environment Configuration

**ENV_MODE Variable:**
- `local`: Services run locally, connect to databases on localhost
- `docker`: All services run in Docker containers

**Database URLs:**
- Local: `DATABASE_URL_LOCAL=postgresql://conhub:conhub_password@localhost:5432/conhub`
- Docker: `DATABASE_URL_DOCKER=postgresql://conhub:conhub_password@postgres:5432/conhub`

Services automatically select the correct URL based on ENV_MODE.

## 🏗️ Project Structure

```
ConHub/
├── database/                      # Database infrastructure
│   ├── docker-compose.yml        # Postgres, Redis, Qdrant
│   ├── postgres/
│   │   ├── init/                 # Initialization scripts
│   │   └── migrations/           # SQL migrations
│   ├── qdrant/config/            # Qdrant configuration
│   └── redis/                    # Redis configuration
│
├── services/                     # Backend microservices
│   ├── auth-service/            # Port 3010
│   ├── billing-service/         # Port 3011
│   ├── ai-service/              # Port 3012
│   ├── data-service/            # Port 3013
│   ├── security-service/        # Port 3014
│   └── webhook-service/         # Port 3015
│
├── shared/                       # Shared Rust libraries
│   ├── models/                  # Data models
│   ├── utils/                   # Utility functions
│   ├── middleware/              # HTTP middleware
│   └── config/                  # Configuration
│
├── frontend/                     # Next.js frontend (3000)
├── indexers/                     # Unified indexer (8080)
├── mcp/                          # MCP components
│   ├── service/                 # MCP protocol service (3004)
│   └── servers/                 # Provider servers (3005-3007)
│       ├── google-drive/
│       ├── dropbox/
│       └── filesystem/
│
├── nginx/                        # API Gateway
│   └── nginx.conf
│
├── Cargo.toml                    # Rust workspace
├── docker-compose.yml            # Application services
└── .env.example                  # Environment template
```

## 🔧 Development

### Building Services

```bash
# Build all services
docker-compose build

# Build specific service
docker-compose build auth-service

# View logs
docker-compose logs -f auth-service
docker-compose logs -f billing-service
```

### Local Rust Development

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build workspace
cargo build --workspace

# Run specific service
cargo run --bin auth-service
cargo run --bin billing-service

# Run tests
cargo test --workspace
```

### Stopping Services

```bash
# Stop application services
docker-compose down

# Stop databases
cd database && docker-compose down

# Stop and remove volumes (CAUTION: deletes data)
docker-compose down -v
cd database && docker-compose down -v
```

## 🚀 Key Features

### Data Source Integration
- **Version Control**: GitHub, GitLab, BitBucket with automatic indexing
- **Cloud Storage**: Google Drive, Dropbox, OneDrive
- **Web Content**: Crawl documentation sites
- **Local Files**: Upload and index documents

### AI Agent Integration
- **Amazon Q**: AWS assistance
- **GitHub Copilot**: AI pair programming
- **Cline**: Software engineering tasks
- **Cursor IDE**: Code generation
- **Unified Context**: Real-time context across all sources

### Advanced Search & Indexing
- **Dual-Engine**: Fast code indexing + semantic document search
- **Automatic Pipeline**: Background indexing on source connection
- **Multi-Source**: Unified search across code, docs, URLs

## 🔐 Service Communication

### API Gateway (Nginx)

All frontend requests go through Nginx:

```
http://localhost/                → Frontend (3000)
http://localhost/api/auth/*      → Auth Service (3010)
http://localhost/api/billing/*   → Billing Service (3011)
http://localhost/api/ai/*        → AI Service (3012)
http://localhost/api/data/*      → Data Service (3013)
http://localhost/api/security/*  → Security Service (3014)
http://localhost/api/webhooks/*  → Webhook Service (3015)
http://localhost/indexer/*       → Unified Indexer (8080)
http://localhost/mcp/*           → MCP Service (3004)
```

### Inter-Service Communication

Services communicate directly using Docker service names:

```rust
// Example: Data service calling Auth service
let auth_url = "http://auth-service:3010/api/auth/verify";
let response = reqwest::get(auth_url).await?;
```

## 📝 API Examples

### Authentication

```bash
# Register user
curl -X POST http://localhost/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"SecurePass123!","name":"Test User"}'

# Login
curl -X POST http://localhost/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"SecurePass123!"}'

# OAuth (Google)
curl http://localhost/api/auth/oauth/google
```

### Repository Indexing

```bash
# Connect repository
curl -X POST http://localhost/api/data/sources \
  -H "Content-Type: application/json" \
  -d '{
    "type": "github",
    "url": "https://github.com/rust-lang/rust",
    "credentials": {"token": "your_github_token"}
  }'

# Index repository
curl -X POST http://localhost/indexer/api/index/repository \
  -H "Content-Type: application/json" \
  -d '{
    "repository_url": "https://github.com/rust-lang/rust",
    "branch": "master"
  }'
```

### Search

```bash
# Search code
curl -X POST http://localhost/indexer/api/search \
  -H "Content-Type: application/json" \
  -d '{"query": "async fn", "source_type": "code", "limit": 10}'
```

### AI Agents

```bash
# Query AI agent
curl -X POST http://localhost/api/ai/chat \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Explain this code",
    "context": "Rust async programming"
  }'
```

## 🔍 Cargo Workspace

ConHub uses a Cargo workspace for efficient Rust development:

```toml
[workspace]
members = [
    "services/auth-service",
    "services/billing-service",
    "services/ai-service",
    "services/data-service",
    "services/security-service",
    "services/webhook-service",
    "shared/models",
    "shared/utils",
    "shared/middleware",
    "shared/config",
    "indexers",
]
```

**Benefits:**
- Single dependency version management
- Shared compilation cache
- Faster builds
- Consistent versions across services

## 🧩 MCP (Model Context Protocol)

MCP provides AI agents with access to various data sources:

### MCP Service (Port 3004)
- Core MCP protocol implementation
- Agent connection management
- WebSocket support
- Integration with GitHub Copilot, Amazon Q, Cline

### MCP Servers
- **Google Drive** (3005) - OAuth2 access to Drive files
- **Dropbox** (3006) - OAuth2 access to Dropbox
- **Filesystem** (3007) - Local filesystem access

### MCP Usage

```bash
# Start all MCP services
docker-compose up -d mcp-service mcp-google-drive mcp-dropbox mcp-filesystem

# Connect agent
curl -X POST http://localhost:3004/mcp/connect \
  -H "Content-Type: application/json" \
  -d '{"agent": "github-copilot", "token": "your-jwt"}'
```

## 🛠️ Troubleshooting

### Services Won't Start

```bash
# Check databases are healthy
cd database && docker-compose ps

# View logs
docker-compose logs postgres
docker-compose logs redis
docker-compose logs qdrant
```

### Port Conflicts

```bash
# Check what's using a port
lsof -i :3010

# Kill process
kill -9 <PID>
```

### Rust Compilation Errors

```bash
# Clean and rebuild
cargo clean
cargo build --workspace
```

### Docker Build Failures

```bash
# Clean Docker cache
docker system prune -a

# Rebuild without cache
docker-compose build --no-cache
```

### Database Migrations

```bash
# Check postgres logs
cd database && docker-compose logs postgres

# Manually run migrations
docker exec -it conhub-postgres psql -U conhub -d conhub -f /migrations/001_initial_schema.sql
```

### Nginx Routing Issues

```bash
# Test nginx config
docker exec conhub-nginx nginx -t

# Reload nginx
docker exec conhub-nginx nginx -s reload

# Check logs
docker-compose logs nginx
```

## 📊 Monitoring

### Health Checks

All services expose `/health` endpoints:

```bash
# Create monitoring script
cat > monitor.sh << 'EOF'
#!/bin/bash
services=(
  "auth-service:3010"
  "billing-service:3011"
  "ai-service:3012"
  "data-service:3013"
  "security-service:3014"
  "webhook-service:3015"
  "unified-indexer:8080"
  "mcp-service:3004"
)

for service in "${services[@]}"; do
  name="${service%:*}"
  port="${service#*:}"
  if curl -f "http://localhost:$port/health" &>/dev/null; then
    echo "✅ $name is healthy"
  else
    echo "❌ $name is down"
  fi
done
EOF

chmod +x monitor.sh
./monitor.sh
```

### Logs

```bash
# View all logs
docker-compose logs -f

# View specific service
docker-compose logs -f auth-service

# Last 100 lines
docker-compose logs --tail=100 auth-service

# Since timestamp
docker-compose logs --since="2024-01-01T00:00:00" auth-service
```

## 🚀 Production Deployment

### Pre-deployment Checklist

- [ ] Set strong `JWT_SECRET`
- [ ] Configure production database URLs
- [ ] Set up SSL certificates for Nginx
- [ ] Configure production Qdrant
- [ ] Set up monitoring/logging
- [ ] Configure database backups
- [ ] Test all services independently
- [ ] Load test critical services

### Production Configuration

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  nginx:
    ports:
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro

  auth-service:
    environment:
      - RUST_LOG=warn
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
```

Deploy:
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

## 🔒 Security

### Authentication
- JWT tokens for service-to-service communication
- OAuth2 for Google Drive, Dropbox, GitHub, etc.
- Webhook signature verification
- Rate limiting on API endpoints

### Best Practices
- Store secrets in `.env` (never commit)
- Use HTTPS in production
- Enable CORS properly
- Implement rate limiting
- Regular security audits
- Keep dependencies updated

## 🎯 Architecture Benefits

1. **Independent Scaling**: Scale services independently based on load
2. **Isolated Failures**: One service failure doesn't affect others
3. **Technology Flexibility**: Each service can use optimal tech
4. **Team Organization**: Teams can own specific services
5. **Deployment Speed**: Deploy individual services quickly
6. **Development Speed**: Smaller codebases, faster compile times
7. **Clear Boundaries**: Well-defined service responsibilities
8. **Database Isolation**: Clean separation of data infrastructure

## 📈 Performance

### Optimizations
- Async processing (Tokio runtime)
- Connection pooling
- Redis caching
- Code splitting (Next.js)
- Vector optimization
- Nginx load balancing

### Resource Limits

Adjust in `docker-compose.yml`:

```yaml
services:
  auth-service:
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes in appropriate service
4. Add tests
5. Submit pull request

## 📄 Environment Variables

Required variables in `.env`:

```bash
# JWT & Security
JWT_SECRET=your-secret-here

# Database
DATABASE_URL=postgresql://conhub:conhub_password@postgres:5432/conhub
REDIS_URL=redis://redis:6379
QDRANT_URL=http://qdrant:6333

# Stripe
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...

# AI Services
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# OAuth
GOOGLE_CLIENT_ID=...
GOOGLE_CLIENT_SECRET=...
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...

# MCP Servers
GOOGLE_DRIVE_CLIENT_ID=...
GOOGLE_DRIVE_CLIENT_SECRET=...
DROPBOX_APP_KEY=...
DROPBOX_APP_SECRET=...
```

## 📚 Additional Resources

- **Architecture**: See project structure above
- **API Docs**: Each service exposes `/health` and service-specific endpoints
- **Cargo Workspace**: All Rust code in workspace for shared dependencies
- **Docker**: All services containerized for easy deployment

## 📊 System Requirements

- **Development**: 8GB RAM, 4 CPU cores
- **Production**: 16GB+ RAM, 8+ CPU cores
- **Storage**: 50GB+ for databases and indexes

## 🐛 Known Issues

None currently. Report issues via GitHub Issues.

## 📜 License

MIT License - see LICENSE file for details.

---

**Total Containers**: 16
- 3 databases (Postgres, Redis, Qdrant)
- 1 frontend
- 6 backend microservices
- 1 unified indexer
- 1 MCP service
- 3 MCP servers
- 1 nginx gateway

**Ports Used**: 80, 3000, 3004-3007, 3010-3015, 5432, 6333-6334, 6379, 8080

---

# Additional Documentation

## ConHub Microservices Architecture Details

This section describes the new microservices architecture where each service manages its own dependencies independently.

### Microservices with Node.js Dependencies

Each of the following microservices has its own `package.json` file and manages dependencies independently:

#### Frontend Service
- **Location**: `frontend/`
- **Package**: `frontend/package.json`
- **Purpose**: Next.js web application
- **Start**: `cd frontend && npm run dev`

#### Scripts Service (Orchestration)
- **Location**: `scripts/`
- **Package**: `scripts/package.json`
- **Purpose**: Orchestration and utility scripts
- **Dependencies**: concurrently, dotenv, nodemon
- **Start**: `cd scripts && npm run start`

#### MCP Service
- **Location**: `mcp/service/`
- **Package**: `mcp/service/package.json`
- **Purpose**: Model Context Protocol service
- **Start**: `cd mcp/service && npm start`

#### MCP Servers
Each MCP server has its own package.json:

**Source Servers**
- **Filesystem**: `mcp/servers/sources/filesystem/`
- **Dropbox**: `mcp/servers/sources/dropbox/`
- **Google Drive**: `mcp/servers/sources/google-drive/`

**Agent Servers**
- **Amazon Q**: `mcp/servers/agents/amazon-q/`
- **Cline**: `mcp/servers/agents/cline/`
- **GitHub Copilot**: `mcp/servers/agents/github-copilot/`

### Rust Microservices

The following services use Cargo.toml for dependency management:
- `ai/` - AI service
- `auth/` - Authentication service
- `backend/` - Main backend service
- `billing/` - Billing service
- `data/` - Data service
- `indexers/` - Indexing service
- `plugins/` - Plugins service
- `security/` - Security service
- `webhook/` - Webhook service

### Microservices Quick Start

#### Using the Orchestration Script
```bash
# Start all development services
node start.js dev

# Start with Docker
node start.js docker

# Stop all services
node start.js stop

# Check status
node start.js status

# Start only frontend
node start.js frontend

# Clean up ports
node start.js cleanup

# Show help
node start.js help
```

#### Manual Service Management
```bash
# Frontend
cd frontend && npm run dev

# Scripts/Orchestration
cd scripts && npm run start

# MCP Service
cd mcp/service && npm start

# Individual MCP servers
cd mcp/servers/sources/filesystem && npm start
cd mcp/servers/sources/dropbox && npm start
# ... etc
```

### Development Workflow

1. **Install Dependencies**: Each microservice manages its own dependencies
   ```bash
   cd frontend && npm install
   cd scripts && npm install
   cd mcp/service && npm install
   # ... for each Node.js microservice
   ```

2. **Start Services**: Use the orchestration script or start individually
   ```bash
   node start.js dev  # Recommended
   ```

3. **Add Dependencies**: Add to the specific microservice's package.json
   ```bash
   cd frontend && npm install new-package
   cd scripts && npm install new-utility
   ```

### Migration Notes

- **Removed**: Root-level `package.json` and `package-lock.json`
- **Added**: Individual `package.json` files for each Node.js microservice
- **Changed**: Scripts now delegate to the scripts microservice
- **Benefit**: True microservice independence with isolated dependencies

### Service URLs

- **Frontend**: http://localhost:3000
- **Auth Service**: http://localhost:3010
- **Billing Service**: http://localhost:3011
- **AI Service**: http://localhost:3012
- **Data Service**: http://localhost:3013
- **Security Service**: http://localhost:3014
- **Webhook Service**: http://localhost:3015
- **Indexer Service**: http://localhost:8080
- **MCP Service**: http://localhost:3004
- **MCP Google Drive**: http://localhost:3005
- **MCP Filesystem**: http://localhost:3006
- **MCP Dropbox**: http://localhost:3007

## Architecture Migration Guide: From Microservices to Plugin System

### Overview

This section outlines the migration from individual microservices for each source and agent to a unified plugin-based architecture that can scale to hundreds of sources and agents.

### Problem Statement

#### Current Issues
- **Resource Overhead**: Each source/agent runs as a separate microservice with its own container, port, and process
- **Management Complexity**: Hundreds of services would be impossible to manage
- **Deployment Complexity**: Each service needs individual deployment, monitoring, and configuration
- **Resource Waste**: Most services are idle most of the time but still consume resources

#### Current Structure
```
mcp/servers/
├── sources/
│   ├── dropbox/          # Separate microservice
│   ├── google-drive/     # Separate microservice
│   └── filesystem/       # Separate microservice
└── agents/
    ├── cline/            # Separate microservice
    ├── amazon-q/         # Separate microservice
    └── github-copilot/   # Separate microservice
```

### New Architecture

#### Plugin-Based System
- **Single Service**: One unified plugins service hosts all sources and agents
- **Dynamic Loading**: Plugins are loaded/unloaded on demand
- **Shared Resources**: All plugins share the same process, memory, and network resources
- **Centralized Management**: Single API for managing all plugins

#### New Structure
```
plugins/                          # Unified plugins service
├── src/
│   ├── main.rs                  # Main service
│   ├── handlers/                # API handlers
│   └── services/                # Core services
├── plugins/                     # Plugin implementations
│   ├── dropbox/                 # Dropbox plugin
│   ├── google-drive/            # Google Drive plugin
│   ├── cline/                   # Cline agent plugin
│   └── amazon-q/                # Amazon Q agent plugin
├── config/
│   └── plugins.json             # Plugin configurations
└── Dockerfile                   # Single container

shared/plugins/                   # Plugin framework
├── src/
│   ├── lib.rs                   # Core plugin traits
│   ├── sources.rs               # Source plugin interfaces
│   ├── agents.rs                # Agent plugin interfaces
│   ├── registry.rs              # Plugin registry
│   └── config.rs                # Configuration management
```

### Key Components

#### 1. Plugin Framework (`shared/plugins/`)
- **Core Traits**: Define interfaces for all plugins
- **Registry System**: Manages plugin lifecycle
- **Configuration**: Unified config management
- **Error Handling**: Standardized error types

#### 2. Unified Service (`plugins/`)
- **Single Process**: Hosts all plugins
- **REST API**: Manage plugins via HTTP
- **Dynamic Loading**: Load/unload plugins at runtime
- **Health Monitoring**: Monitor all plugins from one place

#### 3. Plugin Implementations (`plugins/plugins/`)
- **Modular**: Each plugin is a separate crate
- **Standardized**: All implement the same interfaces
- **Configurable**: Runtime configuration support
- **Isolated**: Plugins can't interfere with each other

### Benefits

#### Scalability
- **Resource Efficiency**: Single process for hundreds of plugins
- **Memory Sharing**: Shared libraries and resources
- **Connection Pooling**: Shared HTTP clients and database connections

#### Management
- **Single API**: One endpoint to manage all plugins
- **Centralized Logging**: All logs in one place
- **Unified Monitoring**: Single health check endpoint
- **Configuration Management**: One config file for all plugins

#### Development
- **Standardized Interfaces**: Consistent plugin development
- **Hot Reloading**: Update plugins without service restart
- **Testing**: Easier to test individual plugins
- **Deployment**: Single container deployment

### Migration Steps

#### Phase 1: Framework Setup ✅
- [x] Create plugin framework (`shared/plugins/`)
- [x] Define core traits and interfaces
- [x] Implement registry system
- [x] Create configuration management

#### Phase 2: Service Implementation ✅
- [x] Create unified plugins service
- [x] Implement REST API handlers
- [x] Add plugin lifecycle management
- [x] Create Docker configuration

#### Phase 3: Plugin Migration
- [ ] Migrate Dropbox source to plugin
- [ ] Migrate Google Drive source to plugin
- [ ] Migrate Cline agent to plugin
- [ ] Migrate Amazon Q agent to plugin
- [ ] Add remaining sources and agents

#### Phase 4: Integration
- [ ] Update data service to use plugins API
- [ ] Update AI service to use plugins API
- [ ] Update frontend to manage plugins
- [ ] Remove old MCP servers

#### Phase 5: Testing & Deployment
- [ ] Integration testing
- [ ] Performance testing
- [ ] Production deployment
- [ ] Monitor and optimize

### API Examples

#### Plugin Management
```bash
# List available plugin types
GET /api/plugins/registry/sources
GET /api/plugins/registry/agents

# Start a plugin
POST /api/plugins/start/dropbox-main

# Stop a plugin
POST /api/plugins/stop/dropbox-main

# Get plugin status
GET /api/plugins/status/dropbox-main
```

#### Source Operations
```bash
# List documents from a source
GET /api/plugins/sources/dropbox-main/documents

# Search documents
POST /api/plugins/sources/dropbox-main/search
{
  "query": "project proposal"
}

# Sync source
POST /api/plugins/sources/dropbox-main/sync
```

#### Agent Operations
```bash
# Chat with an agent
POST /api/plugins/agents/cline-main/chat
{
  "message": "Help me debug this code",
  "context": {...}
}

# Get agent functions
GET /api/plugins/agents/cline-main/functions
```

### Configuration

#### Plugin Configuration (`plugins/config/plugins.json`)
```json
{
  "plugins": {
    "dropbox-main": {
      "instance_id": "dropbox-main",
      "plugin_type": "Source",
      "plugin_name": "dropbox",
      "enabled": true,
      "auto_start": true,
      "config": {
        "enabled": true,
        "settings": {
          "access_token": "your-token",
          "sync_interval_minutes": 30
        }
      }
    }
  }
}
```

#### Environment Variables
```bash
PLUGINS_SERVICE_PORT=3020
PLUGIN_CONFIG_PATH=./config/plugins.json
DATABASE_URL=postgresql://...
```

### Deployment

#### Docker Compose Update
```yaml
services:
  plugins:
    build: ./plugins
    ports:
      - "3020:3020"
    environment:
      - PLUGINS_SERVICE_PORT=3020
      - PLUGIN_CONFIG_PATH=/app/config/plugins.json
    volumes:
      - ./plugins/config:/app/config
    depends_on:
      - postgres
      - qdrant

  # Remove individual MCP services
  # dropbox-mcp: (removed)
  # google-drive-mcp: (removed)
  # cline-mcp: (removed)
```

### Monitoring

#### Health Checks
- **Service Health**: `/health` endpoint
- **Plugin Health**: Individual plugin status
- **Resource Usage**: Memory, CPU per plugin
- **Error Tracking**: Centralized error logging

#### Metrics
- **Plugin Count**: Active/inactive plugins
- **Request Rate**: API requests per plugin
- **Response Time**: Plugin operation latency
- **Error Rate**: Plugin failure rates

### Security

#### Plugin Isolation
- **Memory Isolation**: Plugins can't access each other's data
- **Configuration Isolation**: Separate config per plugin
- **Error Isolation**: Plugin failures don't affect others
- **Resource Limits**: CPU/memory limits per plugin

#### API Security
- **Authentication**: JWT tokens for API access
- **Authorization**: Role-based plugin access
- **Rate Limiting**: Prevent API abuse
- **Input Validation**: Sanitize all inputs

### Performance

#### Optimizations
- **Lazy Loading**: Load plugins only when needed
- **Connection Pooling**: Shared HTTP/DB connections
- **Caching**: Cache plugin responses
- **Async Operations**: Non-blocking plugin operations

#### Scaling
- **Horizontal**: Multiple plugin service instances
- **Load Balancing**: Distribute plugin load
- **Resource Management**: Dynamic resource allocation
- **Auto-scaling**: Scale based on plugin usage

### Troubleshooting

#### Common Issues
1. **Plugin Won't Start**: Check configuration and logs
2. **High Memory Usage**: Monitor plugin resource usage
3. **API Timeouts**: Check plugin response times
4. **Configuration Errors**: Validate plugin config schema

#### Debugging
- **Logs**: Centralized logging with plugin context
- **Metrics**: Real-time plugin performance metrics
- **Health Checks**: Automated plugin health monitoring
- **Tracing**: Request tracing across plugins

### Future Enhancements

#### Plugin Marketplace
- **Plugin Discovery**: Browse available plugins
- **Plugin Installation**: Install plugins from registry
- **Version Management**: Update plugins independently
- **Community Plugins**: Third-party plugin support

#### Advanced Features
- **Plugin Dependencies**: Manage plugin dependencies
- **Plugin Composition**: Combine multiple plugins
- **Plugin Workflows**: Chain plugin operations
- **Plugin Analytics**: Usage analytics per plugin

## Plugins Integration Guide

This section explains how to integrate the data and AI services with the new unified plugins system.

### Overview

The new plugin system replaces individual MCP microservices with a unified plugins service that manages all source and agent plugins through a single API.

### API Endpoints

#### Base URL
```
http://localhost:3020/api
```

#### Plugin Management

##### Get Plugin Status
```http
GET /status
```

Response:
```json
{
  "total_configured": 5,
  "total_active": 4,
  "active_sources": 2,
  "active_agents": 2,
  "enabled_plugins": 5,
  "auto_start_plugins": 4,
  "operation_stats": {...},
  "last_updated": "2024-01-15T10:30:00Z"
}
```

##### List All Plugins
```http
GET /plugins
```

##### Start/Stop Plugins
```http
POST /lifecycle/{instance_id}/start
POST /lifecycle/{instance_id}/stop
POST /lifecycle/{instance_id}/restart
GET /lifecycle/{instance_id}/status
```

#### Source Plugin Operations

##### List Documents
```http
GET /sources/{instance_id}/documents?limit=50&offset=0
```

##### Get Document
```http
GET /sources/{instance_id}/documents/{document_id}
```

##### Search Documents
```http
GET /sources/{instance_id}/search?q={query}&limit=20
```

##### Sync Documents
```http
POST /sources/{instance_id}/sync
```

##### Upload Document
```http
POST /sources/{instance_id}/upload
Content-Type: multipart/form-data

file: <file_data>
metadata: {"title": "Document Title", "tags": ["tag1", "tag2"]}
```

##### Delete Document
```http
DELETE /sources/{instance_id}/documents/{document_id}
```

#### Agent Plugin Operations

##### Send Message
```http
POST /agents/{instance_id}/chat
Content-Type: application/json

{
  "message": {
    "content": "Hello, can you help me with this code?",
    "role": "user",
    "metadata": {}
  },
  "context": {
    "conversation_id": "conv_123",
    "user_id": "user_456",
    "session_data": {}
  }
}
```

##### Stream Chat
```http
POST /agents/{instance_id}/stream
Content-Type: application/json

{
  "message": {...},
  "context": {...}
}
```

##### Get Available Functions
```http
GET /agents/{instance_id}/functions
```

##### Execute Action
```http
POST /agents/{instance_id}/actions/{action_name}
Content-Type: application/json

{
  "parameters": {
    "param1": "value1",
    "param2": "value2"
  },
  "context": {...}
}
```

### Integration Examples

#### Data Service Integration

##### Rust Example (using reqwest)

```rust
use reqwest::Client;
use serde_json::{json, Value};
use anyhow::Result;

pub struct PluginsClient {
    client: Client,
    base_url: String,
}

impl PluginsClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn list_documents(&self, instance_id: &str) -> Result<Value> {
        let url = format!("{}/sources/{}/documents", self.base_url, instance_id);
        let response = self.client.get(&url).send().await?;
        let data = response.json::<Value>().await?;
        Ok(data)
    }

    pub async fn search_documents(&self, instance_id: &str, query: &str) -> Result<Value> {
        let url = format!("{}/sources/{}/search", self.base_url, instance_id);
        let response = self.client
            .get(&url)
            .query(&[("q", query)])
            .send()
            .await?;
        let data = response.json::<Value>().await?;
        Ok(data)
    }
}
```
