# Apache-Powered Robotics & Sensor Integration for ConHub

**Status**: Implementation in progress  
**Branch**: `memory`  
**Scope**: How to use Apache Kafka/Flink/Spark as a scalable backbone for robot sensors, computer vision, and memory, integrated into ConHub's knowledge layer.

---

## 1. Goals

- **Human-like understanding for robots**  
  ConHub acts as a "brain" that can understand and reason over a robot's experiences, not just static documents.

- **Multi-modal ingestion**  
  Connect sensors, computer vision systems, and logs via Apache components.

- **Memory layer around agents**  
  Provide episodic, semantic, and passive context memory for each robot and agent.

- **24×7 scalability**  
  Use Apache's distributed systems to handle continuous, high-volume data streams.

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PERCEPTION LAYER                                │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐           │
│  │ Camera  │  │  LiDAR  │  │   IMU   │  │  Audio  │  │ Tactile │           │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘           │
│       │            │            │            │            │                  │
│       └────────────┴────────────┴────────────┴────────────┘                  │
│                                    │                                         │
│                          ┌─────────▼─────────┐                              │
│                          │   Robot Client    │                              │
│                          │  (Kafka/HTTP)     │                              │
│                          └─────────┬─────────┘                              │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     │
┌────────────────────────────────────┼────────────────────────────────────────┐
│                         APACHE BACKBONE                                      │
│                                    │                                         │
│  ┌─────────────────────────────────▼─────────────────────────────────────┐  │
│  │                         Apache Kafka                                   │  │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐    │  │
│  │  │ robot.*.raw_events│  │ robot.*.cv_events │  │ robot.*.frames   │    │  │
│  │  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘    │  │
│  │           │                     │                     │               │  │
│  │           └─────────────────────┴─────────────────────┘               │  │
│  └───────────────────────────────────┬───────────────────────────────────┘  │
│                                      │                                       │
│  ┌───────────────────────────────────▼───────────────────────────────────┐  │
│  │                         Apache Flink                                   │  │
│  │  ┌────────────────────────┐  ┌────────────────────────┐              │  │
│  │  │ RobotEpisodeBuilderJob │  │ RobotSemanticEventJob  │              │  │
│  │  └───────────┬────────────┘  └───────────┬────────────┘              │  │
│  │              │                           │                            │  │
│  │              ▼                           ▼                            │  │
│  │  ┌──────────────────┐        ┌──────────────────────┐                │  │
│  │  │ robot.*.episodes │        │ robot.*.semantic_events│               │  │
│  │  └────────┬─────────┘        └───────────┬──────────┘                │  │
│  └───────────┼──────────────────────────────┼────────────────────────────┘  │
│              │                              │                                │
└──────────────┼──────────────────────────────┼────────────────────────────────┘
               │                              │
┌──────────────┼──────────────────────────────┼────────────────────────────────┐
│              │     CONHUB KNOWLEDGE LAYER   │                                │
│              │                              │                                │
│  ┌───────────▼──────────────────────────────▼───────────────────────────┐   │
│  │                    indexers/ (RobotMemoryIndexer)                     │   │
│  │  - Consumes Kafka topics                                              │   │
│  │  - Converts to ConHub documents                                       │   │
│  │  - Calls chunker/, vector_rag/, graph_rag/                           │   │
│  └───────────────────────────────┬───────────────────────────────────────┘   │
│                                  │                                           │
│  ┌───────────────────────────────▼───────────────────────────────────────┐   │
│  │                         chunker/                                       │   │
│  │  - RobotEventChunker: chunk activity logs and episodes               │   │
│  │  - SensorSummaryChunker: merge summaries into LLM-ready segments     │   │
│  └───────────────────────────────┬───────────────────────────────────────┘   │
│                                  │                                           │
│  ┌───────────────┬───────────────┴───────────────┬───────────────────────┐   │
│  │               │                               │                       │   │
│  ▼               ▼                               ▼                       │   │
│  ┌───────────┐  ┌───────────────────┐  ┌─────────────────────────────┐   │   │
│  │vector_rag/│  │    graph_rag/     │  │    decision_engine/         │   │   │
│  │           │  │                   │  │                             │   │   │
│  │Collections│  │ Nodes:            │  │ Strategies:                 │   │   │
│  │- episodic │  │ - Robot           │  │ - Vector search             │   │   │
│  │- semantic │  │ - Episode         │  │ - Graph traversal           │   │   │
│  │           │  │ - Location        │  │ - Hybrid fusion             │   │   │
│  │           │  │ - Object          │  │                             │   │   │
│  │           │  │ - Person          │  │                             │   │   │
│  │           │  │ - Task            │  │                             │   │   │
│  └─────┬─────┘  └─────────┬─────────┘  └──────────────┬──────────────┘   │   │
│        │                  │                           │                   │   │
│        └──────────────────┴───────────────────────────┘                   │   │
│                                  │                                        │   │
│  ┌───────────────────────────────▼───────────────────────────────────┐   │   │
│  │                    backend/ + mcp/                                 │   │   │
│  │  - POST /api/robots/{id}/memory/search                            │   │   │
│  │  - GET /api/robots/{id}/context/latest                            │   │   │
│  │  - MCP tools: robot_memory.search, robot_context.latest_state     │   │   │
│  └───────────────────────────────────────────────────────────────────┘   │   │
│                                                                           │   │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Apache Components and How We Use Them

