use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CopilotContextRequest {
    pub prompt: String,
}