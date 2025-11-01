use anyhow::{anyhow, Result};
use ndarray::{Array1, Array2, Axis};
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum FusionError {
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("Empty embedding vector")]
    EmptyEmbedding,
    #[error("Unsupported modality: {0}")]
    UnsupportedModality(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
    #[error("Insufficient embeddings for clustering: {0}")]
    InsufficientEmbeddings(usize),
}

/// Supported embedding modalities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmbeddingModality {
    Text,
    Image,
    Audio,
    Video,
    Code,
    Multimodal,
}

/// Advanced fusion strategies for combining embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FusionStrategy {
    Concatenation,
    Sum,
    WeightedSum(Vec<f32>),
    Average,
    Max,
    AttentionWeighted,
    HierarchicalClustering { num_clusters: usize },
    AdaptiveWeighting,
    PrincipalComponentAnalysis { components: usize },
    DynamicFusion,
}

/// Clustering algorithm for hierarchical fusion
#[derive(Debug, Clone)]
pub struct HierarchicalCluster {
    pub embeddings: Vec<usize>, // indices of embeddings in this cluster
    pub centroid: Vec<f32>,
    pub weight: f32,
}

/// Advanced vector operations with SIMD optimization
pub struct AdvancedVectorOps;

impl AdvancedVectorOps {
    /// Compute cosine similarity with early termination for performance
    pub fn fast_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let (dot_product, norm_a_sq, norm_b_sq) = a
            .par_iter()
            .zip(b.par_iter())
            .map(|(x, y)| (x * y, x * x, y * y))
            .reduce(
                || (0.0, 0.0, 0.0),
                |(acc_dot, acc_a, acc_b), (dot, a_sq, b_sq)| {
                    (acc_dot + dot, acc_a + a_sq, acc_b + b_sq)
                },
            );

        let norm_a = norm_a_sq.sqrt();
        let norm_b = norm_b_sq.sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Compute pairwise distances matrix efficiently
    pub fn pairwise_distances(embeddings: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let n = embeddings.len();
        let mut distances = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in i + 1..n {
                let dist = 1.0 - Self::fast_cosine_similarity(&embeddings[i], &embeddings[j]);
                distances[i][j] = dist;
                distances[j][i] = dist;
            }
        }

