# ConHub API Documentation

## Overview

ConHub is a comprehensive AI-powered platform built on a microservices architecture that connects multiple knowledge sources with AI agents through the Model Context Protocol (MCP). This documentation covers all available APIs across the entire platform.

## Architecture

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

## Services & Ports

| Service | Port | Protocol | Description |
|---------|------|----------|-------------|
| Frontend | 3000 | HTTP | Next.js React application |
| Nginx Gateway | 80 | HTTP | API Gateway & Load Balancer |
| Backend | 8000 | HTTP/GraphQL | Unified GraphQL API gateway |
| Auth Service | 3010 | HTTP | Authentication & JWT management |
| Billing Service | 3011 | HTTP | Stripe payments & subscriptions |
| AI Service | 3012 | HTTP | AI client integrations |
| Data Service | 3013 | HTTP | Data sources & repository management |
| Security Service | 3014 | HTTP | Security policies & audit logs |
| Webhook Service | 3015 | HTTP | External webhook handling |
| Indexing Service | 8080 | HTTP | Code/document indexing & search |
| Vector RAG | 8082 | HTTP | Fusion embeddings & vector search |
| PostgreSQL | 5432 | TCP | Primary database |
| Redis | 6379 | TCP | Cache & sessions |
| Qdrant | 6333 | HTTP | Vector database for semantic search |

## Authentication

All API endpoints (except health checks and public OAuth endpoints) require authentication via JWT tokens. Include the token in the Authorization header:

```
Authorization: Bearer <jwt_token>
```

## Feature Toggles

The platform supports feature toggles that control service availability:

- **Auth**: Controls authentication & database connections
- **Redis**: Controls Redis for sessions/caching (requires Auth)
- **Heavy**: Controls resource-intensive services (indexers, embeddings)
- **Docker**: Switches between Docker and local development mode

---

# 1. Backend Service (Port 8000)

The backend service acts as a unified API gateway providing both REST endpoints and GraphQL federation.

## Base URL
```
http://localhost:8000
```

## REST API Endpoints

### Health Checks
- `GET /health` - Service health check
- `GET /ready` - Readiness check

### Authentication Routes
- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login
- `POST /api/auth/logout` - User logout
- `GET /api/auth/me` - Get current user
- `GET /api/auth/profile` - Get user profile
- `PUT /api/auth/profile` - Update user profile
- `POST /api/auth/verify` - Verify token
- `POST /api/auth/refresh` - Refresh token
- `POST /api/auth/oauth/google` - Google OAuth
- `POST /api/auth/oauth/github` - GitHub OAuth
- `POST /api/auth/oauth/microsoft` - Microsoft OAuth
- `POST /api/auth/reset-password` - Request password reset
- `POST /api/auth/reset-password/confirm` - Confirm password reset

### Data Management Routes
- `POST /api/data/sources` - Connect data source
- `GET /api/data/sources` - List data sources
- `POST /api/data/sources/{id}/sync` - Sync data source
- `GET /api/data-sources` - List data sources (alternative endpoint)
- `POST /api/data-sources/connect` - Connect data source (alternative)
- `DELETE /api/data-sources/{id}` - Delete data source

### Billing Routes
- `GET /api/billing/plans` - Get subscription plans
- `GET /api/billing/dashboard` - Get billing dashboard
- `POST /api/billing/customers` - Create customer
- `POST /api/billing/payment-intents` - Create payment intent
- `POST /api/billing/setup-intents` - Create setup intent
- `GET /api/billing/subscription` - Get current subscription
- `POST /api/billing/subscription` - Create subscription
- `DELETE /api/billing/subscriptions/{subscription_id}` - Cancel subscription
- `POST /api/billing/payment-methods` - Add payment method
- `GET /api/billing/customers/{customer_id}/payment-methods` - Get payment methods
- `GET /api/billing/invoices` - Get current user invoices
- `GET /api/billing/customers/{customer_id}/invoices` - Get customer invoices
- `POST /api/billing/webhooks/stripe` - Handle Stripe webhook

### Security Routes
- `GET /api/security/rulesets` - List security rulesets
- `POST /api/security/rulesets` - Create ruleset
- `GET /api/security/rulesets/{id}` - Get ruleset
- `PUT /api/security/rulesets/{id}` - Update ruleset
- `DELETE /api/security/rulesets/{id}` - Delete ruleset
- `GET /api/security/audit-logs` - Get audit logs
- `GET /api/security/connections` - List social connections
- `POST /api/security/connections/configure` - Configure connection
- `POST /api/security/connections/connect` - Connect social provider
- `POST /api/security/connections/oauth/callback` - OAuth callback
- `DELETE /api/security/connections/{id}` - Disconnect connection
- `GET /api/security/connections/{provider}/files` - List provider files