### 3.1 Apache Kafka – Event Backbone

**Role**: High-throughput, durable event log for robot data.

**Use cases**:

- Ingest all robot-related events:
  - Raw sensor events (IMU, lidar, encoders, etc.)
  - Computer vision events (detections, tracking, segmentation)
  - High-level events (task start/stop, failures, warnings)
- Partition topics per robot and/or per site for horizontal scalability
- Provide replayable history for:
  - Rebuilding episodic memory
  - Re-training models and re-indexing

**Typical topics** (per robot):

| Topic | Description |
|-------|-------------|
| `robot.<robot_id>.raw_events` | Raw sensor readings |
| `robot.<robot_id>.cv_events` | Computer vision detections |
| `robot.<robot_id>.frames` | Frame references (URIs to images) |
| `robot.<robot_id>.episodes` | Processed episodes (from Flink) |
| `robot.<robot_id>.semantic_events` | Semantic facts (from Flink) |
| `robot.<robot_id>.control` | Control commands to robot |
| `robot.<robot_id>.anomalies` | Detected anomalies |

**Connection points**:

- Robots can either:
  - Use a **Kafka client** directly with credentials from `data/`'s registration API
  - Call **HTTP ingestion endpoints** in `data/`, which then publish to Kafka

---

### 3.2 Apache Flink – Real-Time Stream Processing

**Role**: Transform raw event streams into LLM-friendly, structured knowledge.

**Use cases**:

- Windowed sensor aggregation (e.g., 10-second windows)
- Detect patterns and anomalies (e.g., "door open > 5 minutes")
- Build **episodes** as bounded sequences of events:
  - `Episode = {start_time, end_time, location, actions, observations, outcome}`
- Generate short natural-language summaries and structured metadata:
  - "Robot saw 3 people near aisle 5 and delivered package to dock B."

**Jobs**:

| Job | Input Topics | Output Topics |
|-----|--------------|---------------|
| `RobotEpisodeBuilderJob` | `raw_events`, `cv_events` | `episodes` |
| `RobotSemanticEventJob` | `episodes` | `semantic_events`, `anomalies` |

**Integration with ConHub**:

- `indexers/` subscribes to `episodes` and `semantic_events`
- For each message, `indexers/`:
  - Converts it into ConHub's normalized document model
  - Calls `chunker/` → `vector_rag/` → `graph_rag/`

---

### 3.3 Apache Spark – Offline Batch Analytics (Optional)

**Role**: Heavy, offline processing for large volumes of historical robot data.

**Use cases**:

- Long-horizon trend detection (battery health, failure prediction)
- Generating daily/weekly summaries for each robot
- Preparing training datasets for ML models (e.g., navigation, manipulation)

**Data sources**:

- Kafka topics (using Spark structured streaming or batch)
- Iceberg/Hudi tables on object storage

**Outputs**:

- Summarized "daily memory" topics:
  - `robot.<robot_id>.daily_summaries`
- Offline reports written into data lake tables

---

### 3.4 Apache Cassandra – Telemetry & State Store (Optional)

**Role**: Time-series and key-value storage for high-volume telemetry.

**Use cases**:

- Storing raw numeric sensor values, CPU/battery metrics, heartbeat signals
- Fast lookups for "latest N readings" outside LLM context

**Integration**:

- Flink jobs can write to Cassandra for low-latency access
- ConHub itself stores only references and summarized views, not raw telemetry

---

### 3.5 Apache Iceberg / Hudi – Data Lake Tables (Optional)

**Role**: Long-term storage of episodes, semantic events, and logs in object storage.

**Use cases**:

- Rebuilding knowledge indexes for new models
- Offline analytics and compliance

---

## 4. ConHub Microservice Integration

### 4.1 `data/` – Robot and Sensor Connectors

**Responsibilities**:

- Register robots and their sensors
- Manage metadata: robots, streams, schemas
- Provide connection details for Kafka (bootstrap servers, topics, credentials)
- Provide HTTP ingestion endpoints as an alternative to direct Kafka producers

