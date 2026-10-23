use crate::core::syntax::literal::LiteralKind;
use crate::core::syntax::type_syntax::{TypeArgumentSyntax, TypeSyntax, TypeSyntaxKind};

use super::literal::width;
use super::{Type, TypeError, TypeErrorKind};

pub(super) fn resolve(syntax: &TypeSyntax<'_>) -> Result<Type, TypeError> {
    let (name, arguments) = match &syntax.kind {
        TypeSyntaxKind::Named(name) => (*name, None),
        TypeSyntaxKind::Applied { name, arguments } => (*name, Some(arguments)),
    };

    // TODO: Resolve type definitions and argument contracts through a type environment.
    let bits = if let Some(arguments) = arguments {
        if !matches!(name, "Int" | "UInt" | "Float") {
            return Err(TypeError {
                span: syntax.span.clone(),
                kind: TypeErrorKind::Unsupported { construct: "this parameterized type" },
            });
        }

        match arguments.as_slice() {
            [TypeArgumentSyntax::Value(value)] => match &value.kind {
                LiteralKind::Number(number) if number.suffix.is_none() &&
                    number.form == crate::core::syntax::literal::NumberForm::Integer => {
                    Some(width(&number.content.replace('_', ""), value.span.clone())?)
                }

                _ => return Err(TypeError { span: value.span.clone(), kind: TypeErrorKind::InvalidWidth }),
            },

            _ => return Err(TypeError { span: syntax.span.clone(), kind: TypeErrorKind::InvalidWidth }),
        }

    } else {
        None
    };

    match name {
        "Bool" => Ok(Type::Bool),
        "Str" => Ok(Type::Str),
        "Int" => Ok(Type::Int { bits: bits.unwrap_or(32) }),
        "UInt" => Ok(Type::UInt { bits: bits.unwrap_or(32) }),
        "Float" => Ok(Type::Float { bits: bits.unwrap_or(64) }),
        _ => Err(TypeError {
            span: syntax.span.clone(),
            kind: TypeErrorKind::UnknownType { name: name.into() },
        }),
    }
}
