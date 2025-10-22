use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use regex::Regex;
use globset::{Glob, GlobSet, GlobSetBuilder};
use mime_guess::MimeGuess;

/// Enhanced pattern matcher with sophisticated filtering capabilities
#[derive(Debug, Clone)]
pub struct EnhancedPatternMatcher {
    /// Configuration for pattern matching
    config: PatternMatcherConfig,
    
    /// Compiled glob sets for efficient matching
    include_globs: Option<GlobSet>,
    exclude_globs: Option<GlobSet>,
    
    /// Compiled regex patterns
    include_regex: Vec<Regex>,
    exclude_regex: Vec<Regex>,
    
    /// MIME type filters
    mime_filters: MimeFilters,
    
    /// File size filters
    size_filters: SizeFilters,
    
    /// Time-based filters
    time_filters: TimeFilters,
    
    /// Content-based filters
    content_filters: ContentFilters,
    
    /// Custom filter functions
    custom_filters: Vec<CustomFilter>,
    
    /// Statistics
    stats: PatternMatcherStats,
}

/// Configuration for enhanced pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatcherConfig {
    /// Include patterns (glob format)
    pub include_patterns: Vec<String>,
    
    /// Exclude patterns (glob format)
    pub exclude_patterns: Vec<String>,
    
    /// Include regex patterns
    pub include_regex_patterns: Vec<String>,
    
    /// Exclude regex patterns
    pub exclude_regex_patterns: Vec<String>,
    
    /// Case sensitivity for pattern matching
    pub case_sensitive: bool,
    
    /// Enable MIME type detection and filtering
    pub enable_mime_filtering: bool,
    
    /// MIME type configuration
    pub mime_config: MimeFilterConfig,
    
    /// Enable file size filtering
    pub enable_size_filtering: bool,
    
    /// File size configuration
    pub size_config: SizeFilterConfig,
    
    /// Enable time-based filtering
    pub enable_time_filtering: bool,
    
    /// Time-based configuration
    pub time_config: TimeFilterConfig,
    
    /// Enable content-based filtering
    pub enable_content_filtering: bool,
    
    /// Content filtering configuration
    pub content_config: ContentFilterConfig,
    
    /// Enable custom filters
    pub enable_custom_filters: bool,
    
    /// Custom filter configurations
    pub custom_filter_configs: Vec<CustomFilterConfig>,
    
    /// Performance optimization settings
    pub performance: PerformanceConfig,
    
    /// Enable detailed statistics collection
    pub collect_stats: bool,
}

impl Default for PatternMatcherConfig {
    fn default() -> Self {
        Self {
            include_patterns: vec!["**/*".to_string()],
            exclude_patterns: Vec::new(),
            include_regex_patterns: Vec::new(),
            exclude_regex_patterns: Vec::new(),
            case_sensitive: false,
            enable_mime_filtering: true,
            mime_config: MimeFilterConfig::default(),
            enable_size_filtering: true,
            size_config: SizeFilterConfig::default(),
            enable_time_filtering: false,
            time_config: TimeFilterConfig::default(),
            enable_content_filtering: false,
            content_config: ContentFilterConfig::default(),
            enable_custom_filters: false,
            custom_filter_configs: Vec::new(),
            performance: PerformanceConfig::default(),
            collect_stats: true,
        }
    }
}

/// MIME type filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimeFilterConfig {
    /// Allowed MIME types (empty means all allowed)
    pub allowed_types: Vec<String>,
    
    /// Blocked MIME types
    pub blocked_types: Vec<String>,
    
    /// Allowed MIME type categories
    pub allowed_categories: Vec<MimeCategory>,
    
    /// Blocked MIME type categories
    pub blocked_categories: Vec<MimeCategory>,
    
    /// Enable MIME type detection from content
    pub detect_from_content: bool,
    
    /// Enable MIME type detection from file extension
    pub detect_from_extension: bool,
    
    /// Fallback MIME type for unknown files
    pub fallback_mime_type: String,
    
    /// Enable strict MIME type validation
    pub strict_validation: bool,
}

