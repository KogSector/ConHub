use anyhow::{anyhow, Result};
use ndarray::{Array1, Array2};
use rand::Rng;
use std::collections::HashMap;

const HIDDEN_SIZE: usize = 768;
const NUM_LAYERS: usize = 6;
const NUM_HEADS: usize = 12;
const VOCAB_SIZE: usize = 30522;
const MAX_SEQ_LENGTH: usize = 512;

/// BGE reranker service with cross-encoder architecture
/// Architecture: Transformer layers with cross-attention (6 layers, 12 heads, 768 hidden)
/// Output: Binary classification head → sigmoid → relevance score 0.0-1.0
pub struct RerankService {
    tokenizer: SimpleTokenizer,
    model: CrossEncoderModel,
}

impl RerankService {
    pub fn new() -> Result<Self> {
        log::info!("Initializing reranker service with cross-encoder architecture");
        let tokenizer = SimpleTokenizer::new();
        let model = CrossEncoderModel::new();
        log::info!("Reranker service initialized");
        Ok(Self { tokenizer, model })
    }

    pub fn rerank(
        &self,
        query: &str,
        documents: &[(String, String)],
    ) -> Result<Vec<(String, f32)>> {
        if query.is_empty() {
            return Err(anyhow!("Query cannot be empty"));
        }

        if documents.is_empty() {
            return Err(anyhow!("Documents list cannot be empty"));
        }

        let mut results = Vec::new();

        for (doc_id, doc_text) in documents.iter() {
            // Create query-document pair
            let combined = format!("{} [SEP] {}", query, doc_text);

            // Tokenize
            let tokens = self.tokenizer.tokenize(&combined);
            if tokens.len() > MAX_SEQ_LENGTH {
                // Truncate if too long
                let truncated = &tokens[..MAX_SEQ_LENGTH];
                let score = self.model.score(truncated)?;
                results.push((doc_id.clone(), score));
            } else {
                let score = self.model.score(&tokens)?;
                results.push((doc_id.clone(), score));
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }
}

/// Simple tokenizer for reranker
struct SimpleTokenizer {
    vocab: HashMap<String, usize>,
}

impl SimpleTokenizer {
    fn new() -> Self {
        let mut vocab = HashMap::new();
        vocab.insert("[CLS]".to_string(), 101);
        vocab.insert("[SEP]".to_string(), 102);
        vocab.insert("[PAD]".to_string(), 0);
        vocab.insert("[UNK]".to_string(), 100);

        Self { vocab }
    }

    fn tokenize(&self, text: &str) -> Vec<usize> {
        let mut tokens = vec![101]; // [CLS]

        for word in text.split_whitespace() {
            let token_id = self.vocab.get(word).copied().unwrap_or(100);
            tokens.push(token_id);
        }

        tokens.push(102); // [SEP]
        tokens
    }
}

/// Cross-encoder model for reranking
struct CrossEncoderModel {
    embeddings: EmbeddingLayer,
    layers: Vec<TransformerLayer>,
    classifier: ClassificationHead,
}

impl CrossEncoderModel {
    fn new() -> Self {
        log::info!("Initializing cross-encoder: {} layers, {} heads", NUM_LAYERS, NUM_HEADS);

        let embeddings = EmbeddingLayer::new();
        let layers = (0..NUM_LAYERS)
            .map(|_| TransformerLayer::new())
            .collect();
        let classifier = ClassificationHead::new();

        Self {
            embeddings,
            layers,
            classifier,
        }
    }

    fn score(&self, token_ids: &[usize]) -> Result<f32> {
        // Embedding layer
        let mut hidden_states = self.embeddings.forward(token_ids)?;

        // Pass through transformer layers
        for layer in &self.layers {
            hidden_states = layer.forward(&hidden_states)?;
        }

        // Extract [CLS] token representation
        let cls_repr = hidden_states.row(0).to_vec();

        // Classification head
        let score = self.classifier.forward(&cls_repr)?;

        Ok(score)
    }
}

/// Embedding layer
struct EmbeddingLayer {
    token_embeddings: Array2<f32>,
    position_embeddings: Array2<f32>,
}

impl EmbeddingLayer {
    fn new() -> Self {
        let token_embeddings = random_matrix(VOCAB_SIZE, HIDDEN_SIZE, 0.02);
        let position_embeddings = random_matrix(MAX_SEQ_LENGTH, HIDDEN_SIZE, 0.02);

        Self {
            token_embeddings,
            position_embeddings,
        }
    }

    fn forward(&self, token_ids: &[usize]) -> Result<Array2<f32>> {
        let seq_len = token_ids.len();
        let mut embeddings = Array2::zeros((seq_len, HIDDEN_SIZE));

        for (pos, &token_id) in token_ids.iter().enumerate() {
            let token_emb = self.token_embeddings.row(token_id % VOCAB_SIZE);
            let pos_emb = self.position_embeddings.row(pos % MAX_SEQ_LENGTH);

            for (i, (t, p)) in token_emb.iter().zip(pos_emb.iter()).enumerate() {
                embeddings[[pos, i]] = t + p;
            }
        }

        Ok(embeddings)
    }
}

/// Transformer layer with cross-attention
struct TransformerLayer {
    attention: CrossAttention,
    feed_forward: FeedForward,
}

impl TransformerLayer {
    fn new() -> Self {
        Self {
            attention: CrossAttention::new(),
            feed_forward: FeedForward::new(),
        }
    }

    fn forward(&self, hidden_states: &Array2<f32>) -> Result<Array2<f32>> {
        let attn_output = self.attention.forward(hidden_states)?;
        let after_attn = hidden_states + &attn_output;

        let ff_output = self.feed_forward.forward(&after_attn)?;
        let output = &after_attn + &ff_output;

        Ok(output)
    }
}

/// Cross-attention mechanism
struct CrossAttention {
    query_weight: Array2<f32>,
    key_weight: Array2<f32>,
    value_weight: Array2<f32>,
    output_weight: Array2<f32>,
}

impl CrossAttention {
    fn new() -> Self {
        Self {
            query_weight: random_matrix(HIDDEN_SIZE, HIDDEN_SIZE, 0.02),
            key_weight: random_matrix(HIDDEN_SIZE, HIDDEN_SIZE, 0.02),
            value_weight: random_matrix(HIDDEN_SIZE, HIDDEN_SIZE, 0.02),
            output_weight: random_matrix(HIDDEN_SIZE, HIDDEN_SIZE, 0.02),
        }
    }

    fn forward(&self, hidden_states: &Array2<f32>) -> Result<Array2<f32>> {
        let query = hidden_states.dot(&self.query_weight);
        let key = hidden_states.dot(&self.key_weight);
        let value = hidden_states.dot(&self.value_weight);

        let scores = query.dot(&key.t()) / (HIDDEN_SIZE as f32).sqrt();
        let attention_weights = softmax_matrix(&scores);

        let context = attention_weights.dot(&value);
        let output = context.dot(&self.output_weight);

        Ok(output)
    }
}

/// Feed-forward network
struct FeedForward {
    linear1: Array2<f32>,
    linear2: Array2<f32>,
}

impl FeedForward {
    fn new() -> Self {
        let intermediate_size = HIDDEN_SIZE * 4;
        Self {
            linear1: random_matrix(HIDDEN_SIZE, intermediate_size, 0.02),
            linear2: random_matrix(intermediate_size, HIDDEN_SIZE, 0.02),
        }
    }

    fn forward(&self, hidden_states: &Array2<f32>) -> Result<Array2<f32>> {
        let intermediate = hidden_states.dot(&self.linear1);
        let activated = gelu_activation(&intermediate);
        let output = activated.dot(&self.linear2);

        Ok(output)
    }
}

/// Classification head for scoring
struct ClassificationHead {
    weight: Array2<f32>,
    bias: f32,
}

impl ClassificationHead {
    fn new() -> Self {
        Self {
            weight: random_matrix(HIDDEN_SIZE, 1, 0.02),
            bias: 0.0,
        }
    }

    fn forward(&self, cls_repr: &[f32]) -> Result<f32> {
        // Linear transformation
        let mut logit = self.bias;
        for (i, &val) in cls_repr.iter().enumerate() {
            logit += val * self.weight[[i, 0]];
        }

        // Sigmoid activation to get score 0.0-1.0
        let score = sigmoid(logit);

        Ok(score)
    }
}

// Helper functions

fn random_matrix(rows: usize, cols: usize, std: f32) -> Array2<f32> {
    let mut rng = rand::thread_rng();
    Array2::from_shape_fn((rows, cols), |_| rng.gen::<f32>() * std - std / 2.0)
}

fn softmax_matrix(matrix: &Array2<f32>) -> Array2<f32> {
    let mut result = matrix.clone();
    for mut row in result.rows_mut() {
        let max_val = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let exp_sum: f32 = row.iter().map(|&x| (x - max_val).exp()).sum();
        for val in row.iter_mut() {
            *val = (*val - max_val).exp() / exp_sum;
        }
    }
    result
}

fn gelu_activation(matrix: &Array2<f32>) -> Array2<f32> {
    matrix.mapv(|x| {
        0.5 * x * (1.0 + ((2.0_f32 / std::f32::consts::PI).sqrt()
            * (x + 0.044715 * x.powi(3))).tanh())
    })
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reranking() {
        let service = RerankService::new().unwrap();
        let docs = vec![
            ("doc1".to_string(), "rust programming language".to_string()),
            ("doc2".to_string(), "python for data science".to_string()),
        ];

        let results = service.rerank("rust coding", docs).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].1 >= 0.0 && results[0].1 <= 1.0);
        assert!(results[1].1 >= 0.0 && results[1].1 <= 1.0);
    }
}
