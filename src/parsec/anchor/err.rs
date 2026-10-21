use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub struct StartOfInputError {
    location: ErrorLocation,
}

impl StartOfInputError {
    pub fn new(location: ErrorLocation) -> Box<Self> {
        Box::new(Self { location })
    }
}

impl Display for StartOfInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected start of input at byte position {}", self.location.failure_byte)
    }
}

impl Error for StartOfInputError {}

impl ParseError for StartOfInputError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }
}

#[derive(Debug)]
pub struct LineStartError {
    location: ErrorLocation,
}

impl LineStartError {
    pub fn new(location: ErrorLocation) -> Box<Self> {
        Box::new(Self { location })
    }
}

impl Display for LineStartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected start of line at byte position {}", self.location.failure_byte)
    }
}

impl Error for LineStartError {}

impl ParseError for LineStartError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }
}