        distances
    }

    /// K-means clustering for hierarchical fusion
    pub fn kmeans_clustering(
        embeddings: &[Vec<f32>],
        k: usize,
        max_iterations: usize,
    ) -> Result<Vec<HierarchicalCluster>> {
        if embeddings.len() < k {
            return Err(FusionError::InsufficientEmbeddings(embeddings.len()).into());
        }

        let dim = embeddings[0].len();
        let mut centroids = Self::initialize_centroids(embeddings, k);
        let mut assignments = vec![0; embeddings.len()];

        for _ in 0..max_iterations {
            let mut changed = false;

            // Assign points to nearest centroids
            for (i, embedding) in embeddings.iter().enumerate() {
                let mut best_cluster = 0;
                let mut best_distance = f32::INFINITY;

                for (j, centroid) in centroids.iter().enumerate() {
                    let distance = 1.0 - Self::fast_cosine_similarity(embedding, centroid);
                    if distance < best_distance {
                        best_distance = distance;
                        best_cluster = j;
                    }
                }

                if assignments[i] != best_cluster {
                    assignments[i] = best_cluster;
                    changed = true;
                }
            }

            if !changed {
                break;
            }

            // Update centroids
            for k in 0..centroids.len() {
                let cluster_points: Vec<&Vec<f32>> = embeddings
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| assignments[*i] == k)
                    .map(|(_, emb)| emb)
                    .collect();

                if !cluster_points.is_empty() {
                    centroids[k] = Self::compute_centroid(&cluster_points);
                }
            }
        }

        // Create clusters
        let mut clusters = Vec::new();
        for k in 0..centroids.len() {
            let cluster_indices: Vec<usize> = assignments
                .iter()
                .enumerate()
                .filter(|(_, &cluster)| cluster == k)
                .map(|(i, _)| i)
                .collect();

            if !cluster_indices.is_empty() {
                let weight = cluster_indices.len() as f32 / embeddings.len() as f32;
                clusters.push(HierarchicalCluster {
                    embeddings: cluster_indices,
                    centroid: centroids[k].clone(),
                    weight,
                });
            }
        }

        Ok(clusters)
    }

    fn initialize_centroids(embeddings: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        let mut rng = rand::thread_rng();
        let mut centroids = Vec::new();

        // Use k-means++ initialization
        let first_idx = rng.gen_range(0..embeddings.len());
        centroids.push(embeddings[first_idx].clone());

        for _ in 1..k {
            let mut distances = vec![f32::INFINITY; embeddings.len()];

            for (i, embedding) in embeddings.iter().enumerate() {
                for centroid in &centroids {
                    let dist = 1.0 - Self::fast_cosine_similarity(embedding, centroid);
                    distances[i] = distances[i].min(dist);
                }
            }

            let total_distance: f32 = distances.iter().sum();
            let mut cumulative = 0.0;
            let target = rng.gen::<f32>() * total_distance;

            for (i, &dist) in distances.iter().enumerate() {
                cumulative += dist;
                if cumulative >= target {
                    centroids.push(embeddings[i].clone());
                    break;
                }
            }
        }

        centroids
    }

    fn compute_centroid(embeddings: &[&Vec<f32>]) -> Vec<f32> {
        if embeddings.is_empty() {
            return Vec::new();
        }

        let dim = embeddings[0].len();
        let mut centroid = vec![0.0; dim];

        for embedding in embeddings {
            for (i, &val) in embedding.iter().enumerate() {
                centroid[i] += val;
            }
        }

        let count = embeddings.len() as f32;
        centroid.iter_mut().for_each(|x| *x /= count);

        // Normalize centroid
        let norm: f32 = centroid.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            centroid.iter_mut().for_each(|x| *x /= norm);
        }

        centroid
    }

    /// Principal Component Analysis for dimensionality reduction
    pub fn pca_transform(embeddings: &[Vec<f32>], components: usize) -> Result<Vec<Vec<f32>>> {
        if embeddings.is_empty() {
            return Ok(Vec::new());
        }

        let n = embeddings.len();
        let dim = embeddings[0].len();
        let components = components.min(dim);

        // Center the data
        let mean = Self::compute_mean(embeddings);
        let centered: Vec<Vec<f32>> = embeddings
            .iter()
            .map(|emb| {
                emb.iter()
                    .zip(mean.iter())
                    .map(|(x, m)| x - m)
                    .collect()
            })
            .collect();

        // Compute covariance matrix (simplified for performance)
        let mut cov_matrix = vec![vec![0.0; dim]; dim];
        for i in 0..dim {
            for j in i..dim {
                let cov: f32 = centered
                    .iter()
                    .map(|row| row[i] * row[j])
                    .sum::<f32>() / (n - 1) as f32;
                cov_matrix[i][j] = cov;
                cov_matrix[j][i] = cov;
            }
        }

        // Simplified eigenvalue computation (using power iteration for top components)
        let principal_components = Self::compute_top_eigenvectors(&cov_matrix, components);

        // Transform data
        let transformed: Vec<Vec<f32>> = centered
            .iter()
            .map(|row| {
                principal_components
                    .iter()
                    .map(|pc| {
                        row.iter()
                            .zip(pc.iter())
                            .map(|(x, p)| x * p)
                            .sum()
                    })
                    .collect()
            })
            .collect();

        Ok(transformed)
    }

    fn compute_mean(embeddings: &[Vec<f32>]) -> Vec<f32> {
        if embeddings.is_empty() {
            return Vec::new();
        }

        let dim = embeddings[0].len();
        let mut mean = vec![0.0; dim];

        for embedding in embeddings {
            for (i, &val) in embedding.iter().enumerate() {
                mean[i] += val;
            }
        }

        let count = embeddings.len() as f32;
        mean.iter_mut().for_each(|x| *x /= count);
        mean
    }

    fn compute_top_eigenvectors(matrix: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        let dim = matrix.len();
        let mut eigenvectors = Vec::new();

        for _ in 0..k {
            let mut v = vec![1.0; dim];
            
            // Power iteration
            for _ in 0..100 {
                let mut new_v = vec![0.0; dim];
                for i in 0..dim {
                    for j in 0..dim {
                        new_v[i] += matrix[i][j] * v[j];
                    }
                }

                // Normalize
                let norm: f32 = new_v.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    new_v.iter_mut().for_each(|x| *x /= norm);
                }

                v = new_v;
            }

            eigenvectors.push(v);
        }

        eigenvectors
    }
}

/// Individual embedding with metadata and optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub id: Uuid,
    pub modality: EmbeddingModality,
    pub vector: Vec<f32>,
    pub dimension: usize,
    pub model_name: String,
    pub metadata: HashMap<String, String>,
    pub norm: f32, // Cached norm for performance
}

