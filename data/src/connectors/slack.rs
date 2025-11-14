use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::{info, error};
use uuid::Uuid;

use super::traits::Connector;
use super::types::*;
use super::error::ConnectorError;

pub struct SlackConnector {
    client: Client,
    token: Option<String>,
}

impl SlackConnector {
    pub fn new() -> Self {
        Self { client: Client::new(), token: None }
    }

    pub fn factory() -> SlackConnectorFactory { SlackConnectorFactory }

    fn chunk_content(&self, content: &str, channel: &str) -> Vec<DocumentChunk> {
        const CHUNK_SIZE: usize = 2000;
        const OVERLAP: usize = 200;
        let mut chunks = Vec::new();
        let bytes = content.as_bytes();
        let mut start = 0usize;
        let mut idx = 0usize;
        while start < bytes.len() {
            let end = (start + CHUNK_SIZE).min(bytes.len());
            let chunk_end = if end < bytes.len() {
                bytes[start..end].iter().rposition(|&b| b == b'\n' || b == b' ').map(|p| start + p).unwrap_or(end)
            } else { end };
            let s = String::from_utf8_lossy(&bytes[start..chunk_end]).to_string();
            if !s.trim().is_empty() {
                chunks.push(DocumentChunk {
                    chunk_number: idx,
                    content: s,
                    start_offset: start,
                    end_offset: chunk_end,
                    metadata: Some(json!({"channel": channel})),
                });
                idx += 1;
            }
            if chunk_end >= bytes.len() { break; }
            start = chunk_end.saturating_sub(OVERLAP);
        }
        chunks
    }
}

#[async_trait]
impl Connector for SlackConnector {
    fn name(&self) -> &str { "Slack" }
    fn connector_type(&self) -> ConnectorType { ConnectorType::Slack }

    fn validate_config(&self, config: &ConnectorConfigAuth) -> Result<(), ConnectorError> {
        if !config.credentials.contains_key("token") {
            return Err(ConnectorError::InvalidConfiguration("Slack token is required".to_string()));
        }
        Ok(())
    }

    async fn authenticate(&self, _config: &ConnectorConfigAuth) -> Result<Option<String>, ConnectorError> { Ok(None) }
    async fn complete_oauth(&self, _callback_data: OAuthCallbackData) -> Result<OAuthCredentials, ConnectorError> { Err(ConnectorError::UnsupportedOperation("Slack OAuth not implemented".to_string())) }

    async fn connect(&mut self, account: &ConnectedAccount) -> Result<(), ConnectorError> {
        let token = account.credentials.get("token").and_then(|v| v.as_str()).ok_or_else(|| ConnectorError::InvalidConfiguration("Slack token missing".to_string()))?;
        let resp = self.client.get("https://slack.com/api/auth.test").header("Authorization", format!("Bearer {}", token)).send().await?;
        let ok = resp.status().is_success() && resp.json::<Value>().await?.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        if !ok { return Err(ConnectorError::AuthenticationFailed("Slack token invalid".to_string())); }
        self.token = Some(token.to_string());
        info!("✅ Connected to Slack");
        Ok(())
    }

    async fn check_connection(&self, _account: &ConnectedAccount) -> Result<bool, ConnectorError> { Ok(self.token.is_some()) }

