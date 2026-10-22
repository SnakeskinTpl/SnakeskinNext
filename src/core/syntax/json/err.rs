use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub(crate) struct JsonError {
    location: ErrorLocation,
    message: &'static str,
}

impl JsonError {
    pub fn new(location: ErrorLocation, message: &'static str) -> Box<Self> {
        Box::new(Self { location, message })
    }
}

impl Display for JsonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at byte {}", self.message, self.location.failure_byte)
    }
}

impl Error for JsonError {}

impl ParseError for JsonError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }

    fn is_recoverable(&self) -> bool {
        false
    }
}
