use std::error::Error;
use std::fmt::Display;

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub struct NotError(ErrorLocation);

impl NotError {
    pub fn new(location: ErrorLocation) -> Box<Self> {
        Box::new(NotError(location))
    }
}

impl Display for NotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "Inner parser unexpectedly succeeded")
    }
}

impl Error for NotError {}

impl ParseError for NotError {
    fn location(&self) -> &ErrorLocation { &self.0 }
}
