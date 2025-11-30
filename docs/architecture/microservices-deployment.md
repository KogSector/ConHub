# ConHub Microservices Deployment Architecture

## Overview

ConHub follows a microservices architecture where each service is independently deployable and scalable. This document outlines what needs to be deployed and how.

## Deployable Services (Microservices)

### 1. **Auth Service** (Port 3010)
- **Purpose:** Authentication, JWT token management, user sessions
- **Language:** Rust
- **Dockerfile:** `auth/Dockerfile`
- **Dependencies:** PostgreSQL (NeonDB), Redis
- **Deployment:** Independent container/server

### 2. **Data Service** (Port 3013)
- **Purpose:** Data sources, connectors, repository management
- **Language:** Rust
- **Dockerfile:** `data/Dockerfile`
- **Dependencies:** PostgreSQL (NeonDB), Redis
- **Deployment:** Independent container/server

### 3. **Billing Service** (Port 3011)
- **Purpose:** Subscription management, Stripe integration, usage tracking
- **Language:** Rust
- **Dockerfile:** `billing/Dockerfile`
- **Dependencies:** PostgreSQL (NeonDB), Redis
- **Deployment:** Independent container/server

### 4. **Security Service** (Port 3014)
- **Purpose:** Security auditing, access control, threat detection
- **Language:** Rust
- **Dockerfile:** `security/Dockerfile`
- **Dependencies:** PostgreSQL (NeonDB)
- **Deployment:** Independent container/server

### 5. **Webhook Service** (Port 3015)
- **Purpose:** External webhook handling, event processing
- **Language:** Rust
- **Dockerfile:** `webhook/Dockerfile`
- **Dependencies:** PostgreSQL (NeonDB)
- **Deployment:** Independent container/server

### 6. **Client Service** (Port 3012)
- **Purpose:** AI client management, agent coordination
- **Language:** Rust
- **Dockerfile:** `client/Dockerfile`
- **Dependencies:** PostgreSQL (NeonDB), Redis
- **Deployment:** Independent container/server

### 7. **MCP Service** (Port 3004)
- **Purpose:** Model Context Protocol server, connector orchestration
- **Language:** Rust
- **Dockerfile:** `mcp/Dockerfile`
- **Dependencies:** PostgreSQL (NeonDB), Redis
- **Deployment:** Independent container/server
- **Note:** Runs both stdio (MCP protocol) and HTTP (health checks)

### 8. **Backend Service** (Port 8000)
- **Purpose:** GraphQL API gateway, unified backend
- **Language:** Rust
- **Dockerfile:** `backend/Dockerfile`
- **Dependencies:** PostgreSQL (NeonDB), Redis
- **Deployment:** Independent container/server

### 9. **Embedding Service** (Port 8082)
- **Purpose:** Text embeddings, vector generation
- **Language:** Rust
- **Dockerfile:** `embedding/Dockerfile`
- **Dependencies:** PostgreSQL (NeonDB), Redis, AI models
- **Deployment:** Independent container/server

### 10. **Frontend** (Port 3000)
- **Purpose:** Next.js web application, user interface
- **Language:** TypeScript/React
- **Dockerfile:** `frontend/Dockerfile`
- **Dependencies:** Backend API
- **Deployment:** Independent container/server

### 11. **Nginx** (Port 80/443)
- **Purpose:** Reverse proxy, load balancer, SSL termination
- **Language:** Configuration
- **Dockerfile:** `nginx/Dockerfile`
- **Dependencies:** All backend services
- **Deployment:** Independent container (production only)
- **Note:** Optional for development, required for production

## Non-Deployable Components

### **Shared Libraries**
- **Location:** `shared/` directory
- **Components:**
  - `shared/models` - Data models
  - `shared/middleware` - Common middleware
  - `shared/config` - Configuration utilities
  - `shared/utils` - Utility functions
  - `shared/plugins` - Plugin system
- **Purpose:** Code reuse across microservices
- **Deployment:** **NOT DEPLOYED** - Compiled into each service that uses them
- **Note:** These are Rust library crates, not standalone services

### **Database Library**
- **Location:** `database/` directory
- **Purpose:** Database connection pooling, migrations, cache
- **Deployment:** **NOT DEPLOYED** - Compiled into services that need database access

## Deployment Modes

### Development Mode
```bash
npm run dev
```
- All services run locally via `cargo run`
- No Docker containers
- Direct connections to cloud databases (NeonDB, Redis)
- Hot reload enabled

