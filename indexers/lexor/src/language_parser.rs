use crate::types::*;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use uuid::Uuid;
use tree_sitter::{Parser, Tree, Node, Query, QueryCursor};
use once_cell::sync::Lazy;
use serde_json::Value;

pub struct UniversalLanguageParser {
    tree_sitter: TreeSitterParser,
    ctags: CtagsParser,
    language_config: LanguageConfig,
}

struct TreeSitterParser {
    parsers: HashMap<Language, Parser>,
    queries: HashMap<Language, LanguageQueries>,
}

struct CtagsParser {
    ctags_path: String,
    supported_languages: Vec<String>,
}

struct LanguageQueries {
    symbols: Query,
    references: Query,
    definitions: Query,
}

#[derive(Clone)]
pub struct LanguageConfig {
    tree_sitter_languages: HashMap<Language, &'static str>,
    ctags_languages: HashMap<String, Language>,
    file_extensions: HashMap<String, Language>,
}

impl UniversalLanguageParser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let language_config = LanguageConfig::new();
        let tree_sitter = TreeSitterParser::new(&language_config)?;
        let ctags = CtagsParser::new()?;

        Ok(Self {
            tree_sitter,
            ctags,
            language_config,
        })
    }

    pub fn parse_file(&mut self, file_id: Uuid, path: &Path, content: &str) -> Result<ParseResult, Box<dyn std::error::Error>> {
        let language = self.detect_language(path)?;
        
        // Try Tree-sitter first for supported languages
        if self.tree_sitter.supports_language(&language) {
            match self.tree_sitter.parse_file(file_id, content, &language) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    log::warn!("Tree-sitter parsing failed for {:?}: {}, falling back to ctags", path, e);
                }
            }
        }

        // Fallback to ctags for broader language support
        self.ctags.parse_file(file_id, path, content, &language)
    }

    pub fn parse_incremental(&mut self, file_id: Uuid, old_tree: Option<&Tree>, content: &str, language: &Language) -> Result<ParseResult, Box<dyn std::error::Error>> {
        // Only Tree-sitter supports incremental parsing
        if self.tree_sitter.supports_language(language) {
            self.tree_sitter.parse_incremental(file_id, old_tree, content, language)
        } else {
            // For non-Tree-sitter languages, do full parse
            self.ctags.parse_content(file_id, content, language)
        }
    }

    fn detect_language(&self, path: &Path) -> Result<Language, Box<dyn std::error::Error>> {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(language) = self.language_config.file_extensions.get(ext) {
                return Ok(language.clone());
            }
        }

        // Fallback to filename-based detection
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            match filename {
                "Dockerfile" => Ok(Language::Dockerfile),
                "Makefile" | "makefile" => Ok(Language::Make),
                "CMakeLists.txt" => Ok(Language::CMake),
                _ => Ok(Language::Text), // Default fallback
            }
        } else {
            Ok(Language::Text)
        }
    }

    pub fn get_supported_languages(&self) -> Vec<Language> {
        let mut languages = self.tree_sitter.get_supported_languages();
        languages.extend(self.ctags.get_supported_languages());
        languages.sort();
        languages.dedup();
        languages
    }
}

impl TreeSitterParser {
    fn new(config: &LanguageConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut parsers = HashMap::new();
        let mut queries = HashMap::new();

        // Initialize Tree-sitter parsers for supported languages
        for (language, _) in &config.tree_sitter_languages {
            if let Some(ts_lang) = language.tree_sitter_language() {
                let mut parser = Parser::new();
                if parser.set_language(&ts_lang).is_ok() {
                    parsers.insert(language.clone(), parser);
                    
                    if let Ok(lang_queries) = Self::create_language_queries(language) {
                        queries.insert(language.clone(), lang_queries);
                    }
                }
            }
        }

        Ok(Self { parsers, queries })
    }