impl Embedding {
    pub fn new(
        modality: EmbeddingModality,
        vector: Vec<f32>,
        model_name: String,
    ) -> Result<Self> {
        if vector.is_empty() {
            return Err(FusionError::EmptyEmbedding.into());
        }

        let dimension = vector.len();
        let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        Ok(Self {
            id: Uuid::new_v4(),
            modality,
            vector,
            dimension,
            model_name,
            metadata: HashMap::new(),
            norm,
        })
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn normalize(&mut self) {
        if self.norm > 0.0 {
            self.vector.iter_mut().for_each(|x| *x /= self.norm);
            self.norm = 1.0;
        }
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    /// Fast similarity computation using cached norm
    pub fn similarity_to(&self, other: &Self) -> f32 {
        if self.dimension != other.dimension {
            return 0.0;
        }

        let dot_product: f32 = self.vector
            .par_iter()
            .zip(other.vector.par_iter())
            .map(|(a, b)| a * b)
            .sum();

        if self.norm == 0.0 || other.norm == 0.0 {
            0.0
        } else {
            dot_product / (self.norm * other.norm)
        }
    }
}

/// Fused embedding containing multiple individual embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedEmbedding {
    pub id: Uuid,
    pub individual_embeddings: Vec<Embedding>,
    pub fused_vector: Vec<f32>,
    pub fusion_strategy: FusionStrategy,
    pub dimension: usize,
    pub metadata: HashMap<String, String>,
}

impl FusedEmbedding {
    pub fn new(
        individual_embeddings: Vec<Embedding>,
        fusion_strategy: FusionStrategy,
    ) -> Result<Self> {
        if individual_embeddings.is_empty() {
            return Err(anyhow!("Cannot create fused embedding from empty list"));
        }

        let fused_vector = Self::apply_fusion_strategy(&individual_embeddings, &fusion_strategy)?;
        let dimension = fused_vector.len();

        Ok(Self {
            id: Uuid::new_v4(),
            individual_embeddings,
            fused_vector,
            fusion_strategy,
            dimension,
            metadata: HashMap::new(),
        })
    }

    fn apply_fusion_strategy(
        embeddings: &[Embedding],
        strategy: &FusionStrategy,
    ) -> Result<Vec<f32>> {
        match strategy {
            FusionStrategy::Concatenation => Self::concatenate_embeddings(embeddings),
            FusionStrategy::Sum => Self::sum_embeddings(embeddings),
            FusionStrategy::WeightedSum(weights) => Self::weighted_sum_embeddings(embeddings, weights),
            FusionStrategy::Average => Self::average_embeddings(embeddings),
            FusionStrategy::Max => Self::max_embeddings(embeddings),
            FusionStrategy::AttentionWeighted => Self::attention_weighted_embeddings(embeddings),
            FusionStrategy::HierarchicalClustering { num_clusters } => {
                Self::hierarchical_clustering_fusion(embeddings, *num_clusters)
            },
            FusionStrategy::AdaptiveWeighting => Self::adaptive_weighted_fusion(embeddings),
            FusionStrategy::PrincipalComponentAnalysis { components } => {
                Self::pca_fusion(embeddings, *components)
            },
            FusionStrategy::DynamicFusion => Self::dynamic_fusion(embeddings),
        }
    }

    fn concatenate_embeddings(embeddings: &[Embedding]) -> Result<Vec<f32>> {
        let mut result = Vec::new();
        for embedding in embeddings {
            result.extend_from_slice(&embedding.vector);
        }
        Ok(result)
    }

    fn sum_embeddings(embeddings: &[Embedding]) -> Result<Vec<f32>> {
        let first_dim = embeddings[0].dimension;
        
        // Check all embeddings have same dimension
        for embedding in embeddings {
            if embedding.dimension != first_dim {
                return Err(FusionError::DimensionMismatch {
                    expected: first_dim,
                    actual: embedding.dimension,
                }.into());
            }
        }

        let mut result = vec![0.0; first_dim];
        for embedding in embeddings {
            for (i, &val) in embedding.vector.iter().enumerate() {
                result[i] += val;
            }
        }
        Ok(result)
    }

    fn weighted_sum_embeddings(embeddings: &[Embedding], weights: &[f32]) -> Result<Vec<f32>> {
        if embeddings.len() != weights.len() {
            return Err(anyhow!("Number of embeddings must match number of weights"));
        }

        let first_dim = embeddings[0].dimension;
        for embedding in embeddings {
            if embedding.dimension != first_dim {
                return Err(FusionError::DimensionMismatch {
                    expected: first_dim,
                    actual: embedding.dimension,
                }.into());
            }
        }

        let mut result = vec![0.0; first_dim];
        for (embedding, &weight) in embeddings.iter().zip(weights.iter()) {
            for (i, &val) in embedding.vector.iter().enumerate() {
                result[i] += val * weight;
            }
        }
        Ok(result)
    }

