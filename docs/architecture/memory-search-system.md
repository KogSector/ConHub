# ConHub Memory Search System

## Overview

The Memory Search System is ConHub's unified knowledge layer that enables AI agents to search across all connected data sources (code, docs, chat, tickets, robot memory) using intelligent retrieval strategies.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          AI Agents                                    │
│         (GitHub Copilot, Windsurf, Cursor, Cline, etc.)             │
└─────────────────────────┬───────────────────────────────────────────┘
                          │ MCP Protocol
                          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       MCP Service (mcp/)                             │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                  Memory Connector                              │   │
│  │  Tools: memory.search, memory.robot_search,                    │   │
│  │         memory.robot_context, memory.store, memory.analyze     │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────┬───────────────────────────────────────────┘
                          │ HTTP API
                          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   Decision Engine (decision_engine/)                  │
│  ┌────────────────┐  ┌─────────────────┐  ┌───────────────────┐     │
│  │ Query Analyzer │  │ Strategy Select │  │ Context Builder   │     │
│  │  - QueryKind   │  │  - VectorOnly   │  │  - Merge results  │     │
│  │  - Modality    │  │  - GraphOnly    │  │  - Rerank         │     │
│  │  - Entities    │  │  - Hybrid       │  │  - Token budget   │     │
│  └────────────────┘  └─────────────────┘  └───────────────────┘     │
│                              │                                        │
│              ┌───────────────┴───────────────┐                       │
│              ▼                               ▼                        │
│  ┌─────────────────────┐        ┌─────────────────────┐             │
│  │   Vector RAG Client │        │   Graph RAG Client  │             │
│  └─────────────────────┘        └─────────────────────┘             │
└────────────┬────────────────────────────────┬───────────────────────┘
             │                                │
             ▼                                ▼
