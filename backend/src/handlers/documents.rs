use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;

#[derive(Deserialize)]
pub struct CreateDocumentRequest {
    pub name: String,
    pub source: String,
    pub doc_type: String,
    pub size: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Clone)]
pub struct DocumentRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub doc_type: String,
    pub source: String,
    pub size: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Serialize)]
pub struct CreateDocumentResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<DocumentRecord>,
}

// Thread-safe in-memory storage for demo
lazy_static! {
    static ref DOCUMENT_STORAGE: Mutex<HashMap<String, Vec<DocumentRecord>>> = Mutex::new(HashMap::new());
}

pub async fn create_document(req: web::Json<CreateDocumentRequest>) -> Result<HttpResponse> {
    log::info!("Received create document request: {:?}", req.name);
    
    if req.name.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(CreateDocumentResponse {
            success: false,
            message: "Document name is required".to_string(),
            data: None,
        }));
    }

    let user_id = "user_123".to_string();
    let mut storage = DOCUMENT_STORAGE.lock().unwrap();
    let user_docs = storage.entry(user_id.clone()).or_insert_with(Vec::new);

    let document_record = DocumentRecord {
        id: format!("doc_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        user_id: user_id.clone(),
        name: req.name.clone(),
        doc_type: req.doc_type.clone(),
        source: req.source.clone(),
        size: req.size.clone().unwrap_or_else(|| "Unknown".to_string()),
        tags: req.tags.clone().unwrap_or_default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        status: "processed".to_string(),
    };

    user_docs.push(document_record.clone());
    log::info!("Successfully created document with ID: {}", document_record.id);

    Ok(HttpResponse::Created().json(CreateDocumentResponse {
        success: true,
        message: "Document added successfully".to_string(),
        data: Some(document_record),
    }))
}

pub async fn get_documents(query: web::Query<std::collections::HashMap<String, String>>) -> Result<HttpResponse> {
    let user_id = "user_123".to_string();
    let storage = DOCUMENT_STORAGE.lock().unwrap();
    let mut user_docs = storage.get(&user_id).cloned().unwrap_or_default();

    // Apply search filter if provided
    if let Some(search) = query.get("search") {
        let search_lower = search.to_lowercase();
        user_docs.retain(|doc| {
            doc.name.to_lowercase().contains(&search_lower) ||
            doc.doc_type.to_lowercase().contains(&search_lower) ||
            doc.source.to_lowercase().contains(&search_lower) ||
            doc.tags.iter().any(|t| t.to_lowercase().contains(&search_lower))
        });
    }

    // Sort by creation date (newest first)
    user_docs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": user_docs,
        "total": user_docs.len()
    })))
}

pub async fn delete_document(path: web::Path<String>) -> Result<HttpResponse> {
    let doc_id = path.into_inner();
    let user_id = "user_123".to_string();
    let mut storage = DOCUMENT_STORAGE.lock().unwrap();
    
    if let Some(user_docs) = storage.get_mut(&user_id) {
        if let Some(pos) = user_docs.iter().position(|d| d.id == doc_id) {
            user_docs.remove(pos);
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Document deleted successfully"
            })));
        }
    }

    Ok(HttpResponse::NotFound().json(serde_json::json!({
        "success": false,
        "message": "Document not found"
    })))
}

pub async fn get_document_analytics() -> Result<HttpResponse> {
    let user_id = "user_123".to_string();
    let storage = DOCUMENT_STORAGE.lock().unwrap();
    let user_docs = storage.get(&user_id).cloned().unwrap_or_default();

    let total_docs = user_docs.len();
    let processed_docs = user_docs.iter().filter(|d| d.status == "processed").count();
    
    // Collect all unique tags
    let all_tags: std::collections::HashSet<String> = user_docs
        .iter()
        .flat_map(|d| d.tags.iter())
        .cloned()
        .collect();
    
    // Count documents by source
    let mut source_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for doc in &user_docs {
        *source_counts.entry(doc.source.clone()).or_insert(0) += 1;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "total_documents": total_docs,
            "processed_documents": processed_docs,
            "total_tags": all_tags.len(),
            "unique_sources": source_counts.len(),
            "sources": source_counts,
            "all_tags": all_tags.into_iter().collect::<Vec<_>>()
        }
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/documents")
            .route("", web::post().to(create_document))
            .route("", web::get().to(get_documents))
            .route("/{id}", web::delete().to(delete_document))
            .route("/analytics", web::get().to(get_document_analytics))
    );
}