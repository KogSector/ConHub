use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use reqwest::Client;


use crate::models::ApiResponse;

#[derive(serde::Deserialize)]
pub struct AgentQueryRequest {
    pub query: String,
    pub agent_type: Option<String>,
    pub include_code: Option<bool>,
    pub include_documents: Option<bool>,
    pub max_results: Option<usize>,
}

#[derive(serde::Serialize)]
pub struct AgentQueryResponse {
    pub query: String,
    pub context: AgentContext,
    pub response: Option<String>,
    pub sources: Vec<ContextSource>,
}

#[derive(serde::Serialize)]
pub struct AgentContext {
    pub code_results: Vec<CodeSearchResult>,
    pub document_results: Vec<DocumentSearchResult>,
    pub total_sources: usize,
}

#[derive(serde::Serialize)]
pub struct CodeSearchResult {
    pub file_path: String,
    pub content: String,
    pub language: String,
    pub repository: String,
    pub relevance_score: f32,
}

#[derive(serde::Serialize)]
pub struct DocumentSearchResult {
    pub title: String,
    pub content: String,
    pub source_type: String,
    pub url: Option<String>,
    pub relevance_score: f32,
}

#[derive(serde::Serialize)]
pub struct ContextSource {
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub relevance_score: f32,
}


pub async fn query_agents(
    req: web::Json<AgentQueryRequest>,
) -> Result<HttpResponse> {
    let client = Client::new();
    let unified_indexer_url = std::env::var("UNIFIED_INDEXER_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let ai_service_url = std::env::var("AI_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8001".to_string());

    let include_code = req.include_code.unwrap_or(true);
    let include_documents = req.include_documents.unwrap_or(true);
    let max_results = req.max_results.unwrap_or(10);

    let mut code_results = Vec::new();
    let mut document_results = Vec::new();
    let mut sources = Vec::new();

    
    if include_code {
        match perform_code_search(&client, &unified_indexer_url, &req.query, max_results).await {
            Ok(results) => {
                for result in results {
                    sources.push(ContextSource {
                        source_type: "code".to_string(),
                        source_id: result.file_path.clone(),
                        title: result.file_path.clone(),
                        relevance_score: result.relevance_score,
                    });
                    code_results.push(result);
                }
            }
            Err(e) => {
                eprintln!("Code search failed: {}", e);
            }
        }
    }

    
    if include_documents {
        match perform_document_search(&client, &ai_service_url, &req.query, max_results).await {
            Ok(results) => {
                for result in results {
                    sources.push(ContextSource {
                        source_type: "document".to_string(),
                        source_id: result.title.clone(),
                        title: result.title.clone(),
                        relevance_score: result.relevance_score,
                    });
                    document_results.push(result);
                }
            }
            Err(e) => {
                eprintln!("Document search failed: {}", e);
            }
        }
    }

    let context = AgentContext {
        code_results,
        document_results,
        total_sources: sources.len(),
    };

    
    let ai_response = if let Some(agent_type) = &req.agent_type {
        match generate_ai_response(&client, &ai_service_url, &req.query, &context, agent_type).await {
            Ok(response) => Some(response),
            Err(e) => {
                eprintln!("AI response generation failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let response = AgentQueryResponse {
        query: req.query.clone(),
        context,
        response: ai_response,
        sources,
    };

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Context retrieved successfully".to_string(),
        data: Some(response),
        error: None,
    }))
}


async fn perform_code_search(
    client: &Client,
    unified_indexer_url: &str,
    query: &str,
    max_results: usize,
) -> Result<Vec<CodeSearchResult>, Box<dyn std::error::Error>> {
    let search_payload = json!({
        "query": query,
        "limit": max_results,
        "search_type": "semantic"
    });

    let response = client
        .post(&format!("{}/api/search", unified_indexer_url))
        .json(&search_payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Unified indexer search failed: {}", response.status()).into());
    }

    let search_results: serde_json::Value = response.json().await?;
    let mut results = Vec::new();

    if let Some(hits) = search_results["results"].as_array() {
        for hit in hits {
            results.push(CodeSearchResult {
                file_path: hit["file_path"].as_str().unwrap_or("").to_string(),
                content: hit["content"].as_str().unwrap_or("").to_string(),
                language: hit["language"].as_str().unwrap_or("").to_string(),
                repository: hit["project_name"].as_str().unwrap_or("").to_string(),
                relevance_score: hit["score"].as_f64().unwrap_or(0.0) as f32,
            });
        }
    }

    Ok(results)
}


async fn perform_document_search(
    client: &Client,
    ai_service_url: &str,
    query: &str,
    max_results: usize,
) -> Result<Vec<DocumentSearchResult>, Box<dyn std::error::Error>> {
    let _search_payload = json!({
        "query": query,
        "k": max_results
    });

    let response = client
        .post(&format!("{}/vector/search", ai_service_url))
        .form(&[
            ("query", query),
            ("k", &max_results.to_string()),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("AI service search failed: {}", response.status()).into());
    }

    let search_results: serde_json::Value = response.json().await?;
    let mut results = Vec::new();

    if let Some(hits) = search_results["results"].as_array() {
        for hit in hits {
            results.push(DocumentSearchResult {
                title: hit["metadata"]["title"].as_str()
                    .or(hit["metadata"]["filename"].as_str())
                    .unwrap_or("Untitled").to_string(),
                content: hit["content"].as_str().unwrap_or("").to_string(),
                source_type: hit["metadata"]["source_type"].as_str().unwrap_or("document").to_string(),
                url: hit["metadata"]["url"].as_str().map(|s| s.to_string()),
                relevance_score: hit["score"].as_f64().unwrap_or(0.0) as f32,
            });
        }
    }

    Ok(results)
}


async fn generate_ai_response(
    client: &Client,
    ai_service_url: &str,
    query: &str,
    context: &AgentContext,
    agent_type: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    
    let formatted_context = format_context_for_agent(context, agent_type);
    
    let ai_payload = json!({
        "agent_type": agent_type,
        "query": query,
        "context": formatted_context
    });

    let response = client
        .post(&format!("{}/ai/query", ai_service_url))
        .json(&ai_payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("AI query failed: {}", response.status()).into());
    }

    let ai_result: serde_json::Value = response.json().await?;
    Ok(ai_result["response"].as_str().unwrap_or("No response generated").to_string())
}


