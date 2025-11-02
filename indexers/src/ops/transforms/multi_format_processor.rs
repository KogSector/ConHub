use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use base64::{Engine as _, engine::general_purpose};
use mime_guess::MimeGuess;

/// Multi-format processor for handling various file types
pub struct MultiFormatProcessor {
    /// Configuration for processing
    config: ProcessorConfig,
    
    /// Format handlers
    handlers: HashMap<String, Box<dyn FormatHandler>>,
    
    /// Processing statistics
    stats: ProcessorStats,
    
    /// Cache for processed content
    cache: Option<ProcessorCache>,
}

/// Configuration for multi-format processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// Enable PDF processing
    pub enable_pdf: bool,
    
    /// PDF processing configuration
    pub pdf_config: PdfConfig,
    
    /// Enable image processing
    pub enable_images: bool,
    
    /// Image processing configuration
    pub image_config: ImageConfig,
    
    /// Enable document processing (Word, Excel, etc.)
    pub enable_documents: bool,
    
    /// Document processing configuration
    pub document_config: DocumentConfig,
    
    /// Enable text processing
    pub enable_text: bool,
    
    /// Text processing configuration
    pub text_config: TextConfig,
    
    /// Enable archive processing
    pub enable_archives: bool,
    
    /// Archive processing configuration
    pub archive_config: ArchiveConfig,
    
    /// Enable web content processing
    pub enable_web: bool,
    
    /// Web content processing configuration
    pub web_config: WebConfig,
    
    /// Enable caching
    pub enable_caching: bool,
    
    /// Cache configuration
    pub cache_config: CacheConfig,
    
    /// Performance settings
    pub performance: PerformanceConfig,
    
    /// Output format preferences
    pub output_format: OutputFormat,
    
    /// Error handling configuration
    pub error_handling: ErrorHandlingConfig,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            enable_pdf: true,
            pdf_config: PdfConfig::default(),
            enable_images: true,
            image_config: ImageConfig::default(),
            enable_documents: true,
            document_config: DocumentConfig::default(),
            enable_text: true,
            text_config: TextConfig::default(),
            enable_archives: false,
            archive_config: ArchiveConfig::default(),
            enable_web: true,
            web_config: WebConfig::default(),
            enable_caching: true,
            cache_config: CacheConfig::default(),
            performance: PerformanceConfig::default(),
            output_format: OutputFormat::default(),
            error_handling: ErrorHandlingConfig::default(),
        }
    }
}

/// PDF processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfConfig {
    /// Extract text from PDF
    pub extract_text: bool,
    
    /// Convert PDF pages to images
    pub convert_to_images: bool,
    
    /// Image format for PDF conversion
    pub image_format: ImageFormat,
    
    /// Image resolution (DPI)
    pub image_dpi: u32,
    
    /// Image quality (0-100)
    pub image_quality: u8,
    
    /// Extract metadata
    pub extract_metadata: bool,
    
    /// Extract annotations
    pub extract_annotations: bool,
    
    /// Extract forms
    pub extract_forms: bool,
    
    /// Maximum pages to process (0 = all)
    pub max_pages: u32,
    
    /// Page range to process
    pub page_range: Option<PageRange>,
    
    /// OCR configuration for scanned PDFs
    pub ocr_config: Option<OcrConfig>,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            extract_text: true,
            convert_to_images: true,
            image_format: ImageFormat::Png,
            image_dpi: 150,
            image_quality: 85,
            extract_metadata: true,
            extract_annotations: false,
            extract_forms: false,
            max_pages: 0,
            page_range: None,
            ocr_config: None,
        }
    }
}

