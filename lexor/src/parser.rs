use crate::types::*;
use crate::utils::highlight_matches;
use tree_sitter::{Parser, Tree, Node, Query, QueryCursor, TreeCursor};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;
use once_cell::sync::Lazy;

pub struct LanguageParser {
    parsers: HashMap<Language, Parser>,
    queries: HashMap<Language, SymbolQuery>,
}

struct SymbolQuery {
    functions: Query,
    classes: Query,
    variables: Query,
    imports: Query,
}

impl LanguageParser {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();
        let mut queries = HashMap::new();

        // Initialize parsers for supported languages
        for language in [
            Language::Rust,
            Language::JavaScript,
            Language::TypeScript,
            Language::Python,
            Language::Java,
            Language::C,
            Language::Cpp,
            Language::Go,
        ] {
            if let Some(ts_lang) = language.tree_sitter_language() {
                let mut parser = Parser::new();
                if parser.set_language(&ts_lang).is_ok() {
                    parsers.insert(language.clone(), parser);
                    
                    if let Some(query) = Self::create_symbol_queries(&language) {
                        queries.insert(language, query);
                    }
                }
            }
        }

        Self { parsers, queries }
    }

    fn create_symbol_queries(language: &Language) -> Option<SymbolQuery> {
        match language {
            Language::Rust => Some(SymbolQuery {
                functions: Query::new(
                    &tree_sitter_rust::language(),
                    r#"
                    (function_item
                        name: (identifier) @name
                        parameters: (parameters) @params) @function
                    
                    (impl_item
                        type: (type_identifier) @type
                        body: (declaration_list
                            (function_item
                                name: (identifier) @name
                                parameters: (parameters) @params) @method))
                    "#
                ).ok()?,
                classes: Query::new(
                    &tree_sitter_rust::language(),
                    r#"
                    (struct_item
                        name: (type_identifier) @name) @struct
                    
                    (enum_item
                        name: (type_identifier) @name) @enum
                    
                    (trait_item
                        name: (type_identifier) @name) @trait
                    
                    (impl_item
                        type: (type_identifier) @name) @impl
                    "#
                ).ok()?,
                variables: Query::new(
                    &tree_sitter_rust::language(),
                    r#"
                    (let_declaration
                        pattern: (identifier) @name) @variable
                    
                    (const_item
                        name: (identifier) @name) @constant
                    
                    (static_item
                        name: (identifier) @name) @static
                    "#
                ).ok()?,
                imports: Query::new(
                    &tree_sitter_rust::language(),
                    r#"
                    (use_declaration
                        argument: (scoped_identifier) @import) @use
                    
                    (extern_crate_declaration
                        name: (identifier) @import) @extern
                    "#
                ).ok()?,
            }),
            
            Language::JavaScript | Language::TypeScript => Some(SymbolQuery {
                functions: Query::new(
                    &tree_sitter_javascript::language(),
                    r#"
                    (function_declaration
                        name: (identifier) @name
                        parameters: (formal_parameters) @params) @function
                    
                    (method_definition
                        name: (property_identifier) @name
                        parameters: (formal_parameters) @params) @method
                    
                    (arrow_function
                        parameters: (formal_parameters) @params) @arrow_function
                    "#
                ).ok()?,
                classes: Query::new(
                    &tree_sitter_javascript::language(),
                    r#"
                    (class_declaration
                        name: (identifier) @name) @class
                    
                    (interface_declaration
                        name: (type_identifier) @name) @interface
                    "#
                ).ok()?,
                variables: Query::new(
                    &tree_sitter_javascript::language(),
                    r#"
                    (variable_declarator
                        name: (identifier) @name) @variable
                    
                    (lexical_declaration
                        (variable_declarator
                            name: (identifier) @name)) @let_const
                    "#
                ).ok()?,
                imports: Query::new(
                    &tree_sitter_javascript::language(),
                    r#"
                    (import_statement
                        source: (string) @source) @import
                    
                    (import_clause
                        (identifier) @name) @import_name
                    "#
                ).ok()?,
            }),
            
            Language::Python => Some(SymbolQuery {
                functions: Query::new(
                    &tree_sitter_python::language(),
                    r#"
                    (function_definition
                        name: (identifier) @name
                        parameters: (parameters) @params) @function
                    "#
                ).ok()?,
                classes: Query::new(
                    &tree_sitter_python::language(),
                    r#"
                    (class_definition
                        name: (identifier) @name) @class
                    "#
                ).ok()?,
                variables: Query::new(
                    &tree_sitter_python::language(),
                    r#"
                    (assignment
                        left: (identifier) @name) @variable
                    "#
                ).ok()?,
                imports: Query::new(
                    &tree_sitter_python::language(),
                    r#"
                    (import_statement
                        name: (dotted_name) @name) @import
                    
                    (import_from_statement
                        module_name: (dotted_name) @module) @import_from
                    "#
                ).ok()?,
            }),
            
            Language::Java => Some(SymbolQuery {
                functions: Query::new(
                    &tree_sitter_java::language(),
                    r#"
                    (method_declaration
                        name: (identifier) @name
                        parameters: (formal_parameters) @params) @method
                    
                    (constructor_declaration
                        name: (identifier) @name
                        parameters: (formal_parameters) @params) @constructor
                    "#
                ).ok()?,
                classes: Query::new(
                    &tree_sitter_java::language(),
                    r#"
                    (class_declaration
                        name: (identifier) @name) @class
                    
                    (interface_declaration
                        name: (identifier) @name) @interface
                    
                    (enum_declaration
                        name: (identifier) @name) @enum
                    "#
                ).ok()?,
                variables: Query::new(
                    &tree_sitter_java::language(),
                    r#"
                    (field_declaration
                        declarator: (variable_declarator
                            name: (identifier) @name)) @field
                    
                    (local_variable_declaration
                        declarator: (variable_declarator
                            name: (identifier) @name)) @variable
                    "#
                ).ok()?,
                imports: Query::new(
                    &tree_sitter_java::language(),
                    r#"
                    (import_declaration
                        (scoped_identifier) @import) @import_stmt
                    
                    (package_declaration
                        (scoped_identifier) @package) @package_stmt
                    "#
                ).ok()?,
            }),
            
            _ => None,
        }
    }

    pub fn parse_file(&mut self, file_id: Uuid, path: &Path, content: &str, language: &Language) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        let parser = self.parsers.get_mut(language)
            .ok_or("Parser not available for language")?;
        
        let tree = parser.parse(content, None)
            .ok_or("Failed to parse file")?;
        
        let mut symbols = Vec::new();
        
        if let Some(queries) = self.queries.get(language) {
            symbols.extend(self.extract_symbols_from_query(&tree, content, file_id, &queries.functions, SymbolType::Function)?);
            symbols.extend(self.extract_symbols_from_query(&tree, content, file_id, &queries.classes, SymbolType::Class)?);
            symbols.extend(self.extract_symbols_from_query(&tree, content, file_id, &queries.variables, SymbolType::Variable)?);
        }
        
        Ok(symbols)
    }

    fn extract_symbols_from_query(
        &self,
        tree: &Tree,
        content: &str,
        file_id: Uuid,
        query: &Query,
        default_type: SymbolType,
    ) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, tree.root_node(), content.as_bytes());
        let mut symbols = Vec::new();

        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let start_pos = node.start_position();
                let end_pos = node.end_position();
                
                let name = node.utf8_text(content.as_bytes())
                    .unwrap_or("unknown")
                    .to_string();

                let symbol_type = self.determine_symbol_type(&node, &default_type);
                
                let symbol = Symbol {
                    id: Uuid::new_v4(),
                    file_id,
                    name,
                    symbol_type,
                    line: start_pos.row as u32 + 1,
                    column: start_pos.column as u32,
                    end_line: end_pos.row as u32 + 1,
                    end_column: end_pos.column as u32,
                    signature: self.extract_signature(&node, content),
                    scope: self.extract_scope(&node, content),
                    namespace: None,
                };
                
                symbols.push(symbol);
            }
        }

        Ok(symbols)
    }

    fn determine_symbol_type(&self, node: &Node, default_type: &SymbolType) -> SymbolType {
        match node.kind() {
            "function_item" | "function_declaration" | "method_declaration" => SymbolType::Function,
            "method_definition" => SymbolType::Method,
            "class_declaration" | "class_definition" => SymbolType::Class,
            "interface_declaration" => SymbolType::Interface,
            "struct_item" => SymbolType::Struct,
            "enum_item" | "enum_declaration" => SymbolType::Enum,
            "trait_item" => SymbolType::Interface,
            "const_item" | "static_item" => SymbolType::Constant,
            "field_declaration" => SymbolType::Field,
            "parameter" => SymbolType::Parameter,
            "module" | "mod_item" => SymbolType::Module,
            "namespace" => SymbolType::Namespace,
            "macro_definition" => SymbolType::Macro,
            "type_alias" | "type_definition" => SymbolType::Type,
            _ => default_type.clone(),
        }
    }

    fn extract_signature(&self, node: &Node, content: &str) -> Option<String> {
        // Extract function signature or type definition
        if let Ok(text) = node.utf8_text(content.as_bytes()) {
            // Clean up the signature by removing excessive whitespace
            let cleaned = text.lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            
            if cleaned.len() > 200 {
                Some(format!("{}...", &cleaned[..200]))
            } else {
                Some(cleaned)
            }
        } else {
            None
        }
    }

    fn extract_scope(&self, node: &Node, content: &str) -> Option<String> {
        let mut current = node.parent();
        let mut scopes = Vec::new();

        while let Some(parent) = current {
            match parent.kind() {
                "class_declaration" | "class_definition" | "struct_item" | "enum_item" | "trait_item" => {
                    if let Some(name_node) = parent.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(content.as_bytes()) {
                            scopes.push(name.to_string());
                        }
                    }
                }
                "impl_item" => {
                    if let Some(type_node) = parent.child_by_field_name("type") {
                        if let Ok(name) = type_node.utf8_text(content.as_bytes()) {
                            scopes.push(format!("impl {}", name));
                        }
                    }
                }
                "module" | "mod_item" => {
                    if let Some(name_node) = parent.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(content.as_bytes()) {
                            scopes.push(name.to_string());
                        }
                    }
                }
                _ => {}
            }
            current = parent.parent();
        }

        if scopes.is_empty() {
            None
        } else {
            scopes.reverse();
            Some(scopes.join("::"))
        }
    }

    pub fn extract_references(&mut self, file_id: Uuid, content: &str, language: &Language, symbols: &[Symbol]) -> Vec<Reference> {
        let mut references = Vec::new();
        
        // Simple reference extraction based on symbol names
        for symbol in symbols {
            let matches = highlight_matches(content, &symbol.name, true);
            
            for (start, end) in matches {
                // Convert byte position to line/column
                let line_col = self.byte_pos_to_line_col(content, start);
                
                // Skip if this is the definition itself
                if line_col.0 == symbol.line && line_col.1 >= symbol.column && line_col.1 <= symbol.end_column {
                    continue;
                }
                
                let reference = Reference {
                    id: Uuid::new_v4(),
                    symbol_id: symbol.id,
                    file_id,
                    line: line_col.0,
                    column: line_col.1,
                    reference_type: ReferenceType::Usage,
                    context: self.extract_line_context(content, line_col.0),
                };
                
                references.push(reference);
            }
        }
        
        references
    }

    fn byte_pos_to_line_col(&self, content: &str, byte_pos: usize) -> (u32, u32) {
        let mut line = 1;
        let mut col = 0;
        
        for (i, ch) in content.char_indices() {
            if i >= byte_pos {
                break;
            }
            
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        
        (line, col)
    }

    fn extract_line_context(&self, content: &str, line_number: u32) -> String {
        content.lines()
            .nth((line_number - 1) as usize)
            .unwrap_or("")
            .trim()
            .to_string()
    }
}

