use lazy_static::lazy_static;
use regex::Regex;

use crate::models::EntityType;

lazy_static! {
    static ref FUNCTION_PATTERN: Regex = Regex::new(
        r"(?m)^[\t ]*(pub\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)|function\s+(\w+)|def\s+(\w+)|func\s+(\w+)"
    ).unwrap();
    
    static ref CLASS_PATTERN: Regex = Regex::new(
        r"(?m)^[\t ]*(pub\s+)?(?:class|struct|enum|trait|interface)\s+(\w+)"
    ).unwrap();
    
    static ref API_ENDPOINT_PATTERN: Regex = Regex::new(
        r"(?:GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s+(/[a-zA-Z0-9_/\-{}:]*)"
    ).unwrap();
    
    static ref TICKET_PATTERN: Regex = Regex::new(
        r"([A-Z]{2,10}-\d+)"
    ).unwrap();
    
    static ref PR_PATTERN: Regex = Regex::new(
        r"(?:PR|MR|#)(\d+)"
    ).unwrap();
}

#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    pub entity_type: EntityType,
    pub name: String,
    pub confidence: f32,
}

pub struct CodeEntityExtractor;

impl CodeEntityExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract entities from code content
    pub fn extract(&self, content: &str, language: Option<&str>) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();

        // Extract functions
        for cap in FUNCTION_PATTERN.captures_iter(content) {
            // Try each capture group (different patterns have different groups)
            for i in 2..6 {
                if let Some(name) = cap.get(i) {
                    entities.push(ExtractedEntity {
                        entity_type: EntityType::Function,
                        name: name.as_str().to_string(),
                        confidence: 0.9,
                    });
                    break;
                }
            }
        }

        // Extract classes/structs/enums
        for cap in CLASS_PATTERN.captures_iter(content) {
            if let Some(name) = cap.get(2) {
                entities.push(ExtractedEntity {
                    entity_type: EntityType::Class,
                    name: name.as_str().to_string(),
                    confidence: 0.9,
                });
            }
        }

        // Extract API endpoints
        for cap in API_ENDPOINT_PATTERN.captures_iter(content) {
            if let Some(endpoint) = cap.get(1) {
                entities.push(ExtractedEntity {
                    entity_type: EntityType::CodeEntity,
                    name: endpoint.as_str().to_string(),
                    confidence: 0.85,
                });
            }
        }

        // Extract ticket references
        for cap in TICKET_PATTERN.captures_iter(content) {
            if let Some(ticket) = cap.get(1) {
                entities.push(ExtractedEntity {
                    entity_type: EntityType::Issue,
                    name: ticket.as_str().to_string(),
                    confidence: 0.9,
                });
            }
        }

        // Extract PR references
        for cap in PR_PATTERN.captures_iter(content) {
            if let Some(pr) = cap.get(1) {
                entities.push(ExtractedEntity {
                    entity_type: EntityType::PullRequest,
                    name: format!("#{}", pr.as_str()),
                    confidence: 0.9,
                });
            }
        }

        entities
    }
}
