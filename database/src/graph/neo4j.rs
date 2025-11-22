use anyhow::{Result, Context};
use async_trait::async_trait;
use neo4rs::{Graph, query};
use std::sync::Arc;
use uuid::Uuid;
use std::collections::HashMap;

use super::{GraphDb, GraphEntity, GraphRelationship, CanonicalEntity, EntityPath, GraphStatistics};

/// Neo4j implementation of GraphDb
pub struct Neo4jGraphDb {
    graph: Arc<Graph>,
}

impl Neo4jGraphDb {
    /// Create a new Neo4j graph database connection
    pub async fn new(uri: &str, user: &str, password: &str) -> Result<Self> {
        let graph = Graph::new(uri, user, password)
            .await
            .context("Failed to connect to Neo4j")?;
        
        Ok(Self {
            graph: Arc::new(graph),
        })
    }
}

#[async_trait]
impl GraphDb for Neo4jGraphDb {
    async fn initialize(&self) -> Result<()> {
        // Create indexes for faster querying
        let indexes = vec![
            "CREATE INDEX entity_id IF NOT EXISTS FOR (e:Entity) ON (e.id)",
            "CREATE INDEX entity_type IF NOT EXISTS FOR (e:Entity) ON (e.entity_type)",
            "CREATE INDEX entity_source IF NOT EXISTS FOR (e:Entity) ON (e.source)",
            "CREATE INDEX entity_name IF NOT EXISTS FOR (e:Entity) ON (e.name)",
            "CREATE INDEX canonical_id IF NOT EXISTS FOR (c:CanonicalEntity) ON (c.id)",
        ];
        
        for index_query in indexes {
            self.graph.run(query(index_query)).await?;
        }
        
        Ok(())
    }

    async fn insert_entity(&self, entity: &GraphEntity) -> Result<()> {
        let properties_json = serde_json::to_string(&entity.properties)?;
        
        let q = query(
            r#"
            CREATE (e:Entity {
                id: $id,
                entity_type: $entity_type,
                source: $source,
                source_id: $source_id,
                name: $name,
                content: $content,
                properties: $properties,
                created_at: $created_at,
                updated_at: $updated_at
            })
            "#
        )
        .param("id", entity.id.to_string())
        .param("entity_type", entity.entity_type.clone())
        .param("source", entity.source.clone())
        .param("source_id", entity.source_id.clone())
        .param("name", entity.name.clone())
        .param("content", entity.content.clone().unwrap_or_default())
        .param("properties", properties_json)
        .param("created_at", entity.created_at.to_rfc3339())
        .param("updated_at", entity.updated_at.to_rfc3339());
        
        self.graph.run(q).await?;
        Ok(())
    }

    async fn update_entity(&self, entity: &GraphEntity) -> Result<()> {
        let properties_json = serde_json::to_string(&entity.properties)?;
        
        let q = query(
            r#"
            MATCH (e:Entity {id: $id})
            SET e.name = $name,
                e.content = $content,
                e.properties = $properties,
                e.updated_at = $updated_at
            "#
        )
        .param("id", entity.id.to_string())
        .param("name", entity.name.clone())
        .param("content", entity.content.clone().unwrap_or_default())
        .param("properties", properties_json)
        .param("updated_at", entity.updated_at.to_rfc3339());
        
        self.graph.run(q).await?;
        Ok(())
    }

    async fn find_entity(&self, id: Uuid) -> Result<Option<GraphEntity>> {
        let mut result = self.graph.execute(
            query("MATCH (e:Entity {id: $id}) RETURN e")
                .param("id", id.to_string())
        ).await?;
        
        if let Some(row) = result.next().await? {
            let node: neo4rs::Node = row.get("e")?;
            
            let entity = GraphEntity {
                id: Uuid::parse_str(node.get::<String>("id")?.as_str())?,
                entity_type: node.get("entity_type")?,
                source: node.get("source")?,
                source_id: node.get("source_id")?,
                name: node.get("name")?,
                content: node.get::<String>("content").ok(),
                properties: serde_json::from_str(&node.get::<String>("properties")?)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&node.get::<String>("created_at")?)?.with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&node.get::<String>("updated_at")?)?.with_timezone(&chrono::Utc),
            };
            
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    async fn insert_relationship(&self, relationship: &GraphRelationship) -> Result<()> {
        let properties_json = serde_json::to_string(&relationship.properties)?;
        
        // Sanitize relationship type for Cypher (remove special chars, use uppercase)
        let rel_type = relationship.relationship_type
            .to_uppercase()
            .replace("-", "_")
            .replace(" ", "_");
        
        let q = query(&format!(
            r#"
            MATCH (from:Entity {{id: $from_id}})
            MATCH (to:Entity {{id: $to_id}})
            CREATE (from)-[r:{} {{
                id: $id,
                source: $source,
                confidence_score: $confidence_score,
                properties: $properties,
                created_at: $created_at
            }}]->(to)
            "#,
            rel_type
        ))
        .param("from_id", relationship.from_entity_id.to_string())
        .param("to_id", relationship.to_entity_id.to_string())
        .param("id", relationship.id.to_string())
        .param("source", relationship.source.clone())
        .param("confidence_score", relationship.confidence_score as f64)
        .param("properties", properties_json)
        .param("created_at", relationship.created_at.to_rfc3339());
        
        self.graph.run(q).await?;
        Ok(())
    }

