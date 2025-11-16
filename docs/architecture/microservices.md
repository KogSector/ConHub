# ConHub Microservices Architecture

## Overview

ConHub is built using a microservices architecture that provides scalability, maintainability, and fault isolation. Each service has a specific responsibility and communicates with others through well-defined APIs. This document provides a comprehensive guide to each microservice, its responsibilities, and integration patterns.

## Architecture Principles

### 1. Single Responsibility
Each microservice has a single, well-defined responsibility and owns its data.

### 2. Autonomous Services
Services can be developed, deployed, and scaled independently.

### 3. Decentralized Governance
Each team owns their service's technology stack and deployment pipeline.

### 4. Failure Isolation
Service failures are contained and don't cascade to other services.

### 5. Data Consistency
Services maintain their own data stores with eventual consistency across boundaries.

## Service Catalog

### 1. Authentication Service (`auth/`)

**Purpose**: Centralized authentication and authorization for all ConHub services.

**Responsibilities**:
- User registration and login
- JWT token generation and validation
- OAuth provider integration (GitHub, Google, etc.)
- Role-based access control (RBAC)
- Session management
- Multi-factor authentication

**Technology Stack**:
- **Runtime**: Rust with Actix-web
- **Database**: PostgreSQL (users, roles, sessions)
- **Cache**: Redis (session storage, token blacklist)
- **External APIs**: OAuth providers

**Key Components**:
```rust
// Core authentication structures
pub struct AuthService {
    db_pool: PgPool,
    redis_client: RedisClient,
    jwt_keys: JwtKeys,
    oauth_providers: HashMap<String, OAuthProvider>,
}

pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<Role>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
```

**API Endpoints**:
- `POST /auth/register` - User registration
- `POST /auth/login` - User authentication
- `POST /auth/logout` - Session termination
- `GET /auth/me` - Current user profile
- `POST /auth/oauth/{provider}` - OAuth initiation
- `GET /auth/oauth/{provider}/callback` - OAuth callback

**Environment Variables**:
```bash
AUTH_SERVICE_PORT=3001
JWT_SECRET_KEY=<rsa-private-key>
JWT_PUBLIC_KEY=<rsa-public-key>
OAUTH_GITHUB_CLIENT_ID=<github-client-id>
OAUTH_GITHUB_CLIENT_SECRET=<github-client-secret>
```

---

### 2. Data Service (`data/`)

**Purpose**: Core data management, repository connections, and document processing.

**Responsibilities**:
- Repository connection management
- Document ingestion and processing
- Connector orchestration (GitHub, GitLab, Bitbucket)
- File system operations
- Data synchronization
- Search and retrieval

**Technology Stack**:
- **Runtime**: Rust with Actix-web
- **Database**: PostgreSQL (metadata), Qdrant (vectors)
- **Message Queue**: Redis Streams
- **File Storage**: S3-compatible storage

**Key Components**:
```rust
pub struct DataService {
    db_pool: PgPool,
    vector_client: QdrantClient,
    connector_manager: ConnectorManager,
    storage_client: S3Client,
}

pub struct ConnectedAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connector_type: ConnectorType,
    pub credentials: EncryptedCredentials,
    pub status: ConnectionStatus,
}
```

**Connector Framework**:
```rust
#[async_trait]
pub trait Connector {
    fn name(&self) -> &str;
    fn connector_type(&self) -> ConnectorType;
    async fn authenticate(&self, config: &ConnectorConfig) -> Result<String, ConnectorError>;
    async fn sync(&self, account: &ConnectedAccount) -> Result<SyncResult, ConnectorError>;
    async fn list_documents(&self, account: &ConnectedAccount) -> Result<Vec<DocumentMetadata>, ConnectorError>;
}
```

**API Endpoints**:
- `GET /api/data/accounts` - List connected accounts
- `POST /api/data/accounts` - Connect new account
- `DELETE /api/data/accounts/{id}` - Disconnect account
- `POST /api/data/sync/{account_id}` - Trigger sync
- `GET /api/data/documents` - Search documents
- `GET /api/data/documents/{id}` - Get document content

---

### 3. Billing Service (`billing/`)

**Purpose**: Subscription management, payment processing, and usage tracking.

**Responsibilities**:
- Subscription plan management
- Payment processing via Stripe
- Usage tracking and billing
- Invoice generation
- Payment method management
- Subscription lifecycle management

