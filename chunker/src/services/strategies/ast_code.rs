use uuid::Uuid;
use anyhow::{Result, Context};
use tree_sitter::{Language, Parser, Query, QueryCursor, Node};

use conhub_models::chunking::{SourceItem, Chunk};

extern "C" { fn tree_sitter_rust() -> Language; }
extern "C" { fn tree_sitter_python() -> Language; }
extern "C" { fn tree_sitter_typescript() -> Language; }
extern "C" { fn tree_sitter_javascript() -> Language; }
extern "C" { fn tree_sitter_go() -> Language; }
extern "C" { fn tree_sitter_java() -> Language; }
extern "C" { fn tree_sitter_c() -> Language; }
extern "C" { fn tree_sitter_cpp() -> Language; }

/// Advanced AST-based code chunker using tree-sitter
pub struct AstCodeChunker;

impl AstCodeChunker {
    const MAX_TOKENS: usize = 512;
    const OVERLAP_TOKENS: usize = 64;

    pub fn chunk(source_item: &SourceItem) -> Result<Vec<Chunk>> {
        let language = Self::detect_language(source_item);
        
        if let Some(lang) = Self::get_tree_sitter_language(&language) {
            Self::chunk_with_ast(source_item, lang, &language)
        } else {
            // Fallback to regex-based chunking for unsupported languages
            super::code::CodeChunker::chunk(source_item)
        }
    }

    fn detect_language(source_item: &SourceItem) -> String {
        // Try metadata first
        if let Some(lang) = source_item.metadata.get("language") {
            if let Some(lang_str) = lang.as_str() {
                return lang_str.to_lowercase();
            }
        }

        // Try path extension
        if let Some(path) = source_item.metadata.get("path") {
            if let Some(path_str) = path.as_str() {
                if let Some(ext) = path_str.rsplit('.').next() {
                    return ext.to_lowercase();
                }
            }
        }

        "unknown".to_string()
    }

    fn get_tree_sitter_language(lang: &str) -> Option<Language> {
        unsafe {
            match lang {
                "rust" | "rs" => Some(tree_sitter_rust()),
                "python" | "py" => Some(tree_sitter_python()),
                "typescript" | "ts" => Some(tree_sitter_typescript()),
                "javascript" | "js" | "jsx" => Some(tree_sitter_javascript()),
                "go" => Some(tree_sitter_go()),
                "java" => Some(tree_sitter_java()),
                "c" | "h" => Some(tree_sitter_c()),
                "cpp" | "cc" | "cxx" | "hpp" => Some(tree_sitter_cpp()),
                _ => None,
            }
        }
    }

    fn chunk_with_ast(
        source_item: &SourceItem,
        language: Language,
        lang_name: &str,
    ) -> Result<Vec<Chunk>> {
        let mut parser = Parser::new();
        parser.set_language(language)
            .context("Failed to set parser language")?;

        let tree = parser.parse(&source_item.content, None)
            .context("Failed to parse source code")?;

        let root_node = tree.root_node();
        
        // Extract top-level constructs (functions, classes, structs, etc.)
        let constructs = Self::extract_top_level_constructs(
            root_node,
            source_item.content.as_bytes(),
            lang_name,
        );

        if constructs.is_empty() {
            // No constructs found, use sliding window
            return Self::sliding_window_chunks(source_item, lang_name);
        }

        Self::create_chunks_from_constructs(source_item, constructs, lang_name)
    }

    fn extract_top_level_constructs(
        node: Node,
        source: &[u8],
        lang: &str,
    ) -> Vec<(usize, usize, String, String)> {
        let mut constructs = Vec::new();

        // Language-specific node types for top-level constructs
        let function_types = match lang {
            "rust" | "rs" => vec!["function_item", "impl_item"],
            "python" | "py" => vec!["function_definition", "class_definition"],
            "typescript" | "ts" | "javascript" | "js" => {
                vec!["function_declaration", "class_declaration", "method_definition"]
            }
            "go" => vec!["function_declaration", "method_declaration"],
            "java" => vec!["method_declaration", "class_declaration"],
            "c" | "h" | "cpp" | "cc" => vec!["function_definition", "struct_specifier", "class_specifier"],
            _ => vec!["function_definition", "class_definition"],
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let node_type = child.kind();
            
            if function_types.contains(&node_type) {
                let start = child.start_byte();
                let end = child.end_byte();
                
                // Try to extract name
                let name = Self::extract_name(&child, source).unwrap_or_else(|| "anonymous".to_string());
                
                constructs.push((start, end, node_type.to_string(), name));
            }

            // Recursively look inside modules/namespaces
            if matches!(node_type, "module" | "namespace" | "impl_item" | "block") {
                constructs.extend(Self::extract_top_level_constructs(child, source, lang));
            }
        }

        constructs
    }