    fn create_language_queries(language: &Language) -> Result<LanguageQueries, Box<dyn std::error::Error>> {
        let (symbols_query, refs_query, defs_query) = match language {
            Language::Rust => (
                r#"
                (function_item name: (identifier) @name) @function
                (struct_item name: (type_identifier) @name) @struct
                (enum_item name: (type_identifier) @name) @enum
                (trait_item name: (type_identifier) @name) @trait
                (impl_item type: (type_identifier) @name) @impl
                (const_item name: (identifier) @name) @constant
                (static_item name: (identifier) @name) @static
                (mod_item name: (identifier) @name) @module
                "#,
                r#"
                (identifier) @reference
                (type_identifier) @type_reference
                "#,
                r#"
                (function_item name: (identifier) @definition)
                (struct_item name: (type_identifier) @definition)
                (enum_item name: (type_identifier) @definition)
                "#
            ),
            
            Language::JavaScript | Language::TypeScript => (
                r#"
                (function_declaration name: (identifier) @name) @function
                (method_definition name: (property_identifier) @name) @method
                (class_declaration name: (identifier) @name) @class
                (interface_declaration name: (type_identifier) @name) @interface
                (variable_declarator name: (identifier) @name) @variable
                (arrow_function) @arrow_function
                "#,
                r#"
                (identifier) @reference
                (property_identifier) @property_reference
                "#,
                r#"
                (function_declaration name: (identifier) @definition)
                (class_declaration name: (identifier) @definition)
                "#
            ),
            
            Language::Python => (
                r#"
                (function_definition name: (identifier) @name) @function
                (class_definition name: (identifier) @name) @class
                (assignment left: (identifier) @name) @variable
                (import_statement name: (dotted_name) @name) @import
                (import_from_statement module_name: (dotted_name) @name) @import_from
                "#,
                r#"
                (identifier) @reference
                (attribute attribute: (identifier) @attribute_reference)
                "#,
                r#"
                (function_definition name: (identifier) @definition)
                (class_definition name: (identifier) @definition)
                "#
            ),
            
            Language::Java => (
                r#"
                (method_declaration name: (identifier) @name) @method
                (class_declaration name: (identifier) @name) @class
                (interface_declaration name: (identifier) @name) @interface
                (enum_declaration name: (identifier) @name) @enum
                (field_declaration declarator: (variable_declarator name: (identifier) @name)) @field
                (constructor_declaration name: (identifier) @name) @constructor
                "#,
                r#"
                (identifier) @reference
                (type_identifier) @type_reference
                "#,
                r#"
                (method_declaration name: (identifier) @definition)
                (class_declaration name: (identifier) @definition)
                "#
            ),
            
            Language::C | Language::Cpp => (
                r#"
                (function_definition declarator: (function_declarator declarator: (identifier) @name)) @function
                (declaration declarator: (function_declarator declarator: (identifier) @name)) @function_decl
                (struct_specifier name: (type_identifier) @name) @struct
                (enum_specifier name: (type_identifier) @name) @enum
                (typedef_declaration declarator: (type_identifier) @name) @typedef
                "#,
                r#"
                (identifier) @reference
                (type_identifier) @type_reference
                "#,
                r#"
                (function_definition declarator: (function_declarator declarator: (identifier) @definition))
                (struct_specifier name: (type_identifier) @definition)
                "#
            ),
            
            Language::Go => (
                r#"
                (function_declaration name: (identifier) @name) @function
                (method_declaration name: (field_identifier) @name) @method
                (type_declaration (type_spec name: (type_identifier) @name)) @type
                (var_declaration (var_spec name: (identifier) @name)) @variable
                (const_declaration (const_spec name: (identifier) @name)) @constant
                "#,
                r#"
                (identifier) @reference
                (type_identifier) @type_reference
                "#,
                r#"
                (function_declaration name: (identifier) @definition)
                (type_declaration (type_spec name: (type_identifier) @definition))
                "#
            ),
            
            _ => return Err("Unsupported language for Tree-sitter queries".into()),
        };

        let ts_lang = language.tree_sitter_language()
            .ok_or("Tree-sitter language not available")?;

        Ok(LanguageQueries {
            symbols: Query::new(&ts_lang, symbols_query)?,
            references: Query::new(&ts_lang, refs_query)?,
            definitions: Query::new(&ts_lang, defs_query)?,
        })
    }

    fn supports_language(&self, language: &Language) -> bool {
        self.parsers.contains_key(language)
    }

    fn parse_file(&mut self, file_id: Uuid, content: &str, language: &Language) -> Result<ParseResult, Box<dyn std::error::Error>> {
        let parser = self.parsers.get_mut(language)
            .ok_or("Parser not available")?;
        
        let tree = parser.parse(content, None)
            .ok_or("Failed to parse content")?;
        
        self.extract_symbols_and_references(file_id, &tree, content, language)
    }