    async fn list_documents(&self, account: &ConnectedAccount, _filters: Option<SyncFilters>) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        let token = self.token.as_ref().ok_or_else(|| ConnectorError::AuthenticationFailed("Slack not connected".to_string()))?;
        let resp = self.client.get("https://slack.com/api/conversations.list").header("Authorization", format!("Bearer {}", token)).send().await?;
        let v: Value = resp.json().await?;
        let mut docs = Vec::new();
        if let Some(channels) = v.get("channels").and_then(|c| c.as_array()) {
            for ch in channels.iter().take(10) {
                if let (Some(id), Some(name)) = (ch.get("id").and_then(|s| s.as_str()), ch.get("name").and_then(|s| s.as_str())) {
                    docs.push(DocumentMetadata {
                        external_id: id.to_string(),
                        name: name.to_string(),
                        path: None,
                        mime_type: Some("text/plain".to_string()),
                        size: None,
                        created_at: None,
                        modified_at: None,
                        permissions: None,
                        url: None,
                        parent_id: None,
                        is_folder: false,
                        metadata: Some(json!({"channel_id": id, "channel_name": name})),
                    });
                }
            }
        }
        Ok(docs)
    }

    async fn get_document_content(&self, _account: &ConnectedAccount, document_id: &str) -> Result<DocumentContent, ConnectorError> {
        let token = self.token.as_ref().ok_or_else(|| ConnectorError::AuthenticationFailed("Slack not connected".to_string()))?;
        let resp = self.client.get(&format!("https://slack.com/api/conversations.history?channel={}&limit=200", document_id)).header("Authorization", format!("Bearer {}", token)).send().await?;
        let v: Value = resp.json().await?;
        let mut buffer = String::new();
        if let Some(messages) = v.get("messages").and_then(|m| m.as_array()) {
            for m in messages {
                if let Some(text) = m.get("text").and_then(|t| t.as_str()) { buffer.push_str(text); buffer.push('\n'); }
            }
        }
        Ok(DocumentContent { metadata: DocumentMetadata { external_id: document_id.to_string(), name: format!("Slack Channel {}", document_id), path: None, mime_type: Some("text/plain".to_string()), size: None, created_at: None, modified_at: None, permissions: None, url: None, parent_id: None, is_folder: false, metadata: None }, content: buffer.into_bytes(), content_type: ContentType::Text })
    }

    async fn sync(&self, account: &ConnectedAccount, request: &SyncRequestWithFilters) -> Result<(SyncResult, Vec<DocumentForEmbedding>), ConnectorError> {
        let start = std::time::Instant::now();
        let channels = self.list_documents(account, request.filters.clone()).await?;
        let mut docs_for_embed = Vec::new();
        let mut errors = Vec::new();
        for ch in &channels {
            match self.get_document_content(account, &ch.external_id).await {
                Ok(content) => {
                    let text = String::from_utf8_lossy(&content.content).to_string();
                    let chunks = self.chunk_content(&text, &ch.external_id);
                    docs_for_embed.push(DocumentForEmbedding {
                        id: Uuid::new_v4(),
                        source_id: account.id,
                        connector_type: ConnectorType::Slack,
                        external_id: ch.external_id.clone(),
                        name: ch.name.clone(),
                        path: None,
                        content: text,
                        content_type: ContentType::Text,
                        metadata: json!({"channel_id": ch.external_id, "channel_name": ch.name}),
                        chunks: Some(chunks),
                    });
                }
                Err(e) => { error!("Failed to fetch Slack channel {}: {}", ch.external_id, e); errors.push(format!("{}", e)); }
            }
        }
        let result = SyncResult { total_documents: channels.len(), new_documents: docs_for_embed.len(), updated_documents: 0, deleted_documents: 0, failed_documents: errors.len(), sync_duration_ms: start.elapsed().as_millis() as u64, errors };
        Ok((result, docs_for_embed))
    }

    async fn incremental_sync(&self, _account: &ConnectedAccount, _since: chrono::DateTime<chrono::Utc>) -> Result<Vec<DocumentMetadata>, ConnectorError> { Ok(vec![]) }
    async fn disconnect(&mut self, _account: &ConnectedAccount) -> Result<(), ConnectorError> { Ok(()) }
    async fn refresh_credentials(&self, _account: &ConnectedAccount) -> Result<OAuthCredentials, ConnectorError> { Err(ConnectorError::UnsupportedOperation("Slack credential refresh not supported".to_string())) }
}

pub struct SlackConnectorFactory;

impl super::traits::ConnectorFactory for SlackConnectorFactory {
    fn create(&self) -> Box<dyn Connector> { Box::new(SlackConnector::new()) }
    fn connector_type(&self) -> ConnectorType { ConnectorType::Slack }
    fn supports_oauth(&self) -> bool { false }
    fn supports_webhooks(&self) -> bool { false }
}
