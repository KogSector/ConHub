use crate::types::*;

use tantivy::{Index, IndexReader, Score};
use tantivy::query::{QueryParser, TermQuery};
use tantivy::collector::TopDocs;
use tantivy::schema::*;
use tantivy::Term;

use std::collections::HashMap;
use uuid::Uuid;

pub struct SearchEngine {
    #[allow(dead_code)]
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
        
        // Simple query parsing
        let tantivy_query = self.query_parser.parse_query(&query.query)?;
        
        // Execute search
        let top_docs = TopDocs::with_limit(query.limit).and_offset(query.offset);
        let search_results = searcher.search(&tantivy_query, &top_docs)?;
        
        // Convert results
        let mut hits = Vec::new();
        for (score, doc_address) in search_results {
            let doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
            if let Some(hit) = self.convert_document_to_hit(&doc, score, query)? {
                hits.push(hit);
            }
        }
        
        let query_time = start_time.elapsed().as_millis() as u64;
        
        Ok(SearchResult {
            total_hits: hits.len(),
            results: hits,
            facets: HashMap::new(),
            query_time_ms: query_time,
        })
    }

    fn convert_document_to_hit(&self, doc: &tantivy::TantivyDocument, score: Score, _query: &SearchQuery) -> Result<Option<SearchHit>, Box<dyn std::error::Error>> {
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
            "Rust" => Language::Rust,
            "JavaScript" => Language::JavaScript,
            "TypeScript" => Language::TypeScript,
            "Python" => Language::Python,
            "Java" => Language::Java,
            "C" => Language::C,
            "Cpp" => Language::Cpp,
            "Go" => Language::Go,
            _ => Language::Unknown,
        };
        
        let file_type = match file_type_str {
            "Source" => FileType::Source,
            "Documentation" => FileType::Documentation,
            "Configuration" => FileType::Configuration,
            "Data" => FileType::Data,
            "Binary" => FileType::Binary,
            _ => FileType::Unknown,
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
            last_modified: chrono::Utc::now(),
            last_indexed: chrono::Utc::now(),
            checksum: String::new(),
            encoding: "UTF-8".to_string(),
        };
        
        Ok(Some(SearchHit {
            file: indexed_file,
            score,
            highlights: Vec::new(),
            symbols: Vec::new(),
            line_matches: Vec::new(),
        }))
    }

    pub fn suggest_completions(&self, prefix: &str, limit: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let searcher = self.reader.searcher();
        let symbol_name_field = self.schema.get_field("symbol_name").unwrap();
        
        // Use prefix query for suggestions
        let query = TermQuery::new(
            Term::from_field_text(symbol_name_field, prefix),
            IndexRecordOption::Basic,
        );
        
        let top_docs = TopDocs::with_limit(limit);
        let search_results = searcher.search(&query, &top_docs)?;
        
        let mut suggestions = Vec::new();
        for (_score, doc_address) in search_results {
            let doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
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