### Webhook Routes
- `POST /api/webhooks/github` - Handle GitHub webhook
- `POST /api/webhooks/gitlab` - Handle GitLab webhook
- `POST /api/webhooks/stripe` - Handle Stripe webhook

### RAG Routes
- `POST /api/rag/query` - RAG query endpoint
- `POST /api/rag/ingest` - Ingest documents
- `GET /api/rag/sources` - List RAG sources

### Indexing Routes
- `POST /api/indexing/index` - Index documents
- `GET /api/indexing/status` - Get indexing status
- `DELETE /api/indexing/{id}` - Delete index

### Dashboard
- `GET /api/dashboard/stats` - Get dashboard statistics

## GraphQL API

### Endpoint
```
POST /api/graphql
```

### Playground
```
GET /api/graphql
```

### Schema

#### Query Types

**Health & Status**
```graphql
query {
  health: String
  version: String
}
```

**User Information**
```graphql
query {
  me {
    user_id: String
    roles: [String]
  }
}
```

**Embedding Generation**
```graphql
query {
  embed(texts: [String!], normalize: Boolean): EmbeddingResult {
    embeddings: [[Float]]
    dimension: Int
    model: String
    count: Int
  }
}
```

**Document Reranking**
```graphql
query {
  rerank(
    query: String!
    documents: [RerankDocumentInput!]!
    top_k: Int
  ): [RerankResult!] {
    id: String
    score: Float
    index: Int
    document: RerankDocumentOutput {
      id: String
      text: String
      metadata: JSON
    }
  }
}
```

#### Input Types

**RerankDocumentInput**
```graphql
input RerankDocumentInput {
  id: String!
  text: String!
  metadata: JSON
}
```

---

# 2. Authentication Service (Port 3010)

Handles user authentication, OAuth integrations, and JWT token management.

## Base URL
```
http://localhost:3010
```

## Health Check
- `GET /health` - Service health check

## Public Endpoints (No Authentication Required)

### Core Authentication
- `POST /api/auth/login` - User login
- `POST /api/auth/register` - User registration
- `POST /api/auth/forgot-password` - Request password reset
- `POST /api/auth/reset-password` - Reset password

### OAuth Endpoints
- `GET /api/auth/oauth/url` - Get OAuth URL for provider
- `GET /api/auth/oauth/{provider}` - Initiate OAuth flow
- `GET /api/auth/oauth/{provider}/callback` - OAuth callback handler

### Auth0 Integration
- `POST /api/auth/auth0/exchange` - Exchange Auth0 token

### Public Connection Info
- `GET /api/auth/connections/current` - Get current user's connections (public info only)

## Protected Endpoints (Authentication Required)

### User Management
- `POST /api/auth/logout` - User logout
- `GET /api/auth/me` - Get current user details
- `POST /api/auth/verify` - Verify JWT token
- `POST /api/auth/refresh` - Refresh JWT token
- `GET /api/auth/profile` - Get user profile
- `POST /api/auth/oauth/exchange` - Exchange OAuth token

### OAuth & Connections
- `GET /api/auth/connections` - List OAuth connections
- `DELETE /api/auth/connections/{id}` - Delete OAuth connection

### Repository Access
- `GET /api/auth/repos/github` - List GitHub repositories
- `GET /api/auth/repos/github/branches` - List GitHub repository branches
- `GET /api/auth/repos/bitbucket` - List Bitbucket repositories
- `GET /api/auth/repos/bitbucket/branches` - List Bitbucket repository branches
- `POST /api/auth/repos/check` - Check repository access

### Admin Endpoints (Admin Role Required)
- `GET /api/auth/admin/users` - List all users

## Internal Endpoints (Service-to-Service)

- `GET /internal/oauth/{provider}/token` - Get OAuth token for service
- `GET /internal/oauth/{provider}/status` - Check OAuth status

## Request/Response Examples

### Login Request
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

### Login Response
```json
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "Test User"
  }
}
```

### Registration Request
```json
{
  "email": "newuser@example.com",
  "password": "SecurePass123!",
  "name": "New User"
}
```

---

# 3. Data Service (Port 3013)

Manages data sources, repository synchronization, and document ingestion.

## Base URL
```
http://localhost:3013
```

## Health Checks
- `GET /health` - Service health check
- `GET /status` - Detailed service status

## GitHub Integration

### Repository Validation
- `POST /api/github/validate-access` - Validate GitHub repository access

**Request:**
```json
{
  "repo_url": "https://github.com/owner/repo",
  "access_token": "github_pat_..."
}
```

