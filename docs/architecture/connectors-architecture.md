# Connectors Architecture

## Overview

The Connectors Architecture is a unified system for managing external data sources in ConHub. It replaces the previous fragmented approach with a scalable, maintainable solution.

## Architecture Components

### 1. Core Connector Trait (`data/src/connectors/traits.rs`)

All connectors implement the `Connector` trait which defines:

- **Authentication**: OAuth flows and credential management
- **Connection Management**: Establishing and maintaining connections
- **Data Retrieval**: Listing and fetching documents
- **Synchronization**: Full and incremental sync operations
- **Lifecycle Management**: Connect, disconnect, and credential refresh

```rust
#[async_trait]
pub trait Connector: Send + Sync {
    fn name(&self) -> &str;
    fn connector_type(&self) -> ConnectorType;
    async fn authenticate(&self, config: &ConnectorConfig) -> Result<Option<String>, ConnectorError>;
    async fn connect(&mut self, account: &ConnectedAccount) -> Result<(), ConnectorError>;
    async fn sync(&self, account: &ConnectedAccount, request: &SyncRequest) 
        -> Result<(SyncResult, Vec<DocumentForEmbedding>), ConnectorError>;
    // ... more methods
}
```

### 2. Connector Types

Currently implemented connectors:

#### LocalFileConnector
- **Purpose**: Handle file uploads from user's local system
- **Features**: 
  - Multi-file upload support
  - Automatic content type detection
  - Text extraction from various file types
  - Automatic chunking for embeddings

#### GitHubConnector
- **Purpose**: Connect to GitHub repositories
- **Features**:
  - OAuth2 authentication
  - Repository browsing
  - Recursive file listing
  - Webhook support (planned)
  - Incremental syncing based on commits

#### GoogleDriveConnector
- **Purpose**: Connect to Google Drive
- **Features**:
  - OAuth2 authentication
  - Folder and file traversal
  - Google Workspace document export
  - Incremental sync based on modification time
  - Webhook support via Push Notifications (planned)

### 3. Connector Manager (`data/src/connectors/manager.rs`)

Central orchestrator that:
- Manages connector instances
- Handles connection lifecycle
- Maintains active connections
- Coordinates with database for persistence

### 4. Database Schema

**Connected Accounts Table**:
```sql
CREATE TABLE connected_accounts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    connector_type VARCHAR(50) NOT NULL,
    account_name VARCHAR(255) NOT NULL,
    credentials JSONB NOT NULL,
    status JSONB NOT NULL,
    last_sync_at TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
```

**Source Documents Table**:
```sql
CREATE TABLE source_documents (
    id UUID PRIMARY KEY,
    source_id UUID NOT NULL,
    connector_type VARCHAR(50) NOT NULL,
    external_id VARCHAR(500) NOT NULL,
    name VARCHAR(500) NOT NULL,
    content_type VARCHAR(50),
    metadata JSONB,
    indexed_at TIMESTAMPTZ
);
```

## Data Flow

1. **User Connects Source** → Data Service receives connection request
2. **Authentication** → Connector handles OAuth/credential validation
3. **Connection Established** → Account saved to database
4. **Sync Triggered** → Connector fetches documents
5. **Documents Processed** → Content extracted and chunked
6. **Embedding Sent** → Documents sent to embedding service
7. **Indexed** → Embeddings stored in vector database

## Embedding Integration

### Embedding Client (`data/src/services/embedding_client.rs`)

Handles communication with the embedding service:

```rust
pub struct EmbeddingClient {
    client: Client,
    base_url: String,
    enabled: bool,
}

impl EmbeddingClient {
    pub async fn embed_documents(
        &self,
        documents: Vec<DocumentForEmbedding>,
    ) -> Result<BatchEmbedResponse, Box<dyn std::error::Error + Send + Sync>>
}
```

### Document Processing Pipeline

1. **Chunking**: Documents split into ~1000 character chunks with 200 char overlap
2. **Metadata Preservation**: Each chunk maintains link to source
3. **Batch Processing**: Chunks sent in batches of 16 to embedding service
4. **Vector Storage**: Embeddings stored with full metadata in Qdrant

