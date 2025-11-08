# Docker Toggle Feature Documentation

## Overview

The Docker toggle feature provides developers with fine-grained control over whether ConHub runs in Docker containerized mode or local development mode. This is controlled via the `feature-toggles.json` file at the project root.

## Feature Toggle Configuration

### Location
`c:\Users\risha\Desktop\Work\ConHub\feature-toggles.json`

### Available Toggles

```json
{
  "Auth": false,      // Controls authentication and database connections
  "Heavy": false,     // Controls heavy operations (embedding, indexing)
  "Docker": false     // Controls build/run mode (Docker vs Local)
}
```

## Docker Toggle Behavior

### When `Docker: false` (Default - Local Development)

**What Happens:**
- Services run directly on your local machine
- No Docker containers are built or started
- Faster startup time for development
- Direct access to code changes (no rebuild needed)
- Uses `npm run dev:concurrently` to start all services

**Services Started:**
- Frontend (Next.js) - Port 3000
- Auth Service (Rust/Actix) - Port 3010
- Billing Service (Rust/Actix) - Port 3011
- Client Service (Rust/Actix) - Port 3014
- Data Service (Rust/Actix) - Port 3013
- Security Service (Rust/Actix) - Port 3012
- Webhook Service (Rust/Actix) - Port 3015
- Indexers Service (TypeScript) - Port 8080
- MCP Service (TypeScript) - Port 3004

**Environment:**
- ENV_MODE automatically set to `local`
- Database URLs use localhost
- Direct file system access
- Hot reload enabled

### When `Docker: true` (Production-like Environment)

**What Happens:**
- All services run in Docker containers
- Uses `docker-compose.yml` for orchestration
- Isolated environment with networking
- Consistent across all machines
- Uses `docker/setup-and-run.js` script

**Services Started:**
All microservices + Infrastructure:
- PostgreSQL (Port 5432)
- Redis (Port 6379)
- Qdrant Vector DB (Ports 6333, 6334)
- Nginx API Gateway (Port 80)
- All application services (containerized)

**Environment:**
- ENV_MODE automatically set to `docker`
- Database URLs use container names (postgres, redis, qdrant)
- Isolated network: `conhub-network`
- Volume mounts for persistence

## Usage

### Starting Services

#### Method 1: Using npm scripts (Recommended)
```bash
# Reads feature-toggles.json and starts accordingly
npm start
# OR
npm run dev
```

#### Method 2: Explicitly set the toggle
```bash
# For local development (edit feature-toggles.json)
{
  "Docker": false
}
npm start

# For Docker mode (edit feature-toggles.json)
{
  "Docker": true
}
npm start
```

#### Method 3: Direct script execution
```bash
# Local mode
node scripts/services/start.js

# Docker mode
node scripts/docker/setup-and-run.js
```

### Stopping Services

```bash
# Stops services based on current mode
npm run stop

# Force stop Docker containers
npm run docker:stop
```

### Checking Status

```bash
# Check service status
npm run status

# Docker-specific status
npm run docker:status
```

## Smart Start Script

The `scripts/smart-start.js` script is the intelligent entry point that:

1. **Reads** `feature-toggles.json`
2. **Displays** current toggle status
3. **Routes** to appropriate start script based on Docker toggle
4. **Sets** environment variables automatically

### Flow Diagram

```
npm start
    ↓
smart-start.js
    ↓
Read feature-toggles.json
    ↓
Docker: true? ───Yes──→ docker/setup-and-run.js ──→ Docker Compose Up
    ↓
    No
    ↓
services/start.js ──→ Concurrently (Local Services)
```

## Integration with Other Toggles

### Auth Toggle Interaction

- **Auth: false, Docker: false** → Local dev, no databases
- **Auth: false, Docker: true** → Docker containers without auth/databases
- **Auth: true, Docker: false** → Local with databases (you need to run databases separately)
- **Auth: true, Docker: true** → Full Docker stack with auth and databases