/// Image processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// Extract metadata (EXIF, etc.)
    pub extract_metadata: bool,
    
    /// Generate thumbnails
    pub generate_thumbnails: bool,
    
    /// Thumbnail size
    pub thumbnail_size: (u32, u32),
    
    /// Thumbnail format
    pub thumbnail_format: ImageFormat,
    
    /// Extract text using OCR
    pub extract_text_ocr: bool,
    
    /// OCR configuration
    pub ocr_config: Option<OcrConfig>,
    
    /// Supported image formats
    pub supported_formats: Vec<ImageFormat>,
    
    /// Maximum image size to process
    pub max_size: Option<(u32, u32)>,
    
    /// Image quality for conversions
    pub quality: u8,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            extract_metadata: true,
            generate_thumbnails: true,
            thumbnail_size: (200, 200),
            thumbnail_format: ImageFormat::Jpeg,
            extract_text_ocr: false,
            ocr_config: None,
            supported_formats: vec![
                ImageFormat::Jpeg,
                ImageFormat::Png,
                ImageFormat::Gif,
                ImageFormat::Bmp,
                ImageFormat::Tiff,
                ImageFormat::WebP,
            ],
            max_size: Some((4096, 4096)),
            quality: 85,
        }
    }
}

/// Document processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentConfig {
    /// Extract text content
    pub extract_text: bool,
    
    /// Extract metadata
    pub extract_metadata: bool,
    
    /// Extract structure (headings, tables, etc.)
    pub extract_structure: bool,
    
    /// Extract embedded objects
    pub extract_embedded: bool,
    
    /// Supported document formats
    pub supported_formats: Vec<DocumentFormat>,
    
    /// Maximum document size to process
    pub max_size: Option<u64>,
    
    /// Password handling for protected documents
    pub password_handling: PasswordHandling,
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self {
            extract_text: true,
            extract_metadata: true,
            extract_structure: true,
            extract_embedded: false,
            supported_formats: vec![
                DocumentFormat::Word,
                DocumentFormat::Excel,
                DocumentFormat::PowerPoint,
                DocumentFormat::OpenDocument,
                DocumentFormat::RichText,
            ],
            max_size: Some(100 * 1024 * 1024), // 100MB
            password_handling: PasswordHandling::Skip,
        }
    }
}

/// Text processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    /// Detect encoding automatically
    pub auto_detect_encoding: bool,
    
    /// Supported encodings
    pub supported_encodings: Vec<String>,
    
    /// Fallback encoding
    pub fallback_encoding: String,
    
    /// Extract language information
    pub detect_language: bool,
    
    /// Extract structure (markdown, etc.)
    pub extract_structure: bool,
    
    /// Maximum file size to process
    pub max_size: Option<u64>,
    
    /// Line ending normalization
    pub normalize_line_endings: bool,
    
    /// Remove BOM (Byte Order Mark)
    pub remove_bom: bool,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            auto_detect_encoding: true,
            supported_encodings: vec![
                "utf-8".to_string(),
                "utf-16".to_string(),
                "ascii".to_string(),
                "iso-8859-1".to_string(),
            ],
            fallback_encoding: "utf-8".to_string(),
            detect_language: false,
            extract_structure: false,
            max_size: Some(10 * 1024 * 1024), // 10MB
            normalize_line_endings: true,
            remove_bom: true,
        }
    }
}

/// Archive processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    /// Extract archive contents
    pub extract_contents: bool,
    
    /// Process extracted files recursively
    pub recursive_processing: bool,
    
    /// Maximum extraction depth
    pub max_depth: u32,
    
    /// Maximum number of files to extract
    pub max_files: Option<u32>,
    
    /// Maximum total extracted size
    pub max_extracted_size: Option<u64>,
    
    /// Supported archive formats
    pub supported_formats: Vec<ArchiveFormat>,
    
    /// Password handling for protected archives
    pub password_handling: PasswordHandling,
    
    /// Extract metadata only (don't extract contents)
    pub metadata_only: bool,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            extract_contents: false,
            recursive_processing: false,
            max_depth: 3,
            max_files: Some(1000),
            max_extracted_size: Some(1024 * 1024 * 1024), // 1GB
            supported_formats: vec![
                ArchiveFormat::Zip,
                ArchiveFormat::Tar,
                ArchiveFormat::Gzip,
                ArchiveFormat::SevenZ,
            ],
            password_handling: PasswordHandling::Skip,
            metadata_only: true,
        }
    }
}

