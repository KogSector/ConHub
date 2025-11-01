/// Sanitize a string to be used as a database identifier
pub fn sanitize_identifier(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_start_matches(|c: char| c.is_numeric())
        .to_string()
}