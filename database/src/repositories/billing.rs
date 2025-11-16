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
        sqlx::query_as::<_, SubscriptionPlan>(
            "SELECT id, name, description, tier, price_monthly, price_yearly, features, limits, is_active, stripe_price_id, created_at, updated_at FROM subscription_plans WHERE is_active = TRUE ORDER BY price_monthly"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get plans")
    }

    pub async fn get_user_subscription(&self, user_id: &Uuid) -> Result<Option<UserSubscription>> {
        query_as!(UserSubscription, "SELECT id, user_id, plan_id, status, current_period_start, current_period_end, trial_start, trial_end, cancel_at_period_end, cancelled_at, stripe_subscription_id, stripe_customer_id, metadata, created_at, updated_at FROM user_subscriptions WHERE user_id = $1 AND status = 'active'", user_id)
            .fetch_optional(&self.pool).await.context("Failed to get subscription")
    }

    pub async fn create_subscription(&self, user_id: &Uuid, plan_id: &Uuid) -> Result<UserSubscription> {
        query_as!(UserSubscription, 
            "INSERT INTO user_subscriptions (user_id, plan_id, status, current_period_start, current_period_end) VALUES ($1, $2, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP + INTERVAL '1 month') RETURNING id, user_id, plan_id, status, current_period_start, current_period_end, trial_start, trial_end, cancel_at_period_end, cancelled_at, stripe_subscription_id, stripe_customer_id, metadata, created_at, updated_at",
            user_id, plan_id)
            .fetch_one(&self.pool).await.context("Failed to create subscription")
    }

    pub async fn get_payment_methods(&self, user_id: &Uuid) -> Result<Vec<PaymentMethod>> {
        query_as!(PaymentMethod, "SELECT id, user_id, type, is_default, stripe_payment_method_id, last_four, brand, exp_month, exp_year, billing_details, is_active, created_at, updated_at FROM payment_methods WHERE user_id = $1 ORDER BY is_default DESC, created_at DESC", user_id)
            .fetch_all(&self.pool).await.context("Failed to get payment methods")
    }

    pub async fn get_invoices(&self, user_id: &Uuid, pagination: &Pagination) -> Result<PaginatedResult<Invoice>> {
        let total: i64 = query!("SELECT COUNT(*) as count FROM invoices WHERE user_id = $1", user_id)
            .fetch_one(&self.pool).await?.count.unwrap_or(0);
        let invoices = sqlx::query_as::<_, Invoice>(
            "SELECT id, user_id, subscription_id, invoice_number, status, amount_due, amount_paid, currency, due_date, paid_at, stripe_invoice_id, metadata, created_at, updated_at FROM invoices WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(PaginatedResult::new(invoices, total, pagination))
    }
}

#[async_trait]
impl Repository<UserSubscription> for BillingRepository {
    async fn create(&self, entity: &UserSubscription) -> Result<UserSubscription> {
        query_as!(UserSubscription, "INSERT INTO user_subscriptions (id, user_id, plan_id, status, current_period_start, current_period_end) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, user_id, plan_id, status, current_period_start, current_period_end, trial_start, trial_end, cancel_at_period_end, cancelled_at, stripe_subscription_id, stripe_customer_id, metadata, created_at, updated_at",
            entity.id, entity.user_id, entity.plan_id, entity.status, entity.current_period_start, entity.current_period_end)
            .fetch_one(&self.pool).await.context("Failed to create")
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<UserSubscription>> {
        query_as!(UserSubscription, "SELECT id, user_id, plan_id, status, current_period_start, current_period_end, trial_start, trial_end, cancel_at_period_end, cancelled_at, stripe_subscription_id, stripe_customer_id, metadata, created_at, updated_at FROM user_subscriptions WHERE id = $1", id)
            .fetch_optional(&self.pool).await.context("Failed to find")
    }

    async fn update(&self, id: &Uuid, entity: &UserSubscription) -> Result<UserSubscription> {
        query_as!(UserSubscription, "UPDATE user_subscriptions SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 RETURNING id, user_id, plan_id, status, current_period_start, current_period_end, trial_start, trial_end, cancel_at_period_end, cancelled_at, stripe_subscription_id, stripe_customer_id, metadata, created_at, updated_at", entity.status, id)
            .fetch_one(&self.pool).await.context("Failed to update")
    }

    async fn delete(&self, id: &Uuid) -> Result<bool> {
        Ok(query!("DELETE FROM user_subscriptions WHERE id = $1", id).execute(&self.pool).await?.rows_affected() > 0)
    }

    async fn list(&self, pagination: &Pagination) -> Result<PaginatedResult<UserSubscription>> {
        let total: i64 = query!("SELECT COUNT(*) as count FROM user_subscriptions").fetch_one(&self.pool).await?.count.unwrap_or(0);
        let subs = query_as!(UserSubscription, "SELECT id, user_id, plan_id, status, current_period_start, current_period_end, trial_start, trial_end, cancel_at_period_end, cancelled_at, stripe_subscription_id, stripe_customer_id, metadata, created_at, updated_at FROM user_subscriptions ORDER BY created_at DESC LIMIT $1 OFFSET $2", pagination.limit, pagination.offset)
            .fetch_all(&self.pool).await?;
        Ok(PaginatedResult::new(subs, total, pagination))
    }
}
