use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

/// A committed failure preserving the original diagnostic and source context.
#[derive(Debug)]
pub struct CutError {
    cause: Box<dyn ParseError>,
}

impl CutError {
    pub fn new(cause: Box<dyn ParseError>) -> Box<Self> {
        Box::new(Self { cause })
    }
}

impl Display for CutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.cause, f)
    }
}

impl Error for CutError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.cause.as_ref())
    }
}

impl ParseError for CutError {
    fn location(&self) -> &ErrorLocation {
        self.cause.location()
    }

    fn is_recoverable(&self) -> bool {
        false
    }
}
