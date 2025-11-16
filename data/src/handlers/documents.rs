use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use crate::errors::ServiceError;
use uuid::Uuid;
use std::path::Path;
use tokio::fs;
use mime_guess::from_path;

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


lazy_static! {
    static ref DOCUMENT_STORAGE: Mutex<HashMap<String, Vec<DocumentRecord>>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize)]
pub struct ImportDocumentRequest {
    pub provider: String,
    pub file_id: String,
    pub name: String,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
}

pub async fn upload_document(mut payload: Multipart) -> Result<HttpResponse, ServiceError> {
    log::info!("Received file upload request");
    
    let upload_dir = "uploads";
    if !Path::new(upload_dir).exists() {
        fs::create_dir_all(upload_dir).await
            .map_err(|e| ServiceError::InternalError(format!("Failed to create upload directory: {}", e)))?;
    }

    let mut uploaded_files = Vec::new();
    
    while let Some(mut field) = payload.next().await {
        let field = field.map_err(|e| ServiceError::InternalError(format!("Multipart error: {}", e)))?;
        
        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .get_filename()
            .ok_or_else(|| ServiceError::BadRequest("No filename provided".to_string()))?;
        
        let file_id = Uuid::new_v4().to_string();
        let file_extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("txt");
        let stored_filename = format!("{}.{}", file_id, file_extension);
        let filepath = Path::new(upload_dir).join(&stored_filename);
        
        let mut file = fs::File::create(&filepath).await
            .map_err(|e| ServiceError::InternalError(format!("Failed to create file: {}", e)))?;
        
        let mut file_size = 0u64;
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| ServiceError::InternalError(format!("Chunk error: {}", e)))?;
            file_size += data.len() as u64;
            file.write_all(&data).await
                .map_err(|e| ServiceError::InternalError(format!("Failed to write file: {}", e)))?;
        }
        
        let mime_type = from_path(filename).first_or_octet_stream().to_string();
        let user_id = "user_123".to_string(); // TODO: Get from auth context
        
        let document_record = DocumentRecord {
            id: file_id.clone(),
            user_id: user_id.clone(),
            name: filename.to_string(),
            doc_type: mime_type.clone(),
            source: "local_upload".to_string(),
            size: format_file_size(file_size),
            tags: vec!["uploaded".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: "processing".to_string(),
        };
        
        // Store in mock database
        let mut storage = DOCUMENT_STORAGE.lock()
            .map_err(|e| ServiceError::MutexLockError(e.to_string()))?;
        let user_docs = storage.entry(user_id.clone()).or_insert_with(Vec::new);
        user_docs.push(document_record.clone());
        
        // Embedding disabled for now
        // TODO: Re-enable when embedding service is configured
        // if is_heavy_feature_enabled().await {
        //     tokio::spawn(async move {
        //         if let Err(e) = trigger_document_embedding(&file_id, &filepath.to_string_lossy()).await {
        //             log::error!("Failed to trigger embedding for document {}: {}", file_id, e);
        //         }
        //     });
        // }
        
        uploaded_files.push(document_record);
        log::info!("Successfully uploaded file: {} ({})", filename, format_file_size(file_size));
    }
    
    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "message": format!("Successfully uploaded {} file(s)", uploaded_files.len()),
        "data": uploaded_files
    })))
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