    async fn insert_canonical_entity(&self, canonical: &CanonicalEntity) -> Result<()> {
        let properties_json = serde_json::to_string(&canonical.properties)?;
        
        let q = query(
            r#"
            CREATE (c:CanonicalEntity {
                id: $id,
                entity_type: $entity_type,
                canonical_name: $canonical_name,
                properties: $properties,
                confidence_score: $confidence_score,
                source_entity_count: $source_entity_count,
                created_at: $created_at
            })
            "#
        )
        .param("id", canonical.id.to_string())
        .param("entity_type", canonical.entity_type.clone())
        .param("canonical_name", canonical.canonical_name.clone())
        .param("properties", properties_json)
        .param("confidence_score", canonical.confidence_score as f64)
        .param("source_entity_count", canonical.source_entity_count as i64)
        .param("created_at", canonical.created_at.to_rfc3339());
        
        self.graph.run(q).await?;
        Ok(())
    }

    async fn batch_insert_entities(&self, entities: &[GraphEntity]) -> Result<usize> {
        let mut count = 0;
        for entity in entities {
            if self.insert_entity(entity).await.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn batch_insert_relationships(&self, relationships: &[GraphRelationship]) -> Result<usize> {
        let mut count = 0;
        for relationship in relationships {
            if self.insert_relationship(relationship).await.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn find_paths(&self, from_id: Uuid, to_id: Uuid, max_hops: usize) -> Result<Vec<EntityPath>> {
        let q = query(&format!(
            r#"
            MATCH path = (from:Entity {{id: $from_id}})-[*1..{}]-(to:Entity {{id: $to_id}})
            RETURN path
            LIMIT 10
            "#,
            max_hops
        ))
        .param("from_id", from_id.to_string())
        .param("to_id", to_id.to_string());
        
        let mut result = self.graph.execute(q).await?;
        let mut paths = Vec::new();
        
        while let Some(_row) = result.next().await? {
            // Simplified: would need to parse path nodes and relationships
            // For now, return empty paths
        }
        
        Ok(paths)
    }

    async fn get_statistics(&self) -> Result<GraphStatistics> {
        // Count total entities
        let mut result = self.graph.execute(
            query("MATCH (e:Entity) RETURN count(e) as count")
        ).await?;
        
        let total_entities = if let Some(row) = result.next().await? {
            row.get::<i64>("count")? as usize
        } else {
            0
        };
        
        // Count total relationships
        let mut result = self.graph.execute(
            query("MATCH ()-[r]->() RETURN count(r) as count")
        ).await?;
        
        let total_relationships = if let Some(row) = result.next().await? {
            row.get::<i64>("count")? as usize
        } else {
            0
        };
        
        // Count by type
        let mut result = self.graph.execute(
            query("MATCH (e:Entity) RETURN e.entity_type as type, count(e) as count")
        ).await?;
        
        let mut entities_by_type = HashMap::new();
        while let Some(row) = result.next().await? {
            let entity_type: String = row.get("type")?;
            let count: i64 = row.get("count")?;
            entities_by_type.insert(entity_type, count as usize);
        }
        
        // Count by source
        let mut result = self.graph.execute(
            query("MATCH (e:Entity) RETURN e.source as source, count(e) as count")
        ).await?;
        
        let mut entities_by_source = HashMap::new();
        while let Some(row) = result.next().await? {
            let source: String = row.get("source")?;
            let count: i64 = row.get("count")?;
            entities_by_source.insert(source, count as usize);
        }
        
        Ok(GraphStatistics {
            total_entities,
            total_relationships,
            entities_by_type,
            entities_by_source,
        })
    }
}
