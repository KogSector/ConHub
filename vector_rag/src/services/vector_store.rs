use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::time::Duration;
use std::env;

/// Zilliz Cloud REST API client for vector operations
pub struct VectorStoreService {
    client: Client,
    base_url: String,
    api_key: String,
    timeout_secs: u64,
}

// ============================================================================
// Zilliz Cloud API Request/Response Types
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateCollectionRequest {
    collection_name: String,
    dimension: u32,
    metric_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
struct QueryRequest {
    collection_name: String,
    filter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_fields: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
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
    fields: Vec<FieldDescription>,
}

#[derive(Debug, Deserialize, Default)]
struct FieldDescription {
    #[serde(default)]
    name: String,
    #[serde(rename = "type", default)]
    field_type: String,
    #[serde(default)]
    primary_key: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub distance: f32,
    #[serde(flatten)]
    pub fields: Map<String, Value>,
}

impl VectorStoreService {
    /// Create a new Zilliz Cloud vector store service
    /// 
    /// # Arguments
    /// * `endpoint_url` - The Zilliz Cloud public endpoint (e.g., https://xxx.serverless.xxx.zilliz.com)
    /// * `timeout_secs` - Request timeout in seconds
    pub async fn new(endpoint_url: &str, timeout_secs: u64) -> Result<Self> {
        let api_key = env::var("ZILLIZ_API_KEY")
            .map_err(|_| anyhow!("ZILLIZ_API_KEY environment variable not set"))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        // Normalize the base URL
        let base_url = endpoint_url.trim_end_matches('/').to_string();

        log::info!("🔷 Zilliz Cloud client initialized for endpoint: {}", base_url);

        Ok(Self {
            client,
            base_url,
            api_key,
            timeout_secs,
        })
    }

    /// Get the full API URL for a given path
    fn api_url(&self, path: &str) -> String {
        format!("{}/v1/vector{}", self.base_url, path)
    }

    /// Make an authenticated request to Zilliz Cloud
    async fn make_request<T: Serialize, R: for<'de> Deserialize<'de> + Default>(
        &self,
        method: &str,
        path: &str,
        body: Option<&T>,
    ) -> Result<ZillizResponse<R>> {
        let url = self.api_url(path);
        
        let mut request = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };

        request = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if let Some(b) = body {
            request = request.json(b);
        }

        let response = request.send().await?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Zilliz API error ({}): {}", status, error_text));
        }

        let response_body: ZillizResponse<R> = response.json().await?;
        
        if response_body.code != 200 && response_body.code != 0 {
            return Err(anyhow!(
                "Zilliz API error (code {}): {}",
                response_body.code,
                response_body.message.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(response_body)
    }

    /// Check if a collection exists
    pub async fn collection_exists(&self, name: &str) -> Result<bool> {
        let url = format!("{}/v1/vector/collections/describe?collectionName={}", 
                          self.base_url, urlencoding::encode(name));
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status.as_u16() == 404 {
            return Ok(false);
        }
        
        let body: ZillizResponse<CollectionDescription> = response.json().await?;
        
        // Code 0 or 200 means success (collection exists)
        Ok(body.code == 0 || body.code == 200)
    }

    /// Ensure a collection exists, creating it if necessary
    pub async fn ensure_collection(&self, name: &str, dimension: usize) -> Result<()> {
        // Check if collection already exists
        if self.collection_exists(name).await? {
            log::debug!("Collection '{}' already exists", name);
            return Ok(());
        }

        // Create the collection
        let create_req = CreateCollectionRequest {
            collection_name: name.to_string(),
            dimension: dimension as u32,
            metric_type: "COSINE".to_string(),
            primary_field: Some("id".to_string()),
            vector_field: Some("vector".to_string()),
        };

        let _response: ZillizResponse<Value> = self
            .make_request("POST", "/collections/create", Some(&create_req))
            .await?;

        log::info!("✅ Created Zilliz collection '{}' with dimension {}", name, dimension);
        
        Ok(())
    }

    /// Insert or update vectors in a collection
    pub async fn upsert(
        &self,
        collection: &str,
        points: Vec<(String, Vec<f32>, Map<String, Value>)>,
    ) -> Result<usize> {
        if points.is_empty() {
            return Ok(0);
        }

        // Transform points into Zilliz insert format
        let data: Vec<Value> = points
            .into_iter()
            .map(|(id, vector, mut payload)| {
                // Ensure the ID is in the payload
                payload.insert("chunk_id".to_string(), json!(id));
                // Add the vector field
                payload.insert("vector".to_string(), json!(vector));
                json!(payload)
            })
            .collect();

        let insert_count = data.len();
        
        let insert_req = InsertRequest {
            collection_name: collection.to_string(),
            data,
        };

        let response: ZillizResponse<InsertResponseData> = self
            .make_request("POST", "/insert", Some(&insert_req))
            .await?;

        let actual_count = response
            .data
            .map(|d| d.insert_count)
            .unwrap_or(insert_count);

        log::debug!("Inserted {} vectors into collection '{}'", actual_count, collection);

        Ok(actual_count)
    }

    /// Search for similar vectors
    pub async fn search(
        &self,
        collection: &str,
        query_vector: Vec<f32>,
        top_k: usize,
        filter: Option<String>,
        output_fields: Option<Vec<String>>,
    ) -> Result<Vec<SearchResult>> {
        let search_req = SearchRequest {
            collection_name: collection.to_string(),
            vector: query_vector,
            limit: top_k,
            filter,
            output_fields,
        };

        let response: ZillizResponse<Vec<Value>> = self
            .make_request("POST", "/search", Some(&search_req))
            .await?;

        // Parse search results
        let results = response
            .data
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| {
                // Extract distance and fields from result
                let obj = v.as_object()?;
                let distance = obj.get("distance")?.as_f64()? as f32;
                let id = obj
                    .get("id")
                    .or_else(|| obj.get("chunk_id"))
                    .map(|v| v.to_string().trim_matches('"').to_string())
                    .unwrap_or_default();
                
                let mut fields: Map<String, Value> = Map::new();
                for (key, value) in obj {
                    if key != "distance" && key != "id" {
                        fields.insert(key.clone(), value.clone());
                    }
                }

                Some(SearchResult { id, distance, fields })
            })
            .collect();

        Ok(results)
    }

    /// Query vectors by filter
    pub async fn query(
        &self,
        collection: &str,
        filter: String,
        output_fields: Option<Vec<String>>,
        limit: Option<usize>,
    ) -> Result<Vec<Value>> {
        let query_req = QueryRequest {
            collection_name: collection.to_string(),
            filter,
            output_fields,
            limit,
        };

        let response: ZillizResponse<Vec<Value>> = self
            .make_request("POST", "/query", Some(&query_req))
            .await?;

        Ok(response.data.unwrap_or_default())
    }

    /// Delete vectors by filter
    pub async fn delete(&self, collection: &str, filter: String) -> Result<()> {
        let filter_clone = filter.clone();
        let delete_req = DeleteRequest {
            collection_name: collection.to_string(),
            filter,
        };

        let _response: ZillizResponse<Value> = self
            .make_request("POST", "/delete", Some(&delete_req))
            .await?;

        log::debug!("Deleted vectors from collection '{}' with filter: {}", collection, filter_clone);

        Ok(())
    }

    /// Delete vectors by IDs
    pub async fn delete_by_ids(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let filter = format!(
            "chunk_id in [{}]",
            ids.iter().map(|id| format!("\"{}\"", id)).collect::<Vec<_>>().join(", ")
        );

        self.delete(collection, filter).await
    }

    /// Get collection statistics
    pub async fn get_collection_stats(&self, collection: &str) -> Result<Value> {
        let url = format!("{}/v1/vector/collections/describe?collectionName={}", 
                          self.base_url, urlencoding::encode(collection));
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        let body: ZillizResponse<Value> = response.json().await?;
        
        Ok(body.data.unwrap_or(json!({})))
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let response: ZillizResponse<Vec<String>> = self
            .make_request::<(), Vec<String>>("GET", "/collections", None)
            .await?;

        Ok(response.data.unwrap_or_default())
    }

    /// Drop a collection
    pub async fn drop_collection(&self, collection: &str) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct DropRequest {
            collection_name: String,
        }

        let drop_req = DropRequest {
            collection_name: collection.to_string(),
        };

        let _response: ZillizResponse<Value> = self
            .make_request("POST", "/collections/drop", Some(&drop_req))
            .await?;

        log::info!("Dropped collection '{}'", collection);

        Ok(())
    }
}

