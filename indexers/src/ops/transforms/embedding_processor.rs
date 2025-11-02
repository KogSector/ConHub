use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};

/// Advanced embedding processor with multiple model support
pub struct EmbeddingProcessor {
    /// Configuration for embedding processing
    config: EmbeddingConfig,
    
    /// Embedding models
    models: HashMap<String, Box<dyn EmbeddingModel>>,
    
    /// Processing statistics
    stats: EmbeddingStats,
    
    /// Cache for embeddings
    cache: Option<EmbeddingCache>,
    
    /// Batch processor
    batch_processor: Option<BatchProcessor>,
}

/// Configuration for embedding processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Enable text embeddings
    pub enable_text_embeddings: bool,
    
    /// Text embedding configuration
    pub text_config: TextEmbeddingConfig,
    
    /// Enable image embeddings
    pub enable_image_embeddings: bool,
    
    /// Image embedding configuration
    pub image_config: ImageEmbeddingConfig,
    
    /// Enable multimodal embeddings (ColPali)
    pub enable_multimodal_embeddings: bool,
    
    /// Multimodal embedding configuration
    pub multimodal_config: MultimodalEmbeddingConfig,
    
    /// Enable document embeddings
    pub enable_document_embeddings: bool,
    
    /// Document embedding configuration
    pub document_config: DocumentEmbeddingConfig,
    
    /// Enable code embeddings
    pub enable_code_embeddings: bool,
    
    /// Code embedding configuration
    pub code_config: CodeEmbeddingConfig,
    
    /// Enable caching
    pub enable_caching: bool,
    
    /// Cache configuration
    pub cache_config: CacheConfig,
    
    /// Enable batch processing
    pub enable_batch_processing: bool,
    
    /// Batch processing configuration
    pub batch_config: BatchConfig,
    
    /// Performance settings
    pub performance: PerformanceConfig,
    
    /// Quality settings
    pub quality: QualityConfig,
    
    /// Output configuration
    pub output: OutputConfig,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            enable_text_embeddings: true,
            text_config: TextEmbeddingConfig::default(),
            enable_image_embeddings: true,
            image_config: ImageEmbeddingConfig::default(),
            enable_multimodal_embeddings: true,
            multimodal_config: MultimodalEmbeddingConfig::default(),
            enable_document_embeddings: true,
            document_config: DocumentEmbeddingConfig::default(),
            enable_code_embeddings: true,
            code_config: CodeEmbeddingConfig::default(),
            enable_caching: true,
            cache_config: CacheConfig::default(),
            enable_batch_processing: true,
            batch_config: BatchConfig::default(),
            performance: PerformanceConfig::default(),
            quality: QualityConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

/// Text embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEmbeddingConfig {
    /// Primary text embedding model
    pub primary_model: TextEmbeddingModel,
    
    /// Fallback models
    pub fallback_models: Vec<TextEmbeddingModel>,
    
    /// Text preprocessing options
    pub preprocessing: TextPreprocessing,
    
    /// Chunking strategy
    pub chunking: ChunkingConfig,
    
    /// Language detection
    pub language_detection: LanguageDetectionConfig,
    
    /// Semantic analysis
    pub semantic_analysis: SemanticAnalysisConfig,
    
    /// Maximum text length
    pub max_text_length: usize,
    
    /// Minimum text length
    pub min_text_length: usize,
    
    /// Enable multilingual support
    pub multilingual: bool,
    
    /// Supported languages
    pub supported_languages: Vec<String>,
}

impl Default for TextEmbeddingConfig {
    fn default() -> Self {
        Self {
            primary_model: TextEmbeddingModel::SentenceTransformers {
                model_name: "all-MiniLM-L6-v2".to_string(),
                device: Device::Auto,
            },
            fallback_models: vec![
                TextEmbeddingModel::OpenAI {
                    model: "text-embedding-ada-002".to_string(),
                    api_key: None,
                },
            ],
            preprocessing: TextPreprocessing::default(),
            chunking: ChunkingConfig::default(),
            language_detection: LanguageDetectionConfig::default(),
            semantic_analysis: SemanticAnalysisConfig::default(),
            max_text_length: 8192,
            min_text_length: 10,
            multilingual: true,
            supported_languages: vec![
                "en".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "it".to_string(),
                "pt".to_string(),
                "ru".to_string(),
                "ja".to_string(),
                "ko".to_string(),
                "zh".to_string(),
            ],
        }
    }
}

/// Image embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEmbeddingConfig {
    /// Primary image embedding model
    pub primary_model: ImageEmbeddingModel,
    
    /// Fallback models
    pub fallback_models: Vec<ImageEmbeddingModel>,
    
    /// Image preprocessing options
    pub preprocessing: ImagePreprocessing,
    
    /// Feature extraction
    pub feature_extraction: FeatureExtractionConfig,
    
    /// Object detection
    pub object_detection: ObjectDetectionConfig,
    
    /// Scene analysis
    pub scene_analysis: SceneAnalysisConfig,
    
    /// Maximum image size
    pub max_image_size: (u32, u32),
    
    /// Minimum image size
    pub min_image_size: (u32, u32),
    
    /// Supported image formats
    pub supported_formats: Vec<String>,
    
    /// Enable OCR for text in images
    pub enable_ocr: bool,
    
    /// OCR configuration
    pub ocr_config: Option<OcrConfig>,
}

