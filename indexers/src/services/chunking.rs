use crate::config::IndexerConfig;

pub struct ChunkingService {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl ChunkingService {
    pub fn new(config: IndexerConfig) -> Self {
        Self {
            chunk_size: config.chunk_size,
            chunk_overlap: config.chunk_overlap,
        }
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
