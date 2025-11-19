use async_trait::async_trait;
use anyhow::{Result, Context};
use sqlx::{PgPool, query_as, query};
use uuid::Uuid;

use crate::models::{SyncJob, SyncRun, CreateSyncJobInput, Model, Pagination, PaginatedResult};
use super::Repository;

pub struct SyncJobRepository {
    pool: PgPool,
}

impl SyncJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_job(&self, input: &CreateSyncJobInput) -> Result<SyncJob> {
        let connector_type = query!(
            "SELECT connector_type FROM connected_accounts WHERE id = $1",
            input.account_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get connector type")?
        .connector_type;

        let job = query_as!(
            SyncJob,
            r#"
            INSERT INTO sync_jobs (user_id, account_id, connector_type, job_type, priority, config)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            input.user_id,
            input.account_id,
            connector_type,
            input.job_type,
            input.priority.unwrap_or(5),
            input.config
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create sync job")?;

        Ok(job)
    }

    pub async fn find_by_user(&self, user_id: &Uuid, pagination: &Pagination) -> Result<PaginatedResult<SyncJob>> {
        let total: i64 = query!(
            "SELECT COUNT(*) as count FROM sync_jobs WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to count jobs")?
        .count
        .unwrap_or(0);

        let jobs = query_as!(
            SyncJob,
            "SELECT * FROM sync_jobs WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            user_id,
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to find jobs by user")?;

        Ok(PaginatedResult::new(jobs, total, pagination))
    }

    pub async fn find_by_account(&self, account_id: &Uuid) -> Result<Vec<SyncJob>> {
        let jobs = query_as!(
            SyncJob,
            "SELECT * FROM sync_jobs WHERE account_id = $1 ORDER BY created_at DESC",
            account_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to find jobs by account")?;

        Ok(jobs)
    }

    pub async fn find_pending(&self, limit: i32) -> Result<Vec<SyncJob>> {
        let jobs = query_as!(
            SyncJob,
            "SELECT * FROM sync_jobs WHERE status = 'pending' ORDER BY priority, scheduled_at LIMIT $1",
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to find pending jobs")?;

        Ok(jobs)
    }

    pub async fn update_status(&self, id: &Uuid, status: &str) -> Result<()> {
        query!(
            "UPDATE sync_jobs SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            status,
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update job status")?;

        Ok(())
    }

    pub async fn start_job(&self, id: &Uuid) -> Result<()> {
        query!(
            "UPDATE sync_jobs SET status = 'running', started_at = CURRENT_TIMESTAMP WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to start job")?;

        Ok(())
    }

    pub async fn complete_job(&self, id: &Uuid) -> Result<()> {
        query!(
            "UPDATE sync_jobs SET status = 'completed', completed_at = CURRENT_TIMESTAMP WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to complete job")?;

        Ok(())
    }

    pub async fn fail_job(&self, id: &Uuid, error: &str) -> Result<()> {
        query!(
            r#"
            UPDATE sync_jobs 
            SET status = 'failed', 
                error_message = $1, 
                retry_count = retry_count + 1,
                completed_at = CURRENT_TIMESTAMP 
            WHERE id = $2
            "#,
            error,
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to mark job as failed")?;

        Ok(())
    }

    // Sync run operations
    pub async fn create_run(&self, job_id: &Uuid) -> Result<SyncRun> {
        let run_number: i32 = query!(
            "SELECT COALESCE(MAX(run_number), 0) + 1 as next_run FROM sync_runs WHERE job_id = $1",
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get next run number")?
        .next_run
        .unwrap_or(1);

        let run = query_as!(
            SyncRun,
            r#"
            INSERT INTO sync_runs (job_id, run_number, status)
            VALUES ($1, $2, 'running')
            RETURNING *
            "#,
            job_id,
            run_number
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create sync run")?;

        Ok(run)
    }

    pub async fn update_run_progress(&self, run_id: &Uuid, discovered: i32, processed: i32, failed: i32) -> Result<()> {
        query!(
            r#"
            UPDATE sync_runs 
            SET documents_discovered = $1, documents_processed = $2, documents_failed = $3
            WHERE id = $4
            "#,
            discovered,
            processed,
            failed,
            run_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update run progress")?;

        Ok(())
    }

    pub async fn complete_run(&self, run_id: &Uuid) -> Result<()> {
        query!(
            "UPDATE sync_runs SET status = 'completed', completed_at = CURRENT_TIMESTAMP WHERE id = $1",
            run_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to complete run")?;

        Ok(())
    }

    pub async fn get_runs(&self, job_id: &Uuid) -> Result<Vec<SyncRun>> {
        let runs = query_as!(
            SyncRun,
            "SELECT * FROM sync_runs WHERE job_id = $1 ORDER BY run_number DESC",
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get runs")?;

        Ok(runs)
    }
}

#[async_trait]
impl Repository<SyncJob> for SyncJobRepository {
    async fn create(&self, entity: &SyncJob) -> Result<SyncJob> {
        let job = query_as!(
            SyncJob,
            r#"
            INSERT INTO sync_jobs (id, user_id, account_id, connector_type, job_type, status, priority, config, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            entity.id,
            entity.user_id,
            entity.account_id,
            entity.connector_type,
            entity.job_type,
            entity.status,
            entity.priority,
            entity.config,
            entity.metadata
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create sync job")?;

        Ok(job)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<SyncJob>> {
        let job = query_as!(
            SyncJob,
            "SELECT * FROM sync_jobs WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find sync job")?;

        Ok(job)
    }

    async fn update(&self, id: &Uuid, entity: &SyncJob) -> Result<SyncJob> {
        let job = query_as!(
            SyncJob,
            r#"
            UPDATE sync_jobs
            SET status = $1, priority = $2, config = $3, metadata = $4, updated_at = CURRENT_TIMESTAMP
            WHERE id = $5
            RETURNING *
            "#,
            entity.status,
            entity.priority,
            entity.config,
            entity.metadata,
            id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to update sync job")?;

        Ok(job)
    }

    async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = query!(
            "DELETE FROM sync_jobs WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete sync job")?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, pagination: &Pagination) -> Result<PaginatedResult<SyncJob>> {
        let total: i64 = query!("SELECT COUNT(*) as count FROM sync_jobs")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count sync jobs")?
            .count
            .unwrap_or(0);

        let jobs = query_as!(
            SyncJob,
            "SELECT * FROM sync_jobs ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list sync jobs")?;

        Ok(PaginatedResult::new(jobs, total, pagination))
    }
}
