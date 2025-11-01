use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use dashmap::DashMap;
use tokio::sync::{RwLock, Semaphore};
use futures::stream::{self, StreamExt};
use regex::Regex;

use crate::config::IndexerConfig;
use crate::models::*;
use crate::services::chunking::{ChunkingService, ChunkingStrategy, Chunk};

#[derive(Debug, Clone)]
pub struct DocumentTypeAnalyzer {
    html_regex: Regex,
    markdown_regex: Regex,
    pdf_magic: Vec<u8>,
    docx_magic: Vec<u8>,
}

impl DocumentTypeAnalyzer {
    pub fn new() -> Self {
        Self {
            html_regex: Regex::new(r"<[^>]+>").unwrap(),
            markdown_regex: Regex::new(r"^#{1,6}\s|^\*\*|^\*|^\d+\.").unwrap(),
            pdf_magic: b"%PDF".to_vec(),
            docx_magic: b"PK\x03\x04".to_vec(),
        }
    }

    pub fn detect_document_type(&self, content: &[u8], file_path: Option<&Path>) -> DocumentType {
        // Check file extension first
        if let Some(path) = file_path {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "html" | "htm" => return DocumentType::Html,
                    "md" | "markdown" => return DocumentType::Markdown,
                    "pdf" => return DocumentType::Pdf,
                    "docx" => return DocumentType::Docx,
                    "txt" => return DocumentType::PlainText,
                    "json" => return DocumentType::Json,
                    "xml" => return DocumentType::Xml,
                    "csv" => return DocumentType::Csv,
                    _ => {}
                }
            }
        }

        // Check magic bytes
        if content.starts_with(&self.pdf_magic) {
            return DocumentType::Pdf;
        }
        if content.starts_with(&self.docx_magic) {
            return DocumentType::Docx;
        }

        // Check content patterns
        if let Ok(text) = std::str::from_utf8(content) {
            if self.html_regex.is_match(text) {
                return DocumentType::Html;
            }
            if self.markdown_regex.is_match(text) {
                return DocumentType::Markdown;
            }
            if text.trim_start().starts_with('{') || text.trim_start().starts_with('[') {
                return DocumentType::Json;
            }
            if text.trim_start().starts_with("<?xml") || text.trim_start().starts_with('<') {
                return DocumentType::Xml;
            }
        }

        DocumentType::PlainText
    }

    pub fn should_index_document(&self, doc_type: &DocumentType) -> bool {
        matches!(doc_type, 
            DocumentType::Html | 
            DocumentType::Markdown | 
            DocumentType::PlainText | 
            DocumentType::Json | 
            DocumentType::Xml
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    Html,
    Markdown,
    PlainText,
    Pdf,
    Docx,
    Json,
    Xml,
    Csv,
    Unknown,
}

#[derive(Debug, Default)]
pub struct DocumentMetrics {
    pub total_documents_processed: usize,
    pub total_chunks_created: usize,
    pub total_processing_time_ms: u64,
    pub documents_per_type: HashMap<String, usize>,
    pub average_document_size: f64,
    pub average_chunks_per_document: f64,
}

pub struct DocumentIndexingService {
    config: IndexerConfig,
    jobs: Arc<DashMap<String, IndexingJob>>,
    chunking_service: Arc<ChunkingService>,
    document_analyzer: DocumentTypeAnalyzer,
    // Concurrency control
    document_semaphore: Arc<Semaphore>,
    // Performance metrics
    metrics: Arc<RwLock<DocumentMetrics>>,
}

impl DocumentIndexingService {
    pub fn new(config: IndexerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let chunking_service = ChunkingService::new(config.clone())
            .with_strategy(ChunkingStrategy::Hierarchical);

        Ok(Self {
            config: config.clone(),
            jobs: Arc::new(DashMap::new()),
            chunking_service: Arc::new(chunking_service),
            document_analyzer: DocumentTypeAnalyzer::new(),
            document_semaphore: Arc::new(Semaphore::new(8)), // Limit concurrent document processing
            metrics: Arc::new(RwLock::new(DocumentMetrics::default())),
        })
    }

    /// Get current document indexing metrics
    pub async fn get_metrics(&self) -> DocumentMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = DocumentMetrics::default();
    }

    pub async fn index_documentation(
        &self,
        doc_url: String,
        crawl_depth: u32,
        metadata: HashMap<String, String>,
    ) -> Result<IndexingJob, Box<dyn std::error::Error>> {
        let mut job = IndexingJob::new(
            SourceType::Documentation,
            doc_url.clone(),
            metadata,
        );
        
        let job_id = job.id.clone();
        job.start();
        self.jobs.insert(job_id.clone(), job.clone());

        
        let jobs = self.jobs.clone();
        let chunking_service = self.chunking_service.clone();

        tokio::spawn(async move {
            match Self::process_documentation(&doc_url, crawl_depth, chunking_service).await {
                Ok((docs, chunks, embeddings)) => {
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.complete(docs, chunks, embeddings);
                    }
                }
                Err(e) => {
                    log::error!("Documentation indexing failed for {}: {}", job_id, e);
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.fail(e.to_string());
                    }
                }
            }
        });

        Ok(job)
    }

    pub async fn index_file(
        &self,
        file_path: String,
        metadata: HashMap<String, String>,
    ) -> Result<IndexingJob, Box<dyn std::error::Error>> {
        let mut job = IndexingJob::new(
            SourceType::File,
            file_path.clone(),
            metadata,
        );
        
        let job_id = job.id.clone();
        job.start();
        self.jobs.insert(job_id.clone(), job.clone());

        
        let jobs = self.jobs.clone();
        let chunking_service = self.chunking_service.clone();

        tokio::spawn(async move {
            match Self::process_file(&file_path, chunking_service).await {
                Ok((docs, chunks, embeddings)) => {
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.complete(docs, chunks, embeddings);
                    }
                }
                Err(e) => {
                    log::error!("File indexing failed for {}: {}", job_id, e);
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.fail(e.to_string());
                    }
                }
            }
        });

        Ok(job)
    }

    async fn process_documentation(
        doc_url: &str,
        _crawl_depth: u32,
        chunking_service: Arc<ChunkingService>,
    ) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        log::info!("Processing documentation: {}", doc_url);

        
        let client = reqwest::Client::new();
        let response = client
            .get(doc_url)
            .header("User-Agent", "ConHub-Indexer/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to fetch documentation: HTTP {}", response.status()).into());
        }

        let content = response.text().await?;
        
        
        let text = Self::extract_text_from_html(&content)?;
        
        
        let chunks = chunking_service.chunk_text(&text)?;
        let chunk_count = chunks.len();

        log::info!("Documentation processing complete: 1 document, {} chunks", chunk_count);
        
        Ok((1, chunk_count, chunk_count))
    }

    async fn process_file(
        file_path: &str,
        chunking_service: Arc<ChunkingService>,
    ) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        log::info!("Processing file: {}", file_path);

        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        
        let content = tokio::fs::read_to_string(path).await?;
        
        
        let chunks = chunking_service.chunk_text(&content)?;
        let chunk_count = chunks.len();

        log::info!("File processing complete: 1 document, {} chunks", chunk_count);
        
        Ok((1, chunk_count, chunk_count))
    }

    async fn process_file_advanced(&self, file_path: &Path) -> Result<(usize, String, u64), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let content_bytes = std::fs::read(file_path)?;
        let file_size = content_bytes.len() as u64;
        
        // Detect document type
        let doc_type = self.document_analyzer.detect_document_type(&content_bytes, Some(file_path));
        
        if !self.document_analyzer.should_index_document(&doc_type) {
            return Err("Document type not supported for indexing".into());
        }

        // Extract text content based on document type
        let text_content = self.extract_text_by_type(&content_bytes, &doc_type).await?;
        
        // Use appropriate chunking strategy based on document type
        let chunks = self.chunk_by_document_type(&text_content, &doc_type).await?;
        
        // Extract document metadata
        let metadata = self.extract_document_metadata(file_path, &text_content, &doc_type)?;
        
        // Process chunks with enhanced metadata
        let processed_chunks = self.process_document_chunks(chunks, metadata).await?;
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        log::info!("Advanced file processing complete: {} chunks in {}ms", 
                  processed_chunks.len(), processing_time);
        
        Ok((processed_chunks.len(), format!("{:?}", doc_type), file_size))
    }

    async fn extract_text_by_type(&self, content: &[u8], doc_type: &DocumentType) -> Result<String, Box<dyn std::error::Error>> {
        match doc_type {
            DocumentType::Html => {
                let html = std::str::from_utf8(content)?;
                self.extract_text_from_html_advanced(html)
            }
            DocumentType::Markdown => {
                let markdown = std::str::from_utf8(content)?;
                Ok(self.extract_text_from_markdown(markdown))
            }
            DocumentType::Json => {
                let json = std::str::from_utf8(content)?;
                Ok(self.extract_text_from_json(json)?)
            }
            DocumentType::Xml => {
                let xml = std::str::from_utf8(content)?;
                Ok(self.extract_text_from_xml(xml))
            }
            DocumentType::PlainText => {
                Ok(std::str::from_utf8(content)?.to_string())
            }
            _ => Err("Unsupported document type for text extraction".into())
        }
    }

    async fn chunk_by_document_type(&self, content: &str, doc_type: &DocumentType) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        match doc_type {
            DocumentType::Html => {
                self.chunking_service.chunk_with_metadata(content, Some("html")).await
            }
            DocumentType::Markdown => {
                self.chunking_service.chunk_with_metadata(content, Some("markdown")).await
            }
            DocumentType::Json => {
                self.chunking_service.chunk_with_metadata(content, Some("json")).await
            }
            DocumentType::Xml => {
                self.chunking_service.chunk_with_metadata(content, Some("xml")).await
            }
            DocumentType::PlainText => {
                self.chunking_service.chunk_with_metadata(content, Some("text")).await
            }
            _ => {
                // Fallback to basic text chunking
                Ok(self.chunking_service.chunk_text(content)?.into_iter()
                    .enumerate()
                    .map(|(i, text)| Chunk {
                        content: text,
                        metadata: crate::services::chunking::ChunkMetadata {
                            start_offset: i * 1000,
                            end_offset: (i + 1) * 1000,
                            language: Some("text".to_string()),
                            section_type: Some("paragraph".to_string()),
                            importance_score: 0.5,
                            semantic_density: 0.5,
                        }
                    })
                    .collect())
            }
        }
    }

    fn extract_text_from_html(html: &str) -> Result<String, Box<dyn std::error::Error>> {
        
        let document = scraper::Html::parse_document(html);
        
        
        let selector = scraper::Selector::parse("body").unwrap();
        
        let mut text = String::new();
        for element in document.select(&selector) {
            text.push_str(&element.text().collect::<Vec<_>>().join(" "));
        }
        
        
        let cleaned = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        Ok(cleaned)
    }

    fn extract_text_from_html_advanced(&self, html: &str) -> Result<String, Box<dyn std::error::Error>> {
        // More sophisticated HTML parsing
        let mut text = String::new();
        let mut in_tag = false;
        let mut in_script = false;
        let mut in_style = false;
        
        let chars: Vec<char> = html.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i] == '<' {
                in_tag = true;
                
                // Check for script or style tags
                if i + 6 < chars.len() {
                    let tag_start: String = chars[i..i+7].iter().collect();
                    if tag_start.to_lowercase() == "<script" {
                        in_script = true;
                    } else if tag_start.to_lowercase() == "<style>" {
                        in_style = true;
                    }
                }
                
                // Check for closing script or style tags
                if i + 8 < chars.len() {
                    let tag_end: String = chars[i..i+9].iter().collect();
                    if tag_end.to_lowercase() == "</script>" {
                        in_script = false;
                    } else if tag_end.to_lowercase() == "</style>" {
                        in_style = false;
                    }
                }
            } else if chars[i] == '>' {
                in_tag = false;
            } else if !in_tag && !in_script && !in_style {
                text.push(chars[i]);
            }
            
            i += 1;
        }
        
        // Clean up whitespace
        let text = text.split_whitespace().collect::<Vec<&str>>().join(" ");
        Ok(text)
    }

    fn extract_text_from_markdown(&self, markdown: &str) -> String {
        let mut text = String::new();
        
        for line in markdown.lines() {
            let trimmed = line.trim();
            
            // Skip code blocks
            if trimmed.starts_with("```") {
                continue;
            }
            
            // Remove markdown formatting
            let mut clean_line = trimmed.to_string();
            
            // Remove headers
            clean_line = clean_line.trim_start_matches('#').trim().to_string();
            
            // Remove bold/italic
            clean_line = clean_line.replace("**", "").replace("*", "");
            
            // Remove links but keep text
            let link_regex = Regex::new(r"\[([^\]]+)\]\([^\)]+\)").unwrap();
            clean_line = link_regex.replace_all(&clean_line, "$1").to_string();
            
            if !clean_line.is_empty() {
                text.push_str(&clean_line);
                text.push(' ');
            }
        }
        
        text.trim().to_string()
    }

    fn extract_text_from_json(&self, json: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Extract string values from JSON
        let mut text = String::new();
        let mut in_string = false;
        let mut escape_next = false;
        
        for ch in json.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match ch {
                '"' => in_string = !in_string,
                '\\' if in_string => escape_next = true,
                _ if in_string && ch.is_alphanumeric() || ch.is_whitespace() => {
                    text.push(ch);
                }
                _ => {}
            }
        }
        
        Ok(text.split_whitespace().collect::<Vec<&str>>().join(" "))
    }

    fn extract_text_from_xml(&self, xml: &str) -> String {
        // Simple XML tag removal
        let tag_regex = Regex::new(r"<[^>]*>").unwrap();
        let text = tag_regex.replace_all(xml, " ");
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    fn extract_html_metadata(&self, html: &str) -> (Option<String>, Vec<String>, Vec<String>) {
        let mut title = None;
        let mut headings = Vec::new();
        let mut links = Vec::new();
        
        // Extract title
        if let Some(title_match) = Regex::new(r"<title[^>]*>([^<]+)</title>").unwrap().captures(html) {
            title = Some(title_match[1].to_string());
        }
        
        // Extract headings
        for level in 1..=6 {
            let heading_regex = Regex::new(&format!(r"<h{}[^>]*>([^<]+)</h{}>", level, level)).unwrap();
            for cap in heading_regex.captures_iter(html) {
                headings.push(cap[1].to_string());
            }
        }
        
        // Extract links
        let link_regex = Regex::new(r#"<a[^>]+href=["']([^"']+)["'][^>]*>([^<]*)</a>"#).unwrap();
        for cap in link_regex.captures_iter(html) {
            links.push(format!("{} ({})", &cap[2], &cap[1]));
        }
        
        (title, headings, links)
    }

    fn extract_markdown_metadata(&self, markdown: &str) -> (Option<String>, Vec<String>, Vec<String>) {
        let mut title = None;
        let mut headings = Vec::new();
        let mut links = Vec::new();
        
        for line in markdown.lines() {
            let trimmed = line.trim();
            
            // Extract headings
            if trimmed.starts_with('#') {
                let heading = trimmed.trim_start_matches('#').trim().to_string();
                if title.is_none() && trimmed.starts_with("# ") {
                    title = Some(heading.clone());
                }
                headings.push(heading);
            }
            
            // Extract links
            let link_regex = Regex::new(r"\[([^\]]+)\]\(([^\)]+)\)").unwrap();
            for cap in link_regex.captures_iter(trimmed) {
                links.push(format!("{} ({})", &cap[1], &cap[2]));
            }
        }
        
        (title, headings, links)
    }

    fn contains_headings(&self, content: &str, doc_type: &DocumentType) -> bool {
        match doc_type {
            DocumentType::Html => content.contains("<h1") || content.contains("<h2") || content.contains("<h3"),
            DocumentType::Markdown => content.contains('#'),
            _ => false,
        }
    }

    fn contains_links(&self, content: &str, doc_type: &DocumentType) -> bool {
        match doc_type {
            DocumentType::Html => content.contains("<a "),
            DocumentType::Markdown => content.contains("]("),
            _ => false,
        }
    }

    fn contains_code_blocks(&self, content: &str, doc_type: &DocumentType) -> bool {
        match doc_type {
            DocumentType::Html => content.contains("<code>") || content.contains("<pre>"),
            DocumentType::Markdown => content.contains("```") || content.contains("`"),
            _ => false,
        }
    }

    fn calculate_readability_score(&self, content: &str) -> f64 {
        let words: Vec<&str> = content.split_whitespace().collect();
        let sentences: Vec<&str> = content.split(&['.', '!', '?'][..]).collect();
        
        if words.is_empty() || sentences.is_empty() {
            return 0.0;
        }
        
        let avg_words_per_sentence = words.len() as f64 / sentences.len() as f64;
        let avg_syllables_per_word = words.iter()
            .map(|word| self.count_syllables(word))
            .sum::<usize>() as f64 / words.len() as f64;
        
        // Simplified Flesch Reading Ease formula
        206.835 - (1.015 * avg_words_per_sentence) - (84.6 * avg_syllables_per_word)
    }

    fn count_syllables(&self, word: &str) -> usize {
        let vowels = "aeiouAEIOU";
        let mut count = 0;
        let mut prev_was_vowel = false;
        
        for ch in word.chars() {
            let is_vowel = vowels.contains(ch);
            if is_vowel && !prev_was_vowel {
                count += 1;
            }
            prev_was_vowel = is_vowel;
        }
        
        // Handle silent 'e'
        if word.ends_with('e') && count > 1 {
            count -= 1;
        }
        
        count.max(1) // Every word has at least one syllable
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        _offset: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        log::info!("Searching documents: {}", query);
        
        
        let mut results = Vec::new();
        
        if !query.is_empty() {
            results.push(SearchResult {
                id: uuid::Uuid::new_v4().to_string(),
                title: format!("Document search result for: {}", query),
                content: "Sample document content...".to_string(),
                source_type: "documentation".to_string(),
                source_url: "https://docs.example.com".to_string(),
                score: 0.90,
                metadata: HashMap::new(),
            });
        }
        
        results.truncate(limit);
        Ok(results)
    }

    pub async fn get_stats(&self) -> StatusResponse {
        let jobs: Vec<_> = self.jobs.iter().map(|e| e.value().clone()).collect();
        
        let active = jobs.iter().filter(|j| matches!(j.status, IndexingStatus::InProgress)).count();
        let completed = jobs.iter().filter(|j| matches!(j.status, IndexingStatus::Completed)).count();
        let failed = jobs.iter().filter(|j| matches!(j.status, IndexingStatus::Failed)).count();
        let pending = jobs.iter().filter(|j| matches!(j.status, IndexingStatus::Pending)).count();

        StatusResponse {
            active_jobs: active,
            completed_jobs: completed,
            failed_jobs: failed,
            queue_size: pending,
        }
    }

    pub async fn get_job(&self, job_id: &str) -> Option<IndexingJob> {
        self.jobs.get(job_id).map(|e| e.value().clone())
    }
}
