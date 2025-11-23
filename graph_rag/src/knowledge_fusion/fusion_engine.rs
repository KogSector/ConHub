use crate::errors::{GraphError, GraphResult};
use crate::models::{Entity, Relationship, CanonicalEntity};
use crate::entity_resolution::EntityResolver;
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::HashMap;
use tracing::{info, warn};

/// Knowledge fusion engine that merges multi-source data
pub struct FusionEngine {
    db_pool: PgPool,
    entity_resolver: EntityResolver,
}

impl FusionEngine {
    pub fn new(db_pool: PgPool, entity_resolver: EntityResolver) -> Self {
        Self {
            db_pool,
            entity_resolver,
        }
    }

    /// Fuse entities from multiple sources into unified graph
    pub async fn fuse_entities(&self, entities: Vec<Entity>) -> GraphResult<Vec<Uuid>> {
        let mut fused_ids = Vec::new();

        for entity in entities {
            // Insert entity into database
            let entity_id = self.insert_entity(&entity).await?;
            
            // Attempt entity resolution
            match self.entity_resolver.resolve_entity(&entity).await {
                Ok(Some(canonical_id)) => {
                    info!("Entity {} resolved to canonical {}", entity_id, canonical_id);
                    fused_ids.push(canonical_id);
                }
                Ok(None) => {
                    info!("Entity {} has no matches, creating standalone", entity_id);
                    fused_ids.push(entity_id);
                }
                Err(e) => {
                    warn!("Failed to resolve entity {}: {}", entity_id, e);
                    fused_ids.push(entity_id);
                }
            }
        }

        Ok(fused_ids)
    }

    /// Merge relationships from multiple sources
    pub async fn fuse_relationships(&self, relationships: Vec<Relationship>) -> GraphResult<usize> {
        let mut count = 0;

        for rel in relationships {
            match self.insert_relationship(&rel).await {
                Ok(_) => count += 1,
                Err(e) => warn!("Failed to insert relationship: {}", e),
            }
        }

        Ok(count)
    }

    /// Merge properties from multiple source entities
    pub async fn merge_properties(&self, canonical_id: Uuid) -> GraphResult<()> {
        let source_entities = sqlx::query_as::<_, Entity>(
            "SELECT * FROM entities WHERE canonical_id = $1"
        )
        .bind(canonical_id)
        .fetch_all(&self.db_pool)
        .await?;

        let mut merged_props = HashMap::new();

        for entity in source_entities {
            if let Some(props) = entity.properties.as_object() {
                for (key, value) in props {
                    merged_props.entry(key.clone())
                        .or_insert_with(Vec::new)
                        .push(value.clone());
                }
            }
        }

        let merged_json = serde_json::to_value(merged_props)?;

        sqlx::query(
            "UPDATE canonical_entities SET merged_properties = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(merged_json)
        .bind(canonical_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn insert_entity(&self, entity: &Entity) -> GraphResult<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO entities (id, entity_type, source, source_id, name, properties, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (source, source_id) DO UPDATE
            SET name = EXCLUDED.name,
                properties = EXCLUDED.properties,
                updated_at = EXCLUDED.updated_at
            RETURNING id
            "#
        )
        .bind(entity.id)
        .bind(&entity.entity_type)
        .bind(&entity.source)
        .bind(&entity.source_id)
        .bind(&entity.name)
        .bind(&entity.properties)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(id)
    }

    async fn insert_relationship(&self, rel: &Relationship) -> GraphResult<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO relationships (id, from_entity_id, to_entity_id, relationship_type, properties, weight, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            RETURNING id
            "#
        )
        .bind(rel.id)
        .bind(rel.from_entity_id)
        .bind(rel.to_entity_id)
        .bind(&rel.relationship_type)
        .bind(&rel.properties)
        .bind(rel.weight)
        .bind(rel.created_at)
        .bind(rel.updated_at)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(id)
    }
}
