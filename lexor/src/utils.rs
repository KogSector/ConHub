use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};
use sha2::{Sha256, Digest};
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252, ISO_8859_1};
use regex::Regex;
use once_cell::sync::Lazy;

pub fn calculate_file_hash(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}

pub fn detect_encoding(bytes: &[u8]) -> &'static Encoding {
    // Simple encoding detection
    if bytes.is_empty() {
        return UTF_8;
    }
    
    // Check for BOM
    if bytes.len() >= 3 && &bytes[0..3] == b"\xEF\xBB\xBF" {
        return UTF_8;
    }
    
    if bytes.len() >= 2 {
        if &bytes[0..2] == b"\xFF\xFE" || &bytes[0..2] == b"\xFE\xFF" {
            return UTF_8; // Simplified - would need proper UTF-16 handling
        }
    }
    
    // Check if it's valid UTF-8
    if std::str::from_utf8(bytes).is_ok() {
        return UTF_8;
    }
    
    // Fallback to common encodings
    let mut non_ascii_count = 0;
    let mut high_bit_count = 0;
    
    for &byte in bytes.iter().take(1024) {
        if byte > 127 {
            non_ascii_count += 1;
            if byte >= 0x80 && byte <= 0x9F {
                high_bit_count += 1;
            }
        }
    }
    
    if non_ascii_count == 0 {
        UTF_8
    } else if high_bit_count > non_ascii_count / 2 {
        WINDOWS_1252
    } else {
        ISO_8859_1
    }
}

pub fn read_file_with_encoding(path: &Path) -> io::Result<(String, String)> {
    let bytes = fs::read(path)?;
    let encoding = detect_encoding(&bytes);
    
    let (content, _, had_errors) = encoding.decode(&bytes);
    if had_errors {
        log::warn!("Encoding errors detected in file: {:?}", path);
    }
    
    Ok((content.into_owned(), encoding.name().to_string()))
}

pub fn is_binary_file(path: &Path) -> bool {
    if let Ok(bytes) = fs::read(path) {
        is_binary_content(&bytes)
    } else {
        false
    }
}

pub fn is_binary_content(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    
    // Check first 8KB for null bytes
    let check_size = std::cmp::min(bytes.len(), 8192);
    let sample = &bytes[0..check_size];
    
    // If more than 30% are non-printable, consider it binary
    let non_printable = sample.iter()
        .filter(|&&b| b == 0 || (b < 32 && b != 9 && b != 10 && b != 13))
        .count();
    
    non_printable as f64 / sample.len() as f64 > 0.30
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    
    for component in path.components() {
        match component {
            std::path::Component::Normal(name) => {
                components.push(name);
            }
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {
                // Skip current directory references
            }
            _ => {
                // Keep other components (like root)
                components.push(component.as_os_str());
            }
        }
    }
    
    components.iter().collect()
}

pub fn get_relative_path(base: &Path, target: &Path) -> Option<PathBuf> {
    target.strip_prefix(base).ok().map(|p| p.to_path_buf())
}

pub fn count_lines(content: &str) -> u32 {
    if content.is_empty() {
        0
    } else {
        content.lines().count() as u32
    }
}

pub fn extract_words(content: &str) -> Vec<String> {
    static WORD_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap()
    });
    
    WORD_REGEX
        .find_iter(content)
        .map(|m| m.as_str().to_lowercase())
        .collect()
}

pub fn highlight_matches(content: &str, query: &str, case_sensitive: bool) -> Vec<(usize, usize)> {
    let search_content = if case_sensitive { content } else { &content.to_lowercase() };
    let search_query = if case_sensitive { query } else { &query.to_lowercase() };
    
    let mut matches = Vec::new();
    let mut start = 0;
    
    while let Some(pos) = search_content[start..].find(search_query) {
        let absolute_pos = start + pos;
        matches.push((absolute_pos, absolute_pos + search_query.len()));
        start = absolute_pos + 1;
    }
    
    matches
}

pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

pub fn is_ignored_file(path: &Path, ignored_patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    
    for pattern in ignored_patterns {
        if pattern.contains('*') || pattern.contains('?') {
            // Simple glob matching
            if glob_match(pattern, &path_str) {
                return true;
            }
        } else if path_str.contains(pattern) {
            return true;
        }
    }
    
    false
}

fn glob_match(pattern: &str, text: &str) -> bool {
    // Simple glob implementation
    let regex_pattern = pattern
        .replace(".", r"\.")
        .replace("*", ".*")
        .replace("?", ".");
    
    if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
        regex.is_match(text)
    } else {
        false
    }
}

pub fn extract_context_lines(content: &str, line_number: u32, context_size: usize) -> Vec<(u32, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    
    if total_lines == 0 || line_number == 0 {
        return Vec::new();
    }
    
    let line_idx = (line_number - 1) as usize;
    let start = line_idx.saturating_sub(context_size);
    let end = std::cmp::min(line_idx + context_size + 1, total_lines);
    
    lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, &line)| ((start + i + 1) as u32, line.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_binary_content() {
        assert!(!is_binary_content(b"Hello, world!"));
        assert!(is_binary_content(&[0, 1, 2, 3, 4, 5]));
        assert!(!is_binary_content(b""));
    }

    #[test]
    fn test_count_lines() {
        assert_eq!(count_lines(""), 0);
        assert_eq!(count_lines("single line"), 1);
        assert_eq!(count_lines("line 1\nline 2"), 2);
        assert_eq!(count_lines("line 1\nline 2\n"), 2);
    }

    #[test]
    fn test_extract_words() {
        let words = extract_words("hello world_test 123abc");
        assert!(words.contains(&"hello".to_string()));
        assert!(words.contains(&"world_test".to_string()));
        assert!(!words.contains(&"123abc".to_string()));
    }

    #[test]
    fn test_highlight_matches() {
        let matches = highlight_matches("Hello World Hello", "hello", false);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (0, 5));
        assert_eq!(matches[1], (12, 17));
    }

    #[test]
    fn test_normalize_path() {
        let path = Path::new("./src/../target/./debug");
        let normalized = normalize_path(path);
        assert_eq!(normalized, Path::new("target/debug"));
    }
}