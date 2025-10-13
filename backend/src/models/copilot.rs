use serde::{Deserialize, Serialize};
use std::collections::HashMap;












#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotEnhancedContext {
    
    pub copilot_metadata: CopilotContextMetadata,
    pub code_analysis: Option<CodeAnalysisData>,
    pub suggestions: Vec<CopilotSuggestion>,
    pub workspace_info: Option<WorkspaceInfo>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotContextMetadata {
    pub total_files: usize,
    pub total_lines: usize,
    pub language_distribution: HashMap<String, usize>,
    pub context_quality_score: f64,
    pub relevance_threshold: f64,
    pub supports_incremental_updates: bool,
    pub cache_strategy: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisData {
    pub function_signatures: Vec<FunctionSignature>,
    pub type_definitions: Vec<TypeDefinition>,
    pub import_statements: Vec<ImportStatement>,
    pub dependencies: Vec<DependencyInfo>,
    pub patterns: Vec<CodePattern>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub visibility: String,
    pub documentation: Option<String>,
    pub file_path: String,
    pub line_number: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    pub name: String,
    pub kind: String, 
    pub fields: Vec<Field>,
    pub methods: Vec<String>,
    pub file_path: String,
    pub line_number: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub visibility: String,
    pub is_optional: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatement {
    pub module_path: String,
    pub imported_items: Vec<String>,
    pub alias: Option<String>,
    pub file_path: String,
    pub line_number: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub dependency_type: String, 
    pub source: String, 
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub pattern_type: String,
    pub description: String,
    pub examples: Vec<String>,
    pub confidence: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotSuggestion {
    pub suggestion_type: String,
    pub title: String,
    pub description: String,
    pub code_snippet: Option<String>,
    pub confidence: f64,
    pub applicable_context: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub name: String,
    pub root_path: String,
    pub language: String,
    pub framework: Option<String>,
    pub build_system: Option<String>,
    pub project_type: String,
    pub configuration_files: Vec<String>,
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_format: String,
    pub capabilities: Vec<String>,
    pub performance_hints: CopilotToolPerformance,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotToolPerformance {
    pub average_execution_time_ms: u64,
    pub max_input_size: usize,
    pub supports_streaming: bool,
    pub cache_duration_seconds: u32,
    pub rate_limit_per_minute: u32,
}






#[derive(Debug, Serialize, Deserialize)]
pub struct CopilotContextRequestExtended {
    pub base_request: crate::services::github_copilot_integration::CopilotContextRequest,
    pub analysis_depth: AnalysisDepth,
    pub include_suggestions: bool,
    pub suggestion_types: Vec<String>,
    pub performance_mode: PerformanceMode,
    pub streaming_preferences: StreamingPreferences,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Surface,    
    Moderate,   
    Deep,       
    Comprehensive, 
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMode {
    Optimized,  
    Balanced,   
    Complete,   
    Custom(PerformanceConfig),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_response_time_ms: u64,
    pub max_context_size_kb: usize,
    pub enable_caching: bool,
    pub parallel_processing: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingPreferences {
    pub enabled: bool,
    pub chunk_size: usize,
    pub include_partial_results: bool,
    pub compression: bool,
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotIntegrationMetrics {
    pub session_metrics: SessionMetrics,
    pub context_metrics: ContextMetrics,
    pub tool_metrics: ToolMetrics,
    pub performance_metrics: PerformanceMetrics,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub total_sessions: u64,
    pub active_sessions: u32,
    pub average_session_duration_minutes: f64,
    pub sessions_per_user: HashMap<String, u32>,
    pub session_creation_rate_per_hour: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetrics {
    pub context_requests_total: u64,
    pub context_requests_per_hour: f64,
    pub average_context_size_kb: f64,
    pub context_types_requested: HashMap<String, u64>,
    pub cache_hit_rate: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    pub tool_calls_total: u64,
    pub tool_calls_per_hour: f64,
    pub tools_usage_distribution: HashMap<String, u64>,
    pub average_tool_execution_time_ms: HashMap<String, f64>,
    pub tool_success_rate: HashMap<String, f64>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub throughput_requests_per_second: f64,
    pub resource_utilization: ResourceUtilization,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub network_throughput_mbps: f64,
    pub storage_usage_mb: f64,
}