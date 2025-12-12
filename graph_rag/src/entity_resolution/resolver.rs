use crate::errors::{GraphError, GraphResult};
use crate::models::{
    Entity, CanonicalEntity, EntityFeatures, ResolutionCandidate, ResolutionMatch,
    ResolutionConfig, MatchingStrategy, ResolutionResult,
};
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use tracing::{info, debug};

use super::matchers::{
    calculate_email_match,
    calculate_name_similarity,
    calculate_attribute_overlap,
    calculate_graph_similarity,
};

/// Entity resolution engine that matches entities across data sources
#[derive(Clone)]
pub struct EntityResolver {
    db_pool: PgPool,
    config: ResolutionConfig,
}

impl EntityResolver {
    pub fn new(db_pool: PgPool, config: ResolutionConfig) -> Self {
        Self { db_pool, config }
    }

    /// Resolve a single entity against existing entities
    pub async fn resolve_entity(&self, entity: &Entity) -> GraphResult<Option<Uuid>> {
        // Extract features from entity
        let features = self.extract_features(entity)?;
        
        // Find candidates
        let candidates = self.find_resolution_candidates(entity, &features).await?;
        
        if candidates.is_empty() {
            return Ok(None);
        }

        // Calculate confidence scores for each candidate
        let mut matches = Vec::new();
        for candidate in candidates {
            let score = self.calculate_confidence_score(&features, &candidate).await?;
            
            if score >= self.config.min_confidence_threshold {
                matches.push((candidate.entity_id, score));
            }
        }

        if matches.is_empty() {
            return Ok(None);
        }

        // Sort by confidence and take the best match
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let (best_match_id, best_score) = matches[0];

        info!(
            "Resolved entity {} to canonical entity {} with confidence {}",
            entity.id, best_match_id, best_score
        );

        // Get or create canonical entity
        let canonical_id = self.get_or_create_canonical_entity(
            best_match_id,
            entity,
            best_score,
        ).await?;

        Ok(Some(canonical_id))
    }

    /// Batch resolve multiple entities
    pub async fn batch_resolve(
        &self,
        entity_ids: Vec<Uuid>,
    ) -> GraphResult<Vec<ResolutionResult>> {
        let mut results = Vec::new();

        for entity_id in entity_ids {
            let entity = self.get_entity(entity_id).await?;
            
            if let Some(canonical_id) = self.resolve_entity(&entity).await? {
                let resolved_entities = self.get_resolved_entities(canonical_id).await?;
                
                results.push(ResolutionResult {
                    canonical_id,
                    resolved_entities,
                    confidence_score: 1.0,
                    matching_strategy: "composite".to_string(),
                });
            }
        }

        Ok(results)
    }

