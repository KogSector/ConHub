---
trigger: always_on
---

ConHub – Knowledge Layer & Context Engine
ConHub is a knowledge layer and context engine that sits between a user’s data sources and their AI agents.

It connects to many different systems (code hosts, cloud docs, SaaS tools, URLs, etc.), ingests and normalizes their content, and then embeds, indexes, and organizes that content so AI agents can use it safely, accurately, and efficiently.

ConHub exposes this unified knowledge layer to AI agents (via MCP and other interfaces) and provides a shared rules system so that all agents follow the same security, governance, and behavior policies.

## 1. What This File Is

- **Purpose**  
  Explain what ConHub is (a knowledge layer and context engine between user data and AI agents) and define ground rules for AI assistants and agentic tools working in this repository.

- **Audience**  
  AI coding assistants (Copilot, Windsurf, Cursor, Cline, Trae, MCP tools, etc.) and human contributors who want a quick orientation.

---

## 2. Product Vision – What ConHub Is

- **Knowledge layer and context engine**  
  ConHub connects many data sources (repositories, documents, SaaS tools, URLs, chat, etc.), normalizes, chunks, embeds, and indexes their content, and exposes this unified knowledge layer to AI agents via MCP and other APIs.

- **Connect data sources and agents**  
  Upstream, ConHub integrates GitHub, Bitbucket, Google Drive, Dropbox, OneDrive (planned), Notion (planned), Confluence (planned), JIRA (planned), Slack, local files, direct URLs, and more.  
  Downstream, ConHub serves GitHub Copilot, Cline, Trae, Windsurf, Cursor, and other MCP-compatible agents.

- **Shared rules and governance**  
  ConHub provides a central rule system for permissions, safety, and behavior. Rules are agent-agnostic: defined once in ConHub and enforced across all agents and automations.

---

## 3. High-Level Product Goals

- **Unify fragmented knowledge**  
  Bring code, documents, tickets, chat, and web content into a single, queryable knowledge layer.

- **Make content LLM-ready**  
  Structure and chunk content intelligently (functions, classes, sections, paragraphs, message threads) and create high-quality embeddings and indexes for accurate semantic retrieval.

- **Serve many agents consistently**  
  Provide consistent, permission-aware context to any assistant, regardless of vendor, IDE, or model.

- **Centralize rules and security**  
  Enforce access control, safety, compliance, and behavior policies from one place, with strong auditing and governance.

---

## 4. Architecture Overview – Microservices

ConHub is a **microservices monorepo**. Each top-level service folder is an independent microservice with a clear domain. Cross-service behavior must use APIs, events, or shared crates, never ad‑hoc internal imports.

### 4.1 Core Services