/// Web content processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    /// Extract text from HTML
    pub extract_text: bool,
    
    /// Extract metadata
    pub extract_metadata: bool,
    
    /// Extract links
    pub extract_links: bool,
    
    /// Extract images
    pub extract_images: bool,
    
    /// Extract structured data (JSON-LD, microdata)
    pub extract_structured_data: bool,
    
    /// Clean HTML (remove scripts, styles, etc.)
    pub clean_html: bool,
    
    /// Convert to markdown
    pub convert_to_markdown: bool,
    
    /// Supported web formats
    pub supported_formats: Vec<WebFormat>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            extract_text: true,
            extract_metadata: true,
            extract_links: false,
            extract_images: false,
            extract_structured_data: false,
            clean_html: true,
            convert_to_markdown: false,
            supported_formats: vec![
                WebFormat::Html,
                WebFormat::Xml,
                WebFormat::Json,
                WebFormat::Css,
                WebFormat::JavaScript,
            ],
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache directory
    pub cache_dir: Option<PathBuf>,
    
    /// Maximum cache size
    pub max_size: u64,
    
    /// Cache TTL
    pub ttl: std::time::Duration,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Cache key strategy
    pub key_strategy: CacheKeyStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_dir: None,
            max_size: 1024 * 1024 * 1024, // 1GB
            ttl: std::time::Duration::from_secs(24 * 60 * 60), // 24 hours
            enable_compression: true,
            key_strategy: CacheKeyStrategy::ContentHash,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable parallel processing
    pub enable_parallel: bool,
    
    /// Maximum parallel workers
    pub max_workers: usize,
    
    /// Processing timeout
    pub timeout: std::time::Duration,
    
    /// Memory limit per worker
    pub memory_limit: Option<u64>,
    
    /// Enable streaming for large files
    pub enable_streaming: bool,
    
    /// Chunk size for streaming
    pub chunk_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_parallel: true,
            max_workers: 4,
            timeout: std::time::Duration::from_secs(300), // 5 minutes
            memory_limit: Some(512 * 1024 * 1024), // 512MB
            enable_streaming: true,
            chunk_size: 64 * 1024, // 64KB
        }
    }
}

/// Output format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    /// Include original content
    pub include_original: bool,
    
    /// Include extracted text
    pub include_text: bool,
    
    /// Include metadata
    pub include_metadata: bool,
    
    /// Include structure information
    pub include_structure: bool,
    
    /// Include thumbnails/previews
    pub include_previews: bool,
    
    /// Output encoding
    pub encoding: String,
    
    /// Preserve formatting
    pub preserve_formatting: bool,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self {
            include_original: false,
            include_text: true,
            include_metadata: true,
            include_structure: false,
            include_previews: false,
            encoding: "utf-8".to_string(),
            preserve_formatting: false,
        }
    }
}

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    /// Continue processing on errors
    pub continue_on_error: bool,
    
    /// Maximum retries
    pub max_retries: u32,
    
    /// Retry delay
    pub retry_delay: std::time::Duration,
    
    /// Fallback to basic text extraction
    pub fallback_to_text: bool,
    
    /// Log errors
    pub log_errors: bool,
    
    /// Include error information in output
    pub include_errors: bool,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            continue_on_error: true,
            max_retries: 3,
            retry_delay: std::time::Duration::from_secs(1),
            fallback_to_text: true,
            log_errors: true,
            include_errors: false,
        }
    }
}

/// Supported image formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    Bmp,
    Tiff,
    WebP,
    Svg,
    Ico,
    Avif,
    Heic,
}

/// Supported document formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentFormat {
    Word,
    Excel,
    PowerPoint,
    OpenDocument,
    RichText,
    PlainText,
    Csv,
    Tsv,
}