**Response:**
```json
{
  "success": true,
  "has_access": true,
  "repo_info": {
    "name": "repo",
    "full_name": "owner/repo",
    "default_branch": "main",
    "private": false,
    "permissions": {
      "pull": true,
      "push": true,
      "admin": false
    },
    "languages": ["Rust", "JavaScript"],
    "branches": [
      {"name": "main", "protected": false},
      {"name": "develop", "protected": false}
    ]
  }
}
```

### Repository Synchronization
- `POST /api/github/sync-repository` - Sync repository (legacy PAT-based)
- `POST /api/github/sync` - Secure sync (OAuth-based, recommended)

**Secure Sync Request:**
```json
{
  "repo_url": "https://github.com/owner/repo",
  "branch": "main",
  "include_languages": ["Rust", "JavaScript"],
  "exclude_paths": ["node_modules", "target"],
  "max_file_size_mb": 5,
  "file_extensions": [".rs", ".js", ".md"],
  "fetch_issues": true,
  "fetch_prs": true
}
```

**Sync Response:**
```json
{
  "success": true,
  "documents_processed": 150,
  "embeddings_created": 150,
  "sync_duration_ms": 15420,
  "issues_processed": 25,
  "prs_processed": 12,
  "graph_job_id": "550e8400-e29b-41d4-a716-446655440000",
  "error_message": null
}
```

### Repository Information
- `POST /api/github/branches` - Get repository branches
- `POST /api/github/languages` - Get repository languages

## OAuth-Based Repository Management

### Repository Access Check
- `POST /api/repositories/oauth/check` - Check repo access using OAuth

**Request:**
```json
{
  "provider": "github",
  "repo_url": "https://github.com/owner/repo"
}
```

**Response:**
```json
{
  "success": true,
  "provider": "github",
  "name": "repo",
  "full_name": "owner/repo",
  "default_branch": "main",
  "private": false,
  "has_read_access": true,
  "languages": ["Rust", "JavaScript"],
  "code": null,
  "error": null
}
```

### Branch Management
- `GET /api/repositories/oauth/branches?provider=github&repo=owner/repo` - Get branches

**Response:**
```json
{
  "success": true,
  "data": {
    "branches": ["main", "develop", "feature-branch"],
    "default_branch": "main"
  }
}
```

## Data Source Management

### Create Data Source
- `POST /api/data/sources` - Create new data source

**Request:**
```json
{
  "type": "github",
  "url": "https://github.com/owner/repo",
  "credentials": {
    "token": "github_pat_..."
  },
  "config": {
    "name": "My Repository",
    "defaultBranch": "main",
    "fileExtensions": [".rs", ".js", ".md"],
    "fetchIssues": true,
    "fetchPrs": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Data source created successfully. Sync will be triggered automatically.",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My Repository",
    "type": "github",
    "url": "https://github.com/owner/repo",
    "status": "connected",
    "defaultBranch": "main",
    "createdAt": "2023-12-01T10:00:00Z"
  },
  "syncStarted": true
}
```

### List Data Sources
- `GET /api/data-sources` - List all connected data sources
- `GET /api/repositories` - List connected repositories

**Response:**
```json
{
  "success": true,
  "message": "Retrieved 3 data sources",
  "data": {
    "dataSources": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "user_id": "550e8400-e29b-41d4-a716-446655440001",
        "name": "My Repository",
        "source_type": "github",
        "url": "https://github.com/owner/repo",
        "status": "active",
        "default_branch": "main",
        "sync_status": "completed",
        "documents_count": 150,
        "created_at": "2023-12-01T10:00:00Z"
      }
    ]
  }
}
```

## Document Management

### Configure Document Routes
Document management endpoints are configured under `/api/documents` (see handlers::documents::configure)

### Local Filesystem Sync
- Routes configured under `/api/localfs` (see handlers::local_fs::configure)

## GitHub App Integration

If GitHub App is configured, additional endpoints are available:
- `/api/github-app/*` - GitHub App specific endpoints

---

# 4. Billing Service (Port 3011)

Handles Stripe payments, subscriptions, and billing management.

## Base URL
```
http://localhost:3011
```

## Health Check
- `GET /health` - Service health check

## Subscription Plans
- `GET /api/billing/plans` - Get available subscription plans

**Response:**
```json
[
  {
    "id": "plan_free",
    "name": "Free",
    "price": 0,
    "currency": "USD",
    "interval": "month",
    "features": ["Basic search", "Limited storage"],
    "limits": {
      "documents": 100,
      "queries": 1000
    }
  },
  {
    "id": "plan_pro",
    "name": "Pro",
    "price": 2999,
    "currency": "USD",
    "interval": "month",
    "features": ["Advanced search", "Unlimited storage", "Priority support"],
    "limits": {
      "documents": -1,
      "queries": -1
    }
  }
]
```

