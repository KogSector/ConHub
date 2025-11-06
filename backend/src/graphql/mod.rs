pub mod schema;

use async_graphql::{Request as GqlRequest};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use actix_web::{web, HttpRequest, HttpResponse};
use crate::graphql::schema::ConhubSchema;
use conhub_middleware::auth::extract_claims_from_http_request;

pub fn configure_graphql_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/graphql")
            .route("", web::post().to(graphql_handler))
            .route("", web::get().to(graphql_playground))
    );
}

async fn graphql_handler(schema: web::Data<ConhubSchema>, gql: GraphQLRequest, req: HttpRequest) -> GraphQLResponse {
    let mut request: GqlRequest = gql.into_inner();
    // Extract claims and inject into GraphQL context (Send + Sync)
    if let Some(claims) = extract_claims_from_http_request(&req) {
        request = request.data(claims);
    }
    schema.execute(request).await.into()
}

async fn graphql_playground() -> HttpResponse {
    let cfg = GraphQLPlaygroundConfig::new("/api/graphql");
    let html = playground_source(cfg);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}