/// Supported archive formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    Gzip,
    Bzip2,
    Xz,
    SevenZ,
    Rar,
}

/// Supported web formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WebFormat {
    Html,
    Xml,
    Json,
    Css,
    JavaScript,
    Markdown,
}

/// Password handling strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PasswordHandling {
    /// Skip password-protected files
    Skip,
    
    /// Try common passwords
    TryCommon,
    
    /// Use provided password list
    UseList(Vec<String>),
    
    /// Prompt for password (interactive)
    Prompt,
}

/// Cache key strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CacheKeyStrategy {
    /// Use file path
    FilePath,
    
    /// Use content hash
    ContentHash,
    
    /// Use file path + modification time
    PathAndMtime,
    
    /// Use content hash + configuration hash
    ContentAndConfig,
}

/// Page range for PDF processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRange {
    /// Start page (1-based)
    pub start: u32,
    
    /// End page (1-based, inclusive)
    pub end: u32,
}

/// OCR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrConfig {
    /// OCR engine to use
    pub engine: OcrEngine,
    
    /// Languages to detect
    pub languages: Vec<String>,
    
    /// OCR mode
    pub mode: OcrMode,
    
    /// Confidence threshold
    pub confidence_threshold: f32,
    
    /// Enable preprocessing
    pub enable_preprocessing: bool,
    
    /// Preprocessing options
    pub preprocessing: OcrPreprocessing,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            engine: OcrEngine::Tesseract,
            languages: vec!["eng".to_string()],
            mode: OcrMode::Auto,
            confidence_threshold: 0.5,
            enable_preprocessing: true,
            preprocessing: OcrPreprocessing::default(),
        }
    }
}

/// OCR engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OcrEngine {
    Tesseract,
    EasyOcr,
    PaddleOcr,
    AzureCognitive,
    GoogleVision,
    AwsTextract,
}

/// OCR mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OcrMode {
    /// Automatic detection
    Auto,
    
    /// Single text line
    SingleLine,
    
    /// Single word
    SingleWord,
    
    /// Single character
    SingleChar,
    
    /// Sparse text
    SparseText,
    
    /// Uniform text block
    UniformBlock,
}

/// OCR preprocessing options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrPreprocessing {
    /// Resize image
    pub resize: bool,
    
    /// Target size for resizing
    pub target_size: Option<(u32, u32)>,
    
    /// Convert to grayscale
    pub grayscale: bool,
    
    /// Apply noise reduction
    pub denoise: bool,
    
    /// Enhance contrast
    pub enhance_contrast: bool,
    
    /// Deskew image
    pub deskew: bool,
    
    /// Remove borders
    pub remove_borders: bool,
}

impl Default for OcrPreprocessing {
    fn default() -> Self {
        Self {
            resize: false,
            target_size: None,
            grayscale: true,
            denoise: true,
            enhance_contrast: true,
            deskew: true,
            remove_borders: false,
        }
    }
}

/// Format handler trait
pub trait FormatHandler: Send + Sync {
    /// Check if this handler can process the given file
    fn can_handle(&self, path: &Path, mime_type: &str) -> bool;
    
    /// Process the file and return extracted content
    fn process(&self, path: &Path, config: &ProcessorConfig) -> Result<ProcessedContent>;
    
    /// Get supported MIME types
    fn supported_mime_types(&self) -> Vec<String>;
    
    /// Get handler name
    fn name(&self) -> &str;
}

/// Processed content result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedContent {
    /// Original file path
    pub file_path: PathBuf,
    
    /// MIME type
    pub mime_type: String,
    
    /// File size
    pub file_size: u64,
    
    /// Processing timestamp
    pub processed_at: std::time::SystemTime,
    
    /// Extracted text content
    pub text_content: Option<String>,
    
    /// Extracted metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Extracted structure
    pub structure: Option<ContentStructure>,
    
    /// Generated previews/thumbnails
    pub previews: Vec<Preview>,
    
    /// Extracted pages (for multi-page documents)
    pub pages: Vec<PageContent>,
    
    /// Processing errors
    pub errors: Vec<ProcessingError>,
    
    /// Processing statistics
    pub stats: ProcessingStats,
    
    /// Original content (if requested)
    pub original_content: Option<Vec<u8>>,
}

