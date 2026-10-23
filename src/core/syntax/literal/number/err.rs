use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub(crate) struct NumberLiteralError {
    location: ErrorLocation,
}

impl NumberLiteralError {
    pub fn new(location: ErrorLocation) -> Box<dyn ParseError> {
        Box::new(Self { location })
    }
}

impl Display for NumberLiteralError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fractional or exponential literal cannot have an integer suffix at byte {}", self.location.failure_byte)
    }
}

impl Error for NumberLiteralError {}

impl ParseError for NumberLiteralError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }

    fn is_recoverable(&self) -> bool {
        false
    }
}
