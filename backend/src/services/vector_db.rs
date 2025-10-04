use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: VectorDocument,
    pub similarity_score: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VectorDatabase {
    documents: HashMap<String, VectorDocument>,
}

impl VectorDatabase {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Add a document to the vector database
    #[allow(dead_code)]
    pub async fn add_document(
        &mut self,
        content: String,
        metadata: HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::new_v4().to_string();
        
        // Generate embedding (in production, use a real embedding model)
        let embedding = self.generate_embedding(&content).await?;
        
        let document = VectorDocument {
            id: id.clone(),
            content,
            embedding,
            metadata,
            created_at: chrono::Utc::now(),
        };
        
        self.documents.insert(id.clone(), document);
        Ok(id)
    }

    /// Search for similar documents
    #[allow(dead_code)]
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let query_embedding = self.generate_embedding(query).await?;
        
        let mut results: Vec<SearchResult> = self
            .documents
            .values()
            .map(|doc| {
                let similarity = self.cosine_similarity(&query_embedding, &doc.embedding);
                SearchResult {
                    document: doc.clone(),
                    similarity_score: similarity,
                }
            })
            .collect();
        
        // Sort by similarity score (descending)
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        
        // Return top results
        results.truncate(limit);
        Ok(results)
    }

    /// Delete a document
    #[allow(dead_code)]
    pub fn delete_document(&mut self, id: &str) -> bool {
        self.documents.remove(id).is_some()
    }

    /// Get document by ID
    #[allow(dead_code)]
    pub fn get_document(&self, id: &str) -> Option<&VectorDocument> {
        self.documents.get(id)
    }

    /// Get all documents
    #[allow(dead_code)]
    pub fn list_documents(&self) -> Vec<&VectorDocument> {
        self.documents.values().collect()
    }

    /// Generate embedding for text (simplified implementation)
    #[allow(dead_code)]
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        // In production, you would use a real embedding model like:
        // - OpenAI's text-embedding-ada-002
        // - Sentence Transformers
        // - Cohere embeddings
        // - Local models like all-MiniLM-L6-v2
        
        // For now, create a simple hash-based embedding
        let mut embedding = vec![0.0; 384]; // Common embedding dimension
        
        for (i, byte) in text.bytes().enumerate() {
            if i >= embedding.len() { break; }
            embedding[i] = (byte as f32) / 255.0;
        }
        
        // Normalize the embedding
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }
        
        Ok(embedding)
    }

    /// Calculate cosine similarity between two vectors
    #[allow(dead_code)]
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            0.0
        } else {
            dot_product / (magnitude_a * magnitude_b)
        }
    }
}

impl Default for VectorDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Vector database service for managing embeddings and semantic search
#[allow(dead_code)]
pub struct VectorDbService {
    db: VectorDatabase,
}

impl VectorDbService {
    pub fn new() -> Self {
        Self {
            db: VectorDatabase::new(),
        }
    }

    /// Index repository content for semantic search
    #[allow(dead_code)]
    pub async fn index_repository_content(
        &mut self,
        repo_id: &str,
        repo_name: &str,
        content: &str,
        file_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut metadata = HashMap::new();
        metadata.insert("source_type".to_string(), "repository".to_string());
        metadata.insert("repository_id".to_string(), repo_id.to_string());
        metadata.insert("repository_name".to_string(), repo_name.to_string());
        metadata.insert("file_path".to_string(), file_path.to_string());
        
        self.db.add_document(content.to_string(), metadata).await
    }

    /// Index document content for semantic search
    #[allow(dead_code)]
    pub async fn index_document_content(
        &mut self,
        doc_id: &str,
        title: &str,
        content: &str,
        doc_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut metadata = HashMap::new();
        metadata.insert("source_type".to_string(), "document".to_string());
        metadata.insert("document_id".to_string(), doc_id.to_string());
        metadata.insert("title".to_string(), title.to_string());
        metadata.insert("document_type".to_string(), doc_type.to_string());
        
        self.db.add_document(content.to_string(), metadata).await
    }

    /// Index URL content for semantic search
    #[allow(dead_code)]
    pub async fn index_url_content(
        &mut self,
        url: &str,
        title: &str,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut metadata = HashMap::new();
        metadata.insert("source_type".to_string(), "url".to_string());
        metadata.insert("url".to_string(), url.to_string());
        metadata.insert("title".to_string(), title.to_string());
        
        self.db.add_document(content.to_string(), metadata).await
    }

    /// Search across all indexed content
    #[allow(dead_code)]
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        source_type_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = self.db.search(query, limit * 2).await?;
        
        // Apply source type filter if specified
        if let Some(filter) = source_type_filter {
            results.retain(|result| {
                result.document.metadata.get("source_type")
                    .map(|s| s == filter)
                    .unwrap_or(false)
            });
        }
        
        results.truncate(limit);
        Ok(results)
    }

    /// Get statistics about indexed content
    #[allow(dead_code)]
    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        let documents = self.db.list_documents();
        
        stats.insert("total_documents".to_string(), documents.len());
        
        let mut by_type = HashMap::new();
        for doc in documents {
            if let Some(source_type) = doc.metadata.get("source_type") {
                *by_type.entry(source_type.clone()).or_insert(0) += 1;
            }
        }
        
        for (key, value) in by_type {
            stats.insert(format!("{}_documents", key), value);
        }
        
        stats
    }
}

impl Default for VectorDbService {
    fn default() -> Self {
        Self::new()
    }
}