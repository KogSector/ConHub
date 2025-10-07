use crate::types::*;
use crate::utils::*;

use tantivy::{Index, IndexReader, Searcher, Document, Score, TantivyDocument};
use tantivy::query::{QueryParser, Query, BooleanQuery, Occur, TermQuery, FuzzyTermQuery, RegexQuery};
use tantivy::collector::TopDocs;
use tantivy::schema::*;
use tantivy::Term;

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use log::debug;
use dashmap::DashMap;

/// Unified search engine that combines functionality from search.rs, search_optimizations.rs, and search_simple.rs
pub struct SearchEngine {
    index: Index,
    reader: IndexReader,
    schema: Schema,
    query_parser: QueryParser,
    optimizations: SearchOptimizations,
}

impl SearchEngine {
    pub fn new(index: Index, schema: Schema) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = index.reader()?;
        
        // Create query parser for different fields
        let mut query_parser = QueryParser::for_index(&index, vec![
            schema.get_field("content").unwrap(),
            schema.get_field("symbol_name").unwrap(),
            schema.get_field("file_path").unwrap(),
            schema.get_field("file_name").unwrap(),
        ]);
        
        query_parser.set_field_boost(schema.get_field("symbol_name").unwrap(), 3.0);
        query_parser.set_field_boost(schema.get_field("file_name").unwrap(), 2.0);
        
