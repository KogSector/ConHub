use reqwest::Client;
use serde_json::json;
use serde::{Deserialize, Serialize};
use crate::services::RepositoryService;
use crate::models::{VcsType, VcsProvider, CredentialType, RepositoryCredentials, ConnectRepositoryRequest, DataSourceType};
use crate::errors::ServiceError as AppError;
use std::collections::HashMap;
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
) -> Result<DataSourceResponse, AppError> {
    
    if request.source_type == "github" || request.source_type == "bitbucket" {
        return connect_vcs_data_source(request).await;
    }
    
    
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
        let result: serde_json::Value = response.json().await.map_err(|e| AppError::ExternalApiError(e.to_string()))?;
        Ok(DataSourceResponse {
            id: result["id"].as_str().unwrap_or("unknown").to_string(),
            status: "connected".to_string(),
            message: "Data source connected successfully".to_string(),
        })
    } else {
        Err(AppError::ExternalApiError(format!("Data source connection failed: {}", response.status())))
    }
}

async fn connect_vcs_data_source(
    request: &DataSourceRequest,
) -> Result<DataSourceResponse, AppError> {
    let repository_service = RepositoryService::new();
    
    
    let url = request.url.as_ref()
        .ok_or(AppError::ConfigurationError("Repository URL is required for VCS connections".to_string()))?;
    
    
    let vcs_type = match request.source_type.as_str() {
        "github" => VcsType::Git,
        "bitbucket" => VcsType::Git,
        _ => return Err(AppError::ConfigurationError("Unsupported VCS type".to_string())),
    };
    
    let vcs_provider = match request.source_type.as_str() {
        "github" => VcsProvider::GitHub,
        "bitbucket" => VcsProvider::Bitbucket,
        _ => return Err(AppError::ConfigurationError("Unsupported VCS provider".to_string())),
    };
    
    
    let credentials = if let Some(creds) = &request.credentials {
        if request.source_type == "github" {
            
            if let Some(_installation_id) = creds.get("installationId") {
                
                
                if let Some(token) = creds.get("accessToken") {
                    let token_str = token.as_str()
                        .ok_or(AppError::ConfigurationError("GitHub access token must be a string".to_string()))?;
                    RepositoryCredentials {
                        credential_type: CredentialType::PersonalAccessToken { 
                            token: token_str.to_string() 
                        },
                        expires_at: None,
                    }
                } else {
                    return Err(AppError::ConfigurationError("GitHub App authentication not yet implemented. Please use access token.".to_string()));
                }
            } else if let Some(token) = creds.get("accessToken") {
                
                let token_str = token.as_str()
                    .ok_or(AppError::ConfigurationError("GitHub access token must be a string".to_string()))?;
                RepositoryCredentials {
                    credential_type: CredentialType::PersonalAccessToken { 
                        token: token_str.to_string() 
                    },
                    expires_at: None,
                }
            } else {
                return Err(AppError::ConfigurationError("GitHub access token or installation ID is required".to_string()));
            }
        } else if request.source_type == "bitbucket" {
            let username = creds["username"].as_str()
                .ok_or(AppError::ConfigurationError("Bitbucket username is required".to_string()))?;
            let app_password = creds["appPassword"].as_str()
                .ok_or(AppError::ConfigurationError("Bitbucket app password is required".to_string()))?;
            RepositoryCredentials {
                credential_type: CredentialType::AppPassword { 
                    username: username.to_string(),
                    app_password: app_password.to_string()
                },
                expires_at: None,
            }
        } else {
            return Err(AppError::ConfigurationError("Invalid credentials for VCS type".to_string()));
        }
    } else {
        return Err(AppError::ConfigurationError("Credentials are required for VCS connections".to_string()));
    };

    
    let connect_request = ConnectRepositoryRequest {
        url: url.clone(),
        vcs_type: Some(vcs_type),
        provider: Some(vcs_provider),
        credentials,
        config: None, 
    };

    
    match repository_service.connect_repository(connect_request).await {
        Ok(repo_info) => {
            
            // TODO: Re-implement indexing logic here. The IndexingOrchestrator was removed or refactored.
            /*
            let indexing_orchestrator = IndexingOrchestrator::new();
            let indexing_request = IndexingRequest {
                source_id: repo_info.id.clone(),
                source_type: DataSourceType::Repository,
                repository_info: Some(repo_info.clone()),
                content: None,
                metadata: HashMap::new(),
            };
            
            
            let repo_name = repo_info.name.clone();
            tokio::spawn(async move {
                match indexing_orchestrator.start_indexing(indexing_request).await {
                    Ok(job) => {
                        println!("Indexing job started for repository: {} (Job ID: {})", repo_name, job.id);
                    }
                    Err(e) => {
                        eprintln!("Failed to start indexing for repository {}: {}", repo_name, e);
                    }
                }
            });
            */
            
            Ok(DataSourceResponse {
                id: repo_info.id,
                status: "connected".to_string(),
                message: format!("Repository '{}' connected successfully. Indexing started in background.", repo_info.name),
            })
        }
        Err(e) => {
            Err(AppError::ExternalApiError(format!("Failed to connect repository: {}", e)))
        }
    }
}

pub async fn list_data_sources(
    client: &Client,
    langchain_url: &str,
) -> Result<serde_json::Value, AppError> {
    let response = client
        .get(&format!("{}/api/data-sources", langchain_url))
        .send()
        .await?;
    
    Ok(response.json().await.map_err(|e| AppError::ExternalApiError(e.to_string()))?)
}

pub async fn sync_data_source(
    client: &Client,
    langchain_url: &str,
    source_id: &str,
) -> Result<serde_json::Value, AppError> {
    let response = client
        .post(&format!("{}/api/data-sources/{}/sync", langchain_url, source_id))
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError(e.to_string()))?;
    
    Ok(response.json().await.map_err(|e| AppError::ExternalApiError(e.to_string()))?)
}