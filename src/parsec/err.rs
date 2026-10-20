use std::fmt::Display;
use std::error::Error;
use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub struct DataError(ErrorLocation);

impl DataError {
    pub fn new(location: ErrorLocation) -> Box<Self> {
        Box::new(DataError(location))
    }
}

impl Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "Expected more input")
    }
}

impl Error for DataError {}

impl ParseError for DataError {
    fn location(&self) -> &ErrorLocation { &self.0 }
}