impl Default for ImageEmbeddingConfig {
    fn default() -> Self {
        Self {
            primary_model: ImageEmbeddingModel::CLIP {
                model_name: "openai/clip-vit-base-patch32".to_string(),
                device: Device::Auto,
            },
            fallback_models: vec![
                ImageEmbeddingModel::ResNet {
                    model_name: "resnet50".to_string(),
                    device: Device::Auto,
                },
            ],
            preprocessing: ImagePreprocessing::default(),
            feature_extraction: FeatureExtractionConfig::default(),
            object_detection: ObjectDetectionConfig::default(),
            scene_analysis: SceneAnalysisConfig::default(),
            max_image_size: (2048, 2048),
            min_image_size: (32, 32),
            supported_formats: vec![
                "jpeg".to_string(),
                "jpg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "bmp".to_string(),
                "tiff".to_string(),
                "webp".to_string(),
            ],
            enable_ocr: true,
            ocr_config: Some(OcrConfig::default()),
        }
    }
}

/// Multimodal embedding configuration (ColPali)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalEmbeddingConfig {
    /// Primary multimodal model
    pub primary_model: MultimodalEmbeddingModel,
    
    /// Fallback models
    pub fallback_models: Vec<MultimodalEmbeddingModel>,
    
    /// ColPali specific configuration
    pub colpali_config: ColPaliConfig,
    
    /// Vision-language alignment
    pub vision_language_alignment: VisionLanguageConfig,
    
    /// Document understanding
    pub document_understanding: DocumentUnderstandingConfig,
    
    /// Layout analysis
    pub layout_analysis: LayoutAnalysisConfig,
    
    /// Cross-modal retrieval
    pub cross_modal_retrieval: CrossModalRetrievalConfig,
    
    /// Enable page-level embeddings
    pub enable_page_embeddings: bool,
    
    /// Enable region-level embeddings
    pub enable_region_embeddings: bool,
    
    /// Enable element-level embeddings
    pub enable_element_embeddings: bool,
}

impl Default for MultimodalEmbeddingConfig {
    fn default() -> Self {
        Self {
            primary_model: MultimodalEmbeddingModel::ColPali {
                model_name: "vidore/colpali".to_string(),
                device: Device::Auto,
                precision: Precision::Float16,
            },
            fallback_models: vec![
                MultimodalEmbeddingModel::CLIP {
                    model_name: "openai/clip-vit-large-patch14".to_string(),
                    device: Device::Auto,
                },
            ],
            colpali_config: ColPaliConfig::default(),
            vision_language_alignment: VisionLanguageConfig::default(),
            document_understanding: DocumentUnderstandingConfig::default(),
            layout_analysis: LayoutAnalysisConfig::default(),
            cross_modal_retrieval: CrossModalRetrievalConfig::default(),
            enable_page_embeddings: true,
            enable_region_embeddings: true,
            enable_element_embeddings: false,
        }
    }
}

/// Document embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEmbeddingConfig {
    /// Document structure analysis
    pub structure_analysis: DocumentStructureConfig,
    
    /// Hierarchical embeddings
    pub hierarchical_embeddings: HierarchicalEmbeddingConfig,
    
    /// Citation analysis
    pub citation_analysis: CitationAnalysisConfig,
    
    /// Table understanding
    pub table_understanding: TableUnderstandingConfig,
    
    /// Figure understanding
    pub figure_understanding: FigureUnderstandingConfig,
    
    /// Cross-reference analysis
    pub cross_reference_analysis: CrossReferenceConfig,
    
    /// Enable section-level embeddings
    pub enable_section_embeddings: bool,
    
    /// Enable paragraph-level embeddings
    pub enable_paragraph_embeddings: bool,
    
    /// Enable sentence-level embeddings
    pub enable_sentence_embeddings: bool,
}

impl Default for DocumentEmbeddingConfig {
    fn default() -> Self {
        Self {
            structure_analysis: DocumentStructureConfig::default(),
            hierarchical_embeddings: HierarchicalEmbeddingConfig::default(),
            citation_analysis: CitationAnalysisConfig::default(),
            table_understanding: TableUnderstandingConfig::default(),
            figure_understanding: FigureUnderstandingConfig::default(),
            cross_reference_analysis: CrossReferenceConfig::default(),
            enable_section_embeddings: true,
            enable_paragraph_embeddings: true,
            enable_sentence_embeddings: false,
        }
    }
}

/// Code embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEmbeddingConfig {
    /// Primary code embedding model
    pub primary_model: CodeEmbeddingModel,
    
    /// Fallback models
    pub fallback_models: Vec<CodeEmbeddingModel>,
    
    /// Code analysis
    pub code_analysis: CodeAnalysisConfig,
    
    /// Syntax analysis
    pub syntax_analysis: SyntaxAnalysisConfig,
    
    /// Semantic analysis
    pub semantic_analysis: CodeSemanticConfig,
    
    /// Function-level embeddings
    pub function_embeddings: FunctionEmbeddingConfig,
    
    /// Class-level embeddings
    pub class_embeddings: ClassEmbeddingConfig,
    
    /// Supported programming languages
    pub supported_languages: Vec<String>,
    
    /// Enable documentation embeddings
    pub enable_documentation: bool,
    
    /// Enable comment embeddings
    pub enable_comments: bool,
}