## Billing Dashboard
- `GET /api/billing/dashboard` - Get billing dashboard

**Response:**
```json
{
  "current_plan": {
    "id": "plan_pro",
    "name": "Pro",
    "status": "active"
  },
  "usage": {
    "documents": 1250,
    "queries": 15420,
    "storage_gb": 2.5
  },
  "billing": {
    "next_invoice": "2023-12-01T00:00:00Z",
    "amount": 2999,
    "currency": "USD"
  }
}
```

## Customer Management

### Create Customer
- `POST /api/billing/customers` - Create new customer

**Request:**
```json
{
  "email": "customer@example.com",
  "name": "Customer Name"
}
```

**Response:**
```json
{
  "success": true,
  "customer_id": "cus_1234567890"
}
```

## Payment Methods

### Create Payment Intent
- `POST /api/billing/payment-intents` - Create payment intent

**Request:**
```json
{
  "amount": 2999,
  "currency": "USD",
  "customer_id": "cus_1234567890"
}
```

**Response:**
```json
{
  "success": true,
  "payment_intent_id": "pi_1234567890"
}
```

### Create Setup Intent
- `POST /api/billing/setup-intents` - Create setup intent for saving cards

**Request:**
```json
{
  "customer_id": "cus_1234567890"
}
```

**Response:**
```json
{
  "success": true,
  "setup_intent_id": "seti_1234567890"
}
```

### Get Payment Methods
- `GET /api/billing/customers/{customer_id}/payment-methods` - Get customer payment methods

**Response:**
```json
{
  "success": true,
  "payment_methods": [
    {
      "id": "pm_1234567890",
      "type": "card",
      "card": {
        "brand": "visa",
        "last4": "4242",
        "exp_month": 12,
        "exp_year": 2024
      },
      "is_default": true
    }
  ]
}
```

### Add Payment Method
- `POST /api/billing/payment-methods` - Add payment method

## Subscription Management

### Create Subscription
- `POST /api/billing/subscription` - Create subscription

**Request:**
```json
{
  "customer_id": "cus_1234567890",
  "price_id": "price_1234567890",
  "payment_method_id": "pm_1234567890"
}
```

**Response:**
```json
{
  "success": true,
  "subscription_id": "sub_1234567890",
  "status": "active",
  "plan_id": "plan_pro"
}
```

### Get Subscription
- `GET /api/billing/subscription` - Get current user subscription
- `GET /api/billing/subscriptions/{user_id}` - Get user subscription

**Response:**
```json
{
  "id": "sub_1234567890",
  "status": "active",
  "plan": {
    "id": "plan_pro",
    "name": "Pro",
    "price": 2999,
    "currency": "USD"
  },
  "current_period_start": "2023-11-01T00:00:00Z",
  "current_period_end": "2023-12-01T00:00:00Z",
  "cancel_at_period_end": false
}
```

### Cancel Subscription
- `DELETE /api/billing/subscriptions/{subscription_id}` - Cancel subscription

**Response:**
```json
{
  "success": true,
  "message": "Subscription cancelled successfully"
}
```

## Invoices

### Get Invoices
- `GET /api/billing/invoices` - Get current user invoices
- `GET /api/billing/customers/{customer_id}/invoices` - Get customer invoices

**Response:**
```json
{
  "success": true,
  "invoices": [
    {
      "id": "in_1234567890",
      "amount": 2999,
      "currency": "USD",
      "status": "paid",
      "created": "2023-11-01T00:00:00Z",
      "due_date": "2023-11-15T00:00:00Z",
      "paid_at": "2023-11-14T10:30:00Z",
      "invoice_pdf": "https://stripe.com/invoice/pdf/..."
    }
  ]
}
```

## Webhooks

### Handle Stripe Webhook
- `POST /api/billing/webhooks/stripe` - Handle Stripe webhook events

The webhook handler processes events like:
- `invoice.payment_succeeded`
- `invoice.payment_failed`
- `customer.subscription.created`
- `customer.subscription.deleted`
- `customer.subscription.updated`

---

# 5. Security Service (Port 3014)

Manages security policies, audit logs, and social connections.

## Base URL
```
http://localhost:3014
```

## Health Check
- `GET /health` - Service health check

## Security Rulesets

### List Rulesets
- `GET /api/security/rulesets` - List all security rulesets

**Response:**
```json
[
  {
    "id": "ruleset_123",
    "name": "Default Security Policy",
    "description": "Basic security rules for all users",
    "enabled": true,
    "rules": [
      {
        "id": "rule_001",
        "type": "rate_limiting",
        "config": {
          "requests_per_minute": 100,
          "burst_size": 200
        }
      }
    ]
  }
]
```