async fn is_heavy_feature_enabled() -> bool {
    // Read feature toggles from file
    match tokio::fs::read_to_string("feature-toggles.json").await {
        Ok(content) => {
            if let Ok(toggles) = serde_json::from_str::<serde_json::Value>(&content) {
                toggles.get("Heavy").and_then(|v| v.as_bool()).unwrap_or(false)
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

async fn trigger_document_embedding(file_id: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Triggering embedding for document: {}", file_id);
    
    // Read file content
    let content = tokio::fs::read_to_string(file_path).await?;
    
    // Call embedding service
    let client = reqwest::Client::new();
    let embedding_url = std::env::var("EMBEDDING_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    
    let response = client
        .post(&format!("{}/embed/documents", embedding_url))
        .json(&serde_json::json!({
            "documents": [{
                "id": file_id,
                "content": content,
                "metadata": {
                    "file_path": file_path,
                    "timestamp": Utc::now().to_rfc3339()
                }
            }]
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        log::info!("Successfully triggered embedding for document: {}", file_id);
        
        // Update document status to processed
        update_document_status(file_id, "processed").await;
    } else {
        log::error!("Failed to trigger embedding for document: {} - Status: {}", file_id, response.status());
    }
    
    Ok(())
}

async fn update_document_status(file_id: &str, status: &str) {
    if let Ok(mut storage) = DOCUMENT_STORAGE.lock() {
        for user_docs in storage.values_mut() {
            if let Some(doc) = user_docs.iter_mut().find(|d| d.id == file_id) {
                doc.status = status.to_string();
                doc.updated_at = Utc::now();
                break;
            }
        }
    }
}

pub async fn create_document(req: web::Json<CreateDocumentRequest>) -> Result<HttpResponse, ServiceError> {
    log::info!("Received create document request: {:?}", req.name);
    
    if req.name.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(CreateDocumentResponse {
            success: false,
            message: "Document name is required".to_string(),
            data: None,
        }));
    }

    let user_id = "user_123".to_string();
    let mut storage = DOCUMENT_STORAGE.lock()
        .map_err(|e| ServiceError::MutexLockError(e.to_string()))?;
    let user_docs = storage.entry(user_id.clone()).or_insert_with(Vec::new);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    
    let document_record = DocumentRecord {
        id: format!("doc_{}", timestamp),
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

pub async fn get_documents(query: web::Query<std::collections::HashMap<String, String>>) -> Result<HttpResponse, ServiceError> {
    let user_id = "user_123".to_string();
    let storage = DOCUMENT_STORAGE.lock()
        .map_err(|e| ServiceError::MutexLockError(e.to_string()))?;
    let mut user_docs = storage.get(&user_id).cloned().unwrap_or_default();

    
    if let Some(search) = query.get("search") {
        let search_lower = search.to_lowercase();
        user_docs.retain(|doc| {
            doc.name.to_lowercase().contains(&search_lower) ||
            doc.doc_type.to_lowercase().contains(&search_lower) ||
            doc.source.to_lowercase().contains(&search_lower) ||
            doc.tags.iter().any(|t| t.to_lowercase().contains(&search_lower))
        });
    }

    
    user_docs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": user_docs,
        "total": user_docs.len()
    })))
}

pub async fn delete_document(path: web::Path<String>) -> Result<HttpResponse, ServiceError> {
    let doc_id = path.into_inner();
    let user_id = "user_123".to_string();
    let mut storage = DOCUMENT_STORAGE.lock()
        .map_err(|e| ServiceError::MutexLockError(e.to_string()))?;
    
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

pub async fn get_document_analytics() -> Result<HttpResponse, ServiceError> {
    let user_id = "user_123".to_string();
    let storage = DOCUMENT_STORAGE.lock()
        .map_err(|e| ServiceError::MutexLockError(e.to_string()))?;
    let user_docs = storage.get(&user_id).cloned().unwrap_or_default();

    let total_docs = user_docs.len();
    let processed_docs = user_docs.iter().filter(|d| d.status == "processed").count();
    
    
    let all_tags: std::collections::HashSet<String> = user_docs
        .iter()
        .flat_map(|d| d.tags.iter())
        .cloned()
        .collect();
    
    
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
            .route("/upload", web::post().to(upload_document))
            .route("/import", web::post().to(import_document))
            .route("/cloud/files", web::get().to(list_cloud_files))
    );
}

pub async fn upload_documents(mut payload: Multipart) -> Result<HttpResponse, ServiceError> {
    let user_id = "user_123".to_string();
    let upload_dir = std::path::Path::new("data/uploads");
    if !upload_dir.exists() {
        std::fs::create_dir_all(upload_dir)?;
    }

    let mut created: Vec<DocumentRecord> = Vec::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition().cloned();
        let filename = content_disposition
            .and_then(|cd| cd.get_filename().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("upload_{}", chrono::Utc::now().timestamp_millis()));

        let filepath = upload_dir.join(&filename);
        let mut f = tokio::fs::File::create(&filepath).await?;

        let mut size: u64 = 0;
        while let Some(Ok(chunk)) = field.next().await {
            size += chunk.len() as u64;
            f.write_all(&chunk).await?;
        }

        let mut storage = DOCUMENT_STORAGE
            .lock()
            .map_err(|e| ServiceError::MutexLockError(e.to_string()))?;
        let user_docs = storage.entry(user_id.clone()).or_insert_with(Vec::new);

        let record = DocumentRecord {
            id: format!("doc_{}", chrono::Utc::now().timestamp_millis()),
            user_id: user_id.clone(),
            name: filename.clone(),
            doc_type: mime_guess::from_path(&filepath).first_or_octet_stream().essence_str().to_string(),
            source: "local_files".to_string(),
            size: format!("{} B", size),
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: "processed".to_string(),
        };
        user_docs.push(record.clone());
        created.push(record);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Files uploaded",
        "data": created
    })))
}