impl Default for CodeEmbeddingConfig {
    fn default() -> Self {
        Self {
            primary_model: CodeEmbeddingModel::CodeBERT {
                model_name: "microsoft/codebert-base".to_string(),
                device: Device::Auto,
            },
            fallback_models: vec![
                CodeEmbeddingModel::GraphCodeBERT {
                    model_name: "microsoft/graphcodebert-base".to_string(),
                    device: Device::Auto,
                },
            ],
            code_analysis: CodeAnalysisConfig::default(),
            syntax_analysis: SyntaxAnalysisConfig::default(),
            semantic_analysis: CodeSemanticConfig::default(),
            function_embeddings: FunctionEmbeddingConfig::default(),
            class_embeddings: ClassEmbeddingConfig::default(),
            supported_languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "java".to_string(),
                "c++".to_string(),
                "c#".to_string(),
                "go".to_string(),
                "rust".to_string(),
                "php".to_string(),
                "ruby".to_string(),
                "swift".to_string(),
                "kotlin".to_string(),
            ],
            enable_documentation: true,
            enable_comments: true,
        }
    }
}

/// Text embedding models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextEmbeddingModel {
    SentenceTransformers {
        model_name: String,
        device: Device,
    },
    OpenAI {
        model: String,
        api_key: Option<String>,
    },
    Cohere {
        model: String,
        api_key: Option<String>,
    },
    HuggingFace {
        model_name: String,
        device: Device,
        revision: Option<String>,
    },
    Azure {
        endpoint: String,
        api_key: Option<String>,
        deployment_name: String,
    },
    Anthropic {
        model: String,
        api_key: Option<String>,
    },
    Local {
        model_path: String,
        device: Device,
    },
}

/// Image embedding models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageEmbeddingModel {
    CLIP {
        model_name: String,
        device: Device,
    },
    ResNet {
        model_name: String,
        device: Device,
    },
    EfficientNet {
        model_name: String,
        device: Device,
    },
    ViT {
        model_name: String,
        device: Device,
    },
    DINO {
        model_name: String,
        device: Device,
    },
    OpenAI {
        model: String,
        api_key: Option<String>,
    },
    Google {
        model: String,
        api_key: Option<String>,
    },
    Local {
        model_path: String,
        device: Device,
    },
}

/// Multimodal embedding models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultimodalEmbeddingModel {
    ColPali {
        model_name: String,
        device: Device,
        precision: Precision,
    },
    CLIP {
        model_name: String,
        device: Device,
    },
    BLIP {
        model_name: String,
        device: Device,
    },
    ALBEF {
        model_name: String,
        device: Device,
    },
    LayoutLM {
        model_name: String,
        device: Device,
    },
    DocFormer {
        model_name: String,
        device: Device,
    },
    OpenAI {
        model: String,
        api_key: Option<String>,
    },
    Local {
        model_path: String,
        device: Device,
    },
}

/// Code embedding models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeEmbeddingModel {
    CodeBERT {
        model_name: String,
        device: Device,
    },
    GraphCodeBERT {
        model_name: String,
        device: Device,
    },
    CodeT5 {
        model_name: String,
        device: Device,
    },
    UnixCoder {
        model_name: String,
        device: Device,
    },
    CodeGen {
        model_name: String,
        device: Device,
    },
    OpenAI {
        model: String,
        api_key: Option<String>,
    },
    Local {
        model_path: String,
        device: Device,
    },
}

/// Device configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Device {
    CPU,
    CUDA(u32),
    MPS,
    Auto,
}

/// Precision configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Precision {
    Float32,
    Float16,
    BFloat16,
    Int8,
    Auto,
}

/// ColPali specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColPaliConfig {
    /// Enable late interaction
    pub enable_late_interaction: bool,
    
    /// Number of patches
    pub num_patches: Option<u32>,
    
    /// Patch size
    pub patch_size: Option<(u32, u32)>,
    
    /// Enable attention visualization
    pub enable_attention_viz: bool,
    
    /// Query expansion
    pub query_expansion: QueryExpansionConfig,
    
    /// Retrieval configuration
    pub retrieval_config: RetrievalConfig,
    
    /// Enable multi-scale processing
    pub enable_multi_scale: bool,
    
    /// Scale factors
    pub scale_factors: Vec<f32>,
}

impl Default for ColPaliConfig {
    fn default() -> Self {
        Self {
            enable_late_interaction: true,
            num_patches: None,
            patch_size: None,
            enable_attention_viz: false,
            query_expansion: QueryExpansionConfig::default(),
            retrieval_config: RetrievalConfig::default(),
            enable_multi_scale: false,
            scale_factors: vec![0.5, 1.0, 1.5],
        }
    }
}

