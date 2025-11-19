use actix_web::{web, HttpResponse};
use std::sync::Arc;
use std::time::Instant;

use crate::models::{
    BatchEmbedRequest, BatchEmbedResponse, DocumentEmbedResult, 
    EmbedStatus, ErrorResponse, EmbeddedChunk
};
use crate::services::{LlmEmbeddingService, FusionEmbeddingService, vector_store::VectorStoreService};
use std::env;

const MAX_BATCH_DOCUMENTS: usize = 100;
const MAX_CHUNK_LENGTH: usize = 8192;

/// Handler for batch embedding of documents from connectors
pub async fn batch_embed_handler(
    req: web::Json<BatchEmbedRequest>,
    service: web::Data<Arc<FusionEmbeddingService>>,
) -> HttpResponse {
    let start_time = Instant::now();
    
    // Validate batch size
    if req.documents.len() > MAX_BATCH_DOCUMENTS {
        return HttpResponse::PayloadTooLarge().json(ErrorResponse {
            error: format!("Batch size exceeds maximum of {}", MAX_BATCH_DOCUMENTS),
        });
    }
    
    if req.documents.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "No documents provided".to_string(),
        });
    }
    
    log::info!("📦 Processing batch of {} documents", req.documents.len());
    
    let mut results = Vec::new();
    let mut all_embedded_chunks: Vec<EmbeddedChunk> = Vec::new();
    let mut successful = 0;
    let mut failed = 0;
    
    // Process each document
    for document in &req.documents {
        match process_document(document, &service, req.normalize).await {
            Ok((result, chunks)) => {
                if matches!(result.status, EmbedStatus::Success | EmbedStatus::PartialSuccess) {
                    successful += 1;
                } else {
                    failed += 1;
                }
                all_embedded_chunks.extend(chunks);
                results.push(result);
            }
            Err(e) => {
                log::error!("Failed to process document {}: {}", document.name, e);
                failed += 1;
                results.push(DocumentEmbedResult {
                    id: document.id,
                    name: document.name.clone(),
                    status: EmbedStatus::Failed,
                    chunks_processed: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    
    let duration = start_time.elapsed().as_millis() as u64;
    
    log::info!(
        "✅ Batch processing complete: {} successful, {} failed in {}ms",
        successful,
        failed,
        duration
    );
    
    if req.store_in_vector_db && !all_embedded_chunks.is_empty() {
        let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
        let collection = env::var("QDRANT_COLLECTION").unwrap_or_else(|_| "conhub_embeddings".to_string());
        // Get dimension from fusion config (use first model's dimension as reference)
        let dimension = service.get_config().models.first()
            .map(|m| m.dimension)
            .unwrap_or(1536);
        match VectorStoreService::new(&qdrant_url, 5).await {
            Ok(store) => {
                let _ = store.ensure_collection(&collection, dimension).await;
                let mut points: Vec<(String, Vec<f32>, serde_json::Map<String, serde_json::Value>)> = Vec::with_capacity(all_embedded_chunks.len());
                for ch in all_embedded_chunks.into_iter() {
                    let id = format!("{}-{}", ch.document_id, ch.chunk_number);
                    let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                    if let Some(obj) = ch.metadata.as_object() {
                        for (k, v) in obj.iter() { map.insert(k.clone(), v.clone()); }
                    } else {
                        map.insert("metadata".to_string(), ch.metadata.clone());
                    }
                    map.insert("content".to_string(), serde_json::json!(ch.content));
                    points.push((id, ch.embedding.clone(), map));
                }
                let _ = store.upsert(&collection, points).await;
            }
            Err(e) => {
                log::error!("Vector store initialization failed: {}", e);
            }
        }
    }

    HttpResponse::Ok().json(BatchEmbedResponse {
        total_documents: req.documents.len(),
        successful,
        failed,
        results,
        duration_ms: duration,
    })
}

/// Process a single document and generate embeddings for its chunks
async fn process_document(
    document: &crate::models::DocumentForEmbedding,
    service: &Arc<FusionEmbeddingService>,
    normalize: bool,
) -> Result<(DocumentEmbedResult, Vec<EmbeddedChunk>), anyhow::Error> {
    log::debug!("Processing document: {} ({})", document.name, document.id);
    
    // Get chunks from document or create a single chunk from content
    let chunks = if let Some(ref chunks) = document.chunks {
        chunks.clone()
    } else {
        // Create a single chunk from the entire content
        vec![crate::models::DocumentChunk {
            chunk_number: 0,
            content: document.content.clone(),
            start_offset: 0,
            end_offset: document.content.len(),
            metadata: None,
        }]
    };
    
    if chunks.is_empty() {
        return Ok((DocumentEmbedResult {
            id: document.id,
            name: document.name.clone(),
            status: EmbedStatus::Failed,
            chunks_processed: 0,
            error: Some("No chunks to process".to_string()),
        }, Vec::new()));
    }
    
    let mut embedded_chunks = Vec::new();
    let mut successful_chunks = 0;
    let mut failed_chunks = 0;
    
    // Process chunks in batches
    const CHUNK_BATCH_SIZE: usize = 16;
    
    for chunk_batch in chunks.chunks(CHUNK_BATCH_SIZE) {
        // Extract texts from chunks
        let texts: Vec<String> = chunk_batch
            .iter()
            .map(|c| {
                // Truncate if needed
                if c.content.len() > MAX_CHUNK_LENGTH {
                    c.content[..MAX_CHUNK_LENGTH].to_string()
                } else {
                    c.content.clone()
                }
            })
            .collect();
        
        // Generate embeddings using fusion service with source type routing
        let source_type = document.connector_type.as_str();
        match service.generate_embeddings(&texts, source_type).await {
            Ok(embeddings) => {
                // Create embedded chunks
                for (idx, chunk) in chunk_batch.iter().enumerate() {
                    if let Some(embedding) = embeddings.get(idx) {
                        let mut metadata = document.metadata.clone();
                        if let Some(ref chunk_meta) = chunk.metadata {
                            metadata["chunk_metadata"] = chunk_meta.clone();
                        }
                        // Core relational metadata
                        metadata["connector_type"] = serde_json::json!(document.connector_type);
                        metadata["source_id"] = serde_json::json!(document.source_id);
                        metadata["external_id"] = serde_json::json!(document.external_id);
                        metadata["path"] = serde_json::json!(document.path);
                        metadata["chunk_number"] = serde_json::json!(chunk.chunk_number);
                        metadata["start_offset"] = serde_json::json!(chunk.start_offset);
                        metadata["end_offset"] = serde_json::json!(chunk.end_offset);
                        
                        // Embedding metadata for versioning and tracking
                        metadata["embedding_profile"] = serde_json::json!(source_type);
                        metadata["embedding_strategy"] = serde_json::json!("fusion");
                        metadata["normalize_embeddings"] = serde_json::json!(service.get_config().normalize_embeddings);
                        
                        embedded_chunks.push(EmbeddedChunk {
                            document_id: document.id,
                            chunk_number: chunk.chunk_number,
                            content: chunk.content.clone(),
                            embedding: embedding.clone(),
                            metadata,
                        });
                        
                        successful_chunks += 1;
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to generate embeddings for chunk batch: {}", e);
                failed_chunks += chunk_batch.len();
            }
        }
    }
    
    // TODO: Store embedded chunks in vector database (Qdrant)
    // This will be done via a separate service call
    
    let status = if failed_chunks == 0 {
        EmbedStatus::Success
    } else if successful_chunks > 0 {
        EmbedStatus::PartialSuccess
    } else {
        EmbedStatus::Failed
    };
    
    Ok((DocumentEmbedResult {
        id: document.id,
        name: document.name.clone(),
        status,
        chunks_processed: successful_chunks,
        error: if failed_chunks > 0 {
            Some(format!("{} chunks failed to embed", failed_chunks))
        } else {
            None
        },
    }, embedded_chunks))
}
