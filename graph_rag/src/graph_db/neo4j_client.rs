use crate::errors::{GraphError, GraphResult};
use neo4rs::{Graph, query, ConfigBuilder};
use std::sync::Arc;

/// Neo4j client compatible with both local Neo4j and Neo4j AuraDB
pub struct Neo4jClient {
    graph: Arc<Graph>,
    uri: String,
}

impl Neo4jClient {
    /// Create a new Neo4j client
    /// 
    /// # Arguments
    /// * `uri` - Neo4j connection URI. Supports:
    ///   - Local: `bolt://localhost:7687`
    ///   - AuraDB: `neo4j+s://xxxxx.databases.neo4j.io` or `neo4j+ssc://...`
    /// * `user` - Database username
    /// * `password` - Database password
    pub async fn new(uri: &str, user: &str, password: &str) -> GraphResult<Self> {
        tracing::info!("🔷 Connecting to Neo4j at: {}", uri);
        
        // Use ConfigBuilder for more control over connection settings
        // This works with both local Neo4j and AuraDB
        let config = ConfigBuilder::default()
            .uri(uri)
            .user(user)
            .password(password)
            .db("neo4j") // Default database
            .fetch_size(500)
            .max_connections(10)
            .build()
            .map_err(|e| GraphError::Neo4j(format!("Failed to build Neo4j config: {}", e)))?;
        
        let graph = Graph::connect(config)
            .await
            .map_err(|e| GraphError::Neo4j(format!("Failed to connect to Neo4j: {}", e)))?;
        
        // Test the connection
        let mut result = graph.execute(query("RETURN 1 as test"))
            .await
            .map_err(|e| GraphError::Neo4j(format!("Connection test failed: {}", e)))?;
        
        if result.next().await.map_err(|e| GraphError::Neo4j(e.to_string()))?.is_some() {
            tracing::info!("✅ Neo4j connection established successfully");
        }
        
        Ok(Self {
            graph: Arc::new(graph),
            uri: uri.to_string(),
        })
    }

    /// Get the connection URI
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Check if connected to AuraDB
    pub fn is_aura(&self) -> bool {
        self.uri.contains("neo4j.io") || self.uri.starts_with("neo4j+s://") || self.uri.starts_with("neo4j+ssc://")
    }

    /// Create a node with the given label and properties
    pub async fn create_node(
        &self,
        label: &str,
        properties: serde_json::Value,
    ) -> GraphResult<String> {
        let props_str = properties.to_string();
        let cypher = format!(
            "CREATE (n:{} $props) RETURN elementId(n) as node_id",
            label
        );
        
        let mut result = self.graph.execute(query(&cypher).param("props", props_str))
            .await
            .map_err(|e| GraphError::Neo4j(e.to_string()))?;
        
        if let Some(row) = result.next().await.map_err(|e| GraphError::Neo4j(e.to_string()))? {
            let node_id: String = row.get("node_id")
                .map_err(|e| GraphError::Neo4j(e.to_string()))?;
            Ok(node_id)
        } else {
            Err(GraphError::Neo4j("Failed to create node".to_string()))
        }
    }

    /// Create a relationship between two nodes
    pub async fn create_relationship(
        &self,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
        properties: serde_json::Value,
    ) -> GraphResult<()> {
        let props_str = properties.to_string();
        let cypher = format!(
            "MATCH (a), (b) WHERE elementId(a) = $from_id AND elementId(b) = $to_id \
             CREATE (a)-[r:{} $props]->(b)",
            rel_type
        );
        
        self.graph.execute(
            query(&cypher)
                .param("from_id", from_id)
                .param("to_id", to_id)
                .param("props", props_str)
        )
        .await
        .map_err(|e| GraphError::Neo4j(e.to_string()))?;
        
        Ok(())
    }

