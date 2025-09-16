use crate::types::*;
use crate::language_parser::{UniversalLanguageParser, ParseResult};
use crate::ctags_integration::CtagsIntegration;
use std::path::Path;
use std::collections::HashMap;
use uuid::Uuid;
use tree_sitter::Tree;

/// Enhanced parser that combines Tree-sitter and Universal Ctags
/// for comprehensive language support with real-time incremental parsing
pub struct EnhancedLanguageParser {
    universal_parser: UniversalLanguageParser,
    ctags: Option<CtagsIntegration>,
    parse_cache: HashMap<Uuid, CachedParse>,
    config: ParserConfig,
}

#[derive(Clone)]
pub struct ParserConfig {
    pub prefer_tree_sitter: bool,
    pub fallback_to_ctags: bool,
    pub enable_incremental: bool,
    pub cache_results: bool,
    pub max_file_size: usize,
}

struct CachedParse {
    tree: Option<Tree>,
    symbols: Vec<Symbol>,
    references: Vec<Reference>,
    checksum: String,
    timestamp: std::time::Instant,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            prefer_tree_sitter: true,
            fallback_to_ctags: true,
            enable_incremental: true,
            cache_results: true,
            max_file_size: 1024 * 1024, // 1MB
        }
    }
}

impl EnhancedLanguageParser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let universal_parser = UniversalLanguageParser::new()?;
        
        // Try to initialize ctags, but don't fail if it's not available
        let ctags = match CtagsIntegration::new() {
            Ok(ctags) => {
                log::info!("Universal Ctags initialized successfully");
                Some(ctags)
            }
            Err(e) => {
                log::warn!("Universal Ctags not available: {}. Tree-sitter only mode.", e);
                None
            }
        };

        Ok(Self {
            universal_parser,
            ctags,
            parse_cache: HashMap::new(),
            config: ParserConfig::default(),
        })
    }

    pub fn with_config(mut self, config: ParserConfig) -> Self {
        self.config = config;
        self
    }

    /// Parse a file with automatic language detection and parser selection
    pub fn parse_file(&mut self, file_id: Uuid, path: &Path, content: &str) -> Result<ParseResult, Box<dyn std::error::Error>> {
        // Check file size limit
        if content.len() > self.config.max_file_size {
            return Err(format!("File too large: {} bytes (limit: {})", content.len(), self.config.max_file_size).into());
        }

        // Check cache first
        if self.config.cache_results {
            let checksum = self.calculate_checksum(content);
            if let Some(cached) = self.parse_cache.get(&file_id) {
                if cached.checksum == checksum {
                    return Ok(ParseResult {
                        symbols: cached.symbols.clone(),
                        references: cached.references.clone(),
                    });
                }
            }
        }

        let language = self.detect_language(path)?;
        let result = self.parse_with_strategy(file_id, content, &language, None)?;

        // Cache the result
        if self.config.cache_results {
            self.cache_parse_result(file_id, content, &result, None);
        }

        Ok(result)
    }

    /// Parse with incremental updates (Tree-sitter only)
    pub fn parse_incremental(&mut self, file_id: Uuid, content: &str, language: &Language) -> Result<ParseResult, Box<dyn std::error::Error>> {
        if !self.config.enable_incremental {
            return self.parse_content(file_id, content, language);
        }

        let old_tree = self.parse_cache.get(&file_id)
            .and_then(|cached| cached.tree.as_ref());

        let result = self.universal_parser.parse_incremental(file_id, old_tree, content, language)?;

        // Update cache with new tree
        if self.config.cache_results {
            self.cache_parse_result(file_id, content, &result, None);
        }

        Ok(result)
    }

    /// Parse content with known language
    pub fn parse_content(&mut self, file_id: Uuid, content: &str, language: &Language) -> Result<ParseResult, Box<dyn std::error::Error>> {
        self.parse_with_strategy(file_id, content, language, None)
    }

    fn parse_with_strategy(&mut self, file_id: Uuid, content: &str, language: &Language, old_tree: Option<&Tree>) -> Result<ParseResult, Box<dyn std::error::Error>> {
        // Strategy 1: Try Tree-sitter first (if preferred and available)
        if self.config.prefer_tree_sitter {
            match self.try_tree_sitter_parse(file_id, content, language, old_tree) {
                Ok(result) => {
                    log::debug!("Successfully parsed with Tree-sitter for language: {:?}", language);
                    return Ok(result);
                }
                Err(e) => {
                    log::debug!("Tree-sitter parsing failed for {:?}: {}", language, e);
                }
            }
        }

        // Strategy 2: Fallback to Universal Ctags (if available and enabled)
        if self.config.fallback_to_ctags {
            if let Some(ref ctags) = self.ctags {
                if ctags.supports_language(language) {
                    match self.try_ctags_parse(file_id, content, language, ctags) {
                        Ok(symbols) => {
                            log::debug!("Successfully parsed with Universal Ctags for language: {:?}", language);
                            return Ok(ParseResult {
                                symbols,
                                references: Vec::new(), // Ctags doesn't provide references
                            });
                        }
                        Err(e) => {
                            log::debug!("Ctags parsing failed for {:?}: {}", language, e);
                        }
                    }
                }
            }
        }

        // Strategy 3: Simple regex-based parsing as last resort
        log::debug!("Using simple regex parsing for language: {:?}", language);
        let symbols = self.simple_regex_parse(file_id, content, language);
        Ok(ParseResult {
            symbols,
            references: Vec::new(),
        })
    }

    fn try_tree_sitter_parse(&mut self, file_id: Uuid, content: &str, language: &Language, old_tree: Option<&Tree>) -> Result<ParseResult, Box<dyn std::error::Error>> {
        if let Some(old_tree) = old_tree {
            self.universal_parser.parse_incremental(file_id, Some(old_tree), content, language)
        } else {
            self.universal_parser.parse_file(file_id, Path::new(""), content)
        }
    }

    fn try_ctags_parse(&self, file_id: Uuid, content: &str, language: &Language, ctags: &CtagsIntegration) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        ctags.parse_content(file_id, content, language)
    }

    fn simple_regex_parse(&self, file_id: Uuid, content: &str, language: &Language) -> Vec<Symbol> {
        use regex::Regex;
        use once_cell::sync::Lazy;

        let mut symbols = Vec::new();

        // Language-specific regex patterns
        match language {
            Language::Rust => {
                static RUST_PATTERNS: Lazy<Vec<(Regex, SymbolType)>> = Lazy::new(|| vec![
                    (Regex::new(r"fn\s+(\w+)\s*\(").unwrap(), SymbolType::Function),
                    (Regex::new(r"struct\s+(\w+)").unwrap(), SymbolType::Struct),
                    (Regex::new(r"enum\s+(\w+)").unwrap(), SymbolType::Enum),
                    (Regex::new(r"trait\s+(\w+)").unwrap(), SymbolType::Interface),
                    (Regex::new(r"impl\s+(\w+)").unwrap(), SymbolType::Class),
                    (Regex::new(r"const\s+(\w+)").unwrap(), SymbolType::Constant),
                    (Regex::new(r"static\s+(\w+)").unwrap(), SymbolType::Variable),
                ]);
                self.extract_with_patterns(file_id, content, &RUST_PATTERNS, &mut symbols);
            }

            Language::JavaScript | Language::TypeScript => {
                static JS_PATTERNS: Lazy<Vec<(Regex, SymbolType)>> = Lazy::new(|| vec![
                    (Regex::new(r"function\s+(\w+)\s*\(").unwrap(), SymbolType::Function),
                    (Regex::new(r"class\s+(\w+)").unwrap(), SymbolType::Class),
                    (Regex::new(r"interface\s+(\w+)").unwrap(), SymbolType::Interface),
                    (Regex::new(r"const\s+(\w+)\s*=").unwrap(), SymbolType::Constant),
                    (Regex::new(r"let\s+(\w+)\s*=").unwrap(), SymbolType::Variable),
                    (Regex::new(r"var\s+(\w+)\s*=").unwrap(), SymbolType::Variable),
                ]);
                self.extract_with_patterns(file_id, content, &JS_PATTERNS, &mut symbols);
            }

            Language::Python => {
                static PYTHON_PATTERNS: Lazy<Vec<(Regex, SymbolType)>> = Lazy::new(|| vec![
                    (Regex::new(r"def\s+(\w+)\s*\(").unwrap(), SymbolType::Function),
                    (Regex::new(r"class\s+(\w+)").unwrap(), SymbolType::Class),
                    (Regex::new(r"^(\w+)\s*=").unwrap(), SymbolType::Variable),
                ]);
                self.extract_with_patterns(file_id, content, &PYTHON_PATTERNS, &mut symbols);
            }

            Language::Java => {
                static JAVA_PATTERNS: Lazy<Vec<(Regex, SymbolType)>> = Lazy::new(|| vec![
                    (Regex::new(r"public\s+(?:static\s+)?(?:\w+\s+)?(\w+)\s*\(").unwrap(), SymbolType::Method),
                    (Regex::new(r"private\s+(?:static\s+)?(?:\w+\s+)?(\w+)\s*\(").unwrap(), SymbolType::Method),
                    (Regex::new(r"class\s+(\w+)").unwrap(), SymbolType::Class),
                    (Regex::new(r"interface\s+(\w+)").unwrap(), SymbolType::Interface),
                    (Regex::new(r"enum\s+(\w+)").unwrap(), SymbolType::Enum),
                ]);
                self.extract_with_patterns(file_id, content, &JAVA_PATTERNS, &mut symbols);
            }

            Language::C | Language::Cpp => {
                static C_PATTERNS: Lazy<Vec<(Regex, SymbolType)>> = Lazy::new(|| vec![
                    (Regex::new(r"(?:static\s+)?(?:\w+\s+)+(\w+)\s*\([^)]*\)\s*\{").unwrap(), SymbolType::Function),
                    (Regex::new(r"struct\s+(\w+)").unwrap(), SymbolType::Struct),
                    (Regex::new(r"enum\s+(\w+)").unwrap(), SymbolType::Enum),
                    (Regex::new(r"typedef\s+(?:struct\s+)?(?:\w+\s+)?(\w+)").unwrap(), SymbolType::Type),
                    (Regex::new(r"#define\s+(\w+)").unwrap(), SymbolType::Macro),
                ]);
                self.extract_with_patterns(file_id, content, &C_PATTERNS, &mut symbols);
            }

            _ => {
                // Generic patterns for unknown languages
                static GENERIC_PATTERNS: Lazy<Vec<(Regex, SymbolType)>> = Lazy::new(|| vec![
                    (Regex::new(r"function\s+(\w+)").unwrap(), SymbolType::Function),
                    (Regex::new(r"class\s+(\w+)").unwrap(), SymbolType::Class),
                    (Regex::new(r"def\s+(\w+)").unwrap(), SymbolType::Function),
                    (Regex::new(r"fn\s+(\w+)").unwrap(), SymbolType::Function),
                ]);
                self.extract_with_patterns(file_id, content, &GENERIC_PATTERNS, &mut symbols);
            }
        }

        symbols
    }

    fn extract_with_patterns(&self, file_id: Uuid, content: &str, patterns: &[(Regex, SymbolType)], symbols: &mut Vec<Symbol>) {
        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num as u32 + 1;
            
            for (pattern, symbol_type) in patterns {
                if let Some(captures) = pattern.captures(line) {
                    if let Some(name_match) = captures.get(1) {
                        let symbol = Symbol {
                            id: Uuid::new_v4(),
                            file_id,
                            name: name_match.as_str().to_string(),
                            symbol_type: *symbol_type,
                            line: line_num,
                            column: name_match.start() as u32,
                            end_line: line_num,
                            end_column: name_match.end() as u32,
                            signature: Some(line.trim().to_string()),
                            scope: None,
                            namespace: None,
                        };
                        symbols.push(symbol);
                    }
                }
            }
        }
    }

    fn detect_language(&self, path: &Path) -> Result<Language, Box<dyn std::error::Error>> {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Ok(Language::from_extension(ext))
        } else if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            match filename {
                "Dockerfile" => Ok(Language::Dockerfile),
                "Makefile" | "makefile" => Ok(Language::Make),
                "CMakeLists.txt" => Ok(Language::CMake),
                _ => Ok(Language::Text),
            }
        } else {
            Ok(Language::Unknown)
        }
    }

    fn calculate_checksum(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn cache_parse_result(&mut self, file_id: Uuid, content: &str, result: &ParseResult, tree: Option<Tree>) {
        let checksum = self.calculate_checksum(content);
        let cached = CachedParse {
            tree,
            symbols: result.symbols.clone(),
            references: result.references.clone(),
            checksum,
            timestamp: std::time::Instant::now(),
        };
        self.parse_cache.insert(file_id, cached);
    }

    pub fn get_supported_languages(&self) -> Vec<Language> {
        let mut languages = self.universal_parser.get_supported_languages();
        
        if let Some(ref ctags) = self.ctags {
            // Add ctags-supported languages that aren't in Tree-sitter
            for lang_name in ctags.get_supported_languages() {
                if let Some(language) = self.map_ctags_language(lang_name) {
                    if !languages.contains(&language) {
                        languages.push(language);
                    }
                }
            }
        }

        languages.sort();
        languages.dedup();
        languages
    }

    fn map_ctags_language(&self, ctags_lang: &str) -> Option<Language> {
        match ctags_lang {
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
            "Sh" => Some(Language::Shell),
            "SQL" => Some(Language::Sql),
            "HTML" => Some(Language::Html),
            "CSS" => Some(Language::Css),
            "XML" => Some(Language::Xml),
            "JSON" => Some(Language::Json),
            "YAML" => Some(Language::Yaml),
            "Markdown" => Some(Language::Markdown),
            _ => None,
        }
    }

    pub fn clear_cache(&mut self) {
        self.parse_cache.clear();
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            entries: self.parse_cache.len(),
            memory_usage: std::mem::size_of_val(&self.parse_cache),
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub entries: usize,
    pub memory_usage: usize,
}