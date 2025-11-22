use uuid::Uuid;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

use conhub_models::chunking::{SourceItem, Chunk};

lazy_static! {
    // Regex patterns for detecting code structures
    static ref FUNCTION_PATTERN: Regex = Regex::new(
        r"(?m)^[\t ]*(pub\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+\w+|function\s+\w+|def\s+\w+|func\s+\w+"
    ).unwrap();
    
    static ref CLASS_PATTERN: Regex = Regex::new(
        r"(?m)^[\t ]*(pub\s+)?(?:class|struct|enum|trait|interface|impl)\s+\w+"
    ).unwrap();
}

pub struct CodeChunker;

impl CodeChunker {
    const MAX_CHUNK_SIZE: usize = 1500; // tokens approximation
    const CHUNK_OVERLAP: usize = 200;

    /// Chunk code by trying to respect logical boundaries (functions, classes)
    pub fn chunk(source_item: &SourceItem) -> Result<Vec<Chunk>> {
        let content = &source_item.content;
        
        // Extract language from metadata or content_type
        let language = Self::extract_language(source_item);

        // First, try to detect logical code blocks
        let logical_chunks = Self::detect_logical_blocks(content, &language);

        if !logical_chunks.is_empty() {
            return Self::create_chunks_from_blocks(source_item, logical_chunks, &language);
        }

        // Fallback to sliding window if no logical blocks detected
        Self::sliding_window_chunks(source_item, &language)
    }

    fn extract_language(source_item: &SourceItem) -> String {
        // Try metadata first
        if let Some(lang) = source_item.metadata.get("language") {
            if let Some(lang_str) = lang.as_str() {
                return lang_str.to_lowercase();
            }
        }

        // Try content_type
        if source_item.content_type.starts_with("text/code:") {
            return source_item.content_type[10..].to_lowercase();
        }

        // Try path extension from metadata
        if let Some(path) = source_item.metadata.get("path") {
            if let Some(path_str) = path.as_str() {
                if let Some(ext) = path_str.rsplit('.').next() {
                    return ext.to_lowercase();
                }
            }
        }

        "unknown".to_string()
    }

    fn detect_logical_blocks(content: &str, language: &str) -> Vec<(usize, usize, String)> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Simple heuristic: detect function/class starts
        for (idx, line) in lines.iter().enumerate() {
            if FUNCTION_PATTERN.is_match(line) || CLASS_PATTERN.is_match(line) {
                // Find the end of this block (simple brace counting or indentation)
                let end_idx = Self::find_block_end(&lines, idx, language);
                
                let start_offset = lines[..idx].iter().map(|l| l.len() + 1).sum();
                let end_offset = lines[..=end_idx].iter().map(|l| l.len() + 1).sum();
                
                let block_type = if FUNCTION_PATTERN.is_match(line) {
                    "function"
                } else {
                    "class"
                };

                blocks.push((start_offset, end_offset, block_type.to_string()));
            }
        }

