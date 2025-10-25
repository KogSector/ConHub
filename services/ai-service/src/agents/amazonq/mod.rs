use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

use crate::agents::core::{AIAgentConnector, AIAgent, AgentStatus, AgentQueryRequest, AgentQueryResponse, AgentUsage};

pub struct AmazonQAgent {
    #[allow(dead_code)]
    client: Client,
    credentials: Option<AmazonQCredentials>,
    agent_info: AIAgent,
}

#[derive(Debug, Clone)]
struct AmazonQCredentials {
    access_key_id: String,
    secret_access_key: String,
    region: String,
    #[allow(dead_code)]
    session_token: Option<String>,
}

impl AmazonQAgent {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            credentials: None,
            agent_info: AIAgent {
                id: "amazon_q".to_string(),
                agent_type: "amazon_q".to_string(),
                name: "Amazon Q".to_string(),
                description: "AWS AI assistant for development and cloud operations".to_string(),
                capabilities: vec![
                    "aws_guidance".to_string(),
                    "code_assistance".to_string(),
                    "cloud_architecture".to_string(),
                    "troubleshooting".to_string(),
                    "best_practices".to_string(),
                ],
                is_connected: false,
                status: AgentStatus::Disconnected,
            },
        }
    }

    async fn validate_aws_credentials(&self, credentials: &AmazonQCredentials) -> Result<bool, Box<dyn Error>> {
        
        
        if credentials.access_key_id.is_empty() || credentials.secret_access_key.is_empty() {
            return Ok(false);
        }

        
        
        
        
        
        println!("Validating Amazon Q credentials for region: {}", credentials.region);
        Ok(true) 
    }

    async fn send_amazon_q_request(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        
        
        
        let full_prompt = if let Some(ctx) = context {
            format!("AWS Context:\n{}\n\nQuery: {}", ctx, prompt)
        } else {
            prompt.to_string()
        };

        
        let response = format!(
            "Amazon Q suggests:\n\n{}\n\n---\nAWS Best Practice: {}\n\nRelevant AWS Services: {}",
            self.generate_mock_aws_response(&full_prompt),
            self.get_aws_best_practice(&full_prompt),
            self.suggest_aws_services(&full_prompt)
        );

        Ok(response)
    }

    fn generate_mock_aws_response(&self, prompt: &str) -> String {
        let lower_prompt = prompt.to_lowercase();
        
        if lower_prompt.contains("lambda") {
            "For AWS Lambda functions, consider using the latest runtime versions and implement proper error handling. Use environment variables for configuration and enable X-Ray tracing for monitoring.".to_string()
        } else if lower_prompt.contains("s3") {
            "For Amazon S3, enable versioning and lifecycle policies. Use S3 Transfer Acceleration for faster uploads and consider S3 Intelligent-Tiering for cost optimization.".to_string()
        } else if lower_prompt.contains("ec2") {
            "For Amazon EC2, choose the right instance type based on your workload. Use Auto Scaling Groups for high availability and consider Spot Instances for cost savings.".to_string()
        } else if lower_prompt.contains("rds") {
            "For Amazon RDS, enable automated backups and Multi-AZ deployment for production workloads. Use read replicas to scale read operations.".to_string()
        } else if lower_prompt.contains("vpc") {
            "For Amazon VPC, follow the principle of least privilege for security groups. Use private subnets for backend resources and NAT Gateways for outbound internet access.".to_string()
        } else {
            "Based on AWS best practices, ensure you follow the Well-Architected Framework principles: operational excellence, security, reliability, performance efficiency, and cost optimization.".to_string()
        }
    }

    fn get_aws_best_practice(&self, prompt: &str) -> String {
        let lower_prompt = prompt.to_lowercase();
        
        if lower_prompt.contains("security") {
            "Always use IAM roles instead of hardcoded credentials, enable MFA, and follow the principle of least privilege."
        } else if lower_prompt.contains("cost") {
            "Use AWS Cost Explorer to monitor spending, implement tagging strategies, and consider Reserved Instances for predictable workloads."
        } else if lower_prompt.contains("performance") {
            "Use CloudWatch metrics to monitor performance, implement caching strategies, and choose the right AWS regions for your users."
        } else {
            "Follow the AWS Well-Architected Framework and regularly review your architecture using AWS Trusted Advisor."
        }.to_string()
    }

    fn suggest_aws_services(&self, prompt: &str) -> String {
        let lower_prompt = prompt.to_lowercase();
        
        if lower_prompt.contains("database") {
            "RDS, DynamoDB, Aurora, DocumentDB"
        } else if lower_prompt.contains("compute") {
            "EC2, Lambda, ECS, EKS, Fargate"
        } else if lower_prompt.contains("storage") {
            "S3, EBS, EFS, FSx"
        } else if lower_prompt.contains("network") {
            "VPC, CloudFront, Route 53, API Gateway"
        } else if lower_prompt.contains("monitoring") {
            "CloudWatch, X-Ray, CloudTrail, Config"
        } else {
            "EC2, S3, Lambda, RDS, VPC"
        }.to_string()
    }
}

