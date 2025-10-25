# ConHub Microservices Architecture

This document describes the new microservices architecture for ConHub. The monolithic backend has been split into 6 independent microservices, each deployable separately.

## Architecture Overview

ConHub now consists of:

### Databases (in `database/`)
- **PostgreSQL** - Main database (port 5432)
- **Redis** - Cache and sessions (port 6379)
- **Qdrant** - Vector database for embeddings (ports 6333-6334)

### Backend Microservices (in `services/`)
- **Auth Service** - Authentication & OAuth (port 3010)
- **Billing Service** - Payments & Stripe (port 3011)
- **AI Service** - AI agents & LLM operations (port 3012)
- **Data Service** - Data sources & integrations (port 3013)
- **Security Service** - Security & rulesets (port 3014)
- **Webhook Service** - External webhooks (port 3015)

### Other Services
- **Frontend** - Next.js application (port 3000)
- **Unified Indexer** - Code indexing service (port 8080)
- **MCP Service** - MCP protocol service (port 3004)
- **MCP Servers** - Google Drive (3005), Dropbox (3006), Filesystem (3007)
- **Nginx** - API gateway & reverse proxy (port 80)

## Quick Start

### Prerequisites

- Docker and Docker Compose installed
- At least 8GB of available RAM
- Ports 80, 3000, 3004-3007, 3010-3015, 5432, 6333-6334, 6379, 8080 available

### Step 1: Environment Configuration

```bash
# Copy environment template
cp .env.example .env

# Edit .env with your API keys and secrets
nano .env
```

**Required environment variables:**
- `JWT_SECRET` - Generate with: `openssl rand -hex 32`
- `STRIPE_SECRET_KEY` - From Stripe dashboard
- `OPENAI_API_KEY` - From OpenAI
- OAuth credentials (GitHub, Google, etc.)

### Step 2: Create Docker Network

```bash
# Create the shared network
docker network create conhub-network
```

### Step 3: Start Database Infrastructure

```bash
# Start databases first
cd database
docker-compose up -d

# Verify databases are healthy
docker-compose ps
docker-compose logs -f
```

**Wait for all databases to be healthy** before proceeding (usually 30-60 seconds).

### Step 4: Start Application Services

```bash
# Return to root directory
cd ..

# Build and start all application services
docker-compose up -d --build

# Monitor startup logs
docker-compose logs -f
```

**Note:** First build may take 10-20 minutes as Rust services compile.

### Step 5: Verify All Services

```bash
# Check all containers are running
docker-compose ps

# Test individual service health endpoints
curl http://localhost:3010/health  # Auth Service
curl http://localhost:3011/health  # Billing Service
curl http://localhost:3012/health  # AI Service
curl http://localhost:3013/health  # Data Service
curl http://localhost:3014/health  # Security Service
curl http://localhost:3015/health  # Webhook Service
curl http://localhost:8080/health  # Unified Indexer
curl http://localhost:3004/health  # MCP Service

# Test via Nginx (API Gateway)
curl http://localhost/health
curl http://localhost/api/auth/health
curl http://localhost/api/billing/health

# Access frontend
open http://localhost:3000
```

## Development Workflow

### Building Individual Services

```bash
# Build specific microservice
docker-compose build auth-service

# Start specific service
docker-compose up -d auth-service

# View logs for specific service
docker-compose logs -f auth-service
```

### Local Rust Development (Without Docker)

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build workspace
cargo build --workspace

# Run specific service
cargo run --bin auth-service

# Run tests
cargo test --workspace
```

### Stopping Services

```bash
# Stop application services
docker-compose down

# Stop databases
cd database && docker-compose down

# Stop everything and remove volumes (CAUTION: deletes data)
docker-compose down -v
cd database && docker-compose down -v
```

## Service Communication

### API Gateway Routing (Nginx)

All frontend requests go through Nginx on port 80:

- `http://localhost/` → Frontend (port 3000)
- `http://localhost/api/auth/*` → Auth Service (3010)
- `http://localhost/api/billing/*` → Billing Service (3011)
- `http://localhost/api/ai/*` → AI Service (3012)
- `http://localhost/api/data/*` → Data Service (3013)
- `http://localhost/api/security/*` → Security Service (3014)
- `http://localhost/api/webhooks/*` → Webhook Service (3015)
- `http://localhost/indexer/*` → Unified Indexer (8080)
- `http://localhost/mcp/*` → MCP Service (3004)

### Inter-Service Communication

Services communicate directly using Docker service names:

```rust
// Example: Data service calling Auth service
let auth_url = "http://auth-service:3010/api/auth/verify";
let response = reqwest::get(auth_url).await?;
```

## Project Structure