/// Content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStructure {
    /// Document outline/headings
    pub outline: Vec<OutlineItem>,
    
    /// Tables
    pub tables: Vec<Table>,
    
    /// Images
    pub images: Vec<ImageInfo>,
    
    /// Links
    pub links: Vec<Link>,
    
    /// Forms
    pub forms: Vec<Form>,
    
    /// Annotations
    pub annotations: Vec<Annotation>,
}

/// Outline item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineItem {
    /// Heading level
    pub level: u32,
    
    /// Heading text
    pub text: String,
    
    /// Page number (if applicable)
    pub page: Option<u32>,
    
    /// Position in document
    pub position: Option<Position>,
}

/// Table structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Table caption
    pub caption: Option<String>,
    
    /// Number of rows
    pub rows: u32,
    
    /// Number of columns
    pub columns: u32,
    
    /// Table data
    pub data: Vec<Vec<String>>,
    
    /// Page number (if applicable)
    pub page: Option<u32>,
}

/// Image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// Image description/alt text
    pub description: Option<String>,
    
    /// Image dimensions
    pub dimensions: Option<(u32, u32)>,
    
    /// Image format
    pub format: Option<String>,
    
    /// Image data (base64 encoded)
    pub data: Option<String>,
    
    /// Page number (if applicable)
    pub page: Option<u32>,
}

/// Link information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// Link text
    pub text: String,
    
    /// Link URL
    pub url: String,
    
    /// Link type
    pub link_type: LinkType,
    
    /// Page number (if applicable)
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkType {
    Internal,
    External,
    Email,
    Phone,
}

/// Form information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form {
    /// Form name
    pub name: Option<String>,
    
    /// Form fields
    pub fields: Vec<FormField>,
    
    /// Page number (if applicable)
    pub page: Option<u32>,
}

/// Form field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    /// Field name
    pub name: String,
    
    /// Field type
    pub field_type: FormFieldType,
    
    /// Field value
    pub value: Option<String>,
    
    /// Field options (for select fields)
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormFieldType {
    Text,
    Number,
    Email,
    Password,
    Checkbox,
    Radio,
    Select,
    Textarea,
    File,
}

/// Annotation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// Annotation type
    pub annotation_type: AnnotationType,
    
    /// Annotation content
    pub content: String,
    
    /// Author
    pub author: Option<String>,
    
    /// Creation date
    pub created: Option<std::time::SystemTime>,
    
    /// Page number (if applicable)
    pub page: Option<u32>,
    
    /// Position
    pub position: Option<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    Note,
    Highlight,
    Underline,
    Strikeout,
    Comment,
    Drawing,
}

/// Position in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate
    pub x: f64,
    
    /// Y coordinate
    pub y: f64,
    
    /// Width
    pub width: Option<f64>,
    
    /// Height
    pub height: Option<f64>,
}

/// Preview/thumbnail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preview {
    /// Preview type
    pub preview_type: PreviewType,
    
    /// Preview format
    pub format: String,
    
    /// Preview dimensions
    pub dimensions: (u32, u32),
    
    /// Preview data (base64 encoded)
    pub data: String,
    
    /// Page number (if applicable)
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewType {
    Thumbnail,
    FullPage,
    Crop,
}

/// Page content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    /// Page number
    pub page_number: u32,
    
    /// Page text
    pub text: Option<String>,
    
    /// Page image (base64 encoded)
    pub image: Option<String>,
    
    /// Page dimensions
    pub dimensions: Option<(f64, f64)>,
    
    /// Page metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Processing error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingError {
    /// Error type
    pub error_type: ErrorType,
    
    /// Error message
    pub message: String,
    
    /// Error context
    pub context: Option<String>,
    
    /// Timestamp
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    FileNotFound,
    PermissionDenied,
    UnsupportedFormat,
    CorruptedFile,
    PasswordProtected,
    ProcessingTimeout,
    OutOfMemory,
    NetworkError,
    ConfigurationError,
    UnknownError,
}

