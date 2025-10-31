#[derive(Debug, Clone)]
pub struct ToJsonSchemaOptions {
    pub fields_always_required: bool,
    pub supports_format: bool,
    pub extract_descriptions: bool,
    pub top_level_must_be_object: bool,
}