    fn extract_name(node: &Node, source: &[u8]) -> Option<String> {
        // Try to find name field or identifier child
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Ok(name_str) = std::str::from_utf8(&source[name_node.start_byte()..name_node.end_byte()]) {
                return Some(name_str.to_string());
            }
        }

        // Fallback: look for first identifier
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                if let Ok(name_str) = std::str::from_utf8(&source[child.start_byte()..child.end_byte()]) {
                    return Some(name_str.to_string());
                }
            }
        }

        None
    }

    fn create_chunks_from_constructs(
        source_item: &SourceItem,
        constructs: Vec<(usize, usize, String, String)>,
        language: &str,
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let content = &source_item.content;

        for (idx, (start, end, construct_type, name)) in constructs.into_iter().enumerate() {
            let construct_content = &content[start..end];

            // Check token count
            let token_count = Self::estimate_tokens(construct_content);

            if token_count > Self::MAX_TOKENS {
                // Split large constructs
                let sub_chunks = Self::split_large_construct(construct_content, start);
                for (sub_idx, (sub_content, sub_start, sub_end)) in sub_chunks.into_iter().enumerate() {
                    let chunk_id = Self::generate_chunk_id(&source_item.id, idx * 100 + sub_idx);
                    
                    let mut metadata = source_item.metadata.clone();
                    metadata["chunk_index"] = serde_json::json!(idx * 100 + sub_idx);
                    metadata["construct_type"] = serde_json::json!(construct_type);
                    metadata["construct_name"] = serde_json::json!(name);
                    metadata["sub_chunk"] = serde_json::json!(sub_idx);
                    metadata["token_count"] = serde_json::json!(Self::estimate_tokens(&sub_content));

                    chunks.push(Chunk {
                        chunk_id,
                        source_item_id: source_item.id,
                        chunk_index: (idx * 100 + sub_idx) as u32,
                        content: sub_content,
                        start_offset: Some(sub_start as u32),
                        end_offset: Some(sub_end as u32),
                        block_type: Some(construct_type.clone()),
                        language: Some(language.to_string()),
                        metadata,
                    });
                }
            } else {
                let chunk_id = Self::generate_chunk_id(&source_item.id, idx);
                
                let mut metadata = source_item.metadata.clone();
                metadata["chunk_index"] = serde_json::json!(idx);
                metadata["construct_type"] = serde_json::json!(construct_type);
                metadata["construct_name"] = serde_json::json!(name);
                metadata["token_count"] = serde_json::json!(token_count);

                chunks.push(Chunk {
                    chunk_id,
                    source_item_id: source_item.id,
                    chunk_index: idx as u32,
                    content: construct_content.to_string(),
                    start_offset: Some(start as u32),
                    end_offset: Some(end as u32),
                    block_type: Some(construct_type),
                    language: Some(language.to_string()),
                    metadata,
                });
            }
        }

        Ok(chunks)
    }

    fn split_large_construct(content: &str, base_offset: usize) -> Vec<(String, usize, usize)> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_chunk = String::new();
        let mut chunk_start_line = 0;

        for (idx, line) in lines.iter().enumerate() {
            current_chunk.push_str(line);
            current_chunk.push('\n');

            let token_count = Self::estimate_tokens(&current_chunk);

            if token_count >= Self::MAX_TOKENS {
                // Emit current chunk
                let start_offset = base_offset + lines[..chunk_start_line].iter().map(|l| l.len() + 1).sum::<usize>();
                let end_offset = base_offset + lines[..=idx].iter().map(|l| l.len() + 1).sum::<usize>();
                
                if !current_chunk.trim().is_empty() {
                    chunks.push((current_chunk.clone(), start_offset, end_offset));
                }

                // Start new chunk with overlap
                let overlap_lines = Self::OVERLAP_TOKENS / 10; // rough estimate
                chunk_start_line = idx.saturating_sub(overlap_lines);
                current_chunk = lines[chunk_start_line..=idx].join("\n") + "\n";
            }
        }

        // Add final chunk
        if !current_chunk.trim().is_empty() {
            let start_offset = base_offset + lines[..chunk_start_line].iter().map(|l| l.len() + 1).sum::<usize>();
            let end_offset = base_offset + content.len();
            chunks.push((current_chunk, start_offset, end_offset));
        }

        chunks
    }

    fn sliding_window_chunks(source_item: &SourceItem, language: &str) -> Result<Vec<Chunk>> {
        // Fallback to simple sliding window
        super::code::CodeChunker::chunk(source_item)
    }

    fn estimate_tokens(text: &str) -> usize {
        // Simple estimation: ~4 chars per token for code
        text.len() / 4
    }

    fn generate_chunk_id(source_item_id: &Uuid, chunk_index: usize) -> Uuid {
        let namespace = uuid::Uuid::NAMESPACE_OID;
        let name = format!("{}-ast-{}", source_item_id, chunk_index);
        uuid::Uuid::new_v5(&namespace, name.as_bytes())
    }
}
