use crate::core::types::TypeError;
use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub(crate) struct DirectiveTypeError {
    pub location: ErrorLocation,
    pub cause: TypeError,
}

impl std::fmt::Display for DirectiveTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cause.fmt(f)
    }
}

impl std::error::Error for DirectiveTypeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

impl ParseError for DirectiveTypeError {
    fn location(&self) -> &ErrorLocation { &self.location }
    fn is_recoverable(&self) -> bool { false }
}