/// Processing statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessingStats {
    /// Processing duration
    pub duration: std::time::Duration,
    
    /// Memory usage
    pub memory_used: Option<u64>,
    
    /// Number of pages processed
    pub pages_processed: u32,
    
    /// Number of images extracted
    pub images_extracted: u32,
    
    /// Number of tables extracted
    pub tables_extracted: u32,
    
    /// Text length
    pub text_length: usize,
    
    /// OCR confidence (if applicable)
    pub ocr_confidence: Option<f32>,
}

/// Processor statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessorStats {
    /// Total files processed
    pub files_processed: u64,
    
    /// Files processed successfully
    pub files_successful: u64,
    
    /// Files with errors
    pub files_with_errors: u64,
    
    /// Total processing time
    pub total_processing_time: std::time::Duration,
    
    /// Average processing time
    pub avg_processing_time: std::time::Duration,
    
    /// Cache hits
    pub cache_hits: u64,
    
    /// Cache misses
    pub cache_misses: u64,
    
    /// Format statistics
    pub format_stats: HashMap<String, FormatStats>,
}

#[derive(Debug, Clone, Default)]
pub struct FormatStats {
    /// Files processed for this format
    pub files_processed: u64,
    
    /// Successful processing
    pub successful: u64,
    
    /// Errors
    pub errors: u64,
    
    /// Average processing time
    pub avg_processing_time: std::time::Duration,
    
    /// Total size processed
    pub total_size: u64,
}

/// Processor cache
#[derive(Debug, Clone)]
pub struct ProcessorCache {
    /// Cache entries
    entries: HashMap<String, CacheEntry>,
    
    /// Cache configuration
    config: CacheConfig,
    
    /// Current cache size
    current_size: u64,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Cached content
    content: ProcessedContent,
    
    /// Cache timestamp
    timestamp: std::time::SystemTime,
    
    /// Entry size
    size: u64,
}

impl MultiFormatProcessor {
    /// Create a new multi-format processor
    pub fn new(config: ProcessorConfig) -> Result<Self> {
        let mut processor = Self {
            config: config.clone(),
            handlers: HashMap::new(),
            stats: ProcessorStats::default(),
            cache: if config.enable_caching {
                Some(ProcessorCache::new(config.cache_config.clone())?)
            } else {
                None
            },
        };
        
        processor.register_default_handlers()?;
        
        Ok(processor)
    }
    
    /// Register default format handlers
    fn register_default_handlers(&mut self) -> Result<()> {
        // Register handlers for different formats
        // This would be implemented with actual format processing libraries
        
        if self.config.enable_pdf {
            // self.register_handler(Box::new(PdfHandler::new(self.config.pdf_config.clone())?));
        }
        
        if self.config.enable_images {
            // self.register_handler(Box::new(ImageHandler::new(self.config.image_config.clone())?));
        }
        
        if self.config.enable_documents {
            // self.register_handler(Box::new(DocumentHandler::new(self.config.document_config.clone())?));
        }
        
        if self.config.enable_text {
            // self.register_handler(Box::new(TextHandler::new(self.config.text_config.clone())?));
        }
        
        if self.config.enable_archives {
            // self.register_handler(Box::new(ArchiveHandler::new(self.config.archive_config.clone())?));
        }
        
        if self.config.enable_web {
            // self.register_handler(Box::new(WebHandler::new(self.config.web_config.clone())?));
        }
        
        Ok(())
    }
    
    /// Register a format handler
    pub fn register_handler(&mut self, handler: Box<dyn FormatHandler>) {
        let name = handler.name().to_string();
        self.handlers.insert(name, handler);
    }
    