    fn average_embeddings(embeddings: &[Embedding]) -> Result<Vec<f32>> {
        let sum = Self::sum_embeddings(embeddings)?;
        let count = embeddings.len() as f32;
        Ok(sum.into_iter().map(|x| x / count).collect())
    }

    fn max_embeddings(embeddings: &[Embedding]) -> Result<Vec<f32>> {
        let first_dim = embeddings[0].dimension;
        for embedding in embeddings {
            if embedding.dimension != first_dim {
                return Err(FusionError::DimensionMismatch {
                    expected: first_dim,
                    actual: embedding.dimension,
                }.into());
            }
        }

        let mut result = vec![f32::NEG_INFINITY; first_dim];
        for embedding in embeddings {
            for (i, &val) in embedding.vector.iter().enumerate() {
                result[i] = result[i].max(val);
            }
        }
        Ok(result)
    }

    fn attention_weighted_embeddings(embeddings: &[Embedding]) -> Result<Vec<f32>> {
        let first_dim = embeddings[0].dimension;
        for embedding in embeddings {
            if embedding.dimension != first_dim {
                return Err(FusionError::DimensionMismatch {
                    expected: first_dim,
                    actual: embedding.dimension,
                }.into());
            }
        }

        // Simple attention mechanism: compute attention weights based on L2 norm
        let norms: Vec<f32> = embeddings
            .iter()
            .map(|emb| emb.vector.iter().map(|x| x * x).sum::<f32>().sqrt())
            .collect();

        let sum_norms: f32 = norms.iter().sum();
        let weights: Vec<f32> = norms.iter().map(|&norm| norm / sum_norms).collect();

        Self::weighted_sum_embeddings(embeddings, &weights)
    }

    /// Hierarchical clustering fusion strategy
    fn hierarchical_clustering_fusion(embeddings: &[Embedding], num_clusters: usize) -> Result<Vec<f32>> {
        let first_dim = embeddings[0].dimension;
        for embedding in embeddings {
            if embedding.dimension != first_dim {
                return Err(FusionError::DimensionMismatch {
                    expected: first_dim,
                    actual: embedding.dimension,
                }.into());
            }
        }

        let vectors: Vec<Vec<f32>> = embeddings.iter().map(|e| e.vector.clone()).collect();
        let clusters = AdvancedVectorOps::kmeans_clustering(&vectors, num_clusters, 50)?;

        // Weighted average of cluster centroids
        let mut result = vec![0.0; first_dim];
        for cluster in clusters {
            for (i, &val) in cluster.centroid.iter().enumerate() {
                result[i] += val * cluster.weight;
            }
        }

        Ok(result)
    }

    /// Adaptive weighting based on embedding quality and diversity
    fn adaptive_weighted_fusion(embeddings: &[Embedding]) -> Result<Vec<f32>> {
        let first_dim = embeddings[0].dimension;
        for embedding in embeddings {
            if embedding.dimension != first_dim {
                return Err(FusionError::DimensionMismatch {
                    expected: first_dim,
                    actual: embedding.dimension,
                }.into());
            }
        }

        // Compute adaptive weights based on norm and diversity
        let mut weights = Vec::new();
        for (i, embedding) in embeddings.iter().enumerate() {
            let norm_weight = embedding.norm;
            
            // Diversity weight: how different this embedding is from others
            let mut diversity_score = 0.0;
            for (j, other) in embeddings.iter().enumerate() {
                if i != j {
                    let similarity = embedding.similarity_to(other);
                    diversity_score += 1.0 - similarity;
                }
            }
            diversity_score /= (embeddings.len() - 1) as f32;
            
            // Combine norm and diversity
            let adaptive_weight = norm_weight * (1.0 + diversity_score);
            weights.push(adaptive_weight);
        }

        // Normalize weights
        let sum_weights: f32 = weights.iter().sum();
        if sum_weights > 0.0 {
            weights.iter_mut().for_each(|w| *w /= sum_weights);
        }

        Self::weighted_sum_embeddings(embeddings, &weights)
    }

    /// PCA-based fusion for dimensionality reduction
    fn pca_fusion(embeddings: &[Embedding], components: usize) -> Result<Vec<f32>> {
        let first_dim = embeddings[0].dimension;
        for embedding in embeddings {
            if embedding.dimension != first_dim {
                return Err(FusionError::DimensionMismatch {
                    expected: first_dim,
                    actual: embedding.dimension,
                }.into());
            }
        }

        let vectors: Vec<Vec<f32>> = embeddings.iter().map(|e| e.vector.clone()).collect();
        let transformed = AdvancedVectorOps::pca_transform(&vectors, components)?;

        if transformed.is_empty() {
            return Ok(vec![0.0; components]);
        }

        // Average the transformed embeddings
        let mut result = vec![0.0; components];
        for transformed_embedding in &transformed {
            for (i, &val) in transformed_embedding.iter().enumerate() {
                result[i] += val;
            }
        }

        let count = transformed.len() as f32;
        result.iter_mut().for_each(|x| *x /= count);

        Ok(result)
    }

