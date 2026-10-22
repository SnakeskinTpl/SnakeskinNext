use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Range;

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub(crate) struct DuplicateShorthandError {
    pub location: ErrorLocation,
    pub previous_span: Range<usize>,
}

impl Display for DuplicateShorthandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Shorthand at byte {} is already declared at bytes {}..{}",
            self.location.failure_byte, self.previous_span.start, self.previous_span.end)
    }
}

impl Error for DuplicateShorthandError {}

impl ParseError for DuplicateShorthandError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }

    fn is_recoverable(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub(crate) struct ShorthandConfigError {
    pub location: ErrorLocation,
    pub message: &'static str,
}

impl Display for ShorthandConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at byte {}", self.message, self.location.failure_byte)
    }
}

impl Error for ShorthandConfigError {}

impl ParseError for ShorthandConfigError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }

    fn is_recoverable(&self) -> bool {
        false
    }
}