    /// Process a file
    pub fn process_file(&mut self, path: &Path) -> Result<ProcessedContent> {
        let start_time = std::time::Instant::now();
        
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached_content) = cache.get(path, &self.config)? {
                self.stats.cache_hits += 1;
                return Ok(cached_content);
            }
        }
        self.stats.cache_misses += 1;
        
        // Detect MIME type
        let mime_type = self.detect_mime_type(path)?;
        
        // Find appropriate handler
        let handler = self.find_handler(path, &mime_type)?;
        
        // Process file
        let result = match handler.process(path, &self.config) {
            Ok(content) => {
                self.stats.files_successful += 1;
                content
            }
            Err(e) => {
                self.stats.files_with_errors += 1;
                
                if self.config.error_handling.fallback_to_text {
                    // Try basic text extraction as fallback
                    self.fallback_text_extraction(path)?
                } else {
                    return Err(e);
                }
            }
        };
        
        // Update statistics
        let processing_time = start_time.elapsed();
        self.update_stats(&mime_type, processing_time, true);
        
        // Cache result
        if let Some(cache) = &mut self.cache {
            cache.put(path, &result, &self.config)?;
        }
        
        self.stats.files_processed += 1;
        
        Ok(result)
    }
    
    /// Detect MIME type
    fn detect_mime_type(&self, path: &Path) -> Result<String> {
        // Try to detect from file extension first
        if let Some(mime) = MimeGuess::from_path(path).first() {
            return Ok(mime.to_string());
        }
        
        // Try to detect from file content
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut buffer = [0u8; 512];
            if let Ok(bytes_read) = file.read(&mut buffer) {
                // Use a more sophisticated MIME detection library here
                // For now, just basic detection
                if self.is_binary_content(&buffer[..bytes_read]) {
                    return Ok("application/octet-stream".to_string());
                } else {
                    return Ok("text/plain".to_string());
                }
            }
        }
        
        Ok("application/octet-stream".to_string())
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
    
    /// Find appropriate handler for file
    fn find_handler(&self, path: &Path, mime_type: &str) -> Result<&dyn FormatHandler> {
        for handler in self.handlers.values() {
            if handler.can_handle(path, mime_type) {
                return Ok(handler.as_ref());
            }
        }
        
        Err(anyhow::anyhow!("No handler found for file: {} (MIME: {})", path.display(), mime_type))
    }
    
    /// Fallback text extraction
    fn fallback_text_extraction(&self, path: &Path) -> Result<ProcessedContent> {
        let mut content = ProcessedContent {
            file_path: path.to_path_buf(),
            mime_type: "text/plain".to_string(),
            file_size: std::fs::metadata(path)?.len(),
            processed_at: std::time::SystemTime::now(),
            text_content: None,
            metadata: HashMap::new(),
            structure: None,
            previews: Vec::new(),
            pages: Vec::new(),
            errors: Vec::new(),
            stats: ProcessingStats::default(),
            original_content: None,
        };
        
        // Try to read as text
        match std::fs::read_to_string(path) {
            Ok(text) => {
                content.text_content = Some(text);
            }
            Err(_) => {
                // Try to read as binary and convert what we can
                if let Ok(bytes) = std::fs::read(path) {
                    let text = String::from_utf8_lossy(&bytes);
                    content.text_content = Some(text.to_string());
                }
            }
        }
        
        Ok(content)
    }
    
    /// Update processing statistics
    fn update_stats(&mut self, mime_type: &str, processing_time: std::time::Duration, success: bool) {
        self.stats.total_processing_time += processing_time;
        
        if self.stats.files_processed > 0 {
            self.stats.avg_processing_time = std::time::Duration::from_nanos(
                self.stats.total_processing_time.as_nanos() as u64 / self.stats.files_processed
            );
        }
        
        let format_stats = self.stats.format_stats.entry(mime_type.to_string()).or_default();
        format_stats.files_processed += 1;
        
        if success {
            format_stats.successful += 1;
        } else {
            format_stats.errors += 1;
        }
        
        format_stats.avg_processing_time = std::time::Duration::from_nanos(
            (format_stats.avg_processing_time.as_nanos() as u64 + processing_time.as_nanos() as u64) / 2
        );
    }
    
    /// Get processing statistics
    pub fn get_stats(&self) -> &ProcessorStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ProcessorStats::default();
    }
    
    /// Clear cache
    pub fn clear_cache(&mut self) -> Result<()> {
        if let Some(cache) = &mut self.cache {
            cache.clear()?;
        }
        Ok(())
    }
}

