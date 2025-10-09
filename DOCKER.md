# ConHub Docker Configuration

Complete Docker setup for ConHub with all services containerized for development and production environments.

## 🏗️ Architecture Overview

ConHub consists of 7 containerized services:

- **Frontend** (Next.js, Port 3000) - User interface
- **Backend** (Rust, Port 3001) - Core API and business logic
- **Lexor** (Rust, Port 3002) - Code indexing service
- **AI Service** (Python, Port 8001) - Document processing and AI
- **MCP Service** (Node.js, Port 3004) - AI agent connectivity hub
- **LangChain Service** (Node.js, Port 8003) - LangChain integrations
- **PostgreSQL** (Port 5432) - Primary database
- **Redis** (Port 6379) - Caching and sessions

## 🚀 Quick Start

### Development Environment

1. **Prerequisites**
   ```bash
   # Ensure Docker Desktop is installed and running
   docker --version
   docker-compose --version
   ```

2. **Environment Setup**
   ```bash
   # Copy environment template
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Start All Services**
   ```bash
   # Using PowerShell script (Windows)
   .\scripts\docker\docker-dev.ps1 up -Build
   
   # Or using docker-compose directly
   docker-compose up -d --build
   ```

4. **Access Services**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:3001
   - Lexor: http://localhost:3002
   - MCP Service: http://localhost:3004
   - AI Service: http://localhost:8001
   - LangChain: http://localhost:8003

### Production Environment

```bash
# Start production stack
docker-compose -f docker-compose.prod.yml up -d --build

# With Nginx reverse proxy
docker-compose -f docker-compose.prod.yml up -d
```

## 📋 Service Details

### Frontend Service
- **Image**: Multi-stage Next.js build
- **Port**: 3000
- **Features**: 
  - Optimized production build
  - Static file serving
  - Environment variable injection

### Backend Service
- **Image**: Multi-stage Rust build
- **Port**: 3001
- **Features**:
  - Stripe payment integration
  - Email service with SMTP
  - JWT authentication
  - Database migrations
  - Health checks

### Lexor Service
- **Image**: Rust indexing engine
- **Port**: 3002
- **Features**:
  - Code indexing and search
  - Tantivy search engine
  - Persistent data volumes

### AI Service
- **Image**: Python FastAPI
- **Port**: 8001
- **Features**:
  - Document processing
  - Vector embeddings
  - Qdrant integration
  - OpenAI API integration

### MCP Service
- **Image**: Node.js Express
- **Port**: 3004
- **Features**:
  - Model Context Protocol
  - AI agent connectivity
  - WebSocket support
  - Webhook processing

### LangChain Service
- **Image**: Node.js TypeScript
- **Port**: 8003
- **Features**:
  - LangChain integrations
  - AI workflow orchestration

## 🛠️ Development Commands

### Using PowerShell Script (Recommended)

```powershell
# Start all services
.\scripts\docker\docker-dev.ps1 up

# Start with rebuild
.\scripts\docker\docker-dev.ps1 up -Build

# Start specific service
.\scripts\docker\docker-dev.ps1 up -Service backend

# View logs
.\scripts\docker\docker-dev.ps1 logs

# View logs for specific service
.\scripts\docker\docker-dev.ps1 logs -Service frontend

# Restart services
.\scripts\docker\docker-dev.ps1 restart

# Stop services
.\scripts\docker\docker-dev.ps1 down

# Clean environment
.\scripts\docker\docker-dev.ps1 clean
```

### Using Docker Compose Directly

```bash
# Start all services
docker-compose up -d

# Build and start
docker-compose up -d --build

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Remove volumes
docker-compose down -v
```

## 🔧 Configuration

### Environment Variables

Key environment variables for Docker deployment:

```env
# Database
POSTGRES_DB=conhub
POSTGRES_USER=conhub
POSTGRES_PASSWORD=your_secure_password

# Redis
REDIS_URL=redis://redis:6379

# Backend
JWT_SECRET=your_jwt_secret
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...

# Email
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password