/// Text preprocessing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPreprocessing {
    /// Normalize whitespace
    pub normalize_whitespace: bool,
    
    /// Remove special characters
    pub remove_special_chars: bool,
    
    /// Convert to lowercase
    pub lowercase: bool,
    
    /// Remove stop words
    pub remove_stop_words: bool,
    
    /// Stop words languages
    pub stop_words_languages: Vec<String>,
    
    /// Enable stemming
    pub enable_stemming: bool,
    
    /// Enable lemmatization
    pub enable_lemmatization: bool,
    
    /// Remove URLs
    pub remove_urls: bool,
    
    /// Remove emails
    pub remove_emails: bool,
    
    /// Remove phone numbers
    pub remove_phone_numbers: bool,
    
    /// Custom preprocessing rules
    pub custom_rules: Vec<PreprocessingRule>,
}

impl Default for TextPreprocessing {
    fn default() -> Self {
        Self {
            normalize_whitespace: true,
            remove_special_chars: false,
            lowercase: false,
            remove_stop_words: false,
            stop_words_languages: vec!["en".to_string()],
            enable_stemming: false,
            enable_lemmatization: false,
            remove_urls: false,
            remove_emails: false,
            remove_phone_numbers: false,
            custom_rules: Vec::new(),
        }
    }
}

/// Preprocessing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingRule {
    /// Rule name
    pub name: String,
    
    /// Rule type
    pub rule_type: PreprocessingRuleType,
    
    /// Rule parameters
    pub parameters: HashMap<String, String>,
    
    /// Rule enabled
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreprocessingRuleType {
    Regex,
    Replace,
    Remove,
    Transform,
    Custom,
}

/// Chunking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Chunking strategy
    pub strategy: ChunkingStrategy,
    
    /// Chunk size
    pub chunk_size: usize,
    
    /// Chunk overlap
    pub chunk_overlap: usize,
    
    /// Minimum chunk size
    pub min_chunk_size: usize,
    
    /// Maximum chunk size
    pub max_chunk_size: usize,
    
    /// Respect sentence boundaries
    pub respect_sentences: bool,
    
    /// Respect paragraph boundaries
    pub respect_paragraphs: bool,
    
    /// Respect section boundaries
    pub respect_sections: bool,
    
    /// Custom separators
    pub custom_separators: Vec<String>,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            strategy: ChunkingStrategy::Sliding,
            chunk_size: 512,
            chunk_overlap: 50,
            min_chunk_size: 100,
            max_chunk_size: 1024,
            respect_sentences: true,
            respect_paragraphs: true,
            respect_sections: false,
            custom_separators: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkingStrategy {
    Fixed,
    Sliding,
    Semantic,
    Hierarchical,
    Adaptive,
    Custom,
}

/// Language detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDetectionConfig {
    /// Enable language detection
    pub enabled: bool,
    
    /// Detection method
    pub method: LanguageDetectionMethod,
    
    /// Confidence threshold
    pub confidence_threshold: f32,
    
    /// Fallback language
    pub fallback_language: String,
    
    /// Supported languages
    pub supported_languages: Vec<String>,
}

impl Default for LanguageDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: LanguageDetectionMethod::FastText,
            confidence_threshold: 0.8,
            fallback_language: "en".to_string(),
            supported_languages: vec![
                "en".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "it".to_string(),
                "pt".to_string(),
                "ru".to_string(),
                "ja".to_string(),
                "ko".to_string(),
                "zh".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LanguageDetectionMethod {
    FastText,
    Langdetect,
    Polyglot,
    SpaCy,
    Custom,
}

/// Semantic analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysisConfig {
    /// Enable named entity recognition
    pub enable_ner: bool,
    
    /// Enable sentiment analysis
    pub enable_sentiment: bool,
    
    /// Enable topic modeling
    pub enable_topic_modeling: bool,
    
    /// Enable keyword extraction
    pub enable_keyword_extraction: bool,
    
    /// Enable summarization
    pub enable_summarization: bool,
    
    /// NER configuration
    pub ner_config: NerConfig,
    
    /// Sentiment configuration
    pub sentiment_config: SentimentConfig,
    
    /// Topic modeling configuration
    pub topic_config: TopicConfig,
    
    /// Keyword extraction configuration
    pub keyword_config: KeywordConfig,
    
    /// Summarization configuration
    pub summary_config: SummaryConfig,
}

impl Default for SemanticAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_ner: false,
            enable_sentiment: false,
            enable_topic_modeling: false,
            enable_keyword_extraction: false,
            enable_summarization: false,
            ner_config: NerConfig::default(),
            sentiment_config: SentimentConfig::default(),
            topic_config: TopicConfig::default(),
            keyword_config: KeywordConfig::default(),
            summary_config: SummaryConfig::default(),
        }
    }
}

// Additional configuration structs would be defined here...
// For brevity, I'll define the main ones and indicate where others would go

/// Embedding model trait
pub trait EmbeddingModel: Send + Sync {
    /// Get model name
    fn name(&self) -> &str;
    
    /// Check if model can handle the given input type
    fn can_handle(&self, input_type: &InputType) -> bool;
    
    /// Generate embeddings for text
    fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
    
    /// Generate embeddings for image
    fn embed_image(&self, image_data: &[u8]) -> Result<Vec<f32>>;
    
    /// Generate embeddings for multimodal input
    fn embed_multimodal(&self, text: &str, image_data: &[u8]) -> Result<Vec<f32>>;
    
