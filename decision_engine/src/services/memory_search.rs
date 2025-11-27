//! Memory Search Service
//! 
//! Main orchestration service that:
//! 1. Analyzes the query
//! 2. Selects retrieval strategy
//! 3. Executes vector and/or graph retrieval
//! 4. Merges and ranks results
//! 5. Returns context blocks for AI agents

use anyhow::{Result, Context};
use std::time::Instant;
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::models::query::{
    MemoryQuery, RobotMemoryQuery, TimeRange as QueryTimeRange, MemorySearchResponse, DebugInfo,
    QueryKind, ModalityHint, RetrievalStrategy, RetrievalPlan,
    ContextBlock as NewContextBlock, ContextMetadata, QueryAnalysisResult,
};
use crate::models::{ContextBlock as LegacyContextBlock, QueryFilters, TimeRange as ModelsTimeRange};
use crate::services::{
    VectorRagClient,
    GraphRagClient,
    QueryAnalyzer,
    ContextBuilder,
};

/// Memory search service that orchestrates retrieval
pub struct MemorySearchService {
    vector_client: VectorRagClient,
    graph_client: GraphRagClient,
    query_analyzer: QueryAnalyzer,
}

impl MemorySearchService {
    /// Create a new memory search service
    pub fn new(vector_client: VectorRagClient, graph_client: GraphRagClient) -> Self {
        Self {
            vector_client,
            graph_client,
            query_analyzer: QueryAnalyzer::new(),
        }
    }
    
    /// Execute a memory search query
    pub async fn search(&self, query: MemoryQuery) -> Result<MemorySearchResponse> {
        let start_time = Instant::now();
        
        info!("🔍 Memory search: '{}'", query.query);
        
        // Step 1: Analyze the query
        let analysis = self.query_analyzer.analyze(&query);
        
        info!(
            "📊 Query analysis: kind={:?}, modality={:?}, strategy={:?}",
            analysis.kind, analysis.modality, analysis.suggested_plan.strategy
        );
        
        // Step 2: Execute retrieval based on strategy
        let (vector_results, graph_results) = self.execute_retrieval(&query, &analysis).await?;
        
        // Step 3: Merge and rank results
        let blocks = ContextBuilder::merge_results(
            vector_results.clone(),
            graph_results.clone(),
            &analysis.suggested_plan,
        );
        
        let took_ms = start_time.elapsed().as_millis() as u64;
        
        info!(
            "✅ Memory search complete: {} blocks in {}ms",
            blocks.len(),
            took_ms
        );
        
        // Build response
        let debug = if query.include_debug {
            Some(DebugInfo {
                modality_hint: analysis.modality,
                collections_searched: analysis.suggested_plan.vector_collections.clone(),
                graph_queries_executed: analysis.suggested_plan.graph_queries.len(),
                vector_results: vector_results.len(),
                graph_results: graph_results.len(),
            })
        } else {
            None
        };
        
        Ok(MemorySearchResponse {
            blocks,
            total_results: (vector_results.len() + graph_results.len()) as u32,
            query_kind: analysis.kind,
            strategy_used: analysis.suggested_plan.strategy,
            took_ms,
            debug,
        })
    }
    
    /// Execute retrieval based on the plan's strategy
    async fn execute_retrieval(
        &self,
        query: &MemoryQuery,
        analysis: &QueryAnalysisResult,
    ) -> Result<(Vec<NewContextBlock>, Vec<NewContextBlock>)> {
        let plan = &analysis.suggested_plan;
        
        match plan.strategy {
            RetrievalStrategy::VectorOnly => {
                let vector_results = self.execute_vector_search(query, plan).await?;
                Ok((vector_results, vec![]))
            }
            
            RetrievalStrategy::GraphOnly => {
                let graph_results = self.execute_graph_search(query, plan).await?;
                Ok((vec![], graph_results))
            }
            
            RetrievalStrategy::Hybrid => {
                // Execute both in parallel
                let vector_future = self.execute_vector_search(query, plan);
                let graph_future = self.execute_graph_search(query, plan);
                
                let (vector_results, graph_results) = tokio::join!(vector_future, graph_future);
                
                Ok((
                    vector_results.unwrap_or_else(|e| {
                        warn!("Vector search failed: {}", e);
                        vec![]
                    }),
                    graph_results.unwrap_or_else(|e| {
                        warn!("Graph search failed: {}", e);
                        vec![]
                    }),
                ))
            }
            
            RetrievalStrategy::VectorThenGraph => {
                // Vector first
                let vector_results = self.execute_vector_search(query, plan).await?;
                
                // Use vector results as seeds for graph expansion
                let seed_ids: Vec<Uuid> = vector_results.iter()
                    .take(5)
                    .filter_map(|b| Uuid::parse_str(&b.source_id).ok())
                    .collect();
                
                let graph_results = if !seed_ids.is_empty() {
                    self.execute_graph_expansion(seed_ids, plan).await.unwrap_or_default()
                } else {
                    vec![]
                };
                
                Ok((vector_results, graph_results))
            }
            
            RetrievalStrategy::GraphThenVector => {
                // Graph first
                let graph_results = self.execute_graph_search(query, plan).await?;
                
                // Use graph results to filter/boost vector search
                // For now, just run vector search normally
                let vector_results = self.execute_vector_search(query, plan).await?;
                
                Ok((vector_results, graph_results))
            }
        }
    }
    
