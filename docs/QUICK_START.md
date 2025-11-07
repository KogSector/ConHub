# ConHub Quick Start Guide

**Version**: 2.0 (Optimized)  
**Last Updated**: December 2024

---

## Prerequisites

- **Docker** & **Docker Compose** (latest)
- **Node.js** 18+ (for frontend)
- **Rust** 1.75+ (for local development)
- **PostgreSQL** 15+ (optional, can use Docker)
- **Redis** 7+ (optional, can use Docker)

---

## Quick Start (3 Modes)

### Mode 1: Minimal Setup (Frontend Only) ⚡
**Time**: ~10 seconds  
**Use Case**: UI development, no backend dependencies

```bash
# 1. Set feature toggles
echo '{"Auth": false, "Heavy": false}' > feature-toggles.json

# 2. Start frontend
cd frontend
npm install
npm run dev

# Access at http://localhost:3000
```

**What's Running**:
- ✅ Frontend (Next.js)
- ❌ No databases
- ❌ No backend services
- ✅ Mock authentication

---

### Mode 2: Auth Testing 🔐
**Time**: ~30 seconds  
**Use Case**: Authentication flows, user management

```bash
# 1. Set feature toggles
echo '{"Auth": true, "Heavy": false}' > feature-toggles.json

# 2. Start infrastructure
docker-compose up -d postgres redis

# 3. Start auth service
cd auth
cargo run

# 4. Start backend
cd ../backend
cargo run

# 5. Start frontend
cd ../frontend
npm run dev
```

**What's Running**:
- ✅ Frontend (localhost:3000)
- ✅ Backend (localhost:8000)
- ✅ Auth Service (localhost:3010)
- ✅ PostgreSQL (localhost:5432)
- ✅ Redis (localhost:6379)
- ❌ No embedding/indexing

---

### Mode 3: Full Stack 🚀
**Time**: ~60 seconds  
**Use Case**: Production-like environment, all features

```bash
# 1. Set feature toggles
echo '{"Auth": true, "Heavy": true}' > feature-toggles.json

# 2. Start everything
docker-compose up

# Or with rebuild:
docker-compose up --build

# Access at http://localhost:80 (via Nginx)
```

**What's Running**:
- ✅ All microservices
- ✅ All databases (PostgreSQL, Redis, Qdrant)
- ✅ Embedding service
- ✅ Indexing service
- ✅ Nginx API Gateway

---

## Common Commands

### Development

```bash
# Build entire workspace
cargo build --workspace

# Build specific service
cargo build -p conhub-backend

# Run tests
cargo test --workspace

# Watch mode (auto-reload)
cd backend
cargo watch -x run

# Check code
cargo clippy --workspace
cargo fmt --workspace
```

### Docker

```bash
# Build all services
docker-compose build

# Build specific service
docker-compose build backend

# Start services
docker-compose up -d

# View logs
docker-compose logs -f backend

# Restart service
docker-compose restart backend

# Stop all
docker-compose down

# Clean up (including volumes)
docker-compose down -v
```

### Feature Toggles

```bash
# Disable auth (fast development)
echo '{"Auth": false, "Heavy": false}' > feature-toggles.json

# Enable auth, disable heavy operations
echo '{"Auth": true, "Heavy": false}' > feature-toggles.json

# Full production mode
echo '{"Auth": true, "Heavy": true}' > feature-toggles.json

# Hot reload (if endpoint implemented)
curl -X POST http://localhost:8000/admin/reload-toggles
```

---

## Service Ports

| Service | Local Port | Docker Port | Health Check |
|---------|------------|-------------|--------------|
| Frontend | 3000 | 3000 | - |
| Backend | 8000 | 8000 | /health |
| Auth | 3010 | 8001 | /health |
| Billing | 3011 | 8002 | /health |
| Security | 3012 | 8003 | /health |
| Data | 3013 | 8004 | /health |
| Client (AI) | 3014 | 8005 | /health |
| Webhook | 3015 | 8006 | /health |
| Plugins | 3020 | 8007 | /health |
| Embedding | 8082 | 8082 | /health |
| Indexers | 8080 | 8080 | /health |
| PostgreSQL | 5432 | 5432 | - |
| Redis | 6379 | 6379 | - |
| Qdrant | 6333/6334 | 6333/6334 | /health |
| Nginx | 80 | 80 | - |

---

## GraphQL Playground

Access the interactive GraphQL playground:

```
http://localhost:8000/api/graphql
```

### Example Queries

```graphql
# Health check
query {
  health
  version
}

# Get current user
query {
  me {
    user_id
    roles
  }
}

# Generate embeddings (requires Heavy: true)
query {
  embed(
    texts: ["Hello world", "Rust is awesome"]
    normalize: true
  ) {
    embeddings
    dimension
    model
    count
  }
}

# Rerank documents
query {
  rerank(
    query: "best programming language"
    documents: [
      { id: "1", text: "Python is great" }
      { id: "2", text: "Rust is fast" }
    ]
    top_k: 2
  ) {
    id
    score
    index
  }
}
```

---

## Environment Variables

Create a `.env` file in the root:

