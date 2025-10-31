use crate::prelude::*;
use async_trait::async_trait;

use crate::base::json_schema::ToJsonSchemaOptions;
use infer::Infer;
use schemars::schema::SchemaObject;
use std::borrow::Cow;

static INFER: LazyLock<Infer> = LazyLock::new(Infer::new);

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
    // VertexAi,
    Bedrock,
}

/*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAiConfig {
    pub project: String,
    pub region: Option<String>,
}
*/

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiConfig {
    pub org_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum LlmApiConfig {
    // VertexAi(VertexAiConfig),
    OpenAi(OpenAiConfig),
}

#[derive(Debug)]
pub enum OutputFormat<'a> {
    JsonSchema {
        name: Cow<'a, str>,
        schema: Cow<'a, SchemaObject>,
    },
}

#[derive(Debug)]
pub struct LlmGenerateRequest<'a> {
    pub model: &'a str,
    pub system_prompt: Option<Cow<'a, str>>,
    pub user_prompt: Cow<'a, str>,
    pub image: Option<Cow<'a, [u8]>>,
    pub output_format: Option<OutputFormat<'a>>,
}

#[derive(Debug)]
pub struct LlmGenerateResponse {
    pub text: String,
}

#[async_trait]
pub trait LlmGenerationClient: Send + Sync {
    async fn generate(
        &self,
        request: LlmGenerateRequest<'_>,
    ) -> Result<LlmGenerateResponse, Box<dyn std::error::Error>>;

    fn json_schema_options(&self) -> ToJsonSchemaOptions;
}

pub fn detect_image_mime_type(bytes: &[u8]) -> Result<&'static str> {
    let infer = &*INFER;
    match infer.get(bytes) {
        Some(info) if info.mime_type().starts_with("image/") => Ok(info.mime_type()),
        _ => bail!("Unknown or unsupported image format"),
    }
}
