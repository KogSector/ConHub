use uuid::Uuid;
use anyhow::Result;

use conhub_models::chunking::{SourceItem, Chunk};

pub struct ChatChunker;

impl ChatChunker {
    const MESSAGES_PER_CHUNK: usize = 15;
    const OVERLAP_MESSAGES: usize = 3;

    /// Chunk chat/conversation content by message windows with overlap
    pub fn chunk(source_item: &SourceItem) -> Result<Vec<Chunk>> {
        let content = &source_item.content;

        // Try to parse as structured messages if metadata contains them
        if let Some(messages_val) = source_item.metadata.get("messages") {
            if let Some(messages_arr) = messages_val.as_array() {
                return Self::chunk_structured_messages(source_item, messages_arr);
            }
        }

        // Fallback: chunk by line breaks (assuming each message is a line)
        Self::chunk_by_lines(source_item, content)
    }

    fn chunk_structured_messages(
        source_item: &SourceItem,
        messages: &[serde_json::Value],
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let mut chunk_index = 0;

        let mut start_idx = 0;
        while start_idx < messages.len() {
            let end_idx = (start_idx + Self::MESSAGES_PER_CHUNK).min(messages.len());
            let window = &messages[start_idx..end_idx];

            // Build content from messages
            let mut content = String::new();
            for msg in window {
                if let Some(text) = msg.get("text").and_then(|t| t.as_str()) {
                    let author = msg.get("author")
                        .and_then(|a| a.as_str())
                        .unwrap_or("Unknown");
                    content.push_str(&format!("{}: {}\n", author, text));
                } else if let Some(text) = msg.as_str() {
                    content.push_str(&format!("{}\n", text));
                }
            }

            if !content.trim().is_empty() {
                let chunk_id = Self::generate_chunk_id(&source_item.id, chunk_index);
                
                let mut metadata = source_item.metadata.clone();
                metadata["chunk_index"] = serde_json::json!(chunk_index);
                metadata["message_start"] = serde_json::json!(start_idx);
                metadata["message_end"] = serde_json::json!(end_idx);
                metadata["message_count"] = serde_json::json!(end_idx - start_idx);

                chunks.push(Chunk {
                    chunk_id,
                    source_item_id: source_item.id,
                    chunk_index: chunk_index as u32,
                    content,
                    start_offset: None,
                    end_offset: None,
                    block_type: Some("chat".to_string()),
                    language: None,
                    metadata,
                });
                
                chunk_index += 1;
            }

            // Move window with overlap
            start_idx += Self::MESSAGES_PER_CHUNK - Self::OVERLAP_MESSAGES;
            if start_idx >= messages.len() {
                break;
            }
        }

        Ok(chunks)
    }

    fn chunk_by_lines(source_item: &SourceItem, content: &str) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut chunk_index = 0;

        let mut start_line = 0;
        while start_line < lines.len() {
            let end_line = (start_line + Self::MESSAGES_PER_CHUNK).min(lines.len());
            let window = &lines[start_line..end_line];

            let chunk_content = window.join("\n");
            
            if !chunk_content.trim().is_empty() {
                let chunk_id = Self::generate_chunk_id(&source_item.id, chunk_index);
                
                let mut metadata = source_item.metadata.clone();
                metadata["chunk_index"] = serde_json::json!(chunk_index);
                metadata["line_start"] = serde_json::json!(start_line);
                metadata["line_end"] = serde_json::json!(end_line);

                chunks.push(Chunk {
                    chunk_id,
                    source_item_id: source_item.id,
                    chunk_index: chunk_index as u32,
                    content: chunk_content,
                    start_offset: None,
                    end_offset: None,
                    block_type: Some("chat".to_string()),
                    language: None,
                    metadata,
                });
                
                chunk_index += 1;
            }

            start_line += Self::MESSAGES_PER_CHUNK - Self::OVERLAP_MESSAGES;
            if start_line >= lines.len() {
                break;
            }
        }

        Ok(chunks)
    }

    fn generate_chunk_id(source_item_id: &Uuid, chunk_index: usize) -> Uuid {
        let namespace = uuid::Uuid::NAMESPACE_OID;
        let name = format!("{}-{}", source_item_id, chunk_index);
        uuid::Uuid::new_v5(&namespace, name.as_bytes())
    }
}
