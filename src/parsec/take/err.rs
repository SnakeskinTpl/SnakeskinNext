use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub struct TakeError {
    location: ErrorLocation,
    pub min: usize,
    pub matched: usize,
}

impl TakeError {
    pub fn new(location: ErrorLocation, min: usize, matched: usize) -> Box<Self> {
        Box::new(Self { location, min, matched })
    }
}

impl Display for TakeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "At byte position {}, expected at least {} matching characters, matched {}", self.location.failure_byte, self.min, self.matched)?;

        match self.location.found_char {
            Some(actual) => write!(f, ", got {:?}", actual),
            None => write!(f, ", reached end of input"),
        }
    }
}

impl Error for TakeError {}

impl ParseError for TakeError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }
}