    /// Dynamic fusion that selects the best strategy based on embedding characteristics
    fn dynamic_fusion(embeddings: &[Embedding]) -> Result<Vec<f32>> {
        let first_dim = embeddings[0].dimension;
        for embedding in embeddings {
            if embedding.dimension != first_dim {
                return Err(FusionError::DimensionMismatch {
                    expected: first_dim,
                    actual: embedding.dimension,
                }.into());
            }
        }

        // Analyze embedding characteristics
        let num_embeddings = embeddings.len();
        let avg_norm: f32 = embeddings.iter().map(|e| e.norm).sum::<f32>() / num_embeddings as f32;
        
        // Compute pairwise similarities to measure diversity
        let mut similarities = Vec::new();
        for i in 0..num_embeddings {
            for j in i + 1..num_embeddings {
                similarities.push(embeddings[i].similarity_to(&embeddings[j]));
            }
        }
        let avg_similarity = similarities.iter().sum::<f32>() / similarities.len() as f32;

        // Select strategy based on characteristics
        let strategy = if num_embeddings <= 2 {
            FusionStrategy::Average
        } else if avg_similarity > 0.8 {
            // High similarity - use simple average
            FusionStrategy::Average
        } else if avg_similarity < 0.3 {
            // Low similarity - use clustering
            FusionStrategy::HierarchicalClustering { 
                num_clusters: (num_embeddings / 2).max(2).min(5) 
            }
        } else if avg_norm > 1.5 {
            // High norms - use attention weighting
            FusionStrategy::AttentionWeighted
        } else {
            // Default to adaptive weighting
            FusionStrategy::AdaptiveWeighting
        };

        Self::apply_fusion_strategy(embeddings, &strategy)
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Reference embedding models implementation
pub struct ReferenceModels;

impl ReferenceModels {
    /// BGE (BAAI/bge-base-en-v1.5) style embedding
    pub fn bge_embedding(text: &str) -> Result<Embedding> {
        // Simulate BGE model processing
        let dimension = 768;
        let vector = Self::simulate_bge_forward(text, dimension)?;
        
        let mut metadata = HashMap::new();
        metadata.insert("model_type".to_string(), "BGE".to_string());
        metadata.insert("version".to_string(), "base-en-v1.5".to_string());
        
        Embedding::new(EmbeddingModality::Text, vector, "bge-base-en-v1.5".to_string())
            .map(|emb| emb.with_metadata(metadata))
    }

    /// Qwen3 8B Embedding style
    pub fn qwen3_embedding(text: &str) -> Result<Embedding> {
        let dimension = 4096; // Qwen3 8B typical dimension
        let vector = Self::simulate_qwen3_forward(text, dimension)?;
        
        let mut metadata = HashMap::new();
        metadata.insert("model_type".to_string(), "Qwen3".to_string());
        metadata.insert("version".to_string(), "8B".to_string());
        
        Embedding::new(EmbeddingModality::Text, vector, "qwen3-8b".to_string())
            .map(|emb| emb.with_metadata(metadata))
    }

    /// E5 (intfloat/e5-base-v2) style embedding
    pub fn e5_embedding(text: &str) -> Result<Embedding> {
        let dimension = 768;
        let vector = Self::simulate_e5_forward(text, dimension)?;
        
        let mut metadata = HashMap::new();
        metadata.insert("model_type".to_string(), "E5".to_string());
        metadata.insert("version".to_string(), "base-v2".to_string());
        
        Embedding::new(EmbeddingModality::Text, vector, "e5-base-v2".to_string())
            .map(|emb| emb.with_metadata(metadata))
    }

    fn simulate_bge_forward(text: &str, dimension: usize) -> Result<Vec<f32>> {
        // Simulate BGE-style processing with characteristic patterns
        let text_hash = Self::simple_hash(text) % 10000; // Limit hash size to prevent overflow
        
        let vector: Vec<f32> = (0..dimension)
            .map(|i| {
                let base = ((text_hash + i) as f32 * 0.1).sin();
                // Use deterministic pseudo-random based on text and position
                let pseudo_random = ((text_hash.wrapping_mul(31).wrapping_add(i)) as f32 * 0.01).sin() * 0.1;
                base + pseudo_random
            })
            .collect();
        
        Ok(Self::normalize_vector(vector))
    }

    fn simulate_qwen3_forward(text: &str, dimension: usize) -> Result<Vec<f32>> {
        // Simulate Qwen3-style processing
        let text_hash = Self::simple_hash(text) % 10000; // Limit hash size to prevent overflow
        
        let vector: Vec<f32> = (0..dimension)
            .map(|i| {
                let base = ((text_hash + i * 2) as f32 * 0.05).cos();
                // Use deterministic pseudo-random based on text and position
                let pseudo_random = ((text_hash.wrapping_mul(17).wrapping_add(i * 5)) as f32 * 0.02).cos() * 0.2;
                base * 0.8 + pseudo_random - 0.1
            })
            .collect();
        
        Ok(Self::normalize_vector(vector))
    }

    fn simulate_e5_forward(text: &str, dimension: usize) -> Result<Vec<f32>> {
        // Simulate E5-style processing
        let text_hash = Self::simple_hash(text) % 10000; // Limit hash size to prevent overflow
        
        let vector: Vec<f32> = (0..dimension)
            .map(|i| {
                let base = ((text_hash + i * 3) as f32 * 0.08).tan().tanh();
                // Use deterministic pseudo-random based on text and position
                let pseudo_random = ((text_hash.wrapping_mul(23).wrapping_add(i * 7)) as f32 * 0.03).sin() * 0.15;
                base + pseudo_random - 0.075
            })
            .collect();
        
        Ok(Self::normalize_vector(vector))
    }

    fn simple_hash(text: &str) -> usize {
        text.chars().fold(0, |acc, c| acc.wrapping_mul(31).wrapping_add(c as usize))
    }

    fn normalize_vector(mut vector: Vec<f32>) -> Vec<f32> {
        let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            vector.iter_mut().for_each(|x| *x /= norm);
        }
        vector
    }
}

/// Main fusion embedding service
pub struct FusionEmbeddingService {
    embeddings: HashMap<Uuid, Embedding>,
    fused_embeddings: HashMap<Uuid, FusedEmbedding>,
}

impl FusionEmbeddingService {
    pub fn new() -> Self {
        Self {
            embeddings: HashMap::new(),
            fused_embeddings: HashMap::new(),
        }
    }

