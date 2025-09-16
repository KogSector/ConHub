use reqwest::Client;
use serde_json::json;
use serde::{Deserialize, Serialize};
use crate::services::enhanced_repository::RepositoryService;
use crate::models::{VcsType, VcsProvider, CredentialType, RepositoryCredentials, ConnectRepositoryRequest};
use chrono;

#[derive(Deserialize)]
pub struct DataSourceRequest {
    pub source_type: String,
    pub url: Option<String>,
    pub config: serde_json::Value,
    pub credentials: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct DataSourceResponse {
    pub id: String,
    pub status: String,
    pub message: String,
}

pub async fn connect_data_source(
    client: &Client,
    langchain_url: &str,
    request: &DataSourceRequest,
) -> Result<DataSourceResponse, Box<dyn std::error::Error>> {
    // If this is a VCS connection, use our VCS system
    if request.source_type == "github" || request.source_type == "bitbucket" {
        return connect_vcs_data_source(request).await;
    }
    
    // Otherwise, forward to LangChain service for other data sources
    let payload = json!({
        "type": request.source_type,
        "config": request.config,
        "credentials": request.credentials
    });
    
    let response = client
        .post(&format!("{}/api/data-sources/connect", langchain_url))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        Ok(DataSourceResponse {
            id: result["id"].as_str().unwrap_or("unknown").to_string(),
            status: "connected".to_string(),
            message: "Data source connected successfully".to_string(),
        })
    } else {
        Err(format!("Data source connection failed: {}", response.status()).into())
    }
}

async fn connect_vcs_data_source(
    request: &DataSourceRequest,
) -> Result<DataSourceResponse, Box<dyn std::error::Error>> {
    let repository_service = RepositoryService::new();
    
    // Get repository URL
    let url = request.url.as_ref()
        .ok_or("Repository URL is required for VCS connections")?;
    
    // Parse VCS type and provider
    let vcs_type = match request.source_type.as_str() {
        "github" => VcsType::Git,
        "bitbucket" => VcsType::Git,
        _ => return Err("Unsupported VCS type".into()),
    };
    
    let vcs_provider = match request.source_type.as_str() {
        "github" => VcsProvider::GitHub,
        "bitbucket" => VcsProvider::Bitbucket,
        _ => return Err("Unsupported VCS provider".into()),
    };
    
    // Parse credentials
    let credentials = if let Some(creds) = &request.credentials {
        if request.source_type == "github" {
            let token = creds["accessToken"].as_str()
                .ok_or("GitHub access token is required")?;
            RepositoryCredentials {
                credential_type: CredentialType::PersonalAccessToken { 
                    token: token.to_string() 
                },
                expires_at: None,
            }
        } else if request.source_type == "bitbucket" {
            let username = creds["username"].as_str()
                .ok_or("Bitbucket username is required")?;
            let app_password = creds["appPassword"].as_str()
                .ok_or("Bitbucket app password is required")?;
            RepositoryCredentials {
                credential_type: CredentialType::AppPassword { 
                    username: username.to_string(),
                    app_password: app_password.to_string()
                },
                expires_at: None,
            }
        } else {
            return Err("Invalid credentials for VCS type".into());
        }
    } else {
        return Err("Credentials are required for VCS connections".into());
    };

    // Create repository connection request
    let connect_request = ConnectRepositoryRequest {
        url: url.clone(),
        vcs_type: Some(vcs_type),
        provider: Some(vcs_provider),
        credentials,
        config: None, // We'll use defaults for now
    };

    // Connect the repository using our VCS service
    match repository_service.connect_repository(connect_request).await {
        Ok(repo_info) => {
            Ok(DataSourceResponse {
                id: repo_info.id,
                status: "connected".to_string(),
                message: format!("Repository '{}' connected successfully", repo_info.name),
            })
        }
        Err(e) => {
            Err(format!("Failed to connect repository: {}", e).into())
        }
    }
}

pub async fn list_data_sources(
    client: &Client,
    langchain_url: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let response = client
        .get(&format!("{}/api/data-sources", langchain_url))
        .send()
        .await?;
    
    Ok(response.json().await?)
}

pub async fn sync_data_source(
    client: &Client,
    langchain_url: &str,
    source_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let response = client
        .post(&format!("{}/api/data-sources/{}/sync", langchain_url, source_id))
        .send()
        .await?;
    
    Ok(response.json().await?)
}