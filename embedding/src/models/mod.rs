use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TextInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub struct EmbedRequest {
    pub text: TextInput,
    #[serde(default = "default_normalize")]
    pub normalize: bool,
}

fn default_normalize() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct EmbedResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub dimension: usize,
    pub model: String,
    pub count: usize,
}

#[derive(Debug, Deserialize)]
pub struct DocumentInput {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct RerankRequest {
    pub query: String,
    pub documents: Vec<DocumentInput>,
    pub top_k: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RerankResult {
    pub id: String,
    pub score: f32,
}

#[derive(Debug, Serialize)]
pub struct RerankResponse {
    pub results: Vec<RerankResult>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub model_loaded: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