### Docker Mode (Currently Disabled)
```bash
docker-compose up
```
- All services run in Docker containers
- Local PostgreSQL and Redis containers
- Nginx reverse proxy
- **Status:** Disabled via `feature-toggles.json` (Docker: false)

### Production Mode
Each service deployed independently to:
- Azure Container Apps
- AWS ECS/Fargate
- Google Cloud Run
- Kubernetes cluster
- Or any container orchestration platform

## Service Dependencies

```
Frontend (3000)
    ↓
Nginx (80) [Production only]
    ↓
Backend (8000) ← GraphQL Gateway
    ↓
┌───────────────┬──────────────┬─────────────┬──────────────┐
│               │              │             │              │
Auth (3010)  Data (3013)  Billing (3011)  Security (3014)
    │            │              │             │
    └────────────┴──────────────┴─────────────┴──────────────┐
                                                              │
                                                         Webhook (3015)
                                                              │
                                                         Client (3012)
                                                              │
                                                         MCP (3004)
                                                              │
                                                         Embedding (8082)
                                                              │
                                                              ↓
                                            PostgreSQL (NeonDB) + Redis (Azure)
```

## Environment Variables

Each service requires:
- `DATABASE_URL_NEON` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string (rediss:// for TLS)
- `JWT_PUBLIC_KEY_PATH` or `JWT_PRIVATE_KEY_PATH` - JWT keys
- Service-specific variables (see the `.env` file and documentation in each service directory)

## Health Checks

All services expose `/health` endpoint:
```bash
curl http://localhost:3010/health  # Auth
curl http://localhost:3011/health  # Billing
curl http://localhost:3012/health  # Client
curl http://localhost:3013/health  # Data
curl http://localhost:3014/health  # Security
curl http://localhost:3015/health  # Webhook
curl http://localhost:3004/health  # MCP
curl http://localhost:8000/health  # Backend
curl http://localhost:8082/health  # Embedding
curl http://localhost:3000/        # Frontend
```

## Scaling Strategy

### Horizontal Scaling (Multiple Instances)
- ✅ Auth Service - Stateless, can scale
- ✅ Data Service - Stateless, can scale
- ✅ Billing Service - Stateless, can scale
- ✅ Security Service - Stateless, can scale
- ✅ Webhook Service - Stateless, can scale
- ✅ Client Service - Stateless, can scale
- ✅ MCP Service - Stateless, can scale
- ✅ Backend Service - Stateless, can scale
- ⚠️  Embedding Service - Resource intensive, scale carefully
- ✅ Frontend - Stateless, can scale

### Vertical Scaling (More Resources)
- Embedding Service - Requires more CPU/RAM for AI models
- Database - Managed by NeonDB (auto-scaling)
- Redis - Managed by Azure Redis Labs (auto-scaling)

## Nginx Configuration

Nginx acts as a reverse proxy in production:
- Routes `/api/*` to Backend (8000)
- Routes `/mcp/*` to MCP Service (3004)
- Routes `/*` to Frontend (3000)
- Handles SSL/TLS termination
- Provides rate limiting
- Adds security headers

**Development:** Nginx is optional, services accessed directly
**Production:** Nginx is required for unified entry point

## Build and Deploy Commands

### Build All Services
```bash
# Rust services
cargo build --release --manifest-path auth/Cargo.toml
cargo build --release --manifest-path data/Cargo.toml
cargo build --release --manifest-path billing/Cargo.toml
cargo build --release --manifest-path security/Cargo.toml
cargo build --release --manifest-path webhook/Cargo.toml
cargo build --release --manifest-path client/Cargo.toml
cargo build --release --manifest-path mcp/Cargo.toml
cargo build --release --manifest-path backend/Cargo.toml
cargo build --release --manifest-path embedding/Cargo.toml

# Frontend
cd frontend && npm run build
```

### Docker Build (When Enabled)
```bash
docker-compose build
```

### Individual Service Deploy
```bash
# Example: Deploy auth service
docker build -t conhub-auth:latest -f auth/Dockerfile .
docker push conhub-auth:latest
```

## Monitoring and Observability

Each service logs to stdout/stderr:
- Structured logging with `tracing` (Rust services)
- JSON logs for easy parsing
- Health check endpoints for liveness probes
- Metrics endpoints (future enhancement)

## Summary

**Deployable Services:** 11 (10 backend + 1 frontend + nginx)
**Non-Deployable:** 2 (shared libraries + database library)
**Total Microservices Running:** 10-11 (nginx optional in dev)

All services are independently deployable, scalable, and maintainable. The shared libraries are compiled into each service at build time, not deployed separately.
