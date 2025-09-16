use crate::types::*;
use std::process::Command;
use std::path::Path;
use uuid::Uuid;
use serde_json::Value;

pub struct CtagsIntegration {
    ctags_path: String,
    supported_languages: Vec<String>,
}

impl CtagsIntegration {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ctags_path = Self::find_ctags_executable()?;
        let supported_languages = Self::get_ctags_languages(&ctags_path)?;

        Ok(Self {
            ctags_path,
            supported_languages,
        })
    }

    fn find_ctags_executable() -> Result<String, Box<dyn std::error::Error>> {
        // Try different common ctags executables
        for ctags_cmd in &["universal-ctags", "ctags", "exuberant-ctags"] {
            if let Ok(output) = Command::new(ctags_cmd).arg("--version").output() {
                if output.status.success() {
                    return Ok(ctags_cmd.to_string());
                }
            }
        }
        Err("Universal Ctags not found. Please install universal-ctags.".into())
    }

    fn get_ctags_languages(ctags_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = Command::new(ctags_path)
            .args(&["--list-languages"])
            .output()?;

        if !output.status.success() {
            return Err("Failed to get ctags languages".into());
        }

        let languages = String::from_utf8(output.stdout)?
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(languages)
    }

    pub fn parse_file(&self, file_id: Uuid, path: &Path) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        let output = Command::new(&self.ctags_path)
            .args(&[
                "--output-format=json",
                "--fields=+n+S+Z+K+l+s+t",
                "--extras=+r+q",
                "--sort=no",
                "-f", "-",
                path.to_str().unwrap()
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Ctags failed: {}", stderr).into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut symbols = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            match serde_json::from_str::<Value>(line) {
                Ok(tag) => {
                    if let Some(symbol) = self.parse_ctags_entry(file_id, &tag) {
                        symbols.push(symbol);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse ctags JSON line: {} - Error: {}", line, e);
                }
            }
        }

        Ok(symbols)
    }

    pub fn parse_content(&self, file_id: Uuid, content: &str, language: &Language) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        use tempfile::NamedTempFile;
        use std::io::Write;

        // Create temporary file with appropriate extension
        let extension = self.get_file_extension(language);
        let mut temp_file = NamedTempFile::new()?;
        
        // Write content to temporary file
        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;

        // Rename with proper extension for ctags language detection
        let temp_path = temp_file.path();
        let temp_with_ext = temp_path.with_extension(extension);
        std::fs::copy(temp_path, &temp_with_ext)?;

        let result = self.parse_file(file_id, &temp_with_ext);
        
        // Clean up
        let _ = std::fs::remove_file(&temp_with_ext);
        
        result
    }

    fn get_file_extension(&self, language: &Language) -> &str {
        match language {
            Language::Rust => "rs",
            Language::JavaScript => "js",
            Language::TypeScript => "ts",
            Language::Python => "py",
            Language::Java => "java",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Go => "go",
            Language::Php => "php",
            Language::Ruby => "rb",
            Language::CSharp => "cs",
            Language::Swift => "swift",
            Language::Kotlin => "kt",
            Language::Scala => "scala",
            Language::Perl => "pl",
            Language::Lua => "lua",
            Language::Shell => "sh",
            Language::Sql => "sql",
            Language::Html => "html",
            Language::Css => "css",
            Language::Xml => "xml",
            Language::Json => "json",
            Language::Yaml => "yml",
            Language::Markdown => "md",
            _ => "txt",
        }
    }

    fn parse_ctags_entry(&self, file_id: Uuid, tag: &Value) -> Option<Symbol> {
        let name = tag.get("name")?.as_str()?.to_string();
        let kind = tag.get("kind")?.as_str()?;
        let line = tag.get("line")?.as_u64()? as u32;
        
        // Extract additional fields
        let signature = tag.get("signature")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        
        let scope = tag.get("scope")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let namespace = tag.get("namespace")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        // Map ctags kind to our SymbolType
        let symbol_type = self.map_ctags_kind_to_symbol_type(kind);

        Some(Symbol {
            id: Uuid::new_v4(),
            file_id,
            name,
            symbol_type,
            line,
            column: 0, // Ctags doesn't provide column information by default
            end_line: line,
            end_column: 0,
            signature,
            scope,
            namespace,
        })
    }

    fn map_ctags_kind_to_symbol_type(&self, kind: &str) -> SymbolType {
        match kind {
            // Functions and methods
            "function" | "f" | "func" | "method" | "m" => SymbolType::Function,
            "procedure" | "p" => SymbolType::Function,
            
            // Classes and types
            "class" | "c" => SymbolType::Class,
            "interface" | "i" => SymbolType::Interface,
            "struct" | "s" => SymbolType::Struct,
            "enum" | "e" | "enumeration" => SymbolType::Enum,
            "union" | "u" => SymbolType::Struct,
            
            // Variables and fields
            "variable" | "v" | "var" => SymbolType::Variable,
            "field" | "member" | "attribute" => SymbolType::Field,
            "constant" | "d" | "define" | "macro" => SymbolType::Constant,
            "parameter" | "z" => SymbolType::Parameter,
            
            // Modules and namespaces
            "namespace" | "n" => SymbolType::Namespace,
            "module" | "M" => SymbolType::Module,
            "package" | "P" => SymbolType::Module,
            
            // Type definitions
            "typedef" | "t" => SymbolType::Type,
            "alias" | "a" => SymbolType::Type,
            
            // Language-specific mappings
            "constructor" => SymbolType::Method,
            "destructor" => SymbolType::Method,
            "property" => SymbolType::Field,
            "event" => SymbolType::Field,
            "delegate" => SymbolType::Type,
            "label" | "l" => SymbolType::Variable,
            "local" => SymbolType::Variable,
            "externvar" => SymbolType::Variable,
            "header" => SymbolType::Module,
            
            _ => SymbolType::Variable, // Default fallback
        }
    }

    pub fn get_supported_languages(&self) -> &[String] {
        &self.supported_languages
    }

    pub fn supports_language(&self, language: &Language) -> bool {
        let lang_name = match language {
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Java => "Java",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Rust => "Rust",
            Language::Go => "Go",
            Language::Php => "PHP",
            Language::Ruby => "Ruby",
            Language::CSharp => "C#",
            Language::Swift => "Swift",
            Language::Kotlin => "Kotlin",
            Language::Scala => "Scala",
            Language::Perl => "Perl",
            Language::Lua => "Lua",
            Language::Shell => "Sh",
            Language::Sql => "SQL",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Xml => "XML",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Markdown => "Markdown",
            _ => return false,
        };

        self.supported_languages.iter().any(|lang| lang == lang_name)
    }

    pub fn get_language_info(&self) -> Result<Vec<LanguageInfo>, Box<dyn std::error::Error>> {
        let output = Command::new(&self.ctags_path)
            .args(&["--list-languages", "--machinable"])
            .output()?;

        if !output.status.success() {
            return Err("Failed to get language info from ctags".into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut languages = Vec::new();

        for line in stdout.lines() {
            if let Some((name, extensions)) = line.split_once('\t') {
                languages.push(LanguageInfo {
                    name: name.to_string(),
                    extensions: extensions.split(',').map(|s| s.trim().to_string()).collect(),
                });
            }
        }

        Ok(languages)
    }
}

#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub name: String,
    pub extensions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_ctags_integration() {
        if let Ok(ctags) = CtagsIntegration::new() {
            let languages = ctags.get_supported_languages();
            assert!(!languages.is_empty());
            println!("Supported languages: {:?}", languages);
        }
    }

    #[test]
    fn test_parse_rust_code() {
        if let Ok(ctags) = CtagsIntegration::new() {
            let rust_code = r#"
                struct MyStruct {
                    field: i32,
                }
                
                impl MyStruct {
                    fn new() -> Self {
                        Self { field: 0 }
                    }
                    
                    fn method(&self) -> i32 {
                        self.field
                    }
                }
                
                fn main() {
                    let instance = MyStruct::new();
                    println!("{}", instance.method());
                }
            "#;

            let file_id = Uuid::new_v4();
            if let Ok(symbols) = ctags.parse_content(file_id, rust_code, &Language::Rust) {
                assert!(!symbols.is_empty());
                
                // Check for expected symbols
                let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
                assert!(symbol_names.contains(&"MyStruct"));
                assert!(symbol_names.contains(&"new"));
                assert!(symbol_names.contains(&"method"));
                assert!(symbol_names.contains(&"main"));
            }
        }
    }
}