fn format_context_for_agent(context: &AgentContext, agent_type: &str) -> String {
    let mut formatted = String::new();

    match agent_type {
        "github_copilot" => {
            formatted.push_str("# Code Context\n\n");
            for code_result in &context.code_results {
                formatted.push_str(&format!(
                    "## File: {} ({})\n```{}\n{}\n```\n\n",
                    code_result.file_path,
                    code_result.repository,
                    code_result.language,
                    code_result.content
                ));
            }
            
            if !context.document_results.is_empty() {
                formatted.push_str("# Documentation Context\n\n");
                for doc_result in &context.document_results {
                    formatted.push_str(&format!(
                        "## {}\n{}\n\n",
                        doc_result.title,
                        doc_result.content
                    ));
                }
            }
        }
        _ => {
            
            if !context.code_results.is_empty() {
                formatted.push_str("Code Results:\n");
                for code_result in &context.code_results {
                    formatted.push_str(&format!("- {}: {}\n", code_result.file_path, code_result.content));
                }
            }
            
            if !context.document_results.is_empty() {
                formatted.push_str("Document Results:\n");
                for doc_result in &context.document_results {
                    formatted.push_str(&format!("- {}: {}\n", doc_result.title, doc_result.content));
                }
            }
        }
    }

    formatted
}


pub async fn get_agents() -> Result<HttpResponse> {
    let agents = vec![
        json!({
            "id": "github_copilot",
            "name": "GitHub Copilot",
            "description": "AI pair programmer for code assistance",
            "capabilities": ["code_completion", "code_explanation", "bug_fixing"],
            "status": "available"
        }),
        json!({
            "id": "openai_gpt",
            "name": "OpenAI GPT",
            "description": "General purpose AI assistant",
            "capabilities": ["text_generation", "question_answering", "summarization"],
            "status": "available"
        }),
        json!({
            "id": "anthropic_claude",
            "name": "Anthropic Claude",
            "description": "AI assistant for analysis and reasoning",
            "capabilities": ["analysis", "reasoning", "writing"],
            "status": "available"
        })
    ];

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Available AI agents retrieved successfully".to_string(),
        data: Some(agents),
        error: None,
    }))
}


pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/agents")
            .route("/query", web::post().to(query_agents))
            .route("", web::get().to(get_agents))
    );
}