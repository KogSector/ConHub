# GitHub Copilot Integration with ConHub MCP

This document describes the integration between GitHub Copilot and ConHub's Model Context Protocol (MCP) implementation, enabling Copilot to access structured context and resources through standardized protocols.

## Overview

The GitHub Copilot integration allows Copilot to:
- Access structured context from ConHub's repositories, documents, and data sources
- Execute tools and operations through the MCP protocol
- Maintain secure, session-based connections
- Receive real-time updates and streaming responses

## Architecture

```
GitHub Copilot → ConHub API Endpoints → MCP Integration Service → MCP Server → Context Providers
```

### Components

1. **GitHubCopilotIntegration Service** (`backend/src/services/github_copilot_integration.rs`)
   - Core integration service managing Copilot sessions
   - Session management with authentication and rate limiting
   - Context request handling and formatting
   - Tool execution through MCP

2. **Copilot API Handlers** (`backend/src/handlers/github_copilot.rs`)
   - REST API endpoints for Copilot communication
   - Request validation and response formatting
   - Error handling and status reporting

3. **MCP Protocol Bridge**
   - Translates Copilot requests to MCP messages
   - Formats MCP responses for Copilot consumption
   - Maintains protocol compatibility

## API Endpoints

### Authentication & Session Management

#### Initialize Session
```http
POST /api/copilot/session
Content-Type: application/json

{
  "user_id": "string",
  "workspace_id": "optional_string", 
  "auth_token": "github_token"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "session_id": "uuid",
    "expires_at": "2025-09-17T12:00:00Z",
    "capabilities": {
      "context_providers": ["repository", "document", "url", "data_source"],
      "tools": ["search", "analyze", "context_create", "resource_read"],
      "resources": ["files", "documentation", "api_specs", "database_schemas"],
      "real_time_updates": true,
      "streaming_support": true
    }
  }
}
```

#### Get Capabilities
```http
GET /api/copilot/capabilities
```

### Context Access

#### Request Context
```http
POST /api/copilot/context
Content-Type: application/json

{
  "session_id": "uuid",
  "context_type": "repository|document|url|data_source",
  "query": "optional search query",
  "workspace_path": "optional/path/filter",
  "file_patterns": ["*.rs", "*.ts"],
  "include_dependencies": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "context_id": "uuid",
    "name": "Repository Context",
    "type": "repository",
    "description": "ConHub repository context",
    "created_at": "2025-09-17T12:00:00Z",
    "resources": [
      {
        "id": "resource_uuid",
        "name": "main.rs",
        "content": "file content...",
        "relevance_score": 0.95,
        "metadata": {
          "file_type": "rust",
          "size": 1024,
          "last_modified": "2025-09-17T11:00:00Z"
        }
      }
    ],
    "copilot_metadata": {
      "total_resources": 42,
      "context_size": 50000,
      "supports_streaming": true,
      "cache_ttl": 300
    }
  },
  "context_provided": ["main.rs", "lib.rs"],
  "processing_time_ms": 150
}
```

### Tool Execution

#### Call Tool
```http
POST /api/copilot/tools/call
Content-Type: application/json

{
  "session_id": "uuid",
  "tool_name": "search|analyze|context_create|resource_read",
  "arguments": {
    "query": "function definition",
    "scope": "current_file"
  },
  "context_id": "optional_uuid"
}
```

### Health & Status

#### Health Check
```http
GET /api/copilot/health
```

#### Integration Status
```http
GET /api/copilot/status
```

#### Session Management
```http
GET /api/copilot/sessions
POST /api/copilot/sessions/cleanup
```

## Security Features

### Authentication
- GitHub token validation
- Session-based authentication
- Configurable session timeouts
- Maximum sessions per user limits

### Authorization
- Permission-based context access
- Resource-level access control
- Tool execution permissions
- Workspace-scoped access

### Rate Limiting
- Requests per minute limits
- Context requests per hour limits
- Maximum context size restrictions
- Concurrent session limits

### Configuration
```rust
CopilotConfig {
    auth_config: CopilotAuthConfig {
        auth_method: "bearer_token",
        token_validation: true,
        session_timeout: 3600, // 1 hour
        max_sessions_per_user: 5,
    },
    rate_limits: CopilotRateLimits {
        requests_per_minute: 100,
        context_requests_per_hour: 1000,
        max_context_size: 100_000, // 100KB
        max_concurrent_sessions: 50,
    }
}
```

## Usage Examples

### 1. Initialize Copilot Session

```bash
curl -X POST http://localhost:8080/api/copilot/session \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "github_user_123",
    "workspace_id": "conhub_workspace",
    "auth_token": "ghp_xxxxxxxxxxxxxxxxxxxx"
  }'
```

### 2. Request Repository Context

```bash
curl -X POST http://localhost:8080/api/copilot/context \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "session_uuid",
    "context_type": "repository",
    "query": "authentication functions",
    "file_patterns": ["*.rs"],
    "include_dependencies": true
  }'
```

### 3. Execute Search Tool

```bash
curl -X POST http://localhost:8080/api/copilot/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "session_uuid",
    "tool_name": "search",
    "arguments": {
      "query": "async fn authenticate",
      "scope": "repository",
      "include_tests": false
    }
  }'
```

## Integration Benefits

### For GitHub Copilot
- **Rich Context:** Access to structured, metadata-rich context instead of raw file content
- **Real-time Updates:** Live updates when repository or document changes occur
- **Tool Access:** Execute ConHub tools for enhanced code analysis and generation
- **Scoped Access:** Workspace and permission-based context filtering

### For ConHub Users  
- **Enhanced AI:** Copilot suggestions based on comprehensive project context
- **Consistency:** Same context delivery system used across all AI integrations
- **Security:** Fine-grained control over what Copilot can access
- **Performance:** Optimized context delivery with caching and streaming

## Error Handling

Common error scenarios and responses:

### Invalid Session
```json
{
  "success": false,
  "error": "Session not found or expired",
  "code": "SESSION_INVALID"
}
```

### Rate Limit Exceeded
```json
{
  "success": false,
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "retry_after": 60
}
```

### Insufficient Permissions
```json
{
  "success": false,
  "error": "Insufficient permissions for repository access",
  "code": "PERMISSION_DENIED"
}
```

## Monitoring & Observability

The integration provides comprehensive monitoring:

- **Session Metrics:** Active sessions, session duration, cleanup frequency
- **Request Metrics:** Request rates, response times, error rates
- **Context Metrics:** Context size, resource counts, cache hit rates
- **Tool Metrics:** Tool usage, execution times, success rates

Access monitoring data through:
- Health check endpoints
- Status endpoints  
- System logs
- Performance metrics

## Future Enhancements

Planned improvements:
1. **WebSocket Support:** Real-time bidirectional communication
2. **Context Streaming:** Large context delivery in chunks
3. **Advanced Caching:** Multi-level context caching
4. **Webhook Integration:** Event-driven context updates
5. **Enhanced Analytics:** Detailed usage and performance analytics