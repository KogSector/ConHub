//! Relation Builder
//! 
//! Builds relations and semantic facts from robot episodes and other data.
//! This is the "inference" layer that extracts higher-level knowledge from raw events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::robot_memory::{EpisodeMessage, SemanticEventMessage};

// ============================================================================
// RELATION TYPES
// ============================================================================

/// A relation extracted from episode data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// Unique ID for this relation
    pub id: Uuid,
    
    /// Type of relation
    pub relation_type: RelationType,
    
    /// Subject of the relation
    pub subject: String,
    
    /// Predicate (relationship)
    pub predicate: String,
    
    /// Object of the relation
    pub object: String,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    
    /// Number of observations supporting this relation
    pub evidence_count: u32,
    
    /// First time this relation was observed
    pub first_observed_at: DateTime<Utc>,
    
    /// Most recent observation
    pub last_observed_at: DateTime<Utc>,
    
    /// Source episode IDs
    pub source_episodes: Vec<Uuid>,
    
    /// Robot ID this relation belongs to
    pub robot_id: Uuid,
    
    /// Tenant ID
    pub tenant_id: Uuid,
}

/// Types of relations we extract
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// Object is usually at location
    ObjectLocation,
    
    /// Person is usually at location
    PersonLocation,
    
    /// Object is related to another object
    ObjectRelation,
    
    /// Task requires certain objects
    TaskObject,
    
    /// Task is done at certain location
    TaskLocation,
    
    /// Route connects locations
    RouteConnection,
    
    /// Temporal pattern (e.g., activity happens at certain times)
    TemporalPattern,
    
    /// Custom/other
    Custom,
}

// ============================================================================
// GRAPH OPERATIONS
// ============================================================================

