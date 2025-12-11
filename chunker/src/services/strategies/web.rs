//! Web/HTML chunker for URL-scraped content
//! 
//! Handles HTML documents by:
//! - Extracting meaningful text from HTML
//! - Respecting semantic structure (headings, paragraphs, lists)
//! - Removing boilerplate (nav, footer, ads)
//! - Preserving code blocks and pre-formatted text

use uuid::Uuid;
use anyhow::Result;
use regex::Regex;

use conhub_models::chunking::{SourceItem, Chunk};

/// Web/HTML chunker that extracts and chunks content from HTML documents
pub struct WebChunker;

impl WebChunker {
    const MAX_TOKENS: usize = 512;
    const MIN_CHUNK_SIZE: usize = 50;

    pub fn chunk(source_item: &SourceItem) -> Result<Vec<Chunk>> {
        let content = &source_item.content;
        
        // Check if content looks like HTML
        if content.contains("<html") || content.contains("<body") || content.contains("<div") {
            Self::chunk_html(source_item)
        } else {
            // Plain text from URL - use text chunker
            super::text::TextChunker::chunk(source_item)
        }
    }

    fn chunk_html(source_item: &SourceItem) -> Result<Vec<Chunk>> {
        let html = &source_item.content;
        
        // Extract text content from HTML
        let text_content = Self::extract_text_from_html(html);
        
        // Extract code blocks separately
        let code_blocks = Self::extract_code_blocks(html);
        
        // Create chunks from text content
        let mut chunks = Self::create_text_chunks(source_item, &text_content)?;
        
        // Add code block chunks
        chunks.extend(Self::create_code_chunks(source_item, code_blocks, chunks.len())?);
        
        Ok(chunks)
    }

    fn extract_text_from_html(html: &str) -> String {
        let mut text = html.to_string();
        
        // Remove script and style tags with content
        let script_re = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
        text = script_re.replace_all(&text, "").to_string();
        
        let style_re = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
        text = style_re.replace_all(&text, "").to_string();
        
        // Remove nav, footer, aside (boilerplate)
        let nav_re = Regex::new(r"(?is)<nav[^>]*>.*?</nav>").unwrap();
        text = nav_re.replace_all(&text, "").to_string();
        
        let footer_re = Regex::new(r"(?is)<footer[^>]*>.*?</footer>").unwrap();
        text = footer_re.replace_all(&text, "").to_string();
        
        let aside_re = Regex::new(r"(?is)<aside[^>]*>.*?</aside>").unwrap();
        text = aside_re.replace_all(&text, "").to_string();
        
        // Convert headings to markdown-style
        for level in 1..=6 {
            let heading_re = Regex::new(&format!(r"(?is)<h{}[^>]*>(.*?)</h{}>", level, level)).unwrap();
            let prefix = "#".repeat(level);
            text = heading_re.replace_all(&text, |caps: &regex::Captures| {
                format!("\n\n{} {}\n\n", prefix, Self::strip_tags(&caps[1]))
            }).to_string();
        }
        
        // Convert paragraphs to double newlines
        let p_re = Regex::new(r"(?is)<p[^>]*>(.*?)</p>").unwrap();
        text = p_re.replace_all(&text, |caps: &regex::Captures| {
            format!("\n\n{}\n\n", Self::strip_tags(&caps[1]))
        }).to_string();
        
        // Convert list items
        let li_re = Regex::new(r"(?is)<li[^>]*>(.*?)</li>").unwrap();
        text = li_re.replace_all(&text, |caps: &regex::Captures| {
            format!("\n• {}", Self::strip_tags(&caps[1]))
        }).to_string();
        
        // Convert br to newlines
        let br_re = Regex::new(r"(?i)<br\s*/?>").unwrap();
        text = br_re.replace_all(&text, "\n").to_string();
        
        // Remove remaining HTML tags
        text = Self::strip_tags(&text);
        
        // Decode HTML entities
        text = Self::decode_html_entities(&text);
        
        // Normalize whitespace
        let ws_re = Regex::new(r"\n{3,}").unwrap();
        text = ws_re.replace_all(&text, "\n\n").to_string();
        
        let space_re = Regex::new(r"[ \t]+").unwrap();
        text = space_re.replace_all(&text, " ").to_string();
        
        text.trim().to_string()
    }

    fn strip_tags(html: &str) -> String {
        let tag_re = Regex::new(r"<[^>]+>").unwrap();
        tag_re.replace_all(html, "").to_string()
    }

    fn decode_html_entities(text: &str) -> String {
        text.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'")
            .replace("&nbsp;", " ")
            .replace("&#x27;", "'")
            .replace("&#x2F;", "/")
            .replace("&mdash;", "—")
            .replace("&ndash;", "–")
            .replace("&hellip;", "…")
            .replace("&copy;", "©")
            .replace("&reg;", "®")
            .replace("&trade;", "™")
    }

