use uuid::Uuid;
use anyhow::Result;

use conhub_models::chunking::{SourceItem, Chunk};

pub struct TextChunker;

impl TextChunker {
    const MAX_CHUNK_SIZE: usize = 1000; // characters
    const CHUNK_OVERLAP: usize = 200;

    /// Chunk text documents by paragraphs and sliding windows
    pub fn chunk(source_item: &SourceItem) -> Result<Vec<Chunk>> {
        let content = &source_item.content;

        // Try paragraph-based chunking first
        let para_chunks = Self::chunk_by_paragraphs(content);
        
        if !para_chunks.is_empty() {
            return Self::create_chunks_from_paragraphs(source_item, para_chunks);
        }

        // Fallback to sliding window
        Self::sliding_window_chunks(source_item)
    }

    fn chunk_by_paragraphs(content: &str) -> Vec<(usize, usize, String)> {
        let mut chunks = Vec::new();
        let mut current_start = 0;
        let mut current_chunk = String::new();

        for (idx, line) in content.lines().enumerate() {
            let line_with_newline = format!("{}\n", line);
            
            // Empty line indicates paragraph break
            if line.trim().is_empty() {
                if !current_chunk.trim().is_empty() {
                    let end = current_start + current_chunk.len();
                    chunks.push((current_start, end, current_chunk.clone()));
                    current_start = end;
                    current_chunk.clear();
                }
                current_start += line_with_newline.len();
                continue;
            }

            current_chunk.push_str(&line_with_newline);

            // If chunk is getting too large, break here
            if current_chunk.len() > Self::MAX_CHUNK_SIZE {
                let end = current_start + current_chunk.len();
                chunks.push((current_start, end, current_chunk.clone()));
                current_start = end;
                current_chunk.clear();
            }
        }

        // Add final chunk if any
        if !current_chunk.trim().is_empty() {
            let end = current_start + current_chunk.len();
            chunks.push((current_start, end, current_chunk));
        }

        chunks
    }

    fn create_chunks_from_paragraphs(
        source_item: &SourceItem,
        paragraphs: Vec<(usize, usize, String)>,
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();

        for (idx, (start, end, content)) in paragraphs.into_iter().enumerate() {
            let chunk_id = Self::generate_chunk_id(&source_item.id, idx);
            
            let mut metadata = source_item.metadata.clone();
            metadata["chunk_index"] = serde_json::json!(idx);

            chunks.push(Chunk {
                chunk_id,
                source_item_id: source_item.id,
                chunk_index: idx as u32,
                content,
                start_offset: Some(start as u32),
                end_offset: Some(end as u32),
                block_type: Some("text".to_string()),
                language: None,
                metadata,
            });
        }

        Ok(chunks)
    }

    fn sliding_window_chunks(source_item: &SourceItem) -> Result<Vec<Chunk>> {
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
                        block_type: Some("text".to_string()),
                        language: None,
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
        // Look for sentence endings or newlines
        for i in (0..bytes.len()).rev() {
            if bytes[i] == b'\n' {
                return Some(i + 1);
            }
            if i > 0 && (bytes[i] == b'.' || bytes[i] == b'!' || bytes[i] == b'?') {
                if i + 1 < bytes.len() && bytes[i + 1] == b' ' {
                    return Some(i + 2);
                }
            }
        }
        None
    }

    fn generate_chunk_id(source_item_id: &Uuid, chunk_index: usize) -> Uuid {
        let namespace = uuid::Uuid::NAMESPACE_OID;
        let name = format!("{}-{}", source_item_id, chunk_index);
        uuid::Uuid::new_v5(&namespace, name.as_bytes())
    }
}
