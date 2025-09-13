use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub indexed: bool,
    pub last_indexed: Option<DateTime<Utc>>,
    pub repository_type: RepositoryType,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepositoryType {
    Git,
    Mercurial,
    Subversion,
    Bazaar,
    Perforce,
    FileSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    pub id: Uuid,
    pub project_id: Uuid,
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub file_type: FileType,
    pub language: Language,
    pub size: u64,
    pub lines: u32,
    pub last_modified: DateTime<Utc>,
    pub last_indexed: DateTime<Utc>,
    pub checksum: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum FileType {
    Source,
    Documentation,
    Configuration,
    Data,
    Binary,
    Archive,
    Image,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Java,
    C,
    Cpp,
    Go,
    Html,
    Css,
    Json,
    Xml,
    Yaml,
    Toml,
    Markdown,
    Shell,
    Sql,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: Uuid,
    pub file_id: Uuid,
    pub name: String,
    pub symbol_type: SymbolType,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub signature: Option<String>,
    pub scope: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SymbolType {
    Function,
    Method,
    Class,
    Interface,
    Struct,
    Enum,
    Variable,
    Constant,
    Field,
    Parameter,
    Module,
    Namespace,
    Macro,
    Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub id: Uuid,
    pub symbol_id: Uuid,
    pub file_id: Uuid,
    pub line: u32,
    pub column: u32,
    pub reference_type: ReferenceType,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    Definition,
    Declaration,
    Usage,
    Call,
    Import,
    Inheritance,
    Implementation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub query_type: QueryType,
    pub projects: Option<Vec<Uuid>>,
    pub file_types: Option<Vec<FileType>>,
    pub languages: Option<Vec<Language>>,
    pub path_filter: Option<String>,
    pub case_sensitive: bool,
    pub regex: bool,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum QueryType {
    FullText,
    Symbol,
    Path,
    Definition,
    Reference,
    History,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub total_hits: usize,
    pub results: Vec<SearchHit>,
    pub facets: HashMap<String, Vec<FacetValue>>,
    pub query_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub file: IndexedFile,
    pub score: f32,
    pub highlights: Vec<Highlight>,
    pub symbols: Vec<Symbol>,
    pub line_matches: Vec<LineMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub field: String,
    pub fragments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineMatch {
    pub line_number: u32,
    pub content: String,
    pub highlights: Vec<(usize, usize)>, // (start, end) positions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetValue {
    pub value: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub file_id: Uuid,
    pub commit_id: String,
    pub author: String,
    pub author_email: String,
    pub committer: String,
    pub committer_email: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub changes: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub change_type: ChangeType,
    pub old_path: Option<PathBuf>,
    pub new_path: Option<PathBuf>,
    pub lines_added: u32,
    pub lines_deleted: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    pub symbol: Symbol,
    pub definitions: Vec<Reference>,
    pub declarations: Vec<Reference>,
    pub usages: Vec<Reference>,
    pub calls: Vec<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_projects: usize,
    pub total_files: usize,
    pub total_lines: u64,
    pub total_symbols: usize,
    pub total_references: usize,
    pub index_size_bytes: u64,
    pub last_update: DateTime<Utc>,
    pub languages: HashMap<Language, usize>,
    pub file_types: HashMap<FileType, usize>,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "js" | "mjs" | "cjs" => Language::JavaScript,
            "ts" | "tsx" => Language::TypeScript,
            "py" | "pyw" => Language::Python,
            "java" => Language::Java,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Language::Cpp,
            "go" => Language::Go,
            "html" | "htm" => Language::Html,
            "css" => Language::Css,
            "json" => Language::Json,
            "xml" => Language::Xml,
            "yaml" | "yml" => Language::Yaml,
            "toml" => Language::Toml,
            "md" | "markdown" => Language::Markdown,
            "sh" | "bash" | "zsh" => Language::Shell,
            "sql" => Language::Sql,
            _ => Language::Unknown,
        }
    }

    pub fn tree_sitter_language(&self) -> Option<tree_sitter::Language> {
        match self {
            Language::Rust => Some(tree_sitter_rust::language()),
            Language::JavaScript => Some(tree_sitter_javascript::language()),
            Language::TypeScript => Some(tree_sitter_typescript::language_typescript()),
            Language::Python => Some(tree_sitter_python::language()),
            Language::Java => Some(tree_sitter_java::language()),
            Language::C => Some(tree_sitter_c::language()),
            Language::Cpp => Some(tree_sitter_cpp::language()),
            Language::Go => Some(tree_sitter_go::language()),
            _ => None,
        }
    }
}

impl FileType {
    pub fn from_path(path: &std::path::Path) -> Self {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "rs" | "js" | "ts" | "py" | "java" | "c" | "cpp" | "go" | "h" | "hpp" => FileType::Source,
                "md" | "txt" | "rst" | "adoc" => FileType::Documentation,
                "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "cfg" | "conf" => FileType::Configuration,
                "csv" | "tsv" | "dat" => FileType::Data,
                "exe" | "dll" | "so" | "dylib" | "a" | "lib" => FileType::Binary,
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => FileType::Archive,
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "ico" => FileType::Image,
                _ => FileType::Unknown,
            }
        } else {
            FileType::Unknown
        }
    }
}