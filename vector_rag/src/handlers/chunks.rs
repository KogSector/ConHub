use actix_web::{web, HttpResponse};
use std::sync::Arc;
use std::time::Instant;
use std::env;

use conhub_models::chunking::{BatchEmbedChunksRequest, BatchEmbedChunksResponse};
use crate::models::ErrorResponse;
use crate::services::{FusionEmbeddingService, vector_store::VectorStoreService};

const MAX_BATCH_CHUNKS: usize = 256;

/// Handler for embedding chunks from the chunker service (Graph RAG pipeline)
pub async fn batch_embed_chunks_handler(
    req: web::Json<BatchEmbedChunksRequest>,
    service: web::Data<Arc<FusionEmbeddingService>>,
) -> HttpResponse {
    let start_time = Instant::now();
    
    // Validate batch size
    if req.chunks.len() > MAX_BATCH_CHUNKS {
        return HttpResponse::PayloadTooLarge().json(ErrorResponse {
            error: format!("Batch size exceeds maximum of {}", MAX_BATCH_CHUNKS),
        });
    }
    
    if req.chunks.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "No chunks provided".to_string(),
        });
    }
    
    log::info!("📦 [Graph RAG] Embedding batch of {} chunks", req.chunks.len());
    
    let total_chunks = req.chunks.len();
    let mut successful = 0;
    let mut failed = 0;
    
    // Extract just the text content for embedding
    let texts: Vec<String> = req.chunks.iter()
        .map(|c| c.content.clone())
        .collect();
    
    // Generate embeddings
    match service.generate_embeddings(&texts, "graph_chunks").await {
        Ok(embeddings) => {
            successful = embeddings.len();
            
            // Store in vector DB if requested
            if req.store_in_vector_db && !embeddings.is_empty() {
                if let Err(e) = store_chunks_in_vector_db(
                    &req.chunks,
                    &embeddings,
                    &service,
                ).await {
                    log::error!("❌ Failed to store chunks in vector DB: {}", e);
                    // Don't fail the whole request if storage fails
                }
            }
        }
        Err(e) => {
            log::error!("❌ Failed to generate embeddings: {}", e);
            failed = total_chunks;
        }
    }
    
    let duration = start_time.elapsed().as_millis() as u64;
    
    log::info!(
        "✅ Chunk embedding complete: {}/{} successful in {}ms",
        successful,
        total_chunks,
        duration
    );
    
    HttpResponse::Ok().json(BatchEmbedChunksResponse {
        total_chunks,
        successful,
        failed,
        duration_ms: Some(duration),
    })
}

async fn store_chunks_in_vector_db(
    chunks: &[conhub_models::chunking::EmbedChunk],
    embeddings: &[Vec<f32>],
    service: &Arc<FusionEmbeddingService>,
) -> Result<(), Box<dyn std::error::Error>> {
    let zilliz_url = env::var("ZILLIZ_PUBLIC_ENDPOINT")
        .unwrap_or_else(|_| env::var("ZILLIZ_ENDPOINT").unwrap_or_else(|_| "https://localhost:19530".to_string()));
    
    let collection = env::var("ZILLIZ_COLLECTION")
        .unwrap_or_else(|_| "conhub_chunks".to_string());
    
    // Get dimension from fusion config
    let dimension = service.get_config().models.first()
        .map(|m| m.dimension)
        .unwrap_or(1536);
    
    log::info!("💾 Storing {} chunks in Zilliz collection '{}'", chunks.len(), collection);
    
    let store = VectorStoreService::new(&zilliz_url, 30).await?;
    store.ensure_collection(&collection, dimension as usize).await?;
    
    // Prepare points for upsert
    let mut points = Vec::with_capacity(chunks.len());
    
    for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
        // Use chunk_id as the primary key
        let id = chunk.chunk_id.to_string();
        
        // Build payload from metadata
        let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        
        if let Some(obj) = chunk.metadata.as_object() {
            for (k, v) in obj.iter() {
                map.insert(k.clone(), v.clone());
            }
        }
        
        // Add chunk-specific fields
        map.insert("chunk_id".to_string(), serde_json::json!(chunk.chunk_id));
        
        // Keep a searchable content field (truncated if needed)
        let content_preview = if chunk.content.len() > 500 {
            format!("{}...", &chunk.content[..500])
        } else {
            chunk.content.clone()
        };
        map.insert("content_preview".to_string(), serde_json::json!(content_preview));
        
        points.push((id, embedding.clone(), map));
    }
    
    store.upsert(&collection, points).await?;
    
    log::info!("✅ Successfully stored {} chunks in Zilliz Cloud", chunks.len());
    Ok(())
}
