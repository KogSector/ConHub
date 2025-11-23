---
trigger: always_on
---

ConHub – Knowledge Layer & Context Engine
ConHub is a knowledge layer and context engine that sits between a user’s data sources and their AI agents.

It connects to many different systems (code hosts, cloud docs, SaaS tools, URLs, chat, etc.), ingests and normalizes their content, and then embeds, indexes, and organizes that content so AI agents can use it safely, accurately, and efficiently.

ConHub exposes this unified knowledge layer to AI agents (via MCP and other interfaces) and provides a shared rules system so that all agents follow the same security, governance, and behavior policies.

---

## 1. What This File Is

- **Purpose**  
  Describe what ConHub is, how the end‑to‑end architecture works (ingestion → indexing → retrieval), and what users expect from the system.  
  Detailed per‑service rules live in `docs/architecture/microservices.md` and [.windsurf/rules/microservice-guide.md](cci:7://file:///c:/Users/risha/Desktop/Work/ConHub/.windsurf/rules/microservice-guide.md:0:0-0:0).

- **Audience**  
  AI coding assistants and human contributors who need a mental model of ConHub’s overall architecture and data flows.

---

## 2. Product Vision – What ConHub Is

- **Knowledge layer and context engine**  
  ConHub unifies code, docs, tickets, chat, and web content into a single, permission‑aware knowledge layer optimized for LLMs.

- **Connect data sources and agents**  
  Upstream, ConHub integrates GitHub, Bitbucket, Google Drive, Dropbox, Slack, URLs, local files, and more.  
  Downstream, it serves GitHub Copilot, Windsurf, Cursor, Cline, Trae, and any MCP‑compatible agent or HTTP client.

- **Shared rules and governance**  
  All access, safety, and behavior rules are defined once in ConHub and enforced consistently across agents and automations.

---

## 3. High‑Level Product Goals

- **Unify fragmented knowledge**  
  Pull repositories, documents, tickets, and chat into a single, queryable knowledge layer per tenant.

- **Make content LLM‑ready**  
  Normalize and chunk content into semantically meaningful, token‑bounded units and index them in both vector and graph form.

- **Serve many agents consistently**  
  Provide stable, permission‑aware APIs and MCP tools so different agents see the same truth and follow the same rules.

- **Centralize rules and security**  
  Enforce RBAC, data residency, PII controls, and safety policies at ingestion, indexing, and retrieval time.

---

## 4. Core Architecture Overview

ConHub is a **microservices monorepo**. Each top‑level service is independently deployable; cross‑service behavior goes through APIs, events, or shared crates.

At a high level:

- **auth/** – Authentication, users, tokens, RBAC.  
- **data/** – Connectors + ingestion orchestration + document/chunk metadata.  
- **chunker/** – Advanced chunking for code, docs, HTML, and chat.  
- **vector_rag/** – Embeddings, vector search, semantic retrieval.  
- **graph_rag/** – Knowledge graph construction and graph‑based retrieval.  
- **decision_engine/** – Strategy selection (vector / graph / hybrid / auto) and context orchestration.  
- **backend/** – Public HTTP/GraphQL API and BFF for the UI.  
- **mcp/** – MCP server exposing ConHub as tools/resources to AI agents.  
- **frontend/** – Next.js UI for configuration, connections, and insights.  
- **billing/**, **security/**, **indexers/** – Plans/usage, security/audit, and background jobs.

For detailed per‑service responsibilities, see `docs/architecture/microservices.md` and [.windsurf/rules/microservice-guide.md](cci:7://file:///c:/Users/risha/Desktop/Work/ConHub/.windsurf/rules/microservice-guide.md:0:0-0:0).

---

## 5. Ingestion & Indexing – End‑to‑End Flow

### 5.1 Ingestion Pipeline

1. **Connectors (data/)**  
   - Users connect sources (GitHub, Google Drive, Dropbox, URLs, Slack, etc.).  
   - `data/` manages connected accounts, schedules sync jobs, and tracks runs.

2. **Fetch & Normalize (data/)**  
   - For each job, `data/` fetches raw items (files, PRs, wiki pages, tickets, messages, web pages).  
   - Converts them into a **normalized document model** with metadata (tenant, source, repo, path, tags, timestamps).

3. **Chunking (chunker/)**  
   - `data/` sends normalized documents to `chunker/`.  
   - `chunker/` applies content‑specific strategies:
     - Code: AST‑based splitting by functions/classes/methods.
     - Docs/Markdown/HTML: heading‑based, section‑aware chunking.
     - Chat: message‑window chunking that preserves conversation context.
   - Chunks are sized using token limits so they fit into LLM contexts.

4. **Vector Indexing (vector_rag/)**  
   - `chunker/` (or `data/`) sends chunks to `vector_rag/`.  
   - `vector_rag/` generates embeddings and upserts them into the vector DB (e.g. Qdrant) with rich metadata and filters.

5. **Graph Indexing (graph_rag/)**  
   - From chunks and document metadata, `graph_rag/` builds a knowledge graph (files, functions, services, tickets, people, etc.).  
   - Creates nodes/edges representing ownership, references, mentions, dependencies, and temporal relationships.

6. **Sync Jobs & Observability (data/ + indexers/)**  
   - `data/` and `indexers/` track job and run status, failures, retries, and provide visibility into ingestion health.

---

## 6. Retrieval & Decision Engine

### 6.1 Retrieval Components

- **vector_rag/** – Fast semantic search over chunks using embeddings.  
- **graph_rag/** – Graph traversal and relationship‑aware retrieval.  
- **decision_engine/** – Orchestrates both to produce the best context for a query.

### 6.2 Query Flow

1. **Client request**  
   - A client (frontend, backend API consumer, or MCP tool) sends a natural language query plus optional filters (sources, repos, time range, tags).

2. **Decision Engine (decision_engine/)**  
   - Receives the query and filters.  
   - Chooses a strategy:
     - **Vector** – for direct “find docs/code about X” questions.
     - **Graph** – for “who/what depends on/related to” questions.
     - **Hybrid** – for complex “explain how X works / how things connect” queries.
     - **Auto** – rule‑based choice based on query patterns and filters.
   - Calls `vector_rag/` and/or `graph_rag/` in parallel as needed.

3. **Fusion & Ranking**  
   - `decision_engine/` merges vector and graph results, de‑duplicates them, and re‑ranks based on relevance, diversity, and provenance.  
   - It returns **context blocks** (chunks + metadata + provenance) to the caller.

4. **Delivery to Agents**  
   - **backend/** exposes HTTP/GraphQL endpoints for frontend and external clients.  
   - **mcp/** exposes MCP tools/resources so IDE‑embedded agents can call the same retrieval stack.  
   - Agents receive structured, filtered, tenant‑safe context ready to stuff into prompts.

5. **Caching**  
   - Redis caches:
     - Connector responses (to reduce upstream API calls).
     - Chunking/processing results (for incremental sync).  
     - Query results (for repeated queries at retrieval time).

---

## 7. What Users Expect

From a **user/tenant** perspective, ConHub should provide:

- **Easy connections**  
  - Connect GitHub, cloud docs, Slack, and URLs through a UI.  
  - Choose repos, folders, or channels to index.

- **Robust ingestion**  
  - Initial full sync of selected sources.  
  - Automatic incremental syncs that keep the knowledge layer up to date.  
  - Clear status and error reporting for sync jobs.

- **High‑quality retrieval**  
  - Ask natural language questions like:
    - “How does authentication work?”  
    - “Who worked on the billing module and what did they change?”  
    - “Show me all Slack conversations related to this incident.”  
  - Receive **small, targeted, explainable context snippets** from code, docs, and chat.

- **Consistency across agents**  
  - The same tenant‑filtered context regardless of whether the query comes from the web UI, an IDE, MCP, or a CLI agent.

- **Safety and governance**  
  - No cross‑tenant leakage.  
  - Respect repo/space/channel permissions.  
  - Central rules that can block/allow sources, domains, or data types.

---

## 8. How This Relates to Other Docs

- Use **this file** for a **mental model of the product and end‑to‑end architecture**.  
- Use [.windsurf/rules/microservice-guide.md](cci:7://file:///c:/Users/risha/Desktop/Work/ConHub/.windsurf/rules/microservice-guide.md:0:0-0:0) and `docs/architecture/microservices.md` for:
  - Precise per‑service boundaries and responsibilities.
  - Rules for where new code should live.
  - Detailed dependency and communication patterns.

Together, these documents should keep all AI assistants and human contributors aligned on **what ConHub is**, **how the data flows**, and **where new logic belongs**.