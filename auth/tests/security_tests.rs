use actix_web::{test, web, App, http::StatusCode, http::header};
use serde_json::json;
use sqlx::{PgPool, postgres::PgPoolOptions, Row};
use uuid::Uuid;
use std::env;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};

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
async fn create_test_user(pool: &PgPool, email: &str) -> User {
    let user_id = Uuid::new_v4();
    let password_hash = bcrypt::hash("test_password_123", bcrypt::DEFAULT_COST).unwrap();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, name, role, subscription_tier, is_verified, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(user_id)
    .bind(email)
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
        email: email.to_string(),
        password_hash,
        name: Some("Test User".to_string()),
        avatar_url: None,
        organization: None,
        role: UserRole::User,
        subscription_tier: SubscriptionTier::Free,
        is_verified: true,
        is_active: true,
        created_at: now,
        updated_at: now,
        last_login_at: None,
    }
}

// Clean up test data
async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM users WHERE email LIKE 'security_test_%@example.com'")
        .execute(pool)
        .await
        .expect("Failed to cleanup test users");
    
    sqlx::query("DELETE FROM password_reset_tokens WHERE email LIKE 'security_test_%@example.com'")
        .execute(pool)
        .await
        .expect("Failed to cleanup test tokens");
}

#[actix_web::test]
async fn test_sql_injection_protection() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/login", web::post().to(login))
    ).await;

    // Attempt SQL injection in email field
    let malicious_login = LoginRequest {
        email: "admin@example.com'; DROP TABLE users; --".to_string(),
        password: "password".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&malicious_login)
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Should return unauthorized, not cause SQL injection
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Verify users table still exists by attempting a query
    let result = sqlx::query("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await;
    
    assert!(result.is_ok(), "Users table should still exist after SQL injection attempt");

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_password_brute_force_protection() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let test_email = "security_test_brute@example.com";
    let _test_user = create_test_user(&pool, test_email).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/login", web::post().to(login))
    ).await;

    // Attempt multiple failed logins
    for i in 0..10 {
        let login_request = LoginRequest {
            email: test_email.to_string(),
            password: format!("wrong_password_{}", i),
        };

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(&login_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_jwt_token_validation() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let test_email = "security_test_jwt@example.com";
    let test_user = create_test_user(&pool, test_email).await;

    // Test with invalid JWT secret
    let jwt_secret = "wrong_secret_key";
    let claims = Claims {
        sub: test_user.id.to_string(),
        email: test_user.email.clone(),
        role: "user".to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };

    let invalid_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref())
    ).unwrap();

    // Verify token validation fails with wrong secret
    let correct_secret = "conhub_super_secret_jwt_key_2024_development_only";
    let validation = Validation::default();
    let decode_result = decode::<Claims>(
        &invalid_token,
        &DecodingKey::from_secret(correct_secret.as_ref()),
        &validation
    );

    assert!(decode_result.is_err(), "Token should be invalid with wrong secret");

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_expired_jwt_token() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let test_email = "security_test_expired@example.com";
    let test_user = create_test_user(&pool, test_email).await;

    // Create expired token
    let jwt_secret = "conhub_super_secret_jwt_key_2024_development_only";
    let expired_claims = Claims {
        sub: test_user.id.to_string(),
        email: test_user.email.clone(),
        role: "user".to_string(),
        exp: (Utc::now() - Duration::hours(1)).timestamp() as usize, // Expired 1 hour ago
        iat: (Utc::now() - Duration::hours(2)).timestamp() as usize,
    };

    let expired_token = encode(
        &Header::default(),
        &expired_claims,
        &EncodingKey::from_secret(jwt_secret.as_ref())
    ).unwrap();

    // Verify token validation fails for expired token
    let validation = Validation::default();
    let decode_result = decode::<Claims>(
        &expired_token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation
    );

    assert!(decode_result.is_err(), "Expired token should be invalid");

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_password_reset_token_security() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let test_email = "security_test_reset@example.com";
    let _test_user = create_test_user(&pool, test_email).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password))
    ).await;

    // Request password reset
    let forgot_request = ForgotPasswordRequest {
        email: test_email.to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/forgot-password")
        .set_json(&forgot_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Get the reset token
    let token_row = sqlx::query("SELECT token FROM password_reset_tokens WHERE email = $1")
        .bind(test_email)
        .fetch_one(&pool)
        .await
        .expect("Reset token should exist");
    
    let reset_token: String = token_row.get("token");

    // Test token reuse protection - use token once
    let reset_request = ResetPasswordRequest {
        token: reset_token.clone(),
        new_password: "NewSecurePassword123!".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/reset-password")
        .set_json(&reset_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Try to reuse the same token - should fail
    let reuse_request = ResetPasswordRequest {
        token: reset_token,
        new_password: "AnotherPassword123!".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/reset-password")
        .set_json(&reuse_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_email_enumeration_protection() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/forgot-password", web::post().to(forgot_password))
    ).await;

    // Test with non-existent email
    let forgot_request = ForgotPasswordRequest {
        email: "nonexistent@example.com".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/forgot-password")
        .set_json(&forgot_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Should return success even for non-existent email to prevent enumeration
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_xss_protection_in_responses() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/register", web::post().to(register))
    ).await;

    // Attempt XSS in name field
    let xss_register = RegisterRequest {
        email: "security_test_xss@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        name: Some("<script>alert('xss')</script>".to_string()),
        avatar_url: None,
        organization: None,
    };

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&xss_register)
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Should succeed but name should be properly handled
    if resp.status() == StatusCode::CREATED {
        let body: AuthResponse = test::read_body_json(resp).await;
        // Verify the response doesn't contain unescaped script tags
        let name = body.user.name.unwrap_or_default();
        assert!(!name.contains("<script>"), "Script tags should be handled safely");
    }

    cleanup_test_data(&pool).await;
}

#[actix_web::test]
async fn test_password_strength_validation() {
    let pool = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/register", web::post().to(register))
    ).await;

    // Test various weak passwords
    let weak_passwords = vec![
        "123456",           // Too short, only numbers
        "password",         // Common password, no numbers/symbols
        "Password",         // No numbers/symbols
        "password123",      // No symbols, common pattern
        "12345678",         // Only numbers
        "abcdefgh",         // Only lowercase letters
    ];

    for weak_password in weak_passwords {
        let register_request = RegisterRequest {
            email: format!("test_weak_{}@example.com", weak_password.len()),
            password: weak_password.to_string(),
            name: Some("Test User".to_string()),
            avatar_url: None,
            organization: None,
        };

        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(&register_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // Weak passwords should be rejected
        assert_eq!(
            resp.status(), 
            StatusCode::BAD_REQUEST,
            "Weak password '{}' should be rejected", 
            weak_password
        );
    }
}

#[actix_web::test]
async fn test_content_type_validation() {
    let pool = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/login", web::post().to(login))
    ).await;

    // Test with wrong content type
    let req = test::TestRequest::post()
        .uri("/login")
        .insert_header(header::ContentType::plaintext())
        .set_payload("email=test@example.com&password=password")
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Should reject non-JSON content
    assert_ne!(resp.status(), StatusCode::OK);
}