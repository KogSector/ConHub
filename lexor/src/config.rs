use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexorConfig {
    pub data_root: PathBuf,
    pub source_root: PathBuf,
    pub index_dir: PathBuf,
    pub history_cache_dir: PathBuf,
    pub web_app_laf: String,
    pub projects: HashMap<String, ProjectConfig>,
    pub indexer: IndexerConfig,
    pub search: SearchConfig,
    pub web: WebConfig,
    pub repository: RepositoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub path: PathBuf,
    pub description: Option<String>,
    pub tabsize: u32,
    pub navigate_window_enabled: bool,
    pub history_enabled: bool,
    pub remote_scm_supported: bool,
    pub handle_renamed_files: bool,
    pub allowed_symlinks: Vec<PathBuf>,
    pub ignored_names: Vec<String>,
    pub ignored_files: Vec<String>,
    pub ignored_dirs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    pub verbose: bool,
    pub threads: usize,
    pub memory_limit: usize, // MB
    pub include_repositories: Vec<String>,
    pub ignore_repositories: Vec<String>,
    pub generate_html: bool,
    pub optimize_database: bool,
    pub index_version_history: bool,
    pub scan_repos: bool,
    pub list_repos: bool,
    pub list_files: bool,
    pub create_dict: bool,
    pub default_project: Option<String>,
    pub max_indexed_words: usize,
    pub history_cache: bool,
    pub history_cache_time: u64, // seconds
    pub remote_repo_command: Option<String>,
    pub user_page: Option<String>,
    pub user_page_suffix: Option<String>,
    pub bug_page: Option<String>,
    pub bug_pattern: Option<String>,
    pub review_page: Option<String>,
    pub review_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub max_results: usize,
    pub max_context: usize,
    pub max_context_length: usize,
    pub search_timeout: u64, // seconds
    pub cache_pages: bool,
    pub hits_per_page: usize,
    pub context_limit: usize,
    pub context_surplus_limit: usize,
    pub current_index_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub context_path: String,
    pub servlet_request_attributes: HashMap<String, String>,
    pub enable_projects: bool,
    pub compress_xref: bool,
    pub index_word_limit: usize,
    pub group_by_pkg: bool,
    pub last_edit_time: bool,
    pub allow_leading_wildcard: bool,
    pub ignored_names: Vec<String>,
    pub canonical_root: Option<String>,
    pub chat_page: Option<String>,
    pub header_include_file: Option<PathBuf>,
    pub body_include_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub command_timeout: u64, // seconds
    pub invalidate_repositories: bool,
    pub scan_depth: i32,
    pub nest_repositories: bool,
    pub generate_history: bool,
    pub history_reader_time_limit: u64, // seconds
    pub cache_history_in_db: bool,
    pub store_history_cache_in_db: bool,
    pub compressed_history_cache: bool,
    pub handle_renamed_files: bool,
    pub merge_commits_enabled: bool,
    pub tags_enabled: bool,
    pub branches_enabled: bool,
    pub remote_scm_supported: bool,
    pub fetch_history_when_not_in_cache: bool,
}

