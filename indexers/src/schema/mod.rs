// Re-export the base schema types under the top-level `schema` module so
// existing code that refers to `schema::FieldSchema`, `schema::ValueType`,
// etc. continues to work. Keep the `evolution` submodule alongside them.
pub use crate::base::schema::*;

pub mod evolution;