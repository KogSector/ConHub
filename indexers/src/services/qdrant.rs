use std::collections::HashMap;

use crate::models::{ChunkRecord, MutationSet};
use crate::config::IndexerConfig;

#[derive(Clone)]
pub struct QdrantWriter {
    config: IndexerConfig,
}

impl QdrantWriter {
    pub fn new(config: IndexerConfig) -> Self {
        Self { config }
    }

    pub async fn apply_mutation(&self, mutation: &MutationSet) -> anyhow::Result<()> {
        match mutation {
            MutationSet::Upsert { row_key, chunks } => {
                log::info!("Qdrant upsert: row_key={}, chunks={}", row_key, chunks.len());
                if let Some(qdrant_url) = &self.config.qdrant_url {
                    log::debug!("Qdrant endpoint configured: {}", qdrant_url);
                }
            }
            MutationSet::Delete { row_key } => {
                log::info!("Qdrant delete: row_key={}", row_key);
            }
            MutationSet::Skip { row_key } => {
                log::info!("Qdrant skip: row_key={}", row_key);
            }
        }
        Ok(())
    }
}

pub struct MutationBatch {
    mutations: Vec<MutationSet>,
}

impl MutationBatch {
    pub fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    pub fn push(&mut self, mutation: MutationSet) {
        self.mutations.push(mutation);
    }

    pub async fn flush(&mut self, writer: &QdrantWriter) -> anyhow::Result<()> {
        for mutation in &self.mutations {
            writer.apply_mutation(mutation).await?;
        }
        self.mutations.clear();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct InMemorySearchIndex {
    pub rows: HashMap<String, Vec<ChunkRecord>>,
}

impl InMemorySearchIndex {
    pub fn new() -> Self {
        Self {
            rows: HashMap::new(),
        }
    }

    pub fn upsert(&mut self, row_key: String, chunks: Vec<ChunkRecord>) {
        self.rows.insert(row_key, chunks);
    }

    pub fn remove(&mut self, row_key: &str) {
        self.rows.remove(row_key);
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<ChunkRecord> {
        self.rows
            .values()
            .flat_map(|chunks| chunks.iter())
            .filter(|chunk| chunk.text.contains(query))
            .take(limit)
            .cloned()
            .collect()
    }
}