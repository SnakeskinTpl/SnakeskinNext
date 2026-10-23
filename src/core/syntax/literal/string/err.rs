use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub(crate) struct StringLiteralError {
    location: ErrorLocation,
    message: &'static str,
}

impl StringLiteralError {
    pub fn new(location: ErrorLocation, message: &'static str) -> Box<Self> {
        Box::new(Self { location, message })
    }
}

impl Display for StringLiteralError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at byte {}", self.message, self.location.failure_byte)
    }
}

impl Error for StringLiteralError {}

impl ParseError for StringLiteralError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }

    fn is_recoverable(&self) -> bool {
        false
    }
}
