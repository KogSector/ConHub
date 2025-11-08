use anyhow::{anyhow, Result};
use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use super::LlmEmbeddingClient;

static DEFAULT_EMBEDDING_DIMENSIONS: phf::Map<&str, u32> = phf::phf_map! {
    // Qwen3 Embedding default dimensions per model size
    "Qwen/Qwen3-Embedding-0.6B" => 1024,
    "Qwen/Qwen3-Embedding-4B" => 2560,
    "Qwen/Qwen3-Embedding-8B" => 4096,
};

pub struct Client {
    model: Py<PyAny>,
    model_name: String,
}

impl Client {
    pub fn new(model_name: &str) -> Result<Self> {
        // Load SentenceTransformer model via PyO3
        let model: Py<PyAny> = Python::with_gil(|py| -> Result<Py<PyAny>> {
            let st = py
                .import("sentence_transformers")
                .map_err(|e| anyhow!("Failed to import sentence_transformers: {e}"))?;
            let cls = st
                .getattr("SentenceTransformer")
                .map_err(|e| anyhow!("Failed to access SentenceTransformer: {e}"))?;
            let instance = cls
                .call1((model_name,))
                .map_err(|e| anyhow!("Failed to load model {model_name}: {e}"))?;
            Ok(instance.into())
        })?;

        Ok(Self { model, model_name: model_name.to_string() })
    }
}

#[async_trait]
impl LlmEmbeddingClient for Client {
    async fn embed_text<'req>(
        &self,
        request: super::LlmEmbeddingRequest<'req>,
    ) -> Result<super::LlmEmbeddingResponse> {
        // Encode text using SentenceTransformer (no normalization here; done in Rust)
        let embedding: Vec<f32> = Python::with_gil(|py| -> Result<Vec<f32>> {
            let model = self.model.bind(py);
            let kwargs = PyDict::new(py);
            kwargs.set_item("normalize_embeddings", false)?;
            let result = model
                .call_method("encode", &(request.text.as_ref(),), Some(&kwargs))
                .map_err(|e| anyhow!("Python encode() failed: {e}"))?;

            // Convert to list then extract first vector
            let list = result
                .call_method("tolist", (), None)
                .map_err(|e| anyhow!("tolist() failed: {e}"))?;
            let py_list = list.downcast::<PyList>()
                .map_err(|e| anyhow!("Failed to downcast to list: {e}"))?;
            let mut out: Vec<f32> = Vec::with_capacity(py_list.len());
            for item in py_list.iter() {
                let val: f32 = item.extract().map_err(|e| anyhow!("Invalid float in embedding: {e}"))?;
                out.push(val);
            }
            Ok(out)
        })?;

        Ok(super::LlmEmbeddingResponse { embedding })
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        DEFAULT_EMBEDDING_DIMENSIONS.get(model).copied()
    }
}