# AI Services
OPENAI_API_KEY=sk-...
QDRANT_URL=https://your-cluster.qdrant.tech
QDRANT_API_KEY=your_qdrant_key

# Frontend
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY=pk_test_...
```

### Volume Mounts

Persistent data volumes:
- `postgres_data` - Database data
- `redis_data` - Redis cache
- `lexor_data` - Lexor index files
- `ai_service_data` - AI service models and data

## 🏥 Health Checks

All services include health checks:

```bash
# Check service health
docker-compose ps

# View health status
docker inspect conhub-backend --format='{{.State.Health.Status}}'
```

Health check endpoints:
- Backend: `GET /health`
- Frontend: `GET /api/health`
- Lexor: `GET /health`
- MCP Service: `GET /api/health`
- AI Service: `GET /health`
- LangChain: `GET /health`

## 🔒 Security Features

### Container Security
- Non-root users in all containers
- Minimal base images (Alpine/Slim)
- Security headers in Nginx
- Resource limits and reservations

### Network Security
- Internal Docker network
- Service-to-service communication
- Rate limiting in Nginx
- CORS configuration

### Data Security
- Encrypted environment variables
- Secure volume mounts
- Database connection encryption
- JWT token validation

## 📊 Monitoring & Logging

### Log Management
```bash
# View all logs
docker-compose logs -f

# View specific service logs
docker-compose logs -f backend

# Follow logs with timestamps
docker-compose logs -f -t
```

### Resource Monitoring
```bash
# View resource usage
docker stats

# View container processes
docker-compose top
```

## 🚀 Production Deployment

### Production Stack
```bash
# Deploy production environment
docker-compose -f docker-compose.prod.yml up -d --build

# Scale services
docker-compose -f docker-compose.prod.yml up -d --scale backend=2

# Update specific service
docker-compose -f docker-compose.prod.yml up -d --no-deps backend
```

### Nginx Reverse Proxy
- Load balancing
- SSL termination
- Static file serving
- Rate limiting
- Security headers

### Resource Limits
Production containers include resource limits:
- Backend: 1GB memory limit
- AI Service: 2GB memory limit
- Frontend: 512MB memory limit
- Database: 512MB memory limit

## 🔧 Troubleshooting

### Common Issues

1. **Port Conflicts**
   ```bash
   # Check port usage
   netstat -ano | findstr :3000
   
   # Kill process using port
   taskkill /PID <process_id> /F
   ```

2. **Build Failures**
   ```bash
   # Clean build cache
   docker builder prune
   
   # Rebuild without cache
   docker-compose build --no-cache
   ```

3. **Database Connection Issues**
   ```bash
   # Check database logs
   docker-compose logs postgres
   
   # Connect to database
   docker-compose exec postgres psql -U conhub -d conhub
   ```

4. **Service Health Issues**
   ```bash
   # Check service status
   docker-compose ps
   
   # Restart unhealthy service
   docker-compose restart backend
   ```

### Debug Mode
```bash
# Run with debug logging
RUST_LOG=debug docker-compose up

# Run single service in foreground
docker-compose run --rm backend
```

## 📈 Performance Optimization

### Build Optimization
- Multi-stage builds for smaller images
- Layer caching for faster builds
- Dependency caching
- Minimal base images

### Runtime Optimization
- Resource limits and reservations
- Health checks for reliability
- Restart policies
- Volume optimization

### Network Optimization
- Service mesh communication
- Connection pooling
- Caching strategies
- CDN integration (production)

## 🔄 CI/CD Integration

### GitHub Actions Example
```yaml
name: Docker Build and Deploy
on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build and test
        run: |
          docker-compose -f docker-compose.prod.yml build
          docker-compose -f docker-compose.prod.yml up -d
          # Run tests
          docker-compose -f docker-compose.prod.yml down
```

## 📚 Additional Resources

- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Multi-stage Builds](https://docs.docker.com/develop/dev-best-practices/#use-multi-stage-builds)
- [Container Security](https://docs.docker.com/engine/security/)

---

**Note**: This Docker setup provides a complete containerized environment for ConHub with development and production configurations, comprehensive health checks, security features, and monitoring capabilities.
