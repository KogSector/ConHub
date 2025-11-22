# MCP (Model Context Protocol) Service

## Overview

The MCP Service is a complete rewrite in Rust using Actix-web that provides a standardized protocol for AI agents to access context, tools, and communicate with each other.

## Purpose

The MCP Service acts as a central hub that:
1. **Exposes Resources**: Provides access to knowledge base context
2. **Manages Tools**: Offers executable capabilities to AI agents
3. **Facilitates Communication**: Enables agent-to-agent communication
4. **Synchronizes Context**: Keeps agents in sync with shared context

## Architecture

### Core Components

#### 1. MCP Server (`mcp/src/server/mod.rs`)

Central server managing:
- **Resources**: Documents, data, and knowledge artifacts
- **Tools**: Executable capabilities
- **Prompts**: Reusable prompt templates
- **Agents**: Registered AI agents
- **Shared Context**: Context shared between agents

```rust
pub struct MCPServer {
    db_pool: Option<PgPool>,
    agents: Arc<DashMap<Uuid, Agent>>,
    resources: Arc<RwLock<Vec<Resource>>>,
    tools: Arc<RwLock<Vec<Tool>>>,
    prompts: Arc<RwLock<Vec<Prompt>>>,
    shared_contexts: Arc<DashMap<Uuid, SharedContext>>,
}
```

#### 2. Protocol Types (`mcp/src/protocol/mod.rs`)

Defines standard MCP protocol types:

**Resource**:
```rust
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
    pub metadata: Option<serde_json::Value>,
}
```

**Tool**:
```rust
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
    pub returns: Option<serde_json::Value>,
}
```

**Agent**:
```rust
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
```

## API Endpoints

### Resource Management

```
GET  /api/mcp/resources           - List all available resources
GET  /api/mcp/resources/{uri}     - Read specific resource content
```

### Tool Management

```
GET  /api/mcp/tools               - List all available tools
POST /api/mcp/tools/execute       - Execute a tool
```

### Prompt Management

```
GET  /api/mcp/prompts             - List all prompt templates
GET  /api/mcp/prompts/{name}      - Get specific prompt
```

### Agent Management

```
GET    /api/mcp/agents            - List registered agents
POST   /api/mcp/agents            - Register new agent
DELETE /api/mcp/agents/{id}       - Unregister agent
GET    /api/mcp/agents/{id}/context - Get agent's context
```

### Context Synchronization

```
POST /api/mcp/sync                - Sync context between agents
POST /api/mcp/broadcast           - Broadcast message to all agents
```

## Default Tools

### 1. Query Context
Queries the knowledge base for relevant information.

**Parameters**:
```json
{
  "query": "search terms",
  "filters": {
    "connector_type": "github",
    "date_range": "..."
  }
}
```

### 2. Sync Context
Synchronizes context data between agents.

**Parameters**:
```json
{
  "target_agents": ["agent-id-1", "agent-id-2"],
  "context": {
    "key": "value"
  }
}
```

## Default Prompts

### Code Review Prompt
```
Review the following code and provide feedback:

```{{language}}
{{code}}
```

Focus on: {{focus_areas}}
```

## Agent Registration Flow

1. **Agent Registers**:
```json
POST /api/mcp/agents
{
  "name": "GitHub Copilot",
  "agent_type": "code_assistant",
  "capabilities": ["code_completion", "code_review"],
  "endpoint": "https://copilot.github.com/webhook"
}
```

2. **Server Assigns ID**: Returns unique agent ID

3. **Agent Can Now**:
   - Query resources
   - Execute tools
   - Sync context with other agents
   - Receive broadcasts

## Context Synchronization

### Sharing Context

```json
POST /api/mcp/sync
{
  "source_agent_id": "uuid",
  "target_agent_ids": ["uuid1", "uuid2"],
  "context": {
    "type": "code_change",
    "data": {
      "file": "main.rs",
      "changes": "..."
    }
  },
  "context_type": "code_update"
}
```

### Broadcasting