    /// Get embedding dimension
    fn embedding_dimension(&self) -> usize;
    
    /// Get model configuration
    fn config(&self) -> &dyn std::any::Any;
}

/// Input type for embeddings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputType {
    Text,
    Image,
    Multimodal,
    Document,
    Code,
}

/// Embedding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    /// Input content
    pub input: EmbeddingInput,
    
    /// Generated embeddings
    pub embeddings: Vec<EmbeddingVector>,
    
    /// Model information
    pub model_info: ModelInfo,
    
    /// Processing metadata
    pub metadata: EmbeddingMetadata,
    
    /// Processing statistics
    pub stats: EmbeddingProcessingStats,
    
    /// Errors (if any)
    pub errors: Vec<EmbeddingError>,
}

/// Embedding input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingInput {
    /// Input type
    pub input_type: InputType,
    
    /// Text content
    pub text: Option<String>,
    
    /// Image data (base64 encoded)
    pub image: Option<String>,
    
    /// Document structure
    pub document: Option<DocumentStructure>,
    
    /// Code structure
    pub code: Option<CodeStructure>,
    
    /// Input metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Embedding vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    /// Vector values
    pub values: Vec<f32>,
    
    /// Vector type
    pub vector_type: VectorType,
    
    /// Vector metadata
    pub metadata: VectorMetadata,
    
    /// Chunk information (if applicable)
    pub chunk_info: Option<ChunkInfo>,
    
    /// Attention weights (if available)
    pub attention_weights: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorType {
    Dense,
    Sparse,
    Binary,
    Quantized,
}

/// Vector metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    /// Vector dimension
    pub dimension: usize,
    
    /// Vector norm
    pub norm: Option<f32>,
    
    /// Vector quality score
    pub quality_score: Option<f32>,
    
    /// Processing timestamp
    pub timestamp: std::time::SystemTime,
    
    /// Model version
    pub model_version: Option<String>,
    
    /// Additional metadata
    pub additional: HashMap<String, serde_json::Value>,
}

/// Chunk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Chunk index
    pub index: usize,
    
    /// Chunk start position
    pub start: usize,
    
    /// Chunk end position
    pub end: usize,
    
    /// Chunk text
    pub text: String,
    
    /// Chunk overlap with previous
    pub overlap_prev: usize,
    
    /// Chunk overlap with next
    pub overlap_next: usize,
    
    /// Chunk metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name
    pub name: String,
    
    /// Model type
    pub model_type: String,
    
    /// Model version
    pub version: Option<String>,
    
    /// Model provider
    pub provider: String,
    
    /// Model configuration
    pub config: HashMap<String, serde_json::Value>,
    
    /// Model capabilities
    pub capabilities: Vec<String>,
}

/// Embedding metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    /// Processing timestamp
    pub timestamp: std::time::SystemTime,
    
    /// Processing duration
    pub duration: std::time::Duration,
    
    /// Input language (if detected)
    pub language: Option<String>,
    
    /// Input quality score
    pub quality_score: Option<f32>,
    
    /// Preprocessing applied
    pub preprocessing: Vec<String>,
    
    /// Chunking information
    pub chunking_info: Option<ChunkingInfo>,
    
    /// Additional metadata
    pub additional: HashMap<String, serde_json::Value>,
}

/// Chunking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingInfo {
    /// Chunking strategy used
    pub strategy: String,
    
    /// Number of chunks
    pub num_chunks: usize,
    
    /// Average chunk size
    pub avg_chunk_size: usize,
    
    /// Chunk overlap
    pub overlap: usize,
    
    /// Chunking metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Embedding processing statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbeddingProcessingStats {
    /// Processing duration
    pub duration: std::time::Duration,
    
    /// Memory usage
    pub memory_used: Option<u64>,
    
    /// GPU memory usage
    pub gpu_memory_used: Option<u64>,
    
    /// Number of tokens processed
    pub tokens_processed: Option<usize>,
    
    /// Number of patches processed (for images)
    pub patches_processed: Option<usize>,
    
    /// Model inference time
    pub inference_time: std::time::Duration,
    
    /// Preprocessing time
    pub preprocessing_time: std::time::Duration,
    
    /// Postprocessing time
    pub postprocessing_time: std::time::Duration,
}

/// Embedding error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingError {
    /// Error type
    pub error_type: EmbeddingErrorType,
    
    /// Error message
    pub message: String,
    
    /// Error context
    pub context: Option<String>,
    
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    
    /// Recoverable flag
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingErrorType {
    ModelLoadError,
    InputValidationError,
    ProcessingError,
    MemoryError,
    TimeoutError,
    NetworkError,
    ConfigurationError,
    UnknownError,
}

/// Embedding statistics
#[derive(Debug, Clone, Default)]
pub struct EmbeddingStats {
    /// Total embeddings generated
    pub embeddings_generated: u64,
    
    /// Successful embeddings
    pub successful_embeddings: u64,
    
    /// Failed embeddings
    pub failed_embeddings: u64,
    
    /// Total processing time
    pub total_processing_time: std::time::Duration,
    
    /// Average processing time
    pub avg_processing_time: std::time::Duration,
    
    /// Cache hits
    pub cache_hits: u64,
    
    /// Cache misses
    pub cache_misses: u64,
    
