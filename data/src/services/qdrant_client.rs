use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::connectors::types::QdrantConfig;

#[derive(Debug, Clone)]
pub struct QdrantClient {
    client: Client,
    config: QdrantConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantUpsertRequest {
    pub points: Vec<QdrantPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantSearchRequest {
    pub vector: Vec<f32>,
    pub limit: usize,
    pub with_payload: bool,
    pub with_vector: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantSearchResponse {
    pub result: Vec<QdrantSearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantSearchResult {
    pub id: String,
    pub score: f32,
    pub payload: Option<HashMap<String, Value>>,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantCollectionInfo {
    pub status: String,
    pub vectors_count: Option<u64>,
    pub indexed_vectors_count: Option<u64>,
}

impl QdrantClient {
    pub fn new(config: QdrantConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Create a new collection if it doesn't exist
    pub async fn ensure_collection(&self, vector_size: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/collections/{}", self.config.url, self.config.collection_name);
        
        // Check if collection exists
        let response = self.client
            .get(&url)
            .header("api-key", &self.config.api_key)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("✅ Collection '{}' already exists", self.config.collection_name);
            return Ok(());
        }
        
        // Create collection
        info!("🔨 Creating collection '{}'", self.config.collection_name);
        
        let create_request = serde_json::json!({
            "vectors": {
                "size": vector_size,
                "distance": "Cosine"
            }
        });
        
        let response = self.client
            .put(&url)
            .header("api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&create_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("✅ Successfully created collection '{}'", self.config.collection_name);
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("❌ Failed to create collection: {}", error_text);
            Err(format!("Failed to create collection: {}", error_text).into())
        }
    }

    /// Store a vector with metadata
    pub async fn store_vector(
        &self,
        id: &str,
        vector: &[f32],
        metadata: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/collections/{}/points", self.config.url, self.config.collection_name);
        
        let mut payload = HashMap::new();
        if let Value::Object(map) = metadata {
            for (key, value) in map {
                payload.insert(key.clone(), value.clone());
            }
        }
        
        let point = QdrantPoint {
            id: id.to_string(),
            vector: vector.to_vec(),
            payload,
        };
        
        let upsert_request = QdrantUpsertRequest {
            points: vec![point],
        };
        
        let response = self.client
            .put(&url)
            .header("api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&upsert_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("✅ Successfully stored vector with id: {}", id);
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("❌ Failed to store vector: {}", error_text);
            Err(format!("Failed to store vector: {}", error_text).into())
        }
    }

    /// Store multiple vectors in batch
    pub async fn store_vectors_batch(
        &self,
        vectors: Vec<(String, Vec<f32>, Value)>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/collections/{}/points", self.config.url, self.config.collection_name);
        
        let points: Vec<QdrantPoint> = vectors
            .into_iter()
            .map(|(id, vector, metadata)| {
                let mut payload = HashMap::new();
                if let Value::Object(map) = metadata {
                    for (key, value) in map {
                        payload.insert(key, value);
                    }
                }
                
                QdrantPoint {
                    id,
                    vector,
                    payload,
                }
            })
            .collect();
        
        let upsert_request = QdrantUpsertRequest { points };
        
        let response = self.client
            .put(&url)
            .header("api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&upsert_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("✅ Successfully stored {} vectors in batch", upsert_request.points.len());
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("❌ Failed to store vectors in batch: {}", error_text);
            Err(format!("Failed to store vectors in batch: {}", error_text).into())
        }
    }

    /// Search for similar vectors
    pub async fn search_vectors(
        &self,
        query_vector: &[f32],
        limit: usize,
        with_payload: bool,
    ) -> Result<Vec<QdrantSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/collections/{}/points/search", self.config.url, self.config.collection_name);
        
        let search_request = QdrantSearchRequest {
            vector: query_vector.to_vec(),
            limit,
            with_payload,
            with_vector: false,
        };
        
        let response = self.client
            .post(&url)
            .header("api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&search_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let search_response: QdrantSearchResponse = response.json().await?;
            info!("✅ Found {} similar vectors", search_response.result.len());
            Ok(search_response.result)
        } else {
            let error_text = response.text().await?;
            error!("❌ Failed to search vectors: {}", error_text);
            Err(format!("Failed to search vectors: {}", error_text).into())
        }
    }

    /// Get collection information
    pub async fn get_collection_info(&self) -> Result<QdrantCollectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/collections/{}", self.config.url, self.config.collection_name);
        
        let response = self.client
            .get(&url)
            .header("api-key", &self.config.api_key)
            .send()
            .await?;
        
        if response.status().is_success() {
            let info: QdrantCollectionInfo = response.json().await?;
            info!("📊 Collection info: {} vectors", info.vectors_count.unwrap_or(0));
            Ok(info)
        } else {
            let error_text = response.text().await?;
            error!("❌ Failed to get collection info: {}", error_text);
            Err(format!("Failed to get collection info: {}", error_text).into())
        }
    }

    /// Delete vectors by filter
    pub async fn delete_vectors_by_filter(
        &self,
        filter: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/collections/{}/points/delete", self.config.url, self.config.collection_name);
        
        let delete_request = serde_json::json!({
            "filter": filter
        });
        
        let response = self.client
            .post(&url)
            .header("api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&delete_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("✅ Successfully deleted vectors matching filter");
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("❌ Failed to delete vectors: {}", error_text);
            Err(format!("Failed to delete vectors: {}", error_text).into())
        }
    }

    /// Delete specific vector by ID
    pub async fn delete_vector(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/collections/{}/points/delete", self.config.url, self.config.collection_name);
        
        let delete_request = serde_json::json!({
            "points": [id]
        });
        
        let response = self.client
            .post(&url)
            .header("api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&delete_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("✅ Successfully deleted vector with id: {}", id);
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("❌ Failed to delete vector: {}", error_text);
            Err(format!("Failed to delete vector: {}", error_text).into())
        }
    }
}

/// Create a default Qdrant configuration from environment variables
pub fn create_qdrant_config() -> QdrantConfig {
    QdrantConfig {
        url: std::env::var("QDRANT_URL")
            .unwrap_or_else(|_| "https://your-cluster-host:6333".to_string()),
        api_key: std::env::var("QDRANT_API_KEY")
            .unwrap_or_else(|_| "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2Nlc3MiOiJtIn0.uIPbGMwsHdiQlzUd5ad4Yx1HWPhg1hfbu3fiHRkGr6M".to_string()),
        collection_name: std::env::var("QDRANT_COLLECTION")
            .unwrap_or_else(|_| "conhub_embeddings".to_string()),
    }
}
