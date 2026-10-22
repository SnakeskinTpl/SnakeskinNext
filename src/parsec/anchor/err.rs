use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub struct InputStartError {
    location: ErrorLocation,
}

impl InputStartError {
    pub fn new(location: ErrorLocation) -> Box<Self> {
        Box::new(Self { location })
    }
}

impl Display for InputStartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected start of input at byte position {}", self.location.failure_byte)
    }
}

impl Error for InputStartError {}

impl ParseError for InputStartError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }
}

#[derive(Debug)]
pub struct InputEndError {
    location: ErrorLocation,
}

impl InputEndError {
    pub fn new(location: ErrorLocation) -> Box<Self> {
        Box::new(Self { location })
    }
}

impl Display for InputEndError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected end of input at byte position {}", self.location.failure_byte)
    }
}

impl Error for InputEndError {}

impl ParseError for InputEndError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }
}