impl Default for LexorConfig {
    fn default() -> Self {
        Self {
            data_root: PathBuf::from("./lexor_data"),
            source_root: PathBuf::from("./src"),
            index_dir: PathBuf::from("./lexor_data/index"),
            history_cache_dir: PathBuf::from("./lexor_data/historycache"),
            web_app_laf: "default".to_string(),
            projects: HashMap::new(),
            indexer: IndexerConfig::default(),
            search: SearchConfig::default(),
            web: WebConfig::default(),
            repository: RepositoryConfig::default(),
        }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            description: None,
            tabsize: 8,
            navigate_window_enabled: true,
            history_enabled: true,
            remote_scm_supported: true,
            handle_renamed_files: true,
            allowed_symlinks: Vec::new(),
            ignored_names: vec![
                ".git".to_string(),
                ".hg".to_string(),
                ".svn".to_string(),
                "CVS".to_string(),
                "SCCS".to_string(),
                ".bzr".to_string(),
                "_darcs".to_string(),
                ".repo".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                "build".to_string(),
                "dist".to_string(),
                ".idea".to_string(),
                ".vscode".to_string(),
            ],
            ignored_files: vec![
                "*.class".to_string(),
                "*.jar".to_string(),
                "*.war".to_string(),
                "*.ear".to_string(),
                "*.zip".to_string(),
                "*.tar".to_string(),
                "*.gz".to_string(),
                "*.bz2".to_string(),
                "*.7z".to_string(),
                "*.rar".to_string(),
                "*.exe".to_string(),
                "*.dll".to_string(),
                "*.so".to_string(),
                "*.dylib".to_string(),
                "*.a".to_string(),
                "*.lib".to_string(),
                "*.o".to_string(),
                "*.obj".to_string(),
                "*.pyc".to_string(),
                "*.pyo".to_string(),
                "*.pyd".to_string(),
                "*.whl".to_string(),
                "*.egg".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
                "*.temp".to_string(),
                "*.cache".to_string(),
                "*.lock".to_string(),
                "Cargo.lock".to_string(),
                "package-lock.json".to_string(),
                "yarn.lock".to_string(),
            ],
            ignored_dirs: vec![
                "target".to_string(),
                "build".to_string(),
                "dist".to_string(),
                "out".to_string(),
                "bin".to_string(),
                "obj".to_string(),
                "Debug".to_string(),
                "Release".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
                ".pytest_cache".to_string(),
                ".coverage".to_string(),
                ".nyc_output".to_string(),
                "coverage".to_string(),
                ".git".to_string(),
                ".hg".to_string(),
                ".svn".to_string(),
                "CVS".to_string(),
                "SCCS".to_string(),
                ".bzr".to_string(),
                "_darcs".to_string(),
                ".repo".to_string(),
                ".idea".to_string(),
                ".vscode".to_string(),
                ".vs".to_string(),
            ],
        }
    }
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            threads: num_cpus::get(),
            memory_limit: 2048,
            include_repositories: Vec::new(),
            ignore_repositories: Vec::new(),
            generate_html: true,
            optimize_database: true,
            index_version_history: true,
            scan_repos: true,
            list_repos: false,
            list_files: false,
            create_dict: false,
            default_project: None,
            max_indexed_words: 60000,
            history_cache: true,
            history_cache_time: 30,
            remote_repo_command: None,
            user_page: None,
            user_page_suffix: None,
            bug_page: None,
            bug_pattern: None,
            review_page: None,
            review_pattern: None,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 1000,
            max_context: 10,
            max_context_length: 250,
            search_timeout: 30,
            cache_pages: true,
            hits_per_page: 25,
            context_limit: 10,
            context_surplus_limit: 15,
            current_index_version: "1.0".to_string(),
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            context_path: "/source".to_string(),
            servlet_request_attributes: HashMap::new(),
            enable_projects: true,
            compress_xref: true,
            index_word_limit: 60000,
            group_by_pkg: true,
            last_edit_time: true,
            allow_leading_wildcard: true,
            ignored_names: Vec::new(),
            canonical_root: None,
            chat_page: None,
            header_include_file: None,
            body_include_file: None,
        }
    }
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            command_timeout: 120,
            invalidate_repositories: false,
            scan_depth: -1,
            nest_repositories: true,
            generate_history: true,
            history_reader_time_limit: 30,
            cache_history_in_db: false,
            store_history_cache_in_db: false,
            compressed_history_cache: true,
            handle_renamed_files: true,
            merge_commits_enabled: true,
            tags_enabled: true,
            branches_enabled: true,
            remote_scm_supported: true,
            fetch_history_when_not_in_cache: true,
        }
    }
}

impl LexorConfig {
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: LexorConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn ensure_directories(&self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&self.data_root)?;
        std::fs::create_dir_all(&self.index_dir)?;
        std::fs::create_dir_all(&self.history_cache_dir)?;
        Ok(())
    }
}