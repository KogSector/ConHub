use uuid::Uuid;
use anyhow::Result;
use pulldown_cmark::{Parser as MarkdownParser, Event, Tag, HeadingLevel};

use conhub_models::chunking::{SourceItem, Chunk};

/// Enhanced Markdown chunker that respects heading hierarchy
pub struct MarkdownChunker;

impl MarkdownChunker {
    const MAX_TOKENS: usize = 512;
    const MIN_CHUNK_SIZE: usize = 100;

    pub fn chunk(source_item: &SourceItem) -> Result<Vec<Chunk>> {
        let content = &source_item.content;
        
        // Parse markdown and extract sections by headings
        let sections = Self::extract_sections(content);

        if sections.is_empty() {
            // No headings found, fallback to paragraph chunking
            return super::text::TextChunker::chunk(source_item);
        }

        Self::create_chunks_from_sections(source_item, sections)
    }

    fn extract_sections(content: &str) -> Vec<Section> {
        let parser = MarkdownParser::new(content);
        let mut sections = Vec::new();
        let mut current_section: Option<Section> = None;
        let mut heading_stack: Vec<(usize, String)> = Vec::new(); // (level, text)
        let mut current_text = String::new();
        let mut position = 0;

        for event in parser {
            match event {
                Event::Start(Tag::Heading(level, _, _)) => {
                    // Save previous section
                    if let Some(mut section) = current_section.take() {
                        section.content = current_text.clone();
                        section.end_offset = position;
                        sections.push(section);
                    }

                    current_text.clear();
                    
                    // Update heading stack
                    let level_num = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };

                    // Pop headings of same or lower level
                    while heading_stack.last().map(|(l, _)| *l >= level_num).unwrap_or(false) {
                        heading_stack.pop();
                    }

                    current_section = Some(Section {
                        heading_level: level_num,
                        heading_path: heading_stack.iter().map(|(_, t)| t.clone()).collect(),
                        heading_text: String::new(),
                        content: String::new(),
                        start_offset: position,
                        end_offset: position,
                    });
                }
                Event::Text(text) => {
                    if let Some(ref mut section) = current_section {
                        if section.heading_text.is_empty() {
                            // This is the heading text
                            section.heading_text = text.to_string();
                            heading_stack.push((section.heading_level, text.to_string()));
                        } else {
                            current_text.push_str(&text);
                        }
                    } else {
                        current_text.push_str(&text);
                    }
                    position += text.len();
                }
                Event::Code(code) => {
                    current_text.push('`');
                    current_text.push_str(&code);
                    current_text.push('`');
                    position += code.len() + 2;
                }
                Event::SoftBreak | Event::HardBreak => {
                    current_text.push('\n');
                    position += 1;
                }
                _ => {}
            }
        }

        // Add final section
        if let Some(mut section) = current_section {
            section.content = current_text;
            section.end_offset = position;
            sections.push(section);
        }