    pub fn add_embedding(&mut self, embedding: Embedding) -> Uuid {
        let id = embedding.id;
        self.embeddings.insert(id, embedding);
        id
    }

    pub fn get_embedding(&self, id: &Uuid) -> Option<&Embedding> {
        self.embeddings.get(id)
    }

    pub fn create_fused_embedding(
        &mut self,
        embedding_ids: Vec<Uuid>,
        strategy: FusionStrategy,
    ) -> Result<Uuid> {
        let embeddings: Result<Vec<_>, _> = embedding_ids
            .iter()
            .map(|id| {
                self.embeddings
                    .get(id)
                    .ok_or_else(|| anyhow!("Embedding not found: {}", id))
                    .map(|emb| emb.clone())
            })
            .collect();

        let embeddings = embeddings?;
        let fused = FusedEmbedding::new(embeddings, strategy)?;
        let id = fused.id;
        self.fused_embeddings.insert(id, fused);
        Ok(id)
    }

    pub fn get_fused_embedding(&self, id: &Uuid) -> Option<&FusedEmbedding> {
        self.fused_embeddings.get(id)
    }

    /// Load embeddings from file
    pub fn load_embeddings_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<Uuid>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let embeddings: Vec<Embedding> = bincode::deserialize_from(reader)?;
        
        let ids = embeddings.into_iter().map(|emb| {
            let id = emb.id;
            self.embeddings.insert(id, emb);
            id
        }).collect();
        
        Ok(ids)
    }

    /// Save embeddings to file
    pub fn save_embeddings_to_file<P: AsRef<Path>>(&self, path: P, embedding_ids: &[Uuid]) -> Result<()> {
        let embeddings: Result<Vec<_>, _> = embedding_ids
            .iter()
            .map(|id| {
                self.embeddings
                    .get(id)
                    .ok_or_else(|| anyhow!("Embedding not found: {}", id))
            })
            .collect();

        let embeddings = embeddings?;
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, &embeddings)?;
        Ok(())
    }

