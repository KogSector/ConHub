use sqlx::PgPool;
use anyhow::Result;

use conhub_models::auth::{User, RegisterRequest, LoginRequest};
use crate::services::users::UserService;

pub struct LocalAuthService {
    user_service: UserService,
}

impl LocalAuthService {
    pub async fn new(pool: PgPool) -> Result<Self> {
        let user_service = UserService::new(pool).await?;
        Ok(Self {
            user_service,
        })
    }

    
    pub async fn register(&self, request: &RegisterRequest) -> Result<User> {
        self.user_service.create_user(request).await
    }

    
    pub async fn login(&self, request: &LoginRequest) -> Result<User> {
        self.user_service.verify_password(&request.email, &request.password).await
    }
}
