use dgraph::DgraphError;
use juniper::FieldError;
use std::convert::From;
use std::fmt;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum QueryCreationError {
    Composite(CompositeQueryCreationError),
    UnknownField(String),
    MissingArgument(String),
    InvalidArgument(String, String),
}

impl fmt::Display for QueryCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Composite(composite) => write!(f, "{}", composite),
            Self::UnknownField(field) => write!(f, "Unknown field: {}", field),
            Self::MissingArgument(arg) => write!(f, "Missing argument: {}", arg),
            Self::InvalidArgument(arg, msg) => write!(f, "Invalid argument: {} - {}", arg, msg),
        }
    }
}

#[derive(Debug)]
pub struct CompositeQueryCreationError {
    pub at_field: String,
    pub children: Vec<QueryCreationError>,
}

impl fmt::Display for CompositeQueryCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let children = self
            .children
            .iter()
            .map(|child| format!("{}", child))
            .collect::<Vec<_>>()
            .join(",\n");
        write!(f, "{}: {{\n{}\n}}", self.at_field, children)
    }
}

#[derive(Debug)]
pub enum DgraphQueryError {
    Dgraph(DgraphError),
    InvalidUTF(FromUtf8Error),
    InvalidJson(serde_json::Error),
}

impl From<DgraphError> for DgraphQueryError {
    fn from(err: DgraphError) -> DgraphQueryError {
        DgraphQueryError::Dgraph(err)
    }
}

impl From<FromUtf8Error> for DgraphQueryError {
    fn from(err: FromUtf8Error) -> DgraphQueryError {
        DgraphQueryError::InvalidUTF(err)
    }
}

impl From<serde_json::Error> for DgraphQueryError {
    fn from(err: serde_json::Error) -> DgraphQueryError {
        DgraphQueryError::InvalidJson(err)
    }
}

const INTERNAL_ERROR: &str = "internal_error";
const INVALID_INPUT: &str = "invalid_input";
const UNABLE_TO_RESOLVE_FIELD: &str = "Unable to resolve field";

pub fn internal_error() -> FieldError {
    FieldError::new(
        UNABLE_TO_RESOLVE_FIELD,
        graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
    )
}

pub fn invalid_input(msg: &str) -> FieldError {
    FieldError::new(INVALID_INPUT, graphql_value!({ INVALID_INPUT: msg }))
}
