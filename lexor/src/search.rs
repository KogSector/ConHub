use crate::types::*;
use crate::utils::*;

use tantivy::{Index, IndexReader, Searcher, Document, Score};
use tantivy::query::{QueryParser, Query, BooleanQuery, Occur, TermQuery, FuzzyTermQuery, RegexQuery};
use tantivy::collector::TopDocs;
use tantivy::schema::*;
use tantivy::Term;

use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;
use uuid::Uuid;
use log::{debug, warn, error};

pub struct SearchEngine {
    index: Index,
    reader: IndexReader,
    schema: Schema,
    query_parser: QueryParser,
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
            let doc: tantivy::Document = searcher.doc(doc_address)?;
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
        
        // Add filters
        self.add_filters(&mut clauses, query)?;
        
        Ok(Box::new(BooleanQuery::new(clauses)))
    }

    fn build_symbol_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let symbol_name_field = self.schema.get_field("symbol_name").unwrap();
        
        let symbol_query: Box<dyn Query> = if query.case_sensitive {
            Box::new(TermQuery::new(
                Term::from_field_text(symbol_name_field, &query.query),
                IndexRecordOption::Basic,
            ))
        } else {
            Box::new(FuzzyTermQuery::new(
                Term::from_field_text(symbol_name_field, &query.query.to_lowercase()),
                2,
                true,
            ))
        };
        
        let mut clauses = Vec::new();
        clauses.push((Occur::Must, symbol_query));
        self.add_filters(&mut clauses, query)?;
        
        Ok(Box::new(BooleanQuery::new(clauses)))
    }

    fn build_path_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let file_path_field = self.schema.get_field("file_path").unwrap();
        let file_name_field = self.schema.get_field("file_name").unwrap();
        
        let mut clauses = Vec::new();
        
        // Search in both file path and file name
        let path_query = Box::new(TermQuery::new(
            Term::from_field_text(file_path_field, &query.query),
            IndexRecordOption::Basic,
        ));
        
        let name_query = Box::new(TermQuery::new(
            Term::from_field_text(file_name_field, &query.query),
            IndexRecordOption::Basic,
        ));
        
        let path_clauses = vec![
            (Occur::Should, path_query),
            (Occur::Should, name_query),
        ];
        
        clauses.push((Occur::Must, Box::new(BooleanQuery::new(path_clauses))));
        self.add_filters(&mut clauses, query)?;
        
        Ok(Box::new(BooleanQuery::new(clauses)))
    }

    fn build_definition_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let symbol_name_field = self.schema.get_field("symbol_name").unwrap();
        let reference_type_field = self.schema.get_field("reference_type").unwrap();
        
        let mut clauses = Vec::new();
        
        // Symbol name query
        let symbol_query = Box::new(TermQuery::new(
            Term::from_field_text(symbol_name_field, &query.query),
            IndexRecordOption::Basic,
        ));
        clauses.push((Occur::Must, symbol_query));
        
        // Filter for definitions only
        let def_query = Box::new(TermQuery::new(
            Term::from_field_text(reference_type_field, "Definition"),
            IndexRecordOption::Basic,
        ));
        clauses.push((Occur::Must, def_query));
        
        self.add_filters(&mut clauses, query)?;
        
        Ok(Box::new(BooleanQuery::new(clauses)))
    }

    fn build_reference_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let symbol_name_field = self.schema.get_field("symbol_name").unwrap();
        let reference_type_field = self.schema.get_field("reference_type").unwrap();
        
        let mut clauses = Vec::new();
        
        // Symbol name query
        let symbol_query = Box::new(TermQuery::new(
            Term::from_field_text(symbol_name_field, &query.query),
            IndexRecordOption::Basic,
        ));
        clauses.push((Occur::Must, symbol_query));
        
        // Filter for references (not definitions)
        let ref_query = Box::new(TermQuery::new(
            Term::from_field_text(reference_type_field, "Usage"),
            IndexRecordOption::Basic,
        ));
        clauses.push((Occur::Must, ref_query));
        
        self.add_filters(&mut clauses, query)?;
        
        Ok(Box::new(BooleanQuery::new(clauses)))
    }

    fn build_history_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let commit_message_field = self.schema.get_field("commit_message").unwrap();
        let author_field = self.schema.get_field("author").unwrap();
        
        let mut clauses = Vec::new();
        
        // Search in commit messages and author
        let message_query = self.query_parser.parse_query(&query.query)?;
        clauses.push((Occur::Should, message_query));
        
        let author_query = Box::new(TermQuery::new(
            Term::from_field_text(author_field, &query.query),
            IndexRecordOption::Basic,
        ));
        clauses.push((Occur::Should, author_query));
        
        self.add_filters(&mut clauses, query)?;
        
        Ok(Box::new(BooleanQuery::new(clauses)))
    }

    fn build_regex_query(&self, pattern: &str) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        let content_field = self.schema.get_field("content").unwrap();
        Ok(Box::new(RegexQuery::from_pattern(pattern, content_field)?))
    }

    fn build_wildcard_query(&self, pattern: &str) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        // Convert wildcard to regex
        let regex_pattern = pattern
            .replace("*", ".*")
            .replace("?", ".");
        
        self.build_regex_query(&regex_pattern)
    }

    fn add_filters(&self, clauses: &mut Vec<(Occur, Box<dyn Query>)>, query: &SearchQuery) -> Result<(), Box<dyn std::error::Error>> {
        // Project filter
        if let Some(projects) = &query.projects {
            if !projects.is_empty() {
                let project_field = self.schema.get_field("project_id").unwrap();
                let mut project_clauses = Vec::new();
                
                for project_id in projects {
                    let term_query = Box::new(TermQuery::new(
                        Term::from_field_text(project_field, &project_id.to_string()),
                        IndexRecordOption::Basic,
                    ));
                    project_clauses.push((Occur::Should, term_query));
                }
                
                clauses.push((Occur::Must, Box::new(BooleanQuery::new(project_clauses))));
            }
        }
        
        // Language filter
        if let Some(languages) = &query.languages {
            if !languages.is_empty() {
                let language_field = self.schema.get_field("language").unwrap();
                let mut language_clauses = Vec::new();
                
                for language in languages {
                    let term_query = Box::new(TermQuery::new(
                        Term::from_field_text(language_field, &format!("{:?}", language)),
                        IndexRecordOption::Basic,
                    ));
                    language_clauses.push((Occur::Should, term_query));
                }
                
                clauses.push((Occur::Must, Box::new(BooleanQuery::new(language_clauses))));
            }
        }
        
        // File type filter
        if let Some(file_types) = &query.file_types {
            if !file_types.is_empty() {
                let file_type_field = self.schema.get_field("file_type").unwrap();
                let mut file_type_clauses = Vec::new();
                
                for file_type in file_types {
                    let term_query = Box::new(TermQuery::new(
                        Term::from_field_text(file_type_field, &format!("{:?}", file_type)),
                        IndexRecordOption::Basic,
                    ));
                    file_type_clauses.push((Occur::Should, term_query));
                }
                
                clauses.push((Occur::Must, Box::new(BooleanQuery::new(file_type_clauses))));
            }
        }
        
        // Path filter
        if let Some(path_filter) = &query.path_filter {
            let file_path_field = self.schema.get_field("file_path").unwrap();
            let path_query = Box::new(TermQuery::new(
                Term::from_field_text(file_path_field, path_filter),
                IndexRecordOption::Basic,
            ));
            clauses.push((Occur::Must, path_query));
        }
        
        Ok(())
    }

    fn collect_facets(&self, searcher: &Searcher, query: &Box<dyn Query>) -> Result<HashMap<String, Vec<FacetValue>>, Box<dyn std::error::Error>> {
        let mut facets = HashMap::new();
        
        // Language facets
        if let Ok(language_field) = self.schema.get_field("language") {
            let language_facets = self.collect_field_facets(searcher, query, language_field, "language")?;
            facets.insert("language".to_string(), language_facets);
        }
        
        // File type facets
        if let Ok(file_type_field) = self.schema.get_field("file_type") {
            let file_type_facets = self.collect_field_facets(searcher, query, file_type_field, "file_type")?;
            facets.insert("file_type".to_string(), file_type_facets);
        }
        
        // Project facets
        if let Ok(project_field) = self.schema.get_field("project_name") {
            let project_facets = self.collect_field_facets(searcher, query, project_field, "project")?;
            facets.insert("project".to_string(), project_facets);
        }
        
        Ok(facets)
    }

    fn collect_field_facets(&self, searcher: &Searcher, query: &Box<dyn Query>, field: Field, _field_name: &str) -> Result<Vec<FacetValue>, Box<dyn std::error::Error>> {
        // Simplified facet collection - in a real implementation, you'd use Tantivy's facet collectors
        let mut facet_counts = HashMap::new();
        
        let top_docs = TopDocs::with_limit(1000);
        let search_results = searcher.search(query, &top_docs)?;
        
        for (_score, doc_address) in search_results {
            let doc = searcher.doc(doc_address)?;
            if let Some(field_values) = doc.get_all(field).next() {
                if let Some(text_value) = field_values.as_str() {
                    *facet_counts.entry(text_value.to_string()).or_insert(0) += 1;
                }
            }
        }
        
        let mut facets: Vec<FacetValue> = facet_counts
            .into_iter()
            .map(|(value, count)| FacetValue { value, count })
            .collect();
        
        facets.sort_by(|a, b| b.count.cmp(&a.count));
        facets.truncate(20); // Limit to top 20 facets
        
        Ok(facets)
    }

    fn convert_document_to_hit(&self, doc: &Document, score: Score, query: &SearchQuery) -> Result<Option<SearchHit>, Box<dyn std::error::Error>> {
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
        
        let line_count = doc.get_first(self.schema.get_field("line_count").unwrap())
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        // Parse language and file type
        let language = match language_str {
            "Rust" => crate::types::Language::Rust,
            "JavaScript" => crate::types::Language::JavaScript,
            "TypeScript" => crate::types::Language::TypeScript,
            "Python" => crate::types::Language::Python,
            "Java" => crate::types::Language::Java,
            "C" => crate::types::Language::C,
            "Cpp" => crate::types::Language::Cpp,
            _ => crate::types::Language::Unknown,
        };
        
        let file_type = match file_type_str {
            "Source" => crate::types::FileType::Source,
            "Documentation" => crate::types::FileType::Documentation,
            "Configuration" => crate::types::FileType::Configuration,
            "Data" => crate::types::FileType::Data,
            "Binary" => crate::types::FileType::Binary,
            _ => crate::types::FileType::Unknown,
        };
        
        // Create indexed file
        let indexed_file = IndexedFile {
            id: file_id.unwrap(),
            project_id: project_id.unwrap(),
            path: std::path::PathBuf::from(&file_path),
            relative_path: std::path::PathBuf::from(&file_path),
            file_type,
            language,
            size: file_size,
            lines: line_count,
            last_modified: chrono::Utc::now(), // TODO: Get from document
            last_indexed: chrono::Utc::now(),
            checksum: String::new(), // TODO: Get from document
            encoding: "UTF-8".to_string(),
        };
        
        // Extract symbols
        let symbols = self.extract_symbols_from_document(doc)?;
        
        // Generate highlights
        let highlights = self.generate_highlights(doc, query)?;
        
        // Generate line matches
        let line_matches = self.generate_line_matches(doc, query)?;
        
        Ok(Some(SearchHit {
            file: indexed_file,
            score,
            highlights,
            symbols,
            line_matches,
        }))
    }

    fn extract_symbols_from_document(&self, doc: &Document) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        let mut symbols = Vec::new();
        
        // Extract symbol information from document
        let symbol_names: Vec<_> = doc.get_all(self.schema.get_field("symbol_name").unwrap())
            .filter_map(|v| v.as_str())
            .collect();
        
        let symbol_types: Vec<_> = doc.get_all(self.schema.get_field("symbol_type").unwrap())
            .filter_map(|v| v.as_str())
            .collect();
        
        let symbol_lines: Vec<_> = doc.get_all(self.schema.get_field("symbol_line").unwrap())
            .filter_map(|v| v.as_u64())
            .collect();
        
        for (i, name) in symbol_names.iter().enumerate() {
            if let (Some(type_str), Some(line)) = (symbol_types.get(i), symbol_lines.get(i)) {
                let symbol_type = match *type_str {
                    "Function" => SymbolType::Function,
                    "Method" => SymbolType::Method,
                    "Class" => SymbolType::Class,
                    "Variable" => SymbolType::Variable,
                    _ => SymbolType::Function,
                };
                
                symbols.push(Symbol {
                    id: Uuid::new_v4(),
                    file_id: Uuid::new_v4(), // TODO: Get from document
                    name: name.to_string(),
                    symbol_type,
                    line: *line as u32,
                    column: 0,
                    end_line: *line as u32,
                    end_column: 0,
                    signature: None,
                    scope: None,
                    namespace: None,
                });
            }
        }
        
        Ok(symbols)
    }

    fn generate_highlights(&self, doc: &Document, query: &SearchQuery) -> Result<Vec<Highlight>, Box<dyn std::error::Error>> {
        let mut highlights = Vec::new();
        
        // Get content from document
        if let Some(content) = doc.get_first(self.schema.get_field("content").unwrap())
            .and_then(|v| v.as_str()) {
            
            let matches = highlight_matches(content, &query.query, query.case_sensitive);
            if !matches.is_empty() {
                let fragments = self.extract_highlight_fragments(content, &matches, 100);
                highlights.push(Highlight {
                    field: "content".to_string(),
                    fragments,
                });
            }
        }
        
        Ok(highlights)
    }

    fn extract_highlight_fragments(&self, content: &str, matches: &[(usize, usize)], context_size: usize) -> Vec<String> {
        let mut fragments = Vec::new();
        
        for &(start, end) in matches.iter().take(5) { // Limit to 5 fragments
            let fragment_start = start.saturating_sub(context_size);
            let fragment_end = std::cmp::min(end + context_size, content.len());
            
            let fragment = &content[fragment_start..fragment_end];
            let highlight_start = start - fragment_start;
            let highlight_end = end - fragment_start;
            
            let highlighted = format!(
                "{}**{}**{}",
                &fragment[..highlight_start],
                &fragment[highlight_start..highlight_end],
                &fragment[highlight_end..]
            );
            
            fragments.push(highlighted);
        }
        
        fragments
    }

    fn generate_line_matches(&self, doc: &Document, query: &SearchQuery) -> Result<Vec<LineMatch>, Box<dyn std::error::Error>> {
        let mut line_matches = Vec::new();
        
        if let Some(content) = doc.get_first(self.schema.get_field("content").unwrap())
            .and_then(|v| v.as_str()) {
            
            for (line_num, line) in content.lines().enumerate() {
                let matches = highlight_matches(line, &query.query, query.case_sensitive);
                if !matches.is_empty() {
                    line_matches.push(LineMatch {
                        line_number: line_num as u32 + 1,
                        content: line.to_string(),
                        highlights: matches,
                    });
                }
            }
        }
        
        // Limit to reasonable number of line matches
        line_matches.truncate(50);
        
        Ok(line_matches)
    }

    pub fn suggest_completions(&self, prefix: &str, limit: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let searcher = self.reader.searcher();
        let symbol_name_field = self.schema.get_field("symbol_name").unwrap();
        
        // Use prefix query for suggestions
        let query: Box<dyn tantivy::query::Query> = Box::new(TermQuery::new(
            Term::from_field_text(symbol_name_field, prefix),
            IndexRecordOption::Basic,
        ));
        
        let top_docs = TopDocs::with_limit(limit);
        let search_results = searcher.search(&query, &top_docs)?;
        
        let mut suggestions = Vec::new();
        for (_score, doc_address) in search_results {
            let doc = searcher.doc(doc_address)?;
            if let Some(symbol_name) = doc.get_first(symbol_name_field)
                .and_then(|v| v.as_str()) {
                if symbol_name.starts_with(prefix) && !suggestions.contains(&symbol_name.to_string()) {
                    suggestions.push(symbol_name.to_string());
                }
            }
        }
        
        Ok(suggestions)
    }
}