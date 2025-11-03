pub mod integration_tests;
pub mod security_tests;

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment (logging, etc.)
pub fn init_test_env() {
    INIT.call_once(|| {
        // Initialize logging for tests
        env_logger::init();
        
        // Set test environment variables if needed
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("JWT_SECRET", "conhub_super_secret_jwt_key_2024_development_only");
    });
}

#[cfg(test)]
mod test_utils {
    use sqlx::PgPool;
    use uuid::Uuid;
    
    /// Helper to generate unique test emails
    pub fn generate_test_email() -> String {
        format!("test_{}@example.com", Uuid::new_v4())
    }
    
    /// Helper to clean up all test data
    pub async fn cleanup_all_test_data(pool: &PgPool) {
        let _ = sqlx::query("DELETE FROM password_reset_tokens WHERE email LIKE '%@example.com'")
            .execute(pool)
            .await;
            
        let _ = sqlx::query("DELETE FROM users WHERE email LIKE '%@example.com'")
            .execute(pool)
            .await;
    }
}