use crate::config::IndexerConfig;
use std::collections::{HashMap, VecDeque};
use regex::Regex;

#[derive(Debug, Clone)]
pub enum ChunkingStrategy {
    Fixed,
    Semantic,
    Hierarchical,
    Adaptive,
    SyntaxAware,
}

#[derive(Debug, Clone)]
pub struct ChunkMetadata {
    pub start_offset: usize,
    pub end_offset: usize,
    pub language: Option<String>,
    pub section_type: Option<String>,
    pub importance_score: f32,
    pub semantic_density: f32,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub content: String,
    pub metadata: ChunkMetadata,
}

pub struct ChunkingService {
    chunk_size: usize,
    chunk_overlap: usize,
    strategy: ChunkingStrategy,
    // Cached regex patterns for performance
    sentence_boundary: Regex,
    paragraph_boundary: Regex,
    code_block_boundary: Regex,
    heading_pattern: Regex,
}

impl ChunkingService {
    pub fn new(config: IndexerConfig) -> Self {
        Self {
            chunk_size: config.chunk_size,
            chunk_overlap: config.chunk_overlap,
            strategy: ChunkingStrategy::Adaptive,
            sentence_boundary: Regex::new(r"[.!?]+\s+").unwrap(),
            paragraph_boundary: Regex::new(r"\n\s*\n").unwrap(),
            code_block_boundary: Regex::new(r"```[\s\S]*?```|`[^`]+`").unwrap(),
            heading_pattern: Regex::new(r"^#{1,6}\s+").unwrap(),
        }
    }

    pub fn with_strategy(mut self, strategy: ChunkingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Advanced chunking with metadata
    pub fn chunk_with_metadata(&self, text: &str, language: Option<String>) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        match self.strategy {
            ChunkingStrategy::Fixed => self.fixed_chunking(text, language),
            ChunkingStrategy::Semantic => self.semantic_chunking(text, language),
            ChunkingStrategy::Hierarchical => self.hierarchical_chunking(text, language),
            ChunkingStrategy::Adaptive => self.adaptive_chunking(text, language),
            ChunkingStrategy::SyntaxAware => self.syntax_aware_chunking(text, language),
        }
    }

    /// Fixed-size chunking with overlap
    fn fixed_chunking(&self, text: &str, language: Option<String>) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        let mut chunks: Vec<Chunk> = Vec::new();
        let text_len = text.len();

        if text_len == 0 {
            return Ok(chunks);
        }

        if text_len <= self.chunk_size {
            chunks.push(Chunk {
                content: text.to_string(),
                metadata: ChunkMetadata {
                    start_offset: 0,
                    end_offset: text_len,
                    language: language.clone(),
                    section_type: Some("complete".to_string()),
                    importance_score: 1.0,
                    semantic_density: self.calculate_semantic_density(text),
                },
            });
            return Ok(chunks);
        }

        let mut start = 0;
        while start < text_len {
            let end = std::cmp::min(start + self.chunk_size, text_len);
            let chunk_text = &text[start..end];

            // Try to break at word boundaries
            let chunk_content = if end < text_len {
                if let Some(last_space) = chunk_text.rfind(|c: char| c.is_whitespace()) {
                    &chunk_text[..last_space]
                } else {
                    chunk_text
                }
            } else {
                chunk_text
            };

            chunks.push(Chunk {
                content: chunk_content.trim().to_string(),
                metadata: ChunkMetadata {
                    start_offset: start,
                    end_offset: start + chunk_content.len(),
                    language: language.clone(),
                    section_type: Some("fixed".to_string()),
                    importance_score: self.calculate_importance_score(chunk_content),
                    semantic_density: self.calculate_semantic_density(chunk_content),
                },
            });

            if end >= text_len {
                break;
            }

            start = if chunk_content.len() > self.chunk_overlap {
                start + chunk_content.len() - self.chunk_overlap
            } else {
                start + chunk_content.len()
            };
        }

