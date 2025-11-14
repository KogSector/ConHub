use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::Mutex;

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

lazy_static! {
    static ref USER_CONNECTIONS: Mutex<HashMap<String, Vec<ConnectionRecord>>> = Mutex::new(HashMap::new());
    static ref PLATFORM_CONFIG: Mutex<HashMap<String, ConfigureRequest>> = Mutex::new(HashMap::new());
}

pub async fn configure(req: web::Json<ConfigureRequest>) -> actix_web::Result<HttpResponse> {
    PLATFORM_CONFIG.lock().unwrap().insert(req.platform.clone(), req.into_inner());
    Ok(HttpResponse::Ok().json(ApiResponse::<ConnectionAccount> {
        success: true,
        message: "Platform configured".into(),
        data: None,
    }))
}

pub async fn connect(req: web::Json<ConnectRequest>) -> actix_web::Result<HttpResponse> {
    let platform = req.platform.clone();
    let auth_url = format!("https://auth.example/{platform}/oauth?client_id=dummy&state=conhub");
    let account = ConnectionAccount {
        status: "pending_auth".into(),
        credentials: Some(HashMap::from([("auth_url".into(), auth_url)])),
    };
    Ok(HttpResponse::Ok().json(ApiResponse { success: true, message: "Auth required".into(), data: Some(account) }))
}

pub async fn oauth_callback(req: web::Json<OAuthCallbackRequest>) -> actix_web::Result<HttpResponse> {
    let user_id = "user_123".to_string();
    let mut store = USER_CONNECTIONS.lock().unwrap();
    let list = store.entry(user_id.clone()).or_insert_with(Vec::new);
    list.push(ConnectionRecord {
        id: format!("conn_{}", chrono::Utc::now().timestamp_millis()),
        user_id,
        platform: req.platform.clone(),
        username: format!("{}-user", req.platform),
        is_active: true,
        connected_at: chrono::Utc::now(),
        last_sync: None,
    });
    Ok(HttpResponse::Ok().json(ApiResponse::<ConnectionRecord> { success: true, message: "Connected".into(), data: None }))
}

pub async fn list_connections() -> actix_web::Result<HttpResponse> {
    let user_id = "user_123".to_string();
    let store = USER_CONNECTIONS.lock().unwrap();
    let list = store.get(&user_id).cloned().unwrap_or_default();
    Ok(HttpResponse::Ok().json(ApiResponse { success: true, message: "ok".into(), data: Some(list) }))
}

pub async fn disconnect(path: web::Path<String>) -> actix_web::Result<HttpResponse> {
    let conn_id = path.into_inner();
    let user_id = "user_123".to_string();
    let mut store = USER_CONNECTIONS.lock().unwrap();
    if let Some(list) = store.get_mut(&user_id) {
        list.retain(|c| c.id != conn_id);
    }
    Ok(HttpResponse::Ok().json(ApiResponse::<()> { success: true, message: "Disconnected".into(), data: None }))
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
    cfg.service(
        web::scope("/api/security/connections")
            .route("", web::get().to(list_connections))
            .route("/configure", web::post().to(configure))
            .route("/connect", web::post().to(connect))
            .route("/oauth/callback", web::post().to(oauth_callback))
            .route("/{id}", web::delete().to(disconnect))
            .route("/{provider}/files", web::get().to(list_provider_files))
    );
}

