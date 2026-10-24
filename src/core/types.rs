use super::syntax::literal::LiteralKind;

mod environment;
pub(crate) use environment::{Binding, TypeEnvironment};

mod literal;
mod resolve;

mod infer;
pub(crate) use infer::infer;

mod err;
pub(crate) use err::{TypeError, TypeErrorKind};

#[cfg(test)]
mod tests;

/// A semantic type, independent of its spelling in the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Type {
    Bool,
    Str,
    Int { bits: u32 },
    UInt { bits: u32 },
    Float { bits: u32 },
}

/// Literal provenance belongs to the inferred value, not to type equality.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ValueInfo<'a> {
    pub ty: Type,
    /// Exact literal data for inlining; None does not imply a runtime-only value.
    pub literal: Option<LiteralKind<'a>>,
}

impl<'a> ValueInfo<'a> {
    fn computed(ty: Type) -> Self {
        Self { ty, literal: None }
    }
}

impl Type {
    fn is_numeric(self) -> bool {
        matches!(self, Self::Int { .. } | Self::UInt { .. } | Self::Float { .. })
    }
}
