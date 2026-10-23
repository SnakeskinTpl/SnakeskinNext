use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::parsec::{ErrorLocation, ParseError};

#[derive(Debug)]
pub(crate) enum ProcessErrorKind {
    UnknownDirective { name: String },
    UnresolvedDirective,
    MissingBody { name: String },
    UnexpectedEnd,
    MismatchedEnd { expected: String, found: String },
    UnclosedBlock { name: String },
    CrossedImplicitEnd,
}

#[derive(Debug)]
pub(crate) struct ProcessError {
    location: ErrorLocation,
    pub kind: ProcessErrorKind,
}

impl ProcessError {
    pub fn new(location: ErrorLocation, kind: ProcessErrorKind) -> Box<Self> {
        Box::new(Self { location, kind })
    }
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ProcessErrorKind::UnknownDirective { name } => write!(f, "Unknown directive {name:?}"),
            ProcessErrorKind::UnresolvedDirective => write!(f, "Cannot resolve directive without a name"),
            ProcessErrorKind::MissingBody { name } => write!(f, "Directive {name:?} requires a body"),
            ProcessErrorKind::UnexpectedEnd => write!(f, "No open block to close"),
            ProcessErrorKind::MismatchedEnd { expected, found } => write!(f, "Expected end of {expected:?}, found {found:?}"),
            ProcessErrorKind::UnclosedBlock { name } => write!(f, "Unclosed block {name:?}"),
            ProcessErrorKind::CrossedImplicitEnd => write!(f, "Implicit end crosses an unclosed block"),
        }?;

        write!(f, " at byte {}", self.location.failure_byte)
    }
}

impl Error for ProcessError {}

impl ParseError for ProcessError {
    fn location(&self) -> &ErrorLocation {
        &self.location
    }

    fn is_recoverable(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub(crate) struct RegistrationError {
    pub name: String,
    pub reason: &'static str,
}

impl Display for RegistrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot register {:?}: {}", self.name, self.reason)
    }
}

impl Error for RegistrationError {}
