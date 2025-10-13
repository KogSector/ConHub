use std::path::Path;


pub fn detect_file_type(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}


pub fn is_code_file(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "h" | "hpp"
            | "go" | "rb" | "php" | "cs" | "swift" | "kt" | "scala" | "sh" | "bash"
            | "sql" | "yaml" | "yml" | "toml" | "json" | "xml" | "html" | "css" | "scss"
    )
}


pub fn is_doc_file(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "md" | "markdown" | "txt" | "rst" | "adoc" | "asciidoc"
    )
}


pub fn extract_repo_name(url: &str) -> Option<String> {
    url.split('/')
        .last()
        .map(|name| name.trim_end_matches(".git").to_string())
}


pub fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}


pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file("rs"));
        assert!(is_code_file("py"));
        assert!(is_code_file("js"));
        assert!(!is_code_file("txt"));
        assert!(!is_code_file("pdf"));
    }

    #[test]
    fn test_is_doc_file() {
        assert!(is_doc_file("md"));
        assert!(is_doc_file("txt"));
        assert!(!is_doc_file("rs"));
        assert!(!is_doc_file("pdf"));
    }

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git"),
            Some("repo".to_string())
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/my-project"),
            Some("my-project".to_string())
        );
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1023), "1023.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("file/name.txt"), "file_name.txt");
        assert_eq!(sanitize_filename("file:name?.txt"), "file_name_.txt");
    }
}
