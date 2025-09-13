use crate::types::*;
use crate::config::{LexorConfig, ProjectConfig};
use crate::parser::{LanguageParser, SimpleParser};
use crate::utils::*;
use crate::history::HistoryAnalyzer;

use tantivy::schema::*;
use tantivy::{Index, IndexWriter, Document, Term};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::directory::MmapDirectory;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::fs;
use walkdir::WalkDir;
use ignore::WalkBuilder;
use rayon::prelude::*;
use dashmap::DashMap;
use uuid::Uuid;
use chrono::Utc;
use log::{info, warn, error, debug};

pub struct IndexerEngine {
    config: LexorConfig,
    schema: Schema,
    index: Index,
    writer: Arc<Mutex<IndexWriter>>,
    projects: Arc<DashMap<Uuid, Project>>,
    files: Arc<DashMap<Uuid, IndexedFile>>,
    symbols: Arc<DashMap<Uuid, Symbol>>,
    references: Arc<DashMap<Uuid, Reference>>,
    parser: Arc<Mutex<LanguageParser>>,
    history_analyzer: Arc<Mutex<HistoryAnalyzer>>,
}

impl IndexerEngine {
    pub fn new(config: LexorConfig) -> Result<Self, Box<dyn std::error::Error>> {
        config.ensure_directories()?;
        
        let schema = Self::create_schema();
        let index_dir = MmapDirectory::open(&config.index_dir)?;
        let index = Index::open_or_create(index_dir, schema.clone())?;
        
        let writer = index.writer(config.indexer.memory_limit * 1024 * 1024)?;
        
        Ok(Self {
            config,
            schema,
            index,
            writer: Arc::new(Mutex::new(writer)),
            projects: Arc::new(DashMap::new()),
            files: Arc::new(DashMap::new()),
            symbols: Arc::new(DashMap::new()),
            references: Arc::new(DashMap::new()),
            parser: Arc::new(Mutex::new(LanguageParser::new())),
            history_analyzer: Arc::new(Mutex::new(HistoryAnalyzer::new())),
        })
    }

    fn create_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        
        // File fields
        schema_builder.add_text_field("file_id", STRING | STORED);
        schema_builder.add_text_field("project_id", STRING | STORED);
        schema_builder.add_text_field("file_path", TEXT | STORED);
        schema_builder.add_text_field("file_name", TEXT | STORED);
        schema_builder.add_text_field("file_extension", STRING | STORED);
        schema_builder.add_text_field("language", STRING | STORED);
        schema_builder.add_text_field("file_type", STRING | STORED);
        schema_builder.add_u64_field("file_size", INDEXED | STORED);
        schema_builder.add_u64_field("line_count", INDEXED | STORED);
        schema_builder.add_date_field("last_modified", INDEXED | STORED);
        schema_builder.add_text_field("checksum", STRING | STORED);
        
        // Content fields
        schema_builder.add_text_field("content", TEXT);
        schema_builder.add_text_field("content_lines", TEXT);
        
        // Symbol fields
        schema_builder.add_text_field("symbol_id", STRING | STORED);
        schema_builder.add_text_field("symbol_name", TEXT | STORED);
        schema_builder.add_text_field("symbol_type", STRING | STORED);
        schema_builder.add_u64_field("symbol_line", INDEXED | STORED);
        schema_builder.add_u64_field("symbol_column", INDEXED | STORED);
        schema_builder.add_text_field("symbol_signature", TEXT | STORED);
        schema_builder.add_text_field("symbol_scope", TEXT | STORED);
        schema_builder.add_text_field("symbol_namespace", TEXT | STORED);
        
        // Reference fields
        schema_builder.add_text_field("reference_id", STRING | STORED);
        schema_builder.add_text_field("reference_type", STRING | STORED);
        schema_builder.add_u64_field("reference_line", INDEXED | STORED);
        schema_builder.add_u64_field("reference_column", INDEXED | STORED);
        schema_builder.add_text_field("reference_context", TEXT | STORED);
        
        // History fields
        schema_builder.add_text_field("commit_id", STRING | STORED);
        schema_builder.add_text_field("author", TEXT | STORED);
        schema_builder.add_text_field("commit_message", TEXT | STORED);
        schema_builder.add_date_field("commit_date", INDEXED | STORED);
        