### Create Ruleset
- `POST /api/security/rulesets` - Create new security ruleset

**Request:**
```json
{
  "name": "Enhanced Security Policy",
  "description": "Advanced security rules for premium users",
  "enabled": true,
  "rules": [
    {
      "type": "rate_limiting",
      "config": {
        "requests_per_minute": 200,
        "burst_size": 500
      }
    },
    {
      "type": "ip_whitelist",
      "config": {
        "allowed_ips": ["192.168.1.0/24"]
      }
    }
  ]
}
```

### Get Ruleset
- `GET /api/security/rulesets/{id}` - Get specific ruleset

### Update Ruleset
- `PUT /api/security/rulesets/{id}` - Update ruleset

### Delete Ruleset
- `DELETE /api/security/rulesets/{id}` - Delete ruleset

## Audit Logs

### Get Audit Logs
- `GET /api/security/audit-logs` - Get security audit logs

**Query Parameters:**
- `user_id` - Filter by user ID
- `action` - Filter by action type
- `start_date` - Filter by start date
- `end_date` - Filter by end date
- `limit` - Number of logs to return (default: 50)
- `offset` - Offset for pagination (default: 0)

**Response:**
```json
{
  "logs": [
    {
      "id": "audit_123",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "action": "login_success",
      "resource": "/api/auth/login",
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0...",
      "timestamp": "2023-12-01T10:30:00Z",
      "metadata": {
        "method": "POST",
        "status_code": 200
      }
    }
  ],
  "total": 1250,
  "limit": 50,
  "offset": 0
}
```

## Social Connections

### List Connections
- `GET /api/security/connections` - List user's social connections

**Response:**
```json
{
  "connections": [
    {
      "id": "conn_123",
      "provider": "github",
      "provider_user_id": "github_user_123",
      "username": "octocat",
      "display_name": "GitHub User",
      "avatar_url": "https://avatars.githubusercontent.com/u/123",
      "created_at": "2023-11-01T10:00:00Z",
      "last_used": "2023-12-01T09:15:00Z",
      "status": "active"
    }
  ]
}
```

### Configure Connection
- `POST /api/security/connections/configure` - Configure social provider

**Request:**
```json
{
  "provider": "github",
  "client_id": "your_github_client_id",
  "client_secret": "your_github_client_secret",
  "scopes": ["repo", "user:email"]
}
```

### Connect Provider
- `POST /api/security/connections/connect` - Connect social provider

**Request:**
```json
{
  "provider": "github",
  "authorization_code": "github_auth_code",
  "redirect_uri": "http://localhost:3000/auth/github/callback"
}
```

### OAuth Callback
- `POST /api/security/connections/oauth/callback` - Handle OAuth callback

**Request:**
```json
{
  "provider": "github",
  "code": "github_auth_code",
  "state": "csrf_token"
}
```

### Disconnect Connection
- `DELETE /api/security/connections/{id}` - Disconnect social connection

### List Provider Files
- `GET /api/security/connections/{provider}/files` - List files from connected provider

**Response:**
```json
{
  "files": [
    {
      "id": "file_123",
      "name": "README.md",
      "path": "/README.md",
      "size": 1024,
      "type": "file",
      "download_url": "https://github.com/owner/repo/raw/main/README.md",
      "last_modified": "2023-12-01T08:00:00Z"
    }
  ]
}
```

---

# 6. Webhook Service (Port 3015)

Handles external webhooks from various services like GitHub, GitLab, and Stripe.

## Base URL
```
http://localhost:3015
```

## Health Check
- `GET /health` - Service health check

## Webhook Endpoints

### GitHub Webhooks
- `POST /api/webhooks/github` - Handle GitHub webhook events

**Supported Events:**
- `push` - Code push events
- `pull_request` - Pull request events
- `issues` - Issue events
- `release` - Release events

**Request Headers:**
```
X-GitHub-Event: push
X-Hub-Signature-256: sha256=...
User-Agent: GitHub-Hookshot/...
Content-Type: application/json
```

**Payload Example (Push Event):**
```json
{
  "ref": "refs/heads/main",
  "repository": {
    "id": 123456789,
    "name": "repository",
    "full_name": "owner/repository"
  },
  "pusher": {
    "name": "username",
    "email": "user@example.com"
  },
  "commits": [
    {
      "id": "commit_hash",
      "message": "Commit message",
      "timestamp": "2023-12-01T10:30:00Z",
      "added": ["file1.txt"],
      "modified": ["file2.txt"],
      "removed": ["file3.txt"]
    }
  ]
}
```

### GitLab Webhooks
- `POST /api/webhooks/gitlab` - Handle GitLab webhook events

