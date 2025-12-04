# ConHub Observability Architecture

This document describes the comprehensive logging, tracing, and metrics infrastructure for ConHub.

## Overview

ConHub uses a unified observability stack across all microservices and the frontend:

- **Rust Services**: `tracing` ecosystem with structured JSON logging
- **Frontend**: Custom logger with performance tracking and user action logging
- **Correlation**: Trace IDs propagate from frontend through all backend services

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Frontend (Next.js)                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  ConHubLogger                                                        │   │
│  │  - Page views, route changes                                         │   │
│  │  - Button clicks, form submissions                                   │   │
│  │  - API call tracking with timing                                     │   │
│  │  - Error capture (JS errors, unhandled rejections)                  │   │
│  │  - Performance metrics (LCP, FID, page load)                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                         x-trace-id │ x-span-id                              │
│                                    │ x-request-id                           │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Rust Microservices                                   │
│                                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   backend    │  │     auth     │  │     data     │  │     mcp      │    │
│  │   :3000      │  │    :3010     │  │    :3013     │  │    :3004     │    │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   chunker    │  │  vector_rag  │  │  graph_rag   │  │decision_engn │    │
│  │   :3017      │  │    :8082     │  │    :8006     │  │    :3016     │    │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   billing    │  │   security   │  │   webhook    │  │   indexers   │    │
│  │   :3011      │  │    :3014     │  │    :3015     │  │   (worker)   │    │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                                              │
│  Each service uses:                                                          │
│  - conhub-observability crate                                               │
│  - ObservabilityMiddleware (HTTP request/response logging)                  │
│  - Domain event logging (sync jobs, chunking, search, etc.)                 │
│  - Trace ID extraction and propagation                                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Shared Infrastructure

### Rust: `shared/observability/`

The `conhub-observability` crate provides:

#### 1. Tracing Initialization

```rust
use conhub_observability::{init_tracing, TracingConfig};

// In main.rs
init_tracing(TracingConfig::for_service("data-service"));

// Or with custom config
init_tracing(
    TracingConfig::for_service("data-service")
        .json()
        .with_level("debug")
);
```

#### 2. HTTP Middleware

```rust
use conhub_observability::observability;

HttpServer::new(move || {
    App::new()
        .wrap(observability("data-service"))
        // ... routes
})
```

The middleware automatically:
- Extracts/generates trace IDs from headers
- Logs request start with method, path, headers
- Logs response with status, duration
- Warns on slow requests (>1000ms default)
- Errors on 5xx responses

#### 3. Domain Event Logging

```rust
use conhub_observability::domain_events::{
    DomainEvent, EventCategory,
    log_sync_job_started, log_sync_job_completed,
    log_connector_operation, log_search_executed,
};

// Convenience functions
log_sync_job_started("data-service", job_id, "github", Some(&trace_id));
log_sync_job_completed("data-service", job_id, docs_processed, duration_ms);

// Or builder pattern for custom events
DomainEvent::new("data-service", EventCategory::Ingestion, "document_processed")
    .entity("document", doc_id)
    .duration_ms(150)
    .metadata(serde_json::json!({ "size_bytes": 1024 }))
    .success()
    .emit();
```

#### 4. Logging Macros

```rust
use conhub_observability::{log_fn_entry, log_fn_exit, log_db, log_retry, log_security};

log_fn_entry!("process_document", doc_id = doc_id, size = size);
log_db!("INSERT", "documents", doc_id);
log_retry!("fetch_github", 2, 3, error);
log_security!("access_denied", user_id = user_id, resource = "billing");
```

### Frontend: `frontend/lib/logger.ts`

The frontend logger provides:

#### 1. Basic Logging

```typescript
import logger from '@/lib/logger';

logger.debug('Component mounted', { component: 'Dashboard' }, 'component');
logger.info('User action', { action: 'clicked_save' }, 'ui');
logger.warn('Slow operation', { duration: 2000 }, 'performance');
logger.error('API failed', { endpoint: '/api/data' }, 'api');
```

#### 2. User Context

```typescript
logger.setUserId('user-123');
logger.setTenantId('tenant-456');
logger.clearUser(); // On logout
```

#### 3. Trace ID Propagation

