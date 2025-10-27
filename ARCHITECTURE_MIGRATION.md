# Architecture Migration Guide: From Microservices to Plugin System

## Overview

This document outlines the migration from individual microservices for each source and agent to a unified plugin-based architecture that can scale to hundreds of sources and agents.

## Problem Statement

### Current Issues
- **Resource Overhead**: Each source/agent runs as a separate microservice with its own container, port, and process
- **Management Complexity**: Hundreds of services would be impossible to manage
- **Deployment Complexity**: Each service needs individual deployment, monitoring, and configuration
- **Resource Waste**: Most services are idle most of the time but still consume resources

### Current Structure
```
mcp/servers/
├── sources/
│   ├── dropbox/          # Separate microservice
│   ├── google-drive/     # Separate microservice
│   └── filesystem/       # Separate microservice
└── agents/
    ├── cline/            # Separate microservice
    ├── amazon-q/         # Separate microservice
    └── github-copilot/   # Separate microservice
```

## New Architecture

### Plugin-Based System
- **Single Service**: One unified plugins service hosts all sources and agents
- **Dynamic Loading**: Plugins are loaded/unloaded on demand
- **Shared Resources**: All plugins share the same process, memory, and network resources
- **Centralized Management**: Single API for managing all plugins

### New Structure
```
plugins/                          # Unified plugins service
├── src/
│   ├── main.rs                  # Main service
│   ├── handlers/                # API handlers
│   └── services/                # Core services
├── plugins/                     # Plugin implementations
│   ├── dropbox/                 # Dropbox plugin
│   ├── google-drive/            # Google Drive plugin
│   ├── cline/                   # Cline agent plugin
│   └── amazon-q/                # Amazon Q agent plugin
├── config/
│   └── plugins.json             # Plugin configurations
└── Dockerfile                   # Single container

shared/plugins/                   # Plugin framework
├── src/
│   ├── lib.rs                   # Core plugin traits
│   ├── sources.rs               # Source plugin interfaces
│   ├── agents.rs                # Agent plugin interfaces
│   ├── registry.rs              # Plugin registry
│   └── config.rs                # Configuration management
```

## Key Components

### 1. Plugin Framework (`shared/plugins/`)
- **Core Traits**: Define interfaces for all plugins
- **Registry System**: Manages plugin lifecycle
- **Configuration**: Unified config management
- **Error Handling**: Standardized error types

### 2. Unified Service (`plugins/`)
- **Single Process**: Hosts all plugins
- **REST API**: Manage plugins via HTTP
- **Dynamic Loading**: Load/unload plugins at runtime
- **Health Monitoring**: Monitor all plugins from one place

### 3. Plugin Implementations (`plugins/plugins/`)
- **Modular**: Each plugin is a separate crate
- **Standardized**: All implement the same interfaces
- **Configurable**: Runtime configuration support
- **Isolated**: Plugins can't interfere with each other

## Benefits

### Scalability
- **Resource Efficiency**: Single process for hundreds of plugins
- **Memory Sharing**: Shared libraries and resources
- **Connection Pooling**: Shared HTTP clients and database connections

### Management
- **Single API**: One endpoint to manage all plugins
- **Centralized Logging**: All logs in one place
- **Unified Monitoring**: Single health check endpoint
- **Configuration Management**: One config file for all plugins

### Development
- **Standardized Interfaces**: Consistent plugin development
- **Hot Reloading**: Update plugins without service restart
- **Testing**: Easier to test individual plugins
- **Deployment**: Single container deployment

## Migration Steps

### Phase 1: Framework Setup ✅
- [x] Create plugin framework (`shared/plugins/`)
- [x] Define core traits and interfaces
- [x] Implement registry system
- [x] Create configuration management

### Phase 2: Service Implementation ✅
- [x] Create unified plugins service
- [x] Implement REST API handlers
- [x] Add plugin lifecycle management
- [x] Create Docker configuration

### Phase 3: Plugin Migration
- [ ] Migrate Dropbox source to plugin
- [ ] Migrate Google Drive source to plugin
- [ ] Migrate Cline agent to plugin
- [ ] Migrate Amazon Q agent to plugin
- [ ] Add remaining sources and agents

### Phase 4: Integration
- [ ] Update data service to use plugins API
- [ ] Update AI service to use plugins API
- [ ] Update frontend to manage plugins
- [ ] Remove old MCP servers

### Phase 5: Testing & Deployment
- [ ] Integration testing
- [ ] Performance testing
- [ ] Production deployment
- [ ] Monitor and optimize

## API Examples

