use async_graphql::{Context, EmptySubscription, InputObject, Object, Schema};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Semaphore;
use serde::{Deserialize, Serialize};
use crate::config::AppConfig;
use conhub_models::auth::Claims;
use conhub_models::graphql::{EmbeddingResult, RerankResult, RerankDocument as SharedRerankDocument, RerankDocumentOutput};
use std::collections::HashMap;
use conhub_config::feature_toggles::FeatureToggles;
use conhub_utils::cache_manager::{get_cache, CacheError};

#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> String {
        "healthy".to_string()
    }

    async fn version(&self) -> String {
        // Keep simple; can be wired to config later
        "v1".to_string()
    }

    async fn me(&self, ctx: &Context<'_>) -> Option<CurrentUser> {
        let claims = ctx.data_opt::<Claims>()?;
        Some(CurrentUser::from(claims.clone()))
    }

    async fn embed(&self, ctx: &Context<'_>, texts: Vec<String>, normalize: Option<bool>) -> async_graphql::Result<EmbeddingResult> {
        // Check Heavy feature toggle
        let toggles = ctx.data::<FeatureToggles>()?;
        if !toggles.should_enable_embedding() {
            return Err(async_graphql::Error::new(
                "Embedding service is disabled via feature toggles. Enable 'Heavy' feature to use embeddings."
            ));
        }

        // Generate cache key from texts and normalization setting
        let normalize_val = normalize.unwrap_or(true);
        let cache_key = format!("embed:{}:{}", normalize_val, texts.join("|"));
        
        // Try to get from cache first
        let cache = get_cache();
        if let Some(cached_result) = cache.get::<EmbeddingResult>(&cache_key) {
            return Ok(cached_result);
        }

        let cfg = ctx.data::<AppConfig>()?;
        let semaphore = ctx.data::<Arc<Semaphore>>()?;
        let _permit = semaphore.acquire().await.map_err(|_| async_graphql::Error::new("Concurrency limiter closed"))?;
        let url = format!("{}/embed", cfg.embedding_service_url);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(cfg.embedding_request_timeout_ms))
            .build()
            .map_err(|e| async_graphql::Error::new(format!("Failed to build HTTP client: {}", e)))?;

        #[derive(Deserialize)]
        struct EmbedResponse {
            embeddings: Vec<Vec<f32>>,
            dimension: usize,
            model: String,
            count: usize,
        }

        let body = serde_json::json!({
            "text": texts,
            "normalize": normalize.unwrap_or(true)
        });

        let mut last_err: Option<async_graphql::Error> = None;
        for attempt in 0..=cfg.embedding_request_retries {
            match client.post(&url).json(&body).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let parsed: Result<EmbedResponse, _> = resp.json().await;
                        match parsed {
                            Ok(parsed) => {
                                let result = EmbeddingResult {
                                    embeddings: parsed.embeddings,
                                    dimension: parsed.dimension,
                                    model: parsed.model,
                                    count: parsed.count,
                                };
                                
                                // Cache the result for future requests (1 hour TTL)
                                let _ = cache.set(&cache_key, &result, Some(Duration::from_secs(3600)));
                                
                                return Ok(result);
                            }
                            Err(e) => {
                                last_err = Some(async_graphql::Error::new(format!(
                                    "Failed to parse embedding response: {}",
                                    e
                                )));
                            }
                        }
                    } else {
                        let status = resp.status();
                        let text = resp.text().await.unwrap_or_default();
                        // Retry only on transient errors
                        if status.is_server_error() || status.as_u16() == 429 || status.as_u16() == 408 {
                            last_err = Some(async_graphql::Error::new(format!(
                                "Embedding service transient error {}: {}",
                                status, text
                            )));
                        } else {
                            return Err(async_graphql::Error::new(format!(
                                "Embedding service error {}: {}",
                                status, text
                            )));
                        }
                    }
                }
                Err(e) => {
                    last_err = Some(async_graphql::Error::new(format!(
                        "Embedding service request failed: {}",
                        e
                    )));
                }
            }
            // Exponential backoff between retries
            let delay_ms = 100u64.saturating_mul(1u64 << attempt);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
        return Err(last_err.unwrap_or_else(|| async_graphql::Error::new("Embedding request failed")));
    }

    async fn rerank(&self, ctx: &Context<'_>, query: String, documents: Vec<RerankDocumentInput>, top_k: Option<i32>) -> async_graphql::Result<Vec<RerankResult>> {
        let cfg = ctx.data::<AppConfig>()?;
        let semaphore = ctx.data::<Arc<Semaphore>>()?;
        let _permit = semaphore.acquire().await.map_err(|_| async_graphql::Error::new("Concurrency limiter closed"))?;
        let url = format!("{}/rerank", cfg.embedding_service_url);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(cfg.embedding_request_timeout_ms))
            .build()
            .map_err(|e| async_graphql::Error::new(format!("Failed to build HTTP client: {}", e)))?;

        #[derive(Deserialize)]
        struct RerankResponseRest {
            results: Vec<RerankResultRest>,
        }

        #[derive(Deserialize)]
        struct RerankResultRest {
            id: String,
            score: f32,
        }

        let body = serde_json::json!({
            "query": query,
            "documents": documents,
            "top_k": top_k,
        });

        let mut last_err: Option<async_graphql::Error> = None;
        for attempt in 0..=cfg.embedding_request_retries {
            match client.post(&url).json(&body).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let parsed: Result<RerankResponseRest, _> = resp.json().await;
                        match parsed {
                            Ok(parsed) => {
                                let map: HashMap<String, RerankDocumentInput> = documents
                                    .iter()
                                    .cloned()
                                    .map(|d| (d.id.clone(), d))
                                    .collect();
                                let results = parsed
                                    .results
                                    .into_iter()
                                    .enumerate()
                                    .map(|(idx, r)| {
                                        let doc_out = map.get(&r.id).map(|d| RerankDocumentOutput {
                                            id: d.id.clone(),
                                            text: d.text.clone(),
                                            metadata: d.metadata.clone(),
                                        }).unwrap_or(RerankDocumentOutput { id: r.id.clone(), text: String::new(), metadata: None });
                                        RerankResult { id: r.id, score: r.score, index: idx, document: doc_out }
                                    })
                                    .collect::<Vec<_>>();
                                return Ok(results);
                            }
                            Err(e) => {
                                last_err = Some(async_graphql::Error::new(format!(
                                    "Failed to parse rerank response: {}",
                                    e
                                )));
                            }
                        }
                    } else {
                        let status = resp.status();
                        let text = resp.text().await.unwrap_or_default();
                        // Retry only on transient errors
                        if status.is_server_error() || status.as_u16() == 429 || status.as_u16() == 408 {
                            last_err = Some(async_graphql::Error::new(format!(
                                "Rerank service transient error {}: {}",
                                status, text
                            )));
                        } else {
                            return Err(async_graphql::Error::new(format!(
                                "Rerank service error {}: {}",
                                status, text
                            )));
                        }
                    }
                }
                Err(e) => {
                    last_err = Some(async_graphql::Error::new(format!(
                        "Rerank service request failed: {}",
                        e
                    )));
                }
            }
            // Exponential backoff between retries
            let delay_ms = 100u64.saturating_mul(1u64 << attempt);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
        Err(last_err.unwrap_or_else(|| async_graphql::Error::new("Rerank request failed")))
    }
}

#[derive(async_graphql::SimpleObject, Default)]
pub struct CurrentUser {
    pub user_id: Option<String>,
    pub roles: Vec<String>,
}

impl From<Claims> for CurrentUser {
    fn from(c: Claims) -> Self {
        Self {
            user_id: Some(c.sub.to_string()),
            roles: c.roles,
        }
    }
}

pub type ConhubSchema = Schema<QueryRoot, async_graphql::EmptyMutation, EmptySubscription>;

// Local type alias for backward compatibility
pub type RerankDocumentInput = SharedRerankDocument;

pub fn build_schema(cfg: AppConfig, toggles: FeatureToggles) -> ConhubSchema {
    let concurrency_limit = cfg.embedding_max_inflight;
    Schema::build(QueryRoot::default(), async_graphql::EmptyMutation, EmptySubscription)
        .data(cfg)
        .data(toggles)
        .data(Arc::new(Semaphore::new(concurrency_limit)))
        .finish()
}
