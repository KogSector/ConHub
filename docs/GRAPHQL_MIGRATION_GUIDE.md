# GraphQL Migration Guide

## Executive Summary

ConHub is transitioning from REST APIs to a unified GraphQL architecture. This document outlines the current state, migration strategy, and implementation roadmap.

## Current State

### ✅ Implemented GraphQL Features

#### 1. GraphQL Server (Backend Service - Port 8000)
- **Location**: `backend/src/graphql/`
- **Framework**: `async-graphql` with Actix-web integration
- **Endpoint**: `http://localhost:8000/api/graphql`
- **Playground**: `http://localhost:8000/api/graphql` (GET request)

#### 2. Current GraphQL Schema

```graphql
type Query {
  # Health & Status
  health: String!
  version: String!
  
  # User & Auth
  me: CurrentUser
  
  # Embedding Service (Heavy toggle required)
  embed(texts: [String!]!, normalize: Boolean): EmbeddingResult!
  rerank(query: String!, documents: [RerankDocumentInput!]!, top_k: Int): [RerankResult!]!
}

type CurrentUser {
  user_id: String
  roles: [String!]!
}

type EmbeddingResult {
  embeddings: [[Float!]!]!
  dimension: Int!
  model: String!
  count: Int!
}

type RerankResult {
  id: String!
  score: Float!
}

input RerankDocumentInput {
  id: String!
  text: String!
}
```

#### 3. Features Implemented
- ✅ Authentication via JWT in GraphQL context
- ✅ Feature toggle integration (Heavy, Auth)
- ✅ Caching layer for embedding results
- ✅ Retry logic with exponential backoff
- ✅ Concurrency limiting via Semaphore
- ✅ GraphQL Playground for development
- ✅ CORS configuration
- ✅ Logging middleware

### ⏳ REST APIs Still in Use

The following REST endpoints are currently active and need migration:

#### Auth Service (Port 3010)
```
POST   /api/auth/register
POST   /api/auth/login
POST   /api/auth/logout
POST   /api/auth/refresh
POST   /api/auth/forgot-password
POST   /api/auth/reset-password
GET    /api/auth/oauth/google
GET    /api/auth/oauth/github
GET    /api/auth/oauth/microsoft
GET    /api/auth/verify-email
```

#### Billing Service (Port 3011)
```
POST   /api/billing/create-checkout-session
POST   /api/billing/create-portal-session
GET    /api/billing/subscription
POST   /api/billing/webhook
GET    /api/billing/plans
```

#### Data Service (Port 3013)
```
GET    /api/data/sources
POST   /api/data/sources
PUT    /api/data/sources/:id
DELETE /api/data/sources/:id
GET    /api/data/repositories
POST   /api/data/repositories/sync
GET    /api/data/documents
POST   /api/data/documents/upload
```

#### Security Service (Port 3012)
```
GET    /api/security/audit-logs
POST   /api/security/policies
GET    /api/security/policies
PUT    /api/security/policies/:id
DELETE /api/security/policies/:id
GET    /api/security/access-review
```

#### Webhook Service (Port 3015)
```
POST   /api/webhooks/github
POST   /api/webhooks/gitlab
POST   /api/webhooks/notion
POST   /api/webhooks/slack
```

#### Indexing Service (Port 8080)
```
POST   /api/indexing/index
GET    /api/indexing/search
GET    /api/indexing/stats
DELETE /api/indexing/clear
POST   /api/indexing/reindex
```

## Migration Strategy

### Phase 1: Core Queries & Mutations (Priority: HIGH)

#### 1.1 User & Authentication
```graphql
extend type Query {
  # User queries
  me: User
  users(filter: UserFilter, limit: Int, offset: Int): UserConnection!
  user(id: ID!): User
  
  # Auth queries
  verifyToken(token: String!): TokenValidation!
}

extend type Mutation {
  # Authentication
  register(input: RegisterInput!): AuthResponse!
  login(input: LoginInput!): AuthResponse!
  logout: Boolean!
  refreshToken(refreshToken: String!): AuthResponse!
  
  # Password management
  forgotPassword(email: String!): Boolean!
  resetPassword(token: String!, newPassword: String!): Boolean!
  
  # OAuth
  oauthLogin(provider: OAuthProvider!, code: String!): AuthResponse!
}

type User {
  id: ID!
  email: String!
  name: String!
  roles: [String!]!
  createdAt: DateTime!
  updatedAt: DateTime!
  subscription: Subscription
}

type AuthResponse {
  token: String!
  refreshToken: String!
  user: User!
}

input RegisterInput {
  email: String!
  password: String!
  name: String!
}

input LoginInput {
  email: String!
  password: String!
}

enum OAuthProvider {
  GOOGLE
  GITHUB
  MICROSOFT
}
```