┌─────────────────────┐           ┌─────────────────────┐
│    vector_rag/      │           │     graph_rag/      │
│  (Qdrant + embed)   │           │   (Neo4j + graphs)  │
└─────────────────────┘           └─────────────────────┘
```

## Components

### 1. Decision Engine (`decision_engine/`)

The decision engine is the brain of the memory search system.

#### Query Analysis (`services/query_analysis.rs`)

Analyzes natural language queries to determine:

- **QueryKind**: What type of question is being asked?
  - `FactLookup` - "What is X?"
  - `EpisodicLookup` - "What happened when?"
  - `TopologyQuestion` - "Who owns?" "What depends on?"
  - `Explainer` - "How does X work?"
  - `HowTo` - "How do I configure?"
  - `Troubleshooting` - "Why did X fail?"
  - `TaskSupport` - "What should I do next?"
  - `Comparison` - "Difference between X and Y?"
  - `Aggregation` - "How many?" "List all?"

- **ModalityHint**: What content type to prioritize?
  - `Code` - Source code
  - `Docs` - Documentation, markdown
  - `Chat` - Slack, Discord messages
  - `Tickets` - Issues, PRs
  - `RobotEpisodic` - Robot episodes
  - `RobotSemantic` - Robot facts
  - `Mixed` - Unknown/multiple

#### Strategy Selection

Based on query analysis, selects the optimal retrieval strategy:

| Query Kind | Default Strategy |
|------------|-----------------|
| TopologyQuestion | GraphOnly |
| FactLookup | VectorOnly |
| EpisodicLookup | VectorOnly (time-filtered) |
| Explainer | Hybrid |
| HowTo | Hybrid |
| Troubleshooting | VectorThenGraph |
| TaskSupport | Hybrid |
| Comparison | VectorOnly |
| Aggregation | VectorOnly |

#### Context Builder (`services/context_builder.rs`)

Merges and ranks results from multiple retrieval sources:

- **Deduplication** - Removes duplicate chunks
- **Re-ranking strategies**:
  - `ScoreBased` - Simple score sorting
  - `ReciprocalRankFusion` - For hybrid results
  - `DiversityAware` - MMR-like diversity
  - `RecencyBiased` - Boost recent content
- **Token budget** - Enforces max tokens/blocks

### 2. MCP Memory Connector (`mcp/src/connectors/memory.rs`)

Exposes memory search to AI agents via MCP protocol.

#### Tools

| Tool | Description |
|------|-------------|
| `memory.search` | General knowledge search across all sources |
| `memory.robot_search` | Robot episodic and semantic memory search |
| `memory.robot_context` | Get robot's current context snapshot |
| `memory.store` | Store passive context for later retrieval |
| `memory.analyze_query` | Debug: see how a query would be classified |

#### Example Usage

```json
// MCP tool call
{
  "name": "memory.search",
  "arguments": {
    "query": "How does authentication work in the auth service?",
    "sources": ["code", "docs"],
    "max_blocks": 10,
    "strategy": "hybrid"
  }
}
```

### 3. Relation Builder (`indexers/src/relation_builder.rs`)

Extracts relations and builds knowledge graph from robot episodes.

#### Relation Types

- `ObjectLocation` - Object seen at location
- `PersonLocation` - Person seen at location
- `TaskLocation` - Task performed at location
- `TaskObject` - Task involves object
- `RouteConnection` - Route connects locations
- `TemporalPattern` - Recurring time patterns

#### Graph Operations

- Creates nodes: Robot, Episode, Location, Object, Person, Task
- Creates edges: HAD_EPISODE, AT_LOCATION, SAW_OBJECT, INVOLVED_PERSON, RELATED_TO_TASK

## API Endpoints

### Decision Engine (port 3016)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check with features |
| `/api/memory/search` | POST | General memory search |
| `/api/memory/health` | GET | Memory search health |
| `/api/robots/:robot_id/memory/search` | POST | Robot memory search |
| `/api/robots/:robot_id/context/latest` | GET | Robot context snapshot |
| `/context/query` | POST | Legacy context query |
| `/context/stats` | GET | Cache/query stats |

### Request Format

```json
// POST /api/memory/search
{
  "tenant_id": "uuid",
  "user_id": "uuid",
  "query": "How does the billing module work?",
  "sources": ["code", "docs"],
  "filters": {
    "repos": ["backend", "billing"]
  },
  "max_blocks": 20,
  "max_tokens": 8000,
  "force_strategy": "hybrid",  // optional
  "include_debug": true
}
```

### Response Format

```json
{
  "blocks": [
    {
      "id": "uuid",
      "source_id": "document-uuid",
      "text": "The billing module handles...",
      "source_type": "code",
      "score": 0.92,
      "token_count": 150,
      "metadata": {
        "source": "github",
        "repo": "billing",
        "path": "src/services/billing.rs"
      }
    }
  ],
  "total_results": 45,
  "query_kind": "explainer",
  "strategy_used": "hybrid",
  "took_ms": 125,
  "debug": {
    "modality_hint": "code",
    "collections_searched": ["code", "docs"],
    "vector_results": 30,
    "graph_results": 15
  }
}
```

## Environment Variables

```bash
# Decision Engine
DECISION_ENGINE_HOST=0.0.0.0
DECISION_ENGINE_PORT=3016
VECTOR_RAG_URL=http://localhost:8082
GRAPH_RAG_URL=http://localhost:3015
REDIS_URL=redis://localhost:6379

# MCP Service
DECISION_ENGINE_URL=http://localhost:3016

# Indexers
GRAPH_RAG_ENABLED=true
KAFKA_ENABLED=false  # true for production
```

## Files Created/Modified

### New Files

- `decision_engine/src/models/query.rs` - Query types and models
- `decision_engine/src/models/mod.rs` - Model exports
- `decision_engine/src/services/query_analysis.rs` - Query analyzer
- `decision_engine/src/services/context_builder.rs` - Result merger
- `decision_engine/src/services/memory_search.rs` - Main search service
- `decision_engine/src/handlers/memory.rs` - HTTP handlers
- `mcp/src/connectors/memory.rs` - MCP memory connector
- `indexers/src/relation_builder.rs` - Relation extraction

### Modified Files

- `decision_engine/src/main.rs` - Added routes
- `decision_engine/src/services/mod.rs` - Added exports
- `decision_engine/src/handlers/mod.rs` - Added memory module
- `mcp/src/connectors/mod.rs` - Added memory connector
- `mcp/src/connectors/manager.rs` - Initialize memory connector
- `indexers/src/lib.rs` - Added relation builder
- `indexers/src/robot_memory.rs` - Integrated relation builder

## Status

- ✅ Query analysis and classification
- ✅ Strategy selection (vector/graph/hybrid)
- ✅ Memory search HTTP endpoints
- ✅ MCP tools for AI agents
- ✅ Relation builder for knowledge graph
- ✅ Robot memory search
- ✅ All code compiles successfully