```typescript
// Get current trace ID
const traceId = logger.getTraceId();

// Get headers for API calls (auto-included in ApiClient)
const headers = logger.getTraceHeaders();
// Returns: { 'x-trace-id': '...', 'x-span-id': '...', 'x-request-id': '...' }
```

#### 4. Specialized Tracking

```typescript
// Page views (auto-tracked)
logger.trackPageView('/dashboard', { referrer: '/home' });

// Button clicks
logger.trackButtonClick('save-settings', 'Save Settings', { form: 'profile' });

// Form submissions
logger.trackFormSubmit('login-form', 'Login Form', true, { method: 'oauth' });

// Connector actions
logger.trackConnectorAction('github', 'connect', { repos: ['repo1', 'repo2'] });

// Search
logger.trackSearch('authentication code', 15, 250, { filters: ['code'] });

// Feature usage
logger.trackFeatureUsage('memory-search', 'query', { resultsCount: 10 });
```

### React Hooks: `frontend/hooks/useObservability.ts`

```typescript
import {
  useLogger,
  usePageTracking,
  useComponentTracking,
  useAsyncTracking,
  useConnectorTracking,
  useTracedFetch,
} from '@/hooks/useObservability';

// Auto-track page views
function App() {
  usePageTracking();
  return <Routes />;
}

// Track component lifecycle
function Dashboard() {
  useComponentTracking('Dashboard');
  // ...
}

// Track async operations with timing
function DataLoader() {
  const trackAsync = useAsyncTracking();
  
  const loadData = async () => {
    const data = await trackAsync('load_repositories', async () => {
      return await api.getRepositories();
    }, { source: 'github' });
  };
}

// Track connector actions
function ConnectorPanel() {
  const trackConnector = useConnectorTracking();
  
  const onConnect = () => {
    trackConnector('github', 'connect', { scope: 'repo' });
  };
}
```

## Log Schema

### Common Fields (All Logs)

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | ISO 8601 | When the event occurred |
| `level` | enum | `debug`, `info`, `warn`, `error` |
| `service` | string | Service name (e.g., `data-service`, `frontend`) |
| `trace_id` | string | Trace ID for correlation across services |
| `span_id` | string | Span ID within the trace |
| `request_id` | string | Unique request identifier |

### HTTP Events

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | HTTP method |
| `path` | string | Request path |
| `status_code` | number | HTTP status code |
| `duration_ms` | number | Request duration |
| `user_id` | string? | Authenticated user ID |
| `tenant_id` | string? | Tenant ID |

### Domain Events

| Field | Type | Description |
|-------|------|-------------|
| `category` | enum | `sync`, `chunking`, `embedding`, `search`, `graph`, `billing`, `auth`, `robot`, `connector`, `mcp` |
| `event_type` | string | Specific event (e.g., `job_started`, `document_chunked`) |
| `entity_type` | string? | Entity type being operated on |
| `entity_id` | string? | Entity ID |
| `result` | enum | `success`, `failure`, `partial`, `skipped` |
| `duration_ms` | number? | Operation duration |
| `error` | string? | Error message if failed |
| `metadata` | object? | Additional structured data |

### Frontend UI Events

| Field | Type | Description |
|-------|------|-------------|
| `source` | string | `navigation`, `ui`, `api`, `component`, `feature`, `connector`, `search` |
| `session_id` | string | Browser session ID |
| `url` | string | Current page URL |
| `user_agent` | string | Browser user agent |

## Trace Context Propagation

### Headers

Traces are propagated using HTTP headers:

| Header | Description |
|--------|-------------|
| `x-trace-id` | Primary trace ID (spans entire user flow) |
| `x-span-id` | Current span ID |
| `x-parent-span-id` | Parent span ID |
| `x-request-id` | Unique request ID |
| `traceparent` | W3C Trace Context format (for compatibility) |

### Flow

1. **Frontend**: Generates `trace_id` on page load or new user flow
2. **API Client**: Automatically includes trace headers in all requests
3. **Backend Services**: `ObservabilityMiddleware` extracts trace context
4. **Inter-Service Calls**: Services propagate trace context to downstream calls
5. **Logs**: All logs include trace context for correlation

## Configuration

### Environment Variables

