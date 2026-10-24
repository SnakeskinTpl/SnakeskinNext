use crate::core::types::TypeError;
use crate::parsec::{ErrorLocation, ParseError, ParserInput};

#[derive(Debug)]
pub(crate) struct DirectiveTypeError {
    pub location: ErrorLocation,
    pub cause: TypeError,
}

impl DirectiveTypeError {
    /// Preserves the semantic cause and locates it within the directive body.
    pub fn new(cause: TypeError, source: &ParserInput<'_>, start_byte: usize) -> Box<Self> {
        let found_char = source.source_slice(cause.span.clone()).chars().next();

        Box::new(Self {
            location: ErrorLocation {
                start_byte,
                failure_byte: cause.span.start,
                found_char,
            },
            cause,
        })
    }
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
