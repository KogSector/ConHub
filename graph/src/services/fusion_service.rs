use crate::knowledge_fusion::FusionEngine;
use crate::errors::GraphResult;
use crate::models::{Entity, Relationship};
use uuid::Uuid;

pub struct FusionService {
    fusion_engine: FusionEngine,
}

impl FusionService {
    pub fn new(fusion_engine: FusionEngine) -> Self {
        Self { fusion_engine }
    }

    pub async fn fuse_batch(&self, entities: Vec<Entity>, relationships: Vec<Relationship>) -> GraphResult<(Vec<Uuid>, usize)> {
        let entity_ids = self.fusion_engine.fuse_entities(entities).await?;
        let rel_count = self.fusion_engine.fuse_relationships(relationships).await?;
        Ok((entity_ids, rel_count))
    }
}
