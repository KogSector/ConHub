use crate::types::*;
use crate::ctags_integration::CtagsIntegration;
use crate::language_parser::{UniversalLanguageParser, ParseResult};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;
use tree_sitter::{Tree, Node, TreeCursor};

pub struct SymbolExtractor {
    ctags: Option<CtagsIntegration>,
    tree_sitter: UniversalLanguageParser,
    ast_cache: HashMap<Uuid, ASTNode>,
}

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub id: Uuid,
    pub node_type: String,
    pub name: Option<String>,
    pub start_line: u32,
    pub end_line: u32,
    pub children: Vec<Uuid>,
    pub parent: Option<Uuid>,
    pub scope_depth: u32,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SymbolDatabase {
    pub symbols: HashMap<Uuid, EnhancedSymbol>,
    pub ast_nodes: HashMap<Uuid, ASTNode>,
    pub file_asts: HashMap<Uuid, Uuid>, // file_id -> root_ast_node_id
    pub symbol_hierarchy: HashMap<Uuid, Vec<Uuid>>, // parent -> children
}

#[derive(Debug, Clone)]
pub struct EnhancedSymbol {
    pub base: Symbol,
    pub ast_node_id: Option<Uuid>,
    pub complexity: u32,
    pub dependencies: Vec<Uuid>,
    pub usages: Vec<SymbolUsage>,
    pub documentation: Option<String>,
    pub visibility: Visibility,
    pub modifiers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SymbolUsage {
    pub location: Location,
    pub usage_type: UsageType,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub file_id: Uuid,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
pub enum UsageType {
    Definition,
    Declaration,
    Call,
    Reference,
    Assignment,
    Import,
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Package,
}

impl SymbolExtractor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ctags = CtagsIntegration::new().ok();
        let tree_sitter = UniversalLanguageParser::new()?;
        
        Ok(Self {
            ctags,
            tree_sitter,
            ast_cache: HashMap::new(),
        })
    }

    pub fn extract_comprehensive_symbols(&mut self, file_id: Uuid, path: &Path, content: &str) -> Result<SymbolDatabase, Box<dyn std::error::Error>> {
        let language = self.detect_language(path)?;
        
        // Extract symbols using both ctags and tree-sitter
        let tree_sitter_result = self.tree_sitter.parse_file(file_id, path, content)?;
        let ctags_symbols = if let Some(ref ctags) = self.ctags {
            ctags.parse_file(file_id, path)?
        } else {
            Vec::new()
        };

        // Generate AST
        let ast_root = self.generate_ast(file_id, content, &language)?;
        
        // Combine and enhance symbols
        let enhanced_symbols = self.combine_and_enhance_symbols(
            tree_sitter_result.symbols,
            ctags_symbols,
            &ast_root,
            content,
        )?;

        // Build symbol hierarchy
        let symbol_hierarchy = self.build_symbol_hierarchy(&enhanced_symbols);

        let mut symbol_database = SymbolDatabase {
            symbols: enhanced_symbols.into_iter().map(|s| (s.base.id, s)).collect(),
            ast_nodes: HashMap::new(),
            file_asts: HashMap::new(),
            symbol_hierarchy,
        };

        // Add AST to database
        self.add_ast_to_database(&mut symbol_database, file_id, ast_root);

        Ok(symbol_database)
    }

    fn generate_ast(&mut self, file_id: Uuid, content: &str, language: &Language) -> Result<ASTNode, Box<dyn std::error::Error>> {
        if let Some(ts_lang) = language.tree_sitter_language() {
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&ts_lang)?;
            
            if let Some(tree) = parser.parse(content, None) {
                return Ok(self.tree_to_ast(tree.root_node(), content, None, 0));
            }
        }