/// A node to upsert into the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// An edge to upsert into the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from_id: String,
    pub to_id: String,
    pub edge_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Batch of graph operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphBatch {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

// ============================================================================
// RELATION BUILDER
// ============================================================================

/// Builds relations and graph structure from episodes
pub struct RelationBuilder {
    /// Graph RAG service URL
    graph_rag_url: String,
    
    /// HTTP client
    http_client: reqwest::Client,
    
    /// Minimum confidence threshold for relations
    min_confidence: f64,
    
    /// Enable graph updates
    graph_enabled: bool,
}

impl RelationBuilder {
    pub fn new(graph_rag_url: String) -> Self {
        let graph_enabled = std::env::var("GRAPH_RAG_ENABLED")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        
        Self {
            graph_rag_url,
            http_client: reqwest::Client::new(),
            min_confidence: 0.3,
            graph_enabled,
        }
    }
    
    /// Process an episode and extract relations
    pub async fn process_episode(&self, episode: &EpisodeMessage) -> Vec<Relation> {
        info!(
            "🔗 Processing episode {} for relations (robot: {})",
            episode.episode_number, episode.robot_id
        );
        
        let mut relations = Vec::new();
        
        // Extract object-location relations
        if let Some(ref location_id) = episode.location_id {
            for object in &episode.objects_seen {
                let relation = Relation {
                    id: Uuid::new_v4(),
                    relation_type: RelationType::ObjectLocation,
                    subject: object.clone(),
                    predicate: "seen_at".to_string(),
                    object: location_id.clone(),
                    confidence: episode.confidence_score.unwrap_or(0.5),
                    evidence_count: 1,
                    first_observed_at: episode.started_at,
                    last_observed_at: episode.ended_at,
                    source_episodes: vec![],
                    robot_id: episode.robot_id,
                    tenant_id: episode.tenant_id,
                };
                relations.push(relation);
            }
            
            // Person-location relations
            for person in &episode.people_involved {
                let relation = Relation {
                    id: Uuid::new_v4(),
                    relation_type: RelationType::PersonLocation,
                    subject: person.clone(),
                    predicate: "seen_at".to_string(),
                    object: location_id.clone(),
                    confidence: episode.confidence_score.unwrap_or(0.5),
                    evidence_count: 1,
                    first_observed_at: episode.started_at,
                    last_observed_at: episode.ended_at,
                    source_episodes: vec![],
                    robot_id: episode.robot_id,
                    tenant_id: episode.tenant_id,
                };
                relations.push(relation);
            }
            
            // Task-location relations
            for task in &episode.tasks_related {
                let relation = Relation {
                    id: Uuid::new_v4(),
                    relation_type: RelationType::TaskLocation,
                    subject: task.clone(),
                    predicate: "performed_at".to_string(),
                    object: location_id.clone(),
                    confidence: episode.confidence_score.unwrap_or(0.5),
                    evidence_count: 1,
                    first_observed_at: episode.started_at,
                    last_observed_at: episode.ended_at,
                    source_episodes: vec![],
                    robot_id: episode.robot_id,
                    tenant_id: episode.tenant_id,
                };
                relations.push(relation);
            }
        }
        
        // Task-object relations
        for task in &episode.tasks_related {
            for object in &episode.objects_seen {
                let relation = Relation {
                    id: Uuid::new_v4(),
                    relation_type: RelationType::TaskObject,
                    subject: task.clone(),
                    predicate: "involves".to_string(),
                    object: object.clone(),
                    confidence: episode.confidence_score.unwrap_or(0.5) * 0.8, // Lower confidence
                    evidence_count: 1,
                    first_observed_at: episode.started_at,
                    last_observed_at: episode.ended_at,
                    source_episodes: vec![],
                    robot_id: episode.robot_id,
                    tenant_id: episode.tenant_id,
                };
                relations.push(relation);
            }
        }
        
        info!("📊 Extracted {} relations from episode", relations.len());
        
        // Build graph batch
        if self.graph_enabled {
            let batch = self.build_graph_batch(episode);
            if let Err(e) = self.send_graph_batch(batch).await {
                warn!("Failed to send graph batch: {}", e);
            }
        }
        
        relations
    }
    
    /// Build graph nodes and edges from an episode
    fn build_graph_batch(&self, episode: &EpisodeMessage) -> GraphBatch {
        let mut batch = GraphBatch::default();
        
        // Robot node
        batch.nodes.push(GraphNode {
            id: format!("robot:{}", episode.robot_id),
            node_type: "Robot".to_string(),
            name: format!("Robot {}", episode.robot_id),
            properties: {
                let mut props = HashMap::new();
                props.insert("tenant_id".to_string(), serde_json::json!(episode.tenant_id.to_string()));
                props
            },
        });
        
        // Episode node
        let episode_id = format!("episode:{}:{}", episode.robot_id, episode.episode_number);
        batch.nodes.push(GraphNode {
            id: episode_id.clone(),
            node_type: "Episode".to_string(),
            name: format!("Episode {}", episode.episode_number),
            properties: {
                let mut props = HashMap::new();
                props.insert("summary".to_string(), serde_json::json!(episode.summary));
                props.insert("episode_type".to_string(), serde_json::json!(episode.episode_type));
                props.insert("started_at".to_string(), serde_json::json!(episode.started_at.to_rfc3339()));
                props.insert("ended_at".to_string(), serde_json::json!(episode.ended_at.to_rfc3339()));
                props.insert("duration_ms".to_string(), serde_json::json!(episode.duration_ms));
                if let Some(conf) = episode.confidence_score {
                    props.insert("confidence".to_string(), serde_json::json!(conf));
                }
                props
            },
        });
        
        // Robot -> Episode edge
        batch.edges.push(GraphEdge {
            from_id: format!("robot:{}", episode.robot_id),
            to_id: episode_id.clone(),
            edge_type: "HAD_EPISODE".to_string(),
            properties: HashMap::new(),
        });
        
        // Location node and edge
        if let Some(ref location_id) = episode.location_id {
            batch.nodes.push(GraphNode {
                id: format!("location:{}", location_id),
                node_type: "Location".to_string(),
                name: episode.location_name.clone().unwrap_or_else(|| location_id.clone()),
                properties: HashMap::new(),
            });
            
            batch.edges.push(GraphEdge {
                from_id: episode_id.clone(),
                to_id: format!("location:{}", location_id),
                edge_type: "AT_LOCATION".to_string(),
                properties: HashMap::new(),
            });
        }
        
        // Object nodes and edges
        for object in &episode.objects_seen {
            batch.nodes.push(GraphNode {
                id: format!("object:{}", object),
                node_type: "Object".to_string(),
                name: object.clone(),
                properties: HashMap::new(),
            });
            
            batch.edges.push(GraphEdge {
                from_id: episode_id.clone(),
                to_id: format!("object:{}", object),
                edge_type: "SAW_OBJECT".to_string(),
                properties: HashMap::new(),
            });
        }
        
        // Person nodes and edges
        for person in &episode.people_involved {
            batch.nodes.push(GraphNode {
                id: format!("person:{}", person),
                node_type: "Person".to_string(),
                name: person.clone(),
                properties: HashMap::new(),
            });
            
            batch.edges.push(GraphEdge {
                from_id: episode_id.clone(),
                to_id: format!("person:{}", person),
                edge_type: "INVOLVED_PERSON".to_string(),
                properties: HashMap::new(),
            });
        }
        
        // Task nodes and edges
        for task in &episode.tasks_related {
            batch.nodes.push(GraphNode {
                id: format!("task:{}", task),
                node_type: "Task".to_string(),
                name: task.clone(),
                properties: HashMap::new(),
            });
            
            batch.edges.push(GraphEdge {
                from_id: episode_id.clone(),
                to_id: format!("task:{}", task),
                edge_type: "RELATED_TO_TASK".to_string(),
                properties: HashMap::new(),
            });
        }
        
        debug!(
            "Built graph batch: {} nodes, {} edges",
            batch.nodes.len(),
            batch.edges.len()
        );
        
        batch
    }
    
    /// Send graph batch to graph_rag service
    async fn send_graph_batch(&self, batch: GraphBatch) -> Result<(), anyhow::Error> {
        if batch.nodes.is_empty() && batch.edges.is_empty() {
            return Ok(());
        }
        
        info!(
            "📤 Sending graph batch: {} nodes, {} edges",
            batch.nodes.len(),
            batch.edges.len()
        );
        
        // Send nodes
        if !batch.nodes.is_empty() {
            let url = format!("{}/api/graph/nodes/batch", self.graph_rag_url);
            let response = self.http_client
                .post(&url)
                .json(&batch.nodes)
                .send()
                .await?;
            
            if !response.status().is_success() {
                let status = response.status();
                let error = response.text().await.unwrap_or_default();
                warn!("Graph node upsert failed: {} - {}", status, error);
            }
        }
        
        // Send edges
        if !batch.edges.is_empty() {
            let url = format!("{}/api/graph/edges/batch", self.graph_rag_url);
            let response = self.http_client
                .post(&url)
                .json(&batch.edges)
                .send()
                .await?;
            
            if !response.status().is_success() {
                let status = response.status();
                let error = response.text().await.unwrap_or_default();
                warn!("Graph edge upsert failed: {} - {}", status, error);
            }
        }
        
        Ok(())
    }
    
    /// Convert a relation to a semantic fact for storage
    pub fn relation_to_semantic_fact(&self, relation: &Relation) -> SemanticFact {
        SemanticFact {
            id: relation.id,
            robot_id: relation.robot_id,
            tenant_id: relation.tenant_id,
            fact_type: format!("{:?}", relation.relation_type).to_lowercase(),
            subject: relation.subject.clone(),
            predicate: relation.predicate.clone(),
            object_value: Some(relation.object.clone()),
            confidence_score: relation.confidence,
            evidence_count: relation.evidence_count as i32,
            first_observed_at: relation.first_observed_at,
            last_observed_at: relation.last_observed_at,
            source_episode_ids: relation.source_episodes.clone(),
            natural_language: format!(
                "{} {} {}",
                relation.subject, relation.predicate, relation.object
            ),
        }
    }
    
    /// Merge new evidence into an existing relation
    pub fn merge_evidence(&self, existing: &mut Relation, new_episode: &EpisodeMessage) {
        existing.evidence_count += 1;
        existing.last_observed_at = new_episode.ended_at;
        
        // Update confidence based on evidence count (diminishing returns)
        let base_confidence = new_episode.confidence_score.unwrap_or(0.5);
        let evidence_boost = 1.0 - (1.0 / (existing.evidence_count as f64 + 1.0));
        existing.confidence = (existing.confidence + base_confidence * evidence_boost) / 2.0;
        existing.confidence = existing.confidence.min(0.99); // Cap at 0.99
    }
}

/// Semantic fact for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFact {
    pub id: Uuid,
    pub robot_id: Uuid,
    pub tenant_id: Uuid,
    pub fact_type: String,
    pub subject: String,
    pub predicate: String,
    pub object_value: Option<String>,
    pub confidence_score: f64,
    pub evidence_count: i32,
    pub first_observed_at: DateTime<Utc>,
    pub last_observed_at: DateTime<Utc>,
    pub source_episode_ids: Vec<Uuid>,
    pub natural_language: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_relation_extraction() {
        let episode = EpisodeMessage {
            robot_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            episode_number: 1,
            episode_type: "navigation".to_string(),
            started_at: Utc::now(),
            ended_at: Utc::now(),
            duration_ms: 5000,
            summary: "Test episode".to_string(),
            detailed_description: None,
            location_id: Some("warehouse_a".to_string()),
            location_name: Some("Warehouse A".to_string()),
            location_coordinates: None,
            objects_seen: vec!["box_1".to_string(), "pallet_2".to_string()],
            people_involved: vec!["operator_1".to_string()],
            tasks_related: vec!["delivery_123".to_string()],
            observations_count: 10,
            actions_count: 5,
            outcome: None,
            confidence_score: Some(0.9),
        };
        
        let builder = RelationBuilder::new("http://localhost:3015".to_string());
        
        // Use tokio::test to run async
        let rt = tokio::runtime::Runtime::new().unwrap();
        let relations = rt.block_on(builder.process_episode(&episode));
        
        // Should have object-location, person-location, task-location, and task-object relations
        assert!(relations.len() >= 4);
        
        // Check for object-location relation
        let obj_loc = relations.iter()
            .find(|r| r.relation_type == RelationType::ObjectLocation && r.subject == "box_1");
        assert!(obj_loc.is_some());
    }
}