    /// Find a node by label and property
    pub async fn find_node(
        &self,
        label: &str,
        property_name: &str,
        property_value: &str,
    ) -> GraphResult<Option<String>> {
        let cypher = format!(
            "MATCH (n:{} {{{}: $value}}) RETURN elementId(n) as node_id LIMIT 1",
            label, property_name
        );
        
        let mut result = self.graph.execute(query(&cypher).param("value", property_value))
            .await
            .map_err(|e| GraphError::Neo4j(e.to_string()))?;
        
        if let Some(row) = result.next().await.map_err(|e| GraphError::Neo4j(e.to_string()))? {
            let node_id: String = row.get("node_id")
                .map_err(|e| GraphError::Neo4j(e.to_string()))?;
            Ok(Some(node_id))
        } else {
            Ok(None)
        }
    }

    /// Execute a raw Cypher query
    pub async fn execute_cypher(&self, cypher: &str) -> GraphResult<Vec<serde_json::Value>> {
        let mut result = self.graph.execute(query(cypher))
            .await
            .map_err(|e| GraphError::Neo4j(e.to_string()))?;
        
        let mut rows = Vec::new();
        while let Some(row) = result.next().await.map_err(|e| GraphError::Neo4j(e.to_string()))? {
            // Convert row to JSON - simplified approach
            rows.push(serde_json::json!({
                "row": format!("{:?}", row)
            }));
        }
        
        Ok(rows)
    }

    /// Get neighbors of a node
    pub async fn get_neighbors(
        &self,
        node_id: &str,
        relationship_type: Option<&str>,
        direction: Option<&str>,
    ) -> GraphResult<Vec<String>> {
        let rel_pattern = relationship_type
            .map(|t| format!(":{}", t))
            .unwrap_or_default();
        
        let direction_pattern = match direction.unwrap_or("both") {
            "outgoing" => format!("-[r{}]->", rel_pattern),
            "incoming" => format!("<-[r{}]-", rel_pattern),
            _ => format!("-[r{}]-", rel_pattern),
        };
        
        let cypher = format!(
            "MATCH (a){}(b) WHERE elementId(a) = $node_id RETURN elementId(b) as neighbor_id",
            direction_pattern
        );
        
        let mut result = self.graph.execute(query(&cypher).param("node_id", node_id))
            .await
            .map_err(|e| GraphError::Neo4j(e.to_string()))?;
        
        let mut neighbors = Vec::new();
        while let Some(row) = result.next().await.map_err(|e| GraphError::Neo4j(e.to_string()))? {
            if let Ok(neighbor_id) = row.get::<String>("neighbor_id") {
                neighbors.push(neighbor_id);
            }
        }
        
        Ok(neighbors)
    }

    /// Delete a node by ID
    pub async fn delete_node(&self, node_id: &str) -> GraphResult<()> {
        let cypher = "MATCH (n) WHERE elementId(n) = $node_id DETACH DELETE n";
        
        self.graph.execute(query(cypher).param("node_id", node_id))
            .await
            .map_err(|e| GraphError::Neo4j(e.to_string()))?;
        
        Ok(())
    }

    /// Count nodes with a specific label
    pub async fn count_nodes(&self, label: &str) -> GraphResult<i64> {
        let cypher = format!("MATCH (n:{}) RETURN count(n) as count", label);
        
        let mut result = self.graph.execute(query(&cypher))
            .await
            .map_err(|e| GraphError::Neo4j(e.to_string()))?;
        
        if let Some(row) = result.next().await.map_err(|e| GraphError::Neo4j(e.to_string()))? {
            let count: i64 = row.get("count")
                .map_err(|e| GraphError::Neo4j(e.to_string()))?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> GraphResult<serde_json::Value> {
        let node_count = self.count_nodes("").await.unwrap_or(0);
        
        Ok(serde_json::json!({
            "connected": true,
            "uri": self.uri,
            "is_aura": self.is_aura(),
            "node_count": node_count
        }))
    }
}
