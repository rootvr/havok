#![allow(unused)]

use crate::parser;

/// Crate Error type
#[derive(Debug)]
pub enum Error {
    Pest(Box<pest::error::Error<parser::Rule>>),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pest(e) => write!(f, "{e}"),
            Self::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<pest::error::Error<parser::Rule>> for Error {
    fn from(value: pest::error::Error<parser::Rule>) -> Self {
        Self::Pest(Box::new(value))
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

/// Crate Result type
pub type Result<T> = std::result::Result<T, Error>;
