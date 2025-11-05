use sqlx::PgPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Subscription {
    pub id: String,
    pub user_id: String,
    pub plan_id: String,
    pub status: String,
}

#[derive(Debug)]
pub enum BillingError {
    SubscriptionNotFound,
    StripeError(String),
    DatabaseError(String),
}

impl std::fmt::Display for BillingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BillingError::SubscriptionNotFound => write!(f, "Subscription not found"),
            BillingError::StripeError(msg) => write!(f, "Stripe error: {}", msg),
            BillingError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for BillingError {}

pub struct BillingService {
    db_pool: Option<PgPool>,
    stripe_secret_key: Option<String>,
}

impl BillingService {
    pub fn new(db_pool: Option<PgPool>, stripe_secret_key: Option<String>) -> Self {
        Self {
            db_pool,
            stripe_secret_key,
        }
    }

    pub async fn create_subscription(&self, user_id: &str, plan_id: &str) -> Result<Subscription, BillingError> {
        // TODO: Call conhub-billing module when it's created
        log::info!("Creating subscription for user: {}", user_id);

        Err(BillingError::StripeError("Not implemented".to_string()))
    }

    pub async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), BillingError> {
        // TODO: Call conhub-billing module when it's created
        log::info!("Canceling subscription: {}", subscription_id);

        Ok(())
    }

    pub async fn get_user_subscription(&self, user_id: &str) -> Result<Subscription, BillingError> {
        // TODO: Call conhub-billing module when it's created
        log::info!("Getting subscription for user: {}", user_id);

        Err(BillingError::SubscriptionNotFound)
    }
}