    /// Extract features from entity for resolution
    fn extract_features(&self, entity: &Entity) -> GraphResult<EntityFeatures> {
        let props = entity.properties.as_object()
            .ok_or_else(|| GraphError::Internal("Invalid entity properties".to_string()))?;

        let email = props.get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let full_name = props.get("full_name")
            .or(props.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let username = props.get("username")
            .or(props.get("login"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let user_id = props.get("user_id")
            .or(props.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let display_name = props.get("display_name")
            .or(props.get("real_name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let profile_url = props.get("profile_url")
            .or(props.get("html_url"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let associated_repositories = props.get("repositories")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let associated_channels = props.get("channels")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut metadata = HashMap::new();
        for (key, value) in props {
            if let Some(str_val) = value.as_str() {
                metadata.insert(key.clone(), str_val.to_string());
            }
        }

        Ok(EntityFeatures {
            email,
            full_name,
            username,
            user_id,
            display_name,
            profile_url,
            associated_repositories,
            associated_channels,
            metadata,
        })
    }

    /// Find candidate entities for resolution
    async fn find_resolution_candidates(
        &self,
        entity: &Entity,
        features: &EntityFeatures,
    ) -> GraphResult<Vec<ResolutionCandidate>> {
        let mut candidates = Vec::new();

        // Strategy 1: Exact email match (highest confidence)
        if let Some(email) = &features.email {
            let email_matches = sqlx::query_as::<_, Entity>(
                r#"
                SELECT * FROM entities
                WHERE id != $1
                AND entity_type = $2
                AND properties->>'email' = $3
                "#
            )
            .bind(entity.id)
            .bind(&entity.entity_type)
            .bind(email)
            .fetch_all(&self.db_pool)
            .await?;

            for matched_entity in email_matches {
                let matched_features = self.extract_features(&matched_entity)?;
                candidates.push(ResolutionCandidate {
                    entity_id: matched_entity.id,
                    entity_type: matched_entity.entity_type.clone(),
                    source: matched_entity.source.clone(),
                    name: matched_entity.name.clone(),
                    features: matched_features,
                    confidence_score: 0.9,
                });
            }
        }

        // Strategy 2: Fuzzy name match
        if let Some(name) = &features.full_name {
            let name_matches = sqlx::query_as::<_, Entity>(
                r#"
                SELECT * FROM entities
                WHERE id != $1
                AND entity_type = $2
                AND (
                    properties->>'full_name' ILIKE $3 OR
                    properties->>'name' ILIKE $3 OR
                    properties->>'display_name' ILIKE $3
                )
                LIMIT 50
                "#
            )
            .bind(entity.id)
            .bind(&entity.entity_type)
            .bind(format!("%{}%", name))
            .fetch_all(&self.db_pool)
            .await?;

            for matched_entity in name_matches {
                // Skip if already added via email match
                if candidates.iter().any(|c| c.entity_id == matched_entity.id) {
                    continue;
                }

                let matched_features = self.extract_features(&matched_entity)?;
                candidates.push(ResolutionCandidate {
                    entity_id: matched_entity.id,
                    entity_type: matched_entity.entity_type.clone(),
                    source: matched_entity.source.clone(),
                    name: matched_entity.name.clone(),
                    features: matched_features,
                    confidence_score: 0.5,
                });
            }
        }

        // Strategy 3: Username match
        if let Some(username) = &features.username {
            let username_matches = sqlx::query_as::<_, Entity>(
                r#"
                SELECT * FROM entities
                WHERE id != $1
                AND entity_type = $2
                AND (properties->>'username' = $3 OR properties->>'login' = $3)
                "#
            )
            .bind(entity.id)
            .bind(&entity.entity_type)
            .bind(username)
            .fetch_all(&self.db_pool)
            .await?;

            for matched_entity in username_matches {
                if candidates.iter().any(|c| c.entity_id == matched_entity.id) {
                    continue;
                }

                let matched_features = self.extract_features(&matched_entity)?;
                candidates.push(ResolutionCandidate {
                    entity_id: matched_entity.id,
                    entity_type: matched_entity.entity_type.clone(),
                    source: matched_entity.source.clone(),
                    name: matched_entity.name.clone(),
                    features: matched_features,
                    confidence_score: 0.7,
                });
            }
        }

        Ok(candidates)
    }

    /// Calculate composite confidence score
    async fn calculate_confidence_score(
        &self,
        features1: &EntityFeatures,
        candidate: &ResolutionCandidate,
    ) -> GraphResult<f32> {
        let features2 = &candidate.features;

        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        // Email match score
        if let Some(email_score) = calculate_email_match(features1, features2) {
            total_score += email_score * self.config.email_match_weight;
            total_weight += self.config.email_match_weight;
        }

        // Name similarity score
        let name_score = calculate_name_similarity(features1, features2);
        total_score += name_score * self.config.name_similarity_weight;
        total_weight += self.config.name_similarity_weight;

        // Attribute overlap score
        let attr_score = calculate_attribute_overlap(features1, features2);
        total_score += attr_score * self.config.attribute_overlap_weight;
        total_weight += self.config.attribute_overlap_weight;

        // Graph-based similarity (shared connections)
        let graph_score = calculate_graph_similarity(features1, features2);
        total_score += graph_score * self.config.graph_similarity_weight;
        total_weight += self.config.graph_similarity_weight;

        let final_score = if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        };

        debug!(
            "Calculated confidence score: {} for candidate: {}",
            final_score, candidate.entity_id
        );

        Ok(final_score)
    }

    /// Get or create canonical entity
    async fn get_or_create_canonical_entity(
        &self,
        matched_entity_id: Uuid,
        new_entity: &Entity,
        confidence: f32,
    ) -> GraphResult<Uuid> {
        // Check if matched entity already has a canonical entity
        let existing_canonical = sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT canonical_id FROM entities WHERE id = $1"
        )
        .bind(matched_entity_id)
        .fetch_one(&self.db_pool)
        .await?;

        let canonical_id = if let Some(canonical_id) = existing_canonical {
            // Add new entity to existing canonical entity
            sqlx::query(
                "UPDATE entities SET canonical_id = $1, updated_at = NOW() WHERE id = $2"
            )
            .bind(canonical_id)
            .bind(new_entity.id)
            .execute(&self.db_pool)
            .await?;

            // Update canonical entity's source entities list
            sqlx::query(
                r#"
                UPDATE canonical_entities
                SET source_entities = source_entities || $1::jsonb,
                    updated_at = NOW()
                WHERE id = $2
                "#
            )
            .bind(serde_json::json!([new_entity.id]))
            .bind(canonical_id)
            .execute(&self.db_pool)
            .await?;

            canonical_id
        } else {
            // Create new canonical entity
            let canonical_id = Uuid::new_v4();
            let canonical_name = new_entity.name.clone();
            
            sqlx::query(
                r#"
                INSERT INTO canonical_entities 
                (id, entity_type, canonical_name, merged_properties, source_entities, confidence_score, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
                "#
            )
            .bind(canonical_id)
            .bind(&new_entity.entity_type)
            .bind(&canonical_name)
            .bind(serde_json::json!({}))
            .bind(serde_json::json!([matched_entity_id, new_entity.id]))
            .bind(confidence)
            .execute(&self.db_pool)
            .await?;

            // Link both entities to canonical entity
            sqlx::query(
                "UPDATE entities SET canonical_id = $1, updated_at = NOW() WHERE id = ANY($2)"
            )
            .bind(canonical_id)
            .bind(vec![matched_entity_id, new_entity.id])
            .execute(&self.db_pool)
            .await?;

            canonical_id
        };

        Ok(canonical_id)
    }

    async fn get_entity(&self, entity_id: Uuid) -> GraphResult<Entity> {
        sqlx::query_as::<_, Entity>("SELECT * FROM entities WHERE id = $1")
            .bind(entity_id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|_| GraphError::EntityNotFound(entity_id.to_string()))
    }

    async fn get_resolved_entities(&self, canonical_id: Uuid) -> GraphResult<Vec<Uuid>> {
        let entities = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM entities WHERE canonical_id = $1"
        )
        .bind(canonical_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(entities)
    }
}
