# Plugins Integration Guide

This guide explains how to integrate the data and AI services with the new unified plugins system.

## Overview

The new plugin system replaces individual MCP microservices with a unified plugins service that manages all source and agent plugins through a single API.

## API Endpoints

### Base URL
```
http://localhost:3020/api
```

### Plugin Management

#### Get Plugin Status
```http
GET /status
```

Response:
```json
{
  "total_configured": 5,
  "total_active": 4,
  "active_sources": 2,
  "active_agents": 2,
  "enabled_plugins": 5,
  "auto_start_plugins": 4,
  "operation_stats": {...},
  "last_updated": "2024-01-15T10:30:00Z"
}
```

#### List All Plugins
```http
GET /plugins
```

#### Start/Stop Plugins
```http
POST /lifecycle/{instance_id}/start
POST /lifecycle/{instance_id}/stop
POST /lifecycle/{instance_id}/restart
GET /lifecycle/{instance_id}/status
```

### Source Plugin Operations

#### List Documents
```http
GET /sources/{instance_id}/documents?limit=50&offset=0
```

#### Get Document
```http
GET /sources/{instance_id}/documents/{document_id}
```

#### Search Documents
```http
GET /sources/{instance_id}/search?q={query}&limit=20
```

#### Sync Documents
```http
POST /sources/{instance_id}/sync
```

#### Upload Document
```http
POST /sources/{instance_id}/upload
Content-Type: multipart/form-data

file: <file_data>
metadata: {"title": "Document Title", "tags": ["tag1", "tag2"]}
```

#### Delete Document
```http
DELETE /sources/{instance_id}/documents/{document_id}
```

### Agent Plugin Operations

#### Send Message
```http
POST /agents/{instance_id}/chat
Content-Type: application/json

{
  "message": {
    "content": "Hello, can you help me with this code?",
    "role": "user",
    "metadata": {}
  },
  "context": {
    "conversation_id": "conv_123",
    "user_id": "user_456",
    "session_data": {}
  }
}
```

#### Stream Chat
```http
POST /agents/{instance_id}/stream
Content-Type: application/json

{
  "message": {...},
  "context": {...}
}
```

#### Get Available Functions
```http
GET /agents/{instance_id}/functions
```

#### Execute Action
```http
POST /agents/{instance_id}/actions/{action_name}
Content-Type: application/json

{
  "parameters": {
    "param1": "value1",
    "param2": "value2"
  },
  "context": {...}
}
```

## Integration Examples

### Data Service Integration

#### Rust Example (using reqwest)

```rust
use reqwest::Client;
use serde_json::{json, Value};
use anyhow::Result;

pub struct PluginsClient {
    client: Client,
    base_url: String,
}

impl PluginsClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn search_all_sources(&self, query: &str, limit: Option<usize>) -> Result<Vec<Value>> {
        let url = format!("{}/sources/search", self.base_url);
        let response = self.client
            .get(&url)
            .query(&[("q", query)])
            .query(&[("limit", limit.unwrap_or(50).to_string())])
            .send()
            .await?;

        let documents: Vec<Value> = response.json().await?;
        Ok(documents)
    }

    pub async fn sync_source(&self, instance_id: &str) -> Result<Value> {
        let url = format!("{}/sources/{}/sync", self.base_url, instance_id);
        let response = self.client.post(&url).send().await?;
        let result: Value = response.json().await?;
        Ok(result)
    }

    pub async fn get_plugin_status(&self) -> Result<Value> {
        let url = format!("{}/status", self.base_url);
        let response = self.client.get(&url).send().await?;
        let status: Value = response.json().await?;
        Ok(status)
    }
}

// Usage in data service
#[tokio::main]
async fn main() -> Result<()> {
    let plugins_client = PluginsClient::new("http://localhost:3020/api".to_string());
    
    // Search across all source plugins
    let documents = plugins_client.search_all_sources("important document", Some(20)).await?;
    println!("Found {} documents", documents.len());
    
    // Sync a specific source
    let sync_result = plugins_client.sync_source("google-drive-default").await?;
    println!("Sync result: {:?}", sync_result);
    
    Ok(())
}
```

#### Node.js Example

```javascript
const axios = require('axios');

class PluginsClient {
    constructor(baseUrl = 'http://localhost:3020/api') {
        this.baseUrl = baseUrl;
        this.client = axios.create({
            baseURL: baseUrl,
            timeout: 30000,
        });
    }

    async searchAllSources(query, limit = 50) {
        try {
            const response = await this.client.get('/sources/search', {
                params: { q: query, limit }
            });
            return response.data;
        } catch (error) {
            console.error('Search failed:', error.message);
            throw error;
        }
    }

    async syncSource(instanceId) {
        try {
            const response = await this.client.post(`/sources/${instanceId}/sync`);
            return response.data;
        } catch (error) {
            console.error(`Sync failed for ${instanceId}:`, error.message);
            throw error;
        }
    }

    async getPluginStatus() {
        try {
            const response = await this.client.get('/status');
            return response.data;
        } catch (error) {
            console.error('Status check failed:', error.message);
            throw error;
        }
    }
}

// Usage
async function main() {
    const pluginsClient = new PluginsClient();
    
    try {
        // Search across all sources
        const documents = await pluginsClient.searchAllSources('project requirements');
        console.log(`Found ${documents.length} documents`);
        
        // Get system status
        const status = await pluginsClient.getPluginStatus();
        console.log('System status:', status);
        
    } catch (error) {
        console.error('Error:', error.message);
    }
}

main();
```