        Ok(Self {
            index,
            reader,
            schema,
            query_parser,
            optimizations: SearchOptimizations::new(),
        })
    }

    pub fn search(&self, query: &SearchQuery) -> Result<SearchResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let searcher = self.reader.searcher();
        
        let tantivy_query = self.build_query(query)?;
        
        // Execute search
        let top_docs = TopDocs::with_limit(query.limit).and_offset(query.offset);
        let search_results = searcher.search(&tantivy_query, &top_docs)?;
        
        // Collect facets if needed
        let facets = self.collect_facets(&searcher, &tantivy_query)?;
        
        // Convert results
        let mut hits = Vec::new();
        for (score, doc_address) in search_results {
            let doc = searcher.doc(doc_address)?;
            if let Some(hit) = self.convert_document_to_hit(&doc, score, query)? {
                hits.push(hit);
            }
        }
        
        let query_time = start_time.elapsed().as_millis() as u64;
        
        Ok(SearchResult {
            total_hits: hits.len(), // TODO: Get actual total count
            results: hits,
            facets,
            query_time_ms: query_time,
        })
    }

    fn build_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        match query.query_type {
            QueryType::FullText => self.build_fulltext_query(query),
            QueryType::Symbol => self.build_symbol_query(query),
            QueryType::Path => self.build_path_query(query),
            QueryType::Definition => self.build_definition_query(query),
            QueryType::Reference => self.build_reference_query(query),
            QueryType::History => self.build_history_query(query),
        }
    }

    fn build_fulltext_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let mut clauses = Vec::new();
        
        // Main content query
        let content_query = if query.regex {
            self.build_regex_query(&query.query)?
        } else if query.query.contains('*') || query.query.contains('?') {
            self.build_wildcard_query(&query.query)?
        } else {
            self.query_parser.parse_query(&query.query)?
        };
        
        clauses.push((Occur::Must, content_query));
        
        // Apply filters
        if let Some(ref languages) = query.languages {
            if !languages.is_empty() {
                let language_field = self.schema.get_field("language").unwrap();
                let mut language_clauses = Vec::new();
                
                for language in languages {
                    let language_str = format!("{:?}", language);
                    let term = Term::from_field_text(language_field, &language_str);
                    language_clauses.push((Occur::Should, Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>));
                }
                
                if !language_clauses.is_empty() {
                    clauses.push((Occur::Must, Box::new(BooleanQuery::from(language_clauses))));
                }
            }
        }
        
        if let Some(ref file_types) = query.file_types {
            if !file_types.is_empty() {
                let file_type_field = self.schema.get_field("file_type").unwrap();
                let mut file_type_clauses = Vec::new();
                
                for file_type in file_types {
                    let file_type_str = format!("{:?}", file_type);
                    let term = Term::from_field_text(file_type_field, &file_type_str);
                    file_type_clauses.push((Occur::Should, Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>));
                }
                
                if !file_type_clauses.is_empty() {
                    clauses.push((Occur::Must, Box::new(BooleanQuery::from(file_type_clauses))));
                }
            }
        }
        
        // Create boolean query from all clauses
        Ok(Box::new(BooleanQuery::from(clauses)))
    }

    fn build_symbol_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        // Use optimizations for symbol queries
        let symbol_files = self.optimizations.find_symbol_files(&query.query);
        
        if !symbol_files.is_empty() {
            let mut clauses = Vec::new();
            let file_id_field = self.schema.get_field("file_id").unwrap();
            
            for (file_id, _) in symbol_files {
                let term = Term::from_field_text(file_id_field, &file_id.to_string());
                clauses.push((Occur::Should, Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>));
            }
            
            Ok(Box::new(BooleanQuery::from(clauses)))
        } else {
            // Fallback to standard query
            let symbol_field = self.schema.get_field("symbol_name").unwrap();
            let term = Term::from_field_text(symbol_field, &query.query);
            Ok(Box::new(TermQuery::new(term, IndexRecordOption::Basic)))
        }
    }

    fn build_path_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let path_field = self.schema.get_field("file_path").unwrap();
        let term = Term::from_field_text(path_field, &query.query);
        Ok(Box::new(TermQuery::new(term, IndexRecordOption::Basic)))
    }

    fn build_definition_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let symbol_field = self.schema.get_field("symbol_name").unwrap();
        let is_definition_field = self.schema.get_field("is_definition").unwrap();
        
        let mut clauses = Vec::new();
        
        // Symbol name clause
        let term = Term::from_field_text(symbol_field, &query.query);
        clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>));
        
        // Is definition clause
        let term = Term::from_field_text(is_definition_field, "true");
        clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>));
        
        Ok(Box::new(BooleanQuery::from(clauses)))
    }

    fn build_reference_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let symbol_field = self.schema.get_field("symbol_name").unwrap();
        let is_reference_field = self.schema.get_field("is_reference").unwrap();
        
        let mut clauses = Vec::new();
        
        // Symbol name clause
        let term = Term::from_field_text(symbol_field, &query.query);
        clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>));
        
        // Is reference clause
        let term = Term::from_field_text(is_reference_field, "true");
        clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>));
        
        Ok(Box::new(BooleanQuery::from(clauses)))
    }

    fn build_history_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let path_field = self.schema.get_field("file_path").unwrap();
        let _timestamp_field = self.schema.get_field("timestamp").unwrap();
        
        let mut clauses = Vec::new();
        
        // Path clause
        let term = Term::from_field_text(path_field, &query.query);
        clauses.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>));
        
        // TODO: Add timestamp range if needed
        
        Ok(Box::new(BooleanQuery::from(clauses)))
    }

    fn build_regex_query(&self, regex_str: &str) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let content_field = self.schema.get_field("content").unwrap();
        Ok(Box::new(RegexQuery::from_pattern(regex_str, content_field)?))
    }

    fn build_wildcard_query(&self, wildcard_str: &str) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        // Convert wildcard to regex
        let regex_str = wildcard_str
            .replace("*", ".*")
            .replace("?", ".");
        
        self.build_regex_query(&regex_str)
    }

    fn collect_facets(&self, searcher: &Searcher, query: &dyn Query) -> Result<HashMap<String, Vec<FacetValue>>, Box<dyn std::error::Error>> {
        let mut facets = HashMap::new();
        
        // Collect language facets
        let language_field = self.schema.get_field("language").unwrap();
        let language_facets = self.collect_field_facets(searcher, query, language_field)?;
        
        // Convert HashMap<String, u64> to Vec<FacetValue>
        let language_facet_values = language_facets.into_iter()
            .map(|(value, count)| FacetValue { value, count: count as usize })
            .collect();
        
        facets.insert("language".to_string(), language_facet_values);
        
        // Collect file type facets
        let file_type_field = self.schema.get_field("file_type").unwrap();
        let file_type_facets = self.collect_field_facets(searcher, query, file_type_field)?;
        
        // Convert HashMap<String, u64> to Vec<FacetValue>
        let file_type_facet_values = file_type_facets.into_iter()
            .map(|(value, count)| FacetValue { value, count: count as usize })
            .collect();
        
        facets.insert("file_type".to_string(), file_type_facet_values);
        
        Ok(facets)
    }

    fn collect_field_facets(&self, _searcher: &Searcher, _query: &dyn Query, _field: Field) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
        let facets = HashMap::new();
        
        // TODO: Implement proper facet collection
        
        Ok(facets)
    }

    fn convert_document_to_hit(&self, doc: &TantivyDocument, score: Score, _query: &SearchQuery) -> Result<Option<SearchHit>, Box<dyn std::error::Error>> {
        // Extract file information
        let file_id = doc.get_first(self.schema.get_field("file_id").unwrap())
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        let project_id = doc.get_first(self.schema.get_field("project_id").unwrap())
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        if file_id.is_none() || project_id.is_none() {
            return Ok(None);
        }
        
        let file_path = doc.get_first(self.schema.get_field("file_path").unwrap())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let language_str = doc.get_first(self.schema.get_field("language").unwrap())
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        
        let file_type_str = doc.get_first(self.schema.get_field("file_type").unwrap())
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        
        let file_size = doc.get_first(self.schema.get_field("file_size").unwrap())
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        // Extract content snippets
        let content = doc.get_first(self.schema.get_field("content").unwrap())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let snippets = extract_snippets(&content, 3, 150);
        
        // Create an IndexedFile struct
        let indexed_file = IndexedFile {
            id: file_id.unwrap(),
            project_id: project_id.unwrap(),
            path: std::path::PathBuf::from(&file_path),
            relative_path: std::path::PathBuf::from(&file_path), // Using same path for now
            file_type: match file_type_str {
                "Source" => FileType::Source,
                "Documentation" => FileType::Documentation,
                "Configuration" => FileType::Configuration,
                "Data" => FileType::Data,
                "Binary" => FileType::Binary,
                "Archive" => FileType::Archive,
                "Image" => FileType::Image,
                _ => FileType::Unknown,
            },
            language: match language_str {
                "Rust" => Language::Rust,
                "JavaScript" => Language::JavaScript,
                "TypeScript" => Language::TypeScript,
                "Python" => Language::Python,
                "Java" => Language::Java,
                "C" => Language::C,
                "Cpp" => Language::Cpp,
                "Go" => Language::Go,
                "Php" => Language::Php,
                "Ruby" => Language::Ruby,
                "CSharp" => Language::CSharp,
                "Swift" => Language::Swift,
                "Kotlin" => Language::Kotlin,
                "Scala" => Language::Scala,
                "Perl" => Language::Perl,
                "Lua" => Language::Lua,
                "Html" => Language::Html,
                "Css" => Language::Css,
                "Json" => Language::Json,
                "Xml" => Language::Xml,
                "Yaml" => Language::Yaml,
                "Toml" => Language::Toml,
                "Markdown" => Language::Markdown,
                "Shell" => Language::Shell,
                "Sql" => Language::Sql,
                "Dockerfile" => Language::Dockerfile,
                "Make" => Language::Make,
                "CMake" => Language::CMake,
                "Text" => Language::Text,
                _ => Language::Unknown,
            },
            size: file_size,
            lines: 0, // Default value
            last_modified: chrono::Utc::now(), // Default value
            last_indexed: chrono::Utc::now(), // Default value
            checksum: "".to_string(), // Default value
            encoding: "utf-8".to_string(), // Default value
        };
        
        // Create line matches from snippets
        let line_matches = snippets.iter().enumerate().map(|(i, content)| {
            LineMatch {
                line_number: (i + 1) as u32,
                content: content.clone(),
                highlights: Vec::new(), // No highlights for now
            }
        }).collect();
        
        Ok(Some(SearchHit {
            file: indexed_file,
            score: score as f32,
            highlights: Vec::new(), // TODO: Implement highlighting
            symbols: Vec::new(), // No symbols for now
            line_matches,
        }))
    }

    // Add a symbol to the search optimizations
    pub fn index_symbol(&self, symbol_name: &str, file_id: Uuid) {
        self.optimizations.index_symbol(symbol_name, file_id);
    }

    // Add a file extension to the search optimizations
    pub fn index_file_extension(&self, extension: &str, file_id: Uuid) {
        self.optimizations.index_file_extension(extension, file_id);
    }

    // Perform a fuzzy search for symbols
    pub fn fuzzy_search_symbols(&self, query: &str, max_results: usize) -> Vec<(String, u32, f64)> {
        self.optimizations.fuzzy_find_symbols(query, max_results)
    }
}

