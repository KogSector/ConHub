use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Active,
    Cancelled,
    PastDue,
    Unpaid,
    Trialing,
    Incomplete,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Void,
    Uncollectible,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum PaymentMethodType {
    Card,
    BankAccount,
    Paypal,
    Stripe,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tier: crate::models::auth::SubscriptionTier,
    pub price_monthly: Decimal,
    pub price_yearly: Option<Decimal>,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub status: SubscriptionStatus,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub stripe_subscription_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: PaymentMethodType,
    pub is_default: bool,
    pub stripe_payment_method_id: Option<String>,
    pub last_four: Option<String>,
    pub brand: Option<String>,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
    pub billing_details: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub invoice_number: String,
    pub status: InvoiceStatus,
    pub amount_due: Decimal,
    pub amount_paid: Decimal,
    pub tax_amount: Decimal,
    pub currency: String,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub stripe_invoice_id: Option<String>,
    pub hosted_invoice_url: Option<String>,
    pub invoice_pdf_url: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub payment_method_id: Option<Uuid>,
    pub amount: Decimal,
    pub currency: String,
    pub status: PaymentStatus,
    pub stripe_payment_intent_id: Option<String>,
    pub failure_reason: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
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
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BillingAddress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubscriptionRequest {
    pub plan_id: Uuid,
    pub payment_method_id: Option<Uuid>,
    pub coupon_code: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSubscriptionRequest {
    pub plan_id: Option<Uuid>,
    pub cancel_at_period_end: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaymentMethodRequest {
    pub r#type: PaymentMethodType,
    pub stripe_payment_method_id: Option<String>,
    pub is_default: Option<bool>,
    pub billing_details: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBillingAddressRequest {
    #[validate(length(min = 1, max = 255))]
    pub line1: String,
    #[validate(length(max = 255))]
    pub line2: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub city: String,
    #[validate(length(max = 100))]
    pub state: Option<String>,
    #[validate(length(min = 1, max = 20))]
    pub postal_code: String,
    #[validate(length(min = 2, max = 2))]
    pub country: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionWithPlan {
    pub subscription: UserSubscription,
    pub plan: SubscriptionPlan,
}

#[derive(Debug, Serialize)]
pub struct BillingDashboard {
    pub subscription: Option<SubscriptionWithPlan>,
    pub payment_methods: Vec<PaymentMethod>,
    pub recent_invoices: Vec<Invoice>,
    pub usage: Vec<UsageTracking>,
    pub billing_address: Option<BillingAddress>,
}

#[derive(Debug, Serialize)]
pub struct UsageSummary {
    pub repositories: i32,
    pub ai_queries: i32,
    pub storage_gb: i32,
    pub limits: serde_json::Value,
}

impl SubscriptionStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, SubscriptionStatus::Active | SubscriptionStatus::Trialing)
    }
}

impl PaymentStatus {
    pub fn is_successful(&self) -> bool {
        matches!(self, PaymentStatus::Completed)
    }
}