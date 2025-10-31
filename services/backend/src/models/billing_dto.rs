use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubscriptionRequest {
    #[validate(length(min = 1))]
    pub plan_id: String,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub user_id: String,
    pub plan_id: String,
    pub status: String,
    pub current_period_start: Option<i64>,
    pub current_period_end: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: String,
    pub name: String,
    pub price: i64,
    pub currency: String,
    pub interval: String,
    pub features: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CancelSubscriptionRequest {
    pub immediately: Option<bool>,
}