        // Project fields
        schema_builder.add_text_field("project_name", TEXT | STORED);
        schema_builder.add_text_field("project_description", TEXT | STORED);
        
        schema_builder.build()
    }

    pub fn add_project(&self, name: String, path: PathBuf, description: Option<String>) -> Result<Uuid, Box<dyn std::error::Error>> {
        let project_id = Uuid::new_v4();
        
        let repository_type = if path.join(".git").exists() {
            RepositoryType::Git
        } else if path.join(".hg").exists() {
            RepositoryType::Mercurial
        } else if path.join(".svn").exists() {
            RepositoryType::Subversion
        } else {
            RepositoryType::FileSystem
        };

        let project = Project {
            id: project_id,
            name: name.clone(),
            path: path.clone(),
            description,
            indexed: false,
            last_indexed: None,
            repository_type,
            branch: None,
            remote_url: None,
        };

        self.projects.insert(project_id, project);
        
        info!("Added project '{}' with ID: {}", name, project_id);
        Ok(project_id)
    }

    pub fn index_project(&self, project_id: Uuid) -> Result<IndexStats, Box<dyn std::error::Error>> {
        let project = self.projects.get(&project_id)
            .ok_or("Project not found")?
            .clone();

        info!("Starting indexing for project: {}", project.name);
        
        let start_time = std::time::Instant::now();
        let mut stats = IndexStats {
            total_projects: 1,
            total_files: 0,
            total_lines: 0,
            total_symbols: 0,
            total_references: 0,
            index_size_bytes: 0,
            last_update: Utc::now(),
            languages: HashMap::new(),
            file_types: HashMap::new(),
        };

        // Build file walker with ignore patterns
        let walker = WalkBuilder::new(&project.path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        // Collect files to process
        let mut files_to_process = Vec::new();
        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() && !self.should_ignore_file(path) {
                        files_to_process.push(path.to_path_buf());
                    }
                }
                Err(e) => warn!("Error walking directory: {}", e),
            }
        }

        info!("Found {} files to index", files_to_process.len());

        // Process files in parallel
        let processed_files: Vec<_> = files_to_process
            .par_iter()
            .filter_map(|path| {
                match self.process_file(project_id, path) {
                    Ok(Some(file_info)) => Some(file_info),
                    Ok(None) => None,
                    Err(e) => {
                        warn!("Error processing file {:?}: {}", path, e);
                        None
                    }
                }
            })
            .collect();

        // Update statistics
        for (file, symbols, references) in processed_files {
            stats.total_files += 1;
            stats.total_lines += file.lines as u64;
            stats.total_symbols += symbols.len();
            stats.total_references += references.len();
            
            *stats.languages.entry(file.language.clone()).or_insert(0) += 1;
            *stats.file_types.entry(file.file_type.clone()).or_insert(0) += 1;

            // Store in memory maps
            self.files.insert(file.id, file);
            for symbol in symbols {
                self.symbols.insert(symbol.id, symbol);
            }
            for reference in references {
                self.references.insert(reference.id, reference);
            }
        }

        // Update project status
        if let Some(mut project) = self.projects.get_mut(&project_id) {
            project.indexed = true;
            project.last_indexed = Some(Utc::now());
        }

        // Commit index changes
        {
            let mut writer = self.writer.lock().unwrap();
            writer.commit()?;
        }

        let duration = start_time.elapsed();
        info!(
            "Indexing completed for project '{}' in {:.2}s. Files: {}, Symbols: {}, References: {}",
            project.name,
            duration.as_secs_f64(),
            stats.total_files,
            stats.total_symbols,
            stats.total_references
        );

        Ok(stats)
    }

    fn process_file(&self, project_id: Uuid, path: &Path) -> Result<Option<(IndexedFile, Vec<Symbol>, Vec<Reference>)>, Box<dyn std::error::Error>> {
        // Check if file is binary
        if is_binary_file(path) {
            debug!("Skipping binary file: {:?}", path);
            return Ok(None);
        }

        // Read file content
        let (content, encoding) = read_file_with_encoding(path)?;
        if content.is_empty() {
            return Ok(None);
        }

        // Get file metadata
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();
        let last_modified = metadata.modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let last_modified = chrono::DateTime::from_timestamp(last_modified as i64, 0)
            .unwrap_or_else(|| Utc::now());

        // Determine language and file type
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        let language = Language::from_extension(extension);
        let file_type = FileType::from_path(path);

        // Calculate checksum
        let checksum = calculate_file_hash(path)?;
        let line_count = count_lines(&content);

        // Create file record
        let file_id = Uuid::new_v4();
        let project = self.projects.get(&project_id).unwrap();
        let relative_path = get_relative_path(&project.path, path)
            .unwrap_or_else(|| path.to_path_buf());

        let indexed_file = IndexedFile {
            id: file_id,
            project_id,
            path: path.to_path_buf(),
            relative_path,
            file_type: file_type.clone(),
            language: language.clone(),
            size: file_size,
            lines: line_count,
            last_modified,
            last_indexed: Utc::now(),
            checksum,
            encoding,
        };

        // Parse symbols and references
        let (symbols, references) = self.parse_file_content(file_id, &content, &language)?;

        // Index in Tantivy
        self.index_file_content(&indexed_file, &content, &symbols, &references)?;

        debug!("Processed file: {:?} ({} symbols, {} references)", path, symbols.len(), references.len());
        
        Ok(Some((indexed_file, symbols, references)))
    }

    fn parse_file_content(&self, file_id: Uuid, content: &str, language: &Language) -> Result<(Vec<Symbol>, Vec<Reference>), Box<dyn std::error::Error>> {
        let symbols = if language.tree_sitter_language().is_some() {
            // Use Tree-sitter parser
            let mut parser = self.parser.lock().unwrap();
            parser.parse_file(file_id, Path::new(""), content, language)?
        } else {
            // Use simple regex-based parser
            SimpleParser::parse_file_simple(file_id, content)
        };

        let references = if !symbols.is_empty() {
            let mut parser = self.parser.lock().unwrap();
            parser.extract_references(file_id, content, language, &symbols)
        } else {
            Vec::new()
        };

        Ok((symbols, references))
    }

    fn index_file_content(&self, file: &IndexedFile, content: &str, symbols: &[Symbol], references: &[Reference]) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = self.writer.lock().unwrap();
        let schema = &self.schema;

        // Create document for the file
        let mut doc = tantivy::TantivyDocument::default();
        
        // File fields
        doc.add_text(schema.get_field("file_id").unwrap(), &file.id.to_string());
        doc.add_text(schema.get_field("project_id").unwrap(), &file.project_id.to_string());
        doc.add_text(schema.get_field("file_path").unwrap(), &file.path.to_string_lossy());
        doc.add_text(schema.get_field("file_name").unwrap(), 
            file.path.file_name().unwrap_or_default().to_string_lossy().as_ref());
        doc.add_text(schema.get_field("file_extension").unwrap(), 
            file.path.extension().unwrap_or_default().to_string_lossy().as_ref());
        doc.add_text(schema.get_field("language").unwrap(), &format!("{:?}", file.language));
        doc.add_text(schema.get_field("file_type").unwrap(), &format!("{:?}", file.file_type));
        doc.add_u64(schema.get_field("file_size").unwrap(), file.size);
        doc.add_u64(schema.get_field("line_count").unwrap(), file.lines as u64);
        doc.add_date(schema.get_field("last_modified").unwrap(), tantivy::DateTime::from_timestamp_secs(file.last_modified.timestamp()));
        doc.add_text(schema.get_field("checksum").unwrap(), &file.checksum);

        // Content fields
        doc.add_text(schema.get_field("content").unwrap(), content);
        
        // Add line-by-line content for better search
        let lines_with_numbers: String = content.lines()
            .enumerate()
            .map(|(i, line)| format!("{}:{}", i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");
        doc.add_text(schema.get_field("content_lines").unwrap(), &lines_with_numbers);

        // Add symbols to the document
        for symbol in symbols {
            doc.add_text(schema.get_field("symbol_id").unwrap(), &symbol.id.to_string());
            doc.add_text(schema.get_field("symbol_name").unwrap(), &symbol.name);
            doc.add_text(schema.get_field("symbol_type").unwrap(), &format!("{:?}", symbol.symbol_type));
            doc.add_u64(schema.get_field("symbol_line").unwrap(), symbol.line as u64);
            doc.add_u64(schema.get_field("symbol_column").unwrap(), symbol.column as u64);
            
            if let Some(signature) = &symbol.signature {
                doc.add_text(schema.get_field("symbol_signature").unwrap(), signature);
            }
            if let Some(scope) = &symbol.scope {
                doc.add_text(schema.get_field("symbol_scope").unwrap(), scope);
            }
            if let Some(namespace) = &symbol.namespace {
                doc.add_text(schema.get_field("symbol_namespace").unwrap(), namespace);
            }
        }

        // Add references to the document
        for reference in references {
            doc.add_text(schema.get_field("reference_id").unwrap(), &reference.id.to_string());
            doc.add_text(schema.get_field("reference_type").unwrap(), &format!("{:?}", reference.reference_type));
            doc.add_u64(schema.get_field("reference_line").unwrap(), reference.line as u64);
            doc.add_u64(schema.get_field("reference_column").unwrap(), reference.column as u64);
            doc.add_text(schema.get_field("reference_context").unwrap(), &reference.context);
        }

        writer.add_document(doc)?;
        Ok(())
    }

    fn should_ignore_file(&self, path: &Path) -> bool {
        let project_config = ProjectConfig::default(); // Use default for now
        
        // Check file name
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if is_ignored_file(Path::new(file_name), &project_config.ignored_names) {
                return true;
            }
        }
        
        // Check full path
        if is_ignored_file(path, &project_config.ignored_files) {
            return true;
        }
        
        // Check directories in path
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    if project_config.ignored_dirs.contains(&name_str.to_string()) {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    pub fn reindex_project(&self, project_id: Uuid) -> Result<IndexStats, Box<dyn std::error::Error>> {
        info!("Reindexing project: {}", project_id);
        
        // Clear existing data for this project
        self.clear_project_data(project_id)?;
        
        // Reindex
        self.index_project(project_id)
    }

    fn clear_project_data(&self, project_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        // Remove from memory maps
        self.files.retain(|_, file| file.project_id != project_id);
        
        let file_ids: Vec<Uuid> = self.files.iter()
            .filter(|entry| entry.project_id == project_id)
            .map(|entry| entry.id)
            .collect();
        
        for file_id in file_ids {
            self.symbols.retain(|_, symbol| symbol.file_id != file_id);
            self.references.retain(|_, reference| reference.file_id != file_id);
        }

        // Clear from Tantivy index
        let mut writer = self.writer.lock().unwrap();
        let schema = &self.schema;
        let project_id_field = schema.get_field("project_id").unwrap();
        let term = Term::from_field_text(project_id_field, &project_id.to_string());
        writer.delete_term(term);
        writer.commit()?;

        Ok(())
    }

    pub fn get_index_stats(&self) -> IndexStats {
        let mut languages = HashMap::new();
        let mut file_types = HashMap::new();
        let mut total_lines = 0;

        for file in self.files.iter() {
            *languages.entry(file.language.clone()).or_insert(0) += 1;
            *file_types.entry(file.file_type.clone()).or_insert(0) += 1;
            total_lines += file.lines as u64;
        }

        IndexStats {
            total_projects: self.projects.len(),
            total_files: self.files.len(),
            total_lines,
            total_symbols: self.symbols.len(),
            total_references: self.references.len(),
            index_size_bytes: 0, // TODO: Calculate actual index size
            last_update: Utc::now(),
            languages,
            file_types,
        }
    }

    pub fn get_projects(&self) -> Vec<Project> {
        self.projects.iter().map(|entry| entry.value().clone()).collect()
    }

    pub fn get_project(&self, project_id: Uuid) -> Option<Project> {
        self.projects.get(&project_id).map(|entry| entry.value().clone())
    }

    pub fn remove_project(&self, project_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        self.clear_project_data(project_id)?;
        self.projects.remove(&project_id);
        info!("Removed project: {}", project_id);
        Ok(())
    }
}