    fn parse_incremental(&mut self, file_id: Uuid, old_tree: Option<&Tree>, content: &str, language: &Language) -> Result<ParseResult, Box<dyn std::error::Error>> {
        let parser = self.parsers.get_mut(language)
            .ok_or("Parser not available")?;
        
        let tree = parser.parse(content, old_tree)
            .ok_or("Failed to parse content incrementally")?;
        
        self.extract_symbols_and_references(file_id, &tree, content, language)
    }

    fn extract_symbols_and_references(&self, file_id: Uuid, tree: &Tree, content: &str, language: &Language) -> Result<ParseResult, Box<dyn std::error::Error>> {
        let queries = self.queries.get(language)
            .ok_or("Queries not available for language")?;

        let mut symbols = Vec::new();
        let mut references = Vec::new();

        // Extract symbols
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&queries.symbols, tree.root_node(), content.as_bytes());

        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let start_pos = node.start_position();
                let end_pos = node.end_position();
                
                if let Ok(name) = node.utf8_text(content.as_bytes()) {
                    let symbol = Symbol {
                        id: Uuid::new_v4(),
                        file_id,
                        name: name.to_string(),
                        symbol_type: self.determine_symbol_type(&node),
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
        }

        // Extract references
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&queries.references, tree.root_node(), content.as_bytes());

        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let start_pos = node.start_position();
                
