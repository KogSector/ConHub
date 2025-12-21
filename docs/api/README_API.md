# ConHub API Documentation

This directory contains comprehensive API documentation for the ConHub AI-powered development context platform.

## Documentation Files

### 📖 [CONHUB_API_DOCUMENTATION.md](./CONHUB_API_DOCUMENTATION.md)
Complete API documentation covering all microservices:
- Authentication & Authorization
- Data Source Management
- Billing & Subscriptions
- Security Policies & Audit Logs
- Webhook Handling
- Vector RAG & Embeddings
- GraphQL API
- Error Handling & Rate Limiting

### 🔧 [OPENAPI_SPECIFICATION.yaml](./OPENAPI_SPECIFICATION.yaml)
OpenAPI 3.0 specification for all API endpoints:
- Standardized schema definitions
- Request/response examples
- Authentication requirements
- Error response formats
- Ready for API documentation tools

### 🚀 [ConHub_API.postman_collection.json](./ConHub_API.postman_collection.json)
Complete Postman collection for easy API testing:
- All endpoints organized by service
- Pre-configured environment variables
- Sample request bodies
- Authentication setup
- Health checks for all services

## Quick Start

### 1. Import Postman Collection
1. Open Postman
2. Click "Import" → "File"
3. Select `ConHub_API.postman_collection.json`
4. Configure environment variables:
   - `baseUrl`: `http://localhost:8000` (main API gateway)
   - `jwtToken`: Set after successful login
   - Other service URLs are pre-configured

### 2. Test with cURL
```bash
# Health check
curl http://localhost:8000/health

# User registration
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"SecurePass123!","name":"Test User"}'

# GraphQL query
curl -X POST http://localhost:8000/api/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{"query":"{ health, version }"}'
```

### 3. View API Documentation
Open `CONHUB_API_DOCUMENTATION.md` in your browser or markdown viewer for complete documentation.

## Service Architecture

```
Frontend (3000) → Nginx (80) → Backend (8000) → Microservices
                                    ↓
                            GraphQL Federation
                                    ↓
                    ┌───────────────┼───────────────┐
                    ↓               ↓               ↓
              Auth (3010)    Data (3013)      AI (3012)
                    ↓               ↓               ↓
              Billing (3011) Security (3014) Webhook (3015)
                    ┌───────────────┼───────────────┐
                    ↓               ↓               ↓
            Embedding (8082)  Indexers (8080)  Databases
```

## Key Features

### 🔐 Authentication
- JWT-based authentication with RS256 signatures
- OAuth integration (GitHub, Google, Microsoft)
- Role-based access control
- Social connection management

### 📊 Data Management
- Multi-source integration (GitHub, GitLab, cloud storage)
- Real-time synchronization
- File type filtering and processing
- Branch and repository management

### 💳 Billing & Subscriptions
- Stripe payment integration
- Subscription tier management
- Usage tracking and limits
- Invoice generation and management

### 🛡️ Security
- Configurable security rulesets
- Comprehensive audit logging
- Rate limiting and throttling
- Social provider security

### 🔍 Vector RAG
- Fusion embedding generation
- Document reranking
- Semantic vector search
- Multi-model support

### 📡 Webhooks
- GitHub, GitLab integration
- Stripe payment webhooks
- Event-driven processing
- Signature verification

## Environment Configuration

### Development
```bash
# API Gateway
BASE_URL=http://localhost:8000

# Individual Services
AUTH_URL=http://localhost:3010
DATA_URL=http://localhost:3013
BILLING_URL=http://localhost:3011
SECURITY_URL=http://localhost:3014
VECTOR_RAG_URL=http://localhost:8082
```

### Production
```bash
# API Gateway
BASE_URL=https://api.conhub.com

# Individual Services
AUTH_URL=https://auth.conhub.com
DATA_URL=https://data.conhub.com
BILLING_URL=https://billing.conhub.com
SECURITY_URL=https://security.conhub.com
VECTOR_RAG_URL=https://vector.conhub.com
```

## Authentication

All API endpoints (except health checks and public OAuth endpoints) require JWT authentication:

```http
Authorization: Bearer <jwt_token>
```

### Getting a Token
```bash
# Login
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"SecurePass123!"}'

# Extract token from response and use in subsequent requests
export JWT_TOKEN="eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## Rate Limiting

API endpoints implement rate limiting based on subscription tier:

| Tier | Requests/Minute | Burst Size |
|-------|-----------------|-------------|
| Free  | 100             | 200         |
| Pro    | 1,000           | 2,000       |
| Enterprise | 5,000     | 10,000      |

Rate limit headers are included in responses:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1701421800
```

