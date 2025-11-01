use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::models::{EmbedRequest, EmbedResponse, ErrorResponse, TextInput};
use crate::services::LlmEmbeddingService;

const MAX_TEXT_LENGTH: usize = 8192;
const MAX_BATCH_SIZE: usize = 32;

pub async fn embed_handler(
    req: web::Json<EmbedRequest>,
    service: web::Data<Arc<LlmEmbeddingService>>,
) -> HttpResponse {
    // Extract texts
    let texts = match &req.text {
        TextInput::Single(text) => vec![text.clone()],
        TextInput::Multiple(texts) => texts.clone(),
    };

    // Validate batch size
    if texts.len() > MAX_BATCH_SIZE {
        return HttpResponse::PayloadTooLarge().json(ErrorResponse {
            error: format!("Batch size exceeds maximum of {}", MAX_BATCH_SIZE),
        });
    }

    // Validate text content
    for text in &texts {
        if text.is_empty() {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Text cannot be empty".to_string(),
            });
        }

        // Rough token estimate (words * 1.3)
        let estimated_tokens = (text.split_whitespace().count() as f32 * 1.3) as usize;
        if estimated_tokens > MAX_TEXT_LENGTH {
            return HttpResponse::PayloadTooLarge().json(ErrorResponse {
                error: format!("Text exceeds maximum token limit of {}", MAX_TEXT_LENGTH),
            });
        }
    }

    // Generate embeddings
    match service.generate_embeddings(&texts).await {
        Ok(embeddings) => {
            let dimension = service.get_dimension().unwrap_or(0) as usize;
            HttpResponse::Ok().json(EmbedResponse {
                embeddings,
                dimension,
                model: "proprietary".to_string(),
                count: texts.len(),
            })
        }
        Err(e) => {
            log::error!("Embedding generation failed: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Embedding generation failed: {}", e),
            })
        }
    }
}
