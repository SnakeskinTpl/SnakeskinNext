use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub(crate) struct JadeLikeBodyError {
    location: ErrorLocation,
    pub expected: Option<char>,
}

impl JadeLikeBodyError {
    pub fn new(location: ErrorLocation, expected: Option<char>) -> Box<dyn ParseError> {
        Box::new(Self { location, expected })
    }
}

impl Display for JadeLikeBodyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid directive body at byte {}: ", self.location.failure_byte)?;

        match (self.expected, self.location.found_char) {
            (Some(expected), Some(found)) => write!(f, "expected {:?}, found {:?}", expected, found),
            (Some(expected), None) => write!(f, "expected {:?}, reached end of input", expected),
            (None, Some(found)) => write!(f, "unexpected closing delimiter {:?}", found),
            (None, None) => write!(f, "unexpected end of input"),
        }
    }
}

impl Error for JadeLikeBodyError {}

impl ParseError for JadeLikeBodyError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }
}
