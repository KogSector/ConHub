use actix_web::{test, web, App, http::StatusCode};
use serde_json::json;
use sqlx::{PgPool, postgres::PgPoolOptions, Row};
use uuid::Uuid;
use std::env;

use conhub_auth::handlers::auth::{login, register, forgot_password, reset_password};
use conhub_models::auth::*;

// Test database setup
async fn setup_test_db() -> PgPool {
    let database_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/conhub_test".to_string());
    
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

// Helper function to create test user
async fn create_test_user(pool: &PgPool) -> User {
    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", user_id);
    let password_hash = bcrypt::hash("test_password_123", bcrypt::DEFAULT_COST).unwrap();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, name, role, subscription_tier, is_verified, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(user_id)
    .bind(&email)
    .bind(&password_hash)
    .bind("Test User")
    .bind("user")
    .bind("free")
    .bind(true)
    .bind(true)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("Failed to create test user");

    User {
        id: user_id,
        email,
        password_hash,
        name: "Test User".to_string(),
        avatar_url: None,
        organization: None,
        role: UserRole::User,
        subscription_tier: SubscriptionTier::Free,
        is_verified: true,
        is_active: true,
        is_locked: false,
        failed_login_attempts: 0,
        locked_until: None,
        password_changed_at: now,
        email_verified_at: Some(now),
        two_factor_enabled: false,
        two_factor_secret: None,
        backup_codes: None,
        created_at: now,
        updated_at: now,
        last_login_at: None,
        last_login_ip: None,
        last_password_reset: None,
    }
}

// Clean up test data
async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM users WHERE email LIKE 'test_%@example.com'")
        .execute(pool)
        .await
        .expect("Failed to cleanup test users");
    
    sqlx::query("DELETE FROM password_reset_tokens WHERE email LIKE 'test_%@example.com'")
        .execute(pool)
        .await
        .expect("Failed to cleanup test tokens");
}

#[actix_web::test]
async fn test_user_registration_success() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/register", web::post().to(register))
    ).await;

    let unique_email = format!("test_{}@example.com", Uuid::new_v4());
    let register_request = RegisterRequest {
        email: unique_email.clone(),
        password: "SecurePassword123!".to_string(),
        name: "Test User".to_string(),
        avatar_url: None,
        organization: None,
    };

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&register_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: AuthResponse = test::read_body_json(resp).await;
    assert_eq!(body.user.email, unique_email);
    assert!(!body.token.is_empty());

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_user_registration_duplicate_email() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let test_user = create_test_user(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/register", web::post().to(register))
    ).await;

    let register_request = RegisterRequest {
        email: test_user.email.clone(),
        password: "SecurePassword123!".to_string(),
        name: "Test User".to_string(),
        avatar_url: None,
        organization: None,
    };

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&register_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_user_login_success() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let test_user = create_test_user(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/login", web::post().to(login))
    ).await;

    let login_request = LoginRequest {
        email: test_user.email.clone(),
        password: "test_password_123".to_string(),
        two_factor_code: None,
        remember_me: None,
        device_info: None,
    };

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: AuthResponse = test::read_body_json(resp).await;
    assert_eq!(body.user.email, test_user.email);
    assert!(!body.token.is_empty());

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_user_login_invalid_credentials() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let test_user = create_test_user(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/login", web::post().to(login))
    ).await;

    let login_request = LoginRequest {
        email: test_user.email.clone(),
        password: "wrong_password".to_string(),
        two_factor_code: None,
        remember_me: None,
        device_info: None,
    };

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_password_reset_flow() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let test_user = create_test_user(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password))
    ).await;

    // Test forgot password
    let forgot_request = ForgotPasswordRequest {
        email: test_user.email.clone(),
    };

    let req = test::TestRequest::post()
        .uri("/forgot-password")
        .set_json(&forgot_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Get the reset token from database
    let token_row = sqlx::query("SELECT token FROM password_reset_tokens WHERE email = $1")
        .bind(&test_user.email)
        .fetch_one(&pool)
        .await
        .expect("Reset token should exist");
    
    let reset_token: String = token_row.get("token");

    // Test reset password
    let reset_request = ResetPasswordRequest {
        token: reset_token,
        new_password: "NewSecurePassword123!".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/reset-password")
        .set_json(&reset_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify password was changed by trying to login with new password
    let login_request = LoginRequest {
        email: test_user.email.clone(),
        password: "NewSecurePassword123!".to_string(),
        two_factor_code: None,
        remember_me: None,
        device_info: None,
    };

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_invalid_reset_token() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/reset-password", web::post().to(reset_password))
    ).await;

    let reset_request = ResetPasswordRequest {
        token: "invalid_token_123".to_string(),
        new_password: "NewSecurePassword123!".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/reset-password")
        .set_json(&reset_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_validation_errors() {
    let pool = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    ).await;

    // Test invalid email format
    let invalid_register = RegisterRequest {
        email: "invalid_email".to_string(),
        password: "SecurePassword123!".to_string(),
        name: "Test User".to_string(),
        avatar_url: None,
        organization: None,
    };

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&invalid_register)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Test weak password
    let weak_password_register = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "123".to_string(),
        name: "Test User".to_string(),
        avatar_url: None,
        organization: None,
    };

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&weak_password_register)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}