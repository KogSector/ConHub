# ConHub Docker Management Scripts

This directory contains enhanced Docker management scripts for ConHub development. These scripts provide intelligent container management with automatic setup, build optimization, and clean shutdown capabilities.

## 🚀 Quick Start

### For New Developers (First Time Setup)
```bash
npm start
# or
npm run docker:setup
```
This will:
- Check Docker installation and status
- Set up environment variables
- Build containers (only if needed)
- Start all ConHub services
- Display service URLs and status

### For Existing Developers (Daily Use)
```bash
npm start
# or
npm run docker:start
```
This will:
- Detect existing containers
- Start containers without rebuilding (faster)
- Display service status and URLs

### Stop All Services
```bash
npm stop
# or
npm run docker:stop
```
This will:
- Stop all containers gracefully
- Preserve containers for quick restart
- Keep Docker images intact

## 📋 Available Scripts

### Main Commands

| Command | Description | Use Case |
|---------|-------------|----------|
| `npm start` | Smart setup and run | Daily development |
| `npm stop` | Clean stop | End of work session |
| `npm run docker:status` | Check service status | Troubleshooting |

### Advanced Commands

| Command | Description | When to Use |
|---------|-------------|-------------|
| `npm run docker:setup` | Full setup and run | First time or after cleanup |
| `npm run docker:start -- --start-only` | Start existing containers only | Quick restart |
| `npm run docker:rebuild` | Force rebuild all containers | After major changes |
| `npm run docker:stop -- --force` | Force stop with cleanup | When containers are stuck |
| `npm run docker:stop -- --cleanup` | Complete cleanup | Reset environment |
| `npm run docker:logs` | View all service logs | Debugging |

## 🔧 Script Features

### setup-and-run.js
- **Intelligent Detection**: Automatically detects if containers exist
- **Conditional Building**: Only builds when necessary
- **Environment Management**: Handles `.env` file configuration
- **Service Health Checks**: Waits for services to be ready
- **Comprehensive Logging**: Clear progress indicators and status updates
- **Error Recovery**: Provides troubleshooting suggestions

### stop.js
- **Graceful Shutdown**: Stops containers cleanly
- **Preservation Mode**: Keeps containers for quick restart (default)
- **Force Mode**: Removes orphaned containers (`--force`)
- **Cleanup Mode**: Complete environment reset (`--cleanup`)
- **Status Reporting**: Shows remaining containers after stop

## 🌐 Service URLs

After successful startup, these services will be available:

### Core Services
- **Frontend**: http://localhost:3000
- **API Gateway**: http://localhost:8080

### Backend Services
- **Auth Service**: http://localhost:8001
- **Billing Service**: http://localhost:8002
- **AI Service**: http://localhost:8003
- **Data Service**: http://localhost:8004
- **Security Service**: http://localhost:8005
- **Webhook Service**: http://localhost:8006

### Infrastructure
- **PostgreSQL**: localhost:5432
- **Redis**: localhost:6379
- **MinIO**: http://localhost:9000 (Console: http://localhost:9001)

### MCP Services
- **MCP Service**: http://localhost:8010
- **MCP Dropbox**: http://localhost:8011

## 🛠 Troubleshooting

### Docker Not Running
```bash
# Check Docker status
docker --version
docker info

# Start Docker Desktop manually if needed
```

### Containers Won't Start
```bash
# Force rebuild
npm run docker:rebuild

# Check logs
npm run docker:logs

# Complete reset
npm run docker:stop -- --cleanup
npm start
```

### Port Conflicts
```bash
# Stop all containers
npm stop

# Clean up ports
npm run cleanup

# Restart
npm start
```

### Build Failures
```bash
# Force clean rebuild
npm run docker:stop -- --cleanup
npm run docker:rebuild
```

## 📁 File Structure

```
scripts/docker/
├── setup-and-run.js    # Main setup and run script
├── stop.js             # Enhanced stop script
├── status.js           # Service status checker
├── start-all.js        # Legacy start script (preserved)
├── stop-all.js         # Legacy stop script (preserved)
├── docker-dev.js       # Development utilities
└── README.md           # This documentation
```

## 🔄 Migration from Legacy Scripts

The new scripts are backward compatible. Old commands still work:

| Old Command | New Equivalent | Recommendation |
|-------------|----------------|----------------|
| `npm run docker:start` | `npm start` | Use new command |
| `npm run docker:stop` | `npm stop` | Use new command |
| `docker-compose up` | `npm start` | Use npm script |
| `docker-compose down` | `npm stop` | Use npm script |

## ⚡ Performance Tips

1. **Daily Development**: Use `npm start` - it's optimized for speed
2. **After Git Pull**: Use `npm run docker:rebuild` if dependencies changed
3. **Clean Environment**: Use `npm run docker:stop -- --cleanup` monthly
4. **Debugging**: Use `npm run docker:logs` to monitor all services

## 🔒 Environment Variables

The scripts automatically manage these environment variables:
- `ENV_MODE=docker` (set automatically)
- Database connection strings
- Service discovery URLs
- Authentication tokens (if configured)

## 📞 Support

If you encounter issues:
1. Check the troubleshooting section above
2. Run `npm run docker:status` to diagnose
3. Use `npm run docker:logs` to see detailed logs
4. Try `npm run docker:stop -- --cleanup && npm start` for a fresh start

---

**Note**: These scripts require Docker Desktop to be installed and running. The scripts will guide you through any setup issues.