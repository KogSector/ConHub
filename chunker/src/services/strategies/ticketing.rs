//! Ticketing Chunker - Handles Issues and Pull Requests
//!
//! This chunker is optimized for GitHub Issues and PRs, which have:
//! - A title and description (main content)
//! - A conversation thread (comments, reviews)
//! - Metadata (labels, assignees, state, etc.)
//!
//! Chunking strategy:
//! 1. First chunk: Title + Description + Key metadata
//! 2. Subsequent chunks: Groups of comments/reviews (preserving conversation context)

use anyhow::Result;
use uuid::Uuid;
use tracing::debug;

use conhub_models::chunking::{SourceItem, Chunk};

/// Configuration for ticketing chunker
pub struct TicketingChunkerConfig {
    /// Maximum characters per chunk
    pub max_chunk_size: usize,
    /// Number of comments to group together
    pub comments_per_chunk: usize,
    /// Overlap between comment chunks (number of comments)
    pub comment_overlap: usize,
}

impl Default for TicketingChunkerConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 4000, // ~1000 tokens
            comments_per_chunk: 5,
            comment_overlap: 1,
        }
    }
}

pub struct TicketingChunker;

impl TicketingChunker {
    /// Chunk a ticketing item (issue or PR)
    pub fn chunk(item: &SourceItem) -> Result<Vec<Chunk>> {
        Self::chunk_with_config(item, &TicketingChunkerConfig::default())
    }
    
    /// Chunk with custom configuration
    pub fn chunk_with_config(item: &SourceItem, config: &TicketingChunkerConfig) -> Result<Vec<Chunk>> {
        let content = &item.content;
        let mut chunks = Vec::new();
        
        // Determine if this is an issue or PR
        let doc_type = item.metadata.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("issue");
        
        // Parse the content into sections
        let sections = parse_ticketing_content(content);
        
        debug!("📋 Parsing {} with {} sections", doc_type, sections.len());
        
        // Chunk 1: Header (title, metadata, description)
        let header_content = build_header_chunk(&sections, config.max_chunk_size);
        if !header_content.is_empty() {
            chunks.push(Chunk {
                chunk_id: generate_chunk_id(&item.id, 0),
                source_item_id: item.id,
                chunk_index: 0,
                content: header_content,
                start_offset: Some(0),
                end_offset: None,
                block_type: Some(format!("{}_header", doc_type)),
                language: None,
                metadata: serde_json::json!({
                    "chunk_type": "header",
                    "doc_type": doc_type,
                    "title": item.metadata.get("title"),
                    "number": item.metadata.get("number"),
                    "state": item.metadata.get("state"),
                }),
            });
        }
        
        // Chunk 2+: Comments/Reviews in groups
        let conversation_sections: Vec<&TicketingSection> = sections.iter()
            .filter(|s| matches!(s.section_type, SectionType::Comment | SectionType::Review | SectionType::ReviewComment))
            .collect();
        
        if !conversation_sections.is_empty() {
            let comment_chunks = chunk_conversation(
                &conversation_sections,
                config.comments_per_chunk,
                config.comment_overlap,
                config.max_chunk_size,
            );
            
            for (idx, comment_content) in comment_chunks.into_iter().enumerate() {
                let chunk_index = (chunks.len()) as u32;
                chunks.push(Chunk {
                    chunk_id: generate_chunk_id(&item.id, chunk_index),
                    source_item_id: item.id,
                    chunk_index,
                    content: comment_content,
                    start_offset: None,
                    end_offset: None,
                    block_type: Some(format!("{}_conversation", doc_type)),
                    language: None,
                    metadata: serde_json::json!({
                        "chunk_type": "conversation",
                        "doc_type": doc_type,
                        "conversation_chunk": idx + 1,
                        "title": item.metadata.get("title"),
                        "number": item.metadata.get("number"),
                    }),
                });
            }
        }
        
        // If content is too large and we only have header, split it further
        if chunks.len() == 1 && chunks[0].content.len() > config.max_chunk_size {
            let large_content = chunks.remove(0);
            let split_chunks = split_large_content(&large_content.content, config.max_chunk_size);
            
            for (idx, content) in split_chunks.into_iter().enumerate() {
                chunks.push(Chunk {
                    chunk_id: generate_chunk_id(&item.id, idx as u32),
                    source_item_id: item.id,
                    chunk_index: idx as u32,
                    content,
                    start_offset: None,
                    end_offset: None,
                    block_type: Some(format!("{}_content", doc_type)),
                    language: None,
                    metadata: serde_json::json!({
                        "chunk_type": "content",
                        "doc_type": doc_type,
                        "part": idx + 1,
                        "title": item.metadata.get("title"),
                        "number": item.metadata.get("number"),
                    }),
                });
            }
        }
        
        debug!("✂️ Created {} chunks for {}", chunks.len(), doc_type);
        
        Ok(chunks)
    }
}

