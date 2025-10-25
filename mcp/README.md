# MCP (Model Context Protocol) Components

This folder contains all MCP-related services for ConHub. The architecture separates the core MCP service from individual provider implementations.

## Structure

```
mcp/
├── service/              # Main MCP protocol service
│   ├── Dockerfile
│   └── src/
│       ├── rules/        # Agent rules and validation
│       ├── logic/        # Connection and protocol logic
│       ├── protocols/    # MCP protocol implementation
│       └── ...
│
└── servers/              # MCP server implementations
    ├── google-drive/     # Google Drive MCP server
    │   ├── Dockerfile
    │   ├── package.json
    │   └── server.js
    │
    ├── dropbox/          # Dropbox MCP server
    │   ├── Dockerfile
    │   ├── package.json
    │   └── server.js
    │
    ├── filesystem/       # Filesystem MCP server
    │   ├── Dockerfile
    │   ├── package.json
    │   └── server.js
    │
    └── start-all.sh      # Script to start all servers
```

## Components

### MCP Service (Port 3004)

**Purpose:** Core MCP protocol service that manages connections to various AI agents and MCP servers.

**Technology:** Node.js

**Key Features:**
- MCP protocol implementation
- Agent rule management
- Connection pooling
- WebSocket support for real-time communication
- Integration with AI coding assistants (GitHub Copilot, Amazon Q, Cline)

**Environment Variables:**
- `MCP_SERVICE_PORT` - Service port (default: 3004)
- `REDIS_URL` - Redis connection for session management
- `AUTH_SERVICE_URL` - Auth service endpoint
- `GITHUB_COPILOT_WEBHOOK_SECRET` - GitHub Copilot webhook verification
- `AMAZON_Q_WEBHOOK_SECRET` - Amazon Q webhook verification
- `CLINE_WEBHOOK_SECRET` - Cline webhook verification
- `JWT_SECRET` - JWT token signing secret

**Endpoints:**
- Health check, agent connections, MCP protocol endpoints

### MCP Servers

Individual server implementations that provide MCP protocol access to different data sources.

#### Google Drive Server (Port 3005)

**Purpose:** MCP protocol interface for Google Drive

**Environment Variables:**
- `PORT` - Server port (3005)
- `GOOGLE_DRIVE_CLIENT_ID` - OAuth client ID
- `GOOGLE_DRIVE_CLIENT_SECRET` - OAuth client secret

**Features:**
- List files and folders
- Read file contents
- Search files
- OAuth2 authentication

#### Dropbox Server (Port 3006)

**Purpose:** MCP protocol interface for Dropbox

**Environment Variables:**
- `PORT` - Server port (3006)
- `DROPBOX_APP_KEY` - Dropbox app key
- `DROPBOX_APP_SECRET` - Dropbox app secret

**Features:**
- List files and folders
- Read file contents
- Search files
- OAuth2 authentication

#### Filesystem Server (Port 3007)

**Purpose:** MCP protocol interface for local filesystem

**Environment Variables:**
- `PORT` - Server port (3007)
- `MCP_FILESYSTEM_ROOT_PATH` - Root directory path (default: /data)

**Features:**
- List files and directories
- Read file contents
- Watch file changes
- Search files

**Volume Mount:**
- `./data:/data` - Mount local data directory

## Development

### Running the MCP Service

```bash
# Using Docker
cd mcp/service
docker build -t mcp-service .
docker run -p 3004:3004 --env-file ../../.env mcp-service

# Local development
cd mcp/service
npm install
npm start
```

### Running Individual MCP Servers

```bash
# Google Drive Server
cd mcp/servers/google-drive
docker build -t mcp-google-drive .
docker run -p 3005:3005 --env-file ../../../.env mcp-google-drive

# Dropbox Server
cd mcp/servers/dropbox
docker build -t mcp-dropbox .
docker run -p 3006:3006 --env-file ../../../.env mcp-dropbox

# Filesystem Server
cd mcp/servers/filesystem
docker build -t mcp-filesystem .
docker run -p 3007:3007 -v $(pwd)/data:/data --env-file ../../../.env mcp-filesystem
```

### Running All Servers

```bash
cd mcp/servers
./start-all.sh
```

## Docker Compose

All MCP components are included in the main `docker-compose.yml`:

```bash
# Start all MCP services
docker-compose up -d mcp-service mcp-google-drive mcp-dropbox mcp-filesystem

# View logs
docker-compose logs -f mcp-service
docker-compose logs -f mcp-google-drive

# Restart specific service
docker-compose restart mcp-service
```

## API Documentation

### MCP Service Endpoints

**Health Check:**
```bash
GET http://localhost:3004/health
```

**Agent Connection:**
```bash
POST http://localhost:3004/mcp/connect
Content-Type: application/json

{
  "agent": "github-copilot",
  "token": "your-jwt-token"
}
```

**MCP Protocol:**
```bash
WebSocket: ws://localhost:3004/mcp/protocol
```

### MCP Server Endpoints

All MCP servers follow the MCP protocol specification. Common operations:

**List Resources:**
```bash
POST http://localhost:3005/mcp/list
Content-Type: application/json

{
  "path": "/documents"
}
```

