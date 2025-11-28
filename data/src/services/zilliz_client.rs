use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json, Map};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, error, warn, debug};

/// Configuration for Zilliz Cloud connection
#[derive(Debug, Clone)]
pub struct ZillizConfig {
    pub endpoint_url: String,
    pub api_key: String,
    pub collection_name: String,
}

impl ZillizConfig {
    pub fn from_env() -> Self {
        Self {
            endpoint_url: std::env::var("ZILLIZ_PUBLIC_ENDPOINT")
                .or_else(|_| std::env::var("ZILLIZ_ENDPOINT"))
                .unwrap_or_else(|_| "https://localhost:19530".to_string()),
            api_key: std::env::var("ZILLIZ_API_KEY")
                .unwrap_or_else(|_| "your_api_key_here".to_string()),
            collection_name: std::env::var("ZILLIZ_COLLECTION")
                .unwrap_or_else(|_| "conhub_embeddings".to_string()),
        }
    }
}

/// Zilliz Cloud REST API client
#[derive(Debug, Clone)]
pub struct ZillizClient {
    client: Client,
    config: ZillizConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateCollectionRequest {
    collection_name: String,
    dimension: u32,
    metric_type: String,
    primary_field: Option<String>,
    vector_field: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InsertRequest {
    collection_name: String,
    data: Vec<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchRequest {
    collection_name: String,
    vector: Vec<f32>,
    limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_fields: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteRequest {
    collection_name: String,
    filter: String,
}

#[derive(Debug, Deserialize, Default)]
struct ZillizResponse<T: Default> {
    #[serde(default)]
    code: i32,
    #[serde(default)]
    data: Option<T>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct InsertResponseData {
    #[serde(default)]
    insert_count: usize,
    #[serde(default)]
    insert_ids: Vec<Value>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CollectionDescription {
    #[serde(default)]
    collection_name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    fields: Vec<Value>,
}

/// Search result from Zilliz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZillizSearchResult {
    pub id: String,
    pub score: f32,
    pub payload: Option<HashMap<String, Value>>,
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZillizCollectionInfo {
    pub status: String,
    pub vectors_count: Option<u64>,
}

impl ZillizClient {
    /// Create a new Zilliz client
    pub fn new(config: ZillizConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }

    /// Get the API URL for a path
    fn api_url(&self, path: &str) -> String {
        format!("{}/v1/vector{}", self.config.endpoint_url.trim_end_matches('/'), path)
    }

    /// Check if collection exists
    pub async fn collection_exists(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/v1/vector/collections/describe?collectionName={}",
            self.config.endpoint_url.trim_end_matches('/'),
            urlencoding::encode(&self.config.collection_name)
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status.as_u16() == 404 {
            return Ok(false);
        }

        let body: ZillizResponse<CollectionDescription> = response.json().await?;
        Ok(body.code == 0 || body.code == 200)
    }

    /// Ensure collection exists, creating it if necessary
    pub async fn ensure_collection(&self, vector_size: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.collection_exists().await? {
            info!("✅ Collection '{}' already exists", self.config.collection_name);
            return Ok(());
        }

        info!("🔨 Creating collection '{}'", self.config.collection_name);

        let create_req = CreateCollectionRequest {
            collection_name: self.config.collection_name.clone(),
            dimension: vector_size as u32,
            metric_type: "COSINE".to_string(),
            primary_field: Some("id".to_string()),
            vector_field: Some("vector".to_string()),
        };

        let url = self.api_url("/collections/create");
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&create_req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("❌ Failed to create collection: {}", error_text);
            return Err(format!("Failed to create collection: {}", error_text).into());
        }

        let body: ZillizResponse<Value> = response.json().await?;
        if body.code != 0 && body.code != 200 {
            let msg = body.message.unwrap_or_else(|| "Unknown error".to_string());
            error!("❌ Zilliz error: {}", msg);
            return Err(format!("Zilliz error: {}", msg).into());
        }

        info!("✅ Successfully created collection '{}'", self.config.collection_name);
        Ok(())
    }

    /// Store a single vector with metadata
    pub async fn store_vector(
        &self,
        id: &str,
        vector: &[f32],
        metadata: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.store_vectors_batch(vec![(id.to_string(), vector.to_vec(), metadata.clone())]).await
    }

    /// Store multiple vectors in batch
    pub async fn store_vectors_batch(
        &self,
        vectors: Vec<(String, Vec<f32>, Value)>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if vectors.is_empty() {
            return Ok(());
        }

        let data: Vec<Value> = vectors
            .into_iter()
            .map(|(id, vector, metadata)| {
                let mut obj = if let Value::Object(map) = metadata {
                    map
                } else {
                    Map::new()
                };
                obj.insert("chunk_id".to_string(), json!(id));
                obj.insert("vector".to_string(), json!(vector));
                json!(obj)
            })
            .collect();

        let count = data.len();
        let insert_req = InsertRequest {
            collection_name: self.config.collection_name.clone(),
            data,
        };

        let url = self.api_url("/insert");
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&insert_req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("❌ Failed to store vectors: {}", error_text);
            return Err(format!("Failed to store vectors: {}", error_text).into());
        }

        let body: ZillizResponse<InsertResponseData> = response.json().await?;
        if body.code != 0 && body.code != 200 {
            let msg = body.message.unwrap_or_else(|| "Unknown error".to_string());
            error!("❌ Zilliz insert error: {}", msg);
            return Err(format!("Zilliz insert error: {}", msg).into());
        }

        info!("✅ Successfully stored {} vectors in batch", count);
        Ok(())
    }

    /// Search for similar vectors
    pub async fn search_vectors(
        &self,
        query_vector: &[f32],
        limit: usize,
        with_payload: bool,
    ) -> Result<Vec<ZillizSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        self.search_vectors_with_filter(query_vector, limit, None, with_payload).await
    }

    /// Search with filter
    pub async fn search_vectors_with_filter(
        &self,
        query_vector: &[f32],
        limit: usize,
        filter: Option<String>,
        with_payload: bool,
    ) -> Result<Vec<ZillizSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let output_fields = if with_payload {
            Some(vec![
                "chunk_id".to_string(),
                "document_name".to_string(),
                "document_path".to_string(),
                "repository".to_string(),
                "branch".to_string(),
                "content_type".to_string(),
                "chunk_number".to_string(),
                "url".to_string(),
                "connector_type".to_string(),
            ])
        } else {
            None
        };

        let search_req = SearchRequest {
            collection_name: self.config.collection_name.clone(),
            vector: query_vector.to_vec(),
            limit,
            filter,
            output_fields,
        };

        let url = self.api_url("/search");
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&search_req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("❌ Failed to search vectors: {}", error_text);
            return Err(format!("Failed to search vectors: {}", error_text).into());
        }

        let body: ZillizResponse<Vec<Value>> = response.json().await?;
        if body.code != 0 && body.code != 200 {
            let msg = body.message.unwrap_or_else(|| "Unknown error".to_string());
            error!("❌ Zilliz search error: {}", msg);
            return Err(format!("Zilliz search error: {}", msg).into());
        }

        let results: Vec<ZillizSearchResult> = body
            .data
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| {
                let obj = v.as_object()?;
                let distance = obj.get("distance")?.as_f64()? as f32;
                let id = obj
                    .get("id")
                    .or_else(|| obj.get("chunk_id"))
                    .map(|v| v.to_string().trim_matches('"').to_string())
                    .unwrap_or_default();

                let mut payload = HashMap::new();
                for (key, value) in obj {
                    if key != "distance" && key != "id" && key != "vector" {
                        payload.insert(key.clone(), value.clone());
                    }
                }

                Some(ZillizSearchResult {
                    id,
                    score: 1.0 - distance, // Convert distance to similarity
                    payload: if payload.is_empty() { None } else { Some(payload) },
                })
            })
            .collect();

        info!("✅ Found {} similar vectors", results.len());
        Ok(results)
    }