## Error Handling

All APIs return errors in a consistent format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input parameters",
    "details": {
      "field": "email",
      "reason": "Invalid email format"
    },
    "timestamp": "2023-12-01T10:30:00Z",
    "request_id": "req_1234567890"
  }
}
```

### Common Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| VALIDATION_ERROR | Invalid input parameters | 400 |
| AUTHENTICATION_REQUIRED | Missing or invalid JWT | 401 |
| AUTHORIZATION_FAILED | Insufficient permissions | 403 |
| NOT_FOUND | Resource not found | 404 |
| RATE_LIMITED | Too many requests | 429 |
| INTERNAL_ERROR | Server error | 500 |
| SERVICE_UNAVAILABLE | Service temporarily unavailable | 503 |

## WebSocket Support

Real-time updates are available via WebSocket connections:

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8000/ws');

// Handle messages
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  switch(data.type) {
    case 'sync_status':
      updateSyncStatus(data.payload);
      break;
    case 'notification':
      handleNotification(data.payload);
      break;
  }
};
```

## Monitoring & Health

### Health Check Endpoints
All services expose health endpoints:

```bash
# Main API Gateway
curl http://localhost:8000/health

# Individual Services
curl http://localhost:3010/health  # Auth
curl http://localhost:3011/health  # Billing
curl http://localhost:3013/health  # Data
curl http://localhost:3014/health  # Security
curl http://localhost:8082/health  # Vector RAG
```

### Metrics
Services expose Prometheus-compatible metrics:

```bash
# Service metrics
curl http://localhost:8000/metrics
curl http://localhost:3010/metrics  # Auth service
curl http://localhost:3013/metrics  # Data service
```

## SDK Examples

### JavaScript/TypeScript
```javascript
// API Configuration
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000';

// Authentication
export const auth = {
  login: async (credentials) => {
    const response = await fetch(`${API_BASE_URL}/api/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(credentials)
    });
    return response.json();
  },
  
  getCurrentUser: async (token) => {
    const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    return response.json();
  }
};

// Data Sources
export const dataSources = {
  connect: async (source, token) => {
    const response = await fetch(`${API_BASE_URL}/api/data/sources`, {
      method: 'POST',
      headers: { 
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
      },
      body: JSON.stringify(source)
    });
    return response.json();
  },
  
  list: async (token) => {
    const response = await fetch(`${API_BASE_URL}/api/data-sources`, {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    return response.json();
  }
};
```

### Python
```python
import requests
import jwt

class ConHubAPI:
    def __init__(self, base_url="http://localhost:8000"):
        self.base_url = base_url
        self.token = None
    
    def login(self, email, password):
        response = requests.post(
            f"{self.base_url}/api/auth/login",
            json={"email": email, "password": password}
        )
        if response.status_code == 200:
            data = response.json()
            self.token = data.get("token")
        return response.json()
    
    def get_current_user(self):
        headers = {"Authorization": f"Bearer {self.token}"}
        response = requests.get(
            f"{self.base_url}/api/auth/me",
            headers=headers
        )
        return response.json()
    
    def connect_data_source(self, source_config):
        headers = {
            "Authorization": f"Bearer {self.token}",
            "Content-Type": "application/json"
        }
        response = requests.post(
            f"{self.base_url}/api/data/sources",
            json=source_config,
            headers=headers
        )
        return response.json()
```

## Contributing

### Updating Documentation

1. Update the main documentation file: `CONHUB_API_DOCUMENTATION.md`
2. Regenerate OpenAPI spec:
   ```bash
   npm run generate-api-docs
   ```
3. Update Postman collection:
   ```bash
   npm run generate-postman-collection
   ```

### Versioning

The ConHub API uses semantic versioning:
- **Major**: Breaking changes
- **Minor**: New features (backward compatible)
- **Patch**: Bug fixes

Current API version: **v1.0.0**

## Support

For API questions, issues, or support:

- 📖 [Documentation](./CONHUB_API_DOCUMENTATION.md)
- 🐛 [Issues](https://github.com/KogSector/ConHub/issues)
- 💬 [Discussions](https://github.com/KogSector/ConHub/discussions)
- 📧 [Email](mailto:support@conhub.com)

## License

This API documentation is part of the ConHub project, licensed under the MIT License. See the main project repository for full license details.