                if let Ok(name) = node.utf8_text(content.as_bytes()) {
                    // Find matching symbol
                    if let Some(symbol) = symbols.iter().find(|s| s.name == name) {
                        let reference = Reference {
                            id: Uuid::new_v4(),
                            symbol_id: symbol.id,
                            file_id,
                            line: start_pos.row as u32 + 1,
                            column: start_pos.column as u32,
                            reference_type: ReferenceType::Usage,
                            context: self.extract_line_context(content, start_pos.row as u32 + 1),
                        };
                        references.push(reference);
                    }
                }
            }
        }

        Ok(ParseResult { symbols, references })
    }

    fn determine_symbol_type(&self, node: &Node) -> SymbolType {
        match node.kind() {
            "function_item" | "function_declaration" | "function_definition" => SymbolType::Function,
            "method_declaration" | "method_definition" => SymbolType::Method,
            "class_declaration" | "class_definition" => SymbolType::Class,
            "interface_declaration" => SymbolType::Interface,
            "struct_item" | "struct_specifier" => SymbolType::Struct,
            "enum_item" | "enum_declaration" | "enum_specifier" => SymbolType::Enum,
            "trait_item" => SymbolType::Interface,
            "const_item" | "constant" => SymbolType::Constant,
            "field_declaration" | "field" => SymbolType::Field,
            "variable_declarator" | "variable" => SymbolType::Variable,
            "module" | "mod_item" => SymbolType::Module,
            "namespace" => SymbolType::Namespace,
            "macro_definition" => SymbolType::Macro,
            "type_declaration" | "typedef" => SymbolType::Type,
            _ => SymbolType::Variable,
        }
    }

    fn extract_signature(&self, node: &Node, content: &str) -> Option<String> {
        if let Ok(text) = node.utf8_text(content.as_bytes()) {
            let cleaned = text.lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            
            Some(if cleaned.len() > 200 {
                format!("{}...", &cleaned[..200])
            } else {
                cleaned
            })
        } else {
            None
        }
    }

    fn extract_scope(&self, node: &Node, content: &str) -> Option<String> {
        let mut current = node.parent();
        let mut scopes = Vec::new();

        while let Some(parent) = current {
            match parent.kind() {
                "class_declaration" | "class_definition" | "struct_item" | "enum_item" => {
                    if let Some(name_node) = parent.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(content.as_bytes()) {
                            scopes.push(name.to_string());
                        }
                    }
                }
                "module" | "mod_item" | "namespace" => {
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

    fn extract_line_context(&self, content: &str, line_number: u32) -> String {
        content.lines()
            .nth((line_number - 1) as usize)
            .unwrap_or("")
            .trim()
            .to_string()
    }

    fn get_supported_languages(&self) -> Vec<Language> {
        self.parsers.keys().cloned().collect()
    }
}

impl CtagsParser {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ctags_path = Self::find_ctags_executable()?;
        let supported_languages = Self::get_ctags_languages(&ctags_path)?;

        Ok(Self {
            ctags_path,
            supported_languages,
        })
    }

    fn find_ctags_executable() -> Result<String, Box<dyn std::error::Error>> {
        // Try different common ctags executables
        for ctags_cmd in &["universal-ctags", "ctags", "exuberant-ctags"] {
            if Command::new(ctags_cmd).arg("--version").output().is_ok() {
                return Ok(ctags_cmd.to_string());
            }
        }
        Err("Universal Ctags not found. Please install universal-ctags.".into())
    }

    fn get_ctags_languages(ctags_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = Command::new(ctags_path)
            .args(&["--list-languages"])
            .output()?;

        let languages = String::from_utf8(output.stdout)?
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(languages)
    }

    fn parse_file(&self, file_id: Uuid, path: &Path, content: &str, language: &Language) -> Result<ParseResult, Box<dyn std::error::Error>> {
        // Write content to temporary file for ctags processing
        let temp_file = std::env::temp_dir().join(format!("ctags_temp_{}", file_id));
        std::fs::write(&temp_file, content)?;

        let result = self.parse_with_ctags(file_id, &temp_file, language);
        
        // Clean up temporary file
        let _ = std::fs::remove_file(&temp_file);
        
        result
    }

    fn parse_content(&self, file_id: Uuid, content: &str, language: &Language) -> Result<ParseResult, Box<dyn std::error::Error>> {
        let temp_file = std::env::temp_dir().join(format!("ctags_temp_{}", file_id));
        std::fs::write(&temp_file, content)?;

        let result = self.parse_with_ctags(file_id, &temp_file, language);
        
        let _ = std::fs::remove_file(&temp_file);
        result
    }

    fn parse_with_ctags(&self, file_id: Uuid, file_path: &Path, _language: &Language) -> Result<ParseResult, Box<dyn std::error::Error>> {
        let output = Command::new(&self.ctags_path)
            .args(&[
                "--output-format=json",
                "--fields=+n+S+Z+K+l+s",
                "--extras=+r",
                "-f", "-",
                file_path.to_str().unwrap()
            ])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let mut symbols = Vec::new();

        for line in stdout.lines() {
            if let Ok(tag) = serde_json::from_str::<Value>(line) {
                if let Some(symbol) = self.parse_ctags_entry(file_id, &tag) {
                    symbols.push(symbol);
                }
            }
        }

        // Ctags doesn't provide reference information, so we return empty references
        Ok(ParseResult {
            symbols,
            references: Vec::new(),
        })
    }

    fn parse_ctags_entry(&self, file_id: Uuid, tag: &Value) -> Option<Symbol> {
        let name = tag.get("name")?.as_str()?.to_string();
        let kind = tag.get("kind")?.as_str()?;
        let line = tag.get("line")?.as_u64()? as u32;
        let signature = tag.get("signature").and_then(|s| s.as_str()).map(|s| s.to_string());
        let scope = tag.get("scope").and_then(|s| s.as_str()).map(|s| s.to_string());

        let symbol_type = match kind {
            "function" | "f" => SymbolType::Function,
            "method" | "m" => SymbolType::Method,
            "class" | "c" => SymbolType::Class,
            "interface" | "i" => SymbolType::Interface,
            "struct" | "s" => SymbolType::Struct,
            "enum" | "e" => SymbolType::Enum,
            "variable" | "v" => SymbolType::Variable,
            "field" | "member" => SymbolType::Field,
            "constant" | "d" => SymbolType::Constant,
            "namespace" | "n" => SymbolType::Namespace,
            "module" => SymbolType::Module,
            "macro" => SymbolType::Macro,
            "typedef" | "t" => SymbolType::Type,
            _ => SymbolType::Variable,
        };

        Some(Symbol {
            id: Uuid::new_v4(),
            file_id,
            name,
            symbol_type,
            line,
            column: 0, // Ctags doesn't provide column information
            end_line: line,
            end_column: 0,
            signature,
            scope,
            namespace: None,
        })
    }

    fn get_supported_languages(&self) -> Vec<Language> {
        // Map ctags languages to our Language enum
        self.supported_languages.iter()
            .filter_map(|lang| match lang.as_str() {
                "C" => Some(Language::C),
                "C++" => Some(Language::Cpp),
                "Java" => Some(Language::Java),
                "Python" => Some(Language::Python),
                "JavaScript" => Some(Language::JavaScript),
                "TypeScript" => Some(Language::TypeScript),
                "Rust" => Some(Language::Rust),
                "Go" => Some(Language::Go),
                "PHP" => Some(Language::Php),
                "Ruby" => Some(Language::Ruby),
                "C#" => Some(Language::CSharp),
                "Swift" => Some(Language::Swift),
                "Kotlin" => Some(Language::Kotlin),
                "Scala" => Some(Language::Scala),
                "Perl" => Some(Language::Perl),
                "Lua" => Some(Language::Lua),
                "Shell" => Some(Language::Shell),
                "SQL" => Some(Language::Sql),
                "HTML" => Some(Language::Html),
                "CSS" => Some(Language::Css),
                "XML" => Some(Language::Xml),
                "JSON" => Some(Language::Json),
                "YAML" => Some(Language::Yaml),
                "Markdown" => Some(Language::Markdown),
                _ => None,
            })
            .collect()
    }
}

impl LanguageConfig {
    fn new() -> Self {
        let mut tree_sitter_languages = HashMap::new();
        tree_sitter_languages.insert(Language::Rust, "rust");
        tree_sitter_languages.insert(Language::JavaScript, "javascript");
        tree_sitter_languages.insert(Language::TypeScript, "typescript");
        tree_sitter_languages.insert(Language::Python, "python");
        tree_sitter_languages.insert(Language::Java, "java");
        tree_sitter_languages.insert(Language::C, "c");
        tree_sitter_languages.insert(Language::Cpp, "cpp");
        tree_sitter_languages.insert(Language::Go, "go");

        let mut file_extensions = HashMap::new();
        // Rust
        file_extensions.insert("rs".to_string(), Language::Rust);
        // JavaScript/TypeScript
        file_extensions.insert("js".to_string(), Language::JavaScript);
        file_extensions.insert("jsx".to_string(), Language::JavaScript);
        file_extensions.insert("ts".to_string(), Language::TypeScript);
        file_extensions.insert("tsx".to_string(), Language::TypeScript);
        // Python
        file_extensions.insert("py".to_string(), Language::Python);
        file_extensions.insert("pyw".to_string(), Language::Python);
        // Java
        file_extensions.insert("java".to_string(), Language::Java);
        // C/C++
        file_extensions.insert("c".to_string(), Language::C);
        file_extensions.insert("h".to_string(), Language::C);
        file_extensions.insert("cpp".to_string(), Language::Cpp);
        file_extensions.insert("cxx".to_string(), Language::Cpp);
        file_extensions.insert("cc".to_string(), Language::Cpp);
        file_extensions.insert("hpp".to_string(), Language::Cpp);
        // Go
        file_extensions.insert("go".to_string(), Language::Go);
        // Other languages
        file_extensions.insert("php".to_string(), Language::Php);
        file_extensions.insert("rb".to_string(), Language::Ruby);
        file_extensions.insert("cs".to_string(), Language::CSharp);
        file_extensions.insert("swift".to_string(), Language::Swift);
        file_extensions.insert("kt".to_string(), Language::Kotlin);
        file_extensions.insert("scala".to_string(), Language::Scala);
        file_extensions.insert("pl".to_string(), Language::Perl);
        file_extensions.insert("lua".to_string(), Language::Lua);
        file_extensions.insert("sh".to_string(), Language::Shell);
        file_extensions.insert("bash".to_string(), Language::Shell);
        file_extensions.insert("sql".to_string(), Language::Sql);
        file_extensions.insert("html".to_string(), Language::Html);
        file_extensions.insert("css".to_string(), Language::Css);
        file_extensions.insert("xml".to_string(), Language::Xml);
        file_extensions.insert("json".to_string(), Language::Json);
        file_extensions.insert("yml".to_string(), Language::Yaml);
        file_extensions.insert("yaml".to_string(), Language::Yaml);
        file_extensions.insert("md".to_string(), Language::Markdown);

        Self {
            tree_sitter_languages,
            ctags_languages: HashMap::new(),
            file_extensions,
        }
    }
}

pub struct ParseResult {
    pub symbols: Vec<Symbol>,
    pub references: Vec<Reference>,
}