use crate::models::billing::*;
use crate::models::auth::SubscriptionTier;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration, Datelike};
use anyhow::{Result, anyhow};

pub struct BillingService;

impl BillingService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_subscription_plans(&self) -> Result<Vec<SubscriptionPlan>> {
        // Mock data for development
        Ok(vec![
            SubscriptionPlan {
                id: Uuid::new_v4(),
                name: "Free Plan".to_string(),
                description: Some("Perfect for getting started".to_string()),
                tier: SubscriptionTier::Free,
                price_monthly: rust_decimal::Decimal::new(0, 2),
                price_yearly: Some(rust_decimal::Decimal::new(0, 2)),
                features: serde_json::json!({"repositories": 3, "ai_queries": 100, "storage_gb": 1}),
                limits: serde_json::json!({"max_repositories": 3, "max_ai_queries_per_month": 100, "max_storage_gb": 1}),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            SubscriptionPlan {
                id: Uuid::new_v4(),
                name: "Personal Plan".to_string(),
                description: Some("For individual developers".to_string()),
                tier: SubscriptionTier::Personal,
                price_monthly: rust_decimal::Decimal::new(1999, 2),
                price_yearly: Some(rust_decimal::Decimal::new(19999, 2)),
                features: serde_json::json!({"repositories": 20, "ai_queries": 1000, "storage_gb": 10}),
                limits: serde_json::json!({"max_repositories": 20, "max_ai_queries_per_month": 1000, "max_storage_gb": 10}),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ])
    }

    pub async fn get_user_subscription(&self, _user_id: Uuid) -> Result<Option<SubscriptionWithPlan>> {
        // Mock data - no subscription for development
        Ok(None)
    }

    pub async fn create_subscription(&self, _user_id: Uuid, _request: CreateSubscriptionRequest) -> Result<UserSubscription> {
        // Mock subscription creation
        Ok(UserSubscription {
            id: Uuid::new_v4(),
            user_id: _user_id,
            plan_id: _request.plan_id,
            status: SubscriptionStatus::Active,
            current_period_start: Utc::now(),
            current_period_end: Utc::now() + Duration::days(30),
            trial_start: None,
            trial_end: None,
            cancel_at_period_end: false,
            cancelled_at: None,
            stripe_subscription_id: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn cancel_subscription(&self, _user_id: Uuid, _cancel_at_period_end: bool) -> Result<()> {
        // Mock cancellation
        Ok(())
    }

    pub async fn get_payment_methods(&self, _user_id: Uuid) -> Result<Vec<PaymentMethod>> {
        // Mock empty payment methods
        Ok(vec![])
    }

    pub async fn add_payment_method(&self, _user_id: Uuid, _request: CreatePaymentMethodRequest) -> Result<PaymentMethod> {
        // Mock payment method creation
        Ok(PaymentMethod {
            id: Uuid::new_v4(),
            user_id: _user_id,
            r#type: _request.r#type,
            is_default: _request.is_default.unwrap_or(false),
            stripe_payment_method_id: _request.stripe_payment_method_id,
            last_four: None,
            brand: None,
            exp_month: None,
            exp_year: None,
            billing_details: _request.billing_details.unwrap_or_else(|| serde_json::json!({})),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn get_invoices(&self, _user_id: Uuid, _limit: Option<i64>) -> Result<Vec<Invoice>> {
        // Mock empty invoices
        Ok(vec![])
    }

    pub async fn get_usage_tracking(&self, _user_id: Uuid, _period_start: DateTime<Utc>) -> Result<Vec<UsageTracking>> {
        // Mock usage data
        Ok(vec![
            UsageTracking {
                id: Uuid::new_v4(),
                user_id: _user_id,
                resource_type: "repositories".to_string(),
                usage_count: 0,
                period_start: _period_start,
                period_end: _period_start + Duration::days(30),
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            UsageTracking {
                id: Uuid::new_v4(),
                user_id: _user_id,
                resource_type: "ai_queries".to_string(),
                usage_count: 0,
                period_start: _period_start,
                period_end: _period_start + Duration::days(30),
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ])
    }

    pub async fn track_usage(&self, _user_id: Uuid, _resource_type: &str, _count: i32) -> Result<()> {
        // Mock usage tracking
        Ok(())
    }

    pub async fn get_billing_dashboard(&self, user_id: Uuid) -> Result<BillingDashboard> {
        let subscription = self.get_user_subscription(user_id).await?;
        let payment_methods = self.get_payment_methods(user_id).await?;
        let recent_invoices = self.get_invoices(user_id, Some(5)).await?;
        
        let current_month_start = Utc::now().date_naive().with_day(1).unwrap();
        let period_start = current_month_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let usage = self.get_usage_tracking(user_id, period_start).await?;

        Ok(BillingDashboard {
            subscription,
            payment_methods,
            recent_invoices,
            usage,
            billing_address: None,
        })
    }

    pub async fn check_usage_limits(&self, _user_id: Uuid, _resource_type: &str) -> Result<bool> {
        // Mock - always allow for development
        Ok(true)
    }
}