    /// Execute vector search
    async fn execute_vector_search(
        &self,
        query: &MemoryQuery,
        plan: &RetrievalPlan,
    ) -> Result<Vec<NewContextBlock>> {
        info!("📊 Vector search on collections: {:?}", plan.vector_collections);
        
        // Convert to the format expected by vector client
        let filters = QueryFilters {
            source_types: query.sources.clone(),
            repos: query.filters.get("repos")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            channels: query.filters.get("channels")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            time_range: query.time_range.as_ref().map(|tr| ModelsTimeRange {
                from: tr.from,
                to: tr.to,
            }),
            tags: query.filters.get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        };
        
        match self.vector_client.search(&query.query, &filters, plan.max_blocks as usize).await {
            Ok(blocks) => {
                // Convert legacy ContextBlock to new format
                let converted: Vec<NewContextBlock> = blocks.into_iter()
                    .map(|b| {
                        let text = b.content.clone();
                        NewContextBlock {
                            id: b.chunk_id,
                            source_id: b.document_id.to_string(),
                            text: text.clone(),
                            source_type: b.source.source_type,
                            score: b.score,
                            token_count: ContextBuilder::estimate_tokens(&text),
                            metadata: ContextMetadata {
                                source: Some(b.source.external_id),
                                repo: b.source.metadata.get("repo").and_then(|v| v.as_str().map(String::from)),
                                path: b.source.metadata.get("path").and_then(|v| v.as_str().map(String::from)),
                                timestamp: None,
                                robot_id: None,
                                episode_number: None,
                                location: None,
                                extra: b.source.metadata,
                            },
                        }
                    })
                    .collect();
                
                info!("📊 Vector search returned {} results", converted.len());
                Ok(converted)
            }
            Err(e) => {
                error!("Vector search failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Execute graph search
    async fn execute_graph_search(
        &self,
        query: &MemoryQuery,
        plan: &RetrievalPlan,
    ) -> Result<Vec<NewContextBlock>> {
        info!("🕸️ Graph search with {} query specs", plan.graph_queries.len());
        
        // Convert to the format expected by graph client
        let filters = QueryFilters {
            source_types: query.sources.clone(),
            repos: vec![],
            channels: vec![],
            time_range: query.time_range.as_ref().map(|tr| ModelsTimeRange {
                from: tr.from,
                to: tr.to,
            }),
            tags: vec![],
        };
        
        match self.graph_client.search(&query.query, &filters, plan.max_blocks as usize).await {
            Ok(blocks) => {
                // Convert legacy ContextBlock to new format
                let converted: Vec<NewContextBlock> = blocks.into_iter()
                    .map(|b| {
                        let text = b.content.clone();
                        NewContextBlock {
                            id: b.chunk_id,
                            source_id: b.document_id.to_string(),
                            text: text.clone(),
                            source_type: b.source.source_type,
                            score: b.score,
                            token_count: ContextBuilder::estimate_tokens(&text),
                            metadata: ContextMetadata {
                                source: Some(b.source.external_id),
                                repo: b.source.metadata.get("repo").and_then(|v| v.as_str().map(String::from)),
                                path: b.source.metadata.get("path").and_then(|v| v.as_str().map(String::from)),
                                timestamp: None,
                                robot_id: None,
                                episode_number: None,
                                location: None,
                                extra: b.source.metadata,
                            },
                        }
                    })
                    .collect();
                
                info!("🕸️ Graph search returned {} results", converted.len());
                Ok(converted)
            }
            Err(e) => {
                error!("Graph search failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Execute graph expansion from seed nodes
    async fn execute_graph_expansion(
        &self,
        seed_ids: Vec<Uuid>,
        plan: &RetrievalPlan,
    ) -> Result<Vec<NewContextBlock>> {
        info!("🕸️ Graph expansion from {} seeds", seed_ids.len());
        
        let edge_types = plan.graph_queries.first()
            .map(|q| q.edge_types.clone())
            .unwrap_or_else(|| vec!["MENTIONS".to_string(), "BELONGS_TO".to_string()]);
        
        let max_hops = plan.graph_queries.first()
            .map(|q| q.max_hops)
            .unwrap_or(2);
        
        match self.graph_client.expand(
            seed_ids,
            max_hops as usize,
            edge_types,
            plan.max_blocks as usize / 2,
        ).await {
            Ok(blocks) => {
                let converted: Vec<NewContextBlock> = blocks.into_iter()
                    .map(|b| {
                        let text = b.content.clone();
                        NewContextBlock {
                            id: b.chunk_id,
                            source_id: b.document_id.to_string(),
                            text: text.clone(),
                            source_type: b.source.source_type,
                            score: b.score,
                            token_count: ContextBuilder::estimate_tokens(&text),
                            metadata: ContextMetadata {
                                source: Some(b.source.external_id),
                                repo: None,
                                path: None,
                                timestamp: None,
                                robot_id: None,
                                episode_number: None,
                                location: None,
                                extra: b.source.metadata,
                            },
                        }
                    })
                    .collect();
                
                info!("🕸️ Graph expansion returned {} results", converted.len());
                Ok(converted)
            }
            Err(e) => {
                error!("Graph expansion failed: {}", e);
                Err(e)
            }
        }
    }
}

/// Search specifically for robot memory
pub struct RobotMemorySearchService {
    memory_search: MemorySearchService,
}

impl RobotMemorySearchService {
    pub fn new(vector_client: VectorRagClient, graph_client: GraphRagClient) -> Self {
        Self {
            memory_search: MemorySearchService::new(vector_client, graph_client),
        }
    }
    
    /// Search robot memory (episodic + semantic)
    pub async fn search(&self, query: RobotMemoryQuery) -> Result<MemorySearchResponse> {
        info!("🤖 Robot memory search: '{}'", query.query);
        
        // Build sources based on what's requested
        let mut sources = Vec::new();
        if query.include_episodic {
            sources.push("robot_episodic".to_string());
        }
        if query.include_semantic {
            sources.push("robot_semantic".to_string());
        }
        
        // Build filters
        let mut filters = std::collections::HashMap::new();
        filters.insert("robot_id".to_string(), serde_json::json!(query.robot_id.to_string()));
        
        if let Some(ref location) = query.location {
            filters.insert("location".to_string(), serde_json::json!(location));
        }
        
        // Convert to MemoryQuery
        let memory_query = MemoryQuery {
            tenant_id: query.tenant_id,
            user_id: query.robot_id, // Use robot_id as user_id for robot queries
            query: query.query,
            sources,
            time_range: query.time_range,
            filters,
            max_blocks: query.max_blocks,
            max_tokens: 8000,
            force_strategy: None,
            include_debug: false,
        };
        
        self.memory_search.search(memory_query).await
    }
    
    /// Get latest context snapshot for a robot
    pub async fn get_latest_context(&self, robot_id: Uuid, tenant_id: Uuid) -> Result<RobotContextSnapshot> {
        info!("🤖 Getting latest context for robot {}", robot_id);
        
        // Query recent episodes
        let episode_query = RobotMemoryQuery {
            robot_id,
            tenant_id,
            query: "recent activity and observations".to_string(),
            time_range: Some(QueryTimeRange {
                from: chrono::Utc::now() - chrono::Duration::hours(1),
                to: chrono::Utc::now(),
            }),
            location: None,
            include_episodic: true,
            include_semantic: false,
            max_blocks: 5,
        };
        
        let episodes_result = self.search(episode_query).await?;
        
        // Query relevant facts
        let facts_query = RobotMemoryQuery {
            robot_id,
            tenant_id,
            query: "important facts and knowledge".to_string(),
            time_range: None,
            location: None,
            include_episodic: false,
            include_semantic: true,
            max_blocks: 5,
        };
        
        let facts_result = self.search(facts_query).await?;
        
        // Build snapshot
        let recent_episodes: Vec<EpisodeSummary> = episodes_result.blocks.iter()
            .map(|b| EpisodeSummary {
                id: b.id,
                episode_number: b.metadata.episode_number.unwrap_or(0),
                summary: Some(b.text.clone()),
                location_name: b.metadata.location.clone(),
                relevance_score: b.score as f64,
            })
            .collect();
        
        let relevant_facts: Vec<FactSummary> = facts_result.blocks.iter()
            .map(|b| FactSummary {
                id: b.id,
                fact_text: b.text.clone(),
                confidence_score: b.score as f64,
                relevance_score: b.score as f64,
            })
            .collect();
        
        Ok(RobotContextSnapshot {
            robot_id,
            robot_name: format!("Robot {}", robot_id),
            status: RobotStatus::Connected,
            last_heartbeat: Some(chrono::Utc::now()),
            current_location: None,
            current_task: None,
            recent_episodes,
            relevant_facts,
            active_streams: vec![],
            snapshot_time: chrono::Utc::now(),
        })
    }
}

/// Episode summary for robot context
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EpisodeSummary {
    pub id: Uuid,
    pub episode_number: i64,
    pub summary: Option<String>,
    pub location_name: Option<String>,
    pub relevance_score: f64,
}

/// Fact summary for robot context
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FactSummary {
    pub id: Uuid,
    pub fact_text: String,
    pub confidence_score: f64,
    pub relevance_score: f64,
}

/// Robot context snapshot
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RobotContextSnapshot {
    pub robot_id: Uuid,
    pub robot_name: String,
    pub status: RobotStatus,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    pub current_location: Option<LocationCoordinates>,
    pub current_task: Option<String>,
    pub recent_episodes: Vec<EpisodeSummary>,
    pub relevant_facts: Vec<FactSummary>,
    pub active_streams: Vec<String>,
    pub snapshot_time: chrono::DateTime<chrono::Utc>,
}

/// Robot status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RobotStatus {
    Registered,
    Connected,
    Disconnected,
    Error,
}

/// Location coordinates
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocationCoordinates {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub frame: String,
}
