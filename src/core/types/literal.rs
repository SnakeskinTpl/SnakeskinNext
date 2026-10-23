use std::ops::Range;

use crate::core::syntax::literal::{LiteralKind, NumberForm, NumberSuffix};

use super::{Type, TypeError, TypeErrorKind};

// TODO: Validate supported widths once the language and backend contracts are defined.
pub(super) fn width(text: &str, span: Range<usize>) -> Result<u32, TypeError> {
    text.parse::<u32>().ok().filter(|bits| *bits > 0).ok_or(TypeError {
        span,
        kind: TypeErrorKind::InvalidWidth,
    })
}

pub(super) fn literal_type(value: &LiteralKind<'_>, expected: Option<Type>, span: Range<usize>) -> Result<Type, TypeError> {
    match value {
        LiteralKind::Bool(_) => Ok(Type::Bool),
        LiteralKind::String(_) => Ok(Type::Str),
        // TODO: Infer Opt<T> from context when optional types are implemented.
        LiteralKind::Null => Err(TypeError {
            span,
            kind: TypeErrorKind::Unsupported { construct: "null" },
        }),

        LiteralKind::Number(number) => {
            // TODO: Validate representability, accounting for an enclosing unary minus.
            let ty = match &number.suffix {
                Some(NumberSuffix::Int { bits }) => Type::Int {
                    bits: bits.map(|bits| width(bits, span.clone())).transpose()?.unwrap_or(32),
                },

                Some(NumberSuffix::UInt { bits }) => Type::UInt {
                    bits: bits.map(|bits| width(bits, span.clone())).transpose()?.unwrap_or(32),
                },

                Some(NumberSuffix::Float { bits }) => Type::Float {
                    bits: bits.map(|bits| width(bits, span.clone())).transpose()?.unwrap_or(64),
                },

                None => match (number.form.clone(), expected) {
                    (NumberForm::Integer, Some(ty)) if ty.is_numeric() => ty,
                    (NumberForm::Decimal, Some(ty @ Type::Float { .. })) => ty,
                    (NumberForm::Integer, _) => Type::Int { bits: 32 },
                    (NumberForm::Decimal, _) => Type::Float { bits: 64 },
                },
            };

            Ok(ty)
        }
    }
}