**Supported Events:**
- `push` - Code push events
- `merge_request` - Merge request events
- `issue` - Issue events

### Stripe Webhooks
- `POST /api/webhooks/stripe` - Handle Stripe webhook events

**Supported Events:**
- `payment_intent.succeeded`
- `payment_intent.payment_failed`
- `invoice.payment_succeeded`
- `invoice.payment_failed`
- `customer.subscription.created`
- `customer.subscription.deleted`
- `customer.subscription.updated`

**Request Headers:**
```
stripe-signature: t=timestamp,v1=signature
Content-Type: application/json
```

---

# 7. Vector RAG Service (Port 8082)

Provides fusion embedding generation, vector search, and document reranking capabilities.

## Base URL
```
http://localhost:8082
```

## Health Check
- `GET /health` - Service health check

**Response:**
```json
{
  "status": "healthy",
  "service": "vector-rag",
  "version": "1.0.0",
  "embedding_available": true,
  "models_loaded": ["text-embedding-ada-002", "all-MiniLM-L6-v2"]
}
```

## Embedding Generation

### Single Text Embedding
- `POST /embed` - Generate embeddings for text

**Request:**
```json
{
  "text": "This is a sample text for embedding generation.",
  "normalize": true,
  "model": "text-embedding-ada-002"
}
```

**Response:**
```json
{
  "embeddings": [[0.1234, -0.5678, 0.9012, ...]],
  "dimension": 1536,
  "model": "text-embedding-ada-002",
  "count": 1
}
```

### Batch Text Embedding
- `POST /batch/embed` - Generate embeddings for multiple texts

**Request:**
```json
{
  "texts": [
    "First document text",
    "Second document text",
    "Third document text"
  ],
  "normalize": true,
  "model": "text-embedding-ada-002"
}
```

**Response:**
```json
{
  "embeddings": [
    [0.1234, -0.5678, 0.9012, ...],
    [0.2345, -0.6789, 0.0123, ...],
    [0.3456, -0.7890, 0.1234, ...]
  ],
  "dimension": 1536,
  "model": "text-embedding-ada-002",
  "count": 3
}
```

### Batch Chunk Embedding
- `POST /batch/embed/chunks` - Generate embeddings for document chunks

**Request:**
```json
{
  "chunks": [
    {
      "id": "chunk_001",
      "text": "This is chunk 1 content",
      "metadata": {
        "source": "document1.pdf",
        "page": 1
      }
    },
    {
      "id": "chunk_002", 
      "text": "This is chunk 2 content",
      "metadata": {
        "source": "document1.pdf",
        "page": 2
      }
    }
  ],
  "normalize": true,
  "model": "text-embedding-ada-002"
}
```

**Response:**
```json
{
  "results": [
    {
      "chunk_id": "chunk_001",
      "embedding": [0.1234, -0.5678, 0.9012, ...],
      "success": true
    },
    {
      "chunk_id": "chunk_002",
      "embedding": [0.2345, -0.6789, 0.0123, ...],
      "success": true
    }
  ],
  "dimension": 1536,
  "model": "text-embedding-ada-002"
}
```

## Document Reranking

### Rerank Documents
- `POST /rerank` - Rerank documents based on query relevance

**Request:**
```json
{
  "query": "machine learning algorithms",
  "documents": [
    {
      "id": "doc_001",
      "text": "This document discusses various machine learning algorithms including neural networks and decision trees.",
      "metadata": {
        "source": "ml_paper.pdf",
        "relevance_score": 0.8
      }
    },
    {
      "id": "doc_002",
      "text": "This paper focuses on deep learning techniques for computer vision applications.",
      "metadata": {
        "source": "cv_paper.pdf",
        "relevance_score": 0.6
      }
    }
  ],
  "top_k": 5,
  "model": "rerank-model-v1"
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "doc_001",
      "score": 0.95,
      "index": 0,
      "document": {
        "id": "doc_001",
        "text": "This document discusses various machine learning algorithms including neural networks and decision trees.",
        "metadata": {
          "source": "ml_paper.pdf",
          "relevance_score": 0.8
        }
      }
    },
    {
      "id": "doc_002",
      "score": 0.82,
      "index": 1,
      "document": {
        "id": "doc_002",
        "text": "This paper focuses on deep learning techniques for computer vision applications.",
        "metadata": {
          "source": "cv_paper.pdf",
          "relevance_score": 0.6
        }
      }
    }
  ]
}
```

## Vector Search

### Vector Similarity Search
- `POST /vector/search` - Search for similar vectors

