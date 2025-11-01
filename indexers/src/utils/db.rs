use serde::{Deserialize, Serialize};

/// Represents different types of write actions that can be performed on a database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteAction {
    /// Insert a new record
    Insert,
    /// Update an existing record
    Update,
    /// Delete a record
    Delete,
    /// Upsert (insert or update) a record
    Upsert,
}

impl WriteAction {
    /// Check if this action creates or modifies data
    pub fn is_mutating(&self) -> bool {
        matches!(self, WriteAction::Insert | WriteAction::Update | WriteAction::Upsert)
    }

    /// Check if this action removes data
    pub fn is_deleting(&self) -> bool {
        matches!(self, WriteAction::Delete)
    }
}

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