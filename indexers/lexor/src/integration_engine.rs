use crate::symbol_extractor::{SymbolExtractor, SymbolDatabase};
use crate::cross_reference_builder::{CrossReferenceBuilder, SymbolGraph, DependencyAnalysis};
use crate::types::*;
use std::path::Path;
use std::collections::HashMap;
use uuid::Uuid;

/// Integration engine that orchestrates symbol extraction and cross-reference building
pub struct SymbolAnalysisEngine {
    extractor: SymbolExtractor,
    cross_ref_builder: CrossReferenceBuilder,
    project_databases: HashMap<Uuid, SymbolDatabase>,
    global_graph: Option<SymbolGraph>,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub symbol_database: SymbolDatabase,
    pub dependency_analysis: DependencyAnalysis,
    pub symbol_graph: SymbolGraph,
    pub metrics: AnalysisMetrics,
}

#[derive(Debug, Clone)]
pub struct AnalysisMetrics {
    pub total_symbols: usize,
    pub total_files: usize,
    pub total_relationships: usize,
    pub complexity_distribution: HashMap<String, u32>,
    pub language_distribution: HashMap<Language, usize>,
    pub processing_time_ms: u64,
}

impl SymbolAnalysisEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            extractor: SymbolExtractor::new()?,
            cross_ref_builder: CrossReferenceBuilder::new(),
            project_databases: HashMap::new(),
            global_graph: None,
        })
    }

    /// Analyze a single file and return comprehensive symbol information
    pub fn analyze_file(&mut self, file_id: Uuid, path: &Path, content: &str) -> Result<AnalysisResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // Extract symbols and build AST
        let symbol_database = self.extractor.extract_comprehensive_symbols(file_id, path, content)?;

        // Build cross-references (single file scope)
        let mut databases = vec![symbol_database.clone()];
        self.cross_ref_builder.build_cross_references(databases)?;

        // Get analysis results
        let dependency_analysis = self.cross_ref_builder.analyze_dependencies();
        let symbol_graph = self.cross_ref_builder.export_graph_for_database();

        let processing_time = start_time.elapsed().as_millis() as u64;

        let metrics = AnalysisMetrics {
            total_symbols: symbol_database.symbols.len(),
            total_files: 1,
            total_relationships: symbol_graph.edges.values().map(|edges| edges.len()).sum(),
            complexity_distribution: self.calculate_complexity_distribution(&symbol_database),
            language_distribution: self.calculate_language_distribution(path),
            processing_time_ms: processing_time,
        };

        Ok(AnalysisResult {
            symbol_database,
            dependency_analysis,
            symbol_graph,
            metrics,
        })
    }

    /// Analyze multiple files in a project
    pub fn analyze_project(&mut self, project_id: Uuid, files: Vec<(Uuid, &Path, &str)>) -> Result<AnalysisResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut all_databases = Vec::new();
        let mut total_symbols = 0;
        let mut complexity_dist = HashMap::new();
        let mut language_dist = HashMap::new();

        // Process each file
        for (file_id, path, content) in files {
            let database = self.extractor.extract_comprehensive_symbols(file_id, path, content)?;
            total_symbols += database.symbols.len();
            
            // Update distributions
            self.merge_complexity_distribution(&database, &mut complexity_dist);
            self.merge_language_distribution(path, &mut language_dist);
            
            all_databases.push(database);
        }

        // Build global cross-references
        self.cross_ref_builder.build_cross_references(all_databases.clone())?;

        // Merge all databases into one project database
        let project_database = self.merge_databases(all_databases)?;
        
        // Store project database
        self.project_databases.insert(project_id, project_database.clone());

        // Get analysis results
        let dependency_analysis = self.cross_ref_builder.analyze_dependencies();
        let symbol_graph = self.cross_ref_builder.export_graph_for_database();
        self.global_graph = Some(symbol_graph.clone());

        let processing_time = start_time.elapsed().as_millis() as u64;

        let metrics = AnalysisMetrics {
            total_symbols,
            total_files: self.project_databases.get(&project_id).map_or(0, |db| db.file_asts.len()),
            total_relationships: symbol_graph.edges.values().map(|edges| edges.len()).sum(),
            complexity_distribution: complexity_dist,
            language_distribution: language_dist,
            processing_time_ms: processing_time,
        };

        Ok(AnalysisResult {
            symbol_database: project_database,
            dependency_analysis,
            symbol_graph,
            metrics,
        })
    }

    /// Get cross-project analysis for AI agents
    pub fn get_cross_project_analysis(&self, project_ids: Vec<Uuid>) -> Result<CrossProjectAnalysis, Box<dyn std::error::Error>> {
        let mut analysis = CrossProjectAnalysis {
            project_relationships: HashMap::new(),
            shared_symbols: Vec::new(),
            dependency_conflicts: Vec::new(),
            integration_points: Vec::new(),
        };

        // Analyze relationships between projects
        for i in 0..project_ids.len() {
            for j in i+1..project_ids.len() {
                let project_a = project_ids[i];
                let project_b = project_ids[j];
                
                if let (Some(db_a), Some(db_b)) = (
                    self.project_databases.get(&project_a),
                    self.project_databases.get(&project_b)
                ) {
                    let relationship = self.analyze_project_relationship(db_a, db_b)?;
                    analysis.project_relationships.insert((project_a, project_b), relationship);
                }
            }
        }

        Ok(analysis)
    }

    /// Export data for database storage (Neo4j/relational)
    pub fn export_for_database(&self) -> DatabaseExport {
        DatabaseExport {
            symbol_nodes: self.extract_symbol_nodes(),
            relationship_edges: self.extract_relationship_edges(),
            file_metadata: self.extract_file_metadata(),
            project_metadata: self.extract_project_metadata(),
            dependency_graph: self.global_graph.clone(),
        }
    }

    // Helper methods
    fn calculate_complexity_distribution(&self, database: &SymbolDatabase) -> HashMap<String, u32> {
        let mut distribution = HashMap::new();
        
        for symbol in database.symbols.values() {
            let complexity_range = match symbol.complexity {
                0..=5 => "Low",
                6..=10 => "Medium", 
                11..=20 => "High",
                _ => "Very High",
            };
            *distribution.entry(complexity_range.to_string()).or_insert(0) += 1;
        }
        
        distribution
    }

    fn calculate_language_distribution(&self, path: &Path) -> HashMap<Language, usize> {
        let mut distribution = HashMap::new();
        let language = Language::from_extension(
            path.extension().and_then(|e| e.to_str()).unwrap_or("")
        );
        *distribution.entry(language).or_insert(0) += 1;
        distribution
    }

    fn merge_complexity_distribution(&self, database: &SymbolDatabase, target: &mut HashMap<String, u32>) {
        let dist = self.calculate_complexity_distribution(database);
        for (key, value) in dist {
            *target.entry(key).or_insert(0) += value;
        }
    }

    fn merge_language_distribution(&self, path: &Path, target: &mut HashMap<Language, usize>) {
        let dist = self.calculate_language_distribution(path);
        for (key, value) in dist {
            *target.entry(key).or_insert(0) += value;
        }
    }

    fn merge_databases(&self, databases: Vec<SymbolDatabase>) -> Result<SymbolDatabase, Box<dyn std::error::Error>> {
        let mut merged = SymbolDatabase {
            symbols: HashMap::new(),
            ast_nodes: HashMap::new(),
            file_asts: HashMap::new(),
            symbol_hierarchy: HashMap::new(),
        };

        for database in databases {
            merged.symbols.extend(database.symbols);
            merged.ast_nodes.extend(database.ast_nodes);
            merged.file_asts.extend(database.file_asts);
            merged.symbol_hierarchy.extend(database.symbol_hierarchy);
        }

        Ok(merged)
    }

    fn analyze_project_relationship(&self, _db_a: &SymbolDatabase, _db_b: &SymbolDatabase) -> Result<ProjectRelationship, Box<dyn std::error::Error>> {
        // Implementation would analyze shared symbols, dependencies, etc.
        Ok(ProjectRelationship {
            shared_symbol_count: 0,
            dependency_strength: 0.0,
            integration_complexity: 0,
        })
    }

    fn extract_symbol_nodes(&self) -> Vec<DatabaseSymbolNode> {
        // Implementation would extract nodes for database storage
        Vec::new()
    }

    fn extract_relationship_edges(&self) -> Vec<DatabaseRelationshipEdge> {
        // Implementation would extract edges for database storage
        Vec::new()
    }

    fn extract_file_metadata(&self) -> Vec<DatabaseFileMetadata> {
        // Implementation would extract file metadata
        Vec::new()
    }

    fn extract_project_metadata(&self) -> Vec<DatabaseProjectMetadata> {
        // Implementation would extract project metadata
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct CrossProjectAnalysis {
    pub project_relationships: HashMap<(Uuid, Uuid), ProjectRelationship>,
    pub shared_symbols: Vec<SharedSymbol>,
    pub dependency_conflicts: Vec<DependencyConflict>,
    pub integration_points: Vec<IntegrationPoint>,
}

#[derive(Debug, Clone)]
pub struct ProjectRelationship {
    pub shared_symbol_count: usize,
    pub dependency_strength: f64,
    pub integration_complexity: u32,
}

#[derive(Debug, Clone)]
pub struct SharedSymbol {
    pub symbol_name: String,
    pub projects: Vec<Uuid>,
    pub conflict_potential: f64,
}

#[derive(Debug, Clone)]
pub struct DependencyConflict {
    pub symbol_name: String,
    pub conflicting_projects: Vec<Uuid>,
    pub severity: ConflictSeverity,
}

#[derive(Debug, Clone)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct IntegrationPoint {
    pub symbol_id: Uuid,
    pub projects: Vec<Uuid>,
    pub integration_type: IntegrationType,
}

