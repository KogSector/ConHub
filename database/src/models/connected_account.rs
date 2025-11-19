use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConnectedAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connector_type: String,
    pub account_name: String,
    pub account_identifier: String,
    pub credentials: serde_json::Value,
    pub status: serde_json::Value,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConnectedAccountInput {
    pub user_id: Uuid,
    pub connector_type: String,
    pub account_name: String,
    pub account_identifier: String,
    pub credentials: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConnectedAccountInput {
    pub account_name: Option<String>,
    pub credentials: Option<serde_json::Value>,
    pub status: Option<serde_json::Value>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

impl super::Model for ConnectedAccount {
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
