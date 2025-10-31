use crate::prelude::*;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LlmApiType {
    Ollama,
    OpenAi,
    Gemini,
    Anthropic,
    LiteLlm,
    OpenRouter,
    Voyage,
    Vllm,
    VertexAi,
    Bedrock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAiConfig {
    pub project: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiConfig {
    pub org_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum LlmApiConfig {
    VertexAi(VertexAiConfig),
    OpenAi(OpenAiConfig),
}

#[derive(Debug)]
pub struct LlmEmbeddingRequest<'a> {
    pub model: &'a str,
    pub text: Cow<'a, str>,
    pub output_dimension: Option<u32>,
    pub task_type: Option<Cow<'a, str>>,
}

pub struct LlmEmbeddingResponse {
    pub embedding: Vec<f32>,
}

#[async_trait]
pub trait LlmEmbeddingClient: Send + Sync {
    async fn embed_text<'req>(
        &self,
        request: LlmEmbeddingRequest<'req>,
    ) -> Result<LlmEmbeddingResponse>;

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32>;

    fn behavior_version(&self) -> Option<u32> {
        Some(1)
    }
}
