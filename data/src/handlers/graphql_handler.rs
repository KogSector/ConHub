use actix_web::{web, HttpRequest, HttpResponse, Result};
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use uuid::Uuid;

use crate::graphql::DataSchema;

/// GraphQL query/mutation handler
pub async fn graphql_handler(
    schema: web::Data<DataSchema>,
    req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = gql_request.into_inner();
    
    // Extract user ID from request headers/claims if available
    if let Some(user_id_str) = req.headers().get("x-user-id") {
        if let Ok(user_id_str) = user_id_str.to_str() {
            if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                request = request.data(user_id);
            }
        }
    }
    
    schema.execute(request).await.into()
}

/// GraphQL subscription handler
pub async fn graphql_subscription(
    schema: web::Data<DataSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(schema.into_inner().as_ref().clone())
        .start(&req, payload)
}

/// GraphQL Playground handler (development only)
pub async fn graphql_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/graphql")
                .subscription_endpoint("/graphql/ws")
        )))
}