        // Fallback to simple AST generation
        self.generate_simple_ast(file_id, content, language)
    }

    fn tree_to_ast(&self, node: Node, content: &str, parent: Option<Uuid>, depth: u32) -> ASTNode {
        let id = Uuid::new_v4();
        let name = if node.is_named() {
            node.utf8_text(content.as_bytes()).ok().map(|s| s.to_string())
        } else {
            None
        };

        let mut children = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.is_named() {
                let child_ast = self.tree_to_ast(child, content, Some(id), depth + 1);
                children.push(child_ast.id);
            }
        }

        let mut attributes = HashMap::new();
        if let Some(field_name) = node.kind_id().to_string().as_str() {
            attributes.insert("kind".to_string(), field_name.to_string());
        }

        ASTNode {
            id,
            node_type: node.kind().to_string(),
            name,
            start_line: node.start_position().row as u32 + 1,
            end_line: node.end_position().row as u32 + 1,
            children,
            parent,
            scope_depth: depth,
            attributes,
        }
    }

    fn generate_simple_ast(&self, file_id: Uuid, content: &str, _language: &Language) -> Result<ASTNode, Box<dyn std::error::Error>> {
        let root_id = Uuid::new_v4();
        let mut children = Vec::new();

        // Simple line-based AST
        for (line_num, line) in content.lines().enumerate() {
            if !line.trim().is_empty() {
                let child_id = Uuid::new_v4();
                children.push(child_id);
            }
        }

        Ok(ASTNode {
            id: root_id,
            node_type: "file".to_string(),
            name: Some("root".to_string()),
            start_line: 1,
            end_line: content.lines().count() as u32,
            children,
            parent: None,
            scope_depth: 0,
            attributes: HashMap::new(),
        })
    }

    fn combine_and_enhance_symbols(&self, ts_symbols: Vec<Symbol>, ctags_symbols: Vec<Symbol>, ast_root: &ASTNode, content: &str) -> Result<Vec<EnhancedSymbol>, Box<dyn std::error::Error>> {
        let mut enhanced_symbols = Vec::new();
        let mut processed_names = std::collections::HashSet::new();

        // Process Tree-sitter symbols first (higher priority)
        for symbol in ts_symbols {
            if processed_names.insert(format!("{}:{}", symbol.name, symbol.line)) {
                let enhanced = self.enhance_symbol(symbol, ast_root, content)?;
                enhanced_symbols.push(enhanced);
            }
        }

        // Add unique ctags symbols
        for symbol in ctags_symbols {
            let key = format!("{}:{}", symbol.name, symbol.line);
            if !processed_names.contains(&key) {
                let enhanced = self.enhance_symbol(symbol, ast_root, content)?;
                enhanced_symbols.push(enhanced);
            }
        }

        Ok(enhanced_symbols)
    }

    fn enhance_symbol(&self, symbol: Symbol, ast_root: &ASTNode, content: &str) -> Result<EnhancedSymbol, Box<dyn std::error::Error>> {
        let ast_node_id = self.find_ast_node_for_symbol(&symbol, ast_root);
        let complexity = self.calculate_complexity(&symbol, content);
        let visibility = self.determine_visibility(&symbol, content);
        let modifiers = self.extract_modifiers(&symbol, content);
        let documentation = self.extract_documentation(&symbol, content);

        Ok(EnhancedSymbol {
            base: symbol,
            ast_node_id,
            complexity,
            dependencies: Vec::new(), // Will be populated by cross-reference builder
            usages: Vec::new(),
            documentation,
            visibility,
            modifiers,
        })
    }

    fn find_ast_node_for_symbol(&self, symbol: &Symbol, ast_root: &ASTNode) -> Option<Uuid> {
        self.find_ast_node_recursive(symbol, ast_root)
    }

    fn find_ast_node_recursive(&self, symbol: &Symbol, node: &ASTNode) -> Option<Uuid> {
        // Check if this node matches the symbol
        if node.start_line <= symbol.line && node.end_line >= symbol.line {
            if let Some(ref name) = node.name {
                if name == &symbol.name {
                    return Some(node.id);
                }
            }
        }

        // Search children (this would need access to child nodes in a real implementation)
        None
    }

    fn calculate_complexity(&self, symbol: &Symbol, content: &str) -> u32 {
        match symbol.symbol_type {
            SymbolType::Function | SymbolType::Method => {
                self.calculate_cyclomatic_complexity(symbol, content)
            }
            SymbolType::Class => {
                // Count methods and fields
                10 // Placeholder
            }
            _ => 1,
        }
    }

    fn calculate_cyclomatic_complexity(&self, symbol: &Symbol, content: &str) -> u32 {
        let lines: Vec<&str> = content.lines().collect();
        let start = (symbol.line as usize).saturating_sub(1);
        let end = std::cmp::min(symbol.end_line as usize, lines.len());
        
        let mut complexity = 1; // Base complexity
        
        for line in &lines[start..end] {
            let line = line.trim();
            // Count decision points
            if line.contains("if ") || line.contains("else") || 
               line.contains("while ") || line.contains("for ") ||
               line.contains("match ") || line.contains("case ") ||
               line.contains("catch ") || line.contains("&&") || line.contains("||") {
                complexity += 1;
            }
        }
        
        complexity
    }

    fn determine_visibility(&self, symbol: &Symbol, content: &str) -> Visibility {
        if let Some(ref signature) = symbol.signature {
            if signature.contains("public") {
                Visibility::Public
            } else if signature.contains("private") {
                Visibility::Private
            } else if signature.contains("protected") {
                Visibility::Protected
            } else if signature.contains("internal") {
                Visibility::Internal
            } else {
                Visibility::Package
            }
        } else {
            Visibility::Public // Default
        }
    }

    fn extract_modifiers(&self, symbol: &Symbol, _content: &str) -> Vec<String> {
        let mut modifiers = Vec::new();
        
        if let Some(ref signature) = symbol.signature {
            if signature.contains("static") { modifiers.push("static".to_string()); }
            if signature.contains("final") { modifiers.push("final".to_string()); }
            if signature.contains("abstract") { modifiers.push("abstract".to_string()); }
            if signature.contains("async") { modifiers.push("async".to_string()); }
            if signature.contains("const") { modifiers.push("const".to_string()); }
        }
        
        modifiers
    }

    fn extract_documentation(&self, symbol: &Symbol, content: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let symbol_line = (symbol.line as usize).saturating_sub(1);
        
        if symbol_line == 0 { return None; }
        
        let mut doc_lines = Vec::new();
        let mut i = symbol_line.saturating_sub(1);
        
        // Look backwards for documentation comments
        while i > 0 {
            let line = lines[i].trim();
            if line.starts_with("///") || line.starts_with("/**") || line.starts_with("//!") {
                doc_lines.insert(0, line.trim_start_matches("///").trim_start_matches("/**").trim());
            } else if line.is_empty() {
                // Continue through empty lines
            } else {
                break;
            }
            i = i.saturating_sub(1);
        }
        
        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join("\n"))
        }
    }

    fn build_symbol_hierarchy(&self, symbols: &[EnhancedSymbol]) -> HashMap<Uuid, Vec<Uuid>> {
        let mut hierarchy = HashMap::new();
        
        for symbol in symbols {
            if let Some(ref scope) = symbol.base.scope {
                // Find parent symbol by scope
                for parent in symbols {
                    if parent.base.name == *scope {
                        hierarchy.entry(parent.base.id).or_insert_with(Vec::new).push(symbol.base.id);
                        break;
                    }
                }
            }
        }
        
        hierarchy
    }

    fn add_ast_to_database(&self, database: &mut SymbolDatabase, file_id: Uuid, ast_root: ASTNode) {
        database.file_asts.insert(file_id, ast_root.id);
        self.add_ast_node_recursive(database, ast_root);
    }

    fn add_ast_node_recursive(&self, database: &mut SymbolDatabase, node: ASTNode) {
        let node_id = node.id;
        database.ast_nodes.insert(node_id, node);
    }

    fn detect_language(&self, path: &Path) -> Result<Language, Box<dyn std::error::Error>> {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Ok(Language::from_extension(ext))
        } else {
            Ok(Language::Unknown)
        }
    }
}

impl SymbolDatabase {
    pub fn get_symbol_by_name(&self, name: &str) -> Vec<&EnhancedSymbol> {
        self.symbols.values().filter(|s| s.base.name == name).collect()
    }

    pub fn get_symbols_in_scope(&self, scope: &str) -> Vec<&EnhancedSymbol> {
        self.symbols.values()
            .filter(|s| s.base.scope.as_ref().map_or(false, |s| s == scope))
            .collect()
    }

    pub fn get_children_symbols(&self, parent_id: Uuid) -> Vec<&EnhancedSymbol> {
        if let Some(children) = self.symbol_hierarchy.get(&parent_id) {
            children.iter()
                .filter_map(|id| self.symbols.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_ast_node(&self, node_id: Uuid) -> Option<&ASTNode> {
        self.ast_nodes.get(&node_id)
    }

    pub fn get_file_ast_root(&self, file_id: Uuid) -> Option<&ASTNode> {
        self.file_asts.get(&file_id)
            .and_then(|root_id| self.ast_nodes.get(root_id))
    }
}