**Technology Stack**:
- **Runtime**: Rust with Actix-web
- **Database**: PostgreSQL (billing data)
- **Payment Processor**: Stripe API
- **Webhooks**: Stripe webhook handling

**Key Components**:
```rust
pub struct BillingService {
    db_pool: PgPool,
    stripe_client: StripeClient,
}

pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub tier: String,
    pub price_monthly: Decimal,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
}
```

**API Endpoints**:
- `GET /api/billing/plans` - List subscription plans
- `GET /api/billing/dashboard` - Billing dashboard data
- `POST /api/billing/subscription` - Create subscription
- `PUT /api/billing/subscription/{id}` - Update subscription
- `DELETE /api/billing/subscription/{id}` - Cancel subscription
- `POST /api/billing/webhooks/stripe` - Stripe webhook handler

---

### 4. Embedding Service (`embedding/`)

**Purpose**: Document vectorization and semantic search capabilities.

**Responsibilities**:
- Text embedding generation
- Vector similarity search
- AI model integration
- Semantic indexing
- Search result ranking

**Technology Stack**:
- **Runtime**: Python with FastAPI
- **ML Framework**: Transformers, Sentence-Transformers
- **Vector Database**: Qdrant
- **GPU Support**: CUDA for model inference

**Key Components**:
```python
class EmbeddingService:
    def __init__(self):
        self.model = SentenceTransformer('all-MiniLM-L6-v2')
        self.vector_client = QdrantClient()
    
    async def generate_embeddings(self, texts: List[str]) -> List[List[float]]:
        """Generate embeddings for input texts"""
        
    async def search_similar(self, query: str, limit: int = 10) -> List[SearchResult]:
        """Find semantically similar documents"""
```

**API Endpoints**:
- `POST /embed/text` - Generate text embeddings
- `POST /embed/documents` - Batch document embedding
- `POST /search/semantic` - Semantic search
- `GET /health` - Service health check

---

### 5. Frontend Service (`frontend/`)

**Purpose**: User interface and client-side application logic.

**Responsibilities**:
- React-based web application
- User authentication flows
- Repository management UI
- Document browsing and search
- Billing and subscription management
- Real-time updates via WebSocket

**Technology Stack**:
- **Framework**: Next.js 14 with React 18
- **Styling**: Tailwind CSS with shadcn/ui
- **State Management**: React Context + Hooks
- **Authentication**: JWT with refresh tokens
- **Real-time**: WebSocket connections

**Key Components**:
```typescript
// Authentication context
interface AuthContext {
  user: User | null;
  token: string | null;
  login: (credentials: LoginCredentials) => Promise<void>;
  logout: () => void;
  isAuthenticated: boolean;
}

// API client
class ApiClient {
  private baseUrl: string;
  private token: string | null;
  
  async get<T>(endpoint: string): Promise<ApiResponse<T>>;
  async post<T>(endpoint: string, data: any): Promise<ApiResponse<T>>;
}
```

**Routes**:
- `/` - Landing page
- `/dashboard` - Main dashboard
- `/repositories` - Repository management
- `/docs` - Document browser
- `/billing` - Billing and subscriptions
- `/settings` - User settings

---

### 6. Security Service (`security/`)

**Purpose**: Centralized security policies and threat detection.

**Responsibilities**:
- Security policy enforcement
- Threat detection and response
- Audit logging
- Compliance monitoring
- Vulnerability scanning
- Incident response coordination

**Technology Stack**:
- **Runtime**: Rust with Actix-web
- **Database**: PostgreSQL (audit logs)
- **Monitoring**: Prometheus + Grafana
- **Alerting**: PagerDuty integration

**Key Components**:
```rust
pub struct SecurityService {
    audit_logger: AuditLogger,
    threat_detector: ThreatDetector,
    policy_engine: PolicyEngine,
}

pub struct AuditEvent {
    pub event_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}
```

---

### 7. Webhook Service (`webhook/`)

**Purpose**: External webhook handling and event processing.

**Responsibilities**:
- GitHub/GitLab webhook processing
- Stripe payment webhooks
- Real-time repository updates
- Event routing and processing
- Webhook signature verification

