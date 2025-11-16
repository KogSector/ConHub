use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tier: String,
    pub price_monthly: rust_decimal::Decimal,
    pub price_yearly: Option<rust_decimal::Decimal>,
    pub features: Option<serde_json::Value>,
    pub limits: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub stripe_price_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: Option<bool>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub stripe_subscription_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: String,
    pub is_default: Option<bool>,
    pub stripe_payment_method_id: Option<String>,
    pub last_four: Option<String>,
    pub brand: Option<String>,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
    pub billing_details: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub invoice_number: String,
    pub status: String,
    pub amount_due: rust_decimal::Decimal,
    pub amount_paid: Option<rust_decimal::Decimal>,
    pub currency: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub stripe_invoice_id: Option<String>,
    #[sqlx(skip)]
    pub hosted_invoice_url: Option<String>,
    #[sqlx(skip)]
    pub invoice_pdf_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UsageTracking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub resource_type: String,
    pub usage_count: i32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl super::Model for UserSubscription {
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