#### 1.2 Data Sources & Repositories
```graphql
extend type Query {
  # Data sources
  dataSources(filter: DataSourceFilter): [DataSource!]!
  dataSource(id: ID!): DataSource
  
  # Repositories
  repositories(filter: RepositoryFilter): [Repository!]!
  repository(id: ID!): Repository
  
  # Documents
  documents(filter: DocumentFilter, limit: Int, offset: Int): DocumentConnection!
  document(id: ID!): Document
}

extend type Mutation {
  # Data sources
  connectDataSource(input: ConnectDataSourceInput!): DataSource!
  updateDataSource(id: ID!, input: UpdateDataSourceInput!): DataSource!
  disconnectDataSource(id: ID!): Boolean!
  
  # Repositories
  syncRepository(id: ID!): SyncResult!
  
  # Documents
  uploadDocument(input: UploadDocumentInput!): Document!
  deleteDocument(id: ID!): Boolean!
}

type DataSource {
  id: ID!
  type: DataSourceType!
  name: String!
  url: String
  status: ConnectionStatus!
  connectedAt: DateTime!
  lastSyncAt: DateTime
  repositories: [Repository!]!
  documents: [Document!]!
}

type Repository {
  id: ID!
  name: String!
  url: String!
  provider: RepositoryProvider!
  branches: [String!]!
  lastCommit: Commit
  fileCount: Int!
  indexed: Boolean!
}

type Document {
  id: ID!
  title: String!
  content: String
  fileType: String!
  size: Int!
  source: DataSource!
  uploadedAt: DateTime!
}

enum DataSourceType {
  GITHUB
  GITLAB
  BITBUCKET
  GOOGLE_DRIVE
  DROPBOX
  ONEDRIVE
  NOTION
  SLACK
  FILESYSTEM
}

enum ConnectionStatus {
  CONNECTED
  DISCONNECTED
  ERROR
  SYNCING
}

enum RepositoryProvider {
  GITHUB
  GITLAB
  BITBUCKET
}

input ConnectDataSourceInput {
  type: DataSourceType!
  name: String!
  url: String
  credentials: JSON!
}

input UpdateDataSourceInput {
  name: String
  credentials: JSON
}

input UploadDocumentInput {
  title: String!
  content: String!
  fileType: String!
  sourceId: ID!
}
```

#### 1.3 Billing & Subscriptions
```graphql
extend type Query {
  subscription: Subscription
  billingPlans: [BillingPlan!]!
  invoices(limit: Int): [Invoice!]!
}

extend type Mutation {
  createCheckoutSession(planId: ID!): CheckoutSession!
  createPortalSession: PortalSession!
  cancelSubscription: Boolean!
}

type Subscription {
  id: ID!
  plan: BillingPlan!
  status: SubscriptionStatus!
  currentPeriodStart: DateTime!
  currentPeriodEnd: DateTime!
  cancelAtPeriodEnd: Boolean!
}

type BillingPlan {
  id: ID!
  name: String!
  price: Float!
  interval: BillingInterval!
  features: [String!]!
}

type CheckoutSession {
  sessionId: String!
  url: String!
}

type PortalSession {
  url: String!
}

type Invoice {
  id: ID!
  amount: Float!
  status: InvoiceStatus!
  paidAt: DateTime
  dueDate: DateTime!
}

enum SubscriptionStatus {
  ACTIVE
  PAST_DUE
  CANCELED
  INCOMPLETE
  TRIALING
}

enum BillingInterval {
  MONTH
  YEAR
}

enum InvoiceStatus {
  PAID
  OPEN
  VOID
  UNCOLLECTIBLE
}
```

### Phase 2: Advanced Features (Priority: MEDIUM)

#### 2.1 Search & Indexing
```graphql
extend type Query {
  search(query: String!, filters: SearchFilter): SearchResults!
  indexingStats: IndexingStats!
}

extend type Mutation {
  indexContent(input: IndexInput!): IndexResult!
  reindexAll: Boolean!
  clearIndex: Boolean!
}

type SearchResults {
  results: [SearchResult!]!
  totalCount: Int!
  took: Float!
}

type SearchResult {
  id: ID!
  type: SearchResultType!
  title: String!
  snippet: String!
  score: Float!
  document: Document
  repository: Repository
}

type IndexingStats {
  totalDocuments: Int!
  totalRepositories: Int!
  totalSize: Int!
  lastIndexedAt: DateTime
  status: IndexingStatus!
}

enum SearchResultType {
  CODE
  DOCUMENT
  REPOSITORY
  FILE
}

enum IndexingStatus {
  IDLE
  INDEXING
  ERROR
}

input SearchFilter {
  type: SearchResultType
  source: ID
  dateRange: DateRangeInput
}

input IndexInput {
  repositoryId: ID
  documentId: ID
  forceReindex: Boolean
}
```