    /// Model statistics
    pub model_stats: HashMap<String, ModelStats>,
    
    /// Input type statistics
    pub input_type_stats: HashMap<InputType, InputTypeStats>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelStats {
    /// Embeddings generated
    pub embeddings_generated: u64,
    
    /// Successful embeddings
    pub successful: u64,
    
    /// Failed embeddings
    pub failed: u64,
    
    /// Average processing time
    pub avg_processing_time: std::time::Duration,
    
    /// Total processing time
    pub total_processing_time: std::time::Duration,
    
    /// Memory usage
    pub memory_usage: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct InputTypeStats {
    /// Embeddings generated
    pub embeddings_generated: u64,
    
    /// Successful embeddings
    pub successful: u64,
    
    /// Failed embeddings
    pub failed: u64,
    
    /// Average processing time
    pub avg_processing_time: std::time::Duration,
    
    /// Average input size
    pub avg_input_size: usize,
}

/// Embedding cache
#[derive(Debug, Clone)]
pub struct EmbeddingCache {
    /// Cache entries
    entries: HashMap<String, CacheEntry>,
    
    /// Cache configuration
    config: CacheConfig,
    
    /// Current cache size
    current_size: u64,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Cached embedding result
    result: EmbeddingResult,
    
    /// Cache timestamp
    timestamp: std::time::SystemTime,
    
    /// Entry size
    size: u64,
    
    /// Access count
    access_count: u64,
    
    /// Last access time
    last_access: std::time::SystemTime,
}

/// Batch processor
#[derive(Debug, Clone)]
pub struct BatchProcessor {
    /// Batch configuration
    config: BatchConfig,
    
    /// Current batch
    current_batch: Vec<EmbeddingInput>,
    
    /// Batch statistics
    stats: BatchStats,
}

#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    /// Batches processed
    pub batches_processed: u64,
    
    /// Items processed
    pub items_processed: u64,
    
    /// Average batch size
    pub avg_batch_size: f64,
    
    /// Average batch processing time
    pub avg_batch_time: std::time::Duration,
    
    /// Total batch processing time
    pub total_batch_time: std::time::Duration,
}

// Placeholder structs for configuration types
// In a real implementation, these would be fully defined

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImagePreprocessing {
    pub resize: bool,
    pub normalize: bool,
    pub augment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureExtractionConfig {
    pub extract_global_features: bool,
    pub extract_local_features: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectDetectionConfig {
    pub enabled: bool,
    pub confidence_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SceneAnalysisConfig {
    pub enabled: bool,
    pub scene_categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OcrConfig {
    pub engine: String,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisionLanguageConfig {
    pub alignment_method: String,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentUnderstandingConfig {
    pub extract_structure: bool,
    pub extract_tables: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutAnalysisConfig {
    pub detect_regions: bool,
    pub classify_elements: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrossModalRetrievalConfig {
    pub enable_text_to_image: bool,
    pub enable_image_to_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentStructureConfig {
    pub extract_headings: bool,
    pub extract_paragraphs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HierarchicalEmbeddingConfig {
    pub levels: Vec<String>,
    pub aggregation_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CitationAnalysisConfig {
    pub extract_citations: bool,
    pub analyze_references: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableUnderstandingConfig {
    pub extract_structure: bool,
    pub extract_content: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FigureUnderstandingConfig {
    pub extract_captions: bool,
    pub analyze_content: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrossReferenceConfig {
    pub extract_references: bool,
    pub analyze_links: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeAnalysisConfig {
    pub extract_functions: bool,
    pub extract_classes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyntaxAnalysisConfig {
    pub parse_ast: bool,
    pub extract_tokens: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeSemanticConfig {
    pub analyze_dependencies: bool,
    pub extract_patterns: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionEmbeddingConfig {
    pub include_signature: bool,
    pub include_body: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClassEmbeddingConfig {
    pub include_methods: bool,
    pub include_attributes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    pub max_size: u64,
    pub ttl: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub timeout: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceConfig {
    pub max_workers: usize,
    pub timeout: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityConfig {
    pub min_quality_score: f32,
    pub enable_validation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    pub include_metadata: bool,
    pub include_stats: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryExpansionConfig {
    pub enabled: bool,
    pub expansion_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetrievalConfig {
    pub top_k: usize,
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NerConfig {
    pub model: String,
    pub entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SentimentConfig {
    pub model: String,
    pub granularity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TopicConfig {
    pub num_topics: usize,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeywordConfig {
    pub max_keywords: usize,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SummaryConfig {
    pub max_length: usize,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentStructure {
    pub sections: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeStructure {
    pub language: String,
    pub functions: Vec<String>,
    pub classes: Vec<String>,
}

impl EmbeddingProcessor {
    /// Create a new embedding processor
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        let mut processor = Self {
            config: config.clone(),
            models: HashMap::new(),
            stats: EmbeddingStats::default(),
            cache: if config.enable_caching {
                Some(EmbeddingCache::new(config.cache_config.clone())?)
            } else {
                None
            },
            batch_processor: if config.enable_batch_processing {
                Some(BatchProcessor::new(config.batch_config.clone())?)
            } else {
                None
            },
        };
        
        processor.initialize_models()?;
        
        Ok(processor)
    }
    
    /// Initialize embedding models
    fn initialize_models(&mut self) -> Result<()> {
        // Initialize models based on configuration
        // This would be implemented with actual model loading
        
        if self.config.enable_text_embeddings {
            // Load text embedding models
        }
        
        if self.config.enable_image_embeddings {
            // Load image embedding models
        }
        
        if self.config.enable_multimodal_embeddings {
            // Load multimodal embedding models (ColPali, etc.)
        }
        
        if self.config.enable_document_embeddings {
            // Load document embedding models
        }
        
        if self.config.enable_code_embeddings {
            // Load code embedding models
        }
        
        Ok(())
    }
    
    /// Process input and generate embeddings
    pub fn process(&mut self, input: EmbeddingInput) -> Result<EmbeddingResult> {
        let start_time = std::time::Instant::now();
        
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached_result) = cache.get(&input)? {
                self.stats.cache_hits += 1;
                return Ok(cached_result);
            }
        }
        self.stats.cache_misses += 1;
        
        // Determine appropriate model
        let model = self.select_model(&input)?;
        
        // Generate embeddings
        let result = self.generate_embeddings(&input, model)?;
        
        // Update statistics
        let processing_time = start_time.elapsed();
        self.update_stats(&input, &result, processing_time);
        
        // Cache result
        if let Some(cache) = &mut self.cache {
            cache.put(&input, &result)?;
        }
        
        Ok(result)
    }
    
    /// Select appropriate model for input
    fn select_model(&self, input: &EmbeddingInput) -> Result<&dyn EmbeddingModel> {
        // Model selection logic based on input type and configuration
        for model in self.models.values() {
            if model.can_handle(&input.input_type) {
                return Ok(model.as_ref());
            }
        }
        
        Err(anyhow::anyhow!("No suitable model found for input type: {:?}", input.input_type))
    }
    
    /// Generate embeddings using selected model
    fn generate_embeddings(&self, input: &EmbeddingInput, model: &dyn EmbeddingModel) -> Result<EmbeddingResult> {
        let start_time = std::time::Instant::now();
        
        let embeddings = match &input.input_type {
            InputType::Text => {
                if let Some(text) = &input.text {
                    vec![EmbeddingVector {
                        values: model.embed_text(text)?,
                        vector_type: VectorType::Dense,
                        metadata: VectorMetadata {
                            dimension: model.embedding_dimension(),
                            norm: None,
                            quality_score: None,
                            timestamp: std::time::SystemTime::now(),
                            model_version: None,
                            additional: HashMap::new(),
                        },
                        chunk_info: None,
                        attention_weights: None,
                    }]
                } else {
                    return Err(anyhow::anyhow!("Text input required for text embedding"));
                }
            }
            InputType::Image => {
                if let Some(image_b64) = &input.image {
                    let image_data = general_purpose::STANDARD.decode(image_b64)?;
                    vec![EmbeddingVector {
                        values: model.embed_image(&image_data)?,
                        vector_type: VectorType::Dense,
                        metadata: VectorMetadata {
                            dimension: model.embedding_dimension(),
                            norm: None,
                            quality_score: None,
                            timestamp: std::time::SystemTime::now(),
                            model_version: None,
                            additional: HashMap::new(),
                        },
                        chunk_info: None,
                        attention_weights: None,
                    }]
                } else {
                    return Err(anyhow::anyhow!("Image input required for image embedding"));
                }
            }
            InputType::Multimodal => {
                if let (Some(text), Some(image_b64)) = (&input.text, &input.image) {
                    let image_data = general_purpose::STANDARD.decode(image_b64)?;
                    vec![EmbeddingVector {
                        values: model.embed_multimodal(text, &image_data)?,
                        vector_type: VectorType::Dense,
                        metadata: VectorMetadata {
                            dimension: model.embedding_dimension(),
                            norm: None,
                            quality_score: None,
                            timestamp: std::time::SystemTime::now(),
                            model_version: None,
                            additional: HashMap::new(),
                        },
                        chunk_info: None,
                        attention_weights: None,
                    }]
                } else {
                    return Err(anyhow::anyhow!("Both text and image inputs required for multimodal embedding"));
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported input type: {:?}", input.input_type));
            }
        };
        
        let processing_time = start_time.elapsed();
        
        Ok(EmbeddingResult {
            input: input.clone(),
            embeddings,
            model_info: ModelInfo {
                name: model.name().to_string(),
                model_type: "embedding".to_string(),
                version: None,
                provider: "local".to_string(),
                config: HashMap::new(),
                capabilities: vec!["embedding".to_string()],
            },
            metadata: EmbeddingMetadata {
                timestamp: std::time::SystemTime::now(),
                duration: processing_time,
                language: None,
                quality_score: None,
                preprocessing: Vec::new(),
                chunking_info: None,
                additional: HashMap::new(),
            },
            stats: EmbeddingProcessingStats {
                duration: processing_time,
                memory_used: None,
                gpu_memory_used: None,
                tokens_processed: None,
                patches_processed: None,
                inference_time: processing_time,
                preprocessing_time: std::time::Duration::default(),
                postprocessing_time: std::time::Duration::default(),
            },
            errors: Vec::new(),
        })
    }
    
    /// Update processing statistics
    fn update_stats(&mut self, input: &EmbeddingInput, result: &EmbeddingResult, processing_time: std::time::Duration) {
        self.stats.embeddings_generated += 1;
        
        if result.errors.is_empty() {
            self.stats.successful_embeddings += 1;
        } else {
            self.stats.failed_embeddings += 1;
        }
        
        self.stats.total_processing_time += processing_time;
        
        if self.stats.embeddings_generated > 0 {
            self.stats.avg_processing_time = std::time::Duration::from_nanos(
                self.stats.total_processing_time.as_nanos() as u64 / self.stats.embeddings_generated
            );
        }
        
        // Update model statistics
        let model_stats = self.stats.model_stats.entry(result.model_info.name.clone()).or_default();
        model_stats.embeddings_generated += 1;
        
        if result.errors.is_empty() {
            model_stats.successful += 1;
        } else {
            model_stats.failed += 1;
        }
        
        model_stats.total_processing_time += processing_time;
        model_stats.avg_processing_time = std::time::Duration::from_nanos(
            model_stats.total_processing_time.as_nanos() as u64 / model_stats.embeddings_generated
        );
        
        // Update input type statistics
        let input_stats = self.stats.input_type_stats.entry(input.input_type.clone()).or_default();
        input_stats.embeddings_generated += 1;
        
        if result.errors.is_empty() {
            input_stats.successful += 1;
        } else {
            input_stats.failed += 1;
        }
        
        input_stats.avg_processing_time = std::time::Duration::from_nanos(
            (input_stats.avg_processing_time.as_nanos() as u64 + processing_time.as_nanos() as u64) / 2
        );
    }
    
    /// Get processing statistics
    pub fn get_stats(&self) -> &EmbeddingStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = EmbeddingStats::default();
    }
    
    /// Clear cache
    pub fn clear_cache(&mut self) -> Result<()> {
        if let Some(cache) = &mut self.cache {
            cache.clear()?;
        }
        Ok(())
    }
}

impl EmbeddingCache {
    /// Create a new cache
    pub fn new(config: CacheConfig) -> Result<Self> {
        Ok(Self {
            entries: HashMap::new(),
            config,
            current_size: 0,
        })
    }
    
    /// Get cached result
    pub fn get(&self, input: &EmbeddingInput) -> Result<Option<EmbeddingResult>> {
        let key = self.generate_key(input)?;
        
        if let Some(entry) = self.entries.get(&key) {
            // Check if entry is still valid
            if let Ok(elapsed) = std::time::SystemTime::now().duration_since(entry.timestamp) {
                if elapsed <= self.config.ttl {
                    return Ok(Some(entry.result.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Put result in cache
    pub fn put(&mut self, input: &EmbeddingInput, result: &EmbeddingResult) -> Result<()> {
        let key = self.generate_key(input)?;
        let size = self.estimate_size(result);
        
        // Check if we need to evict entries
        while self.current_size + size > self.config.max_size && !self.entries.is_empty() {
            self.evict_lru()?;
        }
        
        let entry = CacheEntry {
            result: result.clone(),
            timestamp: std::time::SystemTime::now(),
            size,
            access_count: 1,
            last_access: std::time::SystemTime::now(),
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
    fn generate_key(&self, input: &EmbeddingInput) -> Result<String> {
        // Generate a hash of the input for caching
        let serialized = serde_json::to_string(input)?;
        Ok(format!("{:x}", md5::compute(serialized.as_bytes())))
    }
    
    /// Estimate result size
    fn estimate_size(&self, result: &EmbeddingResult) -> u64 {
        let mut size = 0u64;
        
        for embedding in &result.embeddings {
            size += embedding.values.len() as u64 * 4; // 4 bytes per f32
        }
        
        size += 1000; // Rough estimate for metadata
        
        size
    }
    
    /// Evict least recently used entry
    fn evict_lru(&mut self) -> Result<()> {
        if let Some((key, _)) = self.entries.iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_size -= entry.size;
            }
        }
        
        Ok(())
    }
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(config: BatchConfig) -> Result<Self> {
        Ok(Self {
            config,
            current_batch: Vec::new(),
            stats: BatchStats::default(),
        })
    }
    
    /// Add input to batch
    pub fn add_to_batch(&mut self, input: EmbeddingInput) -> Result<Option<Vec<EmbeddingInput>>> {
        self.current_batch.push(input);
        
        if self.current_batch.len() >= self.config.batch_size {
            let batch = std::mem::take(&mut self.current_batch);
            Ok(Some(batch))
        } else {
            Ok(None)
        }
    }
    
    /// Flush current batch
    pub fn flush_batch(&mut self) -> Result<Vec<EmbeddingInput>> {
        Ok(std::mem::take(&mut self.current_batch))
    }
    
    /// Get batch statistics
    pub fn get_stats(&self) -> &BatchStats {
        &self.stats
    }
}

impl std::fmt::Debug for EmbeddingProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingProcessor")
            .field("config", &self.config)
            .field("models", &self.models.keys())
            .field("stats", &self.stats)
            .field("cache", &self.cache)
            .field("batch_processor", &self.batch_processor)
            .finish()
    }
}
