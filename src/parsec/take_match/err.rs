use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub struct TakeMatchError {
    location: ErrorLocation,
    pub min: usize,
    pub matched: usize,
    cause: Option<Box<dyn ParseError>>,
}

impl TakeMatchError {
    pub fn new(location: ErrorLocation, min: usize, matched: usize, cause: Option<Box<dyn ParseError>>) -> Box<Self> {
        Box::new(Self { location, min, matched, cause })
    }
}

impl Display for TakeMatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "At byte position {}, expected at least {} characters passing parser checks, matched {}", self.location.failure_byte, self.min, self.matched)?;

        match &self.cause {
            Some(cause) => write!(f, ": {}", cause),
            None => write!(f, ", reached end of input"),
        }
    }
}

impl Error for TakeMatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(|cause| cause.as_ref() as &dyn Error)
    }
}

impl ParseError for TakeMatchError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }

    fn is_recoverable(&self) -> bool {
        self.cause.as_ref().is_none_or(|cause| cause.is_recoverable())
    }
}
