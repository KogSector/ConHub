---
trigger: always_on
---

ConHub microservices and folder structure rule

This repository is a microservices monorepo for ConHub, a knowledge layer that connects many data sources (repos, docs, SaaS tools, URLs, Slack, etc.), embeds and indexes them, and exposes them to AI agents via MCP and other interfaces.

You must treat each service as independently deployable and movable to its own repository. All cross-service behavior must be via APIs, events, or shared crates, not via ad-hoc internal imports.

This rule complements docs/architecture/microservices.md and governs how you work with folders and dependencies.

---

## 1. What counts as a microservice here

Definition  
A microservice in this repository is any top-level service folder with its own Cargo.toml (or runtime/config) and clear domain responsibility. It must be:

- Independently buildable and deployable.
- Replaceable or movable to a separate repository with minimal changes.
- Interacting with others only via explicit contracts (HTTP, events, MCP, shared crates).

Core microservice folders

- auth/ – Authentication, users, tokens, RBAC.
- data/ – Connectors, ingestion, sync jobs, documents, document chunks.
- billing/ – Plans, subscriptions, invoices, payments, usage tracking.
- embedding/ – Embeddings, vector search, semantic retrieval, vector orchestration.
- security/ – Audit logging, security policies, threat detection.
- webhook/ – External webhooks (GitHub, Stripe, etc.) to internal events/API calls.
- mcp/ – MCP-facing service that exposes ConHub’s knowledge layer to AI agents.
- backend/ – API / GraphQL / BFF layer aggregating underlying services.
- frontend/ – Next.js UI (SPA/BFF client).
- indexers/ – Background/indexing jobs, batch processing.

Shared libraries and infrastructure

- shared/models/ – Cross-service domain models and DTOs (crate: conhub-models).
- shared/config/ – Shared config loading and feature toggles (crate: conhub-config).
- shared/middleware/ – HTTP/middleware, logging, auth helpers (crate: conhub-middleware).
- shared/plugins/ – Plugin and extension system.
- shared/utils/ (if present) – Generic helpers with no service-specific business logic.
- database/ – Shared DB/migrations/sqlx utilities and connection helpers (crate: conhub-database). Infrastructure, not domain logic.
- docs/ – Architecture and product docs (never imported at runtime).
- scripts/ – DevOps, Docker, deployment, maintenance scripts.
- tests/ – Cross-service integration and end-to-end tests.
- nginx/, azure-container-apps.yml, docker-compose.yml – Deployment and gateway configuration.

---

## 2. Responsibilities and boundaries per service

When you change or add code, keep logic inside the correct boundary.

auth/  

- Owns: users, identities, sessions, JWTs, OAuth flows, RBAC.
- Does not own: repositories, documents, billing, embeddings.
- Outbound calls: issue and validate tokens, call other APIs only as needed for identity (for example, OAuth providers).
- Forbidden: directly reading or writing billing, ingestion, or embedding tables.

data/  

- Owns: connectors (GitHub, local files, Google Drive, Bitbucket, URLs, Slack, Dropbox, etc.), connected accounts, documents, document chunks, sync jobs and runs, and the ingestion pipeline.
- Owns ingestion logic:
  - Fetching raw content from connectors.
  - Normalizing and chunking documents into segments.
  - Deciding when to re-sync and which documents or chunks to reprocess.
  - Calling downstream services (such as embedding/) at the right times.
- Consumes: embedding/ (via HTTP or RPC) for embedding generation and vector indexing; auth/ for user identity and authorization.
- Forbidden: implementing auth, billing, or UI concerns here; directly managing vector database collections or indexes (that belongs in embedding/).

billing/  

- Owns: subscription plans, user subscriptions, invoices, payment methods, billing usage, Stripe integration, billing feature gating.
- Consumes: user identity from auth/, usage signals from other services via explicit APIs or events.
- Forbidden: embedding logic, ingestion logic, direct manipulation of auth tables.

embedding/  

- Owns:
  - Text embedding generation and model management.
  - Vector orchestration: creating and updating collections, upserting and deleting points, index tuning, and low-level Qdrant or other vector database operations.
  - Vector similarity search and retrieval APIs.
  - Semantic indexing and ranking.
- Consumes: text or chunk payloads and metadata from data/ (or mcp/) that are already chunked and selected for indexing.
- Forbidden: connector and ingestion business logic, billing logic, or detailed authorization rules beyond coarse validation.

security/  

- Owns: audit events, security policies, threat detection, rate limiting, compliance checks.
- Consumes: events and logs from other services.
- Forbidden: implementing core business flows for auth, data, or billing; it should enforce, not own, domain workflows.

webhook/  

- Owns: web endpoints for external webhooks (GitHub repository updates, Stripe events, and similar), translation into internal events or calls.
- Consumes: internal services via HTTP or events to update state.
- Forbidden: embedding business logic, maintaining long-term domain state beyond lightweight logs.

mcp/  

