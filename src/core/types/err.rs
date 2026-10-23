use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Range;

use crate::core::syntax::expression::{BinaryOperator, UnaryOperator};

use super::Type;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TypeError {
    pub span: Range<usize>,
    pub kind: TypeErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TypeErrorKind {
    UnknownIdentifier { name: String },
    UnknownType { name: String },
    Unsupported { construct: &'static str },
    InvalidWidth,
    MismatchedType { expected: Type, found: Type },
    InvalidUnary { operator: UnaryOperator, operand: Type },
    InvalidBinary { operator: BinaryOperator, left: Type, right: Type },
}

impl Display for TypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TypeErrorKind::UnknownIdentifier { name } => write!(f, "Unknown identifier {name:?}"),
            TypeErrorKind::UnknownType { name } => write!(f, "Unknown type {name:?}"),
            TypeErrorKind::Unsupported { construct } => write!(f, "Type inference for {construct} is not implemented"),
            TypeErrorKind::InvalidWidth => write!(f, "Numeric width must be a positive 32-bit integer"),
            TypeErrorKind::MismatchedType { expected, found } => write!(f, "Expected {expected:?}, found {found:?}"),
            TypeErrorKind::InvalidUnary { operator, operand } => write!(f, "Operator {operator:?} is not defined for {operand:?}"),
            TypeErrorKind::InvalidBinary { operator, left, right } => write!(f, "Operator {operator:?} is not defined for {left:?} and {right:?}"),
        }?;

        write!(f, " at bytes {}..{}", self.span.start, self.span.end)
    }
}

impl Error for TypeError {}