/// High-performance search optimization structures
#[derive(Debug, Clone)]
pub struct SearchOptimizations {
    // Symbol name to file mappings for fast symbol lookup
    symbol_index: Arc<DashMap<String, Vec<Uuid>>>, // symbol_name -> [file_ids]
    
    // File extension to file mappings for language-specific searches
    extension_index: Arc<DashMap<String, Vec<Uuid>>>, // extension -> [file_ids]
    
    // Word frequency index for relevance scoring
    word_frequency: Arc<DashMap<String, u32>>,
    
    // Recent access cache for hot paths
    access_cache: Arc<DashMap<String, (Uuid, std::time::Instant)>>, // key -> (value, last_access)
}

impl SearchOptimizations {
    pub fn new() -> Self {
        Self {
            symbol_index: Arc::new(DashMap::new()),
            extension_index: Arc::new(DashMap::new()),
            word_frequency: Arc::new(DashMap::new()),
            access_cache: Arc::new(DashMap::new()),
        }
    }

    /// Add a symbol to the search index
    pub fn index_symbol(&self, symbol_name: &str, file_id: Uuid) {
        let mut entry = self.symbol_index.entry(symbol_name.to_lowercase()).or_insert(Vec::new());
        if !entry.contains(&file_id) {
            entry.push(file_id);
        }
        
        // Update word frequency
        let mut freq = self.word_frequency.entry(symbol_name.to_lowercase()).or_insert(0);
        *freq += 1;
    }