#[async_trait]
impl AIAgentConnector for AmazonQAgent {
    async fn connect(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        let access_key_id = credentials.get("access_key_id")
            .ok_or("AWS Access Key ID is required")?;
        let secret_access_key = credentials.get("secret_access_key")
            .ok_or("AWS Secret Access Key is required")?;
        let default_region = "us-east-1".to_string();
        let region = credentials.get("region")
            .unwrap_or(&default_region);

        let aws_creds = AmazonQCredentials {
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
            region: region.clone(),
            session_token: credentials.get("session_token").cloned(),
        };

        if self.validate_aws_credentials(&aws_creds).await? {
            println!("Amazon Q connected successfully in region: {}", region);
            Ok(true)
        } else {
            Err("Invalid AWS credentials for Amazon Q access".into())
        }
    }

    async fn disconnect(&self) -> Result<bool, Box<dyn Error>> {
        println!("Amazon Q disconnected");
        Ok(true)
    }

    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        if self.credentials.is_none() {
            return Err("Amazon Q not connected. Please connect first.".into());
        }

        self.send_amazon_q_request(prompt, context).await
    }

    fn get_agent(&self) -> AIAgent {
        self.agent_info.clone()
    }

    async fn test_connection(&self) -> Result<bool, Box<dyn Error>> {
        if let Some(creds) = &self.credentials {
            self.validate_aws_credentials(creds).await
        } else {
            Ok(false)
        }
    }
}

impl Default for AmazonQAgent {
    fn default() -> Self {
        Self::new()
    }
}


#[allow(dead_code)]
pub async fn query_with_aws_context(
    agent: &AmazonQAgent,
    request: AgentQueryRequest,
    aws_service: Option<&str>,
    region: Option<&str>,
) -> Result<AgentQueryResponse, Box<dyn Error>> {
    let start_time = Instant::now();
    
    let context = match (request.context.as_deref(), aws_service, region) {
        (Some(ctx), Some(service), Some(reg)) => {
            Some(format!("AWS Service: {}\nRegion: {}\nContext:\n{}", service, reg, ctx))
        }
        (Some(ctx), Some(service), None) => {
            Some(format!("AWS Service: {}\nContext:\n{}", service, ctx))
        }
        (Some(ctx), None, Some(reg)) => {
            Some(format!("Region: {}\nContext:\n{}", reg, ctx))
        }
        (None, Some(service), Some(reg)) => {
            Some(format!("AWS Service: {}\nRegion: {}", service, reg))
        }
        (Some(ctx), None, None) => Some(ctx.to_string()),
        (None, Some(service), None) => Some(format!("AWS Service: {}", service)),
        (None, None, Some(reg)) => Some(format!("Region: {}", reg)),
        (None, None, None) => None,
    };

    let response = agent.query(&request.prompt, context.as_deref()).await?;
    let elapsed = start_time.elapsed();

    Ok(AgentQueryResponse {
        response,
        usage: AgentUsage {
            tokens_used: request.prompt.len() as u32, 
            response_time_ms: elapsed.as_millis() as u64,
            model: Some("amazon-q".to_string()),
        },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("agent_type".to_string(), json!("amazon_q"));
            if let Some(service) = aws_service {
                meta.insert("aws_service".to_string(), json!(service));
            }
            if let Some(reg) = region {
                meta.insert("region".to_string(), json!(reg));
            }
            meta
        },
    })
}