**Request:**
```json
{
  "query_vector": [0.1234, -0.5678, 0.9012, ...],
  "top_k": 10,
  "threshold": 0.7,
  "filter": {
    "source": "github",
    "language": "rust"
  }
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "vector_001",
      "score": 0.95,
      "payload": {
        "text": "Rust function for data processing",
        "source": "github",
        "file_path": "src/main.rs",
        "language": "rust"
      }
    },
    {
      "id": "vector_002",
      "score": 0.87,
      "payload": {
        "text": "Another Rust implementation",
        "source": "github",
        "file_path": "src/utils.rs",
        "language": "rust"
      }
    }
  ],
  "total_found": 25
}
```

### Search by Vector IDs
- `POST /vector/search_by_ids` - Search for specific vectors by IDs

**Request:**
```json
{
  "ids": ["vector_001", "vector_002", "vector_003"],
  "include_payload": true
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "vector_001",
      "vector": [0.1234, -0.5678, 0.9012, ...],
      "payload": {
        "text": "Sample document",
        "source": "github"
      }
    },
    {
      "id": "vector_002",
      "vector": [0.2345, -0.6789, 0.0123, ...],
      "payload": {
        "text": "Another document",
        "source": "gitlab"
      }
    }
  ]
}
```

### Search by Entity
- `GET /vector/search_by_entity/{entity_id}` - Search vectors by entity ID

**Response:**
```json
{
  "entity_id": "entity_123",
  "results": [
    {
      "id": "vector_001",
      "score": 0.95,
      "payload": {
        "text": "Entity-related document",
        "entity_id": "entity_123"
      }
    }
  ]
}
```

## Fusion Embedding

The service supports fusion embedding strategies that combine multiple models:

### Available Strategies
- **concatenation**: Concatenates embeddings from multiple models
- **weighted_sum**: Combines embeddings with learned weights
- **attention**: Uses attention mechanism to combine embeddings

### Fusion Configuration
Fusion configuration is loaded from `config/fusion_config.json`:

```json
{
  "models": [
    {
      "name": "text-embedding-ada-002",
      "provider": "openai",
      "weight": 0.6,
      "api_key": "your_openai_key"
    },
    {
      "name": "all-MiniLM-L6-v2",
      "provider": "huggingface",
      "weight": 0.4,
      "model_path": "/path/to/model"
    }
  ],
  "strategy": "weighted_sum",
  "dimension": 1536
}
```

---

# 8. Indexing Service (Port 8080)

Provides background indexing services for documents and code repositories.

## Base URL
```
http://localhost:8080
```

## Service Overview

The indexing service runs background jobs to:
- Index documents for search
- Process code repositories
- Update search indices
- Maintain robot memory

## Health Check
- `GET /health` - Service health check

**Response:**
```json
{
  "status": "healthy",
  "service": "indexer-service",
  "version": "1.0.0",
  "active_jobs": 3,
  "completed_jobs": 1250,
  "failed_jobs": 2
}
```

## Robot Memory Indexing

The service includes a specialized robot memory indexer that:
- Processes AI conversations and interactions
- Builds knowledge graphs from interactions
- Maintains context for AI agents
- Enables semantic search over conversation history

## Configuration

Indexing service is configured via environment variables:

```bash
# Indexing configuration
INDEXING_CONCURRENT_JOBS=5
INDEXING_BATCH_SIZE=100
INDEXING_RETRY_ATTEMPTS=3

# Database connections
DATABASE_URL=postgresql://...
QDRANT_URL=http://localhost:6333

# Service dependencies
EMBEDDING_SERVICE_URL=http://localhost:8082
AUTH_SERVICE_URL=http://localhost:3010
```

## Job Types

### Document Indexing Jobs
- Process uploaded documents
- Extract text and metadata
- Generate embeddings
- Store in vector database

### Repository Indexing Jobs
- Clone and analyze repositories
- Index source code files
- Extract code structure
- Build cross-reference maps

### Content Processing Jobs
- Chunk large documents
- Extract entities and concepts
- Build knowledge graphs
- Update search indices

---

# Frontend Integration

The ConHub frontend (Next.js) integrates with all backend services through the unified API gateway at port 8000.

## Client-side Configuration

```javascript
// API Configuration
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000';

// Authentication
export const auth = {
  login: (credentials) => fetch(`${API_BASE_URL}/api/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(credentials)
  }),
  
  getCurrentUser: () => fetch(`${API_BASE_URL}/api/auth/me`, {
    headers: { 'Authorization': `Bearer ${getToken()}` }
  })
};

