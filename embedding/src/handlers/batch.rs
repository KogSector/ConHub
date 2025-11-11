use actix_web::{web, HttpResponse};
use std::sync::Arc;
use std::time::Instant;

use crate::models::{
    BatchEmbedRequest, BatchEmbedResponse, DocumentEmbedResult, 
    EmbedStatus, ErrorResponse, EmbeddedChunk
};
use crate::services::LlmEmbeddingService;

const MAX_BATCH_DOCUMENTS: usize = 100;
const MAX_CHUNK_LENGTH: usize = 8192;

/// Handler for batch embedding of documents from connectors
pub async fn batch_embed_handler(
    req: web::Json<BatchEmbedRequest>,
    service: web::Data<Arc<LlmEmbeddingService>>,
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
    let mut successful = 0;
    let mut failed = 0;
    
    // Process each document
    for document in &req.documents {
        match process_document(document, &service, req.normalize).await {
            Ok(result) => {
                if matches!(result.status, EmbedStatus::Success | EmbedStatus::PartialSuccess) {
                    successful += 1;
                } else {
                    failed += 1;
                }
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
    service: &Arc<LlmEmbeddingService>,
    normalize: bool,
) -> Result<DocumentEmbedResult, anyhow::Error> {
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
        return Ok(DocumentEmbedResult {
            id: document.id,
            name: document.name.clone(),
            status: EmbedStatus::Failed,
            chunks_processed: 0,
            error: Some("No chunks to process".to_string()),
        });
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
        
        // Generate embeddings
        match service.generate_embeddings(&texts).await {
            Ok(embeddings) => {
                // Create embedded chunks
                for (idx, chunk) in chunk_batch.iter().enumerate() {
                    if let Some(embedding) = embeddings.get(idx) {
                        let mut metadata = document.metadata.clone();
                        if let Some(ref chunk_meta) = chunk.metadata {
                            metadata["chunk_metadata"] = chunk_meta.clone();
                        }
                        metadata["connector_type"] = serde_json::json!(document.connector_type);
                        metadata["source_id"] = serde_json::json!(document.source_id);
                        metadata["external_id"] = serde_json::json!(document.external_id);
                        metadata["path"] = serde_json::json!(document.path);
                        metadata["chunk_number"] = serde_json::json!(chunk.chunk_number);
                        metadata["start_offset"] = serde_json::json!(chunk.start_offset);
                        metadata["end_offset"] = serde_json::json!(chunk.end_offset);
                        
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
    
    Ok(DocumentEmbedResult {
        id: document.id,
        name: document.name.clone(),
        status,
        chunks_processed: successful_chunks,
        error: if failed_chunks > 0 {
            Some(format!("{} chunks failed to embed", failed_chunks))
        } else {
            None
        },
    })
}
