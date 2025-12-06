use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::PgPool;
use sqlx::Row;
use conhub_middleware::auth::extract_claims_from_request;
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct ConnectionRecord {
    pub id: String,
    pub user_id: String,
    pub platform: String,
    pub username: String,
    pub is_active: bool,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Clone)]
pub struct ConnectionAccount {
    pub status: String,
    pub credentials: Option<HashMap<String, String>>, // e.g., { auth_url }
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Deserialize)]
pub struct ConfigureRequest {
    pub platform: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: Option<String>,
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub platform: String,
    pub account_name: Option<String>,
}

#[derive(Deserialize)]
pub struct OAuthCallbackRequest {
    pub platform: String,
    pub code: String,
}

#[derive(Serialize, Clone)]
pub struct ProviderFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size: u64,
}

/// Extract user_id from Auth0 claims - handles both UUID format and Auth0 sub format
async fn get_user_id_from_request(req: &HttpRequest, pool: &PgPool) -> Option<Uuid> {
    let claims = extract_claims_from_request(req)?;
    
    // First try direct UUID parse (for ConHub-issued tokens)
    if let Ok(uuid) = claims.sub.parse::<Uuid>() {
        return Some(uuid);
    }
    
    // Auth0 sub format (e.g., "google-oauth2|123...") - look up in database
    let result = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE auth0_sub = $1 AND is_active = true"
    )
    .bind(&claims.sub)
    .fetch_optional(pool)
    .await
    .ok()?;
    
    result
}

pub async fn configure(req: web::Json<ConfigureRequest>) -> actix_web::Result<HttpResponse> {
    // Platform configuration is typically stored in env vars or database
    // For now, just acknowledge the request
    tracing::info!("Platform {} configuration received", req.platform);
    Ok(HttpResponse::Ok().json(ApiResponse::<ConnectionAccount> {
        success: true,
        message: "Platform configured".into(),
        data: None,
    }))
}

pub async fn connect(req: web::Json<ConnectRequest>) -> actix_web::Result<HttpResponse> {
    let platform = req.platform.clone();
    
    // Generate real OAuth URLs based on platform
    let auth_url = match platform.as_str() {
        "gmail" | "google_drive" => {
            let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
            let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());
            let scope = if platform == "gmail" {
                "https://www.googleapis.com/auth/gmail.readonly"
            } else {
                "https://www.googleapis.com/auth/drive.readonly"
            };
            format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent",
                client_id, redirect_uri, scope
            )
        }
        "dropbox" => {
            let client_id = std::env::var("DROPBOX_CLIENT_ID").unwrap_or_default();
            let redirect_uri = std::env::var("DROPBOX_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());
            format!(
                "https://www.dropbox.com/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&token_access_type=offline",
                client_id, redirect_uri
            )
        }
        "slack" => {
            let client_id = std::env::var("SLACK_CLIENT_ID").unwrap_or_default();
            let redirect_uri = std::env::var("SLACK_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());
            format!(
                "https://slack.com/oauth/v2/authorize?client_id={}&redirect_uri={}&scope=channels:read,channels:history,users:read",
                client_id, redirect_uri
            )
        }
        _ => {
            // For platforms handled by auth service (github, bitbucket, gitlab)
            format!("http://localhost:3010/api/auth/oauth/{}?redirect_uri=http://localhost:3000/auth/callback", platform)
        }
    };
    
    let account = ConnectionAccount {
        status: "pending_auth".into(),
        credentials: Some(HashMap::from([("auth_url".into(), auth_url)])),
    };
    Ok(HttpResponse::Ok().json(ApiResponse { success: true, message: "Auth required".into(), data: Some(account) }))
}

pub async fn oauth_callback(
    http_req: HttpRequest,
    req: web::Json<OAuthCallbackRequest>,
    pool_opt: web::Data<Option<PgPool>>,
) -> actix_web::Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                message: "Database unavailable".into(),
                data: None,
            }));
        }
    };
    
    let user_id = match get_user_id_from_request(&http_req, pool).await {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                message: "Authentication required".into(),
                data: None,
            }));
        }
    };
    
    // In a real implementation, we would exchange the code for tokens here
    // For now, create a connection record in the database
    let conn_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    let result = sqlx::query(
        r#"
        INSERT INTO social_connections (id, user_id, platform, platform_user_id, access_token, scope, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8)
        ON CONFLICT (user_id, platform) DO UPDATE SET
            access_token = EXCLUDED.access_token,
            is_active = true,
            updated_at = EXCLUDED.updated_at
        "#
    )
    .bind(conn_id)
    .bind(user_id)
    .bind(&req.platform)
    .bind(format!("{}-user", req.platform)) // platform_user_id placeholder
    .bind(&req.code) // In reality, this would be the exchanged access token
    .bind("read")
    .bind(now)
    .bind(now)
    .execute(pool)
    .await;
    
    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::<ConnectionRecord> {
            success: true,
            message: "Connected".into(),
            data: None,
        })),
        Err(e) => {
            tracing::error!("Failed to save connection: {}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: format!("Failed to save connection: {}", e),
                data: None,
            }))
        }
    }
}