### Heavy Toggle Interaction

- **Heavy: false** → Disables embedding and indexing (regardless of Docker mode)
- **Heavy: true** → Enables embedding and indexing (in either mode)

## Implementation Details

### Files Modified

1. **`feature-toggles.json`**
   - Added `Docker` key

2. **`frontend/lib/feature-toggles.ts`**
   - Added `isDockerEnabled()` function

3. **`shared/config/src/feature_toggles.rs`**
   - Added `docker_enabled()` method
   - Added `should_use_docker()` helper

4. **`scripts/smart-start.js`** (New)
   - Main orchestration script
   - Reads toggles and routes execution

5. **`package.json`**
   - Updated `start` script to use `smart-start.js`
   - Updated `dev` script to use `smart-start.js`

6. **`scripts/package.json`**
   - Updated `start` script to use `smart-start.js`

7. **`scripts/services/start.js`**
   - Removed automatic Docker build
   - Simplified local-only logic

## Benefits

### Developer Productivity
- **Faster Iteration**: Local mode skips Docker build time
- **Resource Efficient**: Local mode uses fewer system resources
- **Flexible Testing**: Easy switch between modes for different scenarios

### DevOps Efficiency
- **Consistent Staging**: Docker mode ensures consistent environment
- **CI/CD Ready**: Can toggle based on environment
- **Production Parity**: Docker mode matches production setup

## Troubleshooting

### Issue: Services won't start
**Solution**: Check feature-toggles.json exists and is valid JSON

### Issue: Docker mode fails
**Solution**: 
```bash
# Ensure Docker is running
docker info

# Check docker-compose.yml exists
ls docker-compose.yml

# Verify .env file
ls .env
```

### Issue: Local mode database connection errors
**Solution**: 
- Set `Auth: false` for local dev without databases
- OR start databases manually: `docker-compose up -d postgres redis qdrant`

### Issue: Port conflicts
**Solution**:
```powershell
# Windows PowerShell - Find process using port
netstat -ano | findstr :3000

# Kill process by PID
taskkill /PID <PID> /F
```

## Best Practices

### For Development
1. Use `Docker: false` for day-to-day development
2. Use `Auth: false` and `Heavy: false` for UI work
3. Periodically test with `Docker: true` to ensure compatibility

### For Testing
1. Use `Docker: true` for integration testing
2. Use full toggles enabled for end-to-end testing
3. Test both modes before committing

### For Production
1. Always use `Docker: true` in production environments
2. Enable all toggles (`Auth: true`, `Heavy: true`, `Docker: true`)
3. Use environment-specific configuration files

## Migration from Old System

### Before
```bash
# Always used Docker
npm start  # → docker-compose up
```

### After
```bash
# Intelligent routing
npm start  # → Checks feature-toggles.json → Routes accordingly
```

### Migration Steps
1. No changes needed to existing workflows
2. `npm start` now checks toggles first
3. Default behavior is local mode (`Docker: false`)
4. To restore old Docker-always behavior: Set `Docker: true`

## Environment Variables

The toggle system automatically sets:

| Mode | ENV_MODE | Database URLs | Network |
|------|----------|---------------|---------|
| Local | `local` | `localhost:*` | Host network |
| Docker | `docker` | `postgres:*`, `redis:*`, `qdrant:*` | `conhub-network` |

## Future Enhancements

Potential improvements:
- CLI flag override: `npm start --docker`
- Environment-based defaults: `NODE_ENV=production` → `Docker: true`
- Per-service toggles: Fine-grained control
- Hot toggle reload: Change mode without restart
- Configuration profiles: `dev`, `staging`, `production`

## Support

For issues or questions:
- Check logs: `npm run logs` (Docker mode)
- Review status: `npm run status`
- Clean state: `npm run clean`
- Rebuild: `npm run rebuild`

---

**Last Updated**: November 2024  
**Version**: 1.0.0
