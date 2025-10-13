use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Filter, Rejection, Reply};
use serde_json::Value;

#[derive(Clone)]
pub struct WebhookService {
    
    
}

impl WebhookService {
    pub fn new() -> Self {
        WebhookService {}
    }

    pub async fn handle_github_webhook(&self, body: Value) -> Result<impl Reply, Rejection> {
        
        println!("Received GitHub webhook: {}", body);
        Ok(warp::reply::json(&"OK"))
    }
}

pub fn webhook_routes(
    service: Arc<Mutex<WebhookService>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let service = warp::any().map(move || Arc::clone(&service));

    let github_webhook = warp::path("github")
        .and(warp::post())
        .and(warp::body::json())
        .and(service.clone())
        .and_then(
            |body: Value, service: Arc<Mutex<WebhookService>>| async move {
                service.lock().await.handle_github_webhook(body).await
            },
        );

    warp::path("webhooks").and(github_webhook)
}
