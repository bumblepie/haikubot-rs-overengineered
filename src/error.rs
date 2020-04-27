use dgraph::DgraphError;
use std::string::FromUtf8Error;

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

#[derive(Debug)]
pub enum DgraphQueryError {
    Dgraph(DgraphError),
    InvalidUTF(FromUtf8Error),
    InvalidJson(serde_json::Error),
}

impl std::convert::From<DgraphError> for DgraphQueryError {
    fn from(err: DgraphError) -> DgraphQueryError {
        DgraphQueryError::Dgraph(err)
    }
}

impl std::convert::From<FromUtf8Error> for DgraphQueryError {
    fn from(err: FromUtf8Error) -> DgraphQueryError {
        DgraphQueryError::InvalidUTF(err)
    }
}

impl std::convert::From<serde_json::Error> for DgraphQueryError {
    fn from(err: serde_json::Error) -> DgraphQueryError {
        DgraphQueryError::InvalidJson(err)
    }
}

pub const INTERNAL_ERROR: &str = "internal_error";
pub const UNABLE_TO_RESOLVE_FIELD: &str = "Unable to resolve field";
pub const DB_QUERY_RESULT_PARSE_ERR: &str = "Error parsing DB query result";
pub const DB_QUERY_GENERATION_ERR: &str = "Error generating DB query";