    /// Delete vectors by filter
    pub async fn delete_vectors_by_filter(
        &self,
        filter: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let delete_req = DeleteRequest {
            collection_name: self.config.collection_name.clone(),
            filter,
        };

        let url = self.api_url("/delete");
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&delete_req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("❌ Failed to delete vectors: {}", error_text);
            return Err(format!("Failed to delete vectors: {}", error_text).into());
        }

        info!("✅ Successfully deleted vectors matching filter");
        Ok(())
    }

    /// Delete specific vector by ID
    pub async fn delete_vector(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filter = format!("chunk_id == \"{}\"", id);
        self.delete_vectors_by_filter(filter).await
    }

    /// Get collection information
    pub async fn get_collection_info(&self) -> Result<ZillizCollectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/v1/vector/collections/describe?collectionName={}",
            self.config.endpoint_url.trim_end_matches('/'),
            urlencoding::encode(&self.config.collection_name)
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("❌ Failed to get collection info: {}", error_text);
            return Err(format!("Failed to get collection info: {}", error_text).into());
        }

        // For now return basic info
        Ok(ZillizCollectionInfo {
            status: "ready".to_string(),
            vectors_count: None,
        })
    }
}

/// Build a Zilliz filter string from components
pub fn build_zilliz_filter(
    repository: Option<&str>,
    branch: Option<&str>,
    content_types: Option<&[String]>,
) -> Option<String> {
    let mut conditions = Vec::new();

    if let Some(repo) = repository {
        conditions.push(format!("repository == \"{}\"", repo));
    }

    if let Some(b) = branch {
        conditions.push(format!("branch == \"{}\"", b));
    }

    if let Some(types) = content_types {
        if !types.is_empty() {
            let values = types.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");
            conditions.push(format!("content_type in [{}]", values));
        }
    }

    if conditions.is_empty() {
        None
    } else {
        Some(conditions.join(" && "))
    }
}