// ============================================================================
// Content Parsing
// ============================================================================

#[derive(Debug)]
enum SectionType {
    Title,
    Metadata,
    Description,
    Comment,
    Review,
    ReviewComment,
    CodeBlock,
    Other,
}

#[derive(Debug)]
struct TicketingSection {
    section_type: SectionType,
    author: Option<String>,
    timestamp: Option<String>,
    content: String,
}

fn parse_ticketing_content(content: &str) -> Vec<TicketingSection> {
    let mut sections = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut current_section: Option<TicketingSection> = None;
    let mut in_code_block = false;
    
    for line in lines {
        // Track code blocks
        if line.starts_with("```") {
            in_code_block = !in_code_block;
        }
        
        // Skip processing inside code blocks
        if in_code_block {
            if let Some(ref mut section) = current_section {
                section.content.push_str(line);
                section.content.push('\n');
            }
            continue;
        }
        
        // Detect section headers
        if line.starts_with("# Issue #") || line.starts_with("# Pull Request #") {
            // Save previous section
            if let Some(section) = current_section.take() {
                sections.push(section);
            }
            current_section = Some(TicketingSection {
                section_type: SectionType::Title,
                author: None,
                timestamp: None,
                content: line.to_string() + "\n",
            });
        } else if line.starts_with("**State:**") || line.starts_with("**Author:**") || 
                  line.starts_with("**Labels:**") || line.starts_with("**Branch:**") ||
                  line.starts_with("**Assignees:**") || line.starts_with("**Changes:**") ||
                  line.starts_with("**Merged:**") {
            // Metadata line - append to current or create new
            if let Some(ref mut section) = current_section {
                if matches!(section.section_type, SectionType::Title | SectionType::Metadata) {
                    section.content.push_str(line);
                    section.content.push('\n');
                    section.section_type = SectionType::Metadata;
                } else {
                    sections.push(current_section.take().unwrap());
                    current_section = Some(TicketingSection {
                        section_type: SectionType::Metadata,
                        author: None,
                        timestamp: None,
                        content: line.to_string() + "\n",
                    });
                }
            } else {
                current_section = Some(TicketingSection {
                    section_type: SectionType::Metadata,
                    author: None,
                    timestamp: None,
                    content: line.to_string() + "\n",
                });
            }
        } else if line.starts_with("## Description") {
            if let Some(section) = current_section.take() {
                sections.push(section);
            }
            current_section = Some(TicketingSection {
                section_type: SectionType::Description,
                author: None,
                timestamp: None,
                content: String::new(),
            });
        } else if line.starts_with("## Comments") || line.starts_with("## Discussion") {
            if let Some(section) = current_section.take() {
                sections.push(section);
            }
            // Don't create a section for the header itself
            current_section = None;
        } else if line.starts_with("## Reviews") {
            if let Some(section) = current_section.take() {
                sections.push(section);
            }
            current_section = None;
        } else if line.starts_with("## Code Review Comments") {
            if let Some(section) = current_section.take() {
                sections.push(section);
            }
            current_section = None;
        } else if line.starts_with("### @") {
            // Comment or review header
            if let Some(section) = current_section.take() {
                sections.push(section);
            }
            
            // Parse author from "### @username (timestamp)" or "### @username - STATE"
            let author = line.trim_start_matches("### @")
                .split(|c| c == ' ' || c == '(' || c == '-')
                .next()
                .map(|s| s.to_string());
            
            let is_review = line.contains("APPROVED") || line.contains("CHANGES_REQUESTED") || 
                           line.contains("COMMENTED") || line.contains("DISMISSED");
            
            current_section = Some(TicketingSection {
                section_type: if is_review { SectionType::Review } else { SectionType::Comment },
                author,
                timestamp: extract_timestamp(line),
                content: line.to_string() + "\n",
            });
        } else if line.starts_with("### ") && line.contains(" on `") {
            // Review comment on file
            if let Some(section) = current_section.take() {
                sections.push(section);
            }
            
            let author = line.trim_start_matches("### @")
                .split(" on ")
                .next()
                .map(|s| s.to_string());
            
            current_section = Some(TicketingSection {
                section_type: SectionType::ReviewComment,
                author,
                timestamp: None,
                content: line.to_string() + "\n",
            });
        } else {
            // Regular content line
            if let Some(ref mut section) = current_section {
                section.content.push_str(line);
                section.content.push('\n');
            }
        }
    }
    
    // Don't forget the last section
    if let Some(section) = current_section {
        sections.push(section);
    }
    
    sections
}

fn extract_timestamp(line: &str) -> Option<String> {
    // Try to extract timestamp from "(YYYY-MM-DD HH:MM)" format
    if let Some(start) = line.find('(') {
        if let Some(end) = line.find(')') {
            let timestamp = &line[start + 1..end];
            if timestamp.contains('-') && timestamp.len() > 8 {
                return Some(timestamp.to_string());
            }
        }
    }
    None
}

