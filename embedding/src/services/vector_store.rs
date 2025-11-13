use anyhow::{anyhow, Result};
use qdrant_client::{prelude::*, qdrant::{vectors_config::Config, CreateCollection, Distance, PointStruct, VectorParams, VectorsConfig}};
use serde_json::{Map, Value};

pub struct VectorStoreService {
    client: QdrantClient,
    timeout_secs: u64,
}

impl VectorStoreService {
    pub async fn new(url: &str, timeout_secs: u64) -> Result<Self> {
        let client = QdrantClient::from_url(url).build().map_err(|e| anyhow!("{}", e))?;
        Ok(Self { client, timeout_secs })
    }

    pub async fn ensure_collection(&self, name: &str, dimension: usize) -> Result<()> {
        let collections = self.client.list_collections().await.map_err(|e| anyhow!("{}", e))?.collections;
        let exists = collections.iter().any(|c| c.name == name);
        if exists { return Ok(()); }
        let req = CreateCollection {
            collection_name: name.to_string(),
            vectors_config: Some(VectorsConfig { config: Some(Config::Params(VectorParams { size: dimension as u64, distance: Distance::Cosine.into(), ..Default::default() })) }),
            ..Default::default()
        };
        self.client.create_collection(&req).await.map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    pub async fn upsert(&self, collection: &str, points: Vec<(String, Vec<f32>, Map<String, Value>)>) -> Result<()> {
        if points.is_empty() { return Ok(()); }
        let qpoints: Vec<PointStruct> = points.into_iter().map(|(id, v, payload)| PointStruct::new(id, v, payload)).collect();
        self.client.upsert_points_blocking(collection, None, qpoints, None).await.map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }
}