- **auth/**  
  Owns users, identities, sessions, JWTs, OAuth flows, and RBAC.  
  Does not own connectors, documents, billing, or embeddings.  
  Use this service for identity, tokens, and permission checks only.

- **data/**  
  Owns connectors, connected accounts, documents, document chunks, sync jobs and sync runs, and the ingestion pipeline.  
  Responsibilities: fetching raw content from sources (GitHub, local files, Google Drive, Bitbucket, URL crawler, Slack, Dropbox, etc.), normalizing and chunking documents, deciding what to (re)sync and when, and calling `embedding/` for embeddings and indexing.  
  Do not implement embedding internals or billing logic here, and do not bypass `embedding/` to talk directly to the vector database.

- **embedding/**  
  Owns embedding generation, vector orchestration, similarity search, and ranking.  
  Responsibilities: managing collections and index structures in the vector database, upserting and deleting points, tuning indexes, and performing semantic retrieval.  
  Do not put connector or ingestion business logic here; treat this as a stateless service that consumes chunked content.

- **billing/**  
  Owns plans, subscriptions, invoices, payment methods, usage tracking, and Stripe integration.  
  Any limits, plan-based gating, or paid features live here.  
  Do not mix in embedding or connector-specific logic.

- **mcp/**  
  Owns the MCP protocol implementation and mapping ConHub’s data into MCP tools and resources.  
  Add new MCP tools/resources here when exposing new capabilities to AI agents.  
  Do not move ingestion, embedding, or billing logic into `mcp/`; call the relevant service instead.

- **backend/**  
  Owns public HTTP/GraphQL APIs and the BFF layer for `frontend/`.  
  Aggregates and orchestrates calls to other services.  
  Avoid embedding domain logic that should live in underlying microservices.

- **frontend/**  
  Owns the Next.js UI (dashboards, connection management, billing UI, settings, etc.).  
  Talks to `backend/` (and auth endpoints when appropriate), never directly to databases.

- **security/**  
  Owns audit events, security policies, threat detection, and rate limiting.  
  Used for enforcement and auditing, not business workflows.

- **indexers/**  
  Owns offline and batch jobs, reindexing, and scheduled sync orchestration.  
  Used for long-running or scheduled operations that call into `data/` and `embedding/`.

### 4.2 Shared Layers and Infrastructure

- **shared/** (`config/`, `models/`, `middleware/`, `plugins/`)  
  Owns cross-service models, configuration, middleware, and utilities.  
  Only move types/helpers here if used by multiple services.  
  `shared/` must not depend on any specific service.

- **database/**  
  Owns migrations and database infrastructure (NeonDB, SQLx helpers).  
  Migrations belong in the crate that conceptually owns the data (auth, data, billing, etc.), using this as infrastructure.

---

## 5. Ingestion & Indexing – How the Engine Works

- **Connector layer (data/)**  
  Fetches raw content from external systems using OAuth or tokens.  
  Handles pagination, rate limits, and change detection (incremental sync).

- **Normalization and chunking (data/)**  
  Converts raw items (files, PRs, wiki pages, tickets, messages, web pages) into a common document model.  
  Splits documents into chunks appropriate for LLM retrieval:  
  code (functions, classes, logical blocks),  
  docs (headings, sections, paragraphs),  
  chat (message windows or threads).

- **Embedding and vector indexing (embedding/)**  
  Generates embeddings per chunk using configured models.  
  Stores vectors with metadata for filtering (source, repo, path, user, tags, timestamps, etc.).

- **Sync jobs and runs (data/ + indexers/)**  
  Track progress and failures at job/run level.  
  Enable resuming, retries, and observability into ingestion health.

- **Retrieval (embedding/ + mcp/ + backend/)**  
  Query the vector store with filters and ranking.  
  Package results into context responses optimized for AI agents.

---

## 6. Rule System – Shared Policies for All Agents

- **Central rule engine**  
  Provides one place to define access control, safety rules, and behavioral guidelines.

- **Scope of rules**  
  Access and permissions (who can see which documents, repos, tickets, chats).  
  Safety and compliance (redaction, PII handling, allowed/blocked domains).  
  Behavior (how agents should act, which operations require confirmation, what may not be done).

- **Enforcement points**  
  During ingestion (what gets stored and how).  
  During retrieval (what context can be returned).  
  At MCP/API boundaries (what tools and resources agents can see and call).

---

## 7. Rules for AI Assistants Editing This Repo

- **Respect microservice boundaries**  
  Keep logic inside the service that owns that domain.  
  Do not import internal modules from another service; use HTTP, GraphQL, queues, or shared crates.

- **Choose the right home for new logic**  
  Auth concerns go to `auth/`.  
  Connector and ingestion logic goes to `data/`.  
  Embedding and vector search logic goes to `embedding/`.  
  Pricing, plans, and limits go to `billing/`.  
  MCP tools and AI-facing resources go to `mcp/`.  
  UI/UX changes go to `frontend/`.  
  Cross-cutting models/config/middleware go to `shared/` only if used by multiple services.

- **Database and migrations**  
  Put migrations in the crate that owns the data’s domain.  
  Avoid cross-service table mutations unless clearly documented and justified.  
  Assume NeonDB via `DATABASE_URL_NEON` is the primary database.

- **APIs over backdoors**  
  Prefer HTTP/GraphQL APIs or events when one service needs another’s data or functionality.  
  Do not rely on reading another service’s tables directly unless explicitly allowed.

- **Testing strategy**  
  Unit tests live inside each service crate.  
  Cross-service and integration tests go under `tests/`, using services over HTTP/MCP instead of internal imports.

- **Security and privacy**  
  Always consider tenant boundaries, RBAC, and rule system impact when exposing new features.  
  Include necessary metadata to enable rule-based filtering and auditing.

- **Assistant behavior style**  
  Be concise, use Markdown, and avoid unnecessary chatter.  
  Prefer concrete changes or tool-based proposals over vague advice.  
  When unsure, read relevant files or ask for clarification instead of guessing.

---

## 8. Roadmap & Priorities (High-Level)

- **Connectors**  
  Harden existing connectors (GitHub, local files, Google Drive, Bitbucket, URL crawler, Slack, Dropbox).  
  Add connectors for Notion, Confluence, JIRA, OneDrive, and additional SaaS tools.

- **Retrieval quality**  
  Improve chunking strategies per content type.  
  Enhance hybrid retrieval (semantic + keyword + metadata filtering).  
  Optimize context packaging for different agent types and tasks.

- **Rules and governance**  
  Expand the rule language for per-tenant and per-agent policies.  
  Improve auditing and explainability of why specific context was returned or blocked.

- **Knowledge graph and multi-agent workflows**  
  Model relationships between entities (files, issues, PRs, people, systems).  
  Enable coordinated workflows across multiple agents over the same knowledge layer.