    /// Add a file to the extension index
    pub fn index_file_extension(&self, extension: &str, file_id: Uuid) {
        let mut entry = self.extension_index.entry(extension.to_lowercase()).or_insert(Vec::new());
        if !entry.contains(&file_id) {
            entry.push(file_id);
        }
    }

    /// Fast symbol lookup with ranking by frequency
    pub fn find_symbol_files(&self, symbol_name: &str) -> Vec<(Uuid, u32)> {
        let key = symbol_name.to_lowercase();
        
        // Check cache first
        if let Some(entry) = self.access_cache.get(&key) {
            let (file_id, last_access) = entry.value();
            if last_access.elapsed().as_secs() < 300 { // 5 minute cache
                debug!("Cache hit for symbol: {}", symbol_name);
                return vec![(*file_id, 100)]; // High score for cached items
            }
        }

        if let Some(file_ids) = self.symbol_index.get(&key) {
            let frequency = self.word_frequency.get(&key).map(|f| *f.value()).unwrap_or(1);
            
            let results: Vec<(Uuid, u32)> = file_ids.iter()
                .map(|&file_id| (file_id, frequency))
                .collect();
            
            // Cache the first result for hot paths
            if let Some(&(first_file_id, _)) = results.first() {
                self.access_cache.insert(key, (first_file_id, std::time::Instant::now()));
            }
            
            results
        } else {
            Vec::new()
        }
    }

    /// Find files by extension with performance optimization
    pub fn find_files_by_extension(&self, extension: &str) -> Vec<Uuid> {
        let key = extension.to_lowercase();
        
        if let Some(file_ids) = self.extension_index.get(&key) {
            file_ids.clone()
        } else {
            Vec::new()
        }
    }

    /// Optimized fuzzy search for symbols with similarity ranking
    /// Uses prefix matching and early termination for better performance
    pub fn fuzzy_find_symbols(&self, query: &str, max_results: usize) -> Vec<(String, u32, f64)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::with_capacity(max_results * 2);
        
        // First pass: exact prefix matches (very fast)
        let prefix_matches: Vec<_> = self.symbol_index.iter()
            .filter(|entry| entry.key().starts_with(&query_lower))
            .map(|entry| {
                let symbol = entry.key();
                let frequency = self.word_frequency.get(symbol).map(|f| *f.value()).unwrap_or(1);
                (symbol.clone(), frequency, 1.0) // Perfect match score
            })
            .collect();
        
        // If we have enough prefix matches, skip the expensive fuzzy search
        if prefix_matches.len() >= max_results {
            let mut sorted_matches = prefix_matches;
            sorted_matches.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by frequency
            sorted_matches.truncate(max_results);
            return sorted_matches;
        }
        
        // Add prefix matches to results
        results.extend(prefix_matches);
        
