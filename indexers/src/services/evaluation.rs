use std::collections::HashMap;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use tokio::task::JoinSet;

use crate::models::{ChunkRecord, ContentFingerprint, EvaluationOutcome, MutationSet, RowSnapshot, SourceVersionKind};
use crate::services::chunking::ChunkingService;
use crate::services::embedding::EmbeddingService;
use crate::services::state::{IndexStateManager, MemoizationCache};

#[derive(Clone)]
pub struct EvaluationContext {
    pub row_key: String,
    pub metadata: HashMap<String, String>,
    pub content: String,
    pub language: Option<String>,
}

#[derive(Clone)]
pub struct EvaluationService {
    chunker: Arc<ChunkingService>,
    embeddings: Arc<EmbeddingService>,
    state: IndexStateManager,
}

impl EvaluationService {
    pub fn new(
        chunker: Arc<ChunkingService>,
        embeddings: Arc<EmbeddingService>,
        state: IndexStateManager,
    ) -> Self {
        Self {
            chunker,
            embeddings,
            state,
        }
    }

    pub async fn evaluate(&self, ctx: EvaluationContext) -> anyhow::Result<EvaluationOutcome> {
        let fingerprint = Self::fingerprint(&ctx.content);

        if let Some(cached) = self.state.memoization().get(&fingerprint) {
            return Ok(EvaluationOutcome::memoized(ctx.row_key, fingerprint, cached));
        }

        let chunks = self.chunker.chunk_text(&ctx.content)?;
        let mut join_set = JoinSet::new();
        for chunk in &chunks {
            let embeddings = self.embeddings.clone();
            let chunk_text = chunk.clone();
            join_set.spawn(async move {
                embeddings
                    .generate_embedding(&chunk_text)
                    .await
                    .map(|vector| (chunk_text, vector))
            });
        }

        let mut chunk_records = Vec::with_capacity(chunks.len());
        while let Some(result) = join_set.join_next().await {
            let (chunk_text, embedding) = result??;
            chunk_records.push(ChunkRecord::new(chunk_text, embedding, ctx.metadata.clone()));
        }

        let mutation = if chunk_records.is_empty() {
            MutationSet::deletion(ctx.row_key.clone())
        } else {
            MutationSet::upsert(ctx.row_key.clone(), chunk_records.clone())
        };

        let snapshot = RowSnapshot {
            row_id: ctx.row_key.clone(),
            fingerprint: Some(fingerprint.clone()),
            version_kind: SourceVersionKind::Content,
            last_mutation_at: chrono::Utc::now(),
            mutation: Some(mutation.clone()),
        };

        self.state.memoization().put(fingerprint.clone(), chunk_records.clone());
        self.state
            .indexed_store()
            .upsert(snapshot.clone());

        Ok(EvaluationOutcome::new(
            ctx.row_key,
            fingerprint,
            chunk_records,
            snapshot,
        ))
    }

    fn fingerprint(content: &str) -> ContentFingerprint {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash_bytes = hasher.finalize();
        ContentFingerprint {
            value: hex::encode(hash_bytes),
        }
    }
}