    fn extract_code_blocks(html: &str) -> Vec<(String, Option<String>)> {
        let mut blocks = Vec::new();
        
        // Extract <pre><code> blocks
        let pre_code_re = Regex::new(r#"(?is)<pre[^>]*>\s*<code[^>]*(?:class="[^"]*language-(\w+)[^"]*")?[^>]*>(.*?)</code>\s*</pre>"#).unwrap();
        for caps in pre_code_re.captures_iter(html) {
            let lang = caps.get(1).map(|m| m.as_str().to_string());
            let code = Self::decode_html_entities(&Self::strip_tags(&caps[2]));
            if !code.trim().is_empty() {
                blocks.push((code, lang));
            }
        }
        
        // Extract standalone <pre> blocks
        let pre_re = Regex::new(r"(?is)<pre[^>]*>(.*?)</pre>").unwrap();
        for caps in pre_re.captures_iter(html) {
            let code = Self::decode_html_entities(&Self::strip_tags(&caps[1]));
            if !code.trim().is_empty() && !blocks.iter().any(|(c, _)| c.contains(&code)) {
                blocks.push((code, None));
            }
        }
        
        blocks
    }

    fn create_text_chunks(
        source_item: &SourceItem,
        text: &str,
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        
        // Split by sections (headings)
        let sections = Self::split_by_headings(text);
        
        for (idx, section) in sections.into_iter().enumerate() {
            if section.content.trim().is_empty() {
                continue;
            }
            
            let token_count = Self::estimate_tokens(&section.content);
            
            if token_count > Self::MAX_TOKENS {
                // Split large sections
                let sub_chunks = Self::split_large_section(&section.content);
                for (sub_idx, sub_content) in sub_chunks.into_iter().enumerate() {
                    let chunk_id = Self::generate_chunk_id(&source_item.id, idx * 100 + sub_idx);
                    
                    let mut metadata = source_item.metadata.clone();
                    metadata["chunk_index"] = serde_json::json!(idx * 100 + sub_idx);
                    metadata["heading"] = serde_json::json!(section.heading.clone());
                    metadata["sub_chunk"] = serde_json::json!(sub_idx);
                    metadata["content_type"] = serde_json::json!("web_text");

                    chunks.push(Chunk {
                        chunk_id,
                        source_item_id: source_item.id,
                        chunk_index: (idx * 100 + sub_idx) as u32,
                        content: sub_content,
                        start_offset: None,
                        end_offset: None,
                        block_type: Some("web_text".to_string()),
                        language: None,
                        metadata,
                    });
                }
            } else if token_count >= Self::MIN_CHUNK_SIZE / 4 {
                let chunk_id = Self::generate_chunk_id(&source_item.id, idx);
                
                let mut metadata = source_item.metadata.clone();
                metadata["chunk_index"] = serde_json::json!(idx);
                metadata["heading"] = serde_json::json!(section.heading);
                metadata["content_type"] = serde_json::json!("web_text");

                chunks.push(Chunk {
                    chunk_id,
                    source_item_id: source_item.id,
                    chunk_index: idx as u32,
                    content: section.content,
                    start_offset: None,
                    end_offset: None,
                    block_type: Some("web_text".to_string()),
                    language: None,
                    metadata,
                });
            }
        }
        
        Ok(chunks)
    }

    fn split_by_headings(text: &str) -> Vec<WebSection> {
        let mut sections = Vec::new();
        let heading_re = Regex::new(r"(?m)^(#{1,6})\s+(.+)$").unwrap();
        
        let mut last_end = 0;
        let mut current_heading: Option<String> = None;
        
        for caps in heading_re.captures_iter(text) {
            let match_start = caps.get(0).unwrap().start();
            
            // Add content before this heading
            if match_start > last_end {
                let content = text[last_end..match_start].trim().to_string();
                if !content.is_empty() {
                    sections.push(WebSection {
                        heading: current_heading.clone(),
                        content,
                    });
                }
            }
            
            current_heading = Some(caps[2].to_string());
            last_end = caps.get(0).unwrap().end();
        }
        
        // Add remaining content
        if last_end < text.len() {
            let content = text[last_end..].trim().to_string();
            if !content.is_empty() {
                sections.push(WebSection {
                    heading: current_heading,
                    content,
                });
            }
        }
        
        // If no headings found, treat entire text as one section
        if sections.is_empty() && !text.trim().is_empty() {
            sections.push(WebSection {
                heading: None,
                content: text.trim().to_string(),
            });
        }
        
        sections
    }

    fn split_large_section(content: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        let mut current = String::new();
        
        for para in paragraphs {
            let para_tokens = Self::estimate_tokens(para);
            let current_tokens = Self::estimate_tokens(&current);
            
            if current_tokens + para_tokens > Self::MAX_TOKENS && !current.is_empty() {
                chunks.push(current.trim().to_string());
                current = para.to_string();
            } else {
                if !current.is_empty() {
                    current.push_str("\n\n");
                }
                current.push_str(para);
            }
        }
        
        if !current.trim().is_empty() {
            chunks.push(current.trim().to_string());
        }
        
        chunks
    }

    fn create_code_chunks(
        source_item: &SourceItem,
        code_blocks: Vec<(String, Option<String>)>,
        base_index: usize,
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        
        for (idx, (code, lang)) in code_blocks.into_iter().enumerate() {
            let chunk_id = Self::generate_chunk_id(&source_item.id, base_index + idx);
            
            let mut metadata = source_item.metadata.clone();
            metadata["chunk_index"] = serde_json::json!(base_index + idx);
            metadata["content_type"] = serde_json::json!("code_block");
            if let Some(ref l) = lang {
                metadata["language"] = serde_json::json!(l);
            }

            chunks.push(Chunk {
                chunk_id,
                source_item_id: source_item.id,
                chunk_index: (base_index + idx) as u32,
                content: code,
                start_offset: None,
                end_offset: None,
                block_type: Some("code".to_string()),
                language: lang,
                metadata,
            });
        }
        
        Ok(chunks)
    }

    fn estimate_tokens(text: &str) -> usize {
        text.len() / 4
    }

    fn generate_chunk_id(source_item_id: &Uuid, chunk_index: usize) -> Uuid {
        let namespace = uuid::Uuid::NAMESPACE_OID;
        let name = format!("{}-web-{}", source_item_id, chunk_index);
        uuid::Uuid::new_v5(&namespace, name.as_bytes())
    }
}

#[derive(Debug)]
struct WebSection {
    heading: Option<String>,
    content: String,
}