## API Endpoints

### Connector Management

```
GET    /api/connectors/list              - List available connector types
POST   /api/connectors/connect           - Connect new data source
POST   /api/connectors/oauth/callback    - Complete OAuth flow
POST   /api/connectors/sync              - Trigger sync
DELETE /api/connectors/disconnect/{id}   - Disconnect source
GET    /api/connectors/accounts          - List connected accounts
```

### Request/Response Examples

**Connect GitHub**:
```json
POST /api/connectors/connect
{
  "connector_type": "github",
  "account_name": "My GitHub",
  "credentials": {
    "client_id": "xxx",
    "client_secret": "yyy"
  }
}
```

**Sync Source**:
```json
POST /api/connectors/sync
{
  "account_id": "uuid",
  "incremental": false
}
```

## Frontend Integration

### ConnectSourceModal Component

New modal component provides unified interface for:
- Local file upload
- GitHub connection with OAuth
- Google Drive connection with OAuth

Features:
- Tab-based interface for different connector types
- Real-time connection status
- OAuth redirect handling
- Error handling and user feedback

## Feature Toggles

The connector system respects feature toggles:

- **Auth**: Controls database and authentication (currently `false` for development)
- **Heavy**: Controls embedding and indexing services (currently `false` for lightweight dev)

When `Auth=false`:
- Database connections skipped
- Default claims injected for development

When `Heavy=false`:
- Embedding service returns mock responses
- Indexing operations are logged but not executed

## Error Handling

Comprehensive error types in `data/src/connectors/error.rs`:

```rust
pub enum ConnectorError {
    AuthenticationFailed(String),
    ConnectionFailed(String),
    SyncFailed(String),
    InvalidConfiguration(String),
    DocumentNotFound(String),
    RateLimitExceeded,
    // ... more variants
}
```

## Future Enhancements

### Planned Connectors
- Dropbox
- OneDrive
- Notion
- Slack
- GitLab
- Bitbucket

### Planned Features
- **Webhook Support**: Real-time updates from supported sources
- **Selective Sync**: User-defined filters for what to sync
- **Sync Scheduling**: Automated periodic syncing
- **Conflict Resolution**: Handle document updates and deletions
- **Rate Limiting**: Respect API rate limits
- **Retry Logic**: Automatic retry with exponential backoff
- **Progress Tracking**: Real-time sync progress updates

## Testing

To test connectors:

1. **Local Development**:
   ```bash
   # Set feature toggles
   echo '{"Auth": false, "Heavy": false, "Docker": false}' > feature-toggles.json
   
   # Start data service
   cd data && cargo run
   ```

2. **Test Local File Upload**:
   - Use frontend modal to upload files
   - Check logs for processing confirmation

3. **Test OAuth Connectors**:
   - Obtain OAuth credentials from GitHub/Google
   - Use frontend modal to initiate connection
   - Complete OAuth flow in opened window

## Security Considerations

1. **Credential Storage**: OAuth tokens encrypted in database
2. **Token Refresh**: Automatic refresh for expired tokens
3. **Scope Limitation**: Minimal required scopes requested
4. **User Isolation**: Each user's connections completely isolated
5. **HTTPS Required**: All OAuth flows require HTTPS in production

## Performance Optimization

1. **Connection Pooling**: Reuse HTTP connections
2. **Batch Processing**: Process documents in batches
3. **Async Operations**: All I/O operations are async
4. **Concurrent Processing**: Multiple documents processed concurrently
5. **Caching**: Active connectors cached in memory

## Monitoring and Logging

All operations logged with structured logging:

```rust
info!("🔌 Connecting new data source: {:?}", connector_type);
info!("✅ Sync completed: {} documents", document_count);
error!("Failed to connect: {}", error);
```

Log levels:
- `info`: Normal operations
- `warn`: Recoverable issues
- `error`: Failures requiring attention
- `debug`: Detailed debugging information