        sections
    }

    fn create_chunks_from_sections(
        source_item: &SourceItem,
        sections: Vec<Section>,
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let mut current_group: Vec<&Section> = Vec::new();
        let mut current_tokens = 0;

        for section in &sections {
            let section_tokens = Self::estimate_tokens(&section.content);

            // If this section alone is too big, chunk it separately
            if section_tokens > Self::MAX_TOKENS {
                // Flush current group first
                if !current_group.is_empty() {
                    chunks.extend(Self::create_chunk_from_group(
                        source_item,
                        &current_group,
                        chunks.len(),
                    )?);
                    current_group.clear();
                    current_tokens = 0;
                }

                // Split large section
                chunks.extend(Self::split_large_section(source_item, section, chunks.len())?);
            } else if current_tokens + section_tokens > Self::MAX_TOKENS {
                // Current group is full, flush it
                if !current_group.is_empty() {
                    chunks.extend(Self::create_chunk_from_group(
                        source_item,
                        &current_group,
                        chunks.len(),
                    )?);
                    current_group.clear();
                    current_tokens = 0;
                }
                
                current_group.push(section);
                current_tokens = section_tokens;
            } else {
                // Add to current group
                current_group.push(section);
                current_tokens += section_tokens;
            }
        }

        // Flush remaining group
        if !current_group.is_empty() {
            chunks.extend(Self::create_chunk_from_group(
                source_item,
                &current_group,
                chunks.len(),
            )?);
        }

        Ok(chunks)
    }

    fn create_chunk_from_group(
        source_item: &SourceItem,
        sections: &[&Section],
        base_index: usize,
    ) -> Result<Vec<Chunk>> {
        let mut content = String::new();
        let start_offset = sections.first().map(|s| s.start_offset).unwrap_or(0);
        let end_offset = sections.last().map(|s| s.end_offset).unwrap_or(0);

        for section in sections {
            if !section.heading_text.is_empty() {
                content.push_str(&"#".repeat(section.heading_level));
                content.push(' ');
                content.push_str(&section.heading_text);
                content.push_str("\n\n");
            }
            content.push_str(&section.content);
            content.push_str("\n\n");
        }

        let chunk_id = Self::generate_chunk_id(&source_item.id, base_index);
        
        let mut metadata = source_item.metadata.clone();
        metadata["chunk_index"] = serde_json::json!(base_index);
        metadata["heading_path"] = serde_json::json!(sections.first().map(|s| s.heading_path.clone()).unwrap_or_default());
        metadata["section_count"] = serde_json::json!(sections.len());
        metadata["token_count"] = serde_json::json!(Self::estimate_tokens(&content));

        Ok(vec![Chunk {
            chunk_id,
            source_item_id: source_item.id,
            chunk_index: base_index as u32,
            content: content.trim().to_string(),
            start_offset: Some(start_offset as u32),
            end_offset: Some(end_offset as u32),
            block_type: Some("markdown".to_string()),
            language: Some("markdown".to_string()),
            metadata,
        }])
    }

    fn split_large_section(
        source_item: &SourceItem,
        section: &Section,
        base_index: usize,
    ) -> Result<Vec<Chunk>> {
        // Split by paragraphs
        let paragraphs: Vec<&str> = section.content.split("\n\n").collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_count = 0;

        // Add heading to each chunk
        let heading_prefix = format!(
            "{} {}\n\n",
            "#".repeat(section.heading_level),
            section.heading_text
        );

        for para in paragraphs {
            let para_tokens = Self::estimate_tokens(para);
            let current_tokens = Self::estimate_tokens(&current_chunk);

            if current_tokens + para_tokens > Self::MAX_TOKENS && !current_chunk.is_empty() {
                // Flush current chunk
                let chunk_id = Self::generate_chunk_id(&source_item.id, base_index + chunk_count);
                let full_content = format!("{}{}", heading_prefix, current_chunk.trim());
                
                let mut metadata = source_item.metadata.clone();
                metadata["chunk_index"] = serde_json::json!(base_index + chunk_count);
                metadata["heading_path"] = serde_json::json!(section.heading_path);
                metadata["heading_text"] = serde_json::json!(section.heading_text);
                metadata["sub_chunk"] = serde_json::json!(chunk_count);

                chunks.push(Chunk {
                    chunk_id,
                    source_item_id: source_item.id,
                    chunk_index: (base_index + chunk_count) as u32,
                    content: full_content,
                    start_offset: Some(section.start_offset as u32),
                    end_offset: Some(section.end_offset as u32),
                    block_type: Some("markdown".to_string()),
                    language: Some("markdown".to_string()),
                    metadata,
                });

                current_chunk.clear();
                chunk_count += 1;
            }

            current_chunk.push_str(para);
            current_chunk.push_str("\n\n");
        }

        // Flush final chunk
        if !current_chunk.trim().is_empty() {
            let chunk_id = Self::generate_chunk_id(&source_item.id, base_index + chunk_count);
            let full_content = format!("{}{}", heading_prefix, current_chunk.trim());
            
            let mut metadata = source_item.metadata.clone();
            metadata["chunk_index"] = serde_json::json!(base_index + chunk_count);
            metadata["heading_path"] = serde_json::json!(section.heading_path);
            metadata["heading_text"] = serde_json::json!(section.heading_text);
            metadata["sub_chunk"] = serde_json::json!(chunk_count);

            chunks.push(Chunk {
                chunk_id,
                source_item_id: source_item.id,
                chunk_index: (base_index + chunk_count) as u32,
                content: full_content,
                start_offset: Some(section.start_offset as u32),
                end_offset: Some(section.end_offset as u32),
                block_type: Some("markdown".to_string()),
                language: Some("markdown".to_string()),
                metadata,
            });
        }

        Ok(chunks)
    }

    fn estimate_tokens(text: &str) -> usize {
        // Rough estimation: 4 chars per token
        text.len() / 4
    }

    fn generate_chunk_id(source_item_id: &Uuid, chunk_index: usize) -> Uuid {
        let namespace = uuid::Uuid::NAMESPACE_OID;
        let name = format!("{}-md-{}", source_item_id, chunk_index);
        uuid::Uuid::new_v5(&namespace, name.as_bytes())
    }
}

#[derive(Debug, Clone)]
struct Section {
    heading_level: usize,
    heading_path: Vec<String>,
    heading_text: String,
    content: String,
    start_offset: usize,
    end_offset: usize,
}
