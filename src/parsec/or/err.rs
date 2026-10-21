use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

/// Prioritizes non-recoverable errors, then distance, preserving ties.
pub(crate) fn select_error(start_byte: usize, first: Box<dyn ParseError>, second: Box<dyn ParseError>) -> Box<dyn ParseError> {
    if !first.is_recoverable() {
        return first;
    }
    if !second.is_recoverable() {
        return second;
    }

    match first.byte_position().cmp(&second.byte_position()) {
        std::cmp::Ordering::Greater => first,
        std::cmp::Ordering::Less => second,
        std::cmp::Ordering::Equal => AlternativeError::new(start_byte, first, second),
    }
}

/// Preserves alternative failures at the same byte position.
#[derive(Debug)]
pub struct AlternativeError {
    location: ErrorLocation,
    pub first: Box<dyn ParseError>,
    pub second: Box<dyn ParseError>,
}

impl AlternativeError {
    pub fn new(start_byte: usize, first: Box<dyn ParseError>, second: Box<dyn ParseError>) -> Box<Self> {
        assert_eq!(first.byte_position(), second.byte_position());
        let location = ErrorLocation { start_byte, ..*first.location() };
        Box::new(Self { location, first, second })
    }
}

impl Display for AlternativeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) or ({})", self.first, self.second)
    }
}

impl Error for AlternativeError {}

impl ParseError for AlternativeError {
    fn is_recoverable(&self) -> bool {
        self.first.is_recoverable() && self.second.is_recoverable()
    }

    /// Describes the whole alternative; branches retain their individual contexts.
    fn location(&self) -> &ErrorLocation {
        &self.location
    }
}