        Ok(chunks)
    }

    /// Semantic chunking based on sentence boundaries and content coherence
    fn semantic_chunking(&self, text: &str, language: Option<String>) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();
        let sentences = self.split_into_sentences(text);
        
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut sentence_start = 0;

        for sentence in sentences {
            let sentence_trimmed = sentence.trim();
            if sentence_trimmed.is_empty() {
                sentence_start += sentence.len();
                continue;
            }

            // Check if adding this sentence would exceed chunk size
            if !current_chunk.is_empty() && 
               current_chunk.len() + sentence.len() > self.chunk_size {
                
                // Calculate semantic coherence before splitting
                let coherence_score = self.calculate_semantic_coherence(&current_chunk, sentence);
                
                if coherence_score < 0.3 || current_chunk.len() > self.chunk_size / 2 {
                    // Low coherence or chunk is large enough, create new chunk
                    chunks.push(Chunk {
                        content: current_chunk.trim().to_string(),
                        metadata: ChunkMetadata {
                            start_offset: current_start,
                            end_offset: sentence_start,
                            language: language.clone(),
                            section_type: Some("semantic".to_string()),
                            importance_score: self.calculate_importance_score(&current_chunk),
                            semantic_density: self.calculate_semantic_density(&current_chunk),
                        },
                    });
                    
                    current_chunk = String::new();
                    current_start = sentence_start;
                }
            }

            if current_chunk.is_empty() {
                current_start = sentence_start;
            }
            
            current_chunk.push_str(sentence);
            sentence_start += sentence.len();
        }

        // Add the last chunk if not empty
        if !current_chunk.trim().is_empty() {
            chunks.push(Chunk {
                content: current_chunk.trim().to_string(),
                metadata: ChunkMetadata {
                    start_offset: current_start,
                    end_offset: text.len(),
                    language: language.clone(),
                    section_type: Some("semantic".to_string()),
                    importance_score: self.calculate_importance_score(&current_chunk),
                    semantic_density: self.calculate_semantic_density(&current_chunk),
                },
            });
        }

        Ok(chunks)
    }

    /// Hierarchical chunking that respects document structure
    fn hierarchical_chunking(&self, text: &str, language: Option<String>) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();
        let sections = self.split_into_sections(text);
        
        for (section_start, section_content, section_type) in sections {
            if section_content.len() <= self.chunk_size {
                // Section fits in one chunk
                chunks.push(Chunk {
                    content: section_content.clone(),
                    metadata: ChunkMetadata {
                        start_offset: section_start,
                        end_offset: section_start + section_content.len(),
                        language: language.clone(),
                        section_type: Some(section_type),
                        importance_score: self.calculate_importance_score(&section_content),
                        semantic_density: self.calculate_semantic_density(&section_content),
                    },
                });
            } else {
                // Split large sections using semantic chunking
                let section_chunks = self.semantic_chunking(&section_content, language.clone())?;
                for mut chunk in section_chunks {
                    chunk.metadata.start_offset += section_start;
                    chunk.metadata.end_offset += section_start;
                    chunk.metadata.section_type = Some(section_type.clone());
                    chunks.push(chunk);
                }
            }
        }

        Ok(chunks)
    }

    /// Adaptive chunking that adjusts strategy based on content type
    fn adaptive_chunking(&self, text: &str, language: Option<String>) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        // Analyze content characteristics
        let has_code = self.code_block_boundary.is_match(text);
        let has_headings = self.heading_pattern.is_match(text);
        let avg_sentence_length = self.calculate_average_sentence_length(text);
        let semantic_density = self.calculate_semantic_density(text);

        // Choose strategy based on content analysis
        let strategy = if has_code {
            ChunkingStrategy::SyntaxAware
        } else if has_headings {
            ChunkingStrategy::Hierarchical
        } else if semantic_density > 0.7 || avg_sentence_length > 50.0 {
            ChunkingStrategy::Semantic
        } else {
            ChunkingStrategy::Fixed
        };

        // Apply the chosen strategy
        match strategy {
            ChunkingStrategy::SyntaxAware => self.syntax_aware_chunking(text, language),
            ChunkingStrategy::Hierarchical => self.hierarchical_chunking(text, language),
            ChunkingStrategy::Semantic => self.semantic_chunking(text, language),
            _ => self.fixed_chunking(text, language),
        }
    }

    /// Syntax-aware chunking for code and structured content
    fn syntax_aware_chunking(&self, text: &str, language: Option<String>) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        let mut chunks: Vec<Chunk> = Vec::new();
        
        if let Some(lang) = &language {
            match lang.as_str() {
                "rust" | "python" | "javascript" | "typescript" | "java" | "cpp" | "c" => {
                    return self.code_aware_chunking(text, language);
                }
                "markdown" | "md" => {
                    return self.markdown_aware_chunking(text, language);
                }
                _ => {}
            }
        }

        // Fallback to semantic chunking for unknown syntax
        self.semantic_chunking(text, language)
    }

    /// Code-aware chunking that respects function and class boundaries
    fn code_aware_chunking(&self, text: &str, language: Option<String>) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut line_start = 0;
        let mut brace_depth = 0;
        let mut in_function = false;

        for (i, line) in lines.iter().enumerate() {
            let line_with_newline = if i < lines.len() - 1 {
                format!("{}\n", line)
            } else {
                line.to_string()
            };

            // Track brace depth for scope awareness
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;

            // Detect function/method definitions
            if line.trim().contains("fn ") || line.trim().contains("function ") || 
               line.trim().contains("def ") || line.trim().contains("class ") {
                in_function = true;
            }

            // Check if we should create a new chunk
            let should_split = current_chunk.len() + line_with_newline.len() > self.chunk_size &&
                              !current_chunk.is_empty() &&
                              (brace_depth == 0 || !in_function);

            if should_split {
                chunks.push(Chunk {
                    content: current_chunk.trim().to_string(),
                    metadata: ChunkMetadata {
                        start_offset: current_start,
                        end_offset: line_start,
                        language: language.clone(),
                        section_type: Some("code_block".to_string()),
                        importance_score: self.calculate_code_importance(&current_chunk),
                        semantic_density: self.calculate_semantic_density(&current_chunk),
                    },
                });
                
                current_chunk = String::new();
                current_start = line_start;
                in_function = false;
            }

            if current_chunk.is_empty() {
                current_start = line_start;
            }

            current_chunk.push_str(&line_with_newline);
            line_start += line_with_newline.len();

            // Reset function flag when we exit scope
            if brace_depth == 0 {
                in_function = false;
            }
        }

        // Add the last chunk if not empty
        if !current_chunk.trim().is_empty() {
            chunks.push(Chunk {
                content: current_chunk.trim().to_string(),
                metadata: ChunkMetadata {
                    start_offset: current_start,
                    end_offset: text.len(),
                    language: language.clone(),
                    section_type: Some("code_block".to_string()),
                    importance_score: self.calculate_code_importance(&current_chunk),
                    semantic_density: self.calculate_semantic_density(&current_chunk),
                },
            });
        }

        Ok(chunks)
    }

    /// Markdown-aware chunking that respects heading hierarchy
    fn markdown_aware_chunking(&self, text: &str, language: Option<String>) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut line_start = 0;
        let mut current_heading_level = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_with_newline = if i < lines.len() - 1 {
                format!("{}\n", line)
            } else {
                line.to_string()
            };

            // Check if this is a heading
            let heading_level = if line.trim_start().starts_with('#') {
                line.trim_start().chars().take_while(|&c| c == '#').count()
            } else {
                0
            };

            // Should we start a new chunk?
            let should_split = if heading_level > 0 {
                // New heading at same or higher level (lower number)
                !current_chunk.is_empty() && 
                (current_heading_level == 0 || heading_level <= current_heading_level)
            } else {
                // Size-based splitting
                current_chunk.len() + line_with_newline.len() > self.chunk_size &&
                !current_chunk.is_empty()
            };

            if should_split {
                chunks.push(Chunk {
                    content: current_chunk.trim().to_string(),
                    metadata: ChunkMetadata {
                        start_offset: current_start,
                        end_offset: line_start,
                        language: language.clone(),
                        section_type: Some(format!("markdown_h{}", current_heading_level)),
                        importance_score: self.calculate_markdown_importance(&current_chunk, current_heading_level),
                        semantic_density: self.calculate_semantic_density(&current_chunk),
                    },
                });
                
                current_chunk = String::new();
                current_start = line_start;
            }

            if current_chunk.is_empty() {
                current_start = line_start;
            }

            if heading_level > 0 {
                current_heading_level = heading_level;
            }

            current_chunk.push_str(&line_with_newline);
            line_start += line_with_newline.len();
        }

        // Add the last chunk if not empty
        if !current_chunk.trim().is_empty() {
            chunks.push(Chunk {
                content: current_chunk.trim().to_string(),
                metadata: ChunkMetadata {
                    start_offset: current_start,
                    end_offset: text.len(),
                    language: language.clone(),
                    section_type: Some(format!("markdown_h{}", current_heading_level)),
                    importance_score: self.calculate_markdown_importance(&current_chunk, current_heading_level),
                    semantic_density: self.calculate_semantic_density(&current_chunk),
                },
            });
        }

        Ok(chunks)
    }

    // Helper methods for advanced chunking

    /// Split text into sentences using regex patterns
    fn split_into_sentences(&self, text: &str) -> Vec<&str> {
        self.sentence_boundary.split(text).collect()
    }

    /// Split text into sections based on structure (headings, paragraphs, etc.)
    fn split_into_sections(&self, text: &str) -> Vec<(usize, String, String)> {
        let mut sections = Vec::new();
        let mut current_section = String::new();
        let mut section_start = 0;
        let mut current_type = "content".to_string();
        let mut char_offset = 0;

        for line in text.lines() {
            let line_with_newline = format!("{}\n", line);
            
            // Check if this is a heading
            if self.heading_pattern.is_match(line) {
                // Save previous section if not empty
                if !current_section.trim().is_empty() {
                    sections.push((section_start, current_section.trim().to_string(), current_type.clone()));
                }
                
                // Start new section
                current_section = line_with_newline.clone();
                section_start = char_offset;
                current_type = "heading".to_string();
            } else if self.paragraph_boundary.is_match(&line_with_newline) {
                // Paragraph boundary - might start new section
                if current_section.len() > self.chunk_size {
                    sections.push((section_start, current_section.trim().to_string(), current_type.clone()));
                    current_section = String::new();
                    section_start = char_offset;
                    current_type = "content".to_string();
                }
                current_section.push_str(&line_with_newline);
            } else {
                current_section.push_str(&line_with_newline);
            }
            
            char_offset += line_with_newline.len();
        }

        // Add the last section
        if !current_section.trim().is_empty() {
            sections.push((section_start, current_section.trim().to_string(), current_type));
        }

        sections
    }

    /// Calculate semantic coherence between two text segments
    fn calculate_semantic_coherence(&self, text1: &str, text2: &str) -> f32 {
        // Simple word overlap-based coherence
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Calculate importance score based on content characteristics
    fn calculate_importance_score(&self, text: &str) -> f32 {
        let mut score = 0.5; // Base score
        
        // Boost for headings
        if self.heading_pattern.is_match(text) {
            score += 0.3;
        }
        
        // Boost for code blocks
        if self.code_block_boundary.is_match(text) {
            score += 0.2;
        }
        
        // Boost for longer content (more information)
        let length_factor = (text.len() as f32 / self.chunk_size as f32).min(1.0);
        score += length_factor * 0.2;
        
        // Boost for content with numbers/data
        let number_count = text.matches(char::is_numeric).count();
        if number_count > 5 {
            score += 0.1;
        }
        
        score.min(1.0)
    }

    /// Calculate semantic density (information richness)
    fn calculate_semantic_density(&self, text: &str) -> f32 {
        let words: Vec<&str> = text.split_whitespace().collect();
        let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
        
        if words.is_empty() {
            return 0.0;
        }
        
        let lexical_diversity = unique_words.len() as f32 / words.len() as f32;
        
        // Factor in sentence complexity
        let sentences = self.split_into_sentences(text);
        let avg_sentence_length = if sentences.is_empty() {
            0.0
        } else {
            words.len() as f32 / sentences.len() as f32
        };
        
        // Normalize sentence length (optimal around 15-20 words)
        let sentence_complexity = (avg_sentence_length / 20.0).min(1.0);
        
        (lexical_diversity + sentence_complexity) / 2.0
    }

    /// Calculate average sentence length
    fn calculate_average_sentence_length(&self, text: &str) -> f32 {
        let sentences = self.split_into_sentences(text);
        if sentences.is_empty() {
            return 0.0;
        }
        
        let total_words: usize = sentences.iter()
            .map(|s| s.split_whitespace().count())
            .sum();
        
        total_words as f32 / sentences.len() as f32
    }

    /// Calculate importance score for code content
    fn calculate_code_importance(&self, code: &str) -> f32 {
        let mut score = 0.5;
        
        // Boost for function definitions
        if code.contains("fn ") || code.contains("function ") || code.contains("def ") {
            score += 0.3;
        }
        
        // Boost for class definitions
        if code.contains("class ") || code.contains("struct ") || code.contains("impl ") {
            score += 0.3;
        }
        
        // Boost for imports/includes
        if code.contains("import ") || code.contains("use ") || code.contains("#include") {
            score += 0.2;
        }
        
        // Boost for comments (documentation)
        let comment_lines = code.lines().filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('#')
        }).count();
        
        if comment_lines > 0 {
            score += (comment_lines as f32 / code.lines().count() as f32) * 0.2;
        }
        
        score.min(1.0)
    }

    /// Calculate importance score for markdown content
    fn calculate_markdown_importance(&self, content: &str, heading_level: usize) -> f32 {
        let mut score = 0.5;
        
        // Higher importance for higher-level headings (lower numbers)
        if heading_level > 0 {
            score += (7 - heading_level.min(6)) as f32 * 0.1;
        }
        
        // Boost for lists
        let list_items = content.lines().filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("- ") || trimmed.starts_with("* ") || 
            trimmed.chars().next().map_or(false, |c| c.is_ascii_digit())
        }).count();
        
        if list_items > 0 {
            score += (list_items as f32 / content.lines().count() as f32) * 0.2;
        }
        
        // Boost for code blocks
        if self.code_block_boundary.is_match(content) {
            score += 0.2;
        }
        
        // Boost for links
        let link_count = content.matches("[").count().min(content.matches("](").count());
        if link_count > 0 {
            score += (link_count as f32 * 0.05).min(0.2);
        }
        
        score.min(1.0)
    }

    
    pub fn chunk_text(&self, text: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();
        let text_len = text.len();

        if text_len == 0 {
            return Ok(chunks);
        }

        if text_len <= self.chunk_size {
            chunks.push(text.to_string());
            return Ok(chunks);
        }

        let mut start = 0;
        while start < text_len {
            let end = std::cmp::min(start + self.chunk_size, text_len);
            let chunk_text = &text[start..end];

            
            let chunk = if end < text_len {
                if let Some(last_space) = chunk_text.rfind(|c: char| c.is_whitespace()) {
                    &chunk_text[..last_space]
                } else {
                    chunk_text
                }
            } else {
                chunk_text
            };

            chunks.push(chunk.trim().to_string());

            if end >= text_len {
                break;
            }

            
            start = if chunk.len() > self.chunk_overlap {
                start + chunk.len() - self.chunk_overlap
            } else {
                start + chunk.len()
            };
        }

        Ok(chunks)
    }

    
    pub fn chunk_code(
        &self,
        code: &str,
        language: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        
        
        self.chunk_text(code)
    }

    
    pub fn chunk_markdown(&self, markdown: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_size = 0;

        for line in markdown.lines() {
            
            let is_heading = line.trim_start().starts_with('#');

            
            if current_size + line.len() > self.chunk_size && !current_chunk.is_empty() {
                
                if is_heading {
                    chunks.push(current_chunk.trim().to_string());
                    current_chunk = String::new();
                    current_size = 0;
                }
            }

            current_chunk.push_str(line);
            current_chunk.push('\n');
            current_size += line.len() + 1;

            
            if current_size >= self.chunk_size * 2 {
                chunks.push(current_chunk.trim().to_string());
                current_chunk = String::new();
                current_size = 0;
            }
        }

        
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        if chunks.is_empty() {
            chunks.push(markdown.to_string());
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_short_text() {
        let config = crate::config::IndexerConfig::from_env();
        let chunking = ChunkingService::new(config);
        
        let text = "Short text";
        let chunks = chunking.chunk_text(text).unwrap();
        
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_chunk_long_text() {
        let mut config = crate::config::IndexerConfig::from_env();
        config.chunk_size = 50;
        config.chunk_overlap = 10;
        
        let chunking = ChunkingService::new(config);
        
        let text = "This is a longer text that will be split into multiple chunks. Each chunk should have some overlap with the previous one.";
        let chunks = chunking.chunk_text(text).unwrap();
        
        assert!(chunks.len() > 1);
    }
}