**Key APIs** (implemented):

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/robots/register` | POST | Register a new robot |
| `/api/robots/{robot_id}` | GET | Get robot info |
| `/api/robots/{robot_id}` | DELETE | Delete a robot |
| `/api/robots/{robot_id}/streams` | POST | Declare a sensor stream |
| `/api/robots/{robot_id}/heartbeat` | POST | Update robot heartbeat |
| `/api/ingestion/robots/{robot_id}/events` | POST | Ingest raw events |
| `/api/ingestion/robots/{robot_id}/events/batch` | POST | Ingest batch of events |
| `/api/ingestion/robots/{robot_id}/cv_events` | POST | Ingest CV events |
| `/api/ingestion/robots/{robot_id}/cv_events/batch` | POST | Ingest batch of CV events |
| `/api/ingestion/robots/{robot_id}/frames` | POST | Ingest frame references |

---

### 4.2 `indexers/` – Robot Memory Indexer

**Responsibilities**:

- Consume Kafka topics:
  - `robot.*.episodes`
  - `robot.*.semantic_events`
  - Optional: `robot.*.daily_summaries`
- Normalize messages into ConHub "documents":
  - Text fields: natural-language summaries
  - Metadata: robot_id, location, time, objects, tasks, etc.
- Call downstream services:
  - `chunker/` → LLM-ready chunks
  - `vector_rag/` → embeddings & vector index
  - `graph_rag/` → nodes/edges for relationships

**Memory types**:

- **Episodic memory**:
  - Indexed as "events with time"
  - Stored in a dedicated collection and graph subgraph

- **Semantic memory**:
  - Stable facts derived from many episodes
  - Stored separately for long-term reasoning

---

### 4.3 Retrieval: `decision_engine/`, `backend/`, `mcp/`

**Robot-oriented APIs** (to be implemented):

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/robots/{robot_id}/memory/search` | POST | Query robot memory |
| `/api/robots/{robot_id}/context/latest` | GET | Get latest context snapshot |

**MCP Tools** (conceptual):

- `robot_memory.search`
- `robot_context.latest_state`
- `robot_episodes.trace_task`

---

## 5. Database Schema

New tables in `database/migrations/013_create_robot_memory_tables.sql`:

| Table | Description |
|-------|-------------|
| `robots` | Registered robots with Kafka/HTTP config |
| `robot_streams` | Sensor streams with schemas |
| `robot_episodes` | Episodic memory entries |
| `robot_semantic_facts` | Long-term semantic facts |
| `robot_events_log` | Passive context storage |
| `robot_memory_cache` | Query result cache |

---

## 6. Environment Variables

Add these to your `.env` files:

```bash
# Kafka Configuration
KAFKA_ENABLED=true
KAFKA_BOOTSTRAP_SERVERS=localhost:9092
KAFKA_SASL_MECHANISM=SCRAM-SHA-256
KAFKA_SASL_USERNAME=conhub
KAFKA_SASL_PASSWORD=secret

# Service URLs
CHUNKER_SERVICE_URL=http://localhost:3016
VECTOR_RAG_SERVICE_URL=http://localhost:3017
GRAPH_RAG_SERVICE_URL=http://localhost:3018
```

---

## 7. Getting Started

### 7.1 Development Mode (No Kafka)

The system works without Kafka for development:

1. Start the data service:
   ```bash
   cd data && cargo run
   ```

2. Register a robot:
   ```bash
   curl -X POST http://localhost:3013/api/robots/register \
     -H "Content-Type: application/json" \
     -d '{"name": "test_robot", "robot_type": "mobile", "capabilities": ["navigation", "vision"]}'
   ```

3. Ingest events via HTTP:
   ```bash
   curl -X POST http://localhost:3013/api/ingestion/robots/{robot_id}/events \
     -H "Content-Type: application/json" \
     -d '{"source": "imu", "event_type": "reading", "timestamp": "2025-01-01T00:00:00Z", "payload": {"ax": 0.1}}'
   ```

### 7.2 Production Mode (With Kafka)

1. Set up Kafka cluster
2. Set `KAFKA_ENABLED=true` in environment
3. Deploy Flink jobs for episode building
4. Start the indexer service:
   ```bash
   cd indexers && cargo run
   ```

---

## 8. Next Steps

1. ✅ Implement robot registration and ingestion APIs in `data/`
2. ✅ Create database migration for robot tables
3. ✅ Implement robot memory indexer in `indexers/`
4. ⏳ Provision Kafka and define topic naming conventions
5. ⏳ Implement Flink jobs for episode building
6. ⏳ Build retrieval APIs and MCP tools
7. ⏳ Add real Kafka client (rdkafka) integration

---

## 9. Files Created/Modified

### New Files

- `database/migrations/013_create_robot_memory_tables.sql` - Database schema
- `shared/models/src/robot.rs` - Robot type definitions
- `data/src/services/kafka_client.rs` - Kafka producer client
- `data/src/handlers/robots.rs` - Robot management handlers
- `data/src/handlers/robot_ingestion.rs` - Event ingestion handlers
- `indexers/src/robot_memory.rs` - Robot memory indexer
- `docs/architecture/apache-robotics-integration.md` - This documentation

### Modified Files

- `shared/models/src/lib.rs` - Added robot module export
- `data/src/main.rs` - Added robot routes
- `indexers/src/lib.rs` - Added robot memory exports
- `indexers/src/main.rs` - Added indexer startup
