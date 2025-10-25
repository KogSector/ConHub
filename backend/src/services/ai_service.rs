use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
}

#[derive(Debug)]
pub enum AIError {
    ApiError(String),
    InvalidRequest(String),
}

impl std::fmt::Display for AIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIError::ApiError(msg) => write!(f, "API error: {}", msg),
            AIError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
        }
    }
}

impl std::error::Error for AIError {}

pub struct AIService {
    openai_api_key: Option<String>,
    anthropic_api_key: Option<String>,
}

impl AIService {
    pub fn new(openai_api_key: Option<String>, anthropic_api_key: Option<String>) -> Self {
        Self {
            openai_api_key,
            anthropic_api_key,
        }
    }

    pub async fn generate_chat_response(&self, messages: Vec<Message>) -> Result<ChatResponse, AIError> {
        // TODO: Call conhub-ai module when it's created
        log::info!("Generating chat response for {} messages", messages.len());

        Err(AIError::ApiError("Not implemented".to_string()))
    }

    pub async fn generate_code_completion(&self, code: &str, language: &str) -> Result<String, AIError> {
        // TODO: Call conhub-ai module when it's created
        log::info!("Generating code completion for language: {}", language);

        Err(AIError::ApiError("Not implemented".to_string()))
    }
}