```bash
# Database
DATABASE_URL=postgresql://conhub:conhub_password@localhost:5432/conhub

# Redis
REDIS_URL=redis://localhost:6379

# Qdrant
QDRANT_URL=http://localhost:6333

# Auth
JWT_SECRET=your-super-secret-jwt-key-change-in-production
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
GITHUB_CLIENT_ID=your-github-client-id
GITHUB_CLIENT_SECRET=your-github-client-secret

# Embedding Service
EMBEDDING_SERVICE_URL=http://localhost:8082

# Feature Toggles
FEATURE_TOGGLES_PATH=./feature-toggles.json

# External APIs (optional)
OPENAI_API_KEY=your-openai-api-key
ANTHROPIC_API_KEY=your-anthropic-api-key
STRIPE_SECRET_KEY=your-stripe-secret-key
```

---

## Troubleshooting

### Issue: Port already in use

```bash
# Find process using port
lsof -i :8000  # macOS/Linux
netstat -ano | findstr :8000  # Windows

# Kill process
kill -9 <PID>  # macOS/Linux
taskkill /PID <PID> /F  # Windows
```

### Issue: Database connection failed

```bash
# Check if PostgreSQL is running
docker-compose ps postgres

# Restart PostgreSQL
docker-compose restart postgres

# Check logs
docker-compose logs postgres

# Reset database
docker-compose down -v
docker-compose up postgres
```

### Issue: Cargo build fails

```bash
# Clean build cache
cargo clean

# Update Rust
rustup update stable

# Check workspace
cargo check --workspace

# Build dependencies only
cargo build --workspace --release
```

### Issue: Feature toggles not working

```bash
# Verify file exists
cat feature-toggles.json

# Check environment variable
echo $FEATURE_TOGGLES_PATH

# Check service logs
docker-compose logs backend | grep "Feature"
```

### Issue: Docker build is slow

```bash
# Use BuildKit
export DOCKER_BUILDKIT=1

# Build with cache
docker-compose build --parallel

# Clean old images
docker image prune -a
```

---

## Development Workflow

### 1. Start New Feature

```bash
# 1. Create feature branch
git checkout -b feature/my-feature

# 2. Set minimal mode
echo '{"Auth": false, "Heavy": false}' > feature-toggles.json

# 3. Start frontend
npm run dev:frontend
```

### 2. Test with Auth

```bash
# 1. Enable auth
echo '{"Auth": true, "Heavy": false}' > feature-toggles.json

# 2. Start services
docker-compose up -d postgres redis
cargo run -p auth-service &
cargo run -p conhub-backend &
```

### 3. Full Integration Test

```bash
# 1. Enable everything
echo '{"Auth": true, "Heavy": true}' > feature-toggles.json

# 2. Start all services
docker-compose up --build

# 3. Run tests
cargo test --workspace
npm test
```

### 4. Deploy

```bash
# 1. Build production images
docker-compose -f docker-compose.prod.yml build

# 2. Push to registry
docker-compose push

# 3. Deploy to Azure
./deploy-to-azure.ps1 -Environment production
```

---

## Performance Tips

### Build Performance

```bash
# Use sccache for faster builds
cargo install sccache
export RUSTC_WRAPPER=sccache

# Parallel compilation
export CARGO_BUILD_JOBS=8

# Link faster
export CARGO_INCREMENTAL=1
```

### Docker Performance

```bash
# Use BuildKit
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# Increase resources (Docker Desktop)
# Settings > Resources > Advanced
# - CPUs: 6+
# - Memory: 8GB+
# - Swap: 2GB+
```

### Runtime Performance

```bash
# Monitor resource usage
docker stats

# Check connection pools
curl http://localhost:8000/admin/pool-stats

# Check cache stats
curl http://localhost:8000/admin/cache-stats

# Monitor health
watch -n 5 "curl -s http://localhost:8000/health | jq"
```

---

## Next Steps

1. **Read Documentation**:
   - `README.md` - Project overview
   - `OPTIMIZATIONS.md` - Architecture details
   - `MIGRATION_GUIDE.md` - Upgrade guide

2. **Explore GraphQL**:
   - Open `http://localhost:8000/api/graphql`
   - Try example queries
   - Read schema documentation

3. **Connect Data Sources**:
   - Navigate to frontend
   - Connect GitHub repository
   - Upload documents
   - Test search and embedding

4. **Integrate AI Agent**:
   - Set up MCP connection
   - Configure plugin
   - Test context retrieval

---

## Getting Help

- **Documentation**: `/docs` folder
- **Health Checks**: `http://localhost:[port]/health`
- **Logs**: `docker-compose logs -f <service>`
- **GraphQL**: `http://localhost:8000/api/graphql`

---

## Success Checklist

After setup, verify:

- [ ] Frontend loads at `http://localhost:3000`
- [ ] Backend health check: `curl http://localhost:8000/health`
- [ ] GraphQL playground accessible
- [ ] Feature toggles work (test both modes)
- [ ] Database connections successful
- [ ] Cache is working (check hit rates)
- [ ] All services show "healthy" status

---

**Happy Coding! 🚀**

For detailed architecture information, see:
- `ARCHITECTURE_SUMMARY.md` - Complete architecture overview
- `IMPLEMENTATION_STATUS.md` - Optimization details
