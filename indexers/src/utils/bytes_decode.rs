use encoding_rs::{Encoding, UTF_8};

/// Convert bytes to string with encoding detection
pub fn bytes_to_string(bytes: &[u8]) -> (String, &'static Encoding) {
    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(bytes) {
        return (s.to_string(), UTF_8);
    }
    
    // Fall back to encoding detection
    let (cow, encoding, _had_errors) = encoding_rs::UTF_8.decode(bytes);
    (cow.into_owned(), encoding)
}