impl ProcessorCache {
    /// Create a new cache
    pub fn new(config: CacheConfig) -> Result<Self> {
        Ok(Self {
            entries: HashMap::new(),
            config,
            current_size: 0,
        })
    }
    
    /// Get cached content
    pub fn get(&self, path: &Path, config: &ProcessorConfig) -> Result<Option<ProcessedContent>> {
        let key = self.generate_key(path, config)?;
        
        if let Some(entry) = self.entries.get(&key) {
            // Check if entry is still valid
            if let Ok(elapsed) = std::time::SystemTime::now().duration_since(entry.timestamp) {
                if elapsed <= self.config.ttl {
                    return Ok(Some(entry.content.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Put content in cache
    pub fn put(&mut self, path: &Path, content: &ProcessedContent, config: &ProcessorConfig) -> Result<()> {
        let key = self.generate_key(path, config)?;
        let size = self.estimate_size(content);
        
        // Check if we need to evict entries
        while self.current_size + size > self.config.max_size && !self.entries.is_empty() {
            self.evict_oldest()?;
        }
        
        let entry = CacheEntry {
            content: content.clone(),
            timestamp: std::time::SystemTime::now(),
            size,
        };
        
        self.entries.insert(key, entry);
        self.current_size += size;
        
        Ok(())
    }
    
    /// Clear cache
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.current_size = 0;
        Ok(())
    }
    
    /// Generate cache key
    fn generate_key(&self, path: &Path, config: &ProcessorConfig) -> Result<String> {
        match self.config.key_strategy {
            CacheKeyStrategy::FilePath => {
                Ok(path.to_string_lossy().to_string())
            }
            CacheKeyStrategy::ContentHash => {
                // Calculate content hash
                let content = std::fs::read(path)?;
                Ok(format!("{:x}", md5::compute(&content)))
            }
            CacheKeyStrategy::PathAndMtime => {
                let metadata = std::fs::metadata(path)?;
                let mtime = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();
                Ok(format!("{}:{}", path.to_string_lossy(), mtime))
            }
            CacheKeyStrategy::ContentAndConfig => {
                let content = std::fs::read(path)?;
                let config_hash = format!("{:x}", md5::compute(serde_json::to_string(config)?.as_bytes()));
                Ok(format!("{:x}:{}", md5::compute(&content), config_hash))
            }
        }
    }
    
    /// Estimate content size
    fn estimate_size(&self, content: &ProcessedContent) -> u64 {
        // Rough estimation of memory usage
        let mut size = 0u64;
        
        if let Some(text) = &content.text_content {
            size += text.len() as u64;
        }
        
        size += content.metadata.len() as u64 * 100; // Rough estimate
        size += content.previews.len() as u64 * 10000; // Rough estimate for images
        size += content.pages.len() as u64 * 5000; // Rough estimate
        
        if let Some(original) = &content.original_content {
            size += original.len() as u64;
        }
        
        size
    }
    
    /// Evict oldest entry
    fn evict_oldest(&mut self) -> Result<()> {
        if let Some((key, _)) = self.entries.iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_size -= entry.size;
            }
        }
        
        Ok(())
    }
}

impl std::fmt::Debug for MultiFormatProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiFormatProcessor")
            .field("config", &self.config)
            .field("handlers", &self.handlers.keys())
            .field("stats", &self.stats)
            .field("cache", &self.cache)
            .finish()
    }
}