        // Second pass: fuzzy matching only if needed
        // Use a minimum length threshold to avoid expensive comparisons on very short strings
        if query_lower.len() >= 3 {
            // Calculate how many more results we need
            let remaining_slots = max_results - results.len();
            
            if remaining_slots > 0 {
                let mut fuzzy_matches = Vec::with_capacity(remaining_slots * 2);
                
                // Only process symbols that might be relevant (length similarity heuristic)
                for entry in self.symbol_index.iter() {
                    let symbol = entry.key();
                    
                    // Skip symbols we already matched by prefix
                    if symbol.starts_with(&query_lower) {
                        continue;
                    }
                    
                    // Length-based early termination heuristic
                    let len_ratio = symbol.len() as f64 / query_lower.len() as f64;
                    if len_ratio < 0.5 || len_ratio > 2.0 {
                        continue; // Skip if length difference is too large
                    }
                    
                    let similarity = calculate_similarity(&query_lower, symbol);
                    
                    if similarity > 0.6 { // Threshold for fuzzy matching
                        let frequency = self.word_frequency.get(symbol).map(|f| *f.value()).unwrap_or(1);
                        fuzzy_matches.push((symbol.clone(), frequency, similarity));
                    }
                }
                
                // Sort fuzzy matches by combined score
                fuzzy_matches.sort_by(|a, b| {
                    let combined_a = a.2 * (a.1 as f64).log10();
                    let combined_b = b.2 * (b.1 as f64).log10();
                    combined_b.partial_cmp(&combined_a).unwrap_or(std::cmp::Ordering::Equal)
                });
                
                // Take only what we need
                fuzzy_matches.truncate(remaining_slots);
                results.extend(fuzzy_matches);
            }
        }
        
        results
    }
}

// Optimized helper function to calculate string similarity for fuzzy matching
// Using Wagner-Fischer algorithm with space optimization (O(min(m,n)) space instead of O(m*n))
fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }
    
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }
    
    // Simple Levenshtein distance implementation with space optimization
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    
    let s1_len = s1_chars.len();
    let s2_len = s2_chars.len();
    
    // Ensure s1 is the shorter string to optimize space
    if s1_len > s2_len {
        return calculate_similarity(s2, s1);
    }
    
    // Use a single vector instead of a matrix (space optimization)
    let mut prev_row = (0..=s1_len).collect::<Vec<usize>>();
    let mut curr_row = vec![0; s1_len + 1];
    
    for j in 1..=s2_len {
        curr_row[0] = j;
        
        for i in 1..=s1_len {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            
            curr_row[i] = std::cmp::min(
                std::cmp::min(
                    prev_row[i] + 1,        // deletion
                    curr_row[i-1] + 1       // insertion
                ),
                prev_row[i-1] + cost        // substitution
            );
        }
        
        // Swap rows for next iteration
        std::mem::swap(&mut prev_row, &mut curr_row);
    }
    
    let distance = prev_row[s1_len] as f64;
    let max_len = std::cmp::max(s1_len, s2_len) as f64;
    
    1.0 - (distance / max_len)
}

// Optimized helper function to extract content snippets
fn extract_snippets(content: &str, count: usize, length: usize) -> Vec<String> {
    // Early return for empty content
    if content.is_empty() {
        return Vec::new();
    }
    
    // Use an iterator approach instead of collecting all lines first
    let line_count = content.lines().count();
    
    if line_count <= count {
        // For small content, just return all lines with length check
        return content.lines()
            .map(|line| {
                if line.len() <= length {
                    line.to_string()
                } else {
                    // Use char indices to avoid UTF-8 boundary issues
                    let truncated = match line.char_indices().nth(length) {
                        Some((idx, _)) => &line[..idx],
                        None => line,
                    };
                    format!("{truncated}...")
                }
            })
            .collect();
    }
    
    // For larger content, sample evenly distributed lines
    let step = line_count / count;
    let mut snippets = Vec::with_capacity(count);
    let mut line_iter = content.lines();
    
    for i in 0..count {
        // Skip to the next position
        let _target_idx = i * step;
        
        // Skip lines until we reach the target index
        if i > 0 {
            // Skip (step-1) lines to reach the next sample point
            for _ in 0..(step-1) {
                line_iter.next();
            }
        }
        
        // Get the line at the target position
        if let Some(line) = line_iter.next() {
            if line.len() <= length {
                snippets.push(line.to_string());
            } else {
                // Use char indices to avoid UTF-8 boundary issues
                let truncated = match line.char_indices().nth(length) {
                    Some((idx, _)) => &line[..idx],
                    None => line,
                };
                snippets.push(format!("{truncated}..."));
            }
        }
    }
    
    snippets
}