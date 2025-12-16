use lazy_static::lazy_static;
use regex::Regex;

use crate::models::{EntityType, RelationshipType};

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
    
    // Import patterns for various languages
    static ref IMPORT_PATTERN: Regex = Regex::new(
        r#"(?m)^(?:use\s+([a-zA-Z_][a-zA-Z0-9_:]*)|import\s+(?:\{[^}]+\}\s+from\s+)?['"]([^'"]+)['"]|from\s+([a-zA-Z_][a-zA-Z0-9_.]*)\s+import|require\s*\(['"]([^'"]+)['"]\))"#
    ).unwrap();
    
    // Function call patterns
    static ref FUNCTION_CALL_PATTERN: Regex = Regex::new(
        r"(?m)(?:self\.)?(\w+)\s*\("
    ).unwrap();
    
    // Impl/extends patterns for class relationships
    static ref IMPL_PATTERN: Regex = Regex::new(
        r"(?m)impl(?:<[^>]+>)?\s+(\w+)\s+for\s+(\w+)|class\s+(\w+)\s+extends\s+(\w+)|(\w+)\s*:\s*(\w+)"
    ).unwrap();
}

#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    pub entity_type: EntityType,
    pub name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct ExtractedRelationship {
    pub from_name: String,
    pub to_name: String,
    pub relationship_type: RelationshipType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractionResult {
    pub entities: Vec<ExtractedEntity>,
    pub relationships: Vec<ExtractedRelationship>,
}

pub struct CodeEntityExtractor;

impl CodeEntityExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract entities from code content
    pub fn extract(&self, content: &str, language: Option<&str>) -> Vec<ExtractedEntity> {
        self.extract_with_relationships(content, language).entities
    }

    /// Extract entities and relationships from code content
    pub fn extract_with_relationships(&self, content: &str, language: Option<&str>) -> ExtractionResult {
        let mut result = ExtractionResult::default();
        let mut function_names: Vec<String> = Vec::new();
        let mut class_names: Vec<String> = Vec::new();

        // Extract functions
        for cap in FUNCTION_PATTERN.captures_iter(content) {
            for i in 2..6 {
                if let Some(name) = cap.get(i) {
                    let fn_name = name.as_str().to_string();
                    function_names.push(fn_name.clone());
                    result.entities.push(ExtractedEntity {
                        entity_type: EntityType::Function,
                        name: fn_name,
                        confidence: 0.9,
                    });
                    break;
                }
            }
        }

        // Extract classes/structs/enums
        for cap in CLASS_PATTERN.captures_iter(content) {
            if let Some(name) = cap.get(2) {
                let class_name = name.as_str().to_string();
                class_names.push(class_name.clone());
                result.entities.push(ExtractedEntity {
                    entity_type: EntityType::Class,
                    name: class_name,
                    confidence: 0.9,
                });
            }
        }

        // Extract API endpoints
        for cap in API_ENDPOINT_PATTERN.captures_iter(content) {
            if let Some(endpoint) = cap.get(1) {
                result.entities.push(ExtractedEntity {
                    entity_type: EntityType::CodeEntity,
                    name: endpoint.as_str().to_string(),
                    confidence: 0.85,
                });
            }
        }

        // Extract ticket references
        for cap in TICKET_PATTERN.captures_iter(content) {
            if let Some(ticket) = cap.get(1) {
                result.entities.push(ExtractedEntity {
                    entity_type: EntityType::Issue,
                    name: ticket.as_str().to_string(),
                    confidence: 0.9,
                });
            }
        }

        // Extract PR references
        for cap in PR_PATTERN.captures_iter(content) {
            if let Some(pr) = cap.get(1) {
                result.entities.push(ExtractedEntity {
                    entity_type: EntityType::PullRequest,
                    name: format!("#{}", pr.as_str()),
                    confidence: 0.9,
                });
            }
        }

        // Extract imports and create IMPORTS relationships
        for cap in IMPORT_PATTERN.captures_iter(content) {
            // Try each capture group for different import syntaxes
            for i in 1..5 {
                if let Some(import_name) = cap.get(i) {
                    let import_str = import_name.as_str().to_string();
                    // Create a module entity for the import
                    result.entities.push(ExtractedEntity {
                        entity_type: EntityType::CodeEntity,
                        name: import_str.clone(),
                        confidence: 0.8,
                    });
                    
                    // If we have a class in this file, it imports this module
                    for class_name in &class_names {
                        result.relationships.push(ExtractedRelationship {
                            from_name: class_name.clone(),
                            to_name: import_str.clone(),
                            relationship_type: RelationshipType::Imports,
                            confidence: 0.85,
                        });
                    }
                    break;
                }
            }
        }

        // Extract impl/extends relationships
        for cap in IMPL_PATTERN.captures_iter(content) {
            // Rust: impl Trait for Struct
            if let (Some(trait_name), Some(struct_name)) = (cap.get(1), cap.get(2)) {
                result.relationships.push(ExtractedRelationship {
                    from_name: struct_name.as_str().to_string(),
                    to_name: trait_name.as_str().to_string(),
                    relationship_type: RelationshipType::Implements,
                    confidence: 0.95,
                });
            }
            // JS/TS/Java: class Child extends Parent
            if let (Some(child), Some(parent)) = (cap.get(3), cap.get(4)) {
                result.relationships.push(ExtractedRelationship {
                    from_name: child.as_str().to_string(),
                    to_name: parent.as_str().to_string(),
                    relationship_type: RelationshipType::Implements,
                    confidence: 0.95,
                });
            }
        }

        // Extract function calls within functions (CALLS relationships)
        // Only track calls to functions we've defined in this file
        let defined_functions: std::collections::HashSet<&str> = 
            function_names.iter().map(|s| s.as_str()).collect();
        
        for cap in FUNCTION_CALL_PATTERN.captures_iter(content) {
            if let Some(called_fn) = cap.get(1) {
                let called_name = called_fn.as_str();
                // Only create relationship if calling a function defined in this file
                if defined_functions.contains(called_name) {
                    // Associate with the first function in the file (simplified)
                    if let Some(caller) = function_names.first() {
                        if caller != called_name {
                            result.relationships.push(ExtractedRelationship {
                                from_name: caller.clone(),
                                to_name: called_name.to_string(),
                                relationship_type: RelationshipType::Calls,
                                confidence: 0.7,
                            });
                        }
                    }
                }
            }
        }

        // Create CONTAINS relationships: classes contain their methods
        // If we have both classes and functions, assume functions belong to the first class
        if !class_names.is_empty() && !function_names.is_empty() {
            let primary_class = &class_names[0];
            for fn_name in &function_names {
                result.relationships.push(ExtractedRelationship {
                    from_name: primary_class.clone(),
                    to_name: fn_name.clone(),
                    relationship_type: RelationshipType::Contains,
                    confidence: 0.8,
                });
            }
        }

        result
    }
}