```
ConHub/
├── database/                    # Database infrastructure
│   ├── docker-compose.yml      # Database orchestration
│   ├── postgres/               # PostgreSQL config & migrations
│   ├── qdrant/                 # Qdrant config
│   └── redis/                  # Redis config
│
├── services/                   # Backend microservices
│   ├── auth-service/          # Port 3010
│   ├── billing-service/       # Port 3011
│   ├── ai-service/            # Port 3012
│   ├── data-service/          # Port 3013
│   ├── security-service/      # Port 3014
│   └── webhook-service/       # Port 3015
│
├── shared/                     # Shared Rust libraries
│   ├── models/                # Data models
│   ├── utils/                 # Utility functions
│   ├── middleware/            # HTTP middleware
│   └── config/                # Configuration
│
├── frontend/                   # Next.js frontend (port 3000)
├── indexers/                   # Unified indexer (port 8080)
├── mcp/                        # MCP service (port 3004)
├── mcp-servers/               # MCP servers (ports 3005-3007)
│   ├── google-drive/
│   ├── dropbox/
│   └── filesystem/
│
├── nginx/                      # Nginx configuration
│   └── nginx.conf
│
├── Cargo.toml                  # Workspace configuration
├── docker-compose.yml          # Application services
└── .env.example                # Environment template
```

## Workspace Architecture

ConHub uses a **Cargo workspace** with:

- **6 backend microservices** (independent binaries)
- **4 shared libraries** (models, utils, middleware, config)
- **1 indexer service**
- **Shared dependencies** (defined once in workspace root)

### Building the Workspace

```bash
# Build all services
cargo build --workspace --release

# Build specific service
cargo build --release --bin auth-service

# Check compilation without building
cargo check --workspace
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific service
cargo test --package auth-service

# Run specific test
cargo test test_name
```

## Troubleshooting

### Issue: Services can't connect to databases

**Solution:** Ensure databases are started first and healthy:
```bash
cd database
docker-compose ps
# Wait until all show "healthy" status
```

### Issue: Port conflicts

**Solution:** Check for processes using required ports:
```bash
# Check specific port
lsof -i :3010

# Kill process using port
kill -9 <PID>
```

### Issue: Rust compilation errors

**Solution:**
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build --workspace
```

### Issue: Docker build failures

**Solution:**
```bash
# Clean Docker cache
docker system prune -a

# Rebuild without cache
docker-compose build --no-cache
```

### Issue: Database migrations not running

**Solution:**
```bash
# Check postgres logs
cd database
docker-compose logs postgres

# Manually run migrations (if needed)
docker exec -it conhub-postgres psql -U conhub -d conhub -f /migrations/001_initial_schema.sql
```

### Issue: Nginx routing not working

**Solution:**
```bash
# Verify nginx config syntax
docker exec conhub-nginx nginx -t

# Reload nginx config
docker exec conhub-nginx nginx -s reload

# Check nginx logs
docker-compose logs nginx
```

## Performance Tuning

### Database Connections

Each microservice has its own connection pool. Adjust in service code:

```rust
let pool = PgPoolOptions::new()
    .max_connections(10)  // Adjust based on load
    .connect(&database_url)
    .await?;
```

### Resource Limits

Add resource limits in `docker-compose.yml`:

```yaml
services:
  auth-service:
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
```

## Migration from Old Architecture

The old monolithic backend (`backend/src/main.rs`) is **deprecated** and will be removed after verification.

**Key changes:**
- Single backend (port 3001) → 6 microservices (ports 3010-3015)
- Database configs moved to `database/` folder
- Qdrant now dockerized (was external)
- Nginx routes to individual services
- MCP servers now dockerized

## Production Deployment

### Pre-deployment Checklist

- [ ] Update `.env` with production values
- [ ] Set strong `JWT_SECRET`
- [ ] Configure production database URLs
- [ ] Set up SSL certificates for Nginx
- [ ] Configure production Qdrant instance (or use dockerized)
- [ ] Set up monitoring and logging
- [ ] Configure backup strategy for databases
- [ ] Test all services independently
- [ ] Load test critical services

### Docker Compose Production Override

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
```

Run with:
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

## Monitoring

### Health Checks

All services expose `/health` endpoints for monitoring:

```bash
# Create monitoring script
cat > monitor.sh << 'EOF'
#!/bin/bash
services=("auth-service:3010" "billing-service:3011" "ai-service:3012"
          "data-service:3013" "security-service:3014" "webhook-service:3015")

for service in "${services[@]}"; do
    if curl -f "http://localhost:${service#*:}/health" &>/dev/null; then
        echo "✅ ${service%:*} is healthy"
    else
        echo "❌ ${service%:*} is down"
    fi
done
EOF

chmod +x monitor.sh
./monitor.sh
```

### Logs

```bash
# View all service logs
docker-compose logs -f

# View specific service logs
docker-compose logs -f auth-service

# View last 100 lines
docker-compose logs --tail=100 auth-service

# View logs since timestamp
docker-compose logs --since="2024-01-01T00:00:00" auth-service
```

## Support

For issues or questions:
- Check this README first
- Review error logs: `docker-compose logs`
- Verify environment variables are set correctly
- Ensure all services are healthy: `docker-compose ps`

## License

[Your License Here]
