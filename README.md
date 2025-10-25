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

- Docker & Docker Compose
- At least 8GB RAM
- Ports 80, 3000, 3004-3007, 3010-3015, 5432, 6333-6334, 6379, 8080 available

### Installation (5 Minutes)

```bash
# 1. Clone and configure
git clone <your-repo-url>
cd ConHub
cp .env.example .env
# Edit .env with your API keys (JWT_SECRET, STRIPE_SECRET_KEY, OPENAI_API_KEY, etc.)

# 2. Create Docker network
docker network create conhub-network

# 3. Start databases (wait ~30 seconds for healthy status)
cd database
docker-compose up -d
docker-compose ps  # Verify all healthy

# 4. Start application services (first build takes 10-20 min)
cd ..
docker-compose up -d --build

# 5. Verify services
curl http://localhost/health                # Nginx gateway
curl http://localhost/api/auth/health       # Auth service
curl http://localhost/api/billing/health    # Billing service
curl http://localhost/api/ai/health         # AI service
open http://localhost:3000                  # Frontend
```

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