### Plugin Management
```bash
# List available plugin types
GET /api/plugins/registry/sources
GET /api/plugins/registry/agents

# Start a plugin
POST /api/plugins/start/dropbox-main

# Stop a plugin
POST /api/plugins/stop/dropbox-main

# Get plugin status
GET /api/plugins/status/dropbox-main
```

### Source Operations
```bash
# List documents from a source
GET /api/plugins/sources/dropbox-main/documents

# Search documents
POST /api/plugins/sources/dropbox-main/search
{
  "query": "project proposal"
}

# Sync source
POST /api/plugins/sources/dropbox-main/sync
```

### Agent Operations
```bash
# Chat with an agent
POST /api/plugins/agents/cline-main/chat
{
  "message": "Help me debug this code",
  "context": {...}
}

# Get agent functions
GET /api/plugins/agents/cline-main/functions
```

## Configuration

### Plugin Configuration (`plugins/config/plugins.json`)
```json
{
  "plugins": {
    "dropbox-main": {
      "instance_id": "dropbox-main",
      "plugin_type": "Source",
      "plugin_name": "dropbox",
      "enabled": true,
      "auto_start": true,
      "config": {
        "enabled": true,
        "settings": {
          "access_token": "your-token",
          "sync_interval_minutes": 30
        }
      }
    }
  }
}
```

### Environment Variables
```bash
PLUGINS_SERVICE_PORT=3020
PLUGIN_CONFIG_PATH=./config/plugins.json
DATABASE_URL=postgresql://...
```

## Deployment

### Docker Compose Update
```yaml
services:
  plugins:
    build: ./plugins
    ports:
      - "3020:3020"
    environment:
      - PLUGINS_SERVICE_PORT=3020
      - PLUGIN_CONFIG_PATH=/app/config/plugins.json
    volumes:
      - ./plugins/config:/app/config
    depends_on:
      - postgres
      - qdrant

  # Remove individual MCP services
  # dropbox-mcp: (removed)
  # google-drive-mcp: (removed)
  # cline-mcp: (removed)
```

## Monitoring

### Health Checks
- **Service Health**: `/health` endpoint
- **Plugin Health**: Individual plugin status
- **Resource Usage**: Memory, CPU per plugin
- **Error Tracking**: Centralized error logging

### Metrics
- **Plugin Count**: Active/inactive plugins
- **Request Rate**: API requests per plugin
- **Response Time**: Plugin operation latency
- **Error Rate**: Plugin failure rates

## Security

### Plugin Isolation
- **Memory Isolation**: Plugins can't access each other's data
- **Configuration Isolation**: Separate config per plugin
- **Error Isolation**: Plugin failures don't affect others
- **Resource Limits**: CPU/memory limits per plugin

### API Security
- **Authentication**: JWT tokens for API access
- **Authorization**: Role-based plugin access
- **Rate Limiting**: Prevent API abuse
- **Input Validation**: Sanitize all inputs

## Performance

### Optimizations
- **Lazy Loading**: Load plugins only when needed
- **Connection Pooling**: Shared HTTP/DB connections
- **Caching**: Cache plugin responses
- **Async Operations**: Non-blocking plugin operations

### Scaling
- **Horizontal**: Multiple plugin service instances
- **Load Balancing**: Distribute plugin load
- **Resource Management**: Dynamic resource allocation
- **Auto-scaling**: Scale based on plugin usage

## Troubleshooting

### Common Issues
1. **Plugin Won't Start**: Check configuration and logs
2. **High Memory Usage**: Monitor plugin resource usage
3. **API Timeouts**: Check plugin response times
4. **Configuration Errors**: Validate plugin config schema

### Debugging
- **Logs**: Centralized logging with plugin context
- **Metrics**: Real-time plugin performance metrics
- **Health Checks**: Automated plugin health monitoring
- **Tracing**: Request tracing across plugins

## Future Enhancements

### Plugin Marketplace
- **Plugin Discovery**: Browse available plugins
- **Plugin Installation**: Install plugins from registry
- **Version Management**: Update plugins independently
- **Community Plugins**: Third-party plugin support

### Advanced Features
- **Plugin Dependencies**: Manage plugin dependencies
- **Plugin Composition**: Combine multiple plugins
- **Plugin Workflows**: Chain plugin operations
- **Plugin Analytics**: Usage analytics per plugin

## Conclusion

The new plugin-based architecture provides:
- **Scalability**: Support for hundreds of sources and agents
- **Efficiency**: Reduced resource usage and management overhead
- **Flexibility**: Easy addition of new plugins
- **Maintainability**: Centralized management and monitoring

This migration enables ConHub to scale efficiently while maintaining performance and reliability.