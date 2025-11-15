use std::sync::Arc;
use serde_json::Value;
use tracing::{info, error};
use uuid::Uuid;

use crate::services::qdrant_client::{QdrantClient, QdrantSearchResult};
use crate::connectors::types::QdrantConfig;

/// Vector store service that wraps Qdrant client for embedding storage and retrieval
#[derive(Clone)]
pub struct VectorStoreService {
    qdrant_client: Arc<QdrantClient>,
    vector_dimension: usize,
}

impl VectorStoreService {
    /// Create a new vector store service
    pub async fn new(config: QdrantConfig, vector_dimension: usize) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let qdrant_client = Arc::new(QdrantClient::new(config));
        
        // Ensure collection exists
        qdrant_client.ensure_collection(vector_dimension).await?;
        
        Ok(Self {
            qdrant_client,
            vector_dimension,
        })
    }

    /// Store a single vector with metadata
    pub async fn store_vector(
        &self,
        id: &str,
        vector: &[f32],
        metadata: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if vector.len() != self.vector_dimension {
            return Err(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.vector_dimension,
                vector.len()
            ).into());
        }

        self.qdrant_client.store_vector(id, vector, metadata).await
    }

    /// Store multiple vectors in batch for better performance
    pub async fn store_vectors_batch(
        &self,
        vectors: Vec<(String, Vec<f32>, Value)>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Validate all vectors have correct dimension
        for (id, vector, _) in &vectors {
            if vector.len() != self.vector_dimension {
                return Err(format!(
                    "Vector dimension mismatch for {}: expected {}, got {}",
                    id,
                    self.vector_dimension,
                    vector.len()
                ).into());
            }
        }

        self.qdrant_client.store_vectors_batch(vectors).await
    }

    /// Search for similar vectors
    pub async fn search_similar(
        &self,
        query_vector: &[f32],
        limit: usize,
        with_payload: bool,
    ) -> Result<Vec<QdrantSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        if query_vector.len() != self.vector_dimension {
            return Err(format!(
                "Query vector dimension mismatch: expected {}, got {}",
                self.vector_dimension,
                query_vector.len()
            ).into());
        }

        self.qdrant_client.search_vectors(query_vector, limit, with_payload).await
    }

    /// Search for similar vectors with filtering
    pub async fn search_similar_with_filter(
        &self,
        query_vector: &[f32],
        limit: usize,
        filter: Value,
    ) -> Result<Vec<QdrantSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        if query_vector.len() != self.vector_dimension {
            return Err(format!(
                "Query vector dimension mismatch: expected {}, got {}",
                self.vector_dimension,
                query_vector.len()
            ).into());
        }

        // For now, we'll do a simple search and filter results
        // In a production system, you'd want to use Qdrant's native filtering
        let results = self.qdrant_client.search_vectors(query_vector, limit * 2, true).await?;
        
        // Apply basic filtering (this is a simplified implementation)
        let filtered_results: Vec<QdrantSearchResult> = results
            .into_iter()
            .filter(|result| {
                if let Some(ref payload) = result.payload {
                    // Simple filter matching - in production, use proper filter logic
                    if let Value::Object(filter_map) = &filter {
                        for (key, expected_value) in filter_map {
                            if let Some(actual_value) = payload.get(key) {
                                if actual_value != expected_value {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                    }
                }
                true
            })
            .take(limit)
            .collect();

        Ok(filtered_results)
    }

    /// Delete vectors by repository (useful for re-syncing)
    pub async fn delete_repository_vectors(
        &self,
        repository: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filter = serde_json::json!({
            "must": [{
                "key": "repository",
                "match": {
                    "value": repository
                }
            }]
        });

        self.qdrant_client.delete_vectors_by_filter(filter).await
    }

    /// Delete vectors by branch (useful for branch-specific cleanup)
    pub async fn delete_branch_vectors(
        &self,
        repository: &str,
        branch: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filter = serde_json::json!({
            "must": [
                {
                    "key": "repository",
                    "match": {
                        "value": repository
                    }
                },
                {
                    "key": "branch",
                    "match": {
                        "value": branch
                    }
                }
            ]
        });

        self.qdrant_client.delete_vectors_by_filter(filter).await
    }

    /// Delete a specific vector by ID
    pub async fn delete_vector(
        &self,
        id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.qdrant_client.delete_vector(id).await
    }

    /// Get collection statistics
    pub async fn get_stats(&self) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let info = self.qdrant_client.get_collection_info().await?;
        
        Ok(serde_json::json!({
            "status": info.status,
            "vectors_count": info.vectors_count,
            "indexed_vectors_count": info.indexed_vectors_count,
            "vector_dimension": self.vector_dimension
        }))
    }

    /// Search for code snippets by content similarity
    pub async fn search_code_similarity(
        &self,
        query_vector: &[f32],
        repository: Option<&str>,
        branch: Option<&str>,
        languages: Option<&[String]>,
        limit: usize,
    ) -> Result<Vec<CodeSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut filter_conditions = Vec::new();

        if let Some(repo) = repository {
            filter_conditions.push(serde_json::json!({
                "key": "repository",
                "match": {
                    "value": repo
                }
            }));
        }

        if let Some(branch_name) = branch {
            filter_conditions.push(serde_json::json!({
                "key": "branch",
                "match": {
                    "value": branch_name
                }
            }));
        }

        if let Some(langs) = languages {
            if !langs.is_empty() {
                let language_filter = serde_json::json!({
                    "key": "content_type",
                    "match": {
                        "any": langs
                    }
                });
                filter_conditions.push(language_filter);
            }
        }

        let filter = if filter_conditions.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::json!({
                "must": filter_conditions
            })
        };

        let results = self.search_similar_with_filter(query_vector, limit, filter).await?;
        
        let code_results: Vec<CodeSearchResult> = results
            .into_iter()
            .map(|result| CodeSearchResult {
                id: result.id,
                score: result.score,
                document_name: result.payload
                    .as_ref()
                    .and_then(|p| p.get("document_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                document_path: result.payload
                    .as_ref()
                    .and_then(|p| p.get("document_path"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                repository: result.payload
                    .as_ref()
                    .and_then(|p| p.get("repository"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                branch: result.payload
                    .as_ref()
                    .and_then(|p| p.get("branch"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                chunk_number: result.payload
                    .as_ref()
                    .and_then(|p| p.get("chunk_number"))
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize),
                url: result.payload
                    .as_ref()
                    .and_then(|p| p.get("url"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
            .collect();

        Ok(code_results)
    }
}

/// Result structure for code search
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeSearchResult {
    pub id: String,
    pub score: f32,
    pub document_name: String,
    pub document_path: Option<String>,
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub chunk_number: Option<usize>,
    pub url: Option<String>,
}

/// Create vector store service from environment variables
pub async fn create_vector_store_service() -> Result<VectorStoreService, Box<dyn std::error::Error + Send + Sync>> {
    let config = QdrantConfig {
        url: std::env::var("QDRANT_URL")
            .unwrap_or_else(|_| "https://your-cluster-host:6333".to_string()),
        api_key: std::env::var("QDRANT_API_KEY")
            .unwrap_or_else(|_| "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2Nlc3MiOiJtIn0.uIPbGMwsHdiQlzUd5ad4Yx1HWPhg1hfbu3fiHRkGr6M".to_string()),
        collection_name: std::env::var("QDRANT_COLLECTION")
            .unwrap_or_else(|_| "conhub_embeddings".to_string()),
    };

    // Default Qwen3 embedding dimension (this should match your model)
    let vector_dimension: usize = std::env::var("EMBEDDING_DIMENSION")
        .unwrap_or_else(|_| "1536".to_string())
        .parse()
        .unwrap_or(1536);

    VectorStoreService::new(config, vector_dimension).await
}
