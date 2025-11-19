use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub connector_type: String,
    pub job_type: String,
    pub status: String,
    pub priority: i32,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub config: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncRun {
    pub id: Uuid,
    pub job_id: Uuid,
    pub run_number: i32,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub documents_discovered: Option<i32>,
    pub documents_processed: Option<i32>,
    pub documents_failed: Option<i32>,
    pub documents_skipped: Option<i32>,
    pub bytes_processed: Option<i64>,
    pub error_details: Option<serde_json::Value>,
    pub performance_metrics: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSyncJobInput {
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub job_type: String,
    pub priority: Option<i32>,
    pub config: Option<serde_json::Value>,
}

impl super::Model for SyncJob {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