#### 2.2 Security & Audit
```graphql
extend type Query {
  auditLogs(filter: AuditLogFilter, limit: Int): [AuditLog!]!
  securityPolicies: [SecurityPolicy!]!
  accessReview(userId: ID!): AccessReview!
}

extend type Mutation {
  createSecurityPolicy(input: SecurityPolicyInput!): SecurityPolicy!
  updateSecurityPolicy(id: ID!, input: SecurityPolicyInput!): SecurityPolicy!
  deleteSecurityPolicy(id: ID!): Boolean!
}

type AuditLog {
  id: ID!
  userId: ID!
  action: String!
  resource: String!
  timestamp: DateTime!
  ipAddress: String
  userAgent: String
  status: AuditStatus!
}

type SecurityPolicy {
  id: ID!
  name: String!
  description: String
  rules: [SecurityRule!]!
  enabled: Boolean!
}

type AccessReview {
  userId: ID!
  permissions: [String!]!
  lastReviewedAt: DateTime
  status: AccessStatus!
}

enum AuditStatus {
  SUCCESS
  FAILURE
  WARNING
}

enum AccessStatus {
  APPROVED
  PENDING
  DENIED
}
```

### Phase 3: Real-time Features (Priority: LOW)

#### 3.1 Subscriptions
```graphql
type Subscription {
  # Real-time updates
  repositorySynced(id: ID!): Repository!
  documentIndexed(sourceId: ID!): Document!
  auditLogCreated(userId: ID): AuditLog!
  
  # Notifications
  notificationReceived: Notification!
}

type Notification {
  id: ID!
  type: NotificationType!
  title: String!
  message: String!
  timestamp: DateTime!
  read: Boolean!
}

enum NotificationType {
  INFO
  WARNING
  ERROR
  SUCCESS
}
```

## Implementation Roadmap

### Week 1-2: Foundation
- [ ] Extend GraphQL schema with Phase 1 types
- [ ] Create resolvers for User & Auth mutations
- [ ] Migrate authentication logic to GraphQL
- [ ] Add comprehensive error handling
- [ ] Write integration tests

### Week 3-4: Data Layer
- [ ] Implement data source queries and mutations
- [ ] Create repository and document resolvers
- [ ] Add file upload support in GraphQL
- [ ] Implement pagination and filtering
- [ ] Add caching for frequently accessed data

### Week 5-6: Billing & Search
- [ ] Migrate billing endpoints to GraphQL
- [ ] Implement search and indexing mutations
- [ ] Add security and audit queries
- [ ] Optimize query performance
- [ ] Load testing and optimization

### Week 7-8: Polish & Migration
- [ ] Deprecate REST endpoints (with warnings)
- [ ] Update frontend to use GraphQL
- [ ] Complete documentation
- [ ] Performance benchmarking
- [ ] Production deployment

## Architecture Benefits

### Before (REST)
```
Frontend → Nginx → Multiple Services (Auth, Data, Billing, etc.)
                    ↓         ↓         ↓
                Multiple DBs & APIs
```

### After (GraphQL)
```
Frontend → Nginx → GraphQL Gateway (Backend:8000)
                           ↓
                    Schema Stitching
                           ↓
         ┌─────────────────┼─────────────────┐
         ↓                 ↓                  ↓
    Auth Service     Data Service      Billing Service
         ↓                 ↓                  ↓
    PostgreSQL        Qdrant            Stripe API
```

### Advantages

1. **Single Entry Point**: One GraphQL endpoint instead of multiple REST endpoints
2. **Reduced Over-fetching**: Clients request only what they need
3. **Type Safety**: Strong typing across frontend and backend
4. **Better Developer Experience**: GraphQL Playground, introspection
5. **Easier Versioning**: Deprecation instead of versioning
6. **Optimized Queries**: Batch requests, N+1 problem resolution
7. **Real-time Support**: WebSocket subscriptions built-in

## Migration Checklist

### Backend Changes
- [ ] Extend GraphQL schema for all REST endpoints
- [ ] Implement resolvers with business logic
- [ ] Add authentication & authorization to resolvers
- [ ] Implement caching strategy (DataLoader pattern)
- [ ] Add comprehensive error handling
- [ ] Write unit tests for resolvers
- [ ] Write integration tests for schema
- [ ] Add GraphQL middleware (logging, metrics)
- [ ] Implement rate limiting
- [ ] Add query complexity analysis

### Frontend Changes
- [ ] Install GraphQL client (Apollo Client or urql)
- [ ] Create GraphQL queries/mutations
- [ ] Replace REST API calls with GraphQL
- [ ] Implement optimistic updates
- [ ] Add error boundaries
- [ ] Update state management (if needed)
- [ ] Add loading states
- [ ] Implement pagination
- [ ] Add real-time subscriptions