```bash
# Log level (default: info)
RUST_LOG=info
# Or granular: RUST_LOG=conhub=debug,sqlx=warn

# Log format: "json" or "pretty" (default: pretty in dev, json in prod)
LOG_FORMAT=json

# Environment name
ENVIRONMENT=dev  # dev, staging, prod

# Log span enter/exit events (default: false)
LOG_SPANS=true

# Include file/line in logs (default: true)
LOG_LOCATION=true

# Frontend log level
NEXT_PUBLIC_LOG_LEVEL=info  # debug, info, warn, error
```

### Per-Service Configuration

Each service initializes with:

```rust
init_tracing(TracingConfig::for_service("service-name"));
```

To customize:

```rust
init_tracing(
    TracingConfig::for_service("service-name")
        .with_level("debug")
        .json()
        .with_spans()
        .with_environment("staging")
);
```

## Usage Examples

### Debugging a Failed Sync

1. User reports sync failed for GitHub repo "myorg/myrepo"
2. Find the trace ID in frontend logs or error message
3. Search backend logs for that trace ID:
   ```
   grep "trace_id=<trace-id>" /var/log/data-service.log
   ```
4. Follow the flow:
   - `data-service`: `sync.job_started`, connector operations, errors
   - `chunker`: `chunking.document_chunked` events
   - `vector_rag`: embedding events
   - `graph_rag`: graph indexing events

### Performance Investigation

1. Check slow request warnings in logs:
   ```
   grep "SLOW" /var/log/*/service.log
   ```
2. Look at domain event durations:
   ```json
   {"event_type": "query_executed", "duration_ms": 2500, "strategy": "hybrid"}
   ```
3. Frontend performance metrics in browser console or `/api/logs` endpoint

### Security Audit

1. Filter for security events:
   ```
   grep "target.*security" /var/log/*/service.log
   ```
2. Auth events include user context:
   ```json
   {"category": "auth", "event_type": "login_failed", "user_id": "...", "error": "invalid_password"}
   ```

## Best Practices

### What to Log

**DO log:**
- User-visible operations (sync start/end, search queries)
- Errors with context (what failed, why, what was attempted)
- Performance-relevant operations (durations, counts)
- Security events (login, access denied, permission changes)

**DON'T log:**
- Secrets, tokens, passwords (auto-redacted in middleware)
- Raw document content (log IDs and sizes instead)
- High-frequency internal loops (use sampling)
- PII without explicit consent

### Log Levels

| Level | When to Use |
|-------|-------------|
| `error` | Failures visible to users, requires investigation |
| `warn` | Recoverable issues, degraded functionality |
| `info` | Normal operations, user actions, state changes |
| `debug` | Detailed internal state, development troubleshooting |
| `trace` | Very verbose, per-item iteration (rarely used in prod) |

### Structured Logging

Always use structured fields instead of string interpolation:

```rust
// Good
info!(trace_id = %trace_id, user_id = %user_id, action = "sync_started", "Starting sync");

// Avoid
info!("Starting sync for user {} with trace {}", user_id, trace_id);
```

## Files Reference

### Backend

| File | Purpose |
|------|---------|
| `shared/observability/Cargo.toml` | Crate dependencies |
| `shared/observability/src/lib.rs` | Main exports |
| `shared/observability/src/init.rs` | Tracing initialization |
| `shared/observability/src/middleware.rs` | HTTP middleware |
| `shared/observability/src/trace_context.rs` | Trace ID propagation |
| `shared/observability/src/domain_events.rs` | Domain event logging |
| `shared/observability/src/macros.rs` | Convenience macros |

### Frontend

| File | Purpose |
|------|---------|
| `frontend/lib/logger.ts` | Main logger class |
| `frontend/hooks/useObservability.ts` | React hooks |
| `frontend/lib/api.ts` | API client with trace headers |

## Future Improvements

1. **OpenTelemetry Export**: Add OTLP exporter for Jaeger/Tempo
2. **Log Aggregation**: Configure sinks for Loki/Elasticsearch
3. **Dashboards**: Grafana panels for key metrics
4. **Alerting**: Prometheus alerts for error rates and latency
5. **Sampling**: Implement head-based sampling for high-traffic prod
