use std::sync::Arc;
use serde_json::Value;
use tracing::{info, error};
use uuid::Uuid;

use super::zilliz_client::{ZillizClient, ZillizSearchResult, ZillizConfig, build_zilliz_filter};

/// Vector store service that wraps Zilliz client for embedding storage and retrieval
#[derive(Clone)]
pub struct VectorStoreService {
    zilliz_client: Arc<ZillizClient>,
    vector_dimension: usize,
}

impl VectorStoreService {
    /// Create a new vector store service
    pub async fn new(config: ZillizConfig, vector_dimension: usize) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let zilliz_client = Arc::new(ZillizClient::new(config));
        
        // Ensure collection exists
        zilliz_client.ensure_collection(vector_dimension).await?;
        
        Ok(Self {
            zilliz_client,
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

        self.zilliz_client.store_vector(id, vector, metadata).await
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

        self.zilliz_client.store_vectors_batch(vectors).await
    }

    /// Search for similar vectors
    pub async fn search_similar(
        &self,
        query_vector: &[f32],
        limit: usize,
        with_payload: bool,
    ) -> Result<Vec<ZillizSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        if query_vector.len() != self.vector_dimension {
            return Err(format!(
                "Query vector dimension mismatch: expected {}, got {}",
                self.vector_dimension,
                query_vector.len()
            ).into());
        }

        self.zilliz_client.search_vectors(query_vector, limit, with_payload).await
    }

    /// Search for similar vectors with filtering
    pub async fn search_similar_with_filter(
        &self,
        query_vector: &[f32],
        limit: usize,
        filter: Value,
    ) -> Result<Vec<ZillizSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        if query_vector.len() != self.vector_dimension {
            return Err(format!(
                "Query vector dimension mismatch: expected {}, got {}",
                self.vector_dimension,
                query_vector.len()
            ).into());
        }

        // Build Zilliz filter from the filter Value
        let filter_str = if let Value::Object(filter_map) = &filter {
            let mut conditions = Vec::new();
            for (key, value) in filter_map {
                if let Some(v) = value.as_str() {
                    conditions.push(format!("{} == \"{}\"", key, v));
                }
            }
            if conditions.is_empty() { None } else { Some(conditions.join(" && ")) }
        } else {
            None
        };

        self.zilliz_client.search_vectors_with_filter(query_vector, limit, filter_str, true).await
    }

    /// Delete vectors by repository (useful for re-syncing)
    pub async fn delete_repository_vectors(
        &self,
        repository: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filter = format!("repository == \"{}\"", repository);
        self.zilliz_client.delete_vectors_by_filter(filter).await
    }

    /// Delete vectors by branch (useful for branch-specific cleanup)
    pub async fn delete_branch_vectors(
        &self,
        repository: &str,
        branch: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filter = format!("repository == \"{}\" && branch == \"{}\"", repository, branch);
        self.zilliz_client.delete_vectors_by_filter(filter).await
    }

    /// Delete a specific vector by ID
    pub async fn delete_vector(
        &self,
        id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.zilliz_client.delete_vector(id).await
    }

    /// Get collection statistics
    pub async fn get_stats(&self) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let info = self.zilliz_client.get_collection_info().await?;
        
        Ok(serde_json::json!({
            "status": info.status,
            "vectors_count": info.vectors_count,
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
        // Build Zilliz filter
        let filter = build_zilliz_filter(repository, branch, languages);

        let results = self.zilliz_client.search_vectors_with_filter(query_vector, limit, filter, true).await?;
        
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
    let config = ZillizConfig::from_env();

    // Default embedding dimension (this should match your model)
    let vector_dimension: usize = std::env::var("EMBEDDING_DIMENSION")
        .unwrap_or_else(|_| "1536".to_string())
        .parse()
        .unwrap_or(1536);

    VectorStoreService::new(config, vector_dimension).await
}
