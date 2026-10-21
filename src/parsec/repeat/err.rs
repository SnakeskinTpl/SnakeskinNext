use std::error::Error;
use std::fmt::Display;

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub struct RepeatError {
    location: ErrorLocation,
    kind: RepeatErrorKind,
}

/// Distinguishes insufficient matches from a non-progressing parser.
#[derive(Debug)]
pub enum RepeatErrorKind {
    InsufficientMatches {
        min: usize,
        matched: usize,
        cause: Box<dyn ParseError>,
    },

    NoProgress,
}

impl RepeatError {
    pub fn no_progress(location: ErrorLocation) -> Box<Self> {
        Box::new(Self { location, kind: RepeatErrorKind::NoProgress })
    }

    /// Preserves the failed iteration's diagnostic when the minimum is not met.
    pub fn insufficient_matches(location: ErrorLocation, min: usize, matched: usize, cause: Box<dyn ParseError>) -> Box<Self> {
        Box::new(Self { location, kind: RepeatErrorKind::InsufficientMatches { min, matched, cause } })
    }

    pub fn kind(&self) -> &RepeatErrorKind {
        &self.kind
    }
}

impl Display for RepeatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            RepeatErrorKind::InsufficientMatches { min, matched, cause } => {
                write!(f, "Expected at least {} parser repetitions, matched {}: {}", min, matched, cause)
            }

            RepeatErrorKind::NoProgress => write!(f, "Repeated parser does not advance the input position"),
        }
    }
}

impl Error for RepeatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            RepeatErrorKind::InsufficientMatches { cause, .. } => Some(cause.as_ref()),
            RepeatErrorKind::NoProgress => None,
        }
    }
}

impl ParseError for RepeatError {
    fn is_recoverable(&self) -> bool {
        match &self.kind {
            RepeatErrorKind::InsufficientMatches { cause, .. } => cause.is_recoverable(),
            RepeatErrorKind::NoProgress => false,
        }
    }

    fn location(&self) -> &ErrorLocation {
        &self.location
    }
}