**Technology Stack**:
- **Runtime**: Node.js with Express
- **Queue**: Redis for event processing
- **Database**: PostgreSQL (webhook logs)

---

## Service Communication

### Synchronous Communication
- **HTTP/REST**: Primary communication method
- **Authentication**: JWT tokens for service-to-service auth
- **Load Balancing**: Nginx for request distribution
- **Circuit Breakers**: Hystrix pattern for fault tolerance

### Asynchronous Communication
- **Message Queues**: Redis Streams for event-driven communication
- **Event Sourcing**: Domain events for state changes
- **Pub/Sub**: Redis pub/sub for real-time notifications

### Service Discovery
- **DNS-based**: Kubernetes DNS for service resolution
- **Health Checks**: Regular health check endpoints
- **Load Balancing**: Kubernetes service load balancing

## Data Management

### Database per Service
Each service owns its data and database schema:

- **Auth Service**: User accounts, roles, sessions
- **Data Service**: Repositories, documents, sync jobs
- **Billing Service**: Subscriptions, payments, invoices
- **Security Service**: Audit logs, security events

### Data Consistency
- **Eventual Consistency**: Across service boundaries
- **Saga Pattern**: For distributed transactions
- **Event Sourcing**: For audit trails and state reconstruction

### Shared Data
- **Read Replicas**: For cross-service queries
- **Data Synchronization**: Event-driven data sync
- **Caching**: Redis for frequently accessed data

## Deployment Architecture

### Containerization
```dockerfile
# Example Dockerfile for Rust services
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/service /usr/local/bin/service
CMD ["service"]
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: auth-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: auth-service
  template:
    metadata:
      labels:
        app: auth-service
    spec:
      containers:
      - name: auth-service
        image: conhub/auth-service:latest
        ports:
        - containerPort: 3001
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-secret
              key: url
```

### Service Mesh
- **Istio**: Traffic management and security
- **mTLS**: Automatic mutual TLS between services
- **Observability**: Distributed tracing with Jaeger

## Monitoring & Observability

### Metrics
- **Prometheus**: Metrics collection
- **Grafana**: Metrics visualization
- **Custom Metrics**: Business and technical metrics

### Logging
- **Structured Logging**: JSON-formatted logs
- **Centralized Logging**: ELK stack (Elasticsearch, Logstash, Kibana)
- **Log Correlation**: Request ID tracking across services

### Tracing
- **Distributed Tracing**: Jaeger for request tracing
- **Performance Monitoring**: APM tools for performance insights
- **Error Tracking**: Sentry for error monitoring

## Development Guidelines

### API Design
- **RESTful APIs**: Consistent REST patterns
- **OpenAPI Specs**: API documentation with Swagger
- **Versioning**: API versioning strategy
- **Error Handling**: Consistent error response format

### Testing Strategy
- **Unit Tests**: Service-level unit testing
- **Integration Tests**: Service integration testing
- **Contract Tests**: API contract testing with Pact
- **End-to-End Tests**: Full system testing

### CI/CD Pipeline
```yaml
# Example GitHub Actions workflow
name: Service CI/CD
on:
  push:
    branches: [main]
    paths: ['auth/**']

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Run tests
      run: cargo test
    
  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
    - name: Build and push Docker image
      run: |
        docker build -t conhub/auth-service:${{ github.sha }} .
        docker push conhub/auth-service:${{ github.sha }}
    
  deploy:
    needs: build
    runs-on: ubuntu-latest
    steps:
    - name: Deploy to Kubernetes
      run: kubectl set image deployment/auth-service auth-service=conhub/auth-service:${{ github.sha }}
```

## Troubleshooting Guide

### Common Issues
1. **Service Discovery**: DNS resolution problems
2. **Database Connections**: Connection pool exhaustion
3. **Authentication**: JWT token validation failures
4. **Rate Limiting**: API rate limit exceeded

### Debugging Tools
- **kubectl**: Kubernetes cluster inspection
- **docker logs**: Container log inspection
- **curl**: API endpoint testing
- **pgcli**: Database query tool

### Performance Optimization
- **Connection Pooling**: Database connection optimization
- **Caching**: Redis caching strategies
- **Load Balancing**: Traffic distribution optimization
- **Resource Limits**: CPU and memory optimization

---

**Last Updated**: November 2024  
**Version**: 1.0  
**Maintainer**: ConHub Engineering Team