pub async fn list_connections(
    req: HttpRequest,
    pool_opt: web::Data<Option<PgPool>>,
) -> actix_web::Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            // Return empty list if database is unavailable
            return Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(Vec::<ConnectionRecord>::new()),
            }));
        }
    };
    
    let user_id = match get_user_id_from_request(&req, pool).await {
        Some(id) => id,
        None => {
            // Return empty list if not authenticated (graceful degradation)
            tracing::warn!("list_connections: No authenticated user found");
            return Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(Vec::<ConnectionRecord>::new()),
            }));
        }
    };
    
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, platform, COALESCE(username, platform_user_id) as username, is_active, created_at, last_sync
        FROM social_connections
        WHERE user_id = $1 AND is_active = true
        ORDER BY created_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await;
    
    match rows {
        Ok(rows) => {
            let connections: Vec<ConnectionRecord> = rows.iter().map(|row| {
                ConnectionRecord {
                    id: row.get::<Uuid, _>("id").to_string(),
                    user_id: row.get::<Uuid, _>("user_id").to_string(),
                    platform: row.get("platform"),
                    username: row.get("username"),
                    is_active: row.get("is_active"),
                    connected_at: row.get("created_at"),
                    last_sync: row.get("last_sync"),
                }
            }).collect();
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(connections),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fetch connections: {}", e);
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(Vec::<ConnectionRecord>::new()),
            }))
        }
    }
}

pub async fn disconnect(
    req: HttpRequest,
    path: web::Path<String>,
    pool_opt: web::Data<Option<PgPool>>,
) -> actix_web::Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                message: "Database unavailable".into(),
                data: None,
            }));
        }
    };
    
    let user_id = match get_user_id_from_request(&req, pool).await {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                message: "Authentication required".into(),
                data: None,
            }));
        }
    };
    
    let conn_id_str = path.into_inner();
    let conn_id = match Uuid::parse_str(&conn_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                message: "Invalid connection ID".into(),
                data: None,
            }));
        }
    };
    
    let result = sqlx::query(
        "UPDATE social_connections SET is_active = false, updated_at = NOW() WHERE id = $1 AND user_id = $2"
    )
    .bind(conn_id)
    .bind(user_id)
    .execute(pool)
    .await;
    
    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::<()> {
            success: true,
            message: "Disconnected".into(),
            data: None,
        })),
        Err(e) => {
            tracing::error!("Failed to disconnect: {}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: format!("Failed to disconnect: {}", e),
                data: None,
            }))
        }
    }
}

pub async fn list_provider_files(path: web::Path<String>) -> actix_web::Result<HttpResponse> {
    let provider = path.into_inner();
    let files = vec![
        ProviderFile { id: format!("{provider}_file_1"), name: "Project Brief.pdf".into(), mime_type: "application/pdf".into(), size: 512_000 },
        ProviderFile { id: format!("{provider}_file_2"), name: "Roadmap.md".into(), mime_type: "text/markdown".into(), size: 8_000 },
        ProviderFile { id: format!("{provider}_file_3"), name: "Design.png".into(), mime_type: "image/png".into(), size: 220_000 },
    ];
    Ok(HttpResponse::Ok().json(ApiResponse { success: true, message: "ok".into(), data: Some(files) }))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::resource("/api/security/connections")
                .route(web::get().to(list_connections)),
        )
        .service(
            web::resource("/api/security/connections/configure")
                .route(web::post().to(configure)),
        )
        .service(
            web::resource("/api/security/connections/connect")
                .route(web::post().to(connect)),
        )
        .service(
            web::resource("/api/security/connections/oauth/callback")
                .route(web::post().to(oauth_callback)),
        )
        .service(
            web::resource("/api/security/connections/{id}")
                .route(web::delete().to(disconnect)),
        )
        .service(
            web::resource("/api/security/connections/{provider}/files")
                .route(web::get().to(list_provider_files)),
        );
}
