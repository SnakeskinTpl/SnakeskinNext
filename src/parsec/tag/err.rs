use std::fmt::Display;
use std::error::Error;
use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub struct TagError {
    location: ErrorLocation,
    pub expected_tag: String,
    pub expected: char,
}

impl TagError {
    pub fn new(location: ErrorLocation, expected_tag: impl Into<String>, expected: char) -> Box<Self> {
        Box::new(Self { location, expected_tag: expected_tag.into(), expected })
    }
}

impl Display for TagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.location.actual {
            Some(actual) => write!(f, "At byte position {}, expected character {:?} of tag {:?}, got {:?}", self.location.byte_position, self.expected, self.expected_tag, actual),
            None => write!(f, "At byte position {}, expected character {:?} of tag {:?}, reached end of input", self.location.byte_position, self.expected, self.expected_tag),
        }
    }
}

impl Error for TagError {}

impl ParseError for TagError {
    fn location(&self) -> &ErrorLocation { &self.location }
}
