use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use url::Url;
use lazy_static::lazy_static;

#[derive(Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Clone)]
pub struct UrlRecord {
    pub id: String,
    pub user_id: String,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Serialize)]
pub struct CreateUrlResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<UrlRecord>,
}

// Thread-safe in-memory storage for demo (replace with actual database)
lazy_static! {
    static ref URL_STORAGE: Mutex<HashMap<String, Vec<UrlRecord>>> = Mutex::new(HashMap::new());
}

pub async fn create_url(req: web::Json<CreateUrlRequest>) -> Result<HttpResponse> {
    log::info!("Received create URL request: {:?}", req.url);
    
    // Validate URL
    if req.url.trim().is_empty() {
        log::warn!("Empty URL provided");
        return Ok(HttpResponse::BadRequest().json(CreateUrlResponse {
            success: false,
            message: "URL is required".to_string(),
            data: None,
        }));
    }

    // Validate URL format
    if let Err(e) = Url::parse(&req.url) {
        log::warn!("Invalid URL format: {} - Error: {}", req.url, e);
        return Ok(HttpResponse::BadRequest().json(CreateUrlResponse {
            success: false,
            message: "Invalid URL format".to_string(),
            data: None,
        }));
    }

    // Mock user ID (in real app, extract from JWT/session)
    let user_id = "user_123".to_string();
    
    // Auto-fetch title if not provided
    let title = match &req.title {
        Some(t) if !t.trim().is_empty() => t.clone(),
        _ => fetch_url_title(&req.url).await.unwrap_or_else(|| {
            // Extract domain as fallback title
            Url::parse(&req.url)
                .map(|u| u.host_str().unwrap_or("Unknown").to_string())
                .unwrap_or("Unknown".to_string())
        })
    };

    let mut storage = URL_STORAGE.lock().unwrap();
    let user_urls = storage.entry(user_id.clone()).or_insert_with(Vec::new);

    // Check for duplicates
    if user_urls.iter().any(|u| u.url == req.url) {
        return Ok(HttpResponse::Conflict().json(CreateUrlResponse {
            success: false,
            message: "URL already exists".to_string(),
            data: None,
        }));
    }

    let url_record = UrlRecord {
        id: format!("url_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        user_id: user_id.clone(),
        url: req.url.clone(),
        title,
        description: req.description.clone(),
        tags: req.tags.clone().unwrap_or_default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        status: "active".to_string(),
    };

    user_urls.push(url_record.clone());
    log::info!("Successfully created URL with ID: {}", url_record.id);

    Ok(HttpResponse::Created().json(CreateUrlResponse {
        success: true,
        message: "URL added successfully".to_string(),
        data: Some(url_record),
    }))
}

pub async fn get_urls(query: web::Query<std::collections::HashMap<String, String>>) -> Result<HttpResponse> {
    let user_id = "user_123".to_string(); // Mock user ID
    let storage = URL_STORAGE.lock().unwrap();
    let mut user_urls = storage.get(&user_id).cloned().unwrap_or_default();

    // Apply search filter if provided
    if let Some(search) = query.get("search") {
        let search_lower = search.to_lowercase();
        user_urls.retain(|url| {
            url.title.to_lowercase().contains(&search_lower) ||
            url.url.to_lowercase().contains(&search_lower) ||
            url.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&search_lower)) ||
            url.tags.iter().any(|t| t.to_lowercase().contains(&search_lower))
        });
    }

    // Apply tag filter if provided
    if let Some(tag) = query.get("tag") {
        user_urls.retain(|url| url.tags.contains(tag));
    }

    // Sort by creation date (newest first)
    user_urls.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": user_urls,
        "total": user_urls.len()
    })))
}

pub async fn delete_url(path: web::Path<String>) -> Result<HttpResponse> {
    let url_id = path.into_inner();
    let user_id = "user_123".to_string(); // Mock user ID
    let mut storage = URL_STORAGE.lock().unwrap();
    
    if let Some(user_urls) = storage.get_mut(&user_id) {
        if let Some(pos) = user_urls.iter().position(|u| u.id == url_id) {
            user_urls.remove(pos);
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "URL deleted successfully"
            })));
        }
    }

    Ok(HttpResponse::NotFound().json(serde_json::json!({
        "success": false,
        "message": "URL not found"
    })))
}

pub async fn get_url_analytics() -> Result<HttpResponse> {
    let user_id = "user_123".to_string(); // Mock user ID
    let storage = URL_STORAGE.lock().unwrap();
    let user_urls = storage.get(&user_id).cloned().unwrap_or_default();

    let total_urls = user_urls.len();
    let active_urls = user_urls.iter().filter(|u| u.status == "active").count();
    
    // Collect all unique tags
    let all_tags: std::collections::HashSet<String> = user_urls
        .iter()
        .flat_map(|u| u.tags.iter())
        .cloned()
        .collect();
    
    // Count URLs by domain
    let mut domain_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for url_record in &user_urls {
        if let Ok(parsed_url) = Url::parse(&url_record.url) {
            if let Some(domain) = parsed_url.host_str() {
                *domain_counts.entry(domain.to_string()).or_insert(0) += 1;
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "total_urls": total_urls,
            "active_urls": active_urls,
            "total_tags": all_tags.len(),
            "unique_domains": domain_counts.len(),
            "top_domains": domain_counts.into_iter()
                .collect::<Vec<_>>()
                .into_iter()
                .take(5)
                .collect::<std::collections::HashMap<_, _>>(),
            "all_tags": all_tags.into_iter().collect::<Vec<_>>()
        }
    })))
}

async fn fetch_url_title(url: &str) -> Option<String> {
    // Enhanced title fetching with timeout and user agent
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("ConHub/1.0")
        .build()
        .ok()?;

    match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(html) = response.text().await {
                    // Try multiple title extraction methods
                    if let Some(title) = extract_title_from_html(&html) {
                        return Some(title);
                    }
                }
            }
        }
        Err(_) => return None,
    }
    None
}

fn extract_title_from_html(html: &str) -> Option<String> {
    // Try <title> tag first
    if let Some(start) = html.find("<title>") {
        if let Some(end) = html[start + 7..].find("</title>") {
            let title = &html[start + 7..start + 7 + end];
            let cleaned = title.trim().replace("\n", " ").replace("\r", "");
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        }
    }
    
    // Try og:title meta tag
    if let Some(start) = html.find("property=\"og:title\"") {
        if let Some(content_start) = html[start..].find("content=\"") {
            let content_pos = start + content_start + 9;
            if let Some(content_end) = html[content_pos..].find("\"") {
                let title = &html[content_pos..content_pos + content_end];
                if !title.trim().is_empty() {
                    return Some(title.trim().to_string());
                }
            }
        }
    }
    
    None
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/urls")
            .route("", web::post().to(create_url))
            .route("", web::get().to(get_urls))
            .route("/{id}", web::delete().to(delete_url))
            .route("/analytics", web::get().to(get_url_analytics))
    );
}