// Data Sources
export const dataSources = {
  connect: (source) => fetch(`${API_BASE_URL}/api/data/sources`, {
    method: 'POST',
    headers: { 
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${getToken()}`
    },
    body: JSON.stringify(source)
  }),
  
  list: () => fetch(`${API_BASE_URL}/api/data-sources`, {
    headers: { 'Authorization': `Bearer ${getToken()}` }
  })
};

// GraphQL
export const graphql = {
  query: (query, variables) => fetch(`${API_BASE_URL}/api/graphql`, {
    method: 'POST',
    headers: { 
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${getToken()}`
    },
    body: JSON.stringify({ query, variables })
  })
};
```

## WebSocket Connections

For real-time updates, the frontend can establish WebSocket connections:

```javascript
// Example: Real-time sync status updates
const ws = new WebSocket('ws://localhost:8000/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.type === 'sync_status') {
    updateSyncStatus(data.payload);
  }
};
```

---

# Error Handling

## Standard Error Response Format

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

## Common Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| VALIDATION_ERROR | Invalid input parameters | 400 |
| AUTHENTICATION_REQUIRED | Missing or invalid JWT | 401 |
| AUTHORIZATION_FAILED | Insufficient permissions | 403 |
| NOT_FOUND | Resource not found | 404 |
| CONFLICT | Resource conflict | 409 |
| RATE_LIMITED | Too many requests | 429 |
| INTERNAL_ERROR | Server error | 500 |
| SERVICE_UNAVAILABLE | Service temporarily unavailable | 503 |

## Rate Limiting

Most endpoints implement rate limiting:

- Standard users: 100 requests/minute
- Premium users: 1000 requests/minute
- Enterprise users: 5000 requests/minute

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1701421800
```

---

# Development Tools

## API Testing

### cURL Examples

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

# Connect GitHub repository
curl -X POST http://localhost:8000/api/data/sources \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "type": "github",
    "url": "https://github.com/owner/repo",
    "config": {
      "name": "My Repository",
      "defaultBranch": "main"
    }
  }'
```

### Postman Collection

Import the provided Postman collection (`ConHub_API.postman_collection.json`) for easy API testing.

## Monitoring

### Health Monitoring

All services expose health endpoints:

```bash
# Check all services
for service in auth billing data security webhook; do
  curl -s http://localhost:301${service:0:1}/health | jq .
done
```

### Log Aggregation

Services use structured logging with correlation IDs:

```bash
# View logs with trace correlation
docker-compose logs -f | grep "trace_id=abc123"
```

### Metrics

Services expose metrics in Prometheus format:

```
# Service metrics
curl http://localhost:8000/metrics

# Individual service metrics
curl http://localhost:3010/metrics  # Auth service
curl http://localhost:3013/metrics  # Data service
```

---

# Deployment

## Docker Compose

```yaml
version: '3.8'
services:
  # Frontend
  frontend:
    build: ./frontend
    ports: ["3000:3000"]
    depends_on: [backend]
  
  # Backend services
  backend:
    build: ./backend
    ports: ["8000:8000"]
    depends_on: [auth, data, billing, security, webhook]
  
  auth:
    build: ./auth
    ports: ["3010:3010"]
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - JWT_SECRET=${JWT_SECRET}
  
  data:
    build: ./data
    ports: ["3013:3013"]
    depends_on: [auth]
  
  # ... other services
```

## Environment Variables

Key environment variables for production:

```bash
# Database
DATABASE_URL_NEON=postgresql://user:pass@ep-xxx.region.neon.tech/db
REDIS_URL=redis://localhost:6379
QDRANT_URL=http://localhost:6333

# Authentication
JWT_PRIVATE_KEY_PATH=/app/keys/private_key.pem
JWT_PUBLIC_KEY_PATH=/app/keys/public_key.pem

# External Services
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
STRIPE_SECRET_KEY=sk_test_...

# Feature Flags
FEATURE_AUTH=true
FEATURE_REDIS=true
FEATURE_HEAVY=true
```

## SSL/TLS Configuration

For production deployment with HTTPS:

```nginx
server {
    listen 443 ssl;
    server_name api.conhub.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://localhost:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

# Contributing to API Documentation

## Updating Documentation

1. Update this file (`CONHUB_API_DOCUMENTATION.md`) with new endpoints
2. Update individual service documentation files
3. Regenerate OpenAPI specifications:
   ```bash
   npm run generate-api-docs
   ```
4. Update Postman collection:
   ```bash
   npm run generate-postman-collection
   ```

## API Versioning

The platform uses semantic versioning for APIs:
- Major version changes indicate breaking changes
- Minor version changes add new features
- Patch version changes include bug fixes

Current API version: **v1.0.0**

## Support

For API questions or issues:
- Check the troubleshooting guide: `docs/TROUBLESHOOTING.md`
- Review the development guide: `docs/DEVELOPMENT.md`
- Open an issue on the GitHub repository
