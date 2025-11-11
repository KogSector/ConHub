use async_graphql::*;
use sqlx::PgPool;
use std::sync::Arc;

use super::queries::QueryRoot;
use super::mutations::MutationRoot;
use super::subscriptions::SubscriptionRoot;
use crate::connectors::ConnectorManager;

pub type DataSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

/// Create the GraphQL schema
pub fn create_schema(
    pool: Option<PgPool>,
    connector_manager: Arc<ConnectorManager>,
) -> DataSchema {
    let mut schema_builder = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(pool)
        .data(connector_manager);
    
    // Set limits and configurations
    schema_builder = schema_builder
        .limit_depth(10)
        .limit_complexity(100);
    
    schema_builder.finish()
}

/// Create schema with user context (for authenticated requests)
pub fn create_schema_with_user(
    pool: Option<PgPool>,
    connector_manager: Arc<ConnectorManager>,
    user_id: uuid::Uuid,
) -> DataSchema {
    Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(pool)
        .data(connector_manager)
        .data(user_id)
        .limit_depth(10)
        .limit_complexity(100)
        .finish()
}