**Read Resource:**
```bash
POST http://localhost:3005/mcp/read
Content-Type: application/json

{
  "path": "/documents/file.txt"
}
```

## Architecture

### Communication Flow

```
AI Agent (Copilot/Q/Cline)
    ↓
MCP Service (3004)
    ↓
┌─────────────┬─────────────┬─────────────┐
│ Google Drive│   Dropbox   │ Filesystem  │
│    (3005)   │    (3006)   │   (3007)    │
└─────────────┴─────────────┴─────────────┘
```

### Integration Points

- **Auth Service** - User authentication and authorization
- **Redis** - Session management and caching
- **Backend Services** - Data synchronization and webhooks

## Security

### Authentication

- JWT tokens for service-to-service communication
- OAuth2 for Google Drive and Dropbox
- Webhook signature verification for AI agents

### Access Control

- MCP service validates all requests
- Agent-specific rules and permissions
- Rate limiting on connections

## Troubleshooting

### MCP Service Not Starting

**Check logs:**
```bash
docker-compose logs mcp-service
```

**Common issues:**
- Redis connection failed - Ensure Redis is running
- Auth service unavailable - Ensure auth-service is healthy
- Invalid JWT secret - Check JWT_SECRET environment variable

### MCP Server Connection Issues

**Google Drive:**
- Verify OAuth credentials
- Check GOOGLE_DRIVE_CLIENT_ID and GOOGLE_DRIVE_CLIENT_SECRET
- Ensure OAuth redirect URI is configured

**Dropbox:**
- Verify app key and secret
- Check DROPBOX_APP_KEY and DROPBOX_APP_SECRET
- Ensure app has required permissions

**Filesystem:**
- Verify volume mount: `./data:/data`
- Check file permissions
- Ensure MCP_FILESYSTEM_ROOT_PATH is correct

### WebSocket Connection Drops

**Solutions:**
- Check Nginx WebSocket configuration
- Increase `proxy_read_timeout` in nginx.conf
- Verify network stability
- Check Redis connection pool

## Testing

### Manual Testing

```bash
# Test MCP service health
curl http://localhost:3004/health

# Test Google Drive server
curl http://localhost:3005/health

# Test WebSocket connection
wscat -c ws://localhost:3004/mcp/protocol
```

### Integration Testing

```bash
# Test full flow
npm run test:integration

# Test specific server
npm run test:google-drive
```

## Adding New MCP Servers

To add a new MCP server implementation:

1. **Create directory:**
   ```bash
   mkdir -p mcp/servers/new-provider
   cd mcp/servers/new-provider
   ```

2. **Create package.json:**
   ```json
   {
     "name": "mcp-new-provider",
     "version": "1.0.0",
     "main": "server.js",
     "dependencies": {
       "express": "^4.18.0"
     }
   }
   ```

3. **Create server.js:**
   ```javascript
   const express = require('express');
   const app = express();
   const PORT = process.env.PORT || 3008;

   // Implement MCP protocol endpoints
   app.post('/mcp/list', (req, res) => {
     // Implementation
   });

   app.listen(PORT, () => {
     console.log(`MCP New Provider server on ${PORT}`);
   });
   ```

4. **Create Dockerfile:**
   ```dockerfile
   FROM node:18-alpine
   WORKDIR /app
   COPY package*.json ./
   RUN npm ci --only=production
   COPY . .
   EXPOSE 3008
   CMD ["node", "server.js"]
   ```

5. **Add to docker-compose.yml:**
   ```yaml
   mcp-new-provider:
     build:
       context: ./mcp/servers/new-provider
       dockerfile: Dockerfile
     container_name: conhub-mcp-new-provider
     ports:
       - "3008:3008"
     environment:
       - NODE_ENV=production
       - PORT=3008
     networks:
       - conhub-network
   ```

## Monitoring

### Logs

```bash
# MCP service logs
docker-compose logs -f mcp-service

# All MCP servers
docker-compose logs -f mcp-google-drive mcp-dropbox mcp-filesystem

# Tail last 100 lines
docker-compose logs --tail=100 mcp-service
```

### Metrics

MCP service exposes metrics on `/metrics` endpoint (if configured).

### Health Checks

All services expose `/health` endpoints:
- MCP Service: http://localhost:3004/health
- Google Drive: http://localhost:3005/health
- Dropbox: http://localhost:3006/health
- Filesystem: http://localhost:3007/health

## Production Considerations

1. **Scalability:**
   - MCP service can be horizontally scaled
   - Use Redis for session sharing across instances
   - Consider load balancer for multiple instances

2. **Security:**
   - Enable HTTPS in production
   - Use strong secrets for webhook verification
   - Implement rate limiting
   - Regular security audits

3. **Monitoring:**
   - Set up logging aggregation
   - Configure alerting for service failures
   - Monitor WebSocket connection counts
   - Track API usage per agent

4. **Backup:**
   - MCP service is stateless (uses Redis)
   - Back up Redis data if needed
   - No persistent storage in MCP servers

## Support

For issues or questions about MCP components:
- Check logs first: `docker-compose logs mcp-service`
- Verify environment variables are set
- Ensure all dependencies are running (Redis, auth-service)
- Review MCP protocol specification