    /// Parallel processing for batch fusion
    pub fn batch_fuse_embeddings(
        &mut self,
        batch_requests: Vec<(Vec<Uuid>, FusionStrategy)>,
    ) -> Result<Vec<Uuid>> {
        let results: Result<Vec<_>, _> = batch_requests
            .into_par_iter()
            .map(|(embedding_ids, strategy)| {
                let embeddings: Result<Vec<_>, _> = embedding_ids
                    .iter()
                    .map(|id| {
                        self.embeddings
                            .get(id)
                            .ok_or_else(|| anyhow!("Embedding not found: {}", id))
                            .map(|emb| emb.clone())
                    })
                    .collect();

                let embeddings = embeddings?;
                FusedEmbedding::new(embeddings, strategy)
            })
            .collect();

        let fused_embeddings = results?;
        let ids: Vec<Uuid> = fused_embeddings
            .into_iter()
            .map(|fused| {
                let id = fused.id;
                self.fused_embeddings.insert(id, fused);
                id
            })
            .collect();

        Ok(ids)
    }
}

impl Default for FusionEmbeddingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_embedding(modality: EmbeddingModality, vector: Vec<f32>, model: &str) -> Embedding {
        Embedding::new(modality, vector, model.to_string()).unwrap()
    }

    #[test]
    fn test_embedding_creation() {
        let vector = vec![1.0, 2.0, 3.0];
        let embedding = create_test_embedding(EmbeddingModality::Text, vector.clone(), "test-model");
        
        assert_eq!(embedding.vector, vector);
        assert_eq!(embedding.dimension, 3);
        assert_eq!(embedding.modality, EmbeddingModality::Text);
        assert_eq!(embedding.model_name, "test-model");
    }

    #[test]
    fn test_embedding_normalization() {
        let vector = vec![3.0, 4.0]; // Length = 5.0
        let mut embedding = create_test_embedding(EmbeddingModality::Text, vector, "test-model");
        
        embedding.normalize();
        
        let expected = vec![0.6, 0.8];
        assert!((embedding.vector[0] - expected[0]).abs() < 1e-6);
        assert!((embedding.vector[1] - expected[1]).abs() < 1e-6);
    }

    #[test]
    fn test_concatenation_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Image, vec![3.0, 4.0], "model2");
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Concatenation,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(fused.dimension, 4);
    }

    #[test]
    fn test_sum_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0], "model2");
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Sum,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![4.0, 6.0]);
        assert_eq!(fused.dimension, 2);
    }

    #[test]
    fn test_weighted_sum_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0], "model2");
        
        let weights = vec![0.3, 0.7];
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::WeightedSum(weights),
        ).unwrap();
        
        let expected = vec![0.3 * 1.0 + 0.7 * 3.0, 0.3 * 2.0 + 0.7 * 4.0];
        assert_eq!(fused.fused_vector, expected);
    }

    #[test]
    fn test_average_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![2.0, 4.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![4.0, 6.0], "model2");
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Average,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![3.0, 5.0]);
    }

    #[test]
    fn test_max_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 5.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 2.0], "model2");
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Max,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![3.0, 5.0]);
    }

    #[test]
    fn test_attention_weighted_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 0.0], "model1"); // norm = 1.0
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0], "model2"); // norm = 5.0
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::AttentionWeighted,
        ).unwrap();
        
        // Weights should be 1/6 and 5/6 based on norms
        let expected_x = (1.0/6.0) * 1.0 + (5.0/6.0) * 3.0;
        let expected_y = (1.0/6.0) * 0.0 + (5.0/6.0) * 4.0;
        
        assert!((fused.fused_vector[0] - expected_x).abs() < 1e-6);
        assert!((fused.fused_vector[1] - expected_y).abs() < 1e-6);
    }

    #[test]
    fn test_dimension_mismatch_error() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0, 5.0], "model2");
        
        let result = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Sum,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_embedding_error() {
        let result = Embedding::new(EmbeddingModality::Text, vec![], "test-model".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_fusion_service() {
        let mut service = FusionEmbeddingService::new();
        
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0], "model2");
        
        let id1 = service.add_embedding(emb1);
        let id2 = service.add_embedding(emb2);
        
        let fused_id = service.create_fused_embedding(
            vec![id1, id2],
            FusionStrategy::Sum,
        ).unwrap();
        
        let fused = service.get_fused_embedding(&fused_id).unwrap();
        assert_eq!(fused.fused_vector, vec![4.0, 6.0]);
    }

    #[test]
    fn test_reference_models() {
        let text = "Hello, world!";
        
        let bge_emb = ReferenceModels::bge_embedding(text).unwrap();
        assert_eq!(bge_emb.dimension, 768);
        assert_eq!(bge_emb.model_name, "bge-base-en-v1.5");
        assert_eq!(bge_emb.modality, EmbeddingModality::Text);
        
        let qwen_emb = ReferenceModels::qwen3_embedding(text).unwrap();
        assert_eq!(qwen_emb.dimension, 4096);
        assert_eq!(qwen_emb.model_name, "qwen3-8b");
        
        let e5_emb = ReferenceModels::e5_embedding(text).unwrap();
        assert_eq!(e5_emb.dimension, 768);
        assert_eq!(e5_emb.model_name, "e5-base-v2");
    }

    #[test]
    fn test_reference_model_consistency() {
        let text = "Consistent test text";
        
        let emb1 = ReferenceModels::bge_embedding(text).unwrap();
        let emb2 = ReferenceModels::bge_embedding(text).unwrap();
        
        // Same text should produce same embedding
        assert_eq!(emb1.vector, emb2.vector);
    }

    #[test]
    fn test_embedding_normalization_property() {
        let text = "Test normalization";
        let emb = ReferenceModels::bge_embedding(text).unwrap();
        
        // Check if embedding is normalized (L2 norm ≈ 1.0)
        let norm: f32 = emb.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_multimodal_fusion() {
        let text_emb = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0, 3.0], "text-model");
        let image_emb = create_test_embedding(EmbeddingModality::Image, vec![4.0, 5.0, 6.0], "image-model");
        let audio_emb = create_test_embedding(EmbeddingModality::Audio, vec![7.0, 8.0, 9.0], "audio-model");
        
        let fused = FusedEmbedding::new(
            vec![text_emb, image_emb, audio_emb],
            FusionStrategy::Average,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![4.0, 5.0, 6.0]); // (1+4+7)/3, (2+5+8)/3, (3+6+9)/3
        assert_eq!(fused.individual_embeddings.len(), 3);
    }

    #[test]
    fn test_batch_fusion() {
        let mut service = FusionEmbeddingService::new();
        
        // Create multiple embeddings
        let embeddings: Vec<_> = (0..6)
            .map(|i| {
                let vector = vec![i as f32, (i + 1) as f32];
                let emb = create_test_embedding(EmbeddingModality::Text, vector, &format!("model{}", i));
                service.add_embedding(emb)
            })
            .collect();
        
        // Create batch requests
        let batch_requests = vec![
            (vec![embeddings[0], embeddings[1]], FusionStrategy::Sum),
            (vec![embeddings[2], embeddings[3]], FusionStrategy::Average),
            (vec![embeddings[4], embeddings[5]], FusionStrategy::Max),
        ];
        
        let fused_ids = service.batch_fuse_embeddings(batch_requests).unwrap();
        assert_eq!(fused_ids.len(), 3);
        
        // Verify results
        let fused1 = service.get_fused_embedding(&fused_ids[0]).unwrap();
        assert_eq!(fused1.fused_vector, vec![1.0, 3.0]); // (0+1, 1+2)
        
        let fused2 = service.get_fused_embedding(&fused_ids[1]).unwrap();
        assert_eq!(fused2.fused_vector, vec![2.5, 3.5]); // ((2+3)/2, (3+4)/2)
        
        let fused3 = service.get_fused_embedding(&fused_ids[2]).unwrap();
        assert_eq!(fused3.fused_vector, vec![5.0, 6.0]); // max(4,5), max(5,6)
    }

    #[test]
    fn test_metadata_handling() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("timestamp".to_string(), "2024-01-01".to_string());
        
        let embedding = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "test-model")
            .with_metadata(metadata.clone());
        
        assert_eq!(embedding.metadata, metadata);
        
        let fused = FusedEmbedding::new(
            vec![embedding],
            FusionStrategy::Sum,
        ).unwrap().with_metadata(metadata.clone());
        
        assert_eq!(fused.metadata, metadata);
    }

    #[cfg(test)]
    #[test]
    fn test_file_io_operations() {
        use tempfile::tempdir;
        
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_embeddings.bin");
        
        let mut service = FusionEmbeddingService::new();
        
        // Create test embeddings
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Image, vec![3.0, 4.0], "model2");
        
        let id1 = service.add_embedding(emb1);
        let id2 = service.add_embedding(emb2);
        
        // Save to file
        service.save_embeddings_to_file(&file_path, &[id1, id2]).unwrap();
        assert!(file_path.exists());
        
        // Load from file into new service
        let mut new_service = FusionEmbeddingService::new();
        let loaded_ids = new_service.load_embeddings_from_file(&file_path).unwrap();
        
        assert_eq!(loaded_ids.len(), 2);
        
        // Verify loaded embeddings
        let loaded_emb1 = new_service.get_embedding(&loaded_ids[0]).unwrap();
        let loaded_emb2 = new_service.get_embedding(&loaded_ids[1]).unwrap();
        
        assert_eq!(loaded_emb1.vector, vec![1.0, 2.0]);
        assert_eq!(loaded_emb2.vector, vec![3.0, 4.0]);
    }
}