```json
POST /api/mcp/broadcast
{
  "source_agent_id": "uuid",
  "message": {
    "event": "deployment_complete",
    "details": {...}
  },
  "message_type": "notification"
}
```

## Use Cases

### 1. Code Assistant Integration

**Scenario**: IDE extension connects as agent

```
1. Register as agent: "VS Code Extension"
2. Query context: Get relevant code from GitHub repos
3. Execute tool: "query_context" with current file context
4. Receive context: Get related files and documentation
5. Provide suggestions: Based on comprehensive context
```

### 2. Multi-Agent Collaboration

**Scenario**: Code review bot and deployment bot coordinate

```
1. Code Review Bot: Reviews PR, shares findings via sync
2. Deployment Bot: Receives context via sync
3. Deployment Bot: Makes deployment decision based on review
4. Deployment Bot: Broadcasts deployment status
5. All Agents: Receive deployment notification
```

### 3. Real-time Knowledge Sync

**Scenario**: Documentation update propagates to all agents

```
1. Document updated in Google Drive
2. Connector syncs change
3. MCP broadcasts update to all registered agents
4. Agents refresh their local context caches
```

## Services

### Context Service (`mcp/src/services/context.rs`)

Validates and manages shared context:

```rust
pub struct ContextService;

impl ContextService {
    pub fn validate_context(&self, context: &SharedContext) -> Result<(), MCPError>
    pub fn is_expired(&self, context: &SharedContext) -> bool
}
```

### Tool Service (`mcp/src/services/tools.rs`)

Executes tools with proper validation:

```rust
pub struct ToolService;

impl ToolService {
    pub async fn execute_tool(
        &self,
        tool: &Tool,
        request: &ToolExecutionRequest,
    ) -> Result<ToolExecutionResult, MCPError>
}
```

## Error Handling

```rust
pub enum MCPError {
    ResourceNotFound(String),
    ToolNotFound(String),
    ToolExecutionFailed(String),
    AgentNotFound(String),
    ContextSyncFailed(String),
    InvalidRequest(String),
}
```

## Security

1. **Authentication**: All requests authenticated via middleware
2. **Agent Isolation**: Each agent's context isolated
3. **Access Control**: Agents can only access authorized resources
4. **Rate Limiting**: Prevent abuse of tool execution
5. **Audit Logging**: All operations logged for security review

## Performance

1. **In-Memory Caching**: Active agents and resources cached
2. **Concurrent Access**: DashMap for thread-safe concurrent access
3. **Async Operations**: All I/O operations async
4. **Connection Pooling**: Database connections pooled

## Monitoring

Comprehensive logging:
```
🔧 Executing tool: query_context
🤖 Registering agent: VS Code Extension (uuid)
🔄 Syncing context from agent: uuid
📢 Broadcasting message from agent: uuid
✅ Agent registered: uuid
```

## Future Enhancements

1. **WebSocket Support**: Real-time bidirectional communication
2. **Event Streaming**: Server-sent events for live updates
3. **Tool Marketplace**: Community-contributed tools
4. **Advanced ACL**: Fine-grained access control
5. **Tool Composition**: Chain tools together
6. **Agent Discovery**: Dynamic agent capability discovery
7. **Context TTL**: Automatic cleanup of expired context
8. **Metrics Dashboard**: Real-time monitoring UI

## Integration with Other Services

### With Data Service
- MCP exposes resources from connected data sources
- Agents query indexed documents via MCP

### With Embedding Service
- Tools can trigger embedding jobs
- Context updates trigger re-embedding

### With Frontend
- WebSocket connection for real-time updates
- Agent management UI
- Tool execution interface

## Development

Start the MCP service:

```bash
cd mcp/service
npm start
```

Service starts on port `3004` by default.

Test health:
```bash
curl http://localhost:3004/health
```

## Configuration

Environment variables:

```
MCP_SERVICE_PORT=3004
DATABASE_URL=postgresql://...
```

Feature toggles respected:
- `Auth`: Enable/disable authentication
- `Heavy`: Enable/disable heavy operations
