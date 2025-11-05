use actix_web::{HttpResponse, web};

use crate::models::HealthResponse;
use conhub_config::feature_toggles::FeatureToggles;

pub async fn health_handler(toggles: web::Data<FeatureToggles>) -> HttpResponse {
    let heavy_enabled = toggles.is_enabled("Heavy");
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        model_loaded: heavy_enabled,
    })
}
