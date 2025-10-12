use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Auto-indexing trigger service
/// Automatically triggers indexing when data sources are connected or synced
pub struct AutoIndexTriggerService {
    indexer_url: String,
    client: reqwest::Client,
    active_jobs: Arc<RwLock<HashMap<Uuid, IndexingJobStatus>>>,
}

#[derive(Debug, Clone)]
pub struct IndexingJobStatus {
    pub connection_id: Uuid,
    pub source_type: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

impl AutoIndexTriggerService {
    pub fn new() -> Self {
        let indexer_url = std::env::var("UNIFIED_INDEXER_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        Self {
            indexer_url,
            client: reqwest::Client::new(),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Trigger indexing when a data source is connected
    pub async fn on_source_connected(
        &self,
        connection_id: Uuid,
        source_type: &str,
        source_url: &str,
    ) -> Result<()> {
        log::info!(
            "Auto-indexing triggered for new data source: {} ({})",
            connection_id,
            source_type
        );

        let job_status = IndexingJobStatus {
            connection_id,
            source_type: source_type.to_string(),
            started_at: chrono::Utc::now(),
            status: "in_progress".to_string(),
        };

        self.active_jobs.write().await.insert(connection_id, job_status);

        // Spawn background task to index
        let indexer_url = self.indexer_url.clone();
        let client = self.client.clone();
        let source_type = source_type.to_string();
        let source_url = source_url.to_string();
        let active_jobs = self.active_jobs.clone();

        tokio::spawn(async move {
            let result = Self::index_source(&client, &indexer_url, &source_type, &source_url).await;

            match result {
                Ok(_) => {
                    log::info!("Auto-indexing completed for connection: {}", connection_id);
                    if let Some(mut job) = active_jobs.write().await.get_mut(&connection_id) {
                        job.status = "completed".to_string();
                    }
                }
                Err(e) => {
                    log::error!("Auto-indexing failed for connection {}: {}", connection_id, e);
                    if let Some(mut job) = active_jobs.write().await.get_mut(&connection_id) {
                        job.status = format!("failed: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Trigger indexing when a data source is synced
    pub async fn on_source_synced(
        &self,
        connection_id: Uuid,
        source_type: &str,
        source_url: &str,
    ) -> Result<()> {
        log::info!(
            "Auto-indexing triggered for synced data source: {} ({})",
            connection_id,
            source_type
        );

        // Same as on_source_connected for now
        // In the future, this could do incremental indexing
        self.on_source_connected(connection_id, source_type, source_url).await
    }

    /// Trigger indexing via webhook
    pub async fn on_webhook_received(
        &self,
        source_type: &str,
        source_url: &str,
        event_type: &str,
    ) -> Result<()> {
        log::info!(
            "Auto-indexing triggered by webhook: {} - {} ({})",
            event_type,
            source_url,
            source_type
        );

        // Index the source
        Self::index_source(&self.client, &self.indexer_url, source_type, source_url).await
    }

    /// Index a source based on its type
    async fn index_source(
        client: &reqwest::Client,
        indexer_url: &str,
        source_type: &str,
        source_url: &str,
    ) -> Result<()> {
        let endpoint = match source_type.to_lowercase().as_str() {
            "github" | "gitlab" | "bitbucket" | "repository" => "/api/index/repository",
            "notion" | "confluence" | "documentation" => "/api/index/documentation",
            "url" | "web" | "website" => "/api/index/url",
            _ => "/api/index/url", // Default fallback
        };

        let request_body = serde_json::json!({
            "repository_url": source_url,
            "url": source_url,
            "documentation_url": source_url,
        });

        log::debug!(
            "Sending indexing request to: {}{} for {}",
            indexer_url,
            endpoint,
            source_url
        );

        let response = client
            .post(&format!("{}{}", indexer_url, endpoint))
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(5)) // Quick timeout for async operation
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Indexing request failed: HTTP {} - {}",
                response.status(),
                error_text
            ));
        }

        let result: serde_json::Value = response.json().await?;
        log::info!("Indexing job started: {:?}", result);

        Ok(())
    }

    /// Get status of active indexing jobs
    pub async fn get_active_jobs(&self) -> HashMap<Uuid, IndexingJobStatus> {
        self.active_jobs.read().await.clone()
    }

    /// Schedule periodic re-indexing for a source
    pub async fn schedule_periodic_reindex(
        &self,
        connection_id: Uuid,
        source_type: String,
        source_url: String,
        interval_hours: u64,
    ) {
        let indexer_url = self.indexer_url.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_hours * 3600));

            loop {
                interval.tick().await;
                log::info!("Scheduled re-indexing for connection: {}", connection_id);

                if let Err(e) = Self::index_source(&client, &indexer_url, &source_type, &source_url).await {
                    log::error!("Scheduled re-indexing failed for {}: {}", connection_id, e);
                }
            }
        });
    }
}

impl Default for AutoIndexTriggerService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auto_trigger_creation() {
        let service = AutoIndexTriggerService::new();
        assert!(!service.indexer_url.is_empty());
    }

    #[tokio::test]
    async fn test_active_jobs_tracking() {
        let service = AutoIndexTriggerService::new();
        let jobs = service.get_active_jobs().await;
        assert_eq!(jobs.len(), 0);
    }
}