pub async fn import_document(req: web::Json<ImportDocumentRequest>) -> Result<HttpResponse, ServiceError> {
    let user_id = "user_123".to_string();
    
    // Import document from cloud provider
    let document_content = match req.provider.as_str() {
        "google_drive" => import_from_google_drive(&req.file_id).await?,
        "dropbox" => import_from_dropbox(&req.file_id).await?,
        _ => return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Unsupported provider"
        })))
    };
    
    let mut storage = DOCUMENT_STORAGE
        .lock()
        .map_err(|e| ServiceError::MutexLockError(e.to_string()))?;
    let user_docs = storage.entry(user_id.clone()).or_insert_with(Vec::new);

    let file_id = Uuid::new_v4().to_string();
    let record = DocumentRecord {
        id: file_id.clone(),
        user_id: user_id.clone(),
        name: req.name.clone(),
        doc_type: req.mime_type.clone().unwrap_or_else(|| "application/octet-stream".into()),
        source: req.provider.clone(),
        size: req.size.map(|s| format_file_size(s)).unwrap_or_else(|| "Unknown".into()),
        tags: vec![req.provider.clone()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        status: "processing".to_string(),
    };
    user_docs.push(record.clone());
    
    // Embedding disabled for now
    // TODO: Re-enable when embedding service is configured
    // if is_heavy_feature_enabled().await {
    //     let content_clone = document_content.clone();
    //     tokio::spawn(async move {
    //         if let Err(e) = trigger_content_embedding(&file_id, &content_clone).await {
    //             log::error!("Failed to trigger embedding for imported document {}: {}", file_id, e);
    //         }
    //     });
    // }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Imported document successfully",
        "data": record
    })))
}

async fn import_from_google_drive(file_id: &str) -> Result<String, ServiceError> {
    // TODO: Implement actual Google Drive API integration
    // For now, return mock content
    log::info!("Importing document from Google Drive: {}", file_id);
    
    // Mock Google Drive API call
    Ok(format!("Mock content from Google Drive file: {}", file_id))
}

async fn import_from_dropbox(file_id: &str) -> Result<String, ServiceError> {
    // TODO: Implement actual Dropbox API integration
    // For now, return mock content
    log::info!("Importing document from Dropbox: {}", file_id);
    
    // Mock Dropbox API call
    Ok(format!("Mock content from Dropbox file: {}", file_id))
}

async fn trigger_content_embedding(file_id: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Triggering embedding for imported document: {}", file_id);
    
    // Call embedding service
    let client = reqwest::Client::new();
    let embedding_url = std::env::var("EMBEDDING_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    
    let response = client
        .post(&format!("{}/embed/documents", embedding_url))
        .json(&serde_json::json!({
            "documents": [{
                "id": file_id,
                "content": content,
                "metadata": {
                    "source": "cloud_import",
                    "timestamp": Utc::now().to_rfc3339()
                }
            }]
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        log::info!("Successfully triggered embedding for imported document: {}", file_id);
        update_document_status(file_id, "processed").await;
    } else {
        log::error!("Failed to trigger embedding for imported document: {} - Status: {}", file_id, response.status());
    }
    
    Ok(())
}

pub async fn list_cloud_files(query: web::Query<HashMap<String, String>>) -> Result<HttpResponse, ServiceError> {
    let provider = query.get("provider").ok_or_else(|| {
        ServiceError::BadRequest("Provider parameter is required".to_string())
    })?;
    
    let files = match provider.as_str() {
        "google_drive" => list_google_drive_files().await?,
        "dropbox" => list_dropbox_files().await?,
        _ => return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Unsupported provider"
        })))
    };
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": files,
        "provider": provider
    })))
}

async fn list_google_drive_files() -> Result<Vec<serde_json::Value>, ServiceError> {
    // TODO: Implement actual Google Drive API integration
    log::info!("Listing Google Drive files");
    
    // Mock Google Drive files
    Ok(vec![
        serde_json::json!({
            "id": "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms",
            "name": "Sample Document.docx",
            "mimeType": "application/vnd.google-apps.document",
            "size": 15420,
            "modifiedTime": "2024-01-15T10:30:00Z"
        }),
        serde_json::json!({
            "id": "1mGVIS7_2Fd9FQWoWzeqQD6rnddlaS_AV1KhtnMBNSRs",
            "name": "Project Presentation.pptx",
            "mimeType": "application/vnd.google-apps.presentation",
            "size": 2048000,
            "modifiedTime": "2024-01-14T14:20:00Z"
        })
    ])
}

async fn list_dropbox_files() -> Result<Vec<serde_json::Value>, ServiceError> {
    // TODO: Implement actual Dropbox API integration
    log::info!("Listing Dropbox files");
    
    // Mock Dropbox files
    Ok(vec![
        serde_json::json!({
            "id": "/Documents/Report.pdf",
            "name": "Report.pdf",
            "mimeType": "application/pdf",
            "size": 524288,
            "server_modified": "2024-01-16T09:15:00Z"
        }),
        serde_json::json!({
            "id": "/Documents/Notes.txt",
            "name": "Notes.txt",
            "mimeType": "text/plain",
            "size": 1024,
            "server_modified": "2024-01-15T16:45:00Z"
        })
    ])
}