pub struct SimpleParser;

impl SimpleParser {
    pub fn parse_file_simple(file_id: Uuid, content: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num as u32 + 1;
            
            // Simple regex-based parsing for common patterns
            if let Some(symbol) = Self::parse_function_declaration(file_id, line, line_num) {
                symbols.push(symbol);
            }
            
            if let Some(symbol) = Self::parse_class_declaration(file_id, line, line_num) {
                symbols.push(symbol);
            }
            
            if let Some(symbol) = Self::parse_variable_declaration(file_id, line, line_num) {
                symbols.push(symbol);
            }
        }
        
        symbols
    }

    fn parse_function_declaration(file_id: Uuid, line: &str, line_num: u32) -> Option<Symbol> {
        use regex::Regex;
        
        static FUNCTION_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?:fn|function|def|public|private|protected)?\s*(\w+)\s*\([^)]*\)").unwrap()
        });
        
        if let Some(captures) = FUNCTION_REGEX.captures(line) {
            if let Some(name_match) = captures.get(1) {
                return Some(Symbol {
                    id: Uuid::new_v4(),
                    file_id,
                    name: name_match.as_str().to_string(),
                    symbol_type: SymbolType::Function,
                    line: line_num,
                    column: name_match.start() as u32,
                    end_line: line_num,
                    end_column: name_match.end() as u32,
                    signature: Some(line.trim().to_string()),
                    scope: None,
                    namespace: None,
                });
            }
        }
        
        None
    }

    fn parse_class_declaration(file_id: Uuid, line: &str, line_num: u32) -> Option<Symbol> {
        use regex::Regex;
        
        static CLASS_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?:class|struct|interface|enum)\s+(\w+)").unwrap()
        });
        
        if let Some(captures) = CLASS_REGEX.captures(line) {
            if let Some(name_match) = captures.get(1) {
                return Some(Symbol {
                    id: Uuid::new_v4(),
                    file_id,
                    name: name_match.as_str().to_string(),
                    symbol_type: SymbolType::Class,
                    line: line_num,
                    column: name_match.start() as u32,
                    end_line: line_num,
                    end_column: name_match.end() as u32,
                    signature: Some(line.trim().to_string()),
                    scope: None,
                    namespace: None,
                });
            }
        }
        
        None
    }

    fn parse_variable_declaration(file_id: Uuid, line: &str, line_num: u32) -> Option<Symbol> {
        use regex::Regex;
        
        static VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?:let|var|const|static|final)\s+(\w+)").unwrap()
        });
        
        if let Some(captures) = VAR_REGEX.captures(line) {
            if let Some(name_match) = captures.get(1) {
                return Some(Symbol {
                    id: Uuid::new_v4(),
                    file_id,
                    name: name_match.as_str().to_string(),
                    symbol_type: SymbolType::Variable,
                    line: line_num,
                    column: name_match.start() as u32,
                    end_line: line_num,
                    end_column: name_match.end() as u32,
                    signature: Some(line.trim().to_string()),
                    scope: None,
                    namespace: None,
                });
            }
        }
        
        None
    }
}