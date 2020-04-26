#[derive(Debug)]
pub enum QueryCreationError {
    Composite(CompositeQueryCreationError),
    UnknownField(String),
}

#[derive(Debug)]
pub struct CompositeQueryCreationError {
    pub at_field: String,
    pub children: Vec<QueryCreationError>,
}

pub const INTERNAL_ERROR: &str = "internal_error";
pub const UNABLE_TO_RESOLVE_FIELD: &str = "Unable to resolve field";
pub const DB_QUERY_RESULT_PARSE_ERR: &str = "Error parsing DB query result";
pub const DB_QUERY_GENERATION_ERR: &str = "Error generating DB query";
