use crate::errors::GraphResult;
use crate::models::{Entity, Relationship, CreateEntityRequest, CreateEntityResponse, ResolutionConfig};
use crate::entity_resolution::EntityResolver;
use crate::knowledge_fusion::FusionEngine;
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::HashMap;

pub struct GraphService {
    db_pool: PgPool,
    entity_resolver: EntityResolver,
    fusion_engine: FusionEngine,
}

impl GraphService {
    pub fn new(db_pool: PgPool) -> Self {
        let config = ResolutionConfig::default();
        let entity_resolver = EntityResolver::new(db_pool.clone(), config.clone());
        let fusion_engine = FusionEngine::new(db_pool.clone(), entity_resolver.clone());

        Self {
            db_pool,
            entity_resolver,
            fusion_engine,
        }
    }

    pub async fn create_entity(&self, req: CreateEntityRequest) -> GraphResult<CreateEntityResponse> {
        // Create entity from request
        let mut props = HashMap::new();
        for (k, v) in req.properties {
            props.insert(k, v);
        }

        let entity_type = crate::models::EntityType::from_str(&req.entity_type)
            .ok_or_else(|| crate::errors::GraphError::InvalidEntityType(req.entity_type.clone()))?;

        let source = match req.source.as_str() {
            "github" => crate::models::DataSource::GitHub,
            "slack" => crate::models::DataSource::Slack,
            "notion" => crate::models::DataSource::Notion,
            _ => crate::models::DataSource::GitHub,
        };

        let entity = Entity::new(
            entity_type,
            source,
            req.source_id.clone(),
            req.name.clone(),
            props,
        );

        // Fuse entity (includes resolution)
        let canonical_ids = self.fusion_engine.fuse_entities(vec![entity.clone()]).await?;
        let canonical_id = canonical_ids.first().copied();

        Ok(CreateEntityResponse {
            entity_id: entity.id,
            canonical_id,
            resolved: canonical_id.is_some(),
        })
    }

    pub async fn get_entity(&self, entity_id: Uuid) -> GraphResult<Entity> {
        sqlx::query_as::<_, Entity>("SELECT * FROM entities WHERE id = $1")
            .bind(entity_id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|_| crate::errors::GraphError::EntityNotFound(entity_id.to_string()))
    }
}
