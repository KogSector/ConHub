use async_trait::async_trait;
use anyhow::{Result, Context};
use sqlx::{PgPool, query_as, query};
use uuid::Uuid;

use crate::models::{ConnectedAccount, CreateConnectedAccountInput, UpdateConnectedAccountInput, Model, Pagination, PaginatedResult};
use super::Repository;

pub struct ConnectedAccountRepository {
    pool: PgPool,
}

impl ConnectedAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_account(&self, input: &CreateConnectedAccountInput) -> Result<ConnectedAccount> {
        let account = query_as!(
            ConnectedAccount,
            r#"
            INSERT INTO connected_accounts (user_id, connector_type, account_name, account_identifier, credentials)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            input.user_id,
            input.connector_type,
            input.account_name,
            input.account_identifier,
            input.credentials
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create connected account")?;

        Ok(account)
    }

    pub async fn find_by_user(&self, user_id: &Uuid) -> Result<Vec<ConnectedAccount>> {
        let accounts = query_as!(
            ConnectedAccount,
            "SELECT * FROM connected_accounts WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to find accounts by user")?;

        Ok(accounts)
    }

    pub async fn find_by_user_and_type(&self, user_id: &Uuid, connector_type: &str) -> Result<Vec<ConnectedAccount>> {
        let accounts = query_as!(
            ConnectedAccount,
            "SELECT * FROM connected_accounts WHERE user_id = $1 AND connector_type = $2",
            user_id,
            connector_type
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to find accounts by user and type")?;

        Ok(accounts)
    }

    pub async fn update_account(&self, id: &Uuid, input: &UpdateConnectedAccountInput) -> Result<ConnectedAccount> {
        let account = query_as!(
            ConnectedAccount,
            r#"
            UPDATE connected_accounts
            SET account_name = COALESCE($1, account_name),
                credentials = COALESCE($2, credentials),
                status = COALESCE($3, status),
                last_sync_at = COALESCE($4, last_sync_at),
                metadata = COALESCE($5, metadata),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $6
            RETURNING *
            "#,
            input.account_name,
            input.credentials,
            input.status,
            input.last_sync_at,
            input.metadata,
            id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to update connected account")?;

        Ok(account)
    }

    pub async fn update_last_sync(&self, id: &Uuid) -> Result<()> {
        query!(
            "UPDATE connected_accounts SET last_sync_at = CURRENT_TIMESTAMP WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update last sync")?;

        Ok(())
    }

    pub async fn count_documents(&self, account_id: &Uuid) -> Result<i64> {
        let result = query!(
            "SELECT COUNT(*) as count FROM source_documents WHERE source_id = $1",
            account_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to count documents")?;

        Ok(result.count.unwrap_or(0))
    }
}

#[async_trait]
impl Repository<ConnectedAccount> for ConnectedAccountRepository {
    async fn create(&self, entity: &ConnectedAccount) -> Result<ConnectedAccount> {
        let account = query_as!(
            ConnectedAccount,
            r#"
            INSERT INTO connected_accounts (id, user_id, connector_type, account_name, account_identifier, credentials, status, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            entity.id,
            entity.user_id,
            entity.connector_type,
            entity.account_name,
            entity.account_identifier,
            entity.credentials,
            entity.status,
            entity.metadata
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create connected account")?;

        Ok(account)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<ConnectedAccount>> {
        let account = query_as!(
            ConnectedAccount,
            "SELECT * FROM connected_accounts WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find connected account")?;

        Ok(account)
    }

    async fn update(&self, id: &Uuid, entity: &ConnectedAccount) -> Result<ConnectedAccount> {
        let account = query_as!(
            ConnectedAccount,
            r#"
            UPDATE connected_accounts
            SET account_name = $1, credentials = $2, status = $3, metadata = $4, updated_at = CURRENT_TIMESTAMP
            WHERE id = $5
            RETURNING *
            "#,
            entity.account_name,
            entity.credentials,
            entity.status,
            entity.metadata,
            id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to update connected account")?;

        Ok(account)
    }

    async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = query!(
            "DELETE FROM connected_accounts WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete connected account")?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, pagination: &Pagination) -> Result<PaginatedResult<ConnectedAccount>> {
        let total: i64 = query!("SELECT COUNT(*) as count FROM connected_accounts")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count connected accounts")?
            .count
            .unwrap_or(0);

        let accounts = query_as!(
            ConnectedAccount,
            "SELECT * FROM connected_accounts ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list connected accounts")?;

        Ok(PaginatedResult::new(accounts, total, pagination))
    }
}
