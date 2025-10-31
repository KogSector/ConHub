use crate::prelude::*;
use crate::utils;

use crate::llm::LlmEmbeddingClient;
use pyo3_async_runtimes::generic::run;
use retryable::RetryOptions;
use google_cloud_aiplatform_v1 as vertexai;
use google_cloud_gax::exponential_backoff::ExponentialBackoff;
use google_cloud_gax::options::RequestOptionsBuilder;
use google_cloud_gax::error;
use google_cloud_gax::retry_policy::{Aip194Strict, RetryPolicyExt};
use google_cloud_gax::retry_throttler::{AdaptiveThrottler, SharedRetryThrottler};
use serde_json::Value;
use urlencoding::encode;

fn get_embedding_dimension(model: &str) -> Option<u32> {
    let model = model.to_ascii_lowercase();
    if model.starts_with("gemini-embedding-") {
        Some(3072)
    } else if model.starts_with("text-embedding-") {
        Some(768)
    } else if model.starts_with("embedding-") {
        Some(768)
    } else if model.starts_with("text-multilingual-embedding-") {
        Some(768)
    } else {
        None
    }
}

pub struct AiStudioClient {
    api_key: String,
    client: reqwest::Client,
}

impl AiStudioClient {
    pub fn new(address: Option<String>) -> Result<Self> {
        if address.is_some() {
            api_bail!("Gemini doesn't support custom API address");
        }
        let api_key = match std::env::var("GEMINI_API_KEY") {
            Ok(val) => val,
            Err(_) => api_bail!("GEMINI_API_KEY environment variable must be set"),
        };
        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
        })
    }
}

impl AiStudioClient {
    fn get_api_url(&self, model: &str, api_name: &str) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:{}?key={}",
            encode(model),
            api_name,
            encode(&self.api_key)
        )
    }
}

fn build_embed_payload(
    model: &str,
    text: &str,
    task_type: Option<&str>,
    output_dimension: Option<u32>,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "model": model,
        "content": { "parts": [{ "text": text }] },
    });
    if let Some(task_type) = task_type {
        payload["taskType"] = serde_json::Value::String(task_type.to_string());
    }
    if let Some(output_dimension) = output_dimension {
        payload["outputDimensionality"] = serde_json::json!(output_dimension);
        if model.starts_with("gemini-embedding-") {
            payload["config"] = serde_json::json!({
                "outputDimensionality": output_dimension,
            });
        }
    }
    payload
}

#[derive(Deserialize)]
struct ContentEmbedding {
    values: Vec<f32>,
}
#[derive(Deserialize)]
struct EmbedContentResponse {
    embedding: ContentEmbedding,
}

#[async_trait]
impl LlmEmbeddingClient for AiStudioClient {
    async fn embed_text<'req>(
        &self,
        request: super::LlmEmbeddingRequest<'req>,
    ) -> Result<super::LlmEmbeddingResponse> {
        let url = self.get_api_url(request.model, "embedContent");
        let payload = build_embed_payload(
            request.model,
            request.text.as_ref(),
            request.task_type.as_deref(),
            request.output_dimension,
        );
        let resp = run(
            || async {
                self.client
                    .post(&url)
                    .json(&payload)
                    .send()
                    .await?
                    .error_for_status()
            },
            RetryOptions::default(),
        )
        .await
        .context("Gemini API error")?;
        let embedding_resp: EmbedContentResponse = resp.json().await.context("Invalid JSON")?;
        Ok(super::LlmEmbeddingResponse {
            embedding: embedding_resp.embedding.values,
        })
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        get_embedding_dimension(model)
    }

    fn behavior_version(&self) -> Option<u32> {
        Some(2)
    }
}

pub struct VertexAiClient {
    client: vertexai::client::PredictionService,
    config: super::VertexAiConfig,
}

#[derive(Debug)]
struct CustomizedGoogleCloudRetryPolicy;

impl google_cloud_gax::retry_policy::RetryPolicy for CustomizedGoogleCloudRetryPolicy {
    fn on_error(
        &self,
        state: &google_cloud_gax::retry_state::RetryState,
        error: &error::Error,
    ) -> google_cloud_gax::retry_result::RetryResult {
        use google_cloud_gax::retry_result::RetryResult;

        if let Some(status) = error.status() {
            if status.code() == google_cloud_gax::error::rpc::Code::ResourceExhausted {
                return RetryResult::Continue;
            }
        } else if let Some(code) = error.http_status_code() {
            if code == reqwest::StatusCode::TOO_MANY_REQUESTS.as_u16() {
                return RetryResult::Continue;
            }
        }
        Aip194Strict.on_error(state, error)
    }
}

static SHARED_RETRY_THROTTLER: LazyLock<SharedRetryThrottler> =
    LazyLock::new(|| Arc::new(Mutex::new(AdaptiveThrottler::new(2.0).unwrap())));

impl VertexAiClient {
    pub async fn new(
        address: Option<String>,
        api_config: Option<super::LlmApiConfig>,
    ) -> Result<Self> {
        if address.is_some() {
            api_bail!("VertexAi API address is not supported for VertexAi API type");
        }
        let Some(super::LlmApiConfig::VertexAi(config)) = api_config else {
            api_bail!("VertexAi API config is required for VertexAi API type");
        };
        let client = vertexai::client::PredictionService::builder()
            .with_retry_policy(
                CustomizedGoogleCloudRetryPolicy.with_time_limit(std::time::Duration::from_secs(60)),
            )
            .with_backoff_policy(ExponentialBackoff::default())
            .with_retry_throttler(SHARED_RETRY_THROTTLER.clone())
            .build()
            .await?;
        Ok(Self { client, config })
    }

    fn get_model_path(&self, model: &str) -> String {
        format!(
            "projects/{}/locations/{}/publishers/google/models/{}",
            self.config.project,
            self.config.region.as_deref().unwrap_or("global"),
            model
        )
    }
}

#[async_trait]
impl LlmEmbeddingClient for VertexAiClient {
    async fn embed_text<'req>(
        &self,
        request: super::LlmEmbeddingRequest<'req>,
    ) -> Result<super::LlmEmbeddingResponse> {
        // Create the instances for the request
        let mut instance = serde_json::json!({
            "content": request.text
        });
        // Add task type if specified
        if let Some(task_type) = &request.task_type {
            instance["task_type"] = serde_json::Value::String(task_type.to_string());
        }

        let instances = vec![instance];

        // Prepare the request parameters
        let mut parameters = serde_json::json!({});
        if let Some(output_dimension) = request.output_dimension {
            parameters["outputDimensionality"] = serde_json::Value::Number(output_dimension.into());
        }

        // Build the prediction request using the raw predict builder
        let response = self
            .client
            .predict()
            .set_endpoint(self.get_model_path(request.model))
            .set_instances(instances)
            .set_parameters(parameters)
            .with_idempotency(true)
            .send()
            .await?;

        // Extract the embedding from the response
        let embeddings = response
            .predictions
            .into_iter()
            .next()
            .and_then(|mut e| e.get_mut("embeddings").map(|v| v.take()))
            .ok_or_else(|| anyhow::anyhow!("No embeddings in response"))?;
        let embedding: ContentEmbedding = utils::deser::from_json_value(embeddings)?;
        Ok(super::LlmEmbeddingResponse {
            embedding: embedding.values,
        })
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        get_embedding_dimension(model)
    }
}
