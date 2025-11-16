use conhub_models::billing::*;
use crate::errors::ServiceError;
use uuid::Uuid;
use sqlx::PgPool;
use tracing::{info, error};

pub struct BillingServiceDb {
    pool: PgPool,
}

impl BillingServiceDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_subscription_plans(&self) -> Result<Vec<SubscriptionPlan>, ServiceError> {
        let plans = sqlx::query_as!(
            SubscriptionPlan,
            r#"
            SELECT 
                id,
                name,
                description,
                tier,
                price_monthly,
                price_yearly,
                features,
                limits,
                is_active,
                stripe_price_id,
                created_at,
                updated_at
            FROM subscription_plans 
            WHERE is_active = true 
            ORDER BY price_monthly ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch subscription plans: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(plans)
    }

    pub async fn get_subscription(&self, user_id: Uuid) -> Result<Option<UserSubscription>, ServiceError> {
        let subscription = sqlx::query_as!(
            UserSubscription,
            r#"
            SELECT 
                id,
                user_id,
                plan_id,
                status as "status: SubscriptionStatus",
                current_period_start,
                current_period_end,
                trial_start,
                trial_end,
                cancel_at_period_end,
                cancelled_at,
                stripe_subscription_id,
                stripe_customer_id,
                metadata,
                created_at,
                updated_at
            FROM user_subscriptions 
            WHERE user_id = $1 AND status != 'cancelled'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch user subscription: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(subscription)
    }

    pub async fn get_payment_methods(&self, user_id: Uuid) -> Result<Vec<PaymentMethod>, ServiceError> {
        let payment_methods = sqlx::query_as!(
            PaymentMethod,
            r#"
            SELECT 
                id,
                user_id,
                type as "r#type: PaymentMethodType",
                is_default,
                stripe_payment_method_id,
                last_four,
                brand,
                exp_month,
                exp_year,
                billing_details,
                is_active,
                created_at,
                updated_at
            FROM payment_methods 
            WHERE user_id = $1 AND is_active = true
            ORDER BY is_default DESC, created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch payment methods: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(payment_methods)
    }

    pub async fn get_invoices(&self, user_id: Uuid) -> Result<Vec<Invoice>, ServiceError> {
        let invoices = sqlx::query_as!(
            Invoice,
            r#"
            SELECT 
                id,
                user_id,
                subscription_id,
                invoice_number,
                status as "status: InvoiceStatus",
                amount_due,
                amount_paid,
                currency,
                due_date,
                paid_at,
                stripe_invoice_id,
                metadata,
                created_at,
                updated_at
            FROM invoices 
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 10
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch invoices: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(invoices)
    }

    pub async fn get_usage_tracking(&self, user_id: Uuid) -> Result<Vec<UsageRecord>, ServiceError> {
        let current_month_start = chrono::Utc::now()
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        let usage_records = sqlx::query_as!(
            UsageRecord,
            r#"
            SELECT 
                id,
                user_id,
                resource_type,
                usage_count,
                period_start,
                period_end,
                metadata,
                created_at,
                updated_at
            FROM usage_tracking 
            WHERE user_id = $1 AND period_start >= $2
            ORDER BY resource_type, period_start DESC
            "#,
            user_id,
            current_month_start
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch usage tracking: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(usage_records)
    }

    pub async fn get_billing_dashboard(&self, user_id: Uuid) -> Result<serde_json::Value, ServiceError> {
        info!("Getting billing dashboard for user: {}", user_id);

        // Fetch subscription with plan details
        let subscription_with_plan = sqlx::query!(
            r#"
            SELECT 
                s.id as subscription_id,
                s.status,
                s.current_period_end,
                s.cancel_at_period_end,
                p.name as plan_name,
                p.tier,
                p.price_monthly,
                p.features,
                p.limits
            FROM user_subscriptions s
            JOIN subscription_plans p ON s.plan_id = p.id
            WHERE s.user_id = $1 AND s.status != 'cancelled'
            ORDER BY s.created_at DESC
            LIMIT 1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch subscription with plan: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        // Fetch payment methods
        let payment_methods = self.get_payment_methods(user_id).await?;

        // Fetch recent invoices
        let recent_invoices = self.get_invoices(user_id).await?;

        // Fetch usage data
        let usage = self.get_usage_tracking(user_id).await?;

        // Build dashboard response
        let subscription_data = if let Some(sub) = subscription_with_plan {
            serde_json::json!({
                "subscription": {
                    "id": sub.subscription_id,
                    "status": sub.status,
                    "current_period_end": sub.current_period_end,
                    "cancel_at_period_end": sub.cancel_at_period_end
                },
                "plan": {
                    "name": sub.plan_name,
                    "tier": sub.tier,
                    "price_monthly": sub.price_monthly,
                    "features": sub.features,
                    "limits": sub.limits
                }
            })
        } else {
            serde_json::Value::Null
        };

        let dashboard = serde_json::json!({
            "subscription": subscription_data,
            "payment_methods": payment_methods,
            "recent_invoices": recent_invoices,
            "usage": usage
        });

        Ok(dashboard)
    }

    pub async fn create_subscription(&self, user_id: Uuid, request: &CreateSubscriptionRequest) -> Result<UserSubscription, ServiceError> {
        info!("Creating subscription for user: {} with plan: {}", user_id, request.plan_id);

        let subscription = sqlx::query_as!(
            UserSubscription,
            r#"
            INSERT INTO user_subscriptions (
                user_id, 
                plan_id, 
                status, 
                current_period_start, 
                current_period_end,
                stripe_subscription_id,
                stripe_customer_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
                id,
                user_id,
                plan_id,
                status as "status: SubscriptionStatus",
                current_period_start,
                current_period_end,
                trial_start,
                trial_end,
                cancel_at_period_end,
                cancelled_at,
                stripe_subscription_id,
                stripe_customer_id,
                metadata,
                created_at,
                updated_at
            "#,
            user_id,
            request.plan_id,
            SubscriptionStatus::Active as SubscriptionStatus,
            chrono::Utc::now(),
            chrono::Utc::now() + chrono::Duration::days(30),
            request.stripe_subscription_id,
            request.stripe_customer_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create subscription: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(subscription)
    }

    pub async fn add_payment_method(&self, user_id: Uuid, payment_method: CreatePaymentMethodRequest) -> Result<PaymentMethod, ServiceError> {
        let payment_method_result = sqlx::query_as!(
            PaymentMethod,
            r#"
            INSERT INTO payment_methods (
                user_id,
                type,
                is_default,
                stripe_payment_method_id,
                last_four,
                brand,
                exp_month,
                exp_year,
                billing_details
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING 
                id,
                user_id,
                type as "r#type: PaymentMethodType",
                is_default,
                stripe_payment_method_id,
                last_four,
                brand,
                exp_month,
                exp_year,
                billing_details,
                is_active,
                created_at,
                updated_at
            "#,
            user_id,
            payment_method.r#type as PaymentMethodType,
            payment_method.is_default.unwrap_or(false),
            payment_method.stripe_payment_method_id,
            payment_method.last_four,
            payment_method.brand,
            payment_method.exp_month,
            payment_method.exp_year,
            payment_method.billing_details.unwrap_or_else(|| serde_json::json!({}))
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to add payment method: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(payment_method_result)
    }

    pub async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), ServiceError> {
        let subscription_uuid = Uuid::parse_str(subscription_id)
            .map_err(|_| ServiceError::BadRequest("Invalid subscription ID format".to_string()))?;

        sqlx::query!(
            r#"
            UPDATE user_subscriptions 
            SET 
                status = 'cancelled',
                cancelled_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
            subscription_uuid
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to cancel subscription: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(())
    }

    pub async fn track_usage(&self, user_id: Uuid, resource_type: &str, usage_count: i32) -> Result<(), ServiceError> {
        let current_month_start = chrono::Utc::now()
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        let current_month_end = if current_month_start.month() == 12 {
            current_month_start
                .with_year(current_month_start.year() + 1)
                .unwrap()
                .with_month(1)
                .unwrap()
        } else {
            current_month_start
                .with_month(current_month_start.month() + 1)
                .unwrap()
        };

        sqlx::query!(
            r#"
            INSERT INTO usage_tracking (user_id, resource_type, usage_count, period_start, period_end)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, resource_type, period_start)
            DO UPDATE SET 
                usage_count = usage_tracking.usage_count + $3,
                updated_at = CURRENT_TIMESTAMP
            "#,
            user_id,
            resource_type,
            usage_count,
            current_month_start,
            current_month_end
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to track usage: {}", e);
            ServiceError::DatabaseError(e.to_string())
        })?;

        Ok(())
    }
}
