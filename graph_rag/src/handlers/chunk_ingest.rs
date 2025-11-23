use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use tracing::{info, warn, error};

use conhub_models::chunking::{IngestChunksRequest, IngestChunksResponse};

use crate::services::chunk_processor::ChunkProcessor;
use crate::errors::GraphResult;

/// Handler for ingesting chunks from the chunker service (Graph RAG pipeline)
pub async fn ingest_chunks(
    payload: web::Json<IngestChunksRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    info!(
        "🔍 [Graph RAG] Ingesting {} chunks from source {}",
        payload.chunks.len(),
        payload.source_id
    );

    if payload.chunks.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No chunks provided"
        }));
    }

    let processor = ChunkProcessor::new(pool.get_ref().clone());

    match processor.process_chunks(&payload).await {
        Ok(stats) => {
            info!(
                "✅ Chunk ingestion complete: {} entities, {} relationships from {} chunks",
                stats.entities_created,
                stats.relationships_created,
                stats.chunks_processed
            );

            HttpResponse::Ok().json(IngestChunksResponse {
                total_chunks: payload.chunks.len(),
                chunks_processed: stats.chunks_processed,
                entities_created: stats.entities_created,
                relationships_created: stats.relationships_created,
            })
        }
        Err(e) => {
            error!("❌ Failed to ingest chunks: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to ingest chunks: {}", e)
            }))
        }
    }
}
