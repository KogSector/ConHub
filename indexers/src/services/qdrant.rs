use anyhow::{anyhow, Result};
use qdrant_client::{
    prelude::*,
    qdrant::{
        vectors_config::Config, CreateCollection, Distance, PointStruct, SearchPoints,
        VectorParams, VectorsConfig,
    },
};
use serde_json::Value;
use std::time::Duration;

/// Search result from Qdrant
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: Value,
}

/// Qdrant service for vector search operations
pub struct QdrantService {
    client: QdrantClient,
    timeout: Duration,
}

impl QdrantService {
    /// Create a new Qdrant service
    pub async fn new(url: &str, timeout_secs: u64) -> Result<Self> {
        log::info!("Connecting to Qdrant at: {}", url);

        let client = QdrantClient::from_url(url)
            .build()
            .map_err(|e| anyhow!("Failed to create Qdrant client: {}", e))?;

        let timeout = Duration::from_secs(timeout_secs);

        // Test connection
        match tokio::time::timeout(timeout, client.health_check()).await {
            Ok(Ok(_)) => {
                log::info!("Successfully connected to Qdrant");
            }
            Ok(Err(e)) => {
                log::warn!("Qdrant health check failed: {}", e);
            }
            Err(_) => {
                log::warn!("Qdrant connection timeout");
            }
        }

        Ok(Self { client, timeout })
    }

    /// Create a collection if it doesn't exist
    pub async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        log::info!("Creating Qdrant collection: {} (dim: {})", name, dimension);

        // Check if collection exists
        let collections = match tokio::time::timeout(
            self.timeout,
            self.client.list_collections(),
        )
        .await
        {
            Ok(Ok(response)) => response.collections,
            Ok(Err(e)) => {
                log::error!("Failed to list collections: {}", e);
                return Err(anyhow!("Failed to list collections: {}", e));
            }
            Err(_) => {
                return Err(anyhow!("Timeout listing collections"));
            }
        };

        let collection_exists = collections
            .iter()
            .any(|c| c.name == name);

        if collection_exists {
            log::info!("Collection {} already exists", name);
            return Ok(());
        }

        // Create collection with cosine similarity
        let create_collection = CreateCollection {
            collection_name: name.to_string(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: dimension as u64,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        match tokio::time::timeout(
            self.timeout,
            self.client.create_collection(&create_collection),
        )
        .await
        {
            Ok(Ok(_)) => {
                log::info!("Collection {} created successfully", name);
                Ok(())
            }
            Ok(Err(e)) => {
                log::error!("Failed to create collection: {}", e);
                Err(anyhow!("Failed to create collection: {}", e))
            }
            Err(_) => Err(anyhow!("Timeout creating collection")),
        }
    }

    /// Upsert vectors into a collection
    pub async fn upsert_vectors(
        &self,
        collection: &str,
        points: Vec<(String, Vec<f32>, Value)>,
    ) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        log::debug!("Upserting {} points to collection {}", points.len(), collection);

        let qdrant_points: Vec<PointStruct> = points
            .into_iter()
            .map(|(id, vector, payload)| PointStruct::new(
                id.clone(),
                vector,
                payload,
            ))
            .collect();

        // Retry logic: 1 retry on failure
        for attempt in 0..=1 {
            match tokio::time::timeout(
                self.timeout,
                self.client.upsert_points_blocking(collection, None, qdrant_points.clone(), None),
            )
            .await
            {
                Ok(Ok(_)) => {
                    log::debug!("Successfully upserted points to {}", collection);
                    return Ok(());
                }
                Ok(Err(e)) => {
                    if attempt == 0 {
                        log::warn!("Upsert attempt {} failed: {}, retrying...", attempt + 1, e);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    } else {
                        log::error!("Upsert failed after retry: {}", e);
                        return Err(anyhow!("Failed to upsert points: {}", e));
                    }
                }
                Err(_) => {
                    if attempt == 0 {
                        log::warn!("Upsert attempt {} timeout, retrying...", attempt + 1);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    } else {
                        return Err(anyhow!("Timeout upserting points"));
                    }
                }
            }
        }

        Ok(())
    }

    /// Search vectors in a collection
    pub async fn search_vectors(
        &self,
        collection: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        log::debug!("Searching in collection {} with limit {}", collection, limit);

        let search_points = SearchPoints {
            collection_name: collection.to_string(),
            vector: query_vector,
            limit: limit as u64,
            with_payload: Some(true.into()),
            ..Default::default()
        };

        // Retry logic: 1 retry on failure
        for attempt in 0..=1 {
            match tokio::time::timeout(
                self.timeout,
                self.client.search_points(&search_points),
            )
            .await
            {
                Ok(Ok(response)) => {
                    let results: Vec<SearchResult> = response
                        .result
                        .into_iter()
                        .map(|point| SearchResult {
                            id: point.id.map(|id| id.to_string()).unwrap_or_default(),
                            score: point.score,
                            payload: serde_json::to_value(&point.payload).unwrap_or(Value::Null),
                        })
                        .collect();

                    log::debug!("Found {} results from Qdrant", results.len());
                    return Ok(results);
                }
                Ok(Err(e)) => {
                    if attempt == 0 {
                        log::warn!("Search attempt {} failed: {}, retrying...", attempt + 1, e);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    } else {
                        log::error!("Search failed after retry: {}", e);
                        // Don't fail entire search, return empty results
                        return Ok(Vec::new());
                    }
                }
                Err(_) => {
                    if attempt == 0 {
                        log::warn!("Search attempt {} timeout, retrying...", attempt + 1);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    } else {
                        log::warn!("Search timeout, returning empty results");
                        return Ok(Vec::new());
                    }
                }
            }
        }

        Ok(Vec::new())
    }

    /// Check if Qdrant is available
    pub async fn is_available(&self) -> bool {
        match tokio::time::timeout(self.timeout, self.client.health_check()).await {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Qdrant instance
    async fn test_qdrant_connection() {
        let service = QdrantService::new("http://localhost:6333", 5)
            .await
            .unwrap();
        assert!(service.is_available().await);
    }

    #[tokio::test]
    #[ignore] // Requires running Qdrant instance
    async fn test_collection_creation() {
        let service = QdrantService::new("http://localhost:6333", 5)
            .await
            .unwrap();

        service
            .create_collection("test_collection", 768)
            .await
            .unwrap();
    }
}
