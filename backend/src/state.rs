use crate::config::AppConfig;
use crate::services::{
    auth_service::AuthService,
    billing_service::BillingService,
    data_service::DataService,
    indexing_service::IndexingService,
    security_service::SecurityService,
};
use sqlx::PgPool;
use std::sync::Arc;

pub struct AppState {
    pub db_pool: Option<PgPool>,
    pub redis_client: Option<redis::Client>,
    pub config: AppConfig,

    // Service instances
    pub auth_service: Arc<AuthService>,
    pub billing_service: Arc<BillingService>,
    pub data_service: Arc<DataService>,
    pub indexing_service: Arc<IndexingService>,
    pub security_service: Arc<SecurityService>,
}

impl AppState {
    pub async fn new(
        db_pool: Option<PgPool>,
        redis_client: Option<redis::Client>,
        config: AppConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize services
        let auth_service = Arc::new(AuthService::new(
            db_pool.clone(),
            redis_client.clone(),
            config.jwt_secret.clone(),
        ));

        let billing_service = Arc::new(BillingService::new(
            db_pool.clone(),
            config.stripe_secret_key.clone(),
        ));

        let data_service = Arc::new(DataService::new(
            db_pool.clone(),
            config.clone(),
        ));

        let indexing_service = Arc::new(IndexingService::new(
            config.clone(),
        ));

        let security_service = Arc::new(SecurityService::new(
            db_pool.clone(),
        ));

        Ok(Self {
            db_pool,
            redis_client,
            config,
            auth_service,
            billing_service,
            data_service,
            indexing_service,
            security_service,
        })
    }
}