        blocks
    }

    fn find_block_end(lines: &[&str], start_idx: usize, _language: &str) -> usize {
        // Simple brace-counting heuristic
        let mut brace_count = 0;
        let mut found_opening = false;

        for (offset, line) in lines[start_idx..].iter().enumerate() {
            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    found_opening = true;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }

            if found_opening && brace_count == 0 {
                return start_idx + offset;
            }
        }

        // Didn't find matching brace, return some reasonable length
        (start_idx + 50).min(lines.len() - 1)
    }

    fn create_chunks_from_blocks(
        source_item: &SourceItem,
        blocks: Vec<(usize, usize, String)>,
        language: &str,
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let content = &source_item.content;

        for (idx, (start, end, block_type)) in blocks.iter().enumerate() {
            let block_content = &content[*start..*end];

            // If block is too large, split it further with overlap
            if block_content.len() > Self::MAX_CHUNK_SIZE {
                let sub_chunks = Self::split_large_block(block_content, *start);
                for (sub_idx, (sub_content, sub_start, sub_end)) in sub_chunks.into_iter().enumerate() {
                    let chunk_id = Self::generate_chunk_id(&source_item.id, idx + sub_idx);
                    
                    let mut metadata = source_item.metadata.clone();
                    metadata["chunk_index"] = serde_json::json!(idx + sub_idx);
                    metadata["sub_chunk_index"] = serde_json::json!(sub_idx);

                    chunks.push(Chunk {
                        chunk_id,
                        source_item_id: source_item.id,
                        chunk_index: (idx + sub_idx) as u32,
                        content: sub_content,
                        start_offset: Some(sub_start as u32),
                        end_offset: Some(sub_end as u32),
                        block_type: Some(block_type.clone()),
                        language: Some(language.to_string()),
                        metadata,
                    });
                }
            } else {
                let chunk_id = Self::generate_chunk_id(&source_item.id, idx);
                
                let mut metadata = source_item.metadata.clone();
                metadata["chunk_index"] = serde_json::json!(idx);

                chunks.push(Chunk {
                    chunk_id,
                    source_item_id: source_item.id,
                    chunk_index: idx as u32,
                    content: block_content.to_string(),
                    start_offset: Some(*start as u32),
                    end_offset: Some(*end as u32),
                    block_type: Some(block_type.clone()),
                    language: Some(language.to_string()),
                    metadata,
                });
            }
        }

        Ok(chunks)
    }

    fn split_large_block(content: &str, base_offset: usize) -> Vec<(String, usize, usize)> {
        let mut chunks = Vec::new();
        let bytes = content.as_bytes();
        let mut start = 0;

        while start < bytes.len() {
            let end = (start + Self::MAX_CHUNK_SIZE).min(bytes.len());

            // Try to find a good break point
            let chunk_end = if end < bytes.len() {
                Self::find_break_point(&bytes[start..end])
                    .map(|p| start + p)
                    .unwrap_or(end)
            } else {
                end
            };

            if let Ok(chunk_str) = String::from_utf8(bytes[start..chunk_end].to_vec()) {
                if !chunk_str.trim().is_empty() {
                    chunks.push((
                        chunk_str,
                        base_offset + start,
                        base_offset + chunk_end,
                    ));
                }
            }

            start = chunk_end.saturating_sub(Self::CHUNK_OVERLAP);
            if start >= bytes.len() {
                break;
            }
        }

        chunks
    }

    fn sliding_window_chunks(source_item: &SourceItem, language: &str) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let content = &source_item.content;
        let bytes = content.as_bytes();
        let mut start = 0;
        let mut chunk_index = 0;

        while start < bytes.len() {
            let end = (start + Self::MAX_CHUNK_SIZE).min(bytes.len());

            let chunk_end = if end < bytes.len() {
                Self::find_break_point(&bytes[start..end])
                    .map(|p| start + p)
                    .unwrap_or(end)
            } else {
                end
            };

            if let Ok(chunk_str) = String::from_utf8(bytes[start..chunk_end].to_vec()) {
                if !chunk_str.trim().is_empty() {
                    let chunk_id = Self::generate_chunk_id(&source_item.id, chunk_index);
                    
                    let mut metadata = source_item.metadata.clone();
                    metadata["chunk_index"] = serde_json::json!(chunk_index);

                    chunks.push(Chunk {
                        chunk_id,
                        source_item_id: source_item.id,
                        chunk_index,
                        content: chunk_str,
                        start_offset: Some(start as u32),
                        end_offset: Some(chunk_end as u32),
                        block_type: Some("code".to_string()),
                        language: Some(language.to_string()),
                        metadata,
                    });
                    
                    chunk_index += 1;
                }
            }

            start = chunk_end.saturating_sub(Self::CHUNK_OVERLAP);
            if start >= bytes.len() {
                break;
            }
        }

        Ok(chunks)
    }

    fn find_break_point(bytes: &[u8]) -> Option<usize> {
        // Look for newline near the end
        for i in (0..bytes.len()).rev() {
            if bytes[i] == b'\n' {
                return Some(i + 1);
            }
            // Also break at semicolons for code
            if bytes[i] == b';' || bytes[i] == b'}' {
                return Some(i + 1);
            }
        }
        None
    }

    fn generate_chunk_id(source_item_id: &Uuid, chunk_index: usize) -> Uuid {
        // Generate stable UUID from source_item_id and chunk_index
        let namespace = uuid::Uuid::NAMESPACE_OID;
        let name = format!("{}-{}", source_item_id, chunk_index);
        uuid::Uuid::new_v5(&namespace, name.as_bytes())
    }
}