// ============================================================================
// Helper Functions for Filter Building
// ============================================================================

/// Build a Zilliz-compatible filter string from search parameters
pub fn build_zilliz_filter(
    connector_types: Option<&[String]>,
    repositories: Option<&[String]>,
    authors: Option<&[String]>,
    tags: Option<&[String]>,
    tenant_id: Option<&str>,
) -> Option<String> {
    let mut conditions = Vec::new();

    if let Some(tenant) = tenant_id {
        conditions.push(format!("tenant_id == \"{}\"", tenant));
    }

    if let Some(types) = connector_types {
        if !types.is_empty() {
            let values = types.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");
            conditions.push(format!("connector_type in [{}]", values));
        }
    }

    if let Some(repos) = repositories {
        if !repos.is_empty() {
            let values = repos.iter().map(|r| format!("\"{}\"", r)).collect::<Vec<_>>().join(", ");
            conditions.push(format!("repository in [{}]", values));
        }
    }

    if let Some(auth) = authors {
        if !auth.is_empty() {
            let values = auth.iter().map(|a| format!("\"{}\"", a)).collect::<Vec<_>>().join(", ");
            conditions.push(format!("author in [{}]", values));
        }
    }

    if let Some(tag_list) = tags {
        if !tag_list.is_empty() {
            let values = tag_list.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");
            conditions.push(format!("tags in [{}]", values));
        }
    }

    if conditions.is_empty() {
        None
    } else {
        Some(conditions.join(" && "))
    }
}
