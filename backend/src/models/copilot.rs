use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use crate::models::mcp::McpContext; // Disabled

/// GitHub Copilot specific integration models
/// 
/// These models extend the core MCP types to provide GitHub Copilot
/// with optimized context delivery and tool interaction capabilities.

// ============================================================================
// Copilot-Enhanced Context Types
// ============================================================================

/// Enhanced context specifically formatted for GitHub Copilot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotEnhancedContext {
    // pub base_context: McpContext, // Disabled
    pub copilot_metadata: CopilotContextMetadata,
    pub code_analysis: Option<CodeAnalysisData>,
    pub suggestions: Vec<CopilotSuggestion>,
    pub workspace_info: Option<WorkspaceInfo>,
}

/// Metadata specific to Copilot context formatting
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

/// Code analysis data for enhanced Copilot suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisData {
    pub function_signatures: Vec<FunctionSignature>,
    pub type_definitions: Vec<TypeDefinition>,
    pub import_statements: Vec<ImportStatement>,
    pub dependencies: Vec<DependencyInfo>,
    pub patterns: Vec<CodePattern>,
}

/// Function signature information
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

/// Parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

/// Type definition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    pub name: String,
    pub kind: String, // struct, enum, trait, interface, class, etc.
    pub fields: Vec<Field>,
    pub methods: Vec<String>,
    pub file_path: String,
    pub line_number: u32,
}

/// Field information for types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub visibility: String,
    pub is_optional: bool,
}

/// Import/use statement information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatement {
    pub module_path: String,
    pub imported_items: Vec<String>,
    pub alias: Option<String>,
    pub file_path: String,
    pub line_number: u32,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub dependency_type: String, // dev, runtime, build, etc.
    pub source: String, // crates.io, npm, maven, etc.
}

/// Code pattern recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub pattern_type: String,
    pub description: String,
    pub examples: Vec<String>,
    pub confidence: f64,
}

/// Copilot-specific suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotSuggestion {
    pub suggestion_type: String,
    pub title: String,
    pub description: String,
    pub code_snippet: Option<String>,
    pub confidence: f64,
    pub applicable_context: Vec<String>,
}

/// Workspace information for Copilot
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

// ============================================================================
// Copilot Tool Integration
// ============================================================================

/// Tools specifically designed for GitHub Copilot integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_format: String,
    pub capabilities: Vec<String>,
    pub performance_hints: CopilotToolPerformance,
}

/// Performance hints for Copilot tool usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotToolPerformance {
    pub average_execution_time_ms: u64,
    pub max_input_size: usize,
    pub supports_streaming: bool,
    pub cache_duration_seconds: u32,
    pub rate_limit_per_minute: u32,
}

// ============================================================================
// Copilot Request/Response Extensions
// ============================================================================

/// Extended context request with Copilot-specific options
#[derive(Debug, Serialize, Deserialize)]
pub struct CopilotContextRequestExtended {
    pub base_request: crate::services::github_copilot_integration::CopilotContextRequest,
    pub analysis_depth: AnalysisDepth,
    pub include_suggestions: bool,
    pub suggestion_types: Vec<String>,
    pub performance_mode: PerformanceMode,
    pub streaming_preferences: StreamingPreferences,
}

/// Analysis depth for context requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Surface,    // Basic file content and structure
    Moderate,   // Include function signatures and imports
    Deep,       // Full analysis with patterns and dependencies
    Comprehensive, // Everything including cross-references
}

/// Performance mode preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMode {
    Optimized,  // Prioritize speed
    Balanced,   // Balance speed and completeness  
    Complete,   // Prioritize completeness
    Custom(PerformanceConfig),
}

/// Custom performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_response_time_ms: u64,
    pub max_context_size_kb: usize,
    pub enable_caching: bool,
    pub parallel_processing: bool,
}

/// Streaming preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingPreferences {
    pub enabled: bool,
    pub chunk_size: usize,
    pub include_partial_results: bool,
    pub compression: bool,
}

// ============================================================================
// Copilot Integration Metrics
// ============================================================================

/// Metrics for monitoring Copilot integration performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotIntegrationMetrics {
    pub session_metrics: SessionMetrics,
    pub context_metrics: ContextMetrics,
    pub tool_metrics: ToolMetrics,
    pub performance_metrics: PerformanceMetrics,
}

/// Session-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub total_sessions: u64,
    pub active_sessions: u32,
    pub average_session_duration_minutes: f64,
    pub sessions_per_user: HashMap<String, u32>,
    pub session_creation_rate_per_hour: f64,
}

/// Context-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetrics {
    pub context_requests_total: u64,
    pub context_requests_per_hour: f64,
    pub average_context_size_kb: f64,
    pub context_types_requested: HashMap<String, u64>,
    pub cache_hit_rate: f64,
}

/// Tool usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    pub tool_calls_total: u64,
    pub tool_calls_per_hour: f64,
    pub tools_usage_distribution: HashMap<String, u64>,
    pub average_tool_execution_time_ms: HashMap<String, f64>,
    pub tool_success_rate: HashMap<String, f64>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub throughput_requests_per_second: f64,
    pub resource_utilization: ResourceUtilization,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub network_throughput_mbps: f64,
    pub storage_usage_mb: f64,
}