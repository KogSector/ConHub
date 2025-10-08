use crate::types::*;
use crate::symbol_extractor::{SymbolDatabase, EnhancedSymbol, SymbolUsage, UsageType, Location};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

pub struct CrossReferenceBuilder {
    symbol_graph: SymbolGraph,
    dependency_cache: HashMap<Uuid, Vec<Dependency>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolGraph {
    pub nodes: HashMap<Uuid, SymbolNode>,
    pub edges: HashMap<Uuid, Vec<SymbolEdge>>,
    pub file_dependencies: HashMap<Uuid, Vec<Uuid>>, // file -> dependent files
    pub project_dependencies: HashMap<Uuid, Vec<Uuid>>, // project -> dependent projects
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolNode {
    pub symbol_id: Uuid,
    pub file_id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub symbol_type: SymbolType,
    pub fully_qualified_name: String,
    pub scope_path: Vec<String>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub complexity: u32,
    pub fan_in: u32,  // Number of incoming dependencies
    pub fan_out: u32, // Number of outgoing dependencies
    pub centrality: f64, // Graph centrality measure
    pub stability: f64,  // Stability metric
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEdge {
    pub id: Uuid,
    pub from_symbol: Uuid,
    pub to_symbol: Uuid,
    pub relationship_type: RelationshipType,
    pub strength: f64, // Relationship strength (0.0 to 1.0)
    pub locations: Vec<Location>,
    pub context: EdgeContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Calls,
    References,
    Inherits,
    Implements,
    Contains,
    Uses,
    Imports,
    Defines,
    Overrides,
    Instantiates,
    Accesses,
    Modifies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeContext {
    pub call_count: u32,
    pub conditional: bool, // Is the relationship conditional?
    pub in_loop: bool,     // Is the relationship in a loop?
    pub access_pattern: AccessPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPattern {
    Read,
    Write,
    ReadWrite,
    Execute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub dependent_id: Uuid,
    pub dependency_id: Uuid,
    pub dependency_type: DependencyType,
    pub strength: f64,
    pub is_circular: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Direct,
    Indirect,
    Transitive,
    Circular,
}

#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    pub total_dependencies: usize,
    pub circular_dependencies: Vec<Vec<Uuid>>,
    pub dependency_layers: Vec<Vec<Uuid>>,
    pub critical_paths: Vec<Vec<Uuid>>,
    pub coupling_metrics: CouplingMetrics,
}

#[derive(Debug, Clone)]
pub struct CouplingMetrics {
    pub afferent_coupling: HashMap<Uuid, u32>, // Ca - incoming dependencies
    pub efferent_coupling: HashMap<Uuid, u32>, // Ce - outgoing dependencies
    pub instability: HashMap<Uuid, f64>,       // I = Ce / (Ca + Ce)
    pub abstractness: HashMap<Uuid, f64>,      // A = abstract classes / total classes
}

impl CrossReferenceBuilder {
    pub fn new() -> Self {
        Self {
            symbol_graph: SymbolGraph::new(),
            dependency_cache: HashMap::new(),
        }
    }

    pub fn build_cross_references(&mut self, databases: Vec<SymbolDatabase>) -> Result<(), Box<dyn std::error::Error>> {
        // Clear existing graph
        self.symbol_graph = SymbolGraph::new();
        
        // Build nodes from all symbol databases
        for database in &databases {
            self.add_symbols_to_graph(database)?;
        }

        // Build edges by analyzing symbol relationships
        for database in &databases {
            self.analyze_symbol_relationships(database)?;
        }

        // Calculate graph metrics
        self.calculate_graph_metrics()?;

        Ok(())
    }

    fn add_symbols_to_graph(&mut self, database: &SymbolDatabase) -> Result<(), Box<dyn std::error::Error>> {
        for (symbol_id, enhanced_symbol) in &database.symbols {
            let node = SymbolNode {
                symbol_id: *symbol_id,
                file_id: enhanced_symbol.base.file_id,
                project_id: Uuid::new_v4(), // Would be provided in real implementation
                name: enhanced_symbol.base.name.clone(),
                symbol_type: enhanced_symbol.base.symbol_type,
                fully_qualified_name: self.build_fully_qualified_name(enhanced_symbol),
                scope_path: self.build_scope_path(enhanced_symbol),
                metadata: NodeMetadata {
                    complexity: enhanced_symbol.complexity,
                    fan_in: 0,  // Will be calculated later
                    fan_out: 0, // Will be calculated later
                    centrality: 0.0,
                    stability: 0.0,
                    tags: enhanced_symbol.modifiers.clone(),
                },
            };

            self.symbol_graph.nodes.insert(*symbol_id, node);
        }

        Ok(())
    }

    fn analyze_symbol_relationships(&mut self, database: &SymbolDatabase) -> Result<(), Box<dyn std::error::Error>> {
        for (symbol_id, enhanced_symbol) in &database.symbols {
            // Analyze function calls
            self.analyze_function_calls(*symbol_id, enhanced_symbol, database)?;
            
            // Analyze type relationships
            self.analyze_type_relationships(*symbol_id, enhanced_symbol, database)?;
            
            // Analyze variable references
            self.analyze_variable_references(*symbol_id, enhanced_symbol, database)?;
            
            // Analyze inheritance relationships
            self.analyze_inheritance(*symbol_id, enhanced_symbol, database)?;
        }

        Ok(())
    }

    fn analyze_function_calls(&mut self, symbol_id: Uuid, symbol: &EnhancedSymbol, database: &SymbolDatabase) -> Result<(), Box<dyn std::error::Error>> {
        if matches!(symbol.base.symbol_type, SymbolType::Function | SymbolType::Method) {
            // Find function calls within this symbol's scope
            for usage in &symbol.usages {
                if matches!(usage.usage_type, UsageType::Call) {
                    // Find the target symbol
                    if let Some(target_symbol) = self.find_symbol_by_location(&usage.location, database) {
                        let edge = SymbolEdge {
                            id: Uuid::new_v4(),
                            from_symbol: symbol_id,
                            to_symbol: target_symbol.base.id,
                            relationship_type: RelationshipType::Calls,
                            strength: 1.0,
                            locations: vec![usage.location.clone()],
                            context: EdgeContext {
                                call_count: 1,
                                conditional: usage.context.contains("if") || usage.context.contains("?"),
                                in_loop: usage.context.contains("for") || usage.context.contains("while"),
                                access_pattern: AccessPattern::Execute,
                            },
                        };

                        self.symbol_graph.edges.entry(symbol_id).or_insert_with(Vec::new).push(edge);
                    }
                }
            }
        }

        Ok(())
    }

    fn analyze_type_relationships(&mut self, symbol_id: Uuid, symbol: &EnhancedSymbol, database: &SymbolDatabase) -> Result<(), Box<dyn std::error::Error>> {
        match symbol.base.symbol_type {
            SymbolType::Class | SymbolType::Struct => {
                // Analyze inheritance and composition
                if let Some(ref signature) = symbol.base.signature {
                    // Look for inheritance keywords
                    if signature.contains("extends") || signature.contains(":") {
                        // Parse inheritance relationships
                        self.parse_inheritance_from_signature(symbol_id, signature, database)?;
                    }
                }
            }
            SymbolType::Variable | SymbolType::Field => {
                // Analyze type usage
                if let Some(ref signature) = symbol.base.signature {
                    self.parse_type_usage_from_signature(symbol_id, signature, database)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn analyze_variable_references(&mut self, symbol_id: Uuid, symbol: &EnhancedSymbol, _database: &SymbolDatabase) -> Result<(), Box<dyn std::error::Error>> {
        for usage in &symbol.usages {
            match usage.usage_type {
                UsageType::Reference | UsageType::Assignment => {
                    // Create reference edge
                    // Implementation would find the referenced symbol and create edge
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn analyze_inheritance(&mut self, symbol_id: Uuid, symbol: &EnhancedSymbol, database: &SymbolDatabase) -> Result<(), Box<dyn std::error::Error>> {
        if matches!(symbol.base.symbol_type, SymbolType::Class | SymbolType::Interface) {
            // Look for parent classes/interfaces in the same database
            for (other_id, other_symbol) in &database.symbols {
                if *other_id != symbol_id && self.is_inheritance_relationship(symbol, other_symbol) {
                    let edge = SymbolEdge {
                        id: Uuid::new_v4(),
                        from_symbol: symbol_id,
                        to_symbol: *other_id,
                        relationship_type: if matches!(other_symbol.base.symbol_type, SymbolType::Interface) {
                            RelationshipType::Implements
                        } else {
                            RelationshipType::Inherits
                        },
                        strength: 1.0,
                        locations: vec![Location {
                            file_id: symbol.base.file_id,
                            line: symbol.base.line,
                            column: symbol.base.column,
                        }],
                        context: EdgeContext {
                            call_count: 1,
                            conditional: false,
                            in_loop: false,
                            access_pattern: AccessPattern::Read,
                        },
                    };

                    self.symbol_graph.edges.entry(symbol_id).or_insert_with(Vec::new).push(edge);
                }
            }
        }

        Ok(())
    }

    fn calculate_graph_metrics(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate fan-in and fan-out for each node
        for (node_id, _) in &self.symbol_graph.nodes {
            let fan_out = self.symbol_graph.edges.get(node_id).map_or(0, |edges| edges.len()) as u32;
            let fan_in = self.calculate_fan_in(*node_id);

            if let Some(node) = self.symbol_graph.nodes.get_mut(node_id) {
                node.metadata.fan_in = fan_in;
                node.metadata.fan_out = fan_out;
            }
        }

        // Calculate centrality measures
        self.calculate_centrality_measures()?;

        Ok(())
    }

    fn calculate_fan_in(&self, target_id: Uuid) -> u32 {
        let mut count = 0;
        for edges in self.symbol_graph.edges.values() {
            count += edges.iter().filter(|edge| edge.to_symbol == target_id).count();
        }
        count as u32
    }

    fn calculate_centrality_measures(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simple degree centrality calculation
        let total_nodes = self.symbol_graph.nodes.len() as f64;
        
        for (node_id, node) in &mut self.symbol_graph.nodes {
            let degree = (node.metadata.fan_in + node.metadata.fan_out) as f64;
            node.metadata.centrality = if total_nodes > 1.0 {
                degree / (total_nodes - 1.0)
            } else {
                0.0
            };
        }

        Ok(())
    }

    pub fn analyze_dependencies(&self) -> DependencyAnalysis {
        let mut analysis = DependencyAnalysis {
            total_dependencies: 0,
            circular_dependencies: Vec::new(),
            dependency_layers: Vec::new(),
            critical_paths: Vec::new(),
            coupling_metrics: CouplingMetrics {
                afferent_coupling: HashMap::new(),
                efferent_coupling: HashMap::new(),
                instability: HashMap::new(),
                abstractness: HashMap::new(),
            },
        };

        // Calculate coupling metrics
        for (node_id, node) in &self.symbol_graph.nodes {
            analysis.coupling_metrics.afferent_coupling.insert(*node_id, node.metadata.fan_in);
            analysis.coupling_metrics.efferent_coupling.insert(*node_id, node.metadata.fan_out);
            
            let instability = if node.metadata.fan_in + node.metadata.fan_out > 0 {
                node.metadata.fan_out as f64 / (node.metadata.fan_in + node.metadata.fan_out) as f64
            } else {
                0.0
            };
            analysis.coupling_metrics.instability.insert(*node_id, instability);
        }

        // Detect circular dependencies
        analysis.circular_dependencies = self.detect_circular_dependencies();

        // Calculate dependency layers
        analysis.dependency_layers = self.calculate_dependency_layers();

        analysis.total_dependencies = self.symbol_graph.edges.values().map(|edges| edges.len()).sum();

        analysis
    }

    fn detect_circular_dependencies(&self) -> Vec<Vec<Uuid>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();

        for node_id in self.symbol_graph.nodes.keys() {
            if !visited.contains(node_id) {
                if let Some(cycle) = self.dfs_cycle_detection(*node_id, &mut visited, &mut rec_stack, &mut Vec::new()) {
                    cycles.push(cycle);
                }
            }
        }

        cycles
    }

    fn dfs_cycle_detection(&self, node: Uuid, visited: &mut HashSet<Uuid>, rec_stack: &mut HashSet<Uuid>, path: &mut Vec<Uuid>) -> Option<Vec<Uuid>> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(edges) = self.symbol_graph.edges.get(&node) {
            for edge in edges {
                if !visited.contains(&edge.to_symbol) {
                    if let Some(cycle) = self.dfs_cycle_detection(edge.to_symbol, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&edge.to_symbol) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|&x| x == edge.to_symbol).unwrap();
                    return Some(path[cycle_start..].to_vec());
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
        None
    }

    fn calculate_dependency_layers(&self) -> Vec<Vec<Uuid>> {
        let mut layers = Vec::new();
        let mut remaining_nodes: HashSet<Uuid> = self.symbol_graph.nodes.keys().cloned().collect();

        while !remaining_nodes.is_empty() {
            let mut current_layer = Vec::new();
            
            // Find nodes with no incoming dependencies from remaining nodes
            for &node_id in &remaining_nodes {
                let has_incoming = self.symbol_graph.edges.values()
                    .flat_map(|edges| edges.iter())
                    .any(|edge| edge.to_symbol == node_id && remaining_nodes.contains(&edge.from_symbol));
                
                if !has_incoming {
                    current_layer.push(node_id);
                }
            }

            if current_layer.is_empty() {
                // Circular dependency - add all remaining nodes
                current_layer.extend(remaining_nodes.iter());
            }

            for &node_id in &current_layer {
                remaining_nodes.remove(&node_id);
            }

            layers.push(current_layer);
        }

        layers
    }

    // Helper methods
    fn build_fully_qualified_name(&self, symbol: &EnhancedSymbol) -> String {
        if let Some(ref scope) = symbol.base.scope {
            format!("{}::{}", scope, symbol.base.name)
        } else {
            symbol.base.name.clone()
        }
    }

    fn build_scope_path(&self, symbol: &EnhancedSymbol) -> Vec<String> {
        if let Some(ref scope) = symbol.base.scope {
            scope.split("::").map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        }
    }

    fn find_symbol_by_location(&self, _location: &Location, _database: &SymbolDatabase) -> Option<&EnhancedSymbol> {
        // Implementation would find symbol at given location
        None
    }

    fn parse_inheritance_from_signature(&mut self, _symbol_id: Uuid, _signature: &str, _database: &SymbolDatabase) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation would parse inheritance relationships from signature
        Ok(())
    }

    fn parse_type_usage_from_signature(&mut self, _symbol_id: Uuid, _signature: &str, _database: &SymbolDatabase) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation would parse type usage from signature
        Ok(())
    }

    fn is_inheritance_relationship(&self, _child: &EnhancedSymbol, _parent: &EnhancedSymbol) -> bool {
        // Implementation would determine if there's an inheritance relationship
        false
    }

    pub fn get_symbol_dependencies(&self, symbol_id: Uuid) -> Vec<&SymbolEdge> {
        self.symbol_graph.edges.get(&symbol_id).map_or(Vec::new(), |edges| edges.iter().collect())
    }

    pub fn get_symbol_dependents(&self, symbol_id: Uuid) -> Vec<&SymbolEdge> {
        self.symbol_graph.edges.values()
            .flat_map(|edges| edges.iter())
            .filter(|edge| edge.to_symbol == symbol_id)
            .collect()
    }

    pub fn export_graph_for_database(&self) -> SymbolGraph {
        self.symbol_graph.clone()
    }
}

impl SymbolGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            file_dependencies: HashMap::new(),
            project_dependencies: HashMap::new(),
        }
    }

    pub fn get_node(&self, id: Uuid) -> Option<&SymbolNode> {
        self.nodes.get(&id)
    }

    pub fn get_edges_from(&self, id: Uuid) -> Option<&Vec<SymbolEdge>> {
        self.edges.get(&id)
    }
}