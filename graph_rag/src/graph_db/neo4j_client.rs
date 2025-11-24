use crate::errors::{GraphError, GraphResult};
use neo4rs::{Graph, query};
use std::sync::Arc;

pub struct Neo4jClient {
    graph: Arc<Graph>,
}

impl Neo4jClient {
    pub async fn new(uri: &str, user: &str, password: &str) -> GraphResult<Self> {
        let graph = Graph::new(uri, user, password)
            .await
            .map_err(|e| GraphError::Neo4j(e.to_string()))?;
        
        Ok(Self {
            graph: Arc::new(graph),
        })
    }

    pub async fn create_node(
        &self,
        label: &str,
        properties: serde_json::Value,
    ) -> GraphResult<String> {
        let props_str = properties.to_string();
        let cypher = format!("CREATE (n:{} $props) RETURN id(n) as node_id", label);
        
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

    pub async fn create_relationship(
        &self,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
        properties: serde_json::Value,
    ) -> GraphResult<()> {
        let props_str = properties.to_string();
        let cypher = format!(
            "MATCH (a), (b) WHERE id(a) = $from_id AND id(b) = $to_id \
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
}