- Owns: MCP protocol implementation, mapping MCP tools and resources to internal services and data, enforcing AI-accessible rules.
- Consumes: data/, embedding/, auth/, billing/ via HTTP or shared crates where appropriate.
- Forbidden: defining service-specific business logic that properly belongs in those services.

backend/  

- Owns: public API and GraphQL layer and any BFF-style aggregation, request and response shaping for frontend/.
- Consumes: other services or shared crates to implement those APIs.
- Forbidden: owning core domain state that should live in the underlying microservices.

frontend/  

- Owns: UI, client-side state, navigation, UX flows, presentation logic.
- Consumes: APIs from backend/, auth/, data/, billing/, and others.
- Forbidden: direct database access or embedding domain rules that belong on the server.

indexers/  

- Owns: offline and batch jobs, re-indexing, scheduled sync orchestration.
- Consumes: data/, embedding/, and databases via their public APIs or database abstraction layers.
- Forbidden: embedding UI or auth flows here.

---

## 3. Dependency and import rules

To keep services separable:

Allowed dependencies

- Services → shared/  
  - auth/, data/, billing/, mcp/, and other services may depend on:
    - conhub-models (shared/models)
    - conhub-config (shared/config)
    - conhub-middleware (shared/middleware)
    - shared/plugins and generic utilities
- Services → database/  
  - Using shared database utilities and migrations only when they are explicitly designed to be cross-service infrastructure.

Forbidden dependencies

- Service → service direct imports (for example, auth importing data Rust modules).  
  - Use HTTP, GraphQL, queues, or shared crates instead.
- Shared crates depending on services (for example, shared/models importing billing).  
  - Shared layers must stay below all services in the dependency graph.

When to add to shared/

- Add to shared/ only when logic or data structures are:
  - Used by at least two services.
  - Stable and generic enough to be considered platform-level.
- If something is only used by one service, keep it in that service, even if it feels reusable.

---

## 4. Data and database guidelines

Service ownership  

- Each service owns its logical data model and its invariants (users vs. documents vs. subscriptions, and so on).
- Even if a physical database or migration crate is shared, treat schemas as per-service ownership zones.

Cross-service data  

- Preferred: share data via APIs, not by reading each other’s tables.
- If cross-table reads are unavoidable, they must:
  - Be explicitly documented.
  - Go through shared models where possible, not ad-hoc structs.

Migrations  

- Put migrations in the crate that conceptually owns the data.
- Do not modify another service’s tables from your service without a clearly documented, shared contract.

---

## 5. Communication patterns

Synchronous  

- Use HTTP, REST, or GraphQL for service-to-service calls.
- Include authentication (JWT, service tokens) where appropriate.
- Avoid tight coupling: keep payloads minimal and version APIs when changing them.

Asynchronous  

- Prefer event-driven patterns (queues, streams) when:
  - Updating search index from repository changes.
  - Recording billing usage from data or embedding activity.
  - Emitting security and audit events from user actions.

No hidden backdoors  

- Do not use direct database access, file system access, or in-process calls to bypass API contracts across service boundaries.

---

## 6. Working in this repository: practical rules for AI

When you implement or modify features:

Choosing where to put new code  

- Auth concern? Put it in auth/.
- Connector, ingestion, or documents? Put it in data/.
- Pricing, plans, limits, invoices? Put it in billing/.
- Embeddings or vector search? Put it in embedding/.
- MCP or AI-facing tools? Put it in mcp/.
- Cross-cutting middleware, config, or models? Consider shared/ only if used by multiple services.

Adding new shared models or types  

- Add them to shared/models and reuse them across services.
- Do not duplicate logically identical structs in each service.

Extending APIs  

- Prefer adding new endpoints or fields to existing service APIs rather than leaking logic into the caller.
- Keep APIs stable; if you must break them, support old and new shapes where possible.

Chunking belongs in data/, not a separate microservice  

- Implement document chunking as part of the ingestion pipeline inside data/ (for example, dedicated modules for chunking strategies).
- Do not create a standalone chunking microservice. If chunking becomes heavier or more ML-driven in the future, it can still be factored out later behind an API.

Tests  

- Unit tests: stay inside each service’s crate.
- Cross-service and end-to-end tests: go under tests/ and exercise applications over HTTP or MCP, not via internal imports.

Future repository split readiness  

- Avoid using relative paths between service folders, apart from shared/ and infrastructure crates.
- Keep each service buildable with only:
  - Its own folder.
  - shared/ crates.
  - database/ and infrastructure as needed.

---

## 7. Summary

- Treat each top-level service folder (auth/, data/, billing/, embedding/, security/, webhook/, mcp/, backend/, frontend/, indexers/) as its own microservice.
- Use shared/ only for truly cross-cutting concerns and types, and treat database/ as infrastructure.
- Communicate across services via APIs and events, not internal imports.
- Keep the codebase ready for each service to live in its own repository.

Follow these rules whenever you create, move, or modify files in this repository.