### DevOps Changes
- [ ] Update monitoring for GraphQL metrics
- [ ] Add GraphQL-specific logging
- [ ] Configure APM for GraphQL
- [ ] Update documentation
- [ ] Create migration scripts
- [ ] Add backward compatibility layer
- [ ] Plan deprecation timeline
- [ ] Communicate changes to team

## Testing Strategy

### Unit Tests
```typescript
// Example resolver test
describe('User Resolver', () => {
  it('should return current user', async () => {
    const result = await executeQuery({
      schema,
      query: 'query { me { id email } }',
      context: { user: mockUser }
    });
    expect(result.data.me).toEqual({
      id: mockUser.id,
      email: mockUser.email
    });
  });
});
```

### Integration Tests
```typescript
// Example API test
describe('GraphQL API', () => {
  it('should authenticate and fetch user', async () => {
    const loginResult = await gqlClient.mutate({
      mutation: LOGIN_MUTATION,
      variables: { email, password }
    });
    
    const token = loginResult.data.login.token;
    
    const meResult = await gqlClient.query({
      query: ME_QUERY,
      context: { headers: { Authorization: `Bearer ${token}` } }
    });
    
    expect(meResult.data.me.email).toBe(email);
  });
});
```

## Performance Optimization

### DataLoader Pattern
```rust
// Batch loading to solve N+1 problem
pub struct DocumentLoader {
    pool: PgPool,
}

impl DocumentLoader {
    async fn load_many(&self, ids: Vec<Uuid>) -> Result<Vec<Document>> {
        // Single query to load all documents
        sqlx::query_as!(
            Document,
            "SELECT * FROM documents WHERE id = ANY($1)",
            &ids
        )
        .fetch_all(&self.pool)
        .await
    }
}
```

### Query Complexity
```rust
// Limit query complexity to prevent abuse
let schema = Schema::build(query, mutation, subscription)
    .limit_complexity(100)
    .limit_depth(10)
    .finish();
```

### Caching
```rust
// Cache frequently accessed data
#[Object]
impl QueryRoot {
    #[graphql(cache_control(max_age = 3600))]
    async fn public_repositories(&self) -> Vec<Repository> {
        // Cached for 1 hour
    }
}
```

## Best Practices

### 1. Naming Conventions
- Use camelCase for field names
- Use PascalCase for types
- Use SCREAMING_SNAKE_CASE for enums
- Prefix input types with `Input`
- Suffix connection types with `Connection`

### 2. Error Handling
```graphql
type MutationResult {
  success: Boolean!
  message: String
  code: ErrorCode
  data: JSON
}

enum ErrorCode {
  VALIDATION_ERROR
  AUTHENTICATION_ERROR
  AUTHORIZATION_ERROR
  NOT_FOUND
  INTERNAL_ERROR
}
```

### 3. Pagination
```graphql
type DocumentConnection {
  edges: [DocumentEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type DocumentEdge {
  node: Document!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}
```

### 4. Nullability
- Make fields non-nullable by default
- Use nullable only for optional data
- Always return non-null for IDs
- Use empty arrays instead of null arrays

## Migration Examples

### Before (REST)
```typescript
// Multiple API calls
const user = await fetch('/api/auth/me').then(r => r.json());
const repos = await fetch('/api/data/repositories').then(r => r.json());
const docs = await fetch('/api/data/documents').then(r => r.json());
```

### After (GraphQL)
```typescript
// Single optimized query
const { data } = await client.query({
  query: gql`
    query Dashboard {
      me {
        id
        name
        email
      }
      repositories {
        id
        name
        lastCommit {
          message
          author
        }
      }
      documents(limit: 10) {
        edges {
          node {
            id
            title
          }
        }
      }
    }
  `
});
```

## Monitoring & Observability

### Metrics to Track
- Query execution time
- Resolver execution time
- Cache hit rate
- Error rate by resolver
- Query complexity distribution
- Concurrent requests
- DataLoader batch sizes

### Tools
- **Apollo Studio**: GraphQL observability
- **Prometheus**: Metrics collection
- **Grafana**: Visualization
- **Jaeger**: Distributed tracing

## Resources

### Documentation
- [async-graphql Documentation](https://async-graphql.github.io/async-graphql/)
- [GraphQL Best Practices](https://graphql.org/learn/best-practices/)
- [GraphQL Schema Design](https://www.apollographql.com/docs/apollo-server/schema/schema/)

### Internal Files
- `backend/src/graphql/schema.rs` - Current schema
- `backend/src/graphql/mod.rs` - GraphQL setup
- `backend/src/main.rs` - Server configuration

---

**Status**: Phase 0 Complete (Basic GraphQL server running)  
**Next Milestone**: Complete Phase 1 (Auth & Data queries)  
**Target Completion**: 8 weeks from start date  
**Last Updated**: November 2024