impl Default for MimeFilterConfig {
    fn default() -> Self {
        Self {
            allowed_types: Vec::new(),
            blocked_types: vec![
                "application/x-executable".to_string(),
                "application/x-msdownload".to_string(),
            ],
            allowed_categories: Vec::new(),
            blocked_categories: vec![MimeCategory::Executable],
            detect_from_content: true,
            detect_from_extension: true,
            fallback_mime_type: "application/octet-stream".to_string(),
            strict_validation: false,
        }
    }
}

/// MIME type categories for easier filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MimeCategory {
    Text,
    Image,
    Audio,
    Video,
    Application,
    Document,
    Archive,
    Executable,
    Font,
    Model,
    Multipart,
    Message,
    Chemical,
    Unknown,
}

/// File size filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeFilterConfig {
    /// Minimum file size (in bytes)
    pub min_size: Option<u64>,
    
    /// Maximum file size (in bytes)
    pub max_size: Option<u64>,
    
    /// Size ranges (inclusive)
    pub size_ranges: Vec<SizeRange>,
    
    /// Enable size-based sampling
    pub enable_sampling: bool,
    
    /// Sampling configuration
    pub sampling_config: SizeSamplingConfig,
}

impl Default for SizeFilterConfig {
    fn default() -> Self {
        Self {
            min_size: Some(0),
            max_size: Some(100 * 1024 * 1024), // 100MB
            size_ranges: Vec::new(),
            enable_sampling: false,
            sampling_config: SizeSamplingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeRange {
    /// Minimum size (inclusive)
    pub min: u64,
    
    /// Maximum size (inclusive)
    pub max: u64,
    
    /// Include files in this range
    pub include: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeSamplingConfig {
    /// Sampling rate for small files (0.0 to 1.0)
    pub small_file_rate: f64,
    
    /// Sampling rate for medium files (0.0 to 1.0)
    pub medium_file_rate: f64,
    
    /// Sampling rate for large files (0.0 to 1.0)
    pub large_file_rate: f64,
    
    /// Threshold for small files (in bytes)
    pub small_file_threshold: u64,
    
    /// Threshold for large files (in bytes)
    pub large_file_threshold: u64,
}

impl Default for SizeSamplingConfig {
    fn default() -> Self {
        Self {
            small_file_rate: 1.0,
            medium_file_rate: 1.0,
            large_file_rate: 0.1,
            small_file_threshold: 1024,      // 1KB
            large_file_threshold: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Time-based filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFilterConfig {
    /// Filter by creation time
    pub filter_by_created: bool,
    
    /// Filter by modification time
    pub filter_by_modified: bool,
    
    /// Filter by access time
    pub filter_by_accessed: bool,
    
    /// Minimum age (files older than this)
    pub min_age: Option<Duration>,
    
    /// Maximum age (files newer than this)
    pub max_age: Option<Duration>,
    
    /// Specific time ranges
    pub time_ranges: Vec<TimeRange>,
    
    /// Time zone for time-based filtering
    pub timezone: Option<String>,
}

impl Default for TimeFilterConfig {
    fn default() -> Self {
        Self {
            filter_by_created: false,
            filter_by_modified: true,
            filter_by_accessed: false,
            min_age: None,
            max_age: None,
            time_ranges: Vec::new(),
            timezone: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time
    pub start: SystemTime,
    
    /// End time
    pub end: SystemTime,
    
    /// Include files in this range
    pub include: bool,
}

/// Content-based filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFilterConfig {
    /// Enable text content filtering
    pub enable_text_filtering: bool,
    
    /// Text patterns to include
    pub include_text_patterns: Vec<String>,
    
    /// Text patterns to exclude
    pub exclude_text_patterns: Vec<String>,
    
    /// Enable binary content detection
    pub detect_binary_content: bool,
    
    /// Include binary files
    pub include_binary: bool,
    
    /// Enable encoding detection
    pub detect_encoding: bool,
    
    /// Allowed text encodings
    pub allowed_encodings: Vec<String>,
    
    /// Maximum content size to analyze (in bytes)
    pub max_content_size: u64,
    
    /// Content sampling size (in bytes)
    pub content_sample_size: u64,
}

impl Default for ContentFilterConfig {
    fn default() -> Self {
        Self {
            enable_text_filtering: false,
            include_text_patterns: Vec::new(),
            exclude_text_patterns: Vec::new(),
            detect_binary_content: true,
            include_binary: true,
            detect_encoding: false,
            allowed_encodings: vec!["utf-8".to_string(), "ascii".to_string()],
            max_content_size: 1024 * 1024, // 1MB
            content_sample_size: 8192,     // 8KB
        }
    }
}

/// Custom filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFilterConfig {
    /// Filter name
    pub name: String,
    
    /// Filter type
    pub filter_type: CustomFilterType,
    
    /// Filter parameters
    pub parameters: HashMap<String, String>,
    
    /// Enable filter
    pub enabled: bool,
    
    /// Filter priority (lower numbers = higher priority)
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomFilterType {
    /// JavaScript-based filter
    JavaScript,
    
    /// Python-based filter
    Python,
    
    /// External command filter
    Command,
    
    /// Plugin-based filter
    Plugin,
}

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable pattern caching
    pub enable_caching: bool,
    
    /// Cache size limit
    pub cache_size_limit: usize,
    
    /// Cache TTL
    pub cache_ttl: Duration,
    
    /// Enable parallel processing
    pub enable_parallel: bool,
    
    /// Maximum parallel workers
    pub max_parallel_workers: usize,
    
    /// Batch size for processing
    pub batch_size: usize,
    
    /// Enable early termination on first match
    pub early_termination: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_size_limit: 10000,
            cache_ttl: Duration::from_secs(3600),
            enable_parallel: true,
            max_parallel_workers: 4,
            batch_size: 100,
            early_termination: true,
        }
    }
}

/// MIME type filters
#[derive(Debug, Clone, Default)]
pub struct MimeFilters {
    /// Allowed MIME types
    pub allowed_types: HashSet<String>,
    
    /// Blocked MIME types
    pub blocked_types: HashSet<String>,
    
    /// Allowed categories
    pub allowed_categories: HashSet<MimeCategory>,
    
    /// Blocked categories
    pub blocked_categories: HashSet<MimeCategory>,
}

/// File size filters
#[derive(Debug, Clone, Default)]
pub struct SizeFilters {
    /// Minimum size
    pub min_size: Option<u64>,
    
    /// Maximum size
    pub max_size: Option<u64>,
    
    /// Size ranges
    pub ranges: Vec<SizeRange>,
}

/// Time-based filters
#[derive(Debug, Clone, Default)]
pub struct TimeFilters {
    /// Minimum age
    pub min_age: Option<Duration>,
    
    /// Maximum age
    pub max_age: Option<Duration>,
    
    /// Time ranges
    pub ranges: Vec<TimeRange>,
}

/// Content-based filters
#[derive(Debug, Clone, Default)]
pub struct ContentFilters {
    /// Include text patterns
    pub include_patterns: Vec<Regex>,
    
    /// Exclude text patterns
    pub exclude_patterns: Vec<Regex>,
    
    /// Allowed encodings
    pub allowed_encodings: HashSet<String>,
}

/// Custom filter function
#[derive(Debug, Clone)]
pub struct CustomFilter {
    /// Filter name
    pub name: String,
    
    /// Filter function
    pub filter_fn: fn(&Path, &FileMetadata) -> Result<bool>,
    
    /// Filter priority
    pub priority: u32,
    
    /// Filter enabled
    pub enabled: bool,
}

/// File metadata for filtering
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File path
    pub path: PathBuf,
    
    /// File size
    pub size: Option<u64>,
    
    /// Creation time
    pub created: Option<SystemTime>,
    
    /// Modification time
    pub modified: Option<SystemTime>,
    
    /// Access time
    pub accessed: Option<SystemTime>,
    
    /// MIME type
    pub mime_type: Option<String>,
    
    /// MIME category
    pub mime_category: Option<MimeCategory>,
    
    /// Is directory
    pub is_dir: bool,
    
    /// Is symlink
    pub is_symlink: bool,
    
    /// File permissions
    pub permissions: Option<u32>,
    
    /// Content sample (first few bytes)
    pub content_sample: Option<Vec<u8>>,
    
    /// Text encoding (if detected)
    pub encoding: Option<String>,
    
    /// Is binary file
    pub is_binary: Option<bool>,
}

/// Pattern matcher statistics
#[derive(Debug, Clone, Default)]
pub struct PatternMatcherStats {
    /// Total files processed
    pub files_processed: u64,
    
    /// Files matched
    pub files_matched: u64,
    
    /// Files excluded
    pub files_excluded: u64,
    
    /// Cache hits
    pub cache_hits: u64,
    
    /// Cache misses
    pub cache_misses: u64,
    
    /// Average processing time per file
    pub avg_processing_time: Duration,
    
    /// Total processing time
    pub total_processing_time: Duration,
    
    /// Error count
    pub error_count: u64,
    
    /// Filter statistics
    pub filter_stats: HashMap<String, FilterStats>,
}

#[derive(Debug, Clone, Default)]
pub struct FilterStats {
    /// Number of times filter was applied
    pub applications: u64,
    
    /// Number of matches
    pub matches: u64,
    
    /// Number of exclusions
    pub exclusions: u64,
    
    /// Average execution time
    pub avg_execution_time: Duration,
    
    /// Error count
    pub errors: u64,
}

/// Match result
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Whether the file matches
    pub matches: bool,
    
    /// Reason for match/exclusion
    pub reason: String,
    
    /// Filter that made the decision
    pub filter_name: String,
    
    /// Processing time
    pub processing_time: Duration,
    
    /// File metadata
    pub metadata: FileMetadata,
}

impl EnhancedPatternMatcher {
    /// Create a new enhanced pattern matcher
    pub fn new(config: PatternMatcherConfig) -> Result<Self> {
        let mut matcher = Self {
            config: config.clone(),
            include_globs: None,
            exclude_globs: None,
            include_regex: Vec::new(),
            exclude_regex: Vec::new(),
            mime_filters: MimeFilters::default(),
            size_filters: SizeFilters::default(),
            time_filters: TimeFilters::default(),
            content_filters: ContentFilters::default(),
            custom_filters: Vec::new(),
            stats: PatternMatcherStats::default(),
        };
        
        matcher.compile_patterns()?;
        matcher.setup_filters()?;
        
        Ok(matcher)
    }
    
    /// Compile glob and regex patterns
    fn compile_patterns(&mut self) -> Result<()> {
        // Compile include globs
        if !self.config.include_patterns.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pattern in &self.config.include_patterns {
                let glob = if self.config.case_sensitive {
                    Glob::new(pattern)?
                } else {
                    Glob::new(&pattern.to_lowercase())?
                };
                builder.add(glob);
            }
            self.include_globs = Some(builder.build()?);
        }
        
        // Compile exclude globs
        if !self.config.exclude_patterns.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pattern in &self.config.exclude_patterns {
                let glob = if self.config.case_sensitive {
                    Glob::new(pattern)?
                } else {
                    Glob::new(&pattern.to_lowercase())?
                };
                builder.add(glob);
            }
            self.exclude_globs = Some(builder.build()?);
        }
        
        // Compile include regex patterns
        for pattern in &self.config.include_regex_patterns {
            let regex = if self.config.case_sensitive {
                Regex::new(pattern)?
            } else {
                Regex::new(&format!("(?i){}", pattern))?
            };
            self.include_regex.push(regex);
        }
        
        // Compile exclude regex patterns
        for pattern in &self.config.exclude_regex_patterns {
            let regex = if self.config.case_sensitive {
                Regex::new(pattern)?
            } else {
                Regex::new(&format!("(?i){}", pattern))?
            };
            self.exclude_regex.push(regex);
        }
        
        Ok(())
    }
    
    /// Set up various filters
    fn setup_filters(&mut self) -> Result<()> {
        // Set up MIME filters
        if self.config.enable_mime_filtering {
            self.mime_filters.allowed_types = self.config.mime_config.allowed_types.iter().cloned().collect();
            self.mime_filters.blocked_types = self.config.mime_config.blocked_types.iter().cloned().collect();
            self.mime_filters.allowed_categories = self.config.mime_config.allowed_categories.iter().cloned().collect();
            self.mime_filters.blocked_categories = self.config.mime_config.blocked_categories.iter().cloned().collect();
        }
        
        // Set up size filters
        if self.config.enable_size_filtering {
            self.size_filters.min_size = self.config.size_config.min_size;
            self.size_filters.max_size = self.config.size_config.max_size;
            self.size_filters.ranges = self.config.size_config.size_ranges.clone();
        }
        
        // Set up time filters
        if self.config.enable_time_filtering {
            self.time_filters.min_age = self.config.time_config.min_age;
            self.time_filters.max_age = self.config.time_config.max_age;
            self.time_filters.ranges = self.config.time_config.time_ranges.clone();
        }
        
        // Set up content filters
        if self.config.enable_content_filtering {
            for pattern in &self.config.content_config.include_text_patterns {
                let regex = if self.config.case_sensitive {
                    Regex::new(pattern)?
                } else {
                    Regex::new(&format!("(?i){}", pattern))?
                };
                self.content_filters.include_patterns.push(regex);
            }
            
            for pattern in &self.config.content_config.exclude_text_patterns {
                let regex = if self.config.case_sensitive {
                    Regex::new(pattern)?
                } else {
                    Regex::new(&format!("(?i){}", pattern))?
                };
                self.content_filters.exclude_patterns.push(regex);
            }
            
            self.content_filters.allowed_encodings = self.config.content_config.allowed_encodings.iter().cloned().collect();
        }
        
        Ok(())
    }
    
    /// Check if a file matches the patterns
    pub fn matches(&mut self, path: &Path) -> Result<MatchResult> {
        let start_time = std::time::Instant::now();
        
        // Gather file metadata
        let metadata = self.gather_metadata(path)?;
        
        // Apply filters in order of priority
        let result = self.apply_filters(&metadata)?;
        
        let processing_time = start_time.elapsed();
        
        // Update statistics
        if self.config.collect_stats {
            self.update_stats(&result, processing_time);
        }
        
        Ok(MatchResult {
            matches: result.0,
            reason: result.1,
            filter_name: result.2,
            processing_time,
            metadata,
        })
    }
    
    /// Gather file metadata
    fn gather_metadata(&self, path: &Path) -> Result<FileMetadata> {
        let mut metadata = FileMetadata {
            path: path.to_path_buf(),
            size: None,
            created: None,
            modified: None,
            accessed: None,
            mime_type: None,
            mime_category: None,
            is_dir: path.is_dir(),
            is_symlink: path.is_symlink(),
            permissions: None,
            content_sample: None,
            encoding: None,
            is_binary: None,
        };
        
        // Get file system metadata
        if let Ok(fs_metadata) = std::fs::metadata(path) {
            metadata.size = Some(fs_metadata.len());
            
            if let Ok(created) = fs_metadata.created() {
                metadata.created = Some(created);
            }
            
            if let Ok(modified) = fs_metadata.modified() {
                metadata.modified = Some(modified);
            }
            
            if let Ok(accessed) = fs_metadata.accessed() {
                metadata.accessed = Some(accessed);
            }
        }
        
        // Detect MIME type
        if self.config.enable_mime_filtering && !metadata.is_dir {
            metadata.mime_type = self.detect_mime_type(path)?;
            metadata.mime_category = metadata.mime_type.as_ref().map(|mime| self.categorize_mime_type(mime));
        }
        
        // Read content sample for content-based filtering
        if self.config.enable_content_filtering && !metadata.is_dir {
            metadata.content_sample = self.read_content_sample(path)?;
            
            if let Some(sample) = &metadata.content_sample {
                metadata.is_binary = Some(self.is_binary_content(sample));
                
                if !metadata.is_binary.unwrap_or(true) {
                    metadata.encoding = self.detect_encoding(sample);
                }
            }
        }
        
        Ok(metadata)
    }
    
    /// Apply all filters to determine if file matches
    fn apply_filters(&mut self, metadata: &FileMetadata) -> Result<(bool, String, String)> {
        let path_str = metadata.path.to_string_lossy();
        let path_str = if self.config.case_sensitive {
            path_str.to_string()
        } else {
            path_str.to_lowercase()
        };
        
        // 1. Apply exclude glob patterns first
        if let Some(exclude_globs) = &self.exclude_globs {
            if exclude_globs.is_match(&path_str) {
                return Ok((false, "Excluded by glob pattern".to_string(), "exclude_glob".to_string()));
            }
        }
        
        // 2. Apply exclude regex patterns
        for (i, regex) in self.exclude_regex.iter().enumerate() {
            if regex.is_match(&path_str) {
                return Ok((false, format!("Excluded by regex pattern {}", i), "exclude_regex".to_string()));
            }
        }
        
        // 3. Apply MIME type filters
        if self.config.enable_mime_filtering {
            if let Some(result) = self.apply_mime_filters(metadata)? {
                return Ok(result);
            }
        }
        
        // 4. Apply size filters
        if self.config.enable_size_filtering {
            if let Some(result) = self.apply_size_filters(metadata)? {
                return Ok(result);
            }
        }
        
        // 5. Apply time filters
        if self.config.enable_time_filtering {
            if let Some(result) = self.apply_time_filters(metadata)? {
                return Ok(result);
            }
        }
        
        // 6. Apply content filters
        if self.config.enable_content_filtering {
            if let Some(result) = self.apply_content_filters(metadata)? {
                return Ok(result);
            }
        }
        
        // 7. Apply custom filters
        if self.config.enable_custom_filters {
            if let Some(result) = self.apply_custom_filters(metadata)? {
                return Ok(result);
            }
        }
        
        // 8. Apply include patterns (must match at least one)
        let mut include_match = false;
        
        // Check include globs
        if let Some(include_globs) = &self.include_globs {
            if include_globs.is_match(&path_str) {
                include_match = true;
            }
        } else if self.config.include_patterns.is_empty() {
            include_match = true; // No include patterns means include all
        }
        
        // Check include regex
        if !include_match && !self.include_regex.is_empty() {
            for regex in &self.include_regex {
                if regex.is_match(&path_str) {
                    include_match = true;
                    break;
                }
            }
        }
        
        if include_match {
            Ok((true, "Matched include pattern".to_string(), "include_pattern".to_string()))
        } else {
            Ok((false, "No include pattern matched".to_string(), "no_include_match".to_string()))
        }
    }
    
    /// Apply MIME type filters
    fn apply_mime_filters(&self, metadata: &FileMetadata) -> Result<Option<(bool, String, String)>> {
        if let Some(mime_type) = &metadata.mime_type {
            // Check blocked types
            if self.mime_filters.blocked_types.contains(mime_type) {
                return Ok(Some((false, format!("Blocked MIME type: {}", mime_type), "mime_blocked".to_string())));
            }
            
            // Check blocked categories
            if let Some(category) = &metadata.mime_category {
                if self.mime_filters.blocked_categories.contains(category) {
                    return Ok(Some((false, format!("Blocked MIME category: {:?}", category), "mime_category_blocked".to_string())));
                }
            }
            
            // Check allowed types (if specified)
            if !self.mime_filters.allowed_types.is_empty() && !self.mime_filters.allowed_types.contains(mime_type) {
                return Ok(Some((false, format!("MIME type not in allowed list: {}", mime_type), "mime_not_allowed".to_string())));
            }
            
            // Check allowed categories (if specified)
            if !self.mime_filters.allowed_categories.is_empty() {
                if let Some(category) = &metadata.mime_category {
                    if !self.mime_filters.allowed_categories.contains(category) {
                        return Ok(Some((false, format!("MIME category not in allowed list: {:?}", category), "mime_category_not_allowed".to_string())));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Apply size filters
    fn apply_size_filters(&self, metadata: &FileMetadata) -> Result<Option<(bool, String, String)>> {
        if let Some(size) = metadata.size {
            // Check minimum size
            if let Some(min_size) = self.size_filters.min_size {
                if size < min_size {
                    return Ok(Some((false, format!("File too small: {} < {}", size, min_size), "size_too_small".to_string())));
                }
            }
            
            // Check maximum size
            if let Some(max_size) = self.size_filters.max_size {
                if size > max_size {
                    return Ok(Some((false, format!("File too large: {} > {}", size, max_size), "size_too_large".to_string())));
                }
            }
            
            // Check size ranges
            for range in &self.size_filters.ranges {
                if size >= range.min && size <= range.max {
                    if !range.include {
                        return Ok(Some((false, format!("File size in excluded range: {}-{}", range.min, range.max), "size_range_excluded".to_string())));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Apply time filters
    fn apply_time_filters(&self, metadata: &FileMetadata) -> Result<Option<(bool, String, String)>> {
        let now = SystemTime::now();
        
        // Check modification time
        if self.config.time_config.filter_by_modified {
            if let Some(modified) = metadata.modified {
                if let Some(min_age) = self.time_filters.min_age {
                    if let Ok(age) = now.duration_since(modified) {
                        if age < min_age {
                            return Ok(Some((false, format!("File too new: {:?} < {:?}", age, min_age), "too_new".to_string())));
                        }
                    }
                }
                
                if let Some(max_age) = self.time_filters.max_age {
                    if let Ok(age) = now.duration_since(modified) {
                        if age > max_age {
                            return Ok(Some((false, format!("File too old: {:?} > {:?}", age, max_age), "too_old".to_string())));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Apply content filters
    fn apply_content_filters(&self, metadata: &FileMetadata) -> Result<Option<(bool, String, String)>> {
        if let Some(sample) = &metadata.content_sample {
            // Check if binary content is allowed
            if let Some(is_binary) = metadata.is_binary {
                if is_binary && !self.config.content_config.include_binary {
                    return Ok(Some((false, "Binary content not allowed".to_string(), "binary_excluded".to_string())));
                }
            }
            
            // Check encoding
            if let Some(encoding) = &metadata.encoding {
                if !self.content_filters.allowed_encodings.is_empty() && !self.content_filters.allowed_encodings.contains(encoding) {
                    return Ok(Some((false, format!("Encoding not allowed: {}", encoding), "encoding_not_allowed".to_string())));
                }
            }
            
            // Check text patterns (only for text files)
            if !metadata.is_binary.unwrap_or(true) {
                let content_str = String::from_utf8_lossy(sample);
                
                // Check exclude patterns first
                for (i, pattern) in self.content_filters.exclude_patterns.iter().enumerate() {
                    if pattern.is_match(&content_str) {
                        return Ok(Some((false, format!("Content matches exclude pattern {}", i), "content_exclude_pattern".to_string())));
                    }
                }
                
                // Check include patterns
                if !self.content_filters.include_patterns.is_empty() {
                    let mut found_match = false;
                    for pattern in &self.content_filters.include_patterns {
                        if pattern.is_match(&content_str) {
                            found_match = true;
                            break;
                        }
                    }
                    
                    if !found_match {
                        return Ok(Some((false, "Content doesn't match any include pattern".to_string(), "content_no_include_match".to_string())));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Apply custom filters
    fn apply_custom_filters(&self, metadata: &FileMetadata) -> Result<Option<(bool, String, String)>> {
        for filter in &self.custom_filters {
            if filter.enabled {
                match (filter.filter_fn)(&metadata.path, metadata) {
                    Ok(matches) => {
                        if !matches {
                            return Ok(Some((false, format!("Excluded by custom filter: {}", filter.name), filter.name.clone())));
                        }
                    }
                    Err(e) => {
                        log::warn!("Custom filter '{}' error: {}", filter.name, e);
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Detect MIME type
    fn detect_mime_type(&self, path: &Path) -> Result<Option<String>> {
        let mut mime_type = None;
        
        // Detect from extension
        if self.config.mime_config.detect_from_extension {
            mime_type = MimeGuess::from_path(path).first().map(|m| m.to_string());
        }
        
        // Detect from content if needed
        if mime_type.is_none() && self.config.mime_config.detect_from_content {
            if let Ok(sample) = self.read_content_sample(path)? {
                // Use a more sophisticated MIME detection library here
                // For now, just basic detection
                if self.is_binary_content(&sample) {
                    mime_type = Some("application/octet-stream".to_string());
                } else {
                    mime_type = Some("text/plain".to_string());
                }
            }
        }
        
        Ok(mime_type.or_else(|| Some(self.config.mime_config.fallback_mime_type.clone())))
    }
    
    /// Categorize MIME type
    fn categorize_mime_type(&self, mime_type: &str) -> MimeCategory {
        match mime_type.split('/').next().unwrap_or("") {
            "text" => MimeCategory::Text,
            "image" => MimeCategory::Image,
            "audio" => MimeCategory::Audio,
            "video" => MimeCategory::Video,
            "application" => {
                if mime_type.contains("pdf") || mime_type.contains("document") || mime_type.contains("word") {
                    MimeCategory::Document
                } else if mime_type.contains("zip") || mime_type.contains("tar") || mime_type.contains("archive") {
                    MimeCategory::Archive
                } else if mime_type.contains("executable") || mime_type.contains("msdownload") {
                    MimeCategory::Executable
                } else {
                    MimeCategory::Application
                }
            }
            "font" => MimeCategory::Font,
            "model" => MimeCategory::Model,
            "multipart" => MimeCategory::Multipart,
            "message" => MimeCategory::Message,
            "chemical" => MimeCategory::Chemical,
            _ => MimeCategory::Unknown,
        }
    }
    
    /// Read content sample
    fn read_content_sample(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        if path.is_dir() {
            return Ok(None);
        }
        
        let sample_size = self.config.content_config.content_sample_size as usize;
        let mut buffer = vec![0u8; sample_size];
        
        match std::fs::File::open(path) {
            Ok(mut file) => {
                use std::io::Read;
                match file.read(&mut buffer) {
                    Ok(bytes_read) => {
                        buffer.truncate(bytes_read);
                        Ok(Some(buffer))
                    }
                    Err(_) => Ok(None),
                }
            }
            Err(_) => Ok(None),
        }
    }
    
    /// Check if content is binary
    fn is_binary_content(&self, content: &[u8]) -> bool {
        // Simple heuristic: if more than 30% of bytes are non-printable, consider it binary
        let non_printable_count = content.iter()
            .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
            .count();
        
        let ratio = non_printable_count as f64 / content.len() as f64;
        ratio > 0.3
    }
    
    /// Detect text encoding
    fn detect_encoding(&self, _content: &[u8]) -> Option<String> {
        // Placeholder for encoding detection
        // In a real implementation, you'd use a library like chardet
        Some("utf-8".to_string())
    }
    
    /// Update statistics
    fn update_stats(&mut self, result: &(bool, String, String), processing_time: Duration) {
        self.stats.files_processed += 1;
        
        if result.0 {
            self.stats.files_matched += 1;
        } else {
            self.stats.files_excluded += 1;
        }
        
        self.stats.total_processing_time += processing_time;
        self.stats.avg_processing_time = Duration::from_nanos(
            self.stats.total_processing_time.as_nanos() as u64 / self.stats.files_processed
        );
        
        // Update filter-specific statistics
        let filter_stats = self.stats.filter_stats.entry(result.2.clone()).or_default();
        filter_stats.applications += 1;
        
        if result.0 {
            filter_stats.matches += 1;
        } else {
            filter_stats.exclusions += 1;
        }
        
        filter_stats.avg_execution_time = Duration::from_nanos(
            (filter_stats.avg_execution_time.as_nanos() as u64 + processing_time.as_nanos() as u64) / 2
        );
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> &PatternMatcherStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = PatternMatcherStats::default();
    }
}