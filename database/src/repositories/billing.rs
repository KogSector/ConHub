use async_trait::async_trait;
use anyhow::{Result, Context};
use sqlx::{PgPool, query_as, query};
use uuid::Uuid;

use crate::models::{UserSubscription, SubscriptionPlan, PaymentMethod, Invoice, Model, Pagination, PaginatedResult};
use super::Repository;

pub struct BillingRepository {
    pool: PgPool,
}

impl BillingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_plans(&self) -> Result<Vec<SubscriptionPlan>> {
        query_as!(SubscriptionPlan, "SELECT * FROM subscription_plans WHERE is_active = TRUE ORDER BY price_monthly")
            .fetch_all(&self.pool).await.context("Failed to get plans")
    }

    pub async fn get_user_subscription(&self, user_id: &Uuid) -> Result<Option<UserSubscription>> {
        query_as!(UserSubscription, "SELECT * FROM user_subscriptions WHERE user_id = $1 AND status = 'active'", user_id)
            .fetch_optional(&self.pool).await.context("Failed to get subscription")
    }

    pub async fn create_subscription(&self, user_id: &Uuid, plan_id: &Uuid, billing_cycle: &str) -> Result<UserSubscription> {
        query_as!(UserSubscription, 
            "INSERT INTO user_subscriptions (user_id, plan_id, status, billing_cycle, current_period_start, current_period_end) VALUES ($1, $2, 'active', $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP + INTERVAL '1 month') RETURNING *",
            user_id, plan_id, billing_cycle)
            .fetch_one(&self.pool).await.context("Failed to create subscription")
    }

    pub async fn get_payment_methods(&self, user_id: &Uuid) -> Result<Vec<PaymentMethod>> {
        query_as!(PaymentMethod, "SELECT * FROM payment_methods WHERE user_id = $1 ORDER BY is_default DESC, created_at DESC", user_id)
            .fetch_all(&self.pool).await.context("Failed to get payment methods")
    }

    pub async fn get_invoices(&self, user_id: &Uuid, pagination: &Pagination) -> Result<PaginatedResult<Invoice>> {
        let total: i64 = query!("SELECT COUNT(*) as count FROM invoices WHERE user_id = $1", user_id)
            .fetch_one(&self.pool).await?.count.unwrap_or(0);
        let invoices = query_as!(Invoice, "SELECT * FROM invoices WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3", 
            user_id, pagination.limit, pagination.offset)
            .fetch_all(&self.pool).await?;
        Ok(PaginatedResult::new(invoices, total, pagination))
    }
}

#[async_trait]
impl Repository<UserSubscription> for BillingRepository {
    async fn create(&self, entity: &UserSubscription) -> Result<UserSubscription> {
        query_as!(UserSubscription, "INSERT INTO user_subscriptions (id, user_id, plan_id, status, billing_cycle, current_period_start, current_period_end) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
            entity.id, entity.user_id, entity.plan_id, entity.status, entity.billing_cycle, entity.current_period_start, entity.current_period_end)
            .fetch_one(&self.pool).await.context("Failed to create")
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<UserSubscription>> {
        query_as!(UserSubscription, "SELECT * FROM user_subscriptions WHERE id = $1", id)
            .fetch_optional(&self.pool).await.context("Failed to find")
    }

    async fn update(&self, id: &Uuid, entity: &UserSubscription) -> Result<UserSubscription> {
        query_as!(UserSubscription, "UPDATE user_subscriptions SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 RETURNING *", entity.status, id)
            .fetch_one(&self.pool).await.context("Failed to update")
    }

    async fn delete(&self, id: &Uuid) -> Result<bool> {
        Ok(query!("DELETE FROM user_subscriptions WHERE id = $1", id).execute(&self.pool).await?.rows_affected() > 0)
    }

    async fn list(&self, pagination: &Pagination) -> Result<PaginatedResult<UserSubscription>> {
        let total: i64 = query!("SELECT COUNT(*) as count FROM user_subscriptions").fetch_one(&self.pool).await?.count.unwrap_or(0);
        let subs = query_as!(UserSubscription, "SELECT * FROM user_subscriptions ORDER BY created_at DESC LIMIT $1 OFFSET $2", pagination.limit, pagination.offset)
            .fetch_all(&self.pool).await?;
        Ok(PaginatedResult::new(subs, total, pagination))
    }
}