fn build_header_chunk(sections: &[TicketingSection], max_size: usize) -> String {
    let mut content = String::new();
    
    for section in sections {
        match section.section_type {
            SectionType::Title | SectionType::Metadata | SectionType::Description => {
                if content.len() + section.content.len() <= max_size {
                    content.push_str(&section.content);
                } else {
                    // Truncate if needed
                    let remaining = max_size.saturating_sub(content.len());
                    if remaining > 100 {
                        content.push_str(&section.content[..remaining.min(section.content.len())]);
                        content.push_str("\n...[truncated]");
                    }
                    break;
                }
            }
            _ => break, // Stop at first non-header section
        }
    }
    
    content.trim().to_string()
}

fn chunk_conversation(
    sections: &[&TicketingSection],
    comments_per_chunk: usize,
    overlap: usize,
    max_size: usize,
) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut i = 0;
    
    while i < sections.len() {
        let mut chunk_content = String::new();
        let end = (i + comments_per_chunk).min(sections.len());
        
        for section in &sections[i..end] {
            if chunk_content.len() + section.content.len() <= max_size {
                chunk_content.push_str(&section.content);
                chunk_content.push('\n');
            } else {
                // If single comment is too large, truncate it
                let remaining = max_size.saturating_sub(chunk_content.len());
                if remaining > 200 {
                    chunk_content.push_str(&section.content[..remaining.min(section.content.len())]);
                    chunk_content.push_str("\n...[truncated]\n");
                }
                break;
            }
        }
        
        if !chunk_content.trim().is_empty() {
            chunks.push(chunk_content.trim().to_string());
        }
        
        // Move forward with overlap
        i += comments_per_chunk.saturating_sub(overlap);
        if i == 0 {
            i = 1; // Ensure progress
        }
    }
    
    chunks
}

fn split_large_content(content: &str, max_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    
    for line in content.lines() {
        if current.len() + line.len() + 1 > max_size {
            if !current.is_empty() {
                chunks.push(current.trim().to_string());
                current = String::new();
            }
            
            // If single line is too large, split it
            if line.len() > max_size {
                let mut start = 0;
                while start < line.len() {
                    let end = (start + max_size).min(line.len());
                    chunks.push(line[start..end].to_string());
                    start = end;
                }
            } else {
                current = line.to_string() + "\n";
            }
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }
    
    chunks
}

fn generate_chunk_id(source_id: &Uuid, chunk_index: u32) -> Uuid {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    source_id.hash(&mut hasher);
    chunk_index.hash(&mut hasher);
    
    let hash = hasher.finish();
    let bytes = hash.to_le_bytes();
    
    // Create a UUID from the hash (not cryptographically secure, but deterministic)
    Uuid::from_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[0] ^ bytes[4], bytes[1] ^ bytes[5],
        bytes[2] ^ bytes[6], bytes[3] ^ bytes[7],
        bytes[4] ^ bytes[0], bytes[5] ^ bytes[1],
        bytes[6] ^ bytes[2], bytes[7] ^ bytes[3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_parse_issue_content() {
        let content = r#"# Issue #123: Test Issue

**State:** open
**Author:** @testuser
**Labels:** bug, help-wanted

---

## Description

This is a test issue description.

---

## Comments

### @commenter1 (2024-01-15 10:30)

First comment here.

### @commenter2 (2024-01-15 11:00)

Second comment here.
"#;
        
        let sections = parse_ticketing_content(content);
        assert!(!sections.is_empty());
        
        // Should have title/metadata, description, and comments
        let has_title = sections.iter().any(|s| matches!(s.section_type, SectionType::Title | SectionType::Metadata));
        let has_description = sections.iter().any(|s| matches!(s.section_type, SectionType::Description));
        let has_comments = sections.iter().any(|s| matches!(s.section_type, SectionType::Comment));
        
        assert!(has_title);
        assert!(has_description);
        assert!(has_comments);
    }
    
    #[test]
    fn test_chunk_issue() {
        let item = SourceItem {
            id: Uuid::new_v4(),
            source_id: Uuid::new_v4(),
            source_kind: conhub_models::chunking::SourceKind::Ticketing,
            content_type: "text/issue".to_string(),
            content: "# Issue #1: Test\n\n**State:** open\n\n## Description\n\nTest description.".to_string(),
            metadata: serde_json::json!({
                "type": "issue",
                "number": 1,
                "title": "Test"
            }),
            created_at: Some(Utc::now()),
        };
        
        let chunks = TicketingChunker::chunk(&item).unwrap();
        assert!(!chunks.is_empty());
        assert!(chunks[0].content.contains("Issue #1"));
    }
}