#[derive(Debug, Clone)]
pub enum IntegrationType {
    API,
    SharedLibrary,
    DataStructure,
    Interface,
}

#[derive(Debug, Clone)]
pub struct DatabaseExport {
    pub symbol_nodes: Vec<DatabaseSymbolNode>,
    pub relationship_edges: Vec<DatabaseRelationshipEdge>,
    pub file_metadata: Vec<DatabaseFileMetadata>,
    pub project_metadata: Vec<DatabaseProjectMetadata>,
    pub dependency_graph: Option<SymbolGraph>,
}

#[derive(Debug, Clone)]
pub struct DatabaseSymbolNode {
    pub id: Uuid,
    pub name: String,
    pub symbol_type: String,
    pub file_path: String,
    pub line_number: u32,
    pub complexity: u32,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DatabaseRelationshipEdge {
    pub id: Uuid,
    pub from_symbol: Uuid,
    pub to_symbol: Uuid,
    pub relationship_type: String,
    pub strength: f64,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DatabaseFileMetadata {
    pub id: Uuid,
    pub path: String,
    pub language: String,
    pub symbol_count: usize,
    pub complexity_score: f64,
}

#[derive(Debug, Clone)]
pub struct DatabaseProjectMetadata {
    pub id: Uuid,
    pub name: String,
    pub file_count: usize,
    pub total_symbols: usize,
    pub languages: Vec<String>,
}