### AI Service Integration

#### Chat with Agent Plugin

```rust
use serde_json::json;

pub async fn chat_with_agent(
    plugins_client: &PluginsClient,
    agent_id: &str,
    message: &str,
    conversation_id: &str,
    user_id: &str,
) -> Result<Value> {
    let url = format!("{}/agents/{}/chat", plugins_client.base_url, agent_id);
    
    let payload = json!({
        "message": {
            "content": message,
            "role": "user",
            "metadata": {}
        },
        "context": {
            "conversation_id": conversation_id,
            "user_id": user_id,
            "session_data": {}
        }
    });
    
    let response = plugins_client.client
        .post(&url)
        .json(&payload)
        .send()
        .await?;
    
    let result: Value = response.json().await?;
    Ok(result)
}

// Stream chat example
pub async fn stream_chat_with_agent(
    plugins_client: &PluginsClient,
    agent_id: &str,
    message: &str,
) -> Result<()> {
    let url = format!("{}/agents/{}/stream", plugins_client.base_url, agent_id);
    
    let payload = json!({
        "message": {
            "content": message,
            "role": "user",
            "metadata": {}
        },
        "context": {
            "conversation_id": "stream_conv",
            "user_id": "user_123",
            "session_data": {}
        }
    });
    
    let mut response = plugins_client.client
        .post(&url)
        .json(&payload)
        .send()
        .await?;
    
    // Handle streaming response
    while let Some(chunk) = response.chunk().await? {
        let chunk_str = String::from_utf8_lossy(&chunk);
        print!("{}", chunk_str);
    }
    
    Ok(())
}
```

## Environment Variables

Update your service environment variables to point to the plugins service:

```env
# Replace individual MCP service URLs with plugins service
PLUGINS_SERVICE_URL=http://localhost:3020
PLUGINS_API_BASE=http://localhost:3020/api

# Remove old MCP service variables
# MCP_GOOGLE_DRIVE_URL=http://localhost:3001
# MCP_DROPBOX_URL=http://localhost:3002
# MCP_CLINE_URL=http://localhost:3003
# etc.
```

## Error Handling

The plugins API returns standard HTTP status codes:

- `200` - Success
- `201` - Created
- `400` - Bad Request
- `404` - Plugin/Resource Not Found
- `500` - Internal Server Error
- `503` - Service Unavailable

Error responses include details:

```json
{
  "error": "Plugin not found",
  "details": "Plugin instance 'invalid-plugin' does not exist",
  "code": "PLUGIN_NOT_FOUND"
}
```

## Health Checks

Monitor plugin health:

```rust
pub async fn monitor_plugin_health(plugins_client: &PluginsClient) -> Result<()> {
    loop {
        match plugins_client.get_plugin_status().await {
            Ok(status) => {
                let active = status["total_active"].as_u64().unwrap_or(0);
                let total = status["total_configured"].as_u64().unwrap_or(0);
                
                if active < total {
                    println!("Warning: {}/{} plugins are active", active, total);
                }
            }
            Err(e) => {
                println!("Health check failed: {}", e);
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
```

## Migration Checklist

### For Data Service:
- [ ] Replace MCP service calls with plugins API calls
- [ ] Update document indexing to use `/sources/{id}/documents`
- [ ] Update search functionality to use `/sources/search`
- [ ] Implement bulk sync using `/sources/{id}/sync`
- [ ] Add health monitoring for source plugins

### For AI Service:
- [ ] Replace agent service calls with plugins API calls
- [ ] Update chat endpoints to use `/agents/{id}/chat`
- [ ] Implement streaming chat using `/agents/{id}/stream`
- [ ] Update function calling to use `/agents/{id}/actions/{action}`
- [ ] Add agent plugin health monitoring

### For Frontend:
- [ ] Update API endpoints to point to plugins service
- [ ] Update plugin management UI
- [ ] Add plugin status monitoring
- [ ] Update error handling for new API responses

## Performance Considerations

1. **Connection Pooling**: Use connection pooling for HTTP clients
2. **Caching**: Cache plugin status and capabilities
3. **Timeouts**: Set appropriate timeouts for plugin operations
4. **Retry Logic**: Implement retry logic for transient failures
5. **Load Balancing**: Consider load balancing for high-traffic scenarios

## Security

1. **Authentication**: Implement authentication for plugin API access
2. **Authorization**: Control access to specific plugins per user
3. **Rate Limiting**: Implement rate limiting to prevent abuse
4. **Input Validation**: Validate all inputs to plugin operations
5. **Audit Logging**: Log all plugin operations for security auditing

## Troubleshooting

### Common Issues:

1. **Plugin Not Starting**: Check plugin configuration and logs
2. **API Timeouts**: Increase timeout values for slow operations
3. **Memory Issues**: Monitor plugin memory usage
4. **Network Connectivity**: Verify plugins service is accessible

### Debug Commands:

```bash
# Check plugin status
curl http://localhost:3020/api/status

# Check specific plugin
curl http://localhost:3020/api/lifecycle/google-drive-default/status

# View plugin logs
docker logs conhub-plugins-1

# Restart plugin
curl -X POST http://localhost:3020/api/lifecycle/google-drive-default/restart
```