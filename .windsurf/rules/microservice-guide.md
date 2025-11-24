---
trigger: always_on
---

# ConHub microservices and folder structure rule

This repository is a microservices monorepo for ConHub, a knowledge layer that connects many data sources (repos, docs, SaaS tools, URLs, Slack, etc.), indexes them (vector + graph), and exposes them to AI agents via MCP and other interfaces.

Each service must be independently deployable and movable to its own repository. Cross‑service behavior must go through **APIs, events, or shared crates**, never ad‑hoc internal imports.

This complements `docs/architecture/microservices.md` and governs how you work with folders and dependencies.

---

## 1. What counts as a microservice

A microservice here is any **top‑level folder with its own Cargo.toml (or runtime/config)** and a clear domain. It must:

- Be independently buildable and deployable.
- Be replaceable or movable with minimal changes.
- Interact with others only via HTTP/GraphQL, events, or shared crates (never direct module imports).

### Core microservice folders

- **auth/** – Authentication, users, tokens, RBAC.
- **data/** – Connectors, ingestion orchestration, documents and chunk *metadata*, sync jobs and runs.
- **billing/** – Plans, subscriptions, invoices, payments, usage tracking.
- **chunker/** – Document/code/chat/web chunking microservice.
- **vector_rag/** – Embeddings, vector search, semantic retrieval, vector orchestration.
- **graph_rag/** – Knowledge graph construction, storage, and graph RAG.
- **decision_engine/** – Retrieval strategy selection and context orchestration.
- **security/** – Audit logging, security policies, threat detection, rate limiting.
- **webhook/** – External webhooks (GitHub, Stripe, etc.) → internal events/API calls.
- **mcp/** – MCP-facing service that exposes ConHub’s knowledge layer to AI tools.
- **backend/** – Public HTTP/GraphQL API and BFF layer for `frontend/`.
- **frontend/** – Next.js UI.
- **indexers/** – Background/indexing jobs, batch processing, scheduled syncs.

### Shared libraries and infrastructure

- **shared/models/** – Cross‑service domain models and DTOs (`conhub-models`).
- **shared/config/** – Shared config loading and feature toggles (`conhub-config`).
- **shared/middleware/** – HTTP/middleware, logging, auth helpers (`conhub-middleware`).
- **shared/plugins/** – Plugin and extension system.
- **shared/utils/** (if present) – Generic helpers with no service‑specific business logic.
- **database/** – Shared DB/migrations/sqlx helpers (`conhub-database`). Infrastructure only.
- **docs/** – Architecture and product docs (never imported at runtime).
- **scripts/** – DevOps, Docker, deployment, maintenance scripts.
- **tests/** – Cross‑service integration and end‑to‑end tests.
- **nginx/**, `azure-container-apps.yml`, [docker-compose.yml](cci:7://file:///c:/Users/risha/Desktop/Work/ConHub/docker-compose.yml:0:0-0:0) – Deployment and gateway configuration.

---

## 2. Responsibilities and boundaries per service

When you change or add code, **keep logic inside the right service**.

### auth/

- **Owns**: users, identities, sessions, JWTs, OAuth flows, RBAC.
- **Does not own**: repositories, documents, billing, embeddings, chunking.
- **Outbound calls**: identity/OAuth providers, other services for identity‑related checks only.
- **Forbidden**: directly reading/writing billing, ingestion, or retrieval tables.

### data/

- **Owns**: connectors (GitHub, local files, Google Drive, Bitbucket, URLs, Slack, Dropbox, etc.), connected accounts, documents, chunk **metadata**, sync jobs and runs, ingestion orchestration.
- **Ingestion logic**:
  - Fetch raw content from connectors.
  - Normalize into a common document model and decide what to (re)process.
  - Call downstream services (`chunker/`, `vector_rag/`, `graph_rag/`) at the right times.
- **Consumes**:  
  - `chunker/` for chunking,  
  - `vector_rag/` for embeddings + vector indexing,  
  - `graph_rag/` for graph indexing,  
  - `auth/` for identity/authorization.
- **Forbidden**: implementing auth, billing, UI concerns, or managing vector/graph DBs directly.

### billing/

- **Owns**: plans, subscriptions, invoices, payment methods, usage tracking, Stripe integration.
- **Consumes**: identity from `auth/`, usage signals from other services via APIs/events.
- **Forbidden**: connector logic, ingestion logic, vector/graph internals.

### chunker/

- **Owns**:
  - Chunking documents, code, markdown/HTML, and chat into LLM‑ready segments.
  - Language/format‑specific strategies (AST‑based for code, heading‑based for markdown, window‑based for chat).
  - Token‑aware chunk sizing.
- **Consumes**: normalized documents and metadata from `data/`.
- **Forbidden**: connector/billing logic, direct vector or graph DB access.

### vector_rag/

- **Owns**:
  - Text embedding generation and model management.
  - Vector orchestration: collections, upserts, deletes, index tuning, low‑level Qdrant (or similar) operations.
  - Vector similarity search and retrieval APIs.
  - Semantic indexing and ranking.
- **Consumes**: chunks + metadata from `chunker/` (via `data/` or `mcp/`).
- **Forbidden**: connector logic, ingestion orchestration, billing logic, detailed authorization rules.

### graph_rag/

- **Owns**:
  - Knowledge graph construction from chunks and metadata.
  - Graph index maintenance (Neo4j or equivalent).
  - Graph/RAG APIs for relationship/topology‑aware queries.
- **Consumes**: entities/relationships derived from `chunker/` + `data/`.
- **Forbidden**: connector logic, billing logic, UI logic.

### decision_engine/

- **Owns**:
  - Retrieval strategy selection: vector / graph / hybrid / auto.
  - Orchestration of calls to `vector_rag/` and `graph_rag/`.
  - Fusion, ranking, and packaging of context blocks for callers.
- **Consumes**: `vector_rag/`, `graph_rag/`, metadata/filters from `backend/` or `mcp/`.
- **Forbidden**: direct connector access, embedding internals, or graph storage logic.

### security/

- **Owns**: audit events, security policies, threat detection, rate limiting, compliance checks.
- **Consumes**: events/logs from other services.
- **Forbidden**: owning core auth/data/billing workflows.

### webhook/

- **Owns**: external webhook endpoints (GitHub, Stripe, etc.) → internal events/API calls.
- **Consumes**: internal services via HTTP/events.
- **Forbidden**: long‑term domain state beyond lightweight logs.

### mcp/

- **Owns**: MCP protocol implementation, mapping tools/resources to internal services, enforcing AI‑facing rules.
- **Consumes**: `data/`, `vector_rag/`, `graph_rag/`, `auth/`, `billing/` via HTTP or shared crates.
- **Forbidden**: connector/ingestion/embedding/graph/billing business logic.

### backend/

- **Owns**: public HTTP/GraphQL APIs and BFF orchestration for `frontend/`.
- **Consumes**: other services (especially `decision_engine/`) via HTTP/shared crates.
- **Forbidden**: owning domain state that belongs in underlying microservices.

### frontend/

- **Owns**: UI, client‑side state, navigation, UX flows, presentation logic.
- **Consumes**: APIs from `backend/`, `auth/`, `data/`, `billing/`, etc.
- **Forbidden**: direct DB access, embedding/graph domain rules.

### indexers/

- **Owns**: offline/batch jobs, re‑indexing, scheduled sync orchestration.
- **Consumes**: `data/`, `chunker/`, `vector_rag/`, `graph_rag/`, and databases via public APIs/abstractions.
- **Forbidden**: embedding UI or auth flows.

---

## 3. Dependency and import rules

**Allowed**

- Services → `shared/`:
  - `conhub-models`, `conhub-config`, `conhub-middleware`, shared plugins/utils.
- Services → `database/`:
  - Shared DB utilities and migrations where explicitly designed to be cross‑service infra.

**Forbidden**

- Service → service direct imports (e.g. `auth` importing `data` Rust modules).
- Shared crates depending on services (e.g. `shared/models` importing `billing`).

**When to add to `shared/`**

- Only if:
  - Used by at least two services.
  - Stable and generic enough to be platform‑level.
- If used by only one service, keep it in that service.

---

## 4. Data and database guidelines

- Each service owns its logical data model and invariants.
- Even with a shared DB, treat schemas as per‑service ownership zones.

**Cross‑service data**

- Prefer APIs/events over reading each other’s tables.
- If cross‑table reads are unavoidable:
  - Document them explicitly.
  - Use shared models, not ad‑hoc structs.

**Migrations**

- Put migrations in the crate that conceptually owns the data.
- Do not modify another service’s tables without a documented, shared contract.

---

## 5. Communication patterns

**Synchronous**

- Use HTTP, REST, or GraphQL.
- Include authentication (JWT, service tokens) as appropriate.
- Keep payloads minimal; version APIs when changing them.

**Asynchronous**

- Prefer events/queues/streams when:
  - Updating search/graph indexes from repo changes.
  - Recording billing usage from ingestion/retrieval activity.
  - Emitting security/audit events.

**No hidden backdoors**

- Do not bypass APIs using direct DB, FS, or in‑process calls across services.

---

## 6. Practical rules for AI (and humans)

**Choosing where to put new code**

- Auth? → `auth/`
- Connectors, ingestion, document metadata? → `data/`
- Pricing, plans, limits, invoices? → `billing/`
- Chunking (code/docs/chat/web)? → `chunker/`
- Embeddings/vector search? → `vector_rag/`
- Graph RAG / knowledge graph? → `graph_rag/`
- Retrieval strategy + context fusion? → `decision_engine/`
- MCP or AI‑facing tools? → `mcp/`
- UI/UX? → `frontend/`
- Cross‑cutting config/models/middleware? → `shared/` (only if used by multiple services)

**Extending APIs**

- Prefer adding endpoints/fields to existing services over leaking their logic into callers.
- Keep APIs stable; support old + new shapes when possible.

**Chunking rule**

- Chunking belongs in **`chunker/`**, not `data/`.
- `data/` decides *what* and *when* to process, then calls `chunker/` over HTTP.
- Do not move chunking logic back into `data/`.

**Tests**

- Unit tests: inside each service crate.
- Cross‑service/e2e tests: under `tests/`, using services over HTTP/MCP.

**Future repo split**

- Avoid relative imports between service folders (except `shared/` + infra crates).
- Keep each service buildable with:
  - Its own folder,
  - `shared/` crates,
  - `database/` + infra as needed.

---

## 7. Summary

- Treat each top‑level service folder  
  (`auth/`, `data/`, `billing/`, `chunker/`, `vector_rag/`, `graph_rag/`, `decision_engine/`, `security/`, `webhook/`, `mcp/`, `backend/`, `frontend/`, `indexers/`)  
  as its **own microservice**.
- Use `shared/` only for truly cross‑cutting concerns; treat `database/` as infrastructure.
- Communicate across services via APIs/events; never via internal imports.
- Keep the